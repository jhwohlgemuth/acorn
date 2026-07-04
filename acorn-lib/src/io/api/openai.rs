//! Module for interacting with OpenAI-compatible interfaces
//!
//! See <https://platform.openai.com/docs/api-reference> for more information
//!
use crate::io::api::{bearer_auth_header, ApiResult, Configuration, Endpoint, Fallback, RemoteResource};
use crate::param;
use crate::prelude::var;
use crate::util::Label;
use bon::Builder;
use color_eyre::eyre::eyre;
use dotenvy;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Raw OpenAI chat completion response payload.
///
/// The OpenAPI schema for chat completions is broad and evolves quickly,
/// so the baseline implementation preserves the full JSON document.
pub type ChatCompletionListResponse = serde_json::Value;
/// Raw OpenAI chat completion response payload.
///
/// The OpenAPI schema for chat completions is broad and evolves quickly,
/// so the baseline implementation preserves the full JSON document.
pub type ChatCompletionResponse = serde_json::Value;
/// Raw OpenAI response payload for chat completion message lists.
///
/// The OpenAPI schema for chat completion messages is broad and evolves quickly,
/// so the baseline implementation preserves the full JSON document.
pub type ChatCompletionMessagesResponse = serde_json::Value;
/// Raw OpenAI responses API payload.
///
/// The OpenAPI schema for responses is broad and evolves quickly,
/// so the baseline implementation preserves the full JSON document.
pub type CreateResponseOutput = serde_json::Value;
/// Raw OpenAI responses deletion payload.
///
/// The OpenAPI schema for response deletion is broad and evolves quickly,
/// so the baseline implementation preserves the full JSON document.
pub type DeleteResponseOutput = serde_json::Value;
/// Raw OpenAI model deletion payload.
///
/// The OpenAPI schema for model deletion is broad and evolves quickly,
/// so the baseline implementation preserves the full JSON document.
pub type ModelDeleteOutput = serde_json::Value;
/// Raw OpenAI response input item list payload.
///
/// The OpenAPI schema for response input items is broad and evolves quickly,
/// so the baseline implementation preserves the full JSON document.
pub type ResponseInputItemsOutput = serde_json::Value;
/// OpenAI API error payload
///
/// Matches `#/components/schemas/Error` from the OpenAI OpenAPI specification.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Error {
    /// Stable error code, when available
    pub code: Option<String>,
    /// Human-readable error message
    pub message: String,
    /// Parameter associated with this error, when applicable
    pub param: Option<String>,
    /// Error type identifier
    #[serde(rename = "type")]
    pub error_type: String,
}
/// OpenAI API error response
///
/// Matches `#/components/schemas/ErrorResponse` from the OpenAI OpenAPI specification.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Wrapped error details
    pub error: Error,
}
/// OpenAI list models response
///
/// Matches baseline fields from `#/components/schemas/ListModelsResponse`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListModelsResponse {
    /// Object type, expected to be `list`
    pub object: String,
    /// Collection of models available to the caller
    pub data: Vec<Model>,
}
/// OpenAI model descriptor
///
/// Matches baseline fields from `#/components/schemas/Model`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    /// Model identifier (for example, `gpt-5.4`)
    pub id: String,
    /// Object type, typically `model`
    pub object: String,
    /// Unix timestamp (seconds) when model metadata was created
    pub created: i64,
    /// Owning organization
    pub owned_by: String,
}
/// OpenAI API options
///
/// Configuration options used across OpenAI API operations.
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = with_token, on(String, into))]
pub struct Options {
    /// Bearer token for authentication
    #[builder(start_fn)]
    pub token: String,
    /// Request body payload for POST requests
    pub body: Option<String>,
    /// OpenAI API domain (defaults to `api.openai.com`)
    #[builder(default = String::from("api.openai.com"))]
    pub domain: String,
    /// Optional model identifier for model retrieval
    pub identifier: Option<String>,
    /// Optional cursor for paginated list APIs
    pub after: Option<String>,
    /// Optional page size for list APIs
    pub limit: Option<u32>,
    /// Optional ordering value for list APIs (for example, `asc` or `desc`)
    pub order: Option<String>,
    /// Optional model filter used by selected list APIs
    pub model: Option<String>,
}
impl Configuration for Options {
    /// Build options from OpenAI-related environment variables.
    /// - `OPENAI_API_KEY` -> `token` (defaults to empty string when unset)
    /// - `OPENAI_SERVER_HOST` -> `domain` (defaults to api.openai.com when unset)
    fn from_env() -> Self {
        if let Err(why) = dotenvy::from_filename(".env") {
            debug!("=> {} Load .env — {why}", Label::skip());
        }
        Self {
            token: var("OPENAI_API_KEY").unwrap_or_default(),
            body: None,
            domain: var("OPENAI_SERVER_HOST").unwrap_or_else(|_| String::from("api.openai.com")),
            identifier: None,
            after: None,
            limit: None,
            order: None,
            model: None,
        }
    }
    /// Return a copy of options with request body payload set
    fn with_body(self, value: impl Into<String>) -> Self {
        Self {
            body: Some(value.into()),
            ..self
        }
    }
    /// Return a copy of options with OpenAI server domain set
    fn with_domain(self, value: impl Into<String>) -> Self {
        Self {
            domain: value.into(),
            ..self
        }
    }
}
impl Options {
    /// Return a copy of options with list cursor set
    pub fn with_after(self, value: impl Into<String>) -> Self {
        Self {
            after: Some(value.into()),
            ..self
        }
    }
    /// Return a copy of options with model identifier set
    pub fn with_identifier(self, value: impl Into<String>) -> Self {
        Self {
            identifier: Some(value.into()),
            ..self
        }
    }
    /// Return a copy of options with page size set
    pub fn with_limit(self, value: u32) -> Self {
        Self { limit: Some(value), ..self }
    }
    /// Return a copy of options with model filter set
    pub fn with_model(self, value: impl Into<String>) -> Self {
        Self {
            model: Some(value.into()),
            ..self
        }
    }
    /// Return a copy of options with order value set
    pub fn with_order(self, value: impl Into<String>) -> Self {
        Self {
            order: Some(value.into()),
            ..self
        }
    }
}
/// Create a chat completion response via `POST /chat/completions`.
pub async fn chat_completion(options: Options) -> ApiResult<ChatCompletionResponse> {
    let Options { token, domain, body, .. } = options;
    let template = "openai::api";
    let action = "chat-completion";
    match Endpoint::from_template(template).map(|e| e.with_domain(domain.as_str())) {
        | Ok(endpoint) => match body {
            | Some(value) if !value.is_empty() => {
                let params = bearer_auth_header(&token).into_iter().chain([param!(Body, &value)]).collect();
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle_or::<ChatCompletionResponse, Fallback<ErrorResponse>>(response)
            }
            | Some(_) | None => Err(eyre!("OpenAI chat completion request body is required")),
        },
        | Err(why) => Err(why),
    }
}
/// List stored chat completions via `GET /chat/completions`.
pub async fn chat_completion_list(options: Options) -> ApiResult<ChatCompletionListResponse> {
    let Options {
        token,
        domain,
        after,
        limit,
        order,
        model,
        ..
    } = options;
    let template = "openai::api";
    let action = "chat-completion::list";
    match Endpoint::from_template(template).map(|e| e.with_domain(domain.as_str())) {
        | Ok(endpoint) => {
            let mut params = bearer_auth_header(&token).into_iter().collect::<Vec<_>>();
            if let Some(value) = after.filter(|value| !value.is_empty()) {
                params.push(param!(KeyValuePair, "after", &value));
            }
            if let Some(value) = limit {
                params.push(param!(KeyValuePair, "limit", &value.to_string()));
            }
            if let Some(value) = order.filter(|value| !value.is_empty()) {
                params.push(param!(KeyValuePair, "order", &value));
            }
            if let Some(value) = model.filter(|value| !value.is_empty()) {
                params.push(param!(KeyValuePair, "model", &value));
            }
            let response = endpoint.invoke(action, Some(params)).await;
            endpoint.handle_or::<ChatCompletionListResponse, Fallback<ErrorResponse>>(response)
        }
        | Err(why) => Err(why),
    }
}
/// Delete a stored chat completion by identifier via `DELETE /chat/completions/{completion_id}`.
pub async fn chat_completion_delete(options: Options) -> ApiResult<DeleteResponseOutput> {
    let Options {
        token, domain, identifier, ..
    } = options;
    let template = "openai::api";
    let action = "chat-completion::delete";
    match Endpoint::from_template(template).map(|e| e.with_domain(domain.as_str())) {
        | Ok(endpoint) => match identifier {
            | Some(value) if !value.is_empty() => {
                let params = bearer_auth_header(&token)
                    .into_iter()
                    .chain([param!(TemplateValue, "identifier", &value)])
                    .collect();
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle_or::<DeleteResponseOutput, Fallback<ErrorResponse>>(response)
            }
            | Some(_) | None => Err(eyre!("OpenAI chat completion identifier is required")),
        },
        | Err(why) => Err(why),
    }
}
/// List stored chat completion messages by completion identifier.
pub async fn chat_completion_messages(options: Options) -> ApiResult<ChatCompletionMessagesResponse> {
    let Options {
        token,
        domain,
        identifier,
        after,
        limit,
        order,
        ..
    } = options;
    let template = "openai::api";
    let action = "chat-completion::messages";
    match Endpoint::from_template(template).map(|e| e.with_domain(domain.as_str())) {
        | Ok(endpoint) => match identifier {
            | Some(value) if !value.is_empty() => {
                let mut params = bearer_auth_header(&token)
                    .into_iter()
                    .chain([param!(TemplateValue, "identifier", &value)])
                    .collect::<Vec<_>>();
                if let Some(value) = after.filter(|value| !value.is_empty()) {
                    params.push(param!(KeyValuePair, "after", &value));
                }
                if let Some(value) = limit {
                    params.push(param!(KeyValuePair, "limit", &value.to_string()));
                }
                if let Some(value) = order.filter(|value| !value.is_empty()) {
                    params.push(param!(KeyValuePair, "order", &value));
                }
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle_or::<ChatCompletionMessagesResponse, Fallback<ErrorResponse>>(response)
            }
            | Some(_) | None => Err(eyre!("OpenAI chat completion identifier is required")),
        },
        | Err(why) => Err(why),
    }
}
/// Retrieve a stored chat completion by identifier via `GET /chat/completions/{completion_id}`.
pub async fn chat_completion_retrieve(options: Options) -> ApiResult<ChatCompletionResponse> {
    let Options {
        token, domain, identifier, ..
    } = options;
    let template = "openai::api";
    let action = "chat-completion::retrieve";
    match Endpoint::from_template(template).map(|e| e.with_domain(domain.as_str())) {
        | Ok(endpoint) => match identifier {
            | Some(value) if !value.is_empty() => {
                let params = bearer_auth_header(&token)
                    .into_iter()
                    .chain([param!(TemplateValue, "identifier", &value)])
                    .collect();
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle_or::<ChatCompletionResponse, Fallback<ErrorResponse>>(response)
            }
            | Some(_) | None => Err(eyre!("OpenAI chat completion identifier is required")),
        },
        | Err(why) => Err(why),
    }
}
/// Update a stored chat completion by identifier via `POST /chat/completions/{completion_id}`.
pub async fn chat_completion_update(options: Options) -> ApiResult<ChatCompletionResponse> {
    let Options {
        token,
        domain,
        identifier,
        body,
        ..
    } = options;
    let template = "openai::api";
    let action = "chat-completion::update";
    match Endpoint::from_template(template).map(|e| e.with_domain(domain.as_str())) {
        | Ok(endpoint) => match (identifier, body) {
            | (Some(id), Some(value)) if !id.is_empty() && !value.is_empty() => {
                let params = bearer_auth_header(&token)
                    .into_iter()
                    .chain([param!(TemplateValue, "identifier", &id), param!(Body, &value)])
                    .collect();
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle_or::<ChatCompletionResponse, Fallback<ErrorResponse>>(response)
            }
            | (Some(_), Some(_)) => Err(eyre!("OpenAI chat completion identifier and body are required")),
            | _ => Err(eyre!("OpenAI chat completion identifier and body are required")),
        },
        | Err(why) => Err(why),
    }
}
/// Retrieve details for a specific model identifier.
pub async fn model(options: Options) -> ApiResult<Model> {
    let Options {
        token, domain, identifier, ..
    } = options;
    let template = "openai::api";
    let action = "model";
    match Endpoint::from_template(template).map(|e| e.with_domain(domain.as_str())) {
        | Ok(endpoint) => match identifier {
            | Some(value) if !value.is_empty() => {
                let params = bearer_auth_header(&token)
                    .into_iter()
                    .chain([param!(TemplateValue, "identifier", &value)])
                    .collect();
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle_or::<Model, Fallback<ErrorResponse>>(response)
            }
            | Some(_) | None => Err(eyre!("OpenAI model identifier is required")),
        },
        | Err(why) => Err(why),
    }
}
/// Delete a model by identifier via `DELETE /models/{model}`.
pub async fn model_delete(options: Options) -> ApiResult<ModelDeleteOutput> {
    let Options {
        token, domain, identifier, ..
    } = options;
    let template = "openai::api";
    let action = "model::delete";
    match Endpoint::from_template(template).map(|e| e.with_domain(domain.as_str())) {
        | Ok(endpoint) => match identifier {
            | Some(value) if !value.is_empty() => {
                let params = bearer_auth_header(&token)
                    .into_iter()
                    .chain([param!(TemplateValue, "identifier", &value)])
                    .collect();
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle_or::<ModelDeleteOutput, Fallback<ErrorResponse>>(response)
            }
            | Some(_) | None => Err(eyre!("OpenAI model identifier is required")),
        },
        | Err(why) => Err(why),
    }
}
/// Retrieve all models available to the authenticated caller.
pub async fn models(options: Options) -> ApiResult<ListModelsResponse> {
    let Options { token, domain, .. } = options;
    let template = "openai::api";
    let action = "models";
    match Endpoint::from_template(template).map(|e| e.with_domain(domain.as_str())) {
        | Ok(endpoint) => {
            let params = bearer_auth_header(&token).into_iter().collect();
            let response = endpoint.invoke(action, Some(params)).await;
            endpoint.handle_or::<ListModelsResponse, Fallback<ErrorResponse>>(response)
        }
        | Err(why) => Err(why),
    }
}
/// Create a response via `POST /responses`.
pub async fn response(options: Options) -> ApiResult<CreateResponseOutput> {
    let Options { token, domain, body, .. } = options;
    let template = "openai::api";
    let action = "response";
    match Endpoint::from_template(template).map(|e| e.with_domain(domain.as_str())) {
        | Ok(endpoint) => match body {
            | Some(value) if !value.is_empty() => {
                let params = bearer_auth_header(&token).into_iter().chain([param!(Body, &value)]).collect();
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle_or::<CreateResponseOutput, Fallback<ErrorResponse>>(response)
            }
            | Some(_) | None => Err(eyre!("OpenAI responses request body is required")),
        },
        | Err(why) => Err(why),
    }
}
/// Delete a response by identifier via `DELETE /responses/{response_id}`.
pub async fn response_delete(options: Options) -> ApiResult<DeleteResponseOutput> {
    let Options {
        token, domain, identifier, ..
    } = options;
    let template = "openai::api";
    let action = "response::delete";
    match Endpoint::from_template(template).map(|e| e.with_domain(domain.as_str())) {
        | Ok(endpoint) => match identifier {
            | Some(value) if !value.is_empty() => {
                let params = bearer_auth_header(&token)
                    .into_iter()
                    .chain([param!(TemplateValue, "identifier", &value)])
                    .collect();
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle_or::<DeleteResponseOutput, Fallback<ErrorResponse>>(response)
            }
            | Some(_) | None => Err(eyre!("OpenAI response identifier is required")),
        },
        | Err(why) => Err(why),
    }
}
/// List input items for a response by identifier via `GET /responses/{response_id}/input_items`.
pub async fn response_input_items(options: Options) -> ApiResult<ResponseInputItemsOutput> {
    let Options {
        token,
        domain,
        identifier,
        after,
        limit,
        order,
        ..
    } = options;
    let template = "openai::api";
    let action = "response::input-items";
    match Endpoint::from_template(template).map(|e| e.with_domain(domain.as_str())) {
        | Ok(endpoint) => match identifier {
            | Some(value) if !value.is_empty() => {
                let mut params = bearer_auth_header(&token)
                    .into_iter()
                    .chain([param!(TemplateValue, "identifier", &value)])
                    .collect::<Vec<_>>();
                if let Some(value) = after.filter(|value| !value.is_empty()) {
                    params.push(param!(KeyValuePair, "after", &value));
                }
                if let Some(value) = limit {
                    params.push(param!(KeyValuePair, "limit", &value.to_string()));
                }
                if let Some(value) = order.filter(|value| !value.is_empty()) {
                    params.push(param!(KeyValuePair, "order", &value));
                }
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle_or::<ResponseInputItemsOutput, Fallback<ErrorResponse>>(response)
            }
            | Some(_) | None => Err(eyre!("OpenAI response identifier is required")),
        },
        | Err(why) => Err(why),
    }
}
/// Cancel a response by identifier via `POST /responses/{response_id}/cancel`.
pub async fn response_cancel(options: Options) -> ApiResult<CreateResponseOutput> {
    let Options {
        token, domain, identifier, ..
    } = options;
    let template = "openai::api";
    let action = "response::cancel";
    match Endpoint::from_template(template).map(|e| e.with_domain(domain.as_str())) {
        | Ok(endpoint) => match identifier {
            | Some(value) if !value.is_empty() => {
                let params = bearer_auth_header(&token)
                    .into_iter()
                    .chain([param!(TemplateValue, "identifier", &value)])
                    .collect();
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle_or::<CreateResponseOutput, Fallback<ErrorResponse>>(response)
            }
            | Some(_) | None => Err(eyre!("OpenAI response identifier is required")),
        },
        | Err(why) => Err(why),
    }
}
/// Retrieve a response by identifier via `GET /responses/{response_id}`.
pub async fn response_retrieve(options: Options) -> ApiResult<CreateResponseOutput> {
    let Options {
        token, domain, identifier, ..
    } = options;
    let template = "openai::api";
    let action = "response::retrieve";
    match Endpoint::from_template(template).map(|e| e.with_domain(domain.as_str())) {
        | Ok(endpoint) => match identifier {
            | Some(value) if !value.is_empty() => {
                let params = bearer_auth_header(&token)
                    .into_iter()
                    .chain([param!(TemplateValue, "identifier", &value)])
                    .collect();
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle_or::<CreateResponseOutput, Fallback<ErrorResponse>>(response)
            }
            | Some(_) | None => Err(eyre!("OpenAI response identifier is required")),
        },
        | Err(why) => Err(why),
    }
}
