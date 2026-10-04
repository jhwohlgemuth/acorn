//! Model Context Protocol (MCP) server backed by a canonical tool registry.
use super::tools::{ToolCallContext, ToolRegistry};
use crate::error::ApiResult;
use crate::io::database::schema::Table;
use crate::io::database::Database;
use acorn_schema::agent::tools::{ToolDefinition, ToolEffects};
use alloc::borrow::Cow;
use alloc::sync::Arc;
use color_eyre::eyre::eyre;
use rmcp::model::{
    self, CallToolRequestParams, CallToolResult, Implementation, JsonObject, ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerInfo,
    ToolAnnotations,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::stdio;
use rmcp::{ErrorData, ServerHandler, ServiceExt};
use serde_json::Value;
use std::path::PathBuf;

/// MCP server that exposes the same allowlisted tools used by local inference.
#[derive(Clone, Debug)]
pub struct Server {
    registry: ToolRegistry,
    context: ToolCallContext,
}
struct Tool(model::Tool);
impl From<Tool> for model::Tool {
    fn from(tool: Tool) -> Self {
        tool.0
    }
}
impl Server {
    /// Construct a server from ACORN's built-in tool registry
    pub fn acorn() -> ApiResult<Self> {
        Self::with_offline(false)
    }
    /// Construct a server with ACORN's built-in tools and offline policy
    pub fn with_offline(offline: bool) -> ApiResult<Self> {
        Self::with_policy(offline, false, None)
    }
    /// Construct a server with explicit runtime policy and persistence.
    pub fn with_policy(offline: bool, allow_mutation: bool, database: Option<Database<Table>>) -> ApiResult<Self> {
        ToolRegistry::with_database(database).map(|registry| Self {
            registry,
            context: ToolCallContext::mcp_with_policy(offline, allow_mutation, 1024 * 1024),
        })
    }
    /// Return MCP tool declarations in deterministic order
    pub fn tools(&self) -> ApiResult<Vec<model::Tool>> {
        Ok(self
            .registry
            .available_definitions(self.context)
            .into_iter()
            .map(Tool::from)
            .map(model::Tool::from)
            .collect())
    }
    pub(crate) async fn invoke(&self, name: &str, arguments: JsonObject) -> CallToolResult {
        let result = self.registry.call_with_context(name, Value::Object(arguments), self.context).await;
        match result {
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
                    .with_description("Allowlisted ACORN research-activity-data tools"),
            )
            .with_instructions("Use ACORN tools to inspect and validate research activity data.")
    }
    async fn list_tools(&self, _request: Option<PaginatedRequestParams>, _context: RequestContext<RoleServer>) -> Result<ListToolsResult, ErrorData> {
        self.tools()
            .map(ListToolsResult::with_all_items)
            .map_err(|why| ErrorData::internal_error(why.to_string(), None))
    }
    async fn call_tool(&self, request: CallToolRequestParams, _context: RequestContext<RoleServer>) -> Result<CallToolResult, ErrorData> {
        let name = request.name;
        let known = self
            .registry
            .available_definitions(self.context)
            .iter()
            .any(|definition| definition.name == name);
        match known {
            | true => Ok(self.invoke(&name, request.arguments.unwrap_or_default()).await),
            | false => Err(ErrorData::invalid_params(format!("Unknown or unavailable tool '{name}'"), None)),
        }
    }
    fn get_tool(&self, name: &str) -> Option<model::Tool> {
        self.registry
            .available_definitions(self.context)
            .into_iter()
            .find(|definition| definition.name == name)
            .map(Tool::from)
            .map(model::Tool::from)
    }
}
impl From<ToolDefinition> for Tool {
    fn from(definition: ToolDefinition) -> Self {
        let ToolDefinition {
            name,
            title,
            description,
            input_schema,
            output_schema,
            effects,
            ..
        } = definition;
        let input_schema = match input_schema {
            | Value::Object(schema) => schema,
            | _ => JsonObject::new(),
        };
        let output_schema = output_schema.and_then(|schema| match schema {
            | Value::Object(schema) => Some(schema),
            | _ => None,
        });
        let ToolEffects {
            read_only,
            destructive,
            idempotent,
            open_world,
        } = effects;
        let annotations = ToolAnnotations::new()
            .read_only(read_only)
            .destructive(destructive)
            .idempotent(idempotent)
            .open_world(open_world);
        let tool = model::Tool::new(Cow::Owned(name), Cow::Owned(description), Arc::new(input_schema))
            .with_title(title)
            .with_annotations(annotations);
        Self(match output_schema {
            | Some(schema) => tool.with_raw_output_schema(Arc::new(schema)),
            | None => tool,
        })
    }
}
/// Serve ACORN tools over MCP's standard-input/standard-output transport
pub async fn run(offline: bool, allow_mutation: bool, database_path: Option<PathBuf>) -> ApiResult<()> {
    let database = allow_mutation.then(|| Database::<Table>::from_path(database_path));
    match Server::with_policy(offline, allow_mutation, database) {
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
