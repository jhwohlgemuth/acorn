//! Typed operation registry and JSON-RPC 2.0 transport
use crate::agent::{run_inference, InferenceBackend, InferenceReadiness, InferenceRequest, InferenceResult};
use crate::analyzer::fix::{CheckReferences, FixReport, FixSuggestionRequest};
use crate::io::api::operation::authorize;
use crate::io::api::webhooks::store::OperationQueue;
use crate::io::workflow::{RepositoryChangeSet, RepositoryInput};
use crate::io::ApiResult;
use crate::util::constants::app::{APPLICATION, JSON_RPC_VERSION};
use acorn_core::prelude::{Box, String, Vec};
use acorn_schema::validation::Validate;
use acorn_service::AcornService;
use acorn_service::{DurableOperations, ServiceError, ServiceErrorKind, ServiceResult};
use alloc::sync::Arc;
use color_eyre::eyre::eyre;
use core::fmt;
use core::future::Future;
use core::pin::Pin;
use futures::StreamExt;
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
/// JSON-RPC method-not-found error code.
pub const METHOD_NOT_FOUND: i32 = -32601;
/// JSON-RPC parse error code.
pub const PARSE_ERROR: i32 = -32700;
/// ACORN authorization-policy error code.
pub const POLICY_DENIED: i32 = -32001;
pub use super::operation::{InvocationContext, InvocationOrigin, OperationEffect, OperationEffects, Principal};
type OperationFuture = Pin<Box<dyn Future<Output = Result<Value, RpcError>> + Send>>;
type OperationHandler = Arc<dyn Fn(Value, InvocationContext) -> OperationFuture + Send + Sync>;
enum ParsedCall {
    Invalid { id: Value, error: RpcError },
    Notification { method: MethodName, params: Value },
    Request { id: RequestId, method: MethodName, params: Value },
}
/// String or integer JSON-RPC request identifier.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum RequestId {
    /// Signed integer request identifier.
    Number(i64),
    /// String request identifier.
    String(String),
}
/// Result of dispatching one JSON-RPC document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dispatch {
    /// Serialized JSON-RPC response, absent for notifications.
    pub body: Option<Vec<u8>>,
    /// Recommended HTTP status.
    pub status_code: u16,
}
#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct ErrorResponse {
    error: RpcError,
    id: Value,
    jsonrpc: &'static str,
}
/// Validated JSON-RPC method name in the ACORN namespace.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize, Validate)]
#[serde(try_from = "String", into = "String")]
pub struct MethodName {
    /// Ordered method components following the application namespace.
    #[validate(length(min = 1), each(nonempty))]
    pub parts: Vec<String>,
}
/// JSON-RPC 2.0 notification used by internal event adapters.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Notification {
    /// Protocol version, always [`JSON_RPC_VERSION`].
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
/// Immutable registry of explicitly registered ACORN operations.
#[derive(Clone, Default)]
pub struct OperationRegistry {
    operations: Vec<RegisteredOperation>,
}
#[derive(Clone)]
struct RegisteredOperation {
    definition: OperationDefinition,
    handler: OperationHandler,
}
/// Secret-safe JSON-RPC or operation error.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RpcError {
    /// Stable JSON-RPC or ACORN error code.
    pub code: i32,
    /// Optional bounded public diagnostic data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    /// Stable public error message.
    pub message: String,
}
struct StrictValue(Value);
#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct SuccessResponse {
    id: RequestId,
    jsonrpc: &'static str,
    result: Value,
}
impl Dispatch {
    fn serialize(value: Value) -> Self {
        match serde_json::to_vec(&value) {
            | Ok(body) => Self {
                body: Some(body),
                status_code: 200,
            },
            | Err(_) => Self {
                body: Some(
                    format!(r#"{{"jsonrpc":"{JSON_RPC_VERSION}","id":null,"error":{{"code":-32603,"message":"Internal error"}}}}"#).into_bytes(),
                ),
                status_code: 500,
            },
        }
    }
}
impl fmt::Display for MethodName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{APPLICATION}.{}", self.parts.join("."))
    }
}
impl<'a, const N: usize> From<[&'a str; N]> for MethodName {
    fn from(parts: [&'a str; N]) -> Self {
        Self {
            parts: parts.into_iter().map(str::to_string).collect(),
        }
    }
}
impl TryFrom<&str> for MethodName {
    type Error = color_eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}
impl TryFrom<String> for MethodName {
    type Error = color_eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let prefix = format!("{APPLICATION}.");
        match value.strip_prefix(&prefix) {
            | Some(value) => {
                let name = Self {
                    parts: value.split('.').map(str::to_string).collect(),
                };
                name.validate()
                    .map(|()| name)
                    .map_err(|why| eyre!("Invalid JSON-RPC method name — {why}"))
            }
            | None => Err(eyre!("Operation name '{value}' must use the non-reserved '{APPLICATION}.' namespace")),
        }
    }
}
impl Notification {
    /// Construct a notification for one registered ACORN operation.
    pub fn new(method: MethodName, params: Value) -> ApiResult<Self> {
        method
            .validate()
            .map_err(|why| eyre!("Invalid JSON-RPC method name — {why}"))
            .map(|()| Self {
                jsonrpc: JSON_RPC_VERSION.to_string(),
                method,
                params,
            })
    }
}
impl OperationDefinition {
    /// Create typed operation metadata and its parameter/result schemas.
    pub fn new<I: JsonSchema, O: JsonSchema>(name: MethodName, effects: OperationEffects) -> ApiResult<Self> {
        name.validate()
            .map_err(|why| eyre!("Invalid JSON-RPC method name — {why}"))
            .map(|()| Self {
                effects,
                name,
                parameters_schema: serde_json::to_value(schema_for!(I)).unwrap_or(Value::Null),
                result_schema: serde_json::to_value(schema_for!(O)).unwrap_or(Value::Null),
            })
    }
}
impl DurableOperations for OperationQueue {
    fn state(&self, operation_key: &str) -> ServiceResult<Option<String>> {
        OperationQueue::state(self, operation_key)
            .map(|state| state.map(|state| state.to_string()))
            .map_err(|_| ServiceError::new(ServiceErrorKind::Internal, "Failed to read durable operation state"))
    }
    fn replay(&self, operation_key: &str) -> ServiceResult<bool> {
        OperationQueue::replay(self, operation_key)
            .map_err(|_| ServiceError::new(ServiceErrorKind::Internal, "Failed to queue durable operation replay"))
    }
}
impl fmt::Debug for OperationRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let definitions: Vec<OperationDefinition> = self.operations.iter().map(|operation| operation.definition.clone()).collect();
        formatter.debug_struct("OperationRegistry").field("definitions", &definitions).finish()
    }
}
impl OperationRegistry {
    /// Build the baseline ACORN operation registry.
    pub fn acorn() -> ApiResult<Self> {
        #[derive(Deserialize, JsonSchema)]
        #[serde(deny_unknown_fields)]
        struct OperationInput {
            operation_key: String,
        }
        #[derive(JsonSchema, Serialize)]
        #[serde(deny_unknown_fields)]
        struct OperationStatusOutput {
            state: Option<String>,
        }
        #[derive(JsonSchema, Serialize)]
        #[serde(deny_unknown_fields)]
        struct ReplayOutput {
            queued: bool,
        }
        #[derive(Deserialize, JsonSchema)]
        #[serde(deny_unknown_fields)]
        struct ProcessInput {
            input: RepositoryInput,
            workflow: String,
        }
        #[derive(Deserialize, JsonSchema)]
        #[serde(deny_unknown_fields)]
        struct VersionInput {}
        #[derive(JsonSchema, Serialize)]
        #[serde(deny_unknown_fields)]
        struct VersionOutput {
            version: String,
        }
        let registry = OperationDefinition::new::<VersionInput, VersionOutput>(MethodName::from(["version"]), OperationEffects::default())
            .and_then(|definition| {
                Self::default().register(definition, |_: VersionInput, _| async {
                    Ok(VersionOutput {
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    })
                })
            })
            .and_then(|registry| {
                OperationDefinition::new::<ProcessInput, RepositoryChangeSet>(MethodName::from(["workflows", "process"]), OperationEffects::default())
                    .and_then(|definition| {
                        registry.register(definition, |request: ProcessInput, _| async move {
                            match request.workflow.as_str() {
                                | "repository-quality" => serde_json::to_value(request.input)
                                    .map_err(|why| eyre!("Encode repository workflow request — {why}"))
                                    .and_then(|value| {
                                        serde_json::from_value(value).map_err(|why| eyre!("Decode portable repository workflow request — {why}"))
                                    })
                                    .and_then(|input| {
                                        AcornService
                                            .process_repository(input)
                                            .map_err(|why| eyre!("Repository workflow failed — {why}"))
                                    })
                                    .and_then(|output| {
                                        serde_json::to_value(output).map_err(|why| eyre!("Encode portable repository workflow response — {why}"))
                                    })
                                    .and_then(|value| {
                                        serde_json::from_value::<RepositoryChangeSet>(value)
                                            .map_err(|why| eyre!("Decode repository workflow response — {why}"))
                                    }),
                                | name => Err(eyre!("Unknown repository workflow '{name}'")),
                            }
                        })
                    })
            });
        registry
            .and_then(|registry| {
                OperationDefinition::new::<OperationInput, OperationStatusOutput>(
                    MethodName::from(["operations", "status"]),
                    OperationEffects::default(),
                )
                .and_then(|status| {
                    registry.register(status, |request: OperationInput, _| async move {
                        OperationQueue::configured()
                            .state(&request.operation_key)
                            .map(|state| OperationStatusOutput {
                                state: state.map(|state| state.to_string()),
                            })
                    })
                })
            })
            .and_then(|registry| {
                OperationDefinition::new::<OperationInput, ReplayOutput>(
                    MethodName::from(["operations", "replay"]),
                    OperationEffects {
                        filesystem_read: true,
                        filesystem_write: true,
                        mutation: true,
                        ..OperationEffects::default()
                    },
                )
                .and_then(|replay| {
                    registry.register(replay, |request: OperationInput, _| async move {
                        OperationQueue::configured()
                            .replay(&request.operation_key)
                            .map(|queued| ReplayOutput { queued })
                    })
                })
            })
    }
    /// Dispatch one JSON-RPC 2.0 document through this operation registry.
    pub async fn dispatch(&self, body: &[u8], context: InvocationContext) -> Dispatch {
        match serde_json::from_slice::<StrictValue>(body).map(|StrictValue(value)| value) {
            | Ok(Value::Array(values)) if values.is_empty() => RpcError::new(INVALID_REQUEST, "Invalid Request").dispatch(Value::Null),
            | Ok(Value::Array(values)) => {
                let responses = self.dispatch_calls(values, context).await;
                match responses.is_empty() {
                    | true => Dispatch {
                        body: None,
                        status_code: 204,
                    },
                    | false => Dispatch::serialize(Value::Array(responses)),
                }
            }
            | Ok(value) => match self.dispatch_call(value, context).await {
                | Some(response) => Dispatch::serialize(response),
                | None => Dispatch {
                    body: None,
                    status_code: 204,
                },
            },
            | Err(_) => RpcError::new(PARSE_ERROR, "Parse error").dispatch(Value::Null),
        }
    }
    async fn dispatch_call(&self, value: Value, context: InvocationContext) -> Option<Value> {
        match ParsedCall::from(value) {
            | ParsedCall::Invalid { id, error } => Some(error.value(id)),
            | ParsedCall::Notification { method, params } => {
                self.invoke(&method, params, context).await.ok();
                None
            }
            | ParsedCall::Request { id, method, params } => {
                let context = InvocationContext {
                    correlation_id: Some(id.as_string()),
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
    async fn dispatch_calls(&self, values: Vec<Value>, context: InvocationContext) -> Vec<Value> {
        futures::stream::iter(values)
            .then(|value| self.dispatch_call(value, context.clone()))
            .filter_map(core::future::ready)
            .collect()
            .await
    }
    /// Invoke a registered operation after enforcing declared effects and output bounds.
    pub async fn invoke(&self, method: &MethodName, params: Value, context: InvocationContext) -> Result<Value, RpcError> {
        match self.operations.iter().find(|operation| operation.definition.name == *method) {
            | Some(operation) => match authorize(&operation.definition.effects, &context) {
                | Err(why) => Err(RpcError::new(POLICY_DENIED, why.to_string())),
                | Ok(()) => (operation.handler)(params, context.clone()).await.and_then(|result| {
                    serde_json::to_vec(&result).map_err(|_| RpcError::internal()).and_then(|encoded| {
                        match encoded.len() <= context.max_output_bytes.max(1) {
                            | true => Ok(result),
                            | false => Err(RpcError::new(INTERNAL_ERROR, "Operation result exceeded the configured output limit")),
                        }
                    })
                }),
            },
            | None => Err(RpcError::new(METHOD_NOT_FOUND, "Method not found")),
        }
    }
    /// Register one typed handler, rejecting invalid or duplicate method names.
    pub fn register<I, O, F, Fut>(self, definition: OperationDefinition, handler: F) -> ApiResult<Self>
    where
        I: DeserializeOwned + Send + 'static,
        O: Serialize + Send + 'static,
        F: Fn(I, InvocationContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ApiResult<O>> + Send + 'static,
    {
        match self.operations.iter().any(|operation| operation.definition.name == definition.name) {
            | true => Err(eyre!("Duplicate operation name '{}'", definition.name)),
            | false => {
                let name = definition.name.clone();
                let handler = Arc::new(handler);
                let typed_handler = move |params: Value, context: InvocationContext| -> OperationFuture {
                    let input = serde_json::from_value::<I>(params).map_err(|_| RpcError::new(INVALID_PARAMS, "Invalid method parameters"));
                    let handler = Arc::clone(&handler);
                    let name = name.clone();
                    Box::pin(async move {
                        match input {
                            | Ok(input) => handler(input, context)
                                .await
                                .map_err(|_| RpcError::new(INTERNAL_ERROR, format!("Operation '{name}' failed")))
                                .and_then(|output| serde_json::to_value(output).map_err(|_| RpcError::internal())),
                            | Err(error) => Err(error),
                        }
                    })
                };
                let mut operations = self.operations;
                operations.push(RegisteredOperation {
                    definition,
                    handler: Arc::new(typed_handler),
                });
                operations.sort_by(|left, right| left.definition.name.cmp(&right.definition.name));
                Ok(Self { operations })
            }
        }
    }
    /// Register the inference run, readiness, and fix-suggestion operations with one backend.
    pub fn register_inference<B>(self, backend: B) -> ApiResult<Self>
    where
        B: InferenceBackend + 'static,
    {
        let backend = Arc::new(backend);
        OperationDefinition::new::<InferenceRequest, InferenceResult>(
            MethodName::from(["inference", "run"]),
            OperationEffects {
                inference: true,
                ..OperationEffects::default()
            },
        )
        .and_then(|definition| {
            let backend = Arc::clone(&backend);
            self.register(definition, move |request: InferenceRequest, _| {
                let backend = Arc::clone(&backend);
                async move { run_inference(backend.as_ref(), &request).await }
            })
        })
        .and_then(|registry| {
            OperationDefinition::new::<InferenceRequest, InferenceReadiness>(
                MethodName::from(["inference", "readiness"]),
                OperationEffects {
                    inference: true,
                    ..OperationEffects::default()
                },
            )
            .and_then(|definition| {
                let backend = Arc::clone(&backend);
                registry.register(definition, move |request: InferenceRequest, _| {
                    let backend = Arc::clone(&backend);
                    async move { InferenceReadiness::run(backend.as_ref(), &request).await }
                })
            })
        })
        .and_then(|registry| {
            OperationDefinition::new::<FixSuggestionRequest, FixReport>(
                MethodName::from(["fixes", "suggest"]),
                OperationEffects {
                    inference: true,
                    ..OperationEffects::default()
                },
            )
            .and_then(|definition| {
                let backend = Arc::clone(&backend);
                registry.register(definition, move |request: FixSuggestionRequest, _| {
                    let backend = Arc::clone(&backend);
                    async move { request.references.suggest(backend.as_ref(), &request.options).await }
                })
            })
        })
    }
}
impl From<Value> for ParsedCall {
    fn from(value: Value) -> Self {
        match value {
            | Value::Object(object) => Self::from_object(object),
            | _ => Self::Invalid {
                id: Value::Null,
                error: RpcError::new(INVALID_REQUEST, "Invalid Request"),
            },
        }
    }
}
impl ParsedCall {
    fn from_object(mut object: Map<String, Value>) -> Self {
        let id_value = object.remove("id");
        let jsonrpc = object.remove("jsonrpc");
        let method = object.remove("method").and_then(|value| match value {
            | Value::String(method) => MethodName::try_from(method).ok(),
            | _ => None,
        });
        let params = object.remove("params").unwrap_or_else(|| Value::Object(Map::new()));
        let valid_shape = object.is_empty()
            && matches!(jsonrpc, Some(Value::String(ref version)) if version == JSON_RPC_VERSION)
            && method.is_some()
            && matches!(params, Value::Object(_) | Value::Array(_));
        match (valid_shape, method) {
            | (false, _) | (_, None) => Self::Invalid {
                id: id_value.as_ref().map_or(Value::Null, RequestId::valid_response_value),
                error: RpcError::new(INVALID_REQUEST, "Invalid Request"),
            },
            | (true, Some(method)) => match id_value {
                | None => Self::Notification { method, params },
                | Some(value) => match serde_json::from_value::<RequestId>(value) {
                    | Ok(id) => Self::Request { id, method, params },
                    | Err(_) => Self::Invalid {
                        id: Value::Null,
                        error: RpcError::new(INVALID_REQUEST, "Invalid Request"),
                    },
                },
            },
        }
    }
}
impl RequestId {
    fn as_string(&self) -> String {
        match self {
            | Self::Number(value) => value.to_string(),
            | Self::String(value) => value.clone(),
        }
    }
    fn valid_response_value(value: &Value) -> Value {
        match value {
            | value @ (Value::String(_) | Value::Number(_)) => value.clone(),
            | _ => Value::Null,
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
    fn dispatch(self, id: Value) -> Dispatch {
        Dispatch::serialize(self.value(id))
    }
    /// Construct a public JSON-RPC error.
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            data: None,
            message: message.into(),
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
                let mut result = Ok(());
                loop {
                    match values.next_entry::<String, StrictValue>() {
                        | Ok(Some((key, StrictValue(_)))) if object.contains_key(&key) => {
                            result = Err(A::Error::custom(format!("duplicate field '{key}'")));
                            break;
                        }
                        | Ok(Some((key, StrictValue(value)))) => {
                            object.insert(key, value);
                        }
                        | Ok(None) => break,
                        | Err(why) => {
                            result = Err(why);
                            break;
                        }
                    }
                }
                result.map(|()| StrictValue(Value::Object(object)))
            }
            fn visit_none<E: DeError>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut values: A) -> Result<Self::Value, A::Error> {
                let mut array = Vec::new();
                let mut result = Ok(());
                loop {
                    match values.next_element::<StrictValue>() {
                        | Ok(Some(StrictValue(value))) => array.push(value),
                        | Ok(None) => break,
                        | Err(why) => {
                            result = Err(why);
                            break;
                        }
                    }
                }
                result.map(|()| StrictValue(Value::Array(array)))
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
        value.to_string()
    }
}
#[cfg(test)]
mod tests;
