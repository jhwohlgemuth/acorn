//! Capability bindings for explicitly configured outbound MCP tools.
use super::{BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, CapabilityHandler, ToolCancellation, ToolRegistry};
use crate::error::ApiResult;
use crate::io::api::operation::InvocationContext;
use crate::io::config::ApplicationConfiguration;
use crate::io::mcp::client::{McpClient, McpClientBackend};
use crate::io::mcp::configuration::ExecutorConfiguration;
use acorn_core::prelude::{Box, String, Vec};
use acorn_core::util::SemanticVersion;
use acorn_schema::agent::tools::ToolEffects;
use acorn_schema::validation::Validate;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use color_eyre::eyre::eyre;
use rmcp::model::{CallToolResult, Tool};
use serde_json::{json, Value};

#[derive(Clone)]
struct McpBindingSpec {
    binding_id: String,
    capability_id: String,
    effects: ToolEffects,
    server: String,
    tool: String,
}
impl ApplicationConfiguration {
    pub(super) fn configured_mcp_tools(&self) -> BTreeMap<String, BTreeSet<String>> {
        self.executors.iter().fold(BTreeMap::new(), |mut selected, binding| {
            if let ExecutorConfiguration::Mcp { server, tool } = &binding.executor {
                selected.entry(server.clone()).or_insert_with(BTreeSet::new).insert(tool.clone());
            }
            selected
        })
    }
    fn mcp_binding_specs(&self) -> ApiResult<Vec<McpBindingSpec>> {
        self.executors
            .iter()
            .filter_map(|binding| match &binding.executor {
                | ExecutorConfiguration::Mcp { server, tool } => Some((binding, server, tool)),
                | _ => None,
            })
            .try_fold(
                (BTreeSet::new(), Vec::new()),
                |(mut ids, mut specs), (binding, server, tool)| match binding.validate() {
                    | Err(why) => Err(eyre!("MCP executor binding '{}' is invalid — {why}", binding.id)),
                    | Ok(()) => match ids.insert(binding.id.clone()) {
                        | false => Err(eyre!("Duplicate MCP executor binding ID '{}'", binding.id)),
                        | true => match self.mcp.get(server).and_then(|configured| configured.allowed.get(tool)) {
                            | None => Err(eyre!("MCP tool '{server}/{tool}' is not configured and allowlisted")),
                            | Some(configured) => {
                                specs.push(McpBindingSpec {
                                    binding_id: binding.id.clone(),
                                    capability_id: binding.capability_id.clone(),
                                    effects: configured.operation_effects().into(),
                                    server: server.clone(),
                                    tool: tool.clone(),
                                });
                                Ok((ids, specs))
                            }
                        },
                    },
                },
            )
            .map(|(_, specs)| specs)
    }
}
impl ToolRegistry {
    /// Discover and register only the explicitly configured outbound MCP executor bindings.
    pub async fn register_mcp_bindings(self, configuration: &ApplicationConfiguration, context: InvocationContext) -> ApiResult<Self> {
        let selected_tools = configuration.configured_mcp_tools();
        match selected_tools.is_empty() {
            | true => Ok(self),
            | false => {
                let selected_configuration = selected_tools
                    .iter()
                    .filter_map(|(name, tools)| {
                        configuration.mcp.get(name).map(|configured| {
                            let mut server = configured.clone();
                            server.allowed.retain(|tool, _| tools.contains(tool));
                            (name.clone(), server)
                        })
                    })
                    .collect::<BTreeMap<_, _>>();
                match McpClient::try_from(&selected_configuration) {
                    | Ok(client) => self.register_mcp_bindings_with(configuration, client, context).await,
                    | Err(why) => Err(why),
                }
            }
        }
    }
    pub(super) async fn register_mcp_bindings_with<B>(
        self,
        configuration: &ApplicationConfiguration,
        backend: B,
        context: InvocationContext,
    ) -> ApiResult<Self>
    where
        B: McpClientBackend<CallResult = CallToolResult, Context = InvocationContext, Tool = Tool> + 'static,
        for<'a> B: McpClientBackend<Cancellation<'a> = ToolCancellation<'a>>,
    {
        match configuration.mcp_binding_specs() {
            | Err(why) => Err(why),
            | Ok(specs) => {
                let backend = Arc::new(backend);
                let servers = specs.iter().map(|spec| spec.server.clone()).collect::<BTreeSet<_>>();
                let mut catalogs = Ok(BTreeMap::new());
                for server in servers {
                    catalogs = match catalogs {
                        | Err(why) => Err(why),
                        | Ok(mut entries) => match backend.list_tools(&server, &context, None).await {
                            | Err(why) => Err(eyre!("MCP server '{server}' discovery failed — {why}")),
                            | Ok(tools) => {
                                entries.insert(server, tools);
                                Ok(entries)
                            }
                        },
                    };
                }
                match catalogs {
                    | Err(why) => Err(why),
                    | Ok(catalogs) => specs.into_iter().try_fold(self, |registry, spec| {
                        let matches = catalogs
                            .get(&spec.server)
                            .into_iter()
                            .flatten()
                            .filter(|tool| tool.name.as_ref() == spec.tool)
                            .collect::<Vec<_>>();
                        let tool = match matches.as_slice() {
                            | [tool] => Ok((*tool).clone()),
                            | [] => Err(eyre!(
                                "MCP tool '{}/{}' is not advertised by the configured server",
                                spec.server,
                                spec.tool
                            )),
                            | _ => Err(eyre!("MCP tool '{}/{}' is advertised more than once", spec.server, spec.tool)),
                        };
                        tool.and_then(|tool| {
                            let input_schema = Value::Object(tool.input_schema.as_ref().clone());
                            let output_schema = tool
                                .output_schema
                                .as_ref()
                                .map_or_else(|| json!({"type": "object"}), |schema| Value::Object(schema.as_ref().clone()));
                            let capability = CapabilityDefinition {
                                id: spec.capability_id.clone(),
                                version: SemanticVersion::from((1, 0, 0)),
                                description: format!("Invoke configured MCP tool '{}/{}'.", spec.server, spec.tool),
                                input_schema: input_schema.clone(),
                                output_schema: output_schema.clone(),
                                effects: spec.effects,
                            };
                            let binding = BindingDefinition {
                                id: spec.binding_id.clone(),
                                capability_id: spec.capability_id.clone(),
                                version: capability.version,
                                kind: BindingKind::Remote,
                                input_schema,
                                output_schema,
                                effects: spec.effects,
                                availability: BindingAvailability::Ready,
                                offline_safe: false,
                            };
                            let backend = backend.clone();
                            let call_context = context.clone();
                            let has_output_schema = tool.output_schema.is_some();
                            let server = spec.server.clone();
                            let tool_name = spec.tool.clone();
                            let handler = CapabilityHandler::cancellable(move |arguments, cancellation| {
                                let backend = backend.clone();
                                let context = call_context.clone();
                                let server = server.clone();
                                let tool_name = tool_name.clone();
                                Box::pin(async move {
                                    match backend.call_tool(&server, &tool_name, arguments, &context, Some(cancellation)).await {
                                        | Err(why) => Err(eyre!("MCP tool '{server}/{tool_name}' call failed — {why}")),
                                        | Ok(result) => normalize_result(&server, &tool_name, result, has_output_schema),
                                    }
                                })
                            });
                            registry
                                .register_dynamic_capability(capability)
                                .and_then(|registry| registry.register_dynamic_binding(binding, handler))
                        })
                    }),
                }
            }
        }
    }
}
fn normalize_result(server: &str, tool: &str, result: CallToolResult, has_output_schema: bool) -> ApiResult<Value> {
    match (result.is_error == Some(true), has_output_schema) {
        | (true, _) => Err(eyre!("MCP tool '{server}/{tool}' reported an error")),
        | (false, true) => result
            .structured_content
            .ok_or_else(|| eyre!("MCP tool '{server}/{tool}' omitted its declared structured output")),
        | (false, false) => Ok(json!({
            "content": result.content,
            "structuredContent": result.structured_content,
        })),
    }
}
