//! Immutable, policy-checked outbound MCP client.
use super::config::{McpServerConfiguration, McpTransportConfiguration};
use crate::io::api::operation::{authorize, InvocationContext, OperationEffects};
use crate::io::config::ApplicationConfiguration;
use crate::io::ApiResult;
use acorn_schema::validation::Validate;
use alloc::collections::{BTreeMap, BTreeSet};
use color_eyre::eyre::eyre;
use core::error::Error;
use core::fmt;
use core::time::Duration;
use rmcp::model::{
    CallToolRequest, CallToolRequestParams, CallToolResult, ClientCapabilities, ClientInfo, ClientRequest, Implementation, JsonObject,
    ListToolsRequest, PaginatedRequestParams, ServerResult, Tool,
};
use rmcp::service::{PeerRequestOptions, RunningService};
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::transport::{IntoTransport, StreamableHttpClientTransport, TokioChildProcess};
use rmcp::{RoleClient, ServiceExt};
use secrecy::{ExposeSecret, SecretString};
use serde_json::Value;
use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;
use which::which;

const CATALOG_MAX_BYTES: usize = 1024 * 1024;
const CONTROL_TIMEOUT: Duration = Duration::from_secs(30);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);
enum ResolvedTransport {
    ChildProcess {
        args: Vec<String>,
        command: PathBuf,
        environment: BTreeMap<String, SecretString>,
    },
    StreamableHttp {
        token: Option<SecretString>,
        url: String,
    },
}
struct CallSession<'a, T> {
    arguments: JsonObject,
    output_limit: usize,
    server_name: &'a str,
    timeout: Duration,
    tool_name: &'a str,
    transport: T,
}
struct ClientService {
    name: String,
    service: RunningService<RoleClient, ClientInfo>,
}
struct ListSession<'a, T> {
    allowed: BTreeSet<String>,
    server_name: &'a str,
    transport: T,
}
/// Immutable outbound MCP client constructed from application configuration.
pub struct McpClient {
    servers: BTreeMap<String, ResolvedServer>,
}
struct ResolvedServer {
    allowed: BTreeMap<String, ResolvedTool>,
    transport: ResolvedTransport,
}
struct ResolvedTool {
    effects: OperationEffects,
    max_output: usize,
    timeout: Duration,
}
impl<T> CallSession<'_, T> {
    async fn run<E, A>(self) -> ApiResult<CallToolResult>
    where
        T: IntoTransport<RoleClient, E, A>,
        E: Error + Send + Sync + 'static,
    {
        let Self {
            arguments,
            output_limit,
            server_name,
            timeout,
            tool_name,
            transport,
        } = self;
        match ClientService::connect(server_name, transport).await {
            | Err(why) => Err(why),
            | Ok(mut service) => {
                let allowed = BTreeSet::from([tool_name.to_string()]);
                let operation = service
                    .catalog(&allowed)
                    .await
                    .and_then(|tools| match tools.iter().any(|tool| tool.name.as_ref() == tool_name) {
                        | true => Ok(()),
                        | false => Err(eyre!("MCP tool '{server_name}/{tool_name}' is not advertised by the configured server")),
                    });
                let operation = match operation {
                    | Err(why) => Err(why),
                    | Ok(()) => service.call(tool_name, arguments, timeout, output_limit).await,
                };
                service.finish(operation).await
            }
        }
    }
}
impl ClientService {
    async fn call(&self, tool_name: &str, arguments: JsonObject, timeout: Duration, output_limit: usize) -> ApiResult<CallToolResult> {
        let request = ClientRequest::CallToolRequest(CallToolRequest::new(
            CallToolRequestParams::new(tool_name.to_string()).with_arguments(arguments),
        ));
        let mut options = PeerRequestOptions::default();
        options.timeout = Some(timeout);
        match self.service.send_cancellable_request(request, options).await {
            | Err(_) => Err(eyre!("MCP tool '{}/{tool_name}' request could not be sent", self.name)),
            | Ok(pending) => pending
                .await_response()
                .await
                .map_err(|_| eyre!("MCP tool '{}/{tool_name}' request failed or timed out", self.name))
                .and_then(|response| match response {
                    | ServerResult::CallToolResult(result) => serde_json::to_vec(&result)
                        .map_err(|_| eyre!("MCP tool '{}/{tool_name}' returned an invalid result", self.name))
                        .and_then(|encoded| match encoded.len() <= output_limit {
                            | true => Ok(result),
                            | false => Err(eyre!("MCP tool '{}/{tool_name}' exceeded the configured output limit", self.name)),
                        }),
                    | _ => Err(eyre!("MCP tool '{}/{tool_name}' returned an unexpected response", self.name)),
                }),
        }
    }
    async fn catalog(&self, allowed: &BTreeSet<String>) -> ApiResult<Vec<Tool>> {
        let started = Instant::now();
        let mut cursor = None;
        let mut seen = BTreeSet::new();
        let mut tools = Vec::new();
        loop {
            let remaining = CONTROL_TIMEOUT
                .checked_sub(started.elapsed())
                .ok_or_else(|| eyre!("MCP server '{}' tool catalog timed out", self.name));
            let outcome = match remaining {
                | Err(why) => Err(why),
                | Ok(remaining) => {
                    let mut parameters = PaginatedRequestParams::default();
                    parameters.cursor = cursor.clone();
                    let request = ListToolsRequest::with_param(parameters);
                    let mut options = PeerRequestOptions::default();
                    options.timeout = Some(remaining);
                    match self
                        .service
                        .send_cancellable_request(ClientRequest::ListToolsRequest(request), options)
                        .await
                    {
                        | Err(_) => Err(eyre!("MCP server '{}' tool catalog request could not be sent", self.name)),
                        | Ok(pending) => pending
                            .await_response()
                            .await
                            .map_err(|_| eyre!("MCP server '{}' tool catalog request failed or timed out", self.name))
                            .and_then(|response| match response {
                                | ServerResult::ListToolsResult(page) => Ok(page),
                                | _ => Err(eyre!("MCP server '{}' returned an unexpected tool catalog response", self.name)),
                            })
                            .and_then(|page| {
                                tools.extend(page.tools.into_iter().filter(|tool| allowed.contains(tool.name.as_ref())));
                                serde_json::to_vec(&tools)
                                    .map_err(|_| eyre!("MCP server '{}' returned an invalid tool catalog", self.name))
                                    .and_then(|encoded| match encoded.len() <= CATALOG_MAX_BYTES {
                                        | false => Err(eyre!("MCP server '{}' tool catalog exceeded the configured limit", self.name)),
                                        | true => match page.next_cursor {
                                            | Some(next) if !seen.insert(next.clone()) => {
                                                Err(eyre!("MCP server '{}' repeated a tool catalog cursor", self.name))
                                            }
                                            | Some(next) => {
                                                cursor = Some(next);
                                                Ok(None)
                                            }
                                            | None => Ok(Some(core::mem::take(&mut tools))),
                                        },
                                    })
                            }),
                    }
                }
            };
            match outcome {
                | Ok(None) => {}
                | Ok(Some(tools)) => break Ok(tools),
                | Err(why) => break Err(why),
            }
        }
    }
    async fn close(&mut self) -> ApiResult<()> {
        match self.service.close_with_timeout(SHUTDOWN_TIMEOUT).await {
            | Ok(Some(_)) => Ok(()),
            | Ok(None) => Err(eyre!("MCP server '{}' cleanup timed out", self.name)),
            | Err(_) => Err(eyre!("MCP server '{}' cleanup failed", self.name)),
        }
    }
    async fn connect<T, E, A>(name: &str, transport: T) -> ApiResult<Self>
    where
        T: IntoTransport<RoleClient, E, A>,
        E: Error + Send + Sync + 'static,
    {
        let info = ClientInfo::new(ClientCapabilities::default(), Implementation::new("acorn", env!("CARGO_PKG_VERSION")));
        match tokio::time::timeout(CONTROL_TIMEOUT, info.serve(transport)).await {
            | Err(_) => Err(eyre!("MCP server '{name}' initialization timed out")),
            | Ok(Err(_)) => Err(eyre!("MCP server '{name}' initialization failed")),
            | Ok(Ok(service)) => {
                let mut client = Self {
                    name: name.to_string(),
                    service,
                };
                match client.service.peer_info().is_some_and(|peer| peer.capabilities.tools.is_some()) {
                    | true => Ok(client),
                    | false => {
                        client.close().await.ok();
                        Err(eyre!("MCP server '{name}' does not advertise tool capability"))
                    }
                }
            }
        }
    }
    async fn finish<T>(&mut self, operation: ApiResult<T>) -> ApiResult<T> {
        let cleanup = self.close().await;
        match (operation, cleanup) {
            | (Err(primary), _) => Err(primary),
            | (Ok(value), Ok(())) => Ok(value),
            | (Ok(_), Err(cleanup)) => Err(cleanup),
        }
    }
}
impl<T> ListSession<'_, T> {
    async fn run<E, A>(self) -> ApiResult<Vec<Tool>>
    where
        T: IntoTransport<RoleClient, E, A>,
        E: Error + Send + Sync + 'static,
    {
        let Self {
            allowed,
            server_name,
            transport,
        } = self;
        match ClientService::connect(server_name, transport).await {
            | Err(why) => Err(why),
            | Ok(mut service) => {
                let operation = service.catalog(&allowed).await.map(|mut tools| {
                    tools.sort_by(|left, right| left.name.cmp(&right.name));
                    tools
                });
                service.finish(operation).await
            }
        }
    }
}
impl McpClient {
    /// Call one configured and currently advertised MCP tool.
    pub async fn call_tool(&self, server_name: &str, tool_name: &str, arguments: Value, context: &InvocationContext) -> ApiResult<CallToolResult> {
        match self.prepare_call(server_name, tool_name, arguments, context) {
            | Err(why) => Err(why),
            | Ok((server, tool, arguments)) => {
                let output_limit = tool.max_output.min(context.max_output_bytes.max(1));
                match &server.transport {
                    | ResolvedTransport::ChildProcess { args, command, environment } => {
                        match child_transport(server_name, command, args, environment) {
                            | Err(why) => Err(why),
                            | Ok(transport) => {
                                CallSession {
                                    arguments,
                                    output_limit,
                                    server_name,
                                    timeout: tool.timeout,
                                    tool_name,
                                    transport,
                                }
                                .run()
                                .await
                            }
                        }
                    }
                    | ResolvedTransport::StreamableHttp { token, url } => {
                        CallSession {
                            arguments,
                            output_limit,
                            server_name,
                            timeout: tool.timeout,
                            tool_name,
                            transport: http_transport(url, token.as_ref()),
                        }
                        .run()
                        .await
                    }
                }
            }
        }
    }
    /// List configured tools that are currently advertised and authorized for this context.
    pub async fn list_tools(&self, server_name: &str, context: &InvocationContext) -> ApiResult<Vec<Tool>> {
        match self
            .server(server_name)
            .and_then(|server| validate_transport_policy(server_name, &server.transport, context).map(|()| server))
        {
            | Err(why) => Err(why),
            | Ok(server) => {
                let allowed = server
                    .allowed
                    .iter()
                    .filter(|(_, tool)| authorize(&tool.effects, context).is_ok())
                    .map(|(name, _)| name.clone())
                    .collect::<BTreeSet<_>>();
                match &server.transport {
                    | ResolvedTransport::ChildProcess { args, command, environment } => {
                        match child_transport(server_name, command, args, environment) {
                            | Err(why) => Err(why),
                            | Ok(transport) => {
                                ListSession {
                                    allowed,
                                    server_name,
                                    transport,
                                }
                                .run()
                                .await
                            }
                        }
                    }
                    | ResolvedTransport::StreamableHttp { token, url } => {
                        ListSession {
                            allowed,
                            server_name,
                            transport: http_transport(url, token.as_ref()),
                        }
                        .run()
                        .await
                    }
                }
            }
        }
    }
    fn prepare_call<'a>(
        &'a self,
        server_name: &str,
        tool_name: &str,
        arguments: Value,
        context: &InvocationContext,
    ) -> ApiResult<(&'a ResolvedServer, &'a ResolvedTool, JsonObject)> {
        self.server(server_name).and_then(|server| {
            server
                .allowed
                .get(tool_name)
                .ok_or_else(|| eyre!("MCP tool '{server_name}/{tool_name}' is not allowlisted"))
                .and_then(|tool| {
                    authorize(&tool.effects, context)
                        .map_err(|why| eyre!("MCP tool '{server_name}/{tool_name}' was denied by policy — {why}"))
                        .and_then(|()| validate_transport_policy(server_name, &server.transport, context))
                        .and_then(|()| match arguments {
                            | Value::Object(arguments) => Ok((server, tool, arguments)),
                            | _ => Err(eyre!("MCP tool arguments must be a JSON object")),
                        })
                })
        })
    }
    fn server(&self, name: &str) -> ApiResult<&ResolvedServer> {
        self.servers.get(name).ok_or_else(|| eyre!("MCP server '{name}' is not configured"))
    }
}
impl fmt::Debug for McpClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("McpClient")
            .field("servers", &self.servers.keys().collect::<Vec<_>>())
            .finish()
    }
}
impl TryFrom<&ApplicationConfiguration> for McpClient {
    type Error = color_eyre::Report;

    fn try_from(configuration: &ApplicationConfiguration) -> Result<Self, Self::Error> {
        configuration
            .mcp
            .iter()
            .map(|(name, server)| {
                McpServerConfiguration::validate_name(name)
                    .map_err(|_| eyre!("MCP server '{name}' configuration is invalid"))
                    .and_then(|()| server.resolve(name))
                    .map(|resolved| (name.clone(), resolved))
            })
            .collect::<ApiResult<BTreeMap<_, _>>>()
            .map(|servers| Self { servers })
    }
}
impl McpServerConfiguration {
    fn resolve(&self, server_name: &str) -> ApiResult<ResolvedServer> {
        self.validate()
            .map_err(|_| eyre!("MCP server '{server_name}' configuration is invalid"))
            .and_then(|()| self.transport.resolve(server_name))
            .and_then(|transport| {
                self.allowed
                    .iter()
                    .map(|(tool_name, tool)| {
                        tool.max_output
                            .checked_bytes()
                            .and_then(|bytes| usize::try_from(bytes).ok())
                            .filter(|bytes| *bytes > 0)
                            .map(|max_output| {
                                (
                                    tool_name.clone(),
                                    ResolvedTool {
                                        effects: tool.operation_effects(),
                                        max_output,
                                        timeout: Duration::from(tool.timeout),
                                    },
                                )
                            })
                            .ok_or_else(|| eyre!("MCP server '{server_name}' configuration is invalid"))
                    })
                    .collect::<ApiResult<BTreeMap<_, _>>>()
                    .map(|allowed| ResolvedServer { allowed, transport })
            })
    }
}
impl McpTransportConfiguration {
    fn resolve(&self, name: &str) -> ApiResult<ResolvedTransport> {
        match self {
            | Self::ChildProcess { args, command, environment } => which(command)
                .map_err(|_| eyre!("MCP server '{name}' child command is unavailable"))
                .and_then(|command| {
                    environment
                        .iter()
                        .map(|(child, parent)| {
                            env::var(parent)
                                .ok()
                                .filter(|value| !value.is_empty())
                                .map(SecretString::from)
                                .map(|value| (child.clone(), value))
                                .ok_or_else(|| eyre!("MCP server '{name}' has a missing or empty child environment variable"))
                        })
                        .collect::<ApiResult<BTreeMap<_, _>>>()
                        .map(|environment| ResolvedTransport::ChildProcess {
                            args: args.clone(),
                            command,
                            environment,
                        })
                }),
            | Self::StreamableHttp { bearer_token_env, url } => bearer_token_env
                .as_ref()
                .map(|variable| {
                    env::var(variable)
                        .ok()
                        .filter(|value| !value.is_empty())
                        .map(SecretString::from)
                        .ok_or_else(|| eyre!("MCP server '{name}' has a missing or empty bearer-token environment variable"))
                })
                .transpose()
                .map(|token| ResolvedTransport::StreamableHttp { token, url: url.clone() }),
        }
    }
}
fn child_transport(
    server_name: &str,
    command: &PathBuf,
    args: &[String],
    environment: &BTreeMap<String, SecretString>,
) -> ApiResult<TokioChildProcess> {
    let mut process = Command::new(command);
    process.args(args).env_clear();
    environment.iter().for_each(|(name, value)| {
        process.env(name, value.expose_secret());
    });
    TokioChildProcess::builder(process)
        .stderr(Stdio::null())
        .spawn()
        .map(|(transport, _)| transport)
        .map_err(|_| eyre!("MCP server '{server_name}' child transport could not start"))
}
fn http_transport(url: &str, token: Option<&SecretString>) -> StreamableHttpClientTransport<reqwest::Client> {
    let configuration = StreamableHttpClientTransportConfig::with_uri(url.to_string());
    let configuration = match token {
        | Some(token) => configuration.auth_header(token.expose_secret().to_string()),
        | None => configuration,
    };
    StreamableHttpClientTransport::from_config(configuration)
}
fn validate_transport_policy(server_name: &str, transport: &ResolvedTransport, context: &InvocationContext) -> ApiResult<()> {
    match (context.offline, transport) {
        | (true, ResolvedTransport::StreamableHttp { .. }) => Err(eyre!("MCP server '{server_name}' is unavailable while offline")),
        | _ => Ok(()),
    }
}
