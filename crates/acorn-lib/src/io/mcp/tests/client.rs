use crate::io::api::operation::{InvocationContext, OperationEffect};
use crate::io::config::ApplicationConfiguration;
use crate::io::mcp::client::McpClient;
use crate::io::mcp::config::{McpServerConfiguration, McpToolConfiguration, McpTransportConfiguration};
use acorn_core::time::{Seconds, UnitOfTime};
use acorn_schema::hardware::Memory;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use axum::extract::State;
use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use core::net::SocketAddr;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;
use serde_json::Value;
use temp_env::with_vars;

#[derive(Clone, Copy)]
enum HttpTestMode {
    Malformed,
    NoTools,
    Normal,
    RepeatedCursor,
    Slow,
}
#[derive(Clone)]
struct HttpState {
    cancellations: Arc<AtomicUsize>,
    mode: HttpTestMode,
    requests: Arc<AtomicUsize>,
}
fn configuration(transport: McpTransportConfiguration) -> ApplicationConfiguration {
    ApplicationConfiguration {
        mcp: BTreeMap::from([(
            "research".to_string(),
            McpServerConfiguration {
                allowed: BTreeMap::from([(
                    "resolve".to_string(),
                    McpToolConfiguration {
                        effects: Vec::new(),
                        max_output: Memory::kb(0.125),
                        timeout: Seconds::new(2),
                    },
                )]),
                transport,
            },
        )]),
        ..ApplicationConfiguration::default()
    }
}
async fn http_server(State(state): State<HttpState>, headers: HeaderMap, Json(request): Json<Value>) -> Response {
    if headers.get(AUTHORIZATION).and_then(|value| value.to_str().ok()) != Some("Bearer secret") {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    state.requests.fetch_add(1, Ordering::SeqCst);
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    match request.get("method").and_then(Value::as_str) {
        | Some("initialize") => {
            let capabilities = match state.mode {
                | HttpTestMode::NoTools => serde_json::json!({}),
                | HttpTestMode::Malformed | HttpTestMode::Normal | HttpTestMode::RepeatedCursor | HttpTestMode::Slow => {
                    serde_json::json!({"tools": {}})
                }
            };
            Json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": request.pointer("/params/protocolVersion").cloned().unwrap_or(Value::String("2025-06-18".to_string())),
                    "capabilities": capabilities,
                    "serverInfo": {"name": "test", "version": "1.0.0"}
                }
            }))
            .into_response()
        }
        | Some("notifications/cancelled") => {
            state.cancellations.fetch_add(1, Ordering::SeqCst);
            StatusCode::ACCEPTED.into_response()
        }
        | Some("notifications/initialized") => StatusCode::ACCEPTED.into_response(),
        | Some("tools/call") => {
            if matches!(state.mode, HttpTestMode::Slow) {
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            Json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{"type": "text", "text": "resolved"}],
                    "structuredContent": {"resolved": true},
                    "isError": false
                }
            }))
            .into_response()
        }
        | Some("tools/list") => {
            if matches!(state.mode, HttpTestMode::Malformed) {
                return (StatusCode::OK, "{malformed").into_response();
            }
            let next_cursor = match state.mode {
                | HttpTestMode::RepeatedCursor => Some("repeat"),
                | HttpTestMode::Malformed | HttpTestMode::NoTools | HttpTestMode::Normal | HttpTestMode::Slow => None,
            };
            Json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": [{
                        "name": "resolve",
                        "description": "Resolve an identifier",
                        "inputSchema": {"type": "object"}
                    }],
                    "nextCursor": next_cursor
                }
            }))
            .into_response()
        }
        | _ => StatusCode::BAD_REQUEST.into_response(),
    }
}
async fn start_http_server(mode: HttpTestMode) -> (SocketAddr, Arc<AtomicUsize>, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind HTTP server");
    let address = listener.local_addr().expect("HTTP server address");
    let cancellations = Arc::new(AtomicUsize::new(0));
    let requests = Arc::new(AtomicUsize::new(0));
    let application = Router::new().route("/mcp", post(http_server)).with_state(HttpState {
        cancellations: Arc::clone(&cancellations),
        mode,
        requests: Arc::clone(&requests),
    });
    let server = tokio::spawn(async move {
        let _ = axum::serve(listener, application).await;
    });
    (address, requests, cancellations, server)
}

#[test]
fn test_call_policy_is_checked_before_transport_start() {
    let mut config = configuration(McpTransportConfiguration::StreamableHttp {
        bearer_token_env: None,
        url: "https://example.org/mcp".to_string(),
    });
    config
        .mcp
        .get_mut("research")
        .and_then(|server| server.allowed.get_mut("resolve"))
        .expect("tool configuration")
        .effects
        .push(OperationEffect::Mutation);
    let client = McpClient::try_from(&config).expect("client");
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let why = runtime
        .block_on(client.call_tool("research", "resolve", serde_json::json!({}), &InvocationContext::default()))
        .expect_err("mutation denial");
    assert!(why.to_string().contains("mutation approval"));
}
#[cfg(unix)]
#[tokio::test]
async fn test_child_process_reports_early_exit() {
    let config = configuration(McpTransportConfiguration::ChildProcess {
        args: Vec::new(),
        command: "true".to_string(),
        environment: BTreeMap::new(),
    });
    let client = McpClient::try_from(&config).expect("client");
    let why = client
        .list_tools("research", &InvocationContext::default())
        .await
        .expect_err("early child exit");
    assert!(why.to_string().contains("initialization failed"));
}
#[test]
fn test_client_debug_contains_only_server_aliases() {
    let config = configuration(McpTransportConfiguration::ChildProcess {
        args: Vec::new(),
        command: "true".to_string(),
        environment: BTreeMap::new(),
    });
    let client = McpClient::try_from(&config).expect("client");
    let debug = format!("{client:?}");
    assert!(debug.contains("research"));
    assert!(!debug.contains("true"));
}
#[test]
fn test_client_rejects_missing_environment_values_without_leaking_names() {
    with_vars([("ACORN_MCP_TEST_SECRET", None::<&str>)], || {
        let config = configuration(McpTransportConfiguration::StreamableHttp {
            bearer_token_env: Some("ACORN_MCP_TEST_SECRET".to_string()),
            url: "https://example.org/mcp".to_string(),
        });
        let message = McpClient::try_from(&config).expect_err("missing secret").to_string();
        assert!(!message.contains("ACORN_MCP_TEST_SECRET"));
    });
}
#[tokio::test]
async fn test_streamable_http_is_rejected_offline_before_connection() {
    let config = configuration(McpTransportConfiguration::StreamableHttp {
        bearer_token_env: None,
        url: "https://example.org/mcp".to_string(),
    });
    let client = McpClient::try_from(&config).expect("client");
    let why = client
        .list_tools(
            "research",
            &InvocationContext {
                offline: true,
                ..InvocationContext::default()
            },
        )
        .await
        .expect_err("offline denial");
    assert!(why.to_string().contains("unavailable while offline"));
}
#[tokio::test]
async fn test_streamable_http_list_call_and_bearer_authentication() {
    let (address, requests, _, server) = start_http_server(HttpTestMode::Normal).await;
    let config = configuration(McpTransportConfiguration::StreamableHttp {
        bearer_token_env: Some("ACORN_MCP_HTTP_TOKEN".to_string()),
        url: format!("http://{address}/mcp"),
    });
    let client = with_vars([("ACORN_MCP_HTTP_TOKEN", Some("secret"))], || {
        McpClient::try_from(&config).expect("client")
    });
    let tools = client
        .list_tools("research", &InvocationContext::default())
        .await
        .expect("HTTP tool catalog");
    assert_eq!(tools.iter().map(|tool| tool.name.as_ref()).collect::<Vec<_>>(), ["resolve"]);
    let result = client
        .call_tool(
            "research",
            "resolve",
            serde_json::json!({"identifier": "doi:10.1000/example"}),
            &InvocationContext::default(),
        )
        .await
        .expect("HTTP tool call");
    assert_eq!(result.structured_content, Some(serde_json::json!({"resolved": true})));
    let mut limited = config.clone();
    limited
        .mcp
        .get_mut("research")
        .and_then(|server| server.allowed.get_mut("resolve"))
        .expect("limited tool")
        .max_output = Memory::kb(0.001);
    let limited = with_vars([("ACORN_MCP_HTTP_TOKEN", Some("secret"))], || {
        McpClient::try_from(&limited).expect("limited client")
    });
    let why = limited
        .call_tool("research", "resolve", serde_json::json!({}), &InvocationContext::default())
        .await
        .expect_err("output bound");
    assert!(why.to_string().contains("output limit"));
    let mut unadvertised = config;
    let tools = &mut unadvertised.mcp.get_mut("research").expect("server").allowed;
    let policy = tools.remove("resolve").expect("tool policy");
    tools.insert("missing".to_string(), policy);
    let unadvertised = with_vars([("ACORN_MCP_HTTP_TOKEN", Some("secret"))], || {
        McpClient::try_from(&unadvertised).expect("unadvertised client")
    });
    let why = unadvertised
        .call_tool("research", "missing", serde_json::json!({}), &InvocationContext::default())
        .await
        .expect_err("unadvertised tool");
    assert!(why.to_string().contains("not advertised"));
    assert!(requests.load(Ordering::SeqCst) >= 6);
    server.abort();
}
#[tokio::test]
async fn test_streamable_http_rejects_absent_tool_capability() {
    let (address, _, _, server) = start_http_server(HttpTestMode::NoTools).await;
    let config = configuration(McpTransportConfiguration::StreamableHttp {
        bearer_token_env: Some("ACORN_MCP_HTTP_TOKEN".to_string()),
        url: format!("http://{address}/mcp"),
    });
    let client = with_vars([("ACORN_MCP_HTTP_TOKEN", Some("secret"))], || {
        McpClient::try_from(&config).expect("client")
    });
    let why = client
        .list_tools("research", &InvocationContext::default())
        .await
        .expect_err("missing tool capability");
    assert!(why.to_string().contains("does not advertise tool capability"));
    server.abort();
}
#[tokio::test]
async fn test_streamable_http_rejects_malformed_catalog_messages() {
    let (address, _, _, server) = start_http_server(HttpTestMode::Malformed).await;
    let config = configuration(McpTransportConfiguration::StreamableHttp {
        bearer_token_env: Some("ACORN_MCP_HTTP_TOKEN".to_string()),
        url: format!("http://{address}/mcp"),
    });
    let client = with_vars([("ACORN_MCP_HTTP_TOKEN", Some("secret"))], || {
        McpClient::try_from(&config).expect("client")
    });
    let why = client
        .list_tools("research", &InvocationContext::default())
        .await
        .expect_err("malformed catalog response");
    assert!(why.to_string().contains("tool catalog"));
    server.abort();
}
#[tokio::test]
async fn test_streamable_http_rejects_repeated_catalog_cursors() {
    let (address, _, _, server) = start_http_server(HttpTestMode::RepeatedCursor).await;
    let config = configuration(McpTransportConfiguration::StreamableHttp {
        bearer_token_env: Some("ACORN_MCP_HTTP_TOKEN".to_string()),
        url: format!("http://{address}/mcp"),
    });
    let client = with_vars([("ACORN_MCP_HTTP_TOKEN", Some("secret"))], || {
        McpClient::try_from(&config).expect("client")
    });
    let why = client
        .list_tools("research", &InvocationContext::default())
        .await
        .expect_err("repeated cursor");
    assert!(why.to_string().contains("repeated a tool catalog cursor"));
    server.abort();
}
#[tokio::test]
async fn test_streamable_http_times_out_tool_calls() {
    let (address, _, cancellations, server) = start_http_server(HttpTestMode::Slow).await;
    let mut config = configuration(McpTransportConfiguration::StreamableHttp {
        bearer_token_env: Some("ACORN_MCP_HTTP_TOKEN".to_string()),
        url: format!("http://{address}/mcp"),
    });
    config
        .mcp
        .get_mut("research")
        .and_then(|configured| configured.allowed.get_mut("resolve"))
        .expect("tool")
        .timeout = Seconds::new(1);
    let client = with_vars([("ACORN_MCP_HTTP_TOKEN", Some("secret"))], || {
        McpClient::try_from(&config).expect("client")
    });
    let why = client
        .call_tool("research", "resolve", serde_json::json!({}), &InvocationContext::default())
        .await
        .expect_err("tool timeout");
    assert!(why.to_string().contains("failed or timed out"));
    assert_eq!(cancellations.load(Ordering::SeqCst), 1);
    server.abort();
}
