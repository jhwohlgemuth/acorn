//! Local inference commands
use crate::{
    cli::arguments::{AcpAgentKind, Infer, InferenceArgs, InferenceBackendKind, InferenceEndpointMode},
    io::{read_stdin, write_stdout},
};
use acorn::util::constants::app::{APPLICATION, DEFAULT_NEEDLE_CONFIDENCE_THRESHOLD, DEFAULT_NEEDLE_MAX_ROUNDS};
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
        mcp::{
            BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, CapabilitySelectionContext, ExecutionContext,
            MutationAuthority, ToolRegistry,
        },
        sidecar::{needle::Needle, Sidecar},
        ApiResult,
    },
};
use acorn_core::util::SemanticVersion;
use acorn_core::{time::Milliseconds, Noul};
use acorn_schema::agent::tools::ToolEffects;
use acorn_schema::validation::Validate;
use color_eyre::eyre::{eyre, Result};
use core::time::Duration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env::current_dir;

mod chat;
mod metadata;
pub(crate) mod prompt;

const OLLAMA_LOCAL_URL: &str = "http://localhost:11434";

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
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct VersionInput {}
#[derive(JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
struct VersionOutput {
    version: String,
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
                    | InferenceBackendKind::Ollama | InferenceBackendKind::Openai => Self::openai(arguments, offline, timeout),
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
        let ollama = arguments.backend == InferenceBackendKind::Ollama;
        let backend_name = if ollama { "ollama" } else { "openai" };
        arguments
            .model
            .as_ref()
            .filter(|model| !model.trim().is_empty())
            .ok_or_else(|| eyre!("--model is required with --backend {backend_name}"))
            .map(|model| {
                let options = endpoint_options(arguments);
                let mode = endpoint_mode(arguments);
                let client = if ollama {
                    OpenAiClient::ollama(options, mode, model)
                } else {
                    OpenAiClient::new(options, mode, model)
                };
                Self {
                    agent: None,
                    backend: Backend::OpenAi(client.with_offline(offline)),
                    model: Some(model.clone()),
                    timeout,
                }
            })
    }
}
fn endpoint_mode(arguments: &InferenceArgs) -> EndpointMode {
    match arguments.endpoint_mode.unwrap_or(if arguments.backend == InferenceBackendKind::Ollama {
        InferenceEndpointMode::ChatCompletions
    } else {
        InferenceEndpointMode::Responses
    }) {
        | InferenceEndpointMode::ChatCompletions => EndpointMode::ChatCompletions,
        | InferenceEndpointMode::Responses => EndpointMode::Responses,
    }
}
fn endpoint_options(arguments: &InferenceArgs) -> Options {
    match arguments.backend {
        | InferenceBackendKind::Ollama => match arguments.base_url.as_deref() {
            | Some(base_url) => Options::from_env().with_domain(base_url),
            | None => Options::default().with_domain(OLLAMA_LOCAL_URL),
        },
        | _ => match arguments.base_url.as_deref() {
            | Some(base_url) => Options::from_env().with_domain(base_url),
            | None => Options::from_env(),
        },
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
fn default_threshold() -> Noul {
    DEFAULT_NEEDLE_CONFIDENCE_THRESHOLD
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
fn routing_registry(offline: bool) -> ApiResult<ToolRegistry> {
    let effects = ToolEffects {
        read_only: true,
        destructive: false,
        idempotent: true,
        open_world: false,
    };
    ToolRegistry::default()
        .register_capability(CapabilityDefinition::new::<VersionInput, VersionOutput>(
            format!("{APPLICATION}.version"),
            SemanticVersion::from((1, 0, 0)),
            "Return the version of the running ACORN library.",
            effects,
        ))
        .and_then(|registry| {
            registry.register_capability_binding::<VersionInput, VersionOutput, _, _>(
                BindingDefinition::new::<VersionInput, VersionOutput>(
                    format!("native:{APPLICATION}.version"),
                    format!("{APPLICATION}.version"),
                    SemanticVersion::from((1, 0, 0)),
                    BindingKind::Native,
                    effects,
                    BindingAvailability::Ready,
                ),
                |_: VersionInput| async {
                    Ok(VersionOutput {
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    })
                },
            )
        })
        .and_then(|registry| {
            current_dir()
                .map_err(Into::into)
                .and_then(|root| registry.register_analysis_capability(root, offline))
        })
}
/// Run one bounded Needle inference request and print its native result as JSON.
pub(crate) async fn run(arguments: &Infer, offline: bool) -> ApiResult<()> {
    if arguments.interactive {
        chat::run(arguments, offline).await
    } else if arguments.prompt.is_some() && read_stdin().is_some_and(|piped| !piped.trim().is_empty()) {
        Err(eyre!("Provide a positional prompt or piped input, not both"))
    } else if arguments.route {
        run_routed(arguments, offline).await
    } else {
        tokio::select! {
            result = run_inference(arguments.prompt.as_deref(), arguments.custom_location.as_deref(), arguments.threshold.unwrap_or_else(default_threshold), arguments.max_rounds.unwrap_or(DEFAULT_NEEDLE_MAX_ROUNDS), offline) => result,
            signal = shutdown_signal() => signal.and(Err(eyre!("Needle inference cancelled"))),
        }
    }
}
async fn run_inference(prompt: Option<&str>, custom_location: Option<&str>, threshold: Noul, max_rounds: usize, offline: bool) -> ApiResult<()> {
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
async fn run_routed(arguments: &Infer, offline: bool) -> ApiResult<()> {
    let prepared = if arguments.max_rounds.is_some() {
        Err(eyre!("--max-rounds applies only to the legacy Needle loop"))
    } else {
        resolve_prompt(arguments.prompt.as_deref(), read_stdin)
            .and_then(|prompt| structured_input(arguments.input_json.as_deref()).map(|caller_arguments| (prompt, caller_arguments)))
            .and_then(|(prompt, caller_arguments)| routing_registry(offline).map(|registry| (prompt, caller_arguments, registry)))
            .and_then(|(prompt, caller_arguments, registry)| {
                let scope = arguments.capability.as_ref().map(|id| vec![id.clone()]);
                registry
                    .capability_catalog(
                        scope.as_deref(),
                        CapabilitySelectionContext {
                            offline,
                            allow_remote: false,
                        },
                    )
                    .map(|_| (prompt, caller_arguments, registry, scope))
            })
            .and_then(|(prompt, caller_arguments, registry, scope)| {
                let mutation = if arguments.apply {
                    let grant = (
                        arguments.capability.as_ref(),
                        arguments.binding.as_ref(),
                        caller_arguments.as_ref(),
                        arguments.destination.as_ref(),
                    );
                    match grant {
                        | (Some(capability_id), Some(binding_id), Some(arguments_value), Some(destination)) => {
                            if arguments_value["destination"] == destination.as_str() {
                                Ok(Some(MutationAuthority {
                                    capability_id: capability_id.clone(),
                                    binding_id: binding_id.clone(),
                                    arguments: arguments_value.clone(),
                                    destination: destination.clone(),
                                }))
                            } else {
                                Err(eyre!("--destination must exactly match --input-json destination"))
                            }
                        }
                        | _ => Err(eyre!("--apply requires --capability, --binding, --input-json, and --destination")),
                    }
                } else {
                    Ok(None)
                };
                mutation.map(|mutation| (prompt, caller_arguments, registry, scope, mutation))
            })
    };
    match prepared {
        | Err(why) => Err(why),
        | Ok((prompt, caller_arguments, registry, scope, mutation)) => {
            let prompt = match caller_arguments.as_ref() {
                | Some(value) => format!("{prompt}\n\nCaller-supplied structured arguments (copy exactly): {value}"),
                | None => prompt,
            };
            let context = ExecutionContext {
                selection: CapabilitySelectionContext {
                    offline,
                    allow_remote: false,
                },
                preview: arguments.route_preview,
                binding_override: arguments.binding.clone(),
                caller_arguments,
                mutation,
                ..ExecutionContext::default()
            };
            match Needle::start_with_scoped_registry(arguments.custom_location.as_deref(), offline, registry, scope).await {
                | Err(why) => Err(why),
                | Ok(sidecar) => sidecar
                    .with_confidence_threshold(arguments.threshold.unwrap_or_else(default_threshold))
                    .route_once(prompt, context)
                    .await
                    .and_then(|result| {
                        serde_json::to_string_pretty(&result).map_err(|why| eyre!("Failed to serialize routed inference result — {why}"))
                    })
                    .map(|json| write_stdout(format!("{json}\n"))),
            }
        }
    }
}
async fn shutdown_signal() -> ApiResult<()> {
    #[cfg(unix)]
    {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            | Ok(mut terminate) => tokio::select! {
                signal = tokio::signal::ctrl_c() => signal.map_err(Into::into),
                _ = terminate.recv() => Ok(()),
            },
            | Err(why) => Err(why.into()),
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await.map_err(Into::into)
    }
}
fn structured_input(raw: Option<&str>) -> ApiResult<Option<Value>> {
    raw.map(|raw| {
        serde_json::from_str::<Value>(raw)
            .map_err(|why| eyre!("Invalid --input-json value — {why}"))
            .and_then(|value| {
                if value.is_object() {
                    Ok(value)
                } else {
                    Err(eyre!("--input-json must be a JSON object"))
                }
            })
    })
    .transpose()
}
#[cfg(test)]
mod tests;
