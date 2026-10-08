//! Deterministic JSON-RPC 2.0 operation registry over the canonical service.
use crate::operation::authorize;
use crate::{
    AcornService, CheckReadabilityRequest, InvocationContext, OperationEffects, ProcessRepositoryRequest, ServiceError, ServiceErrorKind,
    ServiceResult, ValidateResearchActivityRequest, ValidateStandardRequest,
};
use acorn_core::prelude::{format, Box, String, ToString, Vec};
use alloc::sync::Arc;
use core::fmt;
use core::future::Future;
use core::pin::Pin;
use futures::{stream, StreamExt};
use schemars::{schema_for, JsonSchema};
use serde::de::{DeserializeOwned, Error as DeError, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// JSON-RPC internal error code.
pub const INTERNAL_ERROR: i32 = -32603;
/// JSON-RPC invalid parameters error code.
pub const INVALID_PARAMS: i32 = -32602;
/// JSON-RPC invalid request error code.
pub const INVALID_REQUEST: i32 = -32600;
/// Supported JSON-RPC protocol version.
pub const JSON_RPC_VERSION: &str = "2.0";
/// JSON-RPC method-not-found error code.
pub const METHOD_NOT_FOUND: i32 = -32601;
/// JSON-RPC parse error code.
pub const PARSE_ERROR: i32 = -32700;
/// ACORN authorization-policy error code.
pub const POLICY_DENIED: i32 = -32001;
type OperationFuture = Pin<Box<dyn Future<Output = Result<Value, RpcError>> + Send>>;
type OperationHandler = Arc<dyn Fn(Value, InvocationContext) -> OperationFuture + Send + Sync>;
/// Persistence port for JSON-RPC operation status and replay.
pub trait DurableOperations: Send + Sync {
    /// Return the public state for an operation key.
    fn state(&self, operation_key: &str) -> ServiceResult<Option<String>>;
    /// Queue replay for an operation key.
    fn replay(&self, operation_key: &str) -> ServiceResult<bool>;
}
enum ParsedCall {
    Invalid { error: RpcError, id: Value },
    Notification { method: MethodName, params: Value },
    Request { id: RequestId, method: MethodName, params: Value },
}
/// String or integer JSON-RPC request identifier.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum RequestId {
    /// Signed integer identifier.
    Number(i64),
    /// String identifier.
    String(String),
}
/// Result of dispatching one JSON-RPC document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dispatch {
    /// Serialized response, absent for notification-only input.
    pub body: Option<Vec<u8>>,
    /// Recommended HTTP status.
    pub status_code: u16,
}
#[derive(Serialize)]
struct ErrorResponse {
    error: RpcError,
    id: Value,
    jsonrpc: &'static str,
}
/// Validated method name in the `acorn.` namespace.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct MethodName(String);
/// JSON-RPC notification used by internal event adapters.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Notification {
    /// Protocol version, always `2.0`.
    pub jsonrpc: String,
    /// Registered operation name.
    pub method: MethodName,
    /// Operation parameters.
    pub params: Value,
}
/// Registered operation metadata.
#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationDefinition {
    /// Declared side effects.
    pub effects: OperationEffects,
    /// JSON-RPC method name.
    pub name: MethodName,
    /// JSON Schema for operation parameters.
    pub parameters_schema: Value,
    /// JSON Schema for the operation result.
    pub result_schema: Value,
}
/// Immutable registry of explicitly registered operations.
#[derive(Clone, Default)]
pub struct OperationRegistry {
    operations: Vec<RegisteredOperation>,
}
#[derive(Clone)]
struct RegisteredOperation {
    definition: OperationDefinition,
    handler: OperationHandler,
}
/// Secret-safe JSON-RPC error.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RpcError {
    /// Stable JSON-RPC or ACORN error code.
    pub code: i32,
    /// Optional bounded public diagnostic data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    /// Stable public message.
    pub message: String,
}
struct StrictValue(Value);
#[derive(Serialize)]
struct SuccessResponse {
    id: RequestId,
    jsonrpc: &'static str,
    result: Value,
}
impl Dispatch {
    fn serialize(value: Value) -> Self {
        serde_json::to_vec(&value).map_or_else(
            |_| RpcError::internal().dispatch(Value::Null),
            |body| Self {
                body: Some(body),
                status_code: 200,
            },
        )
    }
}
impl fmt::Display for MethodName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
impl MethodName {
    /// Construct and validate one namespaced method name.
    pub fn new(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        let suffix = value.strip_prefix("acorn.").unwrap_or_default();
        let valid = !suffix.is_empty()
            && suffix.split('.').all(|part| {
                !part.is_empty()
                    && part
                        .chars()
                        .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-')
            });
        valid
            .then_some(Self(value.clone()))
            .ok_or_else(|| ServiceError::new(ServiceErrorKind::InvalidInput, format!("Invalid ACORN operation name '{value}'")))
    }
}
impl TryFrom<&str> for MethodName {
    type Error = ServiceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
impl TryFrom<String> for MethodName {
    type Error = ServiceError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
impl<'a, const N: usize> From<[&'a str; N]> for MethodName {
    fn from(parts: [&'a str; N]) -> Self {
        Self(format!("acorn.{}", parts.join(".")))
    }
}
impl Notification {
    /// Construct a notification for one registered ACORN operation.
    pub fn new(method: MethodName, params: Value) -> ServiceResult<Self> {
        MethodName::new(method.to_string()).map(|method| Self {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            method,
            params,
        })
    }
}
impl OperationDefinition {
    /// Create typed operation metadata and parameter/result schemas.
    pub fn new<I: JsonSchema, O: JsonSchema>(name: MethodName, effects: OperationEffects) -> ServiceResult<Self> {
        MethodName::new(name.to_string()).map(|name| Self {
            effects,
            name,
            parameters_schema: serde_json::to_value(schema_for!(I)).unwrap_or(Value::Null),
            result_schema: serde_json::to_value(schema_for!(O)).unwrap_or(Value::Null),
        })
    }
}
impl OperationRegistry {
    /// Build the portable ACORN catalog, optionally backing durable operations.
    pub fn acorn(service: AcornService, version: impl Into<String>, durable: Option<Arc<dyn DurableOperations>>) -> ServiceResult<Self> {
        #[derive(Deserialize, JsonSchema)]
        #[serde(deny_unknown_fields)]
        struct Empty {}
        #[derive(JsonSchema, Serialize)]
        struct VersionOutput {
            version: String,
        }
        #[derive(Deserialize, JsonSchema)]
        #[serde(deny_unknown_fields, rename_all = "camelCase")]
        struct OperationInput {
            operation_key: String,
        }
        #[derive(Deserialize, JsonSchema)]
        #[serde(deny_unknown_fields)]
        struct RepositoryWorkflowInput {
            input: ProcessRepositoryRequest,
            workflow: String,
        }
        #[derive(JsonSchema, Serialize)]
        struct OperationStatusOutput {
            state: Option<String>,
        }
        #[derive(JsonSchema, Serialize)]
        struct ReplayOutput {
            queued: bool,
        }
        let version = version.into();
        let status = durable.clone();
        Self::default()
            .register("acorn.version", OperationEffects::default(), move |_: Empty, _| {
                let version = version.clone();
                async move { Ok(VersionOutput { version }) }
            })
            .and_then(|registry| {
                registry.register(
                    "acorn.research-activity.validate",
                    OperationEffects::default(),
                    move |request: ValidateResearchActivityRequest, _| async move { service.validate_research_activity(request) },
                )
            })
            .and_then(|registry| {
                registry.register(
                    "acorn.standard.validate",
                    OperationEffects::default(),
                    move |request: ValidateStandardRequest, _| async move { service.validate_standard(request) },
                )
            })
            .and_then(|registry| {
                registry.register(
                    "acorn.readability.check",
                    OperationEffects::default(),
                    move |request: CheckReadabilityRequest, _| async move { service.check_readability(request) },
                )
            })
            .and_then(|registry| {
                registry.register(
                    "acorn.workflows.process",
                    OperationEffects::default(),
                    move |request: RepositoryWorkflowInput, _| async move {
                        match request.workflow.as_str() {
                            | "repository-quality" => service.process_repository(request.input),
                            | workflow => Err(ServiceError::new(
                                ServiceErrorKind::InvalidInput,
                                format!("Unknown repository workflow '{workflow}'"),
                            )),
                        }
                    },
                )
            })
            .and_then(|registry| {
                registry.register(
                    "acorn.operations.status",
                    OperationEffects::default(),
                    move |request: OperationInput, _| {
                        let durable = status.clone();
                        async move {
                            durable
                                .ok_or_else(durable_unavailable)
                                .and_then(|durable| durable.state(&request.operation_key))
                                .map(|state| OperationStatusOutput { state })
                        }
                    },
                )
            })
            .and_then(|registry| {
                registry.register(
                    "acorn.operations.replay",
                    OperationEffects {
                        mutation: true,
                        ..OperationEffects::default()
                    },
                    move |request: OperationInput, _| {
                        let durable = durable.clone();
                        async move {
                            durable
                                .ok_or_else(durable_unavailable)
                                .and_then(|durable| durable.replay(&request.operation_key))
                                .map(|queued| ReplayOutput { queued })
                        }
                    },
                )
            })
    }
    /// Register one typed handler, rejecting invalid or duplicate method names.
    pub fn register<I, O, F, Fut>(self, name: impl Into<String>, effects: OperationEffects, handler: F) -> ServiceResult<Self>
    where
        I: DeserializeOwned + JsonSchema + Send + 'static,
        O: JsonSchema + Serialize + Send + 'static,
        F: Fn(I, InvocationContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ServiceResult<O>> + Send + 'static,
    {
        MethodName::new(name)
            .and_then(|name| OperationDefinition::new::<I, O>(name, effects))
            .and_then(|definition| self.register_definition(definition, handler))
    }
    /// Register a typed handler with precomputed operation metadata.
    pub fn register_definition<I, O, F, Fut>(self, definition: OperationDefinition, handler: F) -> ServiceResult<Self>
    where
        I: DeserializeOwned + Send + 'static,
        O: Serialize + Send + 'static,
        F: Fn(I, InvocationContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ServiceResult<O>> + Send + 'static,
    {
        MethodName::new(definition.name.to_string()).and_then(|name| {
            match self.operations.iter().any(|operation| operation.definition.name == name) {
                | true => Err(ServiceError::new(
                    ServiceErrorKind::InvalidInput,
                    format!("Duplicate operation name '{name}'"),
                )),
                | false => {
                    let handler = Arc::new(handler);
                    let operation_name = name.clone();
                    let typed = move |params: Value, context: InvocationContext| -> OperationFuture {
                        let input = serde_json::from_value::<I>(params).map_err(|_| RpcError::new(INVALID_PARAMS, "Invalid method parameters"));
                        let handler = Arc::clone(&handler);
                        let operation_name = operation_name.clone();
                        Box::pin(async move {
                            match input {
                                | Ok(input) => handler(input, context)
                                    .await
                                    .map_err(|error| RpcError::from_service(error.with_operation(operation_name.to_string())))
                                    .and_then(|output| serde_json::to_value(output).map_err(|_| RpcError::internal())),
                                | Err(error) => Err(error),
                            }
                        })
                    };
                    let mut operations = self.operations;
                    operations.push(RegisteredOperation {
                        definition,
                        handler: Arc::new(typed),
                    });
                    operations.sort_by(|left, right| left.definition.name.cmp(&right.definition.name));
                    Ok(Self { operations })
                }
            }
        })
    }
    /// Dispatch a single call, ordered batch, or notification document.
    pub async fn dispatch(&self, body: &[u8], context: InvocationContext) -> Dispatch {
        let exceeds_limit = u64::try_from(body.len()).unwrap_or(u64::MAX) > context.max_input.checked_bytes().unwrap_or_default().max(1);
        match exceeds_limit {
            | true => RpcError::new(-32002, "Request exceeded the configured input limit").dispatch(Value::Null),
            | false => match serde_json::from_slice::<StrictValue>(body).map(|StrictValue(value)| value) {
                | Ok(Value::Array(values)) if values.is_empty() => RpcError::new(INVALID_REQUEST, "Invalid Request").dispatch(Value::Null),
                | Ok(Value::Array(values)) => {
                    let responses = stream::iter(values)
                        .then(|value| self.dispatch_call(value, context.clone()))
                        .filter_map(core::future::ready)
                        .collect::<Vec<_>>()
                        .await;
                    match responses.is_empty() {
                        | true => Dispatch {
                            body: None,
                            status_code: 204,
                        },
                        | false => Dispatch::serialize(Value::Array(responses)),
                    }
                }
                | Ok(value) => self.dispatch_call(value, context).await.map_or(
                    Dispatch {
                        body: None,
                        status_code: 204,
                    },
                    Dispatch::serialize,
                ),
                | Err(_) => RpcError::new(PARSE_ERROR, "Parse error").dispatch(Value::Null),
            },
        }
    }
    /// Invoke one registered operation after effect and output-bound checks.
    pub async fn invoke(&self, method: &MethodName, params: Value, context: InvocationContext) -> Result<Value, RpcError> {
        match self.operations.iter().find(|operation| operation.definition.name == *method) {
            | None => Err(RpcError::new(METHOD_NOT_FOUND, "Method not found")),
            | Some(operation) => match authorize(&operation.definition.effects, &context)
                .map_err(|error| ServiceError::new(ServiceErrorKind::PolicyDenied, error.to_string()))
                .map_err(RpcError::from_service)
            {
                | Err(error) => Err(error),
                | Ok(()) => match (operation.handler)(params, context.clone()).await {
                    | Err(error) => Err(error),
                    | Ok(result) => serde_json::to_vec(&result).map_err(|_| RpcError::internal()).and_then(|encoded| {
                        let exceeds_limit =
                            u64::try_from(encoded.len()).unwrap_or(u64::MAX) > context.max_output.checked_bytes().unwrap_or_default().max(1);
                        match exceeds_limit {
                            | true => Err(RpcError::new(INTERNAL_ERROR, "Operation result exceeded the configured output limit")),
                            | false => Ok(result),
                        }
                    }),
                },
            },
        }
    }
    /// Return registered method names in deterministic order.
    pub fn names(&self) -> Vec<MethodName> {
        self.operations.iter().map(|operation| operation.definition.name.clone()).collect()
    }
    /// Return registered operation definitions in deterministic order.
    pub fn definitions(&self) -> Vec<OperationDefinition> {
        self.operations.iter().map(|operation| operation.definition.clone()).collect()
    }
    async fn dispatch_call(&self, value: Value, context: InvocationContext) -> Option<Value> {
        match ParsedCall::from(value) {
            | ParsedCall::Invalid { error, id } => Some(error.value(id)),
            | ParsedCall::Notification { method, params } => {
                let _ = self.invoke(&method, params, context).await;
                None
            }
            | ParsedCall::Request { id, method, params } => {
                let correlation_id = match &id {
                    | RequestId::Number(value) => value.to_string(),
                    | RequestId::String(value) => value.clone(),
                };
                let context = InvocationContext {
                    correlation_id: Some(correlation_id),
                    ..context
                };
                Some(match self.invoke(&method, params, context).await {
                    | Ok(result) => serde_json::to_value(SuccessResponse {
                        id,
                        jsonrpc: JSON_RPC_VERSION,
                        result,
                    })
                    .unwrap_or_else(|_| RpcError::internal().value(Value::Null)),
                    | Err(error) => error.value(serde_json::to_value(id).unwrap_or(Value::Null)),
                })
            }
        }
    }
}
impl ParsedCall {
    fn from(value: Value) -> Self {
        match value {
            | Value::Object(mut object) => Self::from_object(&mut object),
            | _ => Self::Invalid {
                error: RpcError::new(INVALID_REQUEST, "Invalid Request"),
                id: Value::Null,
            },
        }
    }
    fn from_object(object: &mut Map<String, Value>) -> Self {
        let id = object.remove("id");
        let version = object.remove("jsonrpc");
        let method = object
            .remove("method")
            .and_then(|value| value.as_str().map(ToString::to_string))
            .and_then(|value| MethodName::new(value).ok());
        let params = object.remove("params").unwrap_or_else(|| Value::Object(Map::new()));
        let valid = object.is_empty()
            && version == Some(Value::String(JSON_RPC_VERSION.to_string()))
            && method.is_some()
            && matches!(params, Value::Object(_) | Value::Array(_));
        match (valid, method, id) {
            | (true, Some(method), None) => Self::Notification { method, params },
            | (true, Some(method), Some(id)) => serde_json::from_value(id).map_or_else(
                |_| Self::Invalid {
                    error: RpcError::new(INVALID_REQUEST, "Invalid Request"),
                    id: Value::Null,
                },
                |id| Self::Request { id, method, params },
            ),
            | (_, _, id) => Self::Invalid {
                error: RpcError::new(INVALID_REQUEST, "Invalid Request"),
                id: id.filter(|value| value.is_string() || value.is_number()).unwrap_or(Value::Null),
            },
        }
    }
}
impl core::error::Error for RpcError {}
impl fmt::Display for RpcError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({})", self.message, self.code)
    }
}
impl RpcError {
    /// Construct a public JSON-RPC error.
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            data: None,
            message: message.into(),
        }
    }
    fn dispatch(self, id: Value) -> Dispatch {
        Dispatch::serialize(self.value(id))
    }
    fn from_service(error: ServiceError) -> Self {
        let code = match error.kind {
            | ServiceErrorKind::InvalidInput | ServiceErrorKind::Unsupported => INVALID_PARAMS,
            | ServiceErrorKind::PolicyDenied => POLICY_DENIED,
            | ServiceErrorKind::PayloadBounds => -32002,
            | ServiceErrorKind::DurableOperationsUnavailable => -32003,
            | ServiceErrorKind::Internal => INTERNAL_ERROR,
        };
        let data = serde_json::to_value(&error).ok();
        Self {
            code,
            data,
            message: error.message,
        }
    }
    fn internal() -> Self {
        Self::new(INTERNAL_ERROR, "Internal error")
    }
    fn value(self, id: Value) -> Value {
        serde_json::to_value(ErrorResponse {
            error: self,
            id,
            jsonrpc: JSON_RPC_VERSION,
        })
        .unwrap_or(Value::Null)
    }
}
impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StrictVisitor;
        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictValue;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON value without duplicate object fields")
            }
            fn visit_bool<E: DeError>(self, value: bool) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Bool(value)))
            }
            fn visit_f64<E: DeError>(self, value: f64) -> Result<Self::Value, E> {
                serde_json::Number::from_f64(value)
                    .map(Value::Number)
                    .map(StrictValue)
                    .ok_or_else(|| E::custom("invalid JSON number"))
            }
            fn visit_i64<E: DeError>(self, value: i64) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Number(value.into())))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut values: A) -> Result<Self::Value, A::Error> {
                let mut object = Map::new();
                loop {
                    match values.next_entry::<String, StrictValue>() {
                        | Err(error) => break Err(error),
                        | Ok(None) => break Ok(StrictValue(Value::Object(object))),
                        | Ok(Some((key, _))) if object.contains_key(&key) => {
                            break Err(A::Error::custom(format!("duplicate field '{key}'")));
                        }
                        | Ok(Some((key, StrictValue(value)))) => {
                            object.insert(key, value);
                        }
                    }
                }
            }
            fn visit_none<E: DeError>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut values: A) -> Result<Self::Value, A::Error> {
                let mut array = Vec::new();
                loop {
                    match values.next_element::<StrictValue>() {
                        | Err(error) => break Err(error),
                        | Ok(None) => break Ok(StrictValue(Value::Array(array))),
                        | Ok(Some(StrictValue(value))) => array.push(value),
                    }
                }
            }
            fn visit_str<E: DeError>(self, value: &str) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::String(value.to_string())))
            }
            fn visit_string<E: DeError>(self, value: String) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::String(value)))
            }
            fn visit_u64<E: DeError>(self, value: u64) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Number(value.into())))
            }
            fn visit_unit<E: DeError>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
        }
        deserializer.deserialize_any(StrictVisitor)
    }
}
impl From<MethodName> for String {
    fn from(value: MethodName) -> Self {
        value.0
    }
}
fn durable_unavailable() -> ServiceError {
    ServiceError::new(
        ServiceErrorKind::DurableOperationsUnavailable,
        "Durable operation state is unavailable in this server composition",
    )
}
