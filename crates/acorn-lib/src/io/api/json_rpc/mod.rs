//! Host extensions over the canonical portable operation registry.
use crate::agent::{run_inference, InferenceBackend, InferenceReadiness, InferenceRequest, InferenceResult};
use crate::analyzer::fix::{CheckReferences, FixReport, FixSuggestionRequest};
use crate::io::api::webhooks::store::OperationQueue;
use crate::io::ApiResult;
use acorn_service::{AcornService, OperationRegistry, ServiceError, ServiceErrorKind, ServiceResult};
use alloc::sync::Arc;
use core::fmt;
use core::future::Future;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

pub use acorn_service::{
    Dispatch, DurableOperations, InvocationContext, InvocationOrigin, MethodName, Notification, OperationDefinition, OperationEffect,
    OperationEffects, Principal, RequestId, RpcError, INTERNAL_ERROR, INVALID_PARAMS, INVALID_REQUEST, JSON_RPC_VERSION, METHOD_NOT_FOUND,
    PARSE_ERROR, POLICY_DENIED,
};

/// Host registry wrapper that delegates framing, policy, and dispatch to `acorn-service`.
#[derive(Clone, Default)]
pub struct JsonRpcOperationRegistry(OperationRegistry);
impl fmt::Debug for JsonRpcOperationRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OperationRegistry")
            .field("definitions", &self.0.definitions())
            .finish()
    }
}
impl JsonRpcOperationRegistry {
    /// Build the canonical ACORN registry with the configured durable operation queue.
    pub fn acorn() -> ApiResult<Self> {
        let durable: Arc<dyn DurableOperations> = Arc::new(OperationQueue::configured());
        OperationRegistry::acorn(AcornService, env!("CARGO_PKG_VERSION"), Some(durable))
            .map(Self)
            .map_err(color_eyre::Report::from)
    }
    /// Dispatch one JSON-RPC document through the canonical registry.
    pub async fn dispatch(&self, body: &[u8], context: InvocationContext) -> Dispatch {
        self.0.dispatch(body, context).await
    }
    /// Return registered operation definitions in deterministic order.
    pub fn definitions(&self) -> Vec<OperationDefinition> {
        self.0.definitions()
    }
    /// Invoke one registered operation through the canonical policy boundary.
    pub async fn invoke(&self, method: &MethodName, params: Value, context: InvocationContext) -> Result<Value, RpcError> {
        self.0.invoke(method, params, context).await
    }
    /// Return registered operation names in deterministic order.
    pub fn names(&self) -> Vec<MethodName> {
        self.0.names()
    }
    /// Register one host handler while retaining canonical dispatch and error mapping.
    pub fn register<I, O, F, Fut>(self, definition: OperationDefinition, handler: F) -> ApiResult<Self>
    where
        I: DeserializeOwned + Send + 'static,
        O: Serialize + Send + 'static,
        F: Fn(I, InvocationContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ApiResult<O>> + Send + 'static,
    {
        let operation_name = definition.name.to_string();
        self.0
            .register_definition(definition, move |input, context| {
                let future = handler(input, context);
                let operation_name = operation_name.clone();
                async move {
                    future
                        .await
                        .map_err(|_| ServiceError::new(ServiceErrorKind::Internal, format!("Operation '{operation_name}' failed")))
                }
            })
            .map(Self)
            .map_err(color_eyre::Report::from)
    }
    /// Register inference run, readiness, and fix-suggestion host operations.
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
        .map_err(color_eyre::Report::from)
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
            .map_err(color_eyre::Report::from)
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
            .map_err(color_eyre::Report::from)
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

#[cfg(test)]
mod tests;
