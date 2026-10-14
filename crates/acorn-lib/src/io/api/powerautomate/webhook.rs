//! Authenticated durable PowerAutomate form admission.
use super::{FormReceipt, FormSubmission, PowerAutomateConfig};
use crate::io::api;
use crate::io::api::json_rpc::{MethodName, Notification};
use crate::io::api::webhooks::store::OperationQueue;
use crate::io::api::webhooks::{
    self, Delivery, ErrorKind, HeaderSecretVerifier, InboundRequest, OperationSpec, Verifier, WebhookError, WebhookProvider, WebhookRuntime,
};
use crate::io::config::application::FolderRegistry;
use crate::io::ApiResult;
use crate::util::constants::env::POWER_AUTOMATE_INBOUND_SECRET;
use acorn_schema::validation::Validate;
use axum::body::to_bytes;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use jiff::{SignedDuration, Timestamp};
use std::env;

const DELIVERY_HEADER: &str = "x-acorn-delivery-id";
const MAX_FORM_BYTES: usize = 1024 * 1024;
const SECRET_HEADER: &str = "x-acorn-webhook-secret";
const STALE_CLAIM_AFTER: SignedDuration = SignedDuration::from_mins(5);
/// Authenticated PowerAutomate webhook provider.
#[derive(Clone)]
pub struct PowerAutomateProvider {
    config: PowerAutomateConfig,
    routing: FolderRegistry,
    verifier: HeaderSecretVerifier,
}
#[derive(Clone)]
struct RouterState {
    provider: PowerAutomateProvider,
    queue: OperationQueue,
    runtime: WebhookRuntime,
}
impl PowerAutomateProvider {
    /// Construct a provider with an explicit inbound shared secret.
    pub fn new(config: PowerAutomateConfig, routing: FolderRegistry, inbound_secret: impl Into<api::Secret>) -> ApiResult<Self> {
        config
            .validate()
            .map_err(|why| color_eyre::eyre::eyre!("Invalid PowerAutomate configuration — {why}"))
            .and_then(|()| {
                HeaderSecretVerifier::new(SECRET_HEADER, inbound_secret, &[DELIVERY_HEADER])
                    .map_err(Into::into)
                    .map(|verifier| Self { config, routing, verifier })
            })
    }
    /// Construct a provider by resolving the configured inbound secret environment variable.
    pub fn from_config(config: PowerAutomateConfig, routing: FolderRegistry) -> ApiResult<Self> {
        routing
            .variable(POWER_AUTOMATE_INBOUND_SECRET)
            .ok_or_else(|| color_eyre::eyre::eyre!("PowerAutomate inbound shared secret is not referenced by variables.custom"))
            .and_then(|name| env::var(name).map_err(|_| color_eyre::eyre::eyre!("PowerAutomate inbound shared secret is unavailable")))
            .and_then(|secret| Self::new(config, routing, secret))
    }
    /// Build the isolated form intake router.
    pub fn router(self, queue: OperationQueue, runtime: WebhookRuntime) -> Router {
        let _ = runtime.initialize(&queue, STALE_CLAIM_AFTER);
        Router::new()
            .route("/webhooks/powerautomate/forms", post(receive))
            .with_state(RouterState {
                provider: self,
                queue,
                runtime,
            })
    }
    /// Return the deployment configuration used by this provider.
    pub fn config(&self) -> &PowerAutomateConfig {
        &self.config
    }
    /// Return the ACRE routing snapshot supplying folders and variable references.
    pub fn routing(&self) -> &FolderRegistry {
        &self.routing
    }
}
impl WebhookProvider for PowerAutomateProvider {
    type Event = FormSubmission;

    fn receive(&self, request: InboundRequest<'_>, now: Timestamp) -> Result<Delivery<Self::Event>, WebhookError> {
        self.verifier.verify(request, now).and_then(|verified| {
            serde_json::from_slice::<FormSubmission>(verified.raw_body)
                .map_err(|_| WebhookError::new(ErrorKind::BadRequest, "Invalid PowerAutomate form payload"))
                .and_then(|event| {
                    event
                        .entry(&self.routing)
                        .map_err(|_| WebhookError::new(ErrorKind::BadRequest, "Invalid or unauthorized PowerAutomate form payload"))
                        .map(|_| Delivery {
                            delivery_id: verified.delivery_id.to_string(),
                            event,
                        })
                })
        })
    }
    fn classify(&self, delivery: &Delivery<Self::Event>) -> Result<Vec<OperationSpec>, WebhookError> {
        serde_json::to_value(&delivery.event)
            .map_err(|_| WebhookError::new(ErrorKind::BadRequest, "Failed to encode PowerAutomate form operation"))
            .and_then(|params| {
                Notification::new(MethodName::from(["webhooks", "powerautomate", "form"]), params)
                    .map_err(|_| WebhookError::new(ErrorKind::BadRequest, "Failed to create PowerAutomate form operation"))
            })
            .map(|notification| {
                vec![OperationSpec::new(
                    self.name(),
                    format!("form:v1:{}", delivery.event.submission_id.to_ascii_lowercase()),
                    notification,
                )]
            })
    }
    fn name(&self) -> &'static str {
        "powerautomate"
    }
}
async fn receive(
    State(state): State<RouterState>,
    request: Request<axum::body::Body>,
) -> Result<(StatusCode, Json<FormReceipt>), (StatusCode, String)> {
    match state.runtime.is_ready() {
        | false => Err((StatusCode::SERVICE_UNAVAILABLE, "Webhook runtime is not ready".to_string())),
        | true => {
            let (parts, body) = request.into_parts();
            to_bytes(body, MAX_FORM_BYTES)
                .await
                .map_err(|_| (StatusCode::PAYLOAD_TOO_LARGE, "Request body exceeds 1 MiB limit".to_string()))
                .and_then(|raw_body| {
                    webhooks::prepare(
                        &state.provider,
                        InboundRequest {
                            headers: &parts.headers,
                            raw_body: &raw_body,
                        },
                        Timestamp::now(),
                    )
                    .map_err(WebhookError::response)
                    .and_then(|prepared| {
                        let operation_id = prepared
                            .operations
                            .first()
                            .map(|operation| format!("{}:{}", operation.provider, operation.idempotency_key))
                            .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "PowerAutomate form produced no operation".to_string()));
                        operation_id.and_then(|operation_id| {
                            state
                                .queue
                                .enqueue_prepared(&prepared)
                                .map_err(|why| (StatusCode::SERVICE_UNAVAILABLE, why.to_string()))
                                .and_then(|statuses| {
                                    statuses
                                        .first()
                                        .copied()
                                        .map(|status| (StatusCode::ACCEPTED, Json(FormReceipt::new(operation_id, status))))
                                        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "PowerAutomate form was not queued".to_string()))
                                })
                        })
                    })
                })
        }
    }
}
