//! Application configuration projection used by outbound MCP-only builds
use crate::io::mcp::config::McpServerConfiguration;
use acorn_schema::validation::{Validate, ValidationReport};
use alloc::collections::BTreeMap;
use schemars::JsonSchema;
use serde::Serialize;

/// Application configuration fields available without the native analysis stack.
///
/// Enabling `analysis` adds the remaining application fields at the same public path.
/// Unknown non-MCP fields are ignored when this projection is deserialized.
#[derive(Clone, Debug, Default, JsonSchema, Serialize, eserde::Deserialize)]
#[schemars(title = "Application Configuration for Research Enablement")]
pub struct ApplicationConfiguration {
    /// Named outbound MCP servers and their explicit tool allowlists.
    #[eserde(compat)]
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub mcp: BTreeMap<String, McpServerConfiguration>,
}
impl Validate for ApplicationConfiguration {
    fn validate(&self) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::new();
        self.mcp.iter().for_each(|(name, server)| {
            if let Err(errors) = McpServerConfiguration::validate_name(name) {
                report.append_prefixed(&format!("mcp.{name}"), errors);
            }
            if let Err(errors) = server.validate() {
                report.append_prefixed(&format!("mcp.{name}"), errors);
            }
        });
        report.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::ApplicationConfiguration;

    #[test]
    fn test_deserialization_ignores_fields_owned_by_other_capabilities() {
        let configuration = eserde::json::from_str::<ApplicationConfiguration>(r#"{"schemaVersion":"1.0.0","buckets":[],"mcp":{}}"#)
            .expect("MCP projection should ignore unrelated application fields");
        assert!(configuration.mcp.is_empty());
    }
}
