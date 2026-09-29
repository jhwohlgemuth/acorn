//! Shared inference-backend configuration for CLI commands.

use crate::cli::arguments::{InferenceArgs, InferenceBackendKind, InferenceEndpointMode};
use acorn::{
    agent::{InferenceBackend, InferenceCapabilities, InferenceFuture, InferencePermissionPolicy, InferenceRequest},
    analyzer::{
        fix::{FixOptions, FixReport},
        host::service::AnalysisReport,
    },
    io::{
        acp::{AgentProcessConfig, Client as AcpClient, OPENCODE_AGENT_ID},
        api::{
            openai::{Client as OpenAiClient, EndpointMode, Options},
            Configuration,
        },
    },
};
use acorn_core::time::Milliseconds;
use acorn_schema::validation::Validate;
use color_eyre::eyre::{eyre, Result};
use core::time::Duration;
use std::env::current_dir;
#[derive(Debug)]
enum Backend {
    Acp(AcpClient),
    OpenAi(OpenAiClient),
}
/// Fully configured inference backend plus request defaults.
#[derive(Debug)]
pub(crate) struct ConfiguredInference {
    agent: Option<String>,
    backend: Backend,
    model: Option<String>,
    timeout: Milliseconds,
}
impl InferenceBackend for Backend {
    fn capabilities(&self) -> InferenceCapabilities {
        self.into()
    }
    fn infer<'a>(&'a self, request: &'a InferenceRequest) -> InferenceFuture<'a> {
        match self {
            | Self::Acp(client) => client.infer(request),
            | Self::OpenAi(client) => client.infer(request),
        }
    }
}
impl ConfiguredInference {
    /// Validate flags and construct the selected backend.
    pub(crate) fn from_args(arguments: &InferenceArgs, offline: bool) -> Result<Self> {
        arguments.validate().map_err(|why| eyre!(why)).and_then(|()| {
            Milliseconds::try_from(arguments.timeout)
                .map_err(|_| eyre!("--timeout is too large"))
                .and_then(|timeout| match arguments.backend {
                    | InferenceBackendKind::Acp => Self::acp(arguments, offline, timeout),
                    | InferenceBackendKind::Openai => Self::openai(arguments, offline, timeout),
                })
        })
    }
    /// Apply backend-specific selectors to one transport-neutral request.
    pub(crate) fn request(&self, mut request: InferenceRequest) -> InferenceRequest {
        request.agent = self.agent.clone();
        request.model = self.model.clone();
        request.permission_policy = InferencePermissionPolicy::DenyMutation;
        request.timeout = self.timeout;
        request
    }
    /// Run one checked inference request.
    pub(crate) async fn infer(&self, request: &InferenceRequest) -> Result<acorn::agent::InferenceResult> {
        self.backend.infer(request).await
    }
    /// Generate bounded proposals from one already-computed analysis report.
    pub(crate) async fn suggest_fixes(&self, report: &AnalysisReport) -> Result<FixReport> {
        report
            .suggest_fixes(
                &self.backend,
                &FixOptions {
                    agent: self.agent.clone(),
                    model: self.model.clone(),
                    timeout: self.timeout,
                    ..FixOptions::default()
                },
            )
            .await
    }
    fn acp(arguments: &InferenceArgs, offline: bool, timeout: Milliseconds) -> Result<Self> {
        let working_directory = arguments
            .working_directory
            .clone()
            .map(Ok)
            .unwrap_or_else(current_dir)
            .map_err(color_eyre::Report::new);
        working_directory.map(|working_directory| {
            let config = AgentProcessConfig::opencode(arguments.agent_executable.clone(), working_directory).with_timeout(Duration::from(timeout));
            Self {
                agent: Some(OPENCODE_AGENT_ID.to_string()),
                backend: Backend::Acp(AcpClient::new(config).with_offline(offline)),
                model: arguments.model.clone(),
                timeout,
            }
        })
    }
    fn openai(arguments: &InferenceArgs, offline: bool, timeout: Milliseconds) -> Result<Self> {
        arguments
            .model
            .as_ref()
            .filter(|model| !model.trim().is_empty())
            .ok_or_else(|| eyre!("--model is required with --backend openai"))
            .map(|model| {
                let options = arguments
                    .base_url
                    .as_ref()
                    .map_or_else(Options::from_env, |base_url| Options::from_env().with_domain(base_url));
                let mode = match arguments.endpoint_mode.unwrap_or_default() {
                    | InferenceEndpointMode::ChatCompletions => EndpointMode::ChatCompletions,
                    | InferenceEndpointMode::Responses => EndpointMode::Responses,
                };
                Self {
                    agent: None,
                    backend: Backend::OpenAi(OpenAiClient::new(options, mode, model).with_offline(offline)),
                    model: Some(model.clone()),
                    timeout,
                }
            })
    }
}
impl From<&Backend> for InferenceCapabilities {
    fn from(value: &Backend) -> Self {
        match value {
            | Backend::Acp(client) => client.capabilities(),
            | Backend::OpenAi(client) => client.capabilities(),
        }
    }
}
#[cfg(test)]
mod tests;
