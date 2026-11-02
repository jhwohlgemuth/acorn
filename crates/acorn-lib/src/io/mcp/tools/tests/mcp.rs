use super::*;
use crate::io::api::operation::InvocationContext;
use crate::io::config::ApplicationConfiguration;
use crate::io::mcp::client::McpClientBackend;
use crate::io::mcp::config::{McpServerConfiguration, McpToolConfiguration, McpTransportConfiguration};
use crate::io::mcp::{ExecutorBindingConfiguration, ExecutorConfiguration};
use acorn_core::time::{Seconds, UnitOfTime};
use acorn_schema::hardware::Memory;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use color_eyre::eyre::{eyre, Report};
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;
use rmcp::model::{CallToolResult, Tool, ToolAnnotations};
use serde_json::{json, Map, Value};
use std::sync::Mutex;

#[derive(Clone)]
enum CallBehavior {
    Result(CallToolResult),
    WaitForCancellation,
}
#[derive(Clone)]
struct FakeBackend {
    behavior: CallBehavior,
    calls: Arc<Mutex<Vec<(String, String)>>>,
    catalogs: Arc<BTreeMap<String, Vec<Tool>>>,
    cleanups: Arc<AtomicUsize>,
    lists: Arc<Mutex<Vec<String>>>,
}
impl FakeBackend {
    fn new(catalogs: BTreeMap<String, Vec<Tool>>, behavior: CallBehavior) -> Self {
        Self {
            behavior,
            calls: Arc::new(Mutex::new(Vec::new())),
            catalogs: Arc::new(catalogs),
            cleanups: Arc::new(AtomicUsize::new(0)),
            lists: Arc::new(Mutex::new(Vec::new())),
        }
    }
}
impl McpClientBackend for FakeBackend {
    type CallFuture<'a> = Pin<Box<dyn Future<Output = Result<Self::CallResult, Self::Error>> + Send + 'a>>;
    type CallResult = CallToolResult;
    type Cancellation<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
    type Context = InvocationContext;
    type Error = Report;
    type ListFuture<'a> = Pin<Box<dyn Future<Output = Result<Vec<Self::Tool>, Self::Error>> + Send + 'a>>;
    type Tool = Tool;

    fn call_tool<'a>(
        &'a self,
        server_name: &'a str,
        tool_name: &'a str,
        _arguments: Value,
        _context: &'a Self::Context,
        cancellation: Option<Self::Cancellation<'a>>,
    ) -> Self::CallFuture<'a> {
        self.calls
            .lock()
            .expect("call log")
            .push((server_name.to_string(), tool_name.to_string()));
        let behavior = self.behavior.clone();
        let cleanups = self.cleanups.clone();
        Box::pin(async move {
            match behavior {
                | CallBehavior::Result(result) => Ok(result),
                | CallBehavior::WaitForCancellation => match cancellation {
                    | Some(cancellation) => {
                        cancellation.await;
                        cleanups.fetch_add(1, Ordering::SeqCst);
                        Err(eyre!("cancelled"))
                    }
                    | None => core::future::pending().await,
                },
            }
        })
    }
    fn list_tools<'a>(
        &'a self,
        server_name: &'a str,
        _context: &'a Self::Context,
        _cancellation: Option<Self::Cancellation<'a>>,
    ) -> Self::ListFuture<'a> {
        self.lists.lock().expect("list log").push(server_name.to_string());
        let tools = self.catalogs.get(server_name).cloned();
        Box::pin(async move { tools.ok_or_else(|| eyre!("unknown fake server")) })
    }
}
fn binding(id: &str, capability_id: &str, server: &str, tool: &str) -> ExecutorBindingConfiguration {
    ExecutorBindingConfiguration {
        id: id.to_string(),
        capability_id: capability_id.to_string(),
        executor: ExecutorConfiguration::Mcp {
            server: server.to_string(),
            tool: tool.to_string(),
        },
    }
}
fn configuration(bindings: Vec<ExecutorBindingConfiguration>, allowed: &[(&str, &[&str])]) -> ApplicationConfiguration {
    let mcp = allowed
        .iter()
        .map(|(server, tools)| {
            let allowed = tools
                .iter()
                .map(|tool| {
                    (
                        (*tool).to_string(),
                        McpToolConfiguration {
                            effects: Vec::new(),
                            max_output: Memory::mb(1),
                            timeout: Seconds::new(30),
                        },
                    )
                })
                .collect();
            (
                (*server).to_string(),
                McpServerConfiguration {
                    allowed,
                    transport: McpTransportConfiguration::ChildProcess {
                        args: Vec::new(),
                        command: "unused-by-fake".to_string(),
                        environment: BTreeMap::new(),
                    },
                },
            )
        })
        .collect();
    ApplicationConfiguration {
        executors: bindings,
        mcp,
        ..ApplicationConfiguration::default()
    }
}
fn object(value: Value) -> Map<String, Value> {
    value.as_object().expect("object schema").clone()
}
fn remote_selection() -> CapabilitySelectionContext {
    CapabilitySelectionContext {
        offline: false,
        allow_remote: true,
    }
}
fn tool(name: &str, input_property: &str) -> Tool {
    let mut properties = Map::new();
    properties.insert(input_property.to_string(), json!({"type": "string"}));
    let mut input_schema = object(json!({
        "type": "object",
        "required": [input_property]
    }));
    input_schema.insert("properties".to_string(), Value::Object(properties));
    let mut tool =
        Tool::new(name.to_string(), "untrusted secret description", Arc::new(input_schema)).with_raw_output_schema(Arc::new(object(json!({
            "type": "object",
            "properties": {"value": {"type": "string"}},
            "required": ["value"]
        }))));
    let mut annotations = ToolAnnotations::default();
    annotations.destructive_hint = Some(true);
    annotations.read_only_hint = Some(false);
    tool.annotations = Some(annotations);
    tool
}

#[tokio::test]
async fn test_configuration_without_mcp_bindings_performs_no_discovery() {
    let configuration = configuration(Vec::new(), &[("unused", &["other"])]);
    let backend = FakeBackend::new(BTreeMap::new(), CallBehavior::Result(CallToolResult::default()));
    ToolRegistry::default()
        .register_mcp_bindings_with(&configuration, backend.clone(), InvocationContext::default())
        .await
        .expect("empty registration");
    assert!(backend.lists.lock().expect("lists").is_empty());
    assert!(backend.calls.lock().expect("calls").is_empty());
}
#[tokio::test]
async fn test_provider_tool_name_collisions_use_configured_binding_ids() {
    let configuration = configuration(
        vec![
            binding("mcp:one-resolve", "acorn.resolve-one", "one", "resolve"),
            binding("mcp:two-resolve", "acorn.resolve-two", "two", "resolve"),
        ],
        &[("one", &["resolve"]), ("two", &["resolve"])],
    );
    let backend = FakeBackend::new(
        BTreeMap::from([
            ("one".to_string(), vec![tool("resolve", "query")]),
            ("two".to_string(), vec![tool("resolve", "query")]),
        ]),
        CallBehavior::Result(CallToolResult::structured(json!({"value":"unused"}))),
    );
    let registry = ToolRegistry::default()
        .register_mcp_bindings_with(&configuration, backend, InvocationContext::default())
        .await
        .expect("registration");
    assert_eq!(
        registry
            .select_capability_binding("acorn.resolve-one", None, None, remote_selection())
            .expect("first")
            .binding
            .id,
        "mcp:one-resolve"
    );
    assert_eq!(
        registry
            .select_capability_binding("acorn.resolve-two", None, None, remote_selection())
            .expect("second")
            .binding
            .id,
        "mcp:two-resolve"
    );
}
#[tokio::test]
async fn test_registers_and_calls_only_explicit_configured_tool() {
    let configuration = configuration(
        vec![binding("mcp:resolve", "acorn.resolve", "research", "resolve")],
        &[("research", &["resolve", "unselected"]), ("unused", &["other"])],
    );
    let backend = FakeBackend::new(
        BTreeMap::from([("research".to_string(), vec![tool("resolve", "query"), tool("unselected", "value")])]),
        CallBehavior::Result(CallToolResult::structured(json!({"value":"found"}))),
    );
    assert_eq!(
        configuration.configured_mcp_tools(),
        BTreeMap::from([("research".to_string(), ["resolve".to_string()].into_iter().collect())])
    );
    let registry = ToolRegistry::default()
        .register_mcp_bindings_with(&configuration, backend.clone(), InvocationContext::default())
        .await
        .expect("registration");
    assert_eq!(backend.lists.lock().expect("lists").as_slice(), ["research"]);
    let selected = registry
        .select_capability_binding("acorn.resolve", None, None, remote_selection())
        .expect("selection");
    assert!(!selected.capability.description.contains("untrusted"));
    assert!(selected.capability.effects.read_only);
    assert!(!selected.capability.effects.destructive);
    let result = registry
        .execute_selected(
            &selected,
            json!({"query":"needle"}),
            ExecutionContext {
                grounded: true,
                selection: remote_selection(),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(result.outcome, ExecutionOutcome::Completed);
    assert_eq!(result.content, Some(json!({"value":"found"})));
    assert_eq!(
        backend.calls.lock().expect("calls").as_slice(),
        [("research".to_string(), "resolve".to_string())]
    );
}
#[tokio::test]
async fn test_router_cancellation_waits_for_mcp_cleanup() {
    let configuration = configuration(
        vec![binding("mcp:resolve", "acorn.resolve", "research", "resolve")],
        &[("research", &["resolve"])],
    );
    let backend = FakeBackend::new(
        BTreeMap::from([("research".to_string(), vec![tool("resolve", "query")])]),
        CallBehavior::WaitForCancellation,
    );
    let registry = ToolRegistry::default()
        .register_mcp_bindings_with(&configuration, backend.clone(), InvocationContext::default())
        .await
        .expect("registration");
    let selected = registry
        .select_capability_binding("acorn.resolve", None, None, remote_selection())
        .expect("selection");
    let result = registry
        .execute_selected(
            &selected,
            json!({"query":"needle"}),
            ExecutionContext {
                cancellation: Some(Box::pin(tokio::time::sleep(Duration::from_millis(5)))),
                grounded: true,
                selection: remote_selection(),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(result.outcome, ExecutionOutcome::Cancelled);
    assert_eq!(backend.cleanups.load(Ordering::SeqCst), 1);
    assert_eq!(backend.calls.lock().expect("calls").len(), 1);
}
#[tokio::test]
async fn test_schema_drift_duplicate_catalog_entries_and_allowlist_fail_closed() {
    let drift = configuration(
        vec![
            binding("mcp:first", "acorn.shared", "one", "resolve"),
            binding("mcp:second", "acorn.shared", "two", "resolve"),
        ],
        &[("one", &["resolve"]), ("two", &["resolve"])],
    );
    let backend = FakeBackend::new(
        BTreeMap::from([
            ("one".to_string(), vec![tool("resolve", "query")]),
            ("two".to_string(), vec![tool("resolve", "different")]),
        ]),
        CallBehavior::Result(CallToolResult::structured(json!({"value":"unused"}))),
    );
    assert!(ToolRegistry::default()
        .register_mcp_bindings_with(&drift, backend, InvocationContext::default())
        .await
        .expect_err("schema drift")
        .to_string()
        .contains("different contract"));
    let duplicate = configuration(vec![binding("mcp:resolve", "acorn.resolve", "one", "resolve")], &[("one", &["resolve"])]);
    let backend = FakeBackend::new(
        BTreeMap::from([("one".to_string(), vec![tool("resolve", "query"), tool("resolve", "query")])]),
        CallBehavior::Result(CallToolResult::structured(json!({"value":"unused"}))),
    );
    assert!(ToolRegistry::default()
        .register_mcp_bindings_with(&duplicate, backend, InvocationContext::default())
        .await
        .expect_err("duplicate advertisement")
        .to_string()
        .contains("more than once"));
    let missing = configuration(vec![binding("mcp:missing", "acorn.missing", "one", "missing")], &[("one", &["resolve"])]);
    let backend = FakeBackend::new(BTreeMap::new(), CallBehavior::Result(CallToolResult::default()));
    assert!(ToolRegistry::default()
        .register_mcp_bindings_with(&missing, backend.clone(), InvocationContext::default())
        .await
        .expect_err("not allowlisted")
        .to_string()
        .contains("not configured and allowlisted"));
    assert!(backend.lists.lock().expect("lists").is_empty());
    let invalid = configuration(
        vec![binding("invalid binding", "acorn.resolve", "one", "resolve")],
        &[("one", &["resolve"])],
    );
    let backend = FakeBackend::new(
        BTreeMap::from([("one".to_string(), vec![tool("resolve", "query")])]),
        CallBehavior::Result(CallToolResult::default()),
    );
    assert!(ToolRegistry::default()
        .register_mcp_bindings_with(&invalid, backend.clone(), InvocationContext::default())
        .await
        .expect_err("invalid binding")
        .to_string()
        .contains("is invalid"));
    assert!(backend.lists.lock().expect("lists").is_empty());
}
#[tokio::test]
async fn test_semantic_errors_limits_and_offline_policy_do_not_fallback() {
    let configuration = configuration(
        vec![binding("mcp:resolve", "acorn.resolve", "research", "resolve")],
        &[("research", &["resolve"])],
    );
    let backend = FakeBackend::new(
        BTreeMap::from([("research".to_string(), vec![tool("resolve", "query")])]),
        CallBehavior::Result(CallToolResult::structured_error(json!({"secret":"not exposed"}))),
    );
    let registry = ToolRegistry::default()
        .register_mcp_bindings_with(&configuration, backend.clone(), InvocationContext::default())
        .await
        .expect("registration");
    assert!(registry
        .select_capability_binding(
            "acorn.resolve",
            Some("mcp:resolve"),
            None,
            CapabilitySelectionContext {
                offline: true,
                allow_remote: true,
            },
        )
        .is_err());
    let selected = registry
        .select_capability_binding("acorn.resolve", Some("mcp:resolve"), None, remote_selection())
        .expect("selection");
    let result = registry
        .execute_selected(
            &selected,
            json!({"query":"needle"}),
            ExecutionContext {
                grounded: true,
                selection: remote_selection(),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(result.outcome, ExecutionOutcome::ToolFailed);
    assert!(result.content.is_none());
    assert_eq!(backend.calls.lock().expect("calls").len(), 1);
    let backend = FakeBackend::new(
        BTreeMap::from([("research".to_string(), vec![tool("resolve", "query")])]),
        CallBehavior::Result(CallToolResult::structured(json!({"value":"too large"}))),
    );
    let registry = ToolRegistry::default()
        .register_mcp_bindings_with(&configuration, backend.clone(), InvocationContext::default())
        .await
        .expect("registration");
    let selected = registry
        .select_capability_binding("acorn.resolve", None, None, remote_selection())
        .expect("selection");
    let limited = ExecutionContext {
        grounded: true,
        max_output: Memory::from(1_u64),
        selection: remote_selection(),
        ..ExecutionContext::default()
    };
    assert_eq!(
        registry.execute_selected(&selected, json!({"query":"needle"}), limited).await.outcome,
        ExecutionOutcome::ToolFailed
    );
    assert_eq!(backend.calls.lock().expect("calls").len(), 1);
}
