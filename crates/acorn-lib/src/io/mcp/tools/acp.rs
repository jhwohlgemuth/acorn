//! Routed bindings for explicitly trusted ACP agent profiles.
use super::{BindingAvailability, BindingKind, InferenceBackend, InferenceBindingOptions, ToolRegistry};
use crate::error::ApiResult;
use crate::io::acp::{AgentProcessConfig, Client};
use crate::io::config::ApplicationConfiguration;
use crate::io::mcp::ExecutorConfiguration;
use alloc::sync::Arc;
use color_eyre::eyre::eyre;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AcpProfile {
    Codex,
    OpenCode,
}
impl AcpProfile {
    fn id(self) -> &'static str {
        match self {
            | Self::Codex => "codex",
            | Self::OpenCode => "opencode",
        }
    }
    fn process_config(self, working_directory: PathBuf) -> AgentProcessConfig {
        match self {
            | Self::Codex => AgentProcessConfig::codex(None, working_directory),
            | Self::OpenCode => AgentProcessConfig::opencode(None, working_directory),
        }
    }
}
impl TryFrom<&str> for AcpProfile {
    type Error = color_eyre::Report;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            | "codex" => Ok(Self::Codex),
            | "opencode" => Ok(Self::OpenCode),
            | _ => Err(eyre!("Untrusted ACP profile '{value}'")),
        }
    }
}
impl ToolRegistry {
    /// Register explicitly configured trusted ACP profiles without starting an agent.
    pub fn register_acp_bindings(self, configuration: &ApplicationConfiguration, working_directory: PathBuf, offline: bool) -> ApiResult<Self> {
        self.register_acp_bindings_with(configuration, move |profile| {
            let process = profile.process_config(working_directory.clone());
            Ok(Arc::new(Client::new(process).with_offline(offline)) as Arc<dyn InferenceBackend>)
        })
    }
    pub(super) fn register_acp_bindings_with<F>(self, configuration: &ApplicationConfiguration, backend_for: F) -> ApiResult<Self>
    where
        F: Fn(AcpProfile) -> ApiResult<Arc<dyn InferenceBackend>>,
    {
        configuration
            .executors
            .iter()
            .filter_map(|configured| match &configured.executor {
                | ExecutorConfiguration::Acp { profile, model } => Some((configured, profile, model)),
                | _ => None,
            })
            .try_fold(self, |registry, (configured, profile, model)| {
                AcpProfile::try_from(profile.as_str()).and_then(|profile| {
                    backend_for(profile).and_then(|backend| {
                        registry.register_inference_binding(
                            configured,
                            backend,
                            InferenceBindingOptions {
                                agent: Some(profile.id().to_string()),
                                model: model.clone(),
                                kind: BindingKind::Local,
                                availability: BindingAvailability::Ready,
                                offline_safe: false,
                            },
                        )
                    })
                })
            })
    }
}
