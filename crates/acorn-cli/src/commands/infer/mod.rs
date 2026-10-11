//! Local inference commands

use crate::{
    cli::arguments::{AcpAgentKind, InferenceArgs, InferenceBackendKind, InferenceEndpointMode},
    io::{read_stdin, write_stdout},
};
use acorn::{
    agent::{InferenceBackend, InferenceCapabilities, InferenceFuture, InferencePermissionPolicy, InferenceRequest, InferenceResult},
    analyzer::{
        fix::{FixOptions, FixReport},
        host::service::AnalysisReport,
    },
    io::{
        acp::{self, AgentProfile, Codex, OpenCode},
        api::{
            openai::{Client as OpenAiClient, EndpointMode, Options},
            Configuration,
        },
        sidecar::{needle::Needle, Sidecar},
        ApiResult,
    },
};
use acorn_core::{time::Milliseconds, Noul};
use acorn_schema::validation::Validate;
use color_eyre::eyre::{eyre, Result};
use core::time::Duration;
use std::env::current_dir;

pub(crate) mod prompt;

#[derive(Debug)]
enum Backend {
    Acp(acp::Client),
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
    /// Run one checked inference request.
    pub(crate) async fn infer(&self, request: &InferenceRequest) -> Result<InferenceResult> {
        self.backend.infer(request).await
    }
    /// Apply backend-specific selectors to one transport-neutral request.
    pub(crate) fn request(&self, request: InferenceRequest) -> InferenceRequest {
        request
            .with_agent(self.agent.clone())
            .with_model(self.model.clone())
            .with_permission_policy(InferencePermissionPolicy::DenyMutation)
            .with_timeout(self.timeout)
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
            let process_config = match arguments.agent.unwrap_or_default() {
                | AcpAgentKind::Codex => Codex::process_config(arguments.agent_executable.clone(), working_directory),
                | AcpAgentKind::OpenCode => OpenCode::process_config(arguments.agent_executable.clone(), working_directory),
            };
            let config = process_config.with_timeout(Duration::from(timeout));
            let agent = config.agent_id().to_string();
            Self {
                agent: Some(agent),
                backend: Backend::Acp(acp::Client::new(config).with_offline(offline)),
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
fn resolve_prompt<ReadStdin>(prompt: Option<&str>, read: ReadStdin) -> ApiResult<String>
where
    ReadStdin: FnOnce() -> Option<String>,
{
    prompt
        .map(str::to_string)
        .or_else(read)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| eyre!("Provide a prompt argument or pipe a prompt through standard input"))
}
/// Run one bounded Needle inference request and print its native result as JSON.
pub async fn run(prompt: Option<&str>, custom_location: Option<&str>, threshold: Noul, max_rounds: usize, offline: bool) -> ApiResult<()> {
    match resolve_prompt(prompt, read_stdin) {
        | Err(why) => Err(why),
        | Ok(prompt) => {
            let configured = match Needle::start(custom_location, offline).await {
                | Ok(sidecar) => sidecar.with_confidence_threshold(threshold).with_max_rounds(max_rounds),
                | Err(why) => Err(why),
            };
            match configured {
                | Ok(sidecar) => match sidecar.infer(prompt).await {
                    | Ok(result) => serde_json::to_string_pretty(&result)
                        .map_err(|why| eyre!("Failed to serialize Needle inference result — {why}"))
                        .map(|json| write_stdout(format!("{json}\n"))),
                    | Err(why) => Err(why),
                },
                | Err(why) => Err(why),
            }
        }
    }
}

#[cfg(test)]
mod tests;
