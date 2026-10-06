//! Immutable, policy-checked outbound MCP client.
use super::config::{McpServerConfiguration, McpTransportConfiguration};
use crate::io::api::operation::{authorize, InvocationContext, OperationEffects};
use crate::io::config::ApplicationConfiguration;
use crate::io::ApiResult;
use acorn_schema::validation::Validate;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use color_eyre::eyre::{eyre, Report};
use core::error::Error;
use core::fmt;
use core::future::{pending, Future};
use core::pin::Pin;
use core::time::Duration;
use rmcp::model::{
    CallToolRequest, CallToolRequestParams, CallToolResult, ClientCapabilities, ClientInfo, ClientRequest, Implementation, JsonObject,
    ListToolsRequest, PaginatedRequestParams, ServerResult, Tool,
};
use rmcp::service::{PeerRequestOptions, RequestHandle, RunningService};
use rmcp::transport::async_rw::AsyncRwTransport;
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::transport::{IntoTransport, StreamableHttpClientTransport};
use rmcp::{RoleClient, ServiceExt};
use secrecy::{ExposeSecret, SecretString};
use serde_json::Value;
use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use which::which;

const CATALOG_MAX_BYTES: usize = 1024 * 1024;
const CANCELLATION_NOTICE_TIMEOUT: Duration = Duration::from_millis(250);
const CONTROL_TIMEOUT: Duration = Duration::from_secs(30);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);
type CancellationFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
trait ClientOperation {
    type Output: Send;

    fn cancellation_message(&self) -> String;
    fn run<'a>(
        self,
        service: &'a ClientService,
        cancellation: &'a mut CancellationFuture<'_>,
    ) -> Pin<Box<dyn Future<Output = SessionOutcome<Self::Output>> + Send + 'a>>
    where
        Self: Sized + 'a;
}
/// Contract implemented by outbound MCP clients.
///
/// Contract implemented by outbound MCP clients
/// - addresses configured servers by name,
/// - discovers their authorized tools,
/// - invokes tools with JSON arguments,
/// - and finishes bounded cleanup before a cancellation-aware operation resolves.
///
/// Associated domain and future types let embedders depend on this contract through static dispatch.
pub trait McpClientBackend: Send + Sync {
    /// Future returned by a tool call.
    type CallFuture<'a>: Future<Output = Result<Self::CallResult, Self::Error>> + Send + 'a
    where
        Self: 'a;
    /// Result returned by a successful MCP tool exchange.
    type CallResult: Send;
    /// Cancellation signal accepted for one operation.
    type Cancellation<'a>: Future<Output = ()> + Send + 'a
    where
        Self: 'a;
    /// Caller-supplied policy and resource context.
    type Context: Send + Sync;
    /// Failure returned by client operations.
    type Error: fmt::Debug + fmt::Display + Send + Sync + 'static;
    /// Future returned by tool discovery.
    type ListFuture<'a>: Future<Output = Result<Vec<Self::Tool>, Self::Error>> + Send + 'a
    where
        Self: 'a;
    /// Tool declaration returned by discovery.
    type Tool: Send;

    /// Call one tool, stopping early when the optional cancellation signal resolves.
    fn call_tool<'a>(
        &'a self,
        server_name: &'a str,
        tool_name: &'a str,
        arguments: Value,
        context: &'a Self::Context,
        cancellation: Option<Self::Cancellation<'a>>,
    ) -> Self::CallFuture<'a>;
    /// List tools, stopping early when the optional cancellation signal resolves.
    fn list_tools<'a>(
        &'a self,
        server_name: &'a str,
        context: &'a Self::Context,
        cancellation: Option<Self::Cancellation<'a>>,
    ) -> Self::ListFuture<'a>;
}
enum RequestOutcome<T> {
    Cancelled,
    FailedToReceive,
    FailedToSend,
    Response(T),
}
#[derive(Clone)]
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
enum SessionOutcome<T> {
    Cancelled(Option<Report>),
    Completed(ApiResult<T>),
}
struct CallOperation {
    arguments: JsonObject,
    output_limit: usize,
    server_name: String,
    timeout: Duration,
    tool_name: String,
}
struct ClientService {
    child: Option<Child>,
    name: String,
    service: RunningService<RoleClient, ClientInfo>,
}
struct ListOperation {
    allowed: BTreeSet<String>,
    server_name: String,
}
/// Immutable outbound MCP client constructed from configured server entries.
pub struct McpClient {
    servers: BTreeMap<String, ResolvedServer>,
}
struct PreparedTransport<T> {
    child: Option<Child>,
    transport: T,
}
struct ResolvedServer {
    allowed: BTreeMap<String, ResolvedTool>,
    transport: ResolvedTransport,
}
#[derive(Clone)]
struct ResolvedTool {
    effects: OperationEffects,
    max_output: usize,
    timeout: Duration,
}
impl ClientOperation for CallOperation {
    type Output = CallToolResult;

    fn cancellation_message(&self) -> String {
        format!("MCP tool '{}/{}' call was cancelled", self.server_name, self.tool_name)
    }
    fn run<'a>(
        self,
        service: &'a ClientService,
        cancellation: &'a mut CancellationFuture<'_>,
    ) -> Pin<Box<dyn Future<Output = SessionOutcome<Self::Output>> + Send + 'a>>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let Self {
                arguments,
                output_limit,
                server_name,
                timeout,
                tool_name,
            } = self;
            let allowed = BTreeSet::from([tool_name.clone()]);
            match service.catalog(&allowed, cancellation).await {
                | SessionOutcome::Cancelled(cleanup) => SessionOutcome::Cancelled(cleanup),
                | SessionOutcome::Completed(Err(why)) => SessionOutcome::Completed(Err(why)),
                | SessionOutcome::Completed(Ok(tools)) => match tools.iter().any(|tool| tool.name.as_ref() == tool_name) {
                    | false => SessionOutcome::Completed(Err(eyre!(
                        "MCP tool '{server_name}/{tool_name}' is not advertised by the configured server"
                    ))),
                    | true => service.call(&tool_name, arguments, timeout, output_limit, cancellation).await,
                },
            }
        })
    }
}
impl ClientService {
    async fn call(
        &self,
        tool_name: &str,
        arguments: JsonObject,
        timeout: Duration,
        output_limit: usize,
        cancellation: &mut CancellationFuture<'_>,
    ) -> SessionOutcome<CallToolResult> {
        let request = ClientRequest::CallToolRequest(CallToolRequest::new(
            CallToolRequestParams::new(tool_name.to_string()).with_arguments(arguments),
        ));
        match self.request(request, timeout, cancellation).await {
            | RequestOutcome::Cancelled => SessionOutcome::Cancelled(None),
            | RequestOutcome::FailedToSend => SessionOutcome::Completed(Err(eyre!("MCP tool '{}/{tool_name}' request could not be sent", self.name))),
            | RequestOutcome::FailedToReceive => {
                SessionOutcome::Completed(Err(eyre!("MCP tool '{}/{tool_name}' request failed or timed out", self.name)))
            }
            | RequestOutcome::Response(response) => SessionOutcome::Completed(match response {
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
    async fn catalog(&self, allowed: &BTreeSet<String>, cancellation: &mut CancellationFuture<'_>) -> SessionOutcome<Vec<Tool>> {
        let started = Instant::now();
        let mut cursor = None;
        let mut seen = BTreeSet::new();
        let mut tools = Vec::new();
        loop {
            let remaining = CONTROL_TIMEOUT
                .checked_sub(started.elapsed())
                .ok_or_else(|| eyre!("MCP server '{}' tool catalog timed out", self.name));
            let outcome = match remaining {
                | Err(why) => SessionOutcome::Completed(Err(why)),
                | Ok(remaining) => {
                    let mut parameters = PaginatedRequestParams::default();
                    parameters.cursor = cursor.clone();
                    let request = ListToolsRequest::with_param(parameters);
                    match self.request(ClientRequest::ListToolsRequest(request), remaining, cancellation).await {
                        | RequestOutcome::Cancelled => SessionOutcome::Cancelled(None),
                        | RequestOutcome::FailedToSend => {
                            SessionOutcome::Completed(Err(eyre!("MCP server '{}' tool catalog request could not be sent", self.name)))
                        }
                        | RequestOutcome::FailedToReceive => {
                            SessionOutcome::Completed(Err(eyre!("MCP server '{}' tool catalog request failed or timed out", self.name)))
                        }
                        | RequestOutcome::Response(ServerResult::ListToolsResult(page)) => SessionOutcome::Completed({
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
                        | RequestOutcome::Response(_) => {
                            SessionOutcome::Completed(Err(eyre!("MCP server '{}' returned an unexpected tool catalog response", self.name)))
                        }
                    }
                }
            };
            match outcome {
                | SessionOutcome::Cancelled(cleanup) => break SessionOutcome::Cancelled(cleanup),
                | SessionOutcome::Completed(Ok(None)) => {}
                | SessionOutcome::Completed(Ok(Some(tools))) => break SessionOutcome::Completed(Ok(tools)),
                | SessionOutcome::Completed(Err(why)) => break SessionOutcome::Completed(Err(why)),
            }
        }
    }
    async fn close(&mut self) -> ApiResult<()> {
        let started = Instant::now();
        let service = match self.service.close_with_timeout(SHUTDOWN_TIMEOUT).await {
            | Ok(Some(_)) => Ok(()),
            | Ok(None) => Err(eyre!("MCP server '{}' cleanup timed out", self.name)),
            | Err(_) => Err(eyre!("MCP server '{}' cleanup failed", self.name)),
        };
        let remaining = SHUTDOWN_TIMEOUT.saturating_sub(started.elapsed());
        let child = cleanup_child(self.name.clone(), &mut self.child, remaining).await;
        match (service, child) {
            | (Err(why), _) | (Ok(()), Err(why)) => Err(why),
            | (Ok(()), Ok(())) => Ok(()),
        }
    }
    async fn connect<T, E, A>(name: &str, prepared: PreparedTransport<T>, cancellation: &mut CancellationFuture<'_>) -> SessionOutcome<Self>
    where
        T: IntoTransport<RoleClient, E, A>,
        E: Error + Send + Sync + 'static,
    {
        let PreparedTransport { mut child, transport } = prepared;
        let info = ClientInfo::new(ClientCapabilities::default(), Implementation::new("acorn", env!("CARGO_PKG_VERSION")));
        let initialized = {
            let operation = tokio::time::timeout(CONTROL_TIMEOUT, info.serve(transport));
            tokio::pin!(operation);
            tokio::select! {
                biased;
                _ = cancellation.as_mut() => None,
                result = &mut operation => Some(result),
            }
        };
        match initialized {
            | None => SessionOutcome::Cancelled(cleanup_child(name.to_string(), &mut child, Duration::ZERO).await.err()),
            | Some(Err(_)) => {
                let _ = cleanup_child(name.to_string(), &mut child, Duration::ZERO).await;
                SessionOutcome::Completed(Err(eyre!("MCP server '{name}' initialization timed out")))
            }
            | Some(Ok(Err(_))) => {
                let _ = cleanup_child(name.to_string(), &mut child, Duration::ZERO).await;
                SessionOutcome::Completed(Err(eyre!("MCP server '{name}' initialization failed")))
            }
            | Some(Ok(Ok(service))) => {
                let mut client = Self {
                    child,
                    name: name.to_string(),
                    service,
                };
                match client.service.peer_info().is_some_and(|peer| peer.capabilities.tools.is_some()) {
                    | true => SessionOutcome::Completed(Ok(client)),
                    | false => {
                        client.close().await.ok();
                        SessionOutcome::Completed(Err(eyre!("MCP server '{name}' does not advertise tool capability")))
                    }
                }
            }
        }
    }
    async fn finish<T>(&mut self, operation: SessionOutcome<T>, cancellation_message: String) -> ApiResult<T> {
        let cleanup = self.close().await;
        match (operation, cleanup) {
            | (SessionOutcome::Cancelled(detail), cleanup) => {
                let detail = detail.or_else(|| cleanup.err());
                Err(cancellation_error(cancellation_message, detail))
            }
            | (SessionOutcome::Completed(Err(primary)), _) => Err(primary),
            | (SessionOutcome::Completed(Ok(value)), Ok(())) => Ok(value),
            | (SessionOutcome::Completed(Ok(_)), Err(cleanup)) => Err(cleanup),
        }
    }
    async fn request(&self, request: ClientRequest, timeout: Duration, cancellation: &mut CancellationFuture<'_>) -> RequestOutcome<ServerResult> {
        let pending = tokio::select! {
            biased;
            _ = cancellation.as_mut() => return RequestOutcome::Cancelled,
            pending = self.service.send_cancellable_request(request, PeerRequestOptions::default()) => pending,
        };
        match pending {
            | Err(_) => RequestOutcome::FailedToSend,
            | Ok(mut pending) => {
                let response = tokio::select! {
                    biased;
                    _ = cancellation.as_mut() => None,
                    response = tokio::time::timeout(timeout, &mut pending.rx) => Some(response),
                };
                match response {
                    | None => {
                        let _ = tokio::time::timeout(CANCELLATION_NOTICE_TIMEOUT, pending.cancel(Some("caller cancellation".to_string()))).await;
                        RequestOutcome::Cancelled
                    }
                    | Some(Err(_)) => {
                        let _ = tokio::time::timeout(
                            SHUTDOWN_TIMEOUT,
                            pending.cancel(Some(RequestHandle::<RoleClient>::REQUEST_TIMEOUT_REASON.to_string())),
                        )
                        .await;
                        RequestOutcome::FailedToReceive
                    }
                    | Some(Ok(Ok(Ok(response)))) => RequestOutcome::Response(response),
                    | Some(Ok(Err(_))) | Some(Ok(Ok(Err(_)))) => RequestOutcome::FailedToReceive,
                }
            }
        }
    }
}
impl ClientOperation for ListOperation {
    type Output = Vec<Tool>;

    fn cancellation_message(&self) -> String {
        format!("MCP server '{}' tool listing was cancelled", self.server_name)
    }
    fn run<'a>(
        self,
        service: &'a ClientService,
        cancellation: &'a mut CancellationFuture<'_>,
    ) -> Pin<Box<dyn Future<Output = SessionOutcome<Self::Output>> + Send + 'a>>
    where
        Self: 'a,
    {
        Box::pin(async move {
            match service.catalog(&self.allowed, cancellation).await {
                | SessionOutcome::Cancelled(cleanup) => SessionOutcome::Cancelled(cleanup),
                | SessionOutcome::Completed(result) => SessionOutcome::Completed(result.map(|mut tools| {
                    tools.sort_by(|left, right| left.name.cmp(&right.name));
                    tools
                })),
            }
        })
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
impl McpClient {
    /// Call one configured and currently advertised MCP tool.
    pub async fn call_tool(&self, server_name: &str, tool_name: &str, arguments: Value, context: &InvocationContext) -> ApiResult<CallToolResult> {
        <Self as McpClientBackend>::call_tool(self, server_name, tool_name, arguments, context, None).await
    }
    fn call_tool_impl<'a>(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: Value,
        context: &InvocationContext,
        mut cancellation: Pin<Box<dyn Future<Output = ()> + Send + 'a>>,
    ) -> Pin<Box<dyn Future<Output = ApiResult<CallToolResult>> + Send + 'a>> {
        let prepared = self
            .prepare_call(server_name, tool_name, arguments, context)
            .map(|(transport, tool, arguments)| {
                let operation = CallOperation {
                    arguments,
                    output_limit: tool.max_output.min(context.max_output_bytes.max(1)),
                    server_name: server_name.to_string(),
                    timeout: tool.timeout,
                    tool_name: tool_name.to_string(),
                };
                (operation, transport)
            });
        Box::pin(async move {
            match prepared {
                | Err(why) => Err(why),
                | Ok((operation, transport)) => {
                    if cancellation_ready(cancellation.as_mut()).await {
                        Err(cancellation_error(operation.cancellation_message(), None))
                    } else {
                        match transport {
                            | ResolvedTransport::ChildProcess { args, command, environment } => {
                                match child_transport(&operation.server_name, &command, &args, &environment) {
                                    | Err(why) => Err(why),
                                    | Ok(transport) => run_session(operation.server_name.clone(), transport, operation, cancellation).await,
                                }
                            }
                            | ResolvedTransport::StreamableHttp { token, url } => {
                                run_session(
                                    operation.server_name.clone(),
                                    http_transport(&url, token.as_ref()),
                                    operation,
                                    cancellation,
                                )
                                .await
                            }
                        }
                    }
                }
            }
        })
    }
    /// Call one configured MCP tool with caller-driven cancellation.
    pub async fn call_tool_with_cancellation<C>(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: Value,
        context: &InvocationContext,
        cancellation: C,
    ) -> ApiResult<CallToolResult>
    where
        C: Future<Output = ()> + Send,
    {
        <Self as McpClientBackend>::call_tool(self, server_name, tool_name, arguments, context, Some(Box::pin(cancellation))).await
    }
    /// List configured tools that are currently advertised and authorized for this context.
    pub async fn list_tools(&self, server_name: &str, context: &InvocationContext) -> ApiResult<Vec<Tool>> {
        <Self as McpClientBackend>::list_tools(self, server_name, context, None).await
    }
    fn list_tools_impl<'a>(
        &self,
        server_name: &str,
        context: &InvocationContext,
        mut cancellation: Pin<Box<dyn Future<Output = ()> + Send + 'a>>,
    ) -> Pin<Box<dyn Future<Output = ApiResult<Vec<Tool>>> + Send + 'a>> {
        let prepared = self
            .server(server_name)
            .and_then(|server| validate_transport_policy(server_name, &server.transport, context).map(|()| server))
            .map(|server| {
                let operation = ListOperation {
                    allowed: server
                        .allowed
                        .iter()
                        .filter(|(_, tool)| authorize(&tool.effects, context).is_ok())
                        .map(|(name, _)| name.clone())
                        .collect(),
                    server_name: server_name.to_string(),
                };
                (operation, server.transport.clone())
            });
        Box::pin(async move {
            match prepared {
                | Err(why) => Err(why),
                | Ok((operation, transport)) => {
                    if cancellation_ready(cancellation.as_mut()).await {
                        Err(cancellation_error(operation.cancellation_message(), None))
                    } else {
                        match transport {
                            | ResolvedTransport::ChildProcess { args, command, environment } => {
                                match child_transport(&operation.server_name, &command, &args, &environment) {
                                    | Err(why) => Err(why),
                                    | Ok(transport) => run_session(operation.server_name.clone(), transport, operation, cancellation).await,
                                }
                            }
                            | ResolvedTransport::StreamableHttp { token, url } => {
                                run_session(
                                    operation.server_name.clone(),
                                    http_transport(&url, token.as_ref()),
                                    operation,
                                    cancellation,
                                )
                                .await
                            }
                        }
                    }
                }
            }
        })
    }
    /// List configured tools with caller-driven cancellation.
    pub async fn list_tools_with_cancellation<C>(&self, server_name: &str, context: &InvocationContext, cancellation: C) -> ApiResult<Vec<Tool>>
    where
        C: Future<Output = ()> + Send,
    {
        <Self as McpClientBackend>::list_tools(self, server_name, context, Some(Box::pin(cancellation))).await
    }
    fn prepare_call(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: Value,
        context: &InvocationContext,
    ) -> ApiResult<(ResolvedTransport, ResolvedTool, JsonObject)> {
        match self.server(server_name) {
            | Err(why) => Err(why),
            | Ok(server) => server
                .allowed
                .get(tool_name)
                .ok_or_else(|| eyre!("MCP tool '{server_name}/{tool_name}' is not allowlisted"))
                .and_then(|tool| {
                    authorize(&tool.effects, context)
                        .map_err(|why| eyre!("MCP tool '{server_name}/{tool_name}' was denied by policy — {why}"))
                        .and_then(|()| validate_transport_policy(server_name, &server.transport, context))
                        .and_then(|()| match arguments {
                            | Value::Object(arguments) => Ok((server.transport.clone(), tool.clone(), arguments)),
                            | _ => Err(eyre!("MCP tool arguments must be a JSON object")),
                        })
                }),
        }
    }
    fn server(&self, name: &str) -> ApiResult<&ResolvedServer> {
        self.servers.get(name).ok_or_else(|| eyre!("MCP server '{name}' is not configured"))
    }
}
impl McpClientBackend for McpClient {
    type CallFuture<'a> = Pin<Box<dyn Future<Output = ApiResult<CallToolResult>> + Send + 'a>>;
    type CallResult = CallToolResult;
    type Cancellation<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
    type Context = InvocationContext;
    type Error = Report;
    type ListFuture<'a> = Pin<Box<dyn Future<Output = ApiResult<Vec<Tool>>> + Send + 'a>>;
    type Tool = Tool;

    fn call_tool<'a>(
        &'a self,
        server_name: &'a str,
        tool_name: &'a str,
        arguments: Value,
        context: &'a Self::Context,
        cancellation: Option<Self::Cancellation<'a>>,
    ) -> Self::CallFuture<'a> {
        self.call_tool_impl(
            server_name,
            tool_name,
            arguments,
            context,
            cancellation.unwrap_or_else(|| Box::pin(pending())),
        )
    }
    fn list_tools<'a>(
        &'a self,
        server_name: &'a str,
        context: &'a Self::Context,
        cancellation: Option<Self::Cancellation<'a>>,
    ) -> Self::ListFuture<'a> {
        self.list_tools_impl(server_name, context, cancellation.unwrap_or_else(|| Box::pin(pending())))
    }
}
impl TryFrom<&ApplicationConfiguration> for McpClient {
    type Error = color_eyre::Report;

    fn try_from(configuration: &ApplicationConfiguration) -> Result<Self, Self::Error> {
        Self::try_from(&configuration.mcp)
    }
}
impl TryFrom<&BTreeMap<String, McpServerConfiguration>> for McpClient {
    type Error = color_eyre::Report;

    fn try_from(configuration: &BTreeMap<String, McpServerConfiguration>) -> Result<Self, Self::Error> {
        configuration
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
fn cancellation_error(message: String, detail: Option<Report>) -> Report {
    match detail {
        | None => eyre!(message),
        | Some(detail) => eyre!("{message}; {detail}"),
    }
}
async fn cancellation_ready<C>(mut cancellation: Pin<&mut C>) -> bool
where
    C: Future<Output = ()> + Send + ?Sized,
{
    tokio::select! {
        biased;
        _ = cancellation.as_mut() => true,
        _ = core::future::ready(()) => false,
    }
}
fn child_transport(
    server_name: &str,
    command: &PathBuf,
    args: &[String],
    environment: &BTreeMap<String, SecretString>,
) -> ApiResult<PreparedTransport<AsyncRwTransport<RoleClient, ChildStdout, ChildStdin>>> {
    let mut process = Command::new(command);
    process
        .args(args)
        .env_clear()
        .kill_on_drop(true)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    environment.iter().for_each(|(name, value)| {
        process.env(name, value.expose_secret());
    });
    process
        .spawn()
        .map_err(|_| eyre!("MCP server '{server_name}' child transport could not start"))
        .and_then(|mut child| {
            child
                .stdout
                .take()
                .zip(child.stdin.take())
                .map(|(stdout, stdin)| PreparedTransport {
                    child: Some(child),
                    transport: AsyncRwTransport::new(stdout, stdin),
                })
                .ok_or_else(|| eyre!("MCP server '{server_name}' child transport could not start"))
        })
}
async fn cleanup_child(server_name: String, child: &mut Option<Child>, grace: Duration) -> ApiResult<()> {
    match child.take() {
        | None => Ok(()),
        | Some(mut child) => match child.try_wait() {
            | Ok(Some(_)) => Ok(()),
            | Ok(None) | Err(_) => {
                let graceful = match grace.is_zero() {
                    | true => None,
                    | false => Some(tokio::time::timeout(grace, child.wait()).await),
                };
                match graceful {
                    | Some(Ok(Ok(_))) => Ok(()),
                    | _ => child.kill().await.map_err(|_| eyre!("MCP server '{server_name}' child cleanup failed")),
                }
            }
        },
    }
}
fn http_transport(url: &str, token: Option<&SecretString>) -> PreparedTransport<StreamableHttpClientTransport<reqwest::Client>> {
    let configuration = StreamableHttpClientTransportConfig::with_uri(url.to_string());
    let configuration = match token {
        | Some(token) => configuration.auth_header(token.expose_secret().to_string()),
        | None => configuration,
    };
    PreparedTransport {
        child: None,
        transport: StreamableHttpClientTransport::from_config(configuration),
    }
}
async fn run_session<T, E, A, O>(
    server_name: String,
    transport: PreparedTransport<T>,
    operation: O,
    mut cancellation: CancellationFuture<'_>,
) -> ApiResult<O::Output>
where
    T: IntoTransport<RoleClient, E, A>,
    E: Error + Send + Sync + 'static,
    O: ClientOperation,
{
    let cancellation_message = operation.cancellation_message();
    match ClientService::connect(&server_name, transport, &mut cancellation).await {
        | SessionOutcome::Cancelled(detail) => Err(cancellation_error(cancellation_message, detail)),
        | SessionOutcome::Completed(Err(why)) => Err(why),
        | SessionOutcome::Completed(Ok(mut service)) => {
            let result = operation.run(&service, &mut cancellation).await;
            service.finish(result, cancellation_message).await
        }
    }
}
fn validate_transport_policy(server_name: &str, transport: &ResolvedTransport, context: &InvocationContext) -> ApiResult<()> {
    match (context.offline, transport) {
        | (true, ResolvedTransport::StreamableHttp { .. }) => Err(eyre!("MCP server '{server_name}' is unavailable while offline")),
        | _ => Ok(()),
    }
}
