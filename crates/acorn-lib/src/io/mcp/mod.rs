//! Model Context Protocol server backed by ACORN's canonical tool registry.

use crate::error::ApiResult;
use crate::tools::{ToolCallContext, ToolRegistry};
use acorn_schema::schema::agent::tools::ToolDefinition;
use alloc::borrow::Cow;
use alloc::sync::Arc;
use color_eyre::eyre::eyre;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Implementation, JsonObject, ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool,
    ToolAnnotations,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::stdio;
use rmcp::{ErrorData as McpError, ServerHandler, ServiceExt};
use serde_json::Value;

/// MCP server that exposes the same allowlisted tools used by local inference.
#[derive(Clone, Debug)]
pub struct Server {
    registry: ToolRegistry,
    offline: bool,
}

impl Server {
    /// Construct a server from ACORN's built-in tool registry.
    pub fn acorn() -> ApiResult<Self> {
        Self::with_offline(false)
    }

    /// Construct a server with ACORN's built-in tools and offline policy.
    pub fn with_offline(offline: bool) -> ApiResult<Self> {
        ToolRegistry::acorn().map(|registry| Self { registry, offline })
    }

    /// Return MCP tool declarations in deterministic order.
    pub fn tools(&self) -> ApiResult<Vec<Tool>> {
        self.registry
            .definitions()
            .into_iter()
            .filter(|definition| definition.exposure.mcp)
            .map(tool_from_definition)
            .collect()
    }

    fn invoke(&self, name: &str, arguments: JsonObject) -> CallToolResult {
        match self
            .registry
            .call_with_context(name, Value::Object(arguments), ToolCallContext::mcp(self.offline))
        {
            | Ok(result) => CallToolResult::structured(result.structured_content),
            | Err(why) => CallToolResult::structured_error(serde_json::json!({ "message": why.to_string() })),
        }
    }
}

impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new("acorn", env!("CARGO_PKG_VERSION"))
                    .with_title("ACORN")
                    .with_description("Allowlisted ACORN research-data tools"),
            )
            .with_instructions("Use ACORN tools to inspect and validate research activity data.")
    }

    async fn list_tools(&self, _request: Option<PaginatedRequestParams>, _context: RequestContext<RoleServer>) -> Result<ListToolsResult, McpError> {
        self.tools()
            .map(ListToolsResult::with_all_items)
            .map_err(|why| McpError::internal_error(why.to_string(), None))
    }

    async fn call_tool(&self, request: CallToolRequestParams, _context: RequestContext<RoleServer>) -> Result<CallToolResult, McpError> {
        let name = request.name;
        let known = self
            .registry
            .definitions()
            .iter()
            .any(|definition| definition.name == name && definition.exposure.mcp);
        match known {
            | true => Ok(self.invoke(&name, request.arguments.unwrap_or_default())),
            | false => Err(McpError::invalid_params(format!("Unknown or unavailable tool '{name}'"), None)),
        }
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.registry
            .definitions()
            .into_iter()
            .find(|definition| definition.name == name && definition.exposure.mcp)
            .and_then(|definition| tool_from_definition(definition).ok())
    }
}

/// Serve ACORN tools over MCP's standard-input/standard-output transport.
pub async fn run(offline: bool) -> ApiResult<()> {
    match Server::with_offline(offline) {
        | Ok(server) => match server.serve(stdio()).await {
            | Ok(service) => service
                .waiting()
                .await
                .map(|_| ())
                .map_err(|why| eyre!("ACORN MCP server stopped with an error — {why}")),
            | Err(why) => Err(eyre!("Failed to start ACORN MCP server — {why}")),
        },
        | Err(why) => Err(why),
    }
}

fn tool_from_definition(definition: ToolDefinition) -> ApiResult<Tool> {
    let input_schema = object_schema(&definition.name, "input", definition.input_schema)?;
    let output_schema = definition
        .output_schema
        .map(|schema| object_schema(&definition.name, "output", schema))
        .transpose()?;
    let annotations = ToolAnnotations::new()
        .read_only(definition.effects.read_only)
        .destructive(definition.effects.destructive)
        .idempotent(definition.effects.idempotent)
        .open_world(definition.effects.open_world);
    let tool = Tool::new(Cow::Owned(definition.name), Cow::Owned(definition.description), Arc::new(input_schema))
        .with_title(definition.title)
        .with_annotations(annotations);
    Ok(match output_schema {
        | Some(schema) => tool.with_raw_output_schema(Arc::new(schema)),
        | None => tool,
    })
}

fn object_schema(name: &str, kind: &str, schema: Value) -> ApiResult<JsonObject> {
    match schema {
        | Value::Object(object) => Ok(object),
        | _ => Err(eyre!("Tool '{name}' {kind} schema must be a JSON object")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_catalog_uses_the_canonical_registry() {
        let server = Server::acorn().expect("server");
        let tools = server.tools().expect("MCP tools");
        let registry = ToolRegistry::acorn().expect("registry");
        let definitions = registry
            .definitions()
            .into_iter()
            .filter(|definition| definition.exposure.mcp)
            .collect::<Vec<_>>();
        assert_eq!(tools.len(), definitions.len());
        tools.iter().zip(definitions).for_each(|(tool, definition)| {
            assert_eq!(tool.name, definition.name);
            assert_eq!(tool.description.as_deref(), Some(definition.description.as_str()));
            assert_eq!(Value::Object(tool.input_schema.as_ref().clone()), definition.input_schema);
        });
    }

    #[test]
    fn mcp_invocation_dispatches_through_the_shared_handler() {
        let server = Server::acorn().expect("server");
        let result = server.invoke("acorn.version", JsonObject::new());
        assert_eq!(
            result
                .structured_content
                .expect("structured result")
                .get("version")
                .and_then(Value::as_str),
            Some(env!("CARGO_PKG_VERSION"))
        );
        let mut invalid = JsonObject::new();
        invalid.insert("unexpected".to_string(), Value::Bool(true));
        assert_eq!(server.invoke("acorn.version", invalid).is_error, Some(true));
    }
}
