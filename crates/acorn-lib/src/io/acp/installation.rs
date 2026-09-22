//! Read-only ACP agent installation preflight.
use super::{AgentInstallation, AgentProcessConfig, Error};
use acorn_cmd::{find_executable, CommandOutput, CommandSpec};
use acorn_core::{util::SemanticVersion, validation::ValidationError};
use acorn_schema::validation::{Validate, ValidationReport};
use core::time::Duration;
use std::{fs::canonicalize, io::ErrorKind, path::PathBuf};

const VERSION_PROBE_LIMIT: Duration = Duration::from_secs(10);

impl AgentProcessConfig {
    /// Resolve the configured executable and run its version command.
    pub fn check_installation(&self) -> Result<AgentInstallation, Error> {
        self.validate()
            .map_err(|report| Error::InvalidConfiguration {
                agent_id: self.agent_id.clone(),
                message: report.to_string(),
            })
            .and_then(|()| self.resolve_executable())
            .and_then(|executable| self.probe_version(executable))
    }
    fn probe_version(&self, executable: PathBuf) -> Result<AgentInstallation, Error> {
        let timeout = self.timeout.min(VERSION_PROBE_LIMIT);
        let command = self.environment.iter().fold(
            CommandSpec::new(executable.as_os_str())
                .args(self.version_arguments.iter().cloned())
                .current_dir(self.working_directory.clone()),
            |command, (key, value)| command.env(key.clone(), value.clone()),
        );
        match command.output_with_timeout(timeout) {
            | Err(why) if why.kind() == ErrorKind::TimedOut => Err(Error::VersionProbeTimeout {
                agent_id: self.agent_id.clone(),
                timeout,
            }),
            | Err(why) => Err(Error::VersionProbe {
                agent_id: self.agent_id.clone(),
                message: why.to_string(),
            }),
            | Ok(output) if !output.status.success() => Err(Error::VersionProbe {
                agent_id: self.agent_id.clone(),
                message: match output.stderr().trim().is_empty() {
                    | true => format!("command exited with status {}", output.status),
                    | false => format!("command exited with status {} — {}", output.status, output.stderr()),
                },
            }),
            | Ok(output) => {
                let version_raw = [output.stdout(), output.stderr()]
                    .into_iter()
                    .filter(|value| !value.trim().is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
                match version_raw.is_empty() {
                    | true => Err(Error::VersionProbe {
                        agent_id: self.agent_id.clone(),
                        message: "version command emitted no output".to_string(),
                    }),
                    | false => Ok(AgentInstallation {
                        agent_id: self.agent_id.clone(),
                        executable,
                        version: version_raw
                            .chars()
                            .any(|character| character.is_ascii_digit())
                            .then(|| SemanticVersion::from_string(&version_raw)),
                        version_raw,
                    }),
                }
            }
        }
    }
    fn resolve_executable(&self) -> Result<PathBuf, Error> {
        match find_executable(self.executable.as_os_str()) {
            | Some(executable) => Ok(canonicalize(&executable).unwrap_or(executable)),
            | None if (self.executable.is_absolute() || self.executable.components().count() > 1) && self.executable.exists() => {
                Err(Error::ExecutableNotUsable {
                    agent_id: self.agent_id.clone(),
                    executable: self.executable.clone(),
                })
            }
            | None => Err(Error::ExecutableNotFound {
                agent_id: self.agent_id.clone(),
                executable: self.executable.clone(),
            }),
        }
    }
}
impl Validate for AgentProcessConfig {
    fn validate(&self) -> Result<(), ValidationReport> {
        let agent_id_empty = self.agent_id.trim().is_empty();
        let timeout_zero = self.timeout.is_zero();
        let version_arguments_empty = self.version_arguments.is_empty();
        let working_directory_exists = self.working_directory.is_dir();
        match (agent_id_empty, timeout_zero, version_arguments_empty, working_directory_exists) {
            | (true, _, _, _) => Err(ValidationReport::from_error(
                "agent_id",
                ValidationError::new("configuration").with_message("Agent identifier cannot be empty"),
            )),
            | (_, true, _, _) => Err(ValidationReport::from_error(
                "timeout",
                ValidationError::new("configuration").with_message("Timeout must be greater than zero"),
            )),
            | (_, _, true, _) => Err(ValidationReport::from_error(
                "version_arguments",
                ValidationError::new("configuration").with_message("Version arguments cannot be empty"),
            )),
            | (_, _, _, false) => Err(ValidationReport::from_error(
                "working_directory",
                ValidationError::new("configuration").with_message(format!("Working directory does not exist: {}", self.working_directory.display())),
            )),
            | (false, false, false, true) => Ok(()),
        }
    }
}
