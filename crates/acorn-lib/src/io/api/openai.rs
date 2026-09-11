//! Module for interacting with OpenAI-compatible interfaces.
//!
//! This module intentionally implements the OpenAI-compatible endpoint subset
//! supported by [llama-swap](https://github.com/mostlygeek/llama-swap):
//! completions, chat completions, responses, embeddings, model listing, audio
//! speech/transcriptions/voices, and image generations/edits. It does not try
//! to mirror the full OpenAI API surface because ACORN primarily needs a stable
//! interface for local OpenAI-compatible routers and model-swapping proxies.
//!
//! Set `OPENAI_SERVER_HOST` to a llama-swap host such as `localhost:8080` or
//! `http://localhost:8080`; when unset, requests default to `api.openai.com`.
//! Set `OPENAI_API_KEY` when the upstream server requires Bearer auth.
//!
//! # Examples
//!
//! Create a chat completion against a local llama-swap server:
//!
//! ```no_run
//! use acorn::io::api::Configuration;
//! use acorn::io::api::openai;
//!
//! # async fn example() -> color_eyre::Result<()> {
//! let body = r#"{
//!     "model": "qwen3-coder",
//!     "messages": [{"role": "user", "content": "Summarize ACORN."}]
//! }"#;
//! let options = openai::Options::from_env()
//!     .with_domain("http://localhost:8080")
//!     .with_body(body);
//! let output = openai::chat_completion(&options).await?;
//! println!("{output:#}");
//! # Ok(())
//! # }
//! ```
//!
//! List models exposed by the configured OpenAI-compatible server:
//!
//! ```no_run
//! use acorn::io::api::Configuration;
//! use acorn::io::api::openai;
//!
//! # async fn example() -> color_eyre::Result<()> {
//! let options = openai::Options::from_env().with_domain("http://localhost:8080");
//! let models = openai::models(&options).await?;
//! for model in models.data {
//!     println!("{}", model.id);
//! }
//! # Ok(())
//! # }
//! ```
use crate::io::api::{ApiResult, Configuration, Endpoint, Fallback, Param, Params, RemoteResource, TextResponse};
use crate::param;
use crate::util::constants::app::DEFAULT_OPENAI_DOMAIN;
use crate::util::constants::env::{OPENAI_API_KEY, OPENAI_SERVER_HOST};
use acorn_core::options::{ApiExtension, ApiOptions};
use color_eyre::eyre::eyre;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

/// Raw OpenAI audio speech response payload.
///
/// Audio speech usually returns non-JSON audio bytes, so this preserves the raw response text.
pub type AudioSpeechResponse = TextResponse;
/// OpenAI API options
///
/// Configuration options used across OpenAI API operations.
pub type Options = ApiOptions<Extension, Param>;
/// Raw OpenAI API response payload.
///
/// The OpenAI-compatible schema is broad and evolves quickly,
/// so the baseline implementation preserves the full JSON document.
pub type Response = serde_json::Value;
enum BodyMode {
    Empty,
    Required,
}
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
/// OpenAI-specific API option defaults.
#[derive(Clone, Debug, Default)]
pub struct Extension;
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
impl ApiExtension for Extension {
    fn default_domain() -> String {
        String::from(DEFAULT_OPENAI_DOMAIN)
    }
    fn env_token_var() -> &'static str {
        OPENAI_API_KEY
    }
    fn env_domain_var() -> &'static str {
        OPENAI_SERVER_HOST
    }
}
/// Create audio speech via `POST /audio/speech`.
pub async fn audio_speech(options: &Options) -> ApiResult<AudioSpeechResponse> {
    invoke(options, "audio-speech", BodyMode::Required).await
}
/// Create an audio transcription via `POST /audio/transcriptions`.
pub async fn audio_transcription(options: &Options) -> ApiResult<Response> {
    invoke(options, "audio-transcription", BodyMode::Required).await
}
/// Retrieve available audio voices via `GET /audio/voices`.
pub async fn audio_voices(options: &Options) -> ApiResult<Response> {
    invoke(options, "audio-voices", BodyMode::Empty).await
}
/// Create a chat completion response via `POST /chat/completions`.
pub async fn chat_completion(options: &Options) -> ApiResult<Response> {
    invoke(options, "chat-completion", BodyMode::Required).await
}
/// Create a completion via `POST /completions`.
pub async fn completion(options: &Options) -> ApiResult<Response> {
    invoke(options, "completion", BodyMode::Required).await
}
/// Create embeddings via `POST /embeddings`.
pub async fn embedding(options: &Options) -> ApiResult<Response> {
    invoke(options, "embedding", BodyMode::Required).await
}
/// Edit an image via `POST /images/edits`.
pub async fn image_edit(options: &Options) -> ApiResult<Response> {
    invoke(options, "image-edit", BodyMode::Required).await
}
/// Create an image via `POST /images/generations`.
pub async fn image_generation(options: &Options) -> ApiResult<Response> {
    invoke(options, "image-generation", BodyMode::Required).await
}
async fn invoke<R>(options: &Options, action: &str, body_mode: BodyMode) -> ApiResult<R>
where
    R: for<'de> Deserialize<'de>,
{
    let template = "openai::api";
    match Endpoint::from_template(template).map(|e| e.with_domain(options.domain())) {
        | Ok(endpoint) => match (body_mode, &options.body) {
            | (BodyMode::Required, Some(value)) if !value.is_empty() => {
                let params = Params::new()
                    .with_auth(ExposeSecret::expose_secret(options.token()), None)
                    .with(param!(Body, value.as_str()))
                    .with_custom(options.params())
                    .build();
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle_or::<R, Fallback<ErrorResponse>>(response)
            }
            | (BodyMode::Required, _) => Err(eyre!(format!("OpenAI {action} request body is required"))),
            | (BodyMode::Empty, _) => {
                let params = Params::from_config(options).with_custom(options.params()).build();
                let response = endpoint.invoke(action, Some(params)).await;
                endpoint.handle_or::<R, Fallback<ErrorResponse>>(response)
            }
        },
        | Err(why) => Err(why),
    }
}
/// Retrieve all models available to the authenticated caller.
pub async fn models(options: &Options) -> ApiResult<ListModelsResponse> {
    invoke(options, "models", BodyMode::Empty).await
}
/// Create a response via `POST /responses`.
pub async fn response(options: &Options) -> ApiResult<Response> {
    invoke(options, "response", BodyMode::Required).await
}
