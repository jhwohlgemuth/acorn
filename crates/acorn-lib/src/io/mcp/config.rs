//! Declarative allowlist and transport configuration for outbound MCP servers.
use crate::io::api::operation::{OperationEffect, OperationEffects};
use acorn_core::time::{Seconds, UnitOfTime};
use acorn_core::validation::ValidationError;
use acorn_schema::hardware::Memory;
use acorn_schema::validation::{Validate, ValidationReport};
use alloc::collections::{BTreeMap, BTreeSet};
use core::fmt;
use core::net::IpAddr;
use core::str::FromStr;
use http::Uri;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
/// Transport used to communicate with one outbound MCP server.
#[derive(Clone, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", tag = "type")]
pub enum McpTransportConfiguration {
    /// Spawn a configured executable directly and communicate over standard I/O.
    ChildProcess {
        /// Fixed arguments supplied to the executable.
        #[serde(default)]
        args: Vec<String>,
        /// Executable path or name resolved without a shell.
        command: String,
        /// Child variable names mapped to parent environment-variable names.
        #[serde(default)]
        environment: BTreeMap<String, String>,
    },
    /// Connect to an MCP Streamable HTTP endpoint.
    StreamableHttp {
        /// Optional environment variable containing a bearer token.
        #[serde(default)]
        bearer_token_env: Option<String>,
        /// HTTPS endpoint, or an HTTP loopback endpoint for local development.
        url: String,
    },
}
/// Configuration for one named outbound MCP server.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct McpServerConfiguration {
    /// Tools ACORN may discover and call on this server.
    pub allowed: BTreeMap<String, McpToolConfiguration>,
    /// Configured transport used to start a fresh session for each operation.
    pub transport: McpTransportConfiguration,
}
/// Policy for one allowlisted remote MCP tool.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct McpToolConfiguration {
    /// Effects enforced locally before transport startup.
    #[serde(default)]
    pub effects: Vec<OperationEffect>,
    /// Maximum serialized complete MCP result size.
    #[serde(default = "default_max_output")]
    pub max_output: Memory,
    /// Per-call timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout: Seconds,
}
impl McpServerConfiguration {
    pub(crate) fn validate_name(name: &str) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::new();
        if name.trim().is_empty() {
            report.add("name", ValidationError::new("required").with_message("MCP server name must not be empty"));
        }
        report.finish()
    }
}
impl Validate for McpServerConfiguration {
    fn validate(&self) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::new();
        if self.allowed.is_empty() {
            report.add(
                "allowed",
                ValidationError::new("required").with_message("At least one MCP tool must be allowlisted"),
            );
        }
        if let Err(errors) = self.transport.validate() {
            report.append_prefixed("transport", errors);
        }
        self.allowed.iter().for_each(|(tool_name, tool)| {
            let prefix = format!("allowed.{tool_name}");
            if tool_name.trim().is_empty() {
                report.add(
                    format!("{prefix}.name"),
                    ValidationError::new("required").with_message("MCP tool name must not be empty"),
                );
            }
            if let Err(errors) = tool.validate() {
                report.append_prefixed(&prefix, errors);
            }
        });
        report.finish()
    }
}
impl McpToolConfiguration {
    /// Return the normalized effects enforced for this tool.
    pub fn operation_effects(&self) -> OperationEffects {
        self.effects.iter().copied().collect()
    }
}
impl Validate for McpToolConfiguration {
    fn validate(&self) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::new();
        let mut unique = BTreeSet::new();
        self.effects.iter().enumerate().for_each(|(index, effect)| {
            if !unique.insert(effect) {
                report.add(
                    format!("effects[{index}]"),
                    ValidationError::new("duplicate").with_message("MCP tool effects must not contain duplicates"),
                );
            }
        });
        if self
            .max_output
            .checked_bytes()
            .and_then(|bytes| usize::try_from(bytes).ok())
            .is_none_or(|bytes| bytes == 0)
        {
            report.add(
                "max_output",
                ValidationError::new("range").with_message("MCP tool output limit must be greater than zero and fit this platform"),
            );
        }
        if self.timeout.is_zero() {
            report.add(
                "timeout",
                ValidationError::new("range").with_message("MCP tool timeout must be greater than zero"),
            );
        }
        report.finish()
    }
}
impl fmt::Debug for McpTransportConfiguration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::ChildProcess { args, environment, .. } => formatter
                .debug_struct("ChildProcess")
                .field("argument_count", &args.len())
                .field("environment_count", &environment.len())
                .finish(),
            | Self::StreamableHttp { bearer_token_env, .. } => formatter
                .debug_struct("StreamableHttp")
                .field("bearer_token_configured", &bearer_token_env.is_some())
                .finish(),
        }
    }
}
impl Validate for McpTransportConfiguration {
    fn validate(&self) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::new();
        match self {
            | Self::ChildProcess { command, environment, .. } => {
                if command.trim().is_empty() {
                    report.add(
                        "command",
                        ValidationError::new("required").with_message("MCP child command must not be empty"),
                    );
                }
                environment.iter().for_each(|(child, parent)| {
                    if !is_environment_name(child) || !is_environment_name(parent) {
                        report.add(
                            "environment",
                            ValidationError::new("environment-variable")
                                .with_message("MCP child environment mappings must contain environment-variable names"),
                        );
                    }
                });
            }
            | Self::StreamableHttp { bearer_token_env, url } => {
                if bearer_token_env.as_deref().is_some_and(|name| !is_environment_name(name)) {
                    report.add(
                        "bearer_token_env",
                        ValidationError::new("environment-variable").with_message("MCP bearer token configuration must name an environment variable"),
                    );
                }
                if !is_secure_http_url(url) {
                    report.add(
                        "url",
                        ValidationError::new("url").with_message("MCP HTTP endpoints must use HTTPS, except for HTTP loopback endpoints"),
                    );
                }
            }
        }
        report.finish()
    }
}
fn default_max_output() -> Memory {
    Memory::mb(1)
}
fn default_timeout() -> Seconds {
    Seconds::new(DEFAULT_TIMEOUT_SECONDS)
}
fn is_environment_name(value: &str) -> bool {
    let mut characters = value.chars();
    characters.next().is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}
fn is_loopback_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost") || IpAddr::from_str(host.trim_matches(['[', ']'])).is_ok_and(|address| address.is_loopback())
}
fn is_secure_http_url(value: &str) -> bool {
    Uri::from_str(value).is_ok_and(|uri| {
        let scheme = uri.scheme_str();
        let host = uri.host();
        let has_userinfo = uri.authority().is_some_and(|authority| authority.as_str().contains('@'));
        !has_userinfo
            && host.is_some()
            && match scheme {
                | Some("https") => true,
                | Some("http") => host.is_some_and(is_loopback_host),
                | _ => false,
            }
    })
}
