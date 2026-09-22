//! Typed OpenAI-compatible inference adapter.
use super::{
    ChatCompletionRequest, ChatCompletionResponse, ChatMessageRequest, ChatResponseFormat, EndpointMode, Options, ProviderErrorResponse,
    ResponsesContent, ResponsesIncompleteDetails, ResponsesRequest, ResponsesResponse, ResponsesText,
};
use crate::{
    agent::{
        InferenceBackend, InferenceCapabilities, InferenceFinishReason, InferenceFuture, InferenceProvenance, InferenceRequest, InferenceResult,
        InferenceUsage,
    },
    io::{
        api::{Configuration, Endpoint},
        http::HttpResponse,
    },
};
use acorn_core::Scheme;
use acorn_macros::With;
use core::{fmt, time::Duration};
use secrecy::ExposeSecret;
use serde_json::Value;

const MAX_RESPONSE_BYTES: usize = 1024 * 1024;

trait Decode {
    fn decode(self, mode: EndpointMode, schema: Option<&Value>, selected_model: &str) -> Result<InferenceResult, InferenceError>;
}
trait InferenceRequestExt {
    fn build(&self, mode: EndpointMode, model: &str, structured_output: bool) -> Result<Value, InferenceError>;
    fn build_chat(&self, model: &str, structured_output: bool) -> Result<Value, InferenceError>;
    fn build_responses(&self, model: &str, structured_output: bool) -> Result<Value, InferenceError>;
}
/// Typed failure returned by OpenAI-compatible inference.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum InferenceError {
    /// HTTP inference does not support an agent selector.
    #[error("OpenAI-compatible inference does not support agent selector '{0}'")]
    AgentSelection(String),
    /// The configured endpoint would send credentials over plaintext remote HTTP.
    #[error("OpenAI-compatible endpoint '{0}' must use HTTPS unless it is loopback")]
    InsecureEndpoint(String),
    /// A request field is invalid.
    #[error("Invalid OpenAI-compatible inference request — {0}")]
    InvalidRequest(String),
    /// The endpoint returned a malformed success envelope.
    #[error("Invalid OpenAI-compatible inference response — {0}")]
    InvalidResponse(String),
    /// The supplied response schema cannot be compiled.
    #[error("Invalid inference response schema — {0}")]
    InvalidSchema(String),
    /// A remote endpoint requires a bearer credential.
    #[error("Remote OpenAI-compatible inference requires a bearer credential")]
    MissingAuthentication,
    /// Neither client configuration nor the request selected a model.
    #[error("OpenAI-compatible inference requires a configured model")]
    MissingModel,
    /// Offline policy rejected the configured remote endpoint.
    #[error("OpenAI-compatible inference through remote endpoint '{0}' is unavailable offline")]
    Offline(String),
    /// The provider returned a structured error envelope.
    #[error("OpenAI-compatible provider failed with HTTP {status} — {message}")]
    Provider {
        /// Provider error code, when supplied.
        code: Option<String>,
        /// Provider diagnostic without credentials or prompt content.
        message: String,
        /// HTTP status associated with the error.
        status: u16,
    },
    /// Parsed structured output does not satisfy the supplied JSON Schema.
    #[error("OpenAI-compatible structured output failed schema validation — {0}")]
    SchemaViolation(String),
    /// The request exceeded its caller-supplied deadline.
    #[error("OpenAI-compatible inference exceeded the {0:?} timeout")]
    Timeout(Duration),
    /// The HTTP transport failed before a provider response was available.
    #[error("OpenAI-compatible inference transport failed — {0}")]
    Transport(String),
}
/// OpenAI-compatible inference backend using the existing endpoint configuration.
#[derive(Clone, With)]
pub struct Client {
    #[with(skip)]
    capabilities: InferenceCapabilities,
    #[with(skip)]
    mode: EndpointMode,
    #[with(skip)]
    model: String,
    offline: bool,
    #[with(skip)]
    options: Options,
}
struct ValidatedRequest {
    loopback: bool,
    model: String,
}
impl Decode for ChatCompletionResponse {
    fn decode(self, _: EndpointMode, schema: Option<&Value>, selected_model: &str) -> Result<InferenceResult, InferenceError> {
        match self.choices.into_iter().next() {
            | None => Err(InferenceError::InvalidResponse("chat completion did not include a choice".to_string())),
            | Some(choice) => match choice.message.refusal.filter(|value| !value.trim().is_empty()) {
                | Some(refusal) => Ok(InferenceFinishReason::Refusal.normalized(
                    self.id,
                    self.model.unwrap_or_else(|| selected_model.to_string()),
                    None,
                    refusal,
                    self.usage.map(|usage| InferenceUsage {
                        input_tokens: usage.prompt_tokens,
                        output_tokens: usage.completion_tokens,
                    }),
                )),
                | None => match choice.message.content.filter(|value| !value.trim().is_empty()) {
                    | None => Err(InferenceError::InvalidResponse(
                        "chat completion did not include text or a refusal".to_string(),
                    )),
                    | Some(text) => parse_structured(&text, schema).map(|structured| {
                        InferenceFinishReason::finish(choice.finish_reason.as_deref()).normalized(
                            self.id,
                            self.model.unwrap_or_else(|| selected_model.to_string()),
                            structured,
                            text,
                            self.usage.map(|usage| InferenceUsage {
                                input_tokens: usage.prompt_tokens,
                                output_tokens: usage.completion_tokens,
                            }),
                        )
                    }),
                },
            },
        }
    }
}
impl Client {
    /// Create a client with an explicit endpoint mode and default model.
    pub fn new(options: Options, mode: EndpointMode, model: impl Into<String>) -> Self {
        Self {
            capabilities: InferenceCapabilities {
                attachments: false,
                mutating_tools: false,
                streaming: false,
                structured_output: false,
                tools: false,
            },
            mode,
            model: model.into(),
            offline: false,
            options,
        }
    }
    /// Run one checked OpenAI-compatible inference request.
    pub async fn infer_checked(&self, request: &InferenceRequest) -> Result<InferenceResult, InferenceError> {
        match validate_response_schema(request.response_schema.as_ref()).and_then(|()| self.validate_request(request)) {
            | Err(why) => Err(why),
            | Ok(validated) => match request.build(self.mode, &validated.model, self.capabilities.structured_output) {
                | Err(why) => Err(why),
                | Ok(body) => match serde_json::to_string(&body)
                    .map(|body| self.options.clone().with_body(body))
                    .map_err(|why| InferenceError::InvalidRequest(why.to_string()))
                {
                    | Err(why) => Err(why),
                    | Ok(options) => {
                        let timeout = Duration::from_millis(request.timeout_ms);
                        match tokio::time::timeout(timeout, self.mode.invoke(&options, validated.loopback, MAX_RESPONSE_BYTES)).await {
                            | Err(_) => Err(InferenceError::Timeout(timeout)),
                            | Ok(Err(why)) => Err(InferenceError::Transport(why.to_string())),
                            | Ok(Ok(response)) => response.decode(self.mode, request.response_schema.as_ref(), &validated.model),
                        }
                    }
                },
            },
        }
    }
    /// Configure whether the selected endpoint and model enforce JSON Schema output.
    pub fn with_structured_output(self, supported: bool) -> Self {
        Self {
            capabilities: InferenceCapabilities {
                structured_output: supported,
                ..self.capabilities
            },
            ..self
        }
    }
    fn validate_request(&self, request: &InferenceRequest) -> Result<ValidatedRequest, InferenceError> {
        Endpoint::from_template("openai::api")
            .map(|endpoint| endpoint.with_domain(self.options.domain()))
            .map_err(|why| InferenceError::InvalidRequest(why.to_string()))
            .and_then(|endpoint| {
                let loopback = endpoint.is_loopback();
                let secure = endpoint.scheme.as_ref().is_none_or(|scheme| *scheme == Scheme::HTTPS);
                let model = request.model.as_deref().unwrap_or(&self.model).trim().to_string();
                [
                    request
                        .prompt
                        .body
                        .trim()
                        .is_empty()
                        .then(|| InferenceError::InvalidRequest("inference prompt body cannot be empty".to_string())),
                    (request.timeout_ms == 0).then(|| InferenceError::InvalidRequest("inference timeout must be greater than zero".to_string())),
                    request.agent.as_ref().map(|agent| InferenceError::AgentSelection(agent.clone())),
                    model.is_empty().then_some(InferenceError::MissingModel),
                    (!loopback && !secure).then(|| InferenceError::InsecureEndpoint(endpoint.domain.clone())),
                    (self.offline && !loopback).then(|| InferenceError::Offline(endpoint.domain.clone())),
                    (!loopback && ExposeSecret::expose_secret(self.options.token()).trim().is_empty())
                        .then_some(InferenceError::MissingAuthentication),
                ]
                .into_iter()
                .flatten()
                .next()
                .map_or_else(|| Ok(ValidatedRequest { loopback, model }), Err)
            })
    }
}
impl fmt::Debug for Client {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Client")
            .field("capabilities", &self.capabilities)
            .field("mode", &self.mode)
            .field("offline", &self.offline)
            .finish()
    }
}
impl InferenceBackend for Client {
    fn capabilities(&self) -> InferenceCapabilities {
        self.capabilities
    }
    fn infer<'a>(&'a self, request: &'a InferenceRequest) -> InferenceFuture<'a> {
        Box::pin(async move { self.infer_checked(request).await.map_err(color_eyre::Report::new) })
    }
}
impl Decode for HttpResponse {
    fn decode(self, mode: EndpointMode, schema: Option<&Value>, selected_model: &str) -> Result<InferenceResult, InferenceError> {
        match (200..=299).contains(&self.status_code) {
            | false => Err(serde_json::from_slice::<ProviderErrorResponse>(&self.body)
                .map(|response| ProviderErrorResponse {
                    status: self.status_code,
                    ..response
                })
                .map(InferenceError::from)
                .unwrap_or_else(|_| InferenceError::Provider {
                    code: None,
                    message: String::from_utf8_lossy(self.body.get(..self.body.len().min(1024)).unwrap_or(&self.body)).into_owned(),
                    status: self.status_code,
                })),
            | true => match mode {
                | EndpointMode::ChatCompletions => serde_json::from_slice::<ChatCompletionResponse>(&self.body)
                    .map_err(|why| InferenceError::InvalidResponse(why.to_string()))
                    .and_then(|response| response.decode(mode, schema, selected_model)),
                | EndpointMode::Responses => serde_json::from_slice::<ResponsesResponse>(&self.body)
                    .map_err(|why| InferenceError::InvalidResponse(why.to_string()))
                    .and_then(|response| response.decode(mode, schema, selected_model)),
            },
        }
    }
}
impl From<ProviderErrorResponse> for InferenceError {
    fn from(value: ProviderErrorResponse) -> Self {
        Self::Provider {
            code: value.error.code,
            message: value.error.message,
            status: value.status,
        }
    }
}
impl InferenceFinishReason {
    fn finish(reason: Option<&str>) -> Self {
        match reason {
            | Some("content_filter") => Self::Refusal,
            | Some("length") => Self::Length,
            | Some("stop") | None => Self::Stop,
            | Some(other) => Self::Other(other.to_string()),
        }
    }
    fn normalized(
        self,
        request_id: String,
        model: String,
        structured: Option<Value>,
        text: String,
        usage: Option<InferenceUsage>,
    ) -> InferenceResult {
        InferenceResult {
            finish_reason: self,
            provenance: InferenceProvenance {
                agent: None,
                backend: "openai".to_string(),
                model: Some(model),
                request_id: Some(request_id),
            },
            structured,
            text,
            usage,
        }
    }
}
impl InferenceRequestExt for InferenceRequest {
    fn build(&self, mode: EndpointMode, model: &str, structured_output: bool) -> Result<Value, InferenceError> {
        match mode {
            | EndpointMode::ChatCompletions => self.build_chat(model, structured_output),
            | EndpointMode::Responses => self.build_responses(model, structured_output),
        }
    }
    fn build_chat(&self, model: &str, structured_output: bool) -> Result<Value, InferenceError> {
        let response_format = self
            .response_schema
            .as_ref()
            .filter(|_| structured_output)
            .map(|schema| ChatResponseFormat {
                json_schema: schema.clone().into(),
                kind: "json_schema",
            });
        serde_json::to_value(ChatCompletionRequest {
            messages: vec![ChatMessageRequest {
                content: self.prompt.body.clone(),
                role: "user",
            }],
            model: model.to_string(),
            response_format,
            store: false,
            stream: false,
        })
        .map_err(|why| InferenceError::InvalidRequest(why.to_string()))
    }
    fn build_responses(&self, model: &str, structured_output: bool) -> Result<Value, InferenceError> {
        let text = self.response_schema.as_ref().filter(|_| structured_output).map(|schema| ResponsesText {
            format: schema.clone().into(),
        });
        serde_json::to_value(ResponsesRequest {
            input: self.prompt.body.clone(),
            model: model.to_string(),
            store: false,
            stream: false,
            text,
        })
        .map_err(|why| InferenceError::InvalidRequest(why.to_string()))
    }
}
impl Decode for ResponsesResponse {
    fn decode(self, _: EndpointMode, schema: Option<&Value>, selected_model: &str) -> Result<InferenceResult, InferenceError> {
        match self.error {
            | Some(error) => Err(InferenceError::Provider {
                code: error.code,
                message: error.message,
                status: 200,
            }),
            | None => {
                let refusal = self
                    .output
                    .iter()
                    .flat_map(|output| output.content.iter())
                    .find_map(|content| match content {
                        | ResponsesContent::Refusal { refusal } if !refusal.trim().is_empty() => Some(refusal.clone()),
                        | _ => None,
                    });
                let text = self
                    .output
                    .iter()
                    .flat_map(|output| output.content.iter())
                    .filter_map(|content| match content {
                        | ResponsesContent::OutputText { text } => Some(text.as_str()),
                        | _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("");
                let usage = self.usage.map(|usage| InferenceUsage {
                    input_tokens: usage.input_tokens,
                    output_tokens: usage.output_tokens,
                });
                let model = self.model.unwrap_or_else(|| selected_model.to_string());
                match refusal {
                    | Some(refusal) => Ok(InferenceFinishReason::Refusal.normalized(self.id, model, None, refusal, usage)),
                    | None if text.trim().is_empty() => Err(InferenceError::InvalidResponse(
                        "response did not include output text or a refusal".to_string(),
                    )),
                    | None => parse_structured(&text, schema).map(|structured| {
                        responses_finish_reason(self.status.as_deref(), self.incomplete_details.as_ref())
                            .normalized(self.id, model, structured, text, usage)
                    }),
                }
            }
        }
    }
}
fn parse_structured(text: &str, schema: Option<&Value>) -> Result<Option<Value>, InferenceError> {
    match schema {
        | None => Ok(None),
        | Some(schema) => serde_json::from_str::<Value>(text)
            .map_err(|why| InferenceError::InvalidResponse(format!("structured output is not valid JSON — {why}")))
            .and_then(|value| match jsonschema::validator_for(schema) {
                | Err(why) => Err(InferenceError::InvalidSchema(why.to_string())),
                | Ok(validator) => match validator.validate(&value) {
                    | Err(why) => Err(InferenceError::SchemaViolation(why.to_string())),
                    | Ok(()) => Ok(Some(value)),
                },
            }),
    }
}
fn responses_finish_reason(status: Option<&str>, details: Option<&ResponsesIncompleteDetails>) -> InferenceFinishReason {
    match (status, details.and_then(|details| details.reason.as_deref())) {
        | (Some("incomplete"), Some("max_output_tokens")) => InferenceFinishReason::Length,
        | (Some("completed") | None, _) => InferenceFinishReason::Stop,
        | (Some(status), _) => InferenceFinishReason::Other(status.to_string()),
    }
}
fn validate_response_schema(schema: Option<&Value>) -> Result<(), InferenceError> {
    match schema {
        | None => Ok(()),
        | Some(schema) => jsonschema::validator_for(schema)
            .map(|_| ())
            .map_err(|why| InferenceError::InvalidSchema(why.to_string())),
    }
}
