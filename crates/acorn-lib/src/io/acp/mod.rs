//! Agent Client Protocol process, readiness, and inference adapter.
use crate::agent::{InferenceCapabilities, InferencePermissionPolicy};
use acorn_cmd::args;
use acorn_core::util::SemanticVersion;
use acorn_macros::With;
use acorn_schema::software::Software;
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use bon::Builder;
pub use client::Client;
use core::time::Duration;
use schemars::JsonSchema;
use serde::Serialize;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

mod client;
mod installation;

/// Static launch profile implemented by a supported ACP agent.
pub trait AgentProfile: Sized {
    /// Arguments used to start the ACP transport.
    const ACP_ARGUMENTS: &'static [&'static str];
    /// Software identity used as the stable agent identifier.
    const AGENT: Software;
    /// Environment overrides supplied to the ACP process.
    const ENVIRONMENT: &'static [(&'static str, &'static str)] = &[];
    /// Software identity used as the default executable.
    const EXECUTABLE: Software;
    /// Guidance appended when the configured executable cannot be found.
    const MISSING_EXECUTABLE_HELP: &'static str = "";
    /// Arguments used by the read-only installation version probe.
    const VERSION_ARGUMENTS: &'static [&'static str] = &["--version"];
    /// Build immutable process configuration from this launch profile.
    fn process_config(executable: Option<PathBuf>, working_directory: PathBuf) -> AgentProcessConfig {
        let config = AgentProcessConfig::new(
            Self::AGENT.command(),
            executable.unwrap_or_else(|| PathBuf::from(Self::EXECUTABLE.command())),
            working_directory,
        );
        config
            .with_acp_arguments(Self::ACP_ARGUMENTS.iter().copied())
            .with_environment(Self::ENVIRONMENT.iter().copied())
            .with_missing_executable_help(Self::MISSING_EXECUTABLE_HELP)
            .with_version_arguments(Self::VERSION_ARGUMENTS.iter().copied())
    }
}
/// Read-only observer for normalized ACP session events.
///
/// Observers receive progress metadata only. They cannot change ACORN's
/// permission decision, which remains governed by [`InferencePermissionPolicy`].
pub trait SessionObserver: Send + Sync {
    /// Observe one normalized session event.
    fn observe(&self, event: &AcpSessionEvent);
}
/// Normalized, transport-independent event emitted during an ACP session.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "type")]
pub enum AcpSessionEvent {
    /// A bounded fragment of assistant text was received.
    TextDelta {
        /// Text appended to the final inference response.
        text: String,
    },
    /// The agent requested permission and ACORN applied its fixed policy.
    PermissionRequested {
        /// Number of permission choices advertised by the agent.
        option_count: usize,
    },
}
/// Typed failure returned by ACP installation, readiness, and inference operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The request selected a different agent from the configured process.
    #[error("ACP agent '{requested}' was requested, but '{configured}' is configured")]
    AgentMismatch {
        /// Configured process agent identifier.
        configured: String,
        /// Agent identifier requested by the caller.
        requested: String,
    },
    /// The agent process exited before the requested operation completed.
    #[error("ACP agent '{agent_id}' exited before completion — {message}")]
    ChildExit {
        /// Agent identifier.
        agent_id: String,
        /// Bounded SDK diagnostic.
        message: String,
    },
    /// The configured executable could not be resolved.
    #[error("ACP agent '{agent_id}' executable was not found: {}{missing_executable_help}", .executable.display())]
    ExecutableNotFound {
        /// Agent identifier.
        agent_id: String,
        /// Configured executable or command name.
        executable: PathBuf,
        /// Agent-specific installation guidance, including its leading separator.
        missing_executable_help: &'static str,
    },
    /// An explicit executable path exists but cannot be executed.
    #[error("ACP agent '{agent_id}' executable is not usable: {}", .executable.display())]
    ExecutableNotUsable {
        /// Agent identifier.
        agent_id: String,
        /// Configured executable path.
        executable: PathBuf,
    },
    /// The immutable process descriptor cannot be represented by the ACP SDK.
    #[error("ACP agent '{agent_id}' configuration is invalid — {message}")]
    InvalidConfiguration {
        /// Agent identifier.
        agent_id: String,
        /// Safe configuration diagnostic.
        message: String,
    },
    /// The requested model is not advertised by the initialized session.
    #[error("ACP agent '{agent_id}' did not advertise requested model '{model}'")]
    ModelUnavailable {
        /// Agent identifier.
        agent_id: String,
        /// Requested model identifier.
        model: String,
    },
    /// ACP initialization or capability negotiation failed.
    #[error("ACP agent '{agent_id}' initialization failed — {message}")]
    Negotiation {
        /// Agent identifier.
        agent_id: String,
        /// Bounded SDK diagnostic.
        message: String,
    },
    /// ACP process inference was rejected by offline policy.
    #[error("ACP inference through '{agent_id}' is unavailable in offline mode")]
    Offline {
        /// Agent identifier.
        agent_id: String,
    },
    /// A requested tool permission was denied by the request policy.
    #[error("ACP agent '{agent_id}' requested a permission denied by policy")]
    PermissionDenied {
        /// Agent identifier.
        agent_id: String,
    },
    /// The ACP process could not be started or connected.
    #[error("ACP agent '{agent_id}' process failed — {message}")]
    Process {
        /// Agent identifier.
        agent_id: String,
        /// Bounded SDK diagnostic.
        message: String,
    },
    /// The initialized ACP connection failed while processing a prompt.
    #[error("ACP agent '{agent_id}' protocol operation failed — {message}")]
    Protocol {
        /// Agent identifier.
        agent_id: String,
        /// Bounded SDK diagnostic.
        message: String,
    },
    /// ACP session creation or configuration failed.
    #[error("ACP agent '{agent_id}' session failed — {message}")]
    Session {
        /// Agent identifier.
        agent_id: String,
        /// Bounded SDK diagnostic.
        message: String,
    },
    /// An ACP operation exceeded its configured timeout.
    #[error("ACP agent '{agent_id}' exceeded the {timeout:?} timeout")]
    Timeout {
        /// Agent identifier.
        agent_id: String,
        /// Timeout applied to the operation.
        timeout: Duration,
    },
    /// The version command failed or emitted no usable output.
    #[error("ACP agent '{agent_id}' version probe failed — {message}")]
    VersionProbe {
        /// Agent identifier.
        agent_id: String,
        /// Safe version-probe diagnostic.
        message: String,
    },
    /// The version command exceeded its configured timeout.
    #[error("ACP agent '{agent_id}' version probe exceeded {timeout:?}")]
    VersionProbeTimeout {
        /// Agent identifier.
        agent_id: String,
        /// Timeout applied to the version probe.
        timeout: Duration,
    },
}
/// Successful read-only installation and version preflight.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentInstallation {
    /// Stable configured agent identifier.
    pub agent_id: String,
    /// Resolved executable path used for subsequent startup.
    pub executable: PathBuf,
    /// Raw trimmed version output reported by the executable.
    pub version_raw: String,
    /// Parsed semantic version when the output includes a canonical numeric version.
    pub version: Option<SemanticVersion>,
}
/// Immutable, trusted launch data for one ACP agent process.
#[derive(Builder, Clone, Debug, Eq, PartialEq, With)]
pub struct AgentProcessConfig {
    /// Arguments used to start the ACP transport.
    #[builder(default)]
    #[with(skip)]
    acp_arguments: Vec<OsString>,
    /// Stable agent identifier used in requests and provenance.
    #[builder(into)]
    agent_id: String,
    /// Environment overrides supplied to the child process.
    #[builder(default)]
    #[with(skip)]
    environment: BTreeMap<OsString, OsString>,
    /// Executable path or command name.
    #[builder(into)]
    executable: PathBuf,
    /// Guidance appended when the configured executable cannot be found.
    #[builder(default)]
    missing_executable_help: &'static str,
    /// Maximum permission policy granted to the child process.
    #[builder(default = InferencePermissionPolicy::DenyMutation)]
    permission_policy: InferencePermissionPolicy,
    /// Maximum duration for readiness and inference operations.
    #[builder(default = Duration::from_secs(120))]
    timeout: Duration,
    /// Arguments used by the read-only installation version probe.
    #[builder(default = args!["--version"])]
    #[with(skip)]
    version_arguments: Vec<OsString>,
    /// Working directory supplied as the ACP session root.
    #[builder(into)]
    working_directory: PathBuf,
}
/// One authentication method advertised during ACP initialization.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuthenticationMethod {
    /// Stable authentication method identifier.
    pub id: String,
    /// Human-readable method name.
    pub name: String,
    /// Whether authentication requires a separately launched terminal flow.
    pub terminal: bool,
}
/// Codex ACP launch profile backed by the `codex-acp` adapter.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Codex;
/// OpenCode ACP launch profile.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OpenCode;
/// Result of an explicit ACP initialize handshake that sends no user prompt.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolReadiness {
    /// Agent implementation name reported by the peer.
    pub agent_name: Option<String>,
    /// Agent implementation version reported by the peer.
    pub agent_version: Option<String>,
    /// Authentication methods advertised by the peer.
    pub authentication_methods: Vec<AuthenticationMethod>,
    /// Negotiated transport-neutral inference capabilities.
    pub capabilities: InferenceCapabilities,
    /// Installation preflight result used for this handshake.
    pub installation: AgentInstallation,
    /// Negotiated ACP wire protocol version.
    pub protocol_version: u16,
}
impl AgentProcessConfig {
    /// Borrow the fixed ACP launch argument vector.
    pub fn acp_arguments(&self) -> &[OsString] {
        &self.acp_arguments
    }
    /// Borrow the stable agent identifier.
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }
    /// Create launch data for `codex-acp` and `codex-acp --version`.
    pub fn codex(executable: Option<PathBuf>, working_directory: PathBuf) -> Self {
        Codex::process_config(executable, working_directory)
    }
    /// Borrow the child environment overrides.
    pub fn environment(&self) -> &BTreeMap<OsString, OsString> {
        &self.environment
    }
    /// Borrow the executable path or command name.
    pub fn executable(&self) -> &Path {
        &self.executable
    }
    /// Create trusted launch data for an ACP agent.
    pub fn new(agent_id: impl Into<String>, executable: impl Into<PathBuf>, working_directory: impl Into<PathBuf>) -> Self {
        Self::builder()
            .agent_id(agent_id)
            .executable(executable)
            .working_directory(working_directory)
            .build()
    }
    /// Create launch data for `opencode acp` and `opencode --version`.
    pub fn opencode(executable: Option<PathBuf>, working_directory: PathBuf) -> Self {
        OpenCode::process_config(executable, working_directory)
    }
    /// Read the maximum permission policy granted to the child process.
    pub const fn permission_policy(&self) -> InferencePermissionPolicy {
        self.permission_policy
    }
    /// Read the maximum duration for readiness and installation operations.
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }
    /// Borrow the fixed version-probe argument vector.
    pub fn version_arguments(&self) -> &[OsString] {
        &self.version_arguments
    }
    /// Replace the fixed ACP launch argument vector.
    pub fn with_acp_arguments<I, S>(self, arguments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        Self {
            acp_arguments: arguments.into_iter().map(Into::into).collect(),
            ..self
        }
    }
    /// Replace the child environment overrides.
    pub fn with_environment<I, K, V>(self, environment: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<OsString>,
        V: Into<OsString>,
    {
        Self {
            environment: environment.into_iter().map(|(key, value)| (key.into(), value.into())).collect(),
            ..self
        }
    }
    /// Replace the fixed version-probe argument vector.
    pub fn with_version_arguments<I, S>(self, arguments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        Self {
            version_arguments: arguments.into_iter().map(Into::into).collect(),
            ..self
        }
    }
    /// Borrow the working directory supplied as the ACP session root.
    pub fn working_directory(&self) -> &Path {
        &self.working_directory
    }
}
impl AgentProfile for Codex {
    const ACP_ARGUMENTS: &'static [&'static str] = &[];
    const AGENT: Software = Software::Codex;
    const ENVIRONMENT: &'static [(&'static str, &'static str)] = &[("INITIAL_AGENT_MODE", "read-only")];
    const EXECUTABLE: Software = Software::CodexAcp;
    const MISSING_EXECUTABLE_HELP: &'static str = "\nInstall it with `npm install -g @agentclientprotocol/codex-acp` and ensure `codex-acp` is on PATH. The global install does not require `npx`. If npm is unavailable, install Node.js/npm or download a standalone binary from https://github.com/agentclientprotocol/codex-acp/releases.";
}
impl AgentProfile for OpenCode {
    const ACP_ARGUMENTS: &'static [&'static str] = &["acp"];
    const AGENT: Software = Software::OpenCode;
    const EXECUTABLE: Software = Software::OpenCode;
}

#[cfg(test)]
mod tests;
