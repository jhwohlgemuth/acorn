//! Non-secret declarations for explicitly configured capability executors.
use crate::io::{api::Endpoint, mcp::config::McpServerConfiguration};
use acorn_core::validation::ValidationError;
use acorn_core::OneOrMany;
use acorn_schema::validation::{is_identifier, Validate, ValidationReport};
use alloc::collections::BTreeMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// How an endpoint binding may send caller data beyond the local host.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EndpointDataPolicy {
    /// Permit only a loopback endpoint.
    #[default]
    LocalOnly,
    /// Permit a configured remote endpoint when caller policy also allows it.
    AllowRemote,
}
/// A configured provider selector. Values are names and aliases, never credentials.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", tag = "type")]
pub enum ExecutorConfiguration {
    /// One tool on a named outbound MCP server's allowlist.
    Mcp {
        /// Named outbound server alias.
        server: String,
        /// Tool name on the server's allowlist.
        tool: String,
    },
    /// A trusted installed ACP agent profile.
    Acp {
        /// Trusted ACP profile name.
        profile: String,
        /// Optional model selector.
        model: Option<String>,
    },
    /// A named ACRE endpoint and explicit model.
    Endpoint {
        /// Named ACRE endpoint.
        endpoint: String,
        /// Model hosted by the endpoint.
        model: String,
        /// Caller-data transfer policy.
        #[serde(default)]
        data_policy: EndpointDataPolicy,
    },
    /// A named local sidecar profile, verified at execution time.
    Sidecar {
        /// Named installed sidecar profile.
        profile: String,
    },
}
/// One stable binding of a capability contract to an explicit provider selector.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ExecutorBindingConfiguration {
    /// Binding ID unique across this application configuration.
    pub id: String,
    /// Namespaced capability ID implemented by this binding.
    pub capability_id: String,
    /// Provider selector; no discovery occurs while parsing this declaration.
    pub executor: ExecutorConfiguration,
}
impl ExecutorBindingConfiguration {
    /// Check references to configured MCP allowlists and named endpoints without I/O.
    pub fn validate_references(
        &self,
        mcp: &BTreeMap<String, McpServerConfiguration>,
        endpoints: Option<&OneOrMany<Endpoint>>,
    ) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::new();
        match &self.executor {
            | ExecutorConfiguration::Mcp { server, tool } => {
                if !mcp.contains_key(server) {
                    report.add(
                        "executor.server",
                        ValidationError::new("unknown").with_message("MCP server alias is not configured"),
                    );
                } else if !mcp.get(server).is_some_and(|config| config.allowed.contains_key(tool)) {
                    report.add(
                        "executor.tool",
                        ValidationError::new("not-allowed").with_message("MCP tool is not allowlisted"),
                    );
                }
            }
            | ExecutorConfiguration::Endpoint { endpoint, .. } => {
                let matches = endpoints.map_or(0, |configured| configured.iter().filter(|item| item.name == *endpoint).count());
                match matches {
                    | 0 => report.add(
                        "executor.endpoint",
                        ValidationError::new("unknown").with_message("Inference endpoint name is not configured"),
                    ),
                    | 1 => {}
                    | _ => report.add(
                        "executor.endpoint",
                        ValidationError::new("ambiguous").with_message("Inference endpoint name must identify exactly one configured endpoint"),
                    ),
                }
            }
            | _ => {}
        }
        report.finish()
    }
}
impl Validate for ExecutorBindingConfiguration {
    fn validate(&self) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::new();
        if !valid_id(&self.id) {
            report.add(
                "id",
                ValidationError::new("identifier").with_message("Executor binding ID must be a valid namespaced ID of at most 128 bytes"),
            );
        }
        if !valid_id(&self.capability_id) {
            report.add(
                "capabilityId",
                ValidationError::new("identifier").with_message("Capability ID must be a valid namespaced ID of at most 128 bytes"),
            );
        }
        match &self.executor {
            | ExecutorConfiguration::Mcp { server, tool } => {
                if !is_identifier(server) {
                    report.add(
                        "executor.server",
                        ValidationError::new("identifier").with_message("MCP server alias must be a valid identifier"),
                    );
                }
                if tool.trim().is_empty() {
                    report.add(
                        "executor.tool",
                        ValidationError::new("required").with_message("MCP tool name is required"),
                    );
                }
            }
            | ExecutorConfiguration::Acp { profile, model } => {
                if !matches!(profile.as_str(), "codex" | "opencode") {
                    report.add(
                        "executor.profile",
                        ValidationError::new("trusted-profile").with_message("ACP profile must be codex or opencode"),
                    );
                }
                if model.as_deref().is_some_and(|value| value.trim().is_empty()) {
                    report.add(
                        "executor.model",
                        ValidationError::new("required").with_message("ACP model cannot be empty"),
                    );
                }
            }
            | ExecutorConfiguration::Endpoint { endpoint, model, .. } => {
                if !is_identifier(endpoint) {
                    report.add(
                        "executor.endpoint",
                        ValidationError::new("identifier").with_message("Endpoint name must be a valid identifier"),
                    );
                }
                if model.trim().is_empty() {
                    report.add(
                        "executor.model",
                        ValidationError::new("required").with_message("Endpoint model is required"),
                    );
                }
            }
            | ExecutorConfiguration::Sidecar { profile } => {
                if !is_identifier(profile) {
                    report.add(
                        "executor.profile",
                        ValidationError::new("identifier").with_message("Sidecar profile must be a valid identifier"),
                    );
                }
            }
        }
        report.finish()
    }
}
fn valid_id(value: &str) -> bool {
    value.len() <= 128 && value.split(':').all(is_identifier)
}
