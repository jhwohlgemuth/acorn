use super::{
    endpoint_mode, endpoint_options, native_routing_registry, remote_allowed, resolve_prompt, routing_registry, run_routed, structured_input,
    Backend, ConfiguredInference, InferenceArgs, InferenceBackendKind, Milliseconds,
};
use crate::cli::arguments::{AcpAgentKind, InferenceEndpointMode};
use crate::cli::{Arguments, Commands};
use acorn::io::api::{openai::EndpointMode, Configuration};
use acorn::io::config::ApplicationConfiguration;
use acorn::io::mcp::config::{McpServerConfiguration, McpToolConfiguration, McpTransportConfiguration};
use acorn::io::mcp::{ExecutorBindingConfiguration, ExecutorConfiguration};
use acorn_core::time::UnitOfTime;
use acorn_schema::hardware::Memory;
use acorn_schema::software::Software;
use alloc::collections::BTreeMap;
use clap::Parser;
use core::cell::Cell;
use serde_json::json;
use std::path::PathBuf;

mod chat;
mod metadata;
mod prompt;

fn arguments(backend: InferenceBackendKind) -> InferenceArgs {
    InferenceArgs::init().backend(backend).working_directory(PathBuf::from(".")).build()
}

#[test]
fn test_acp_is_the_default_backend_shape() {
    let configured = ConfiguredInference::from_args(&arguments(InferenceBackendKind::Acp), false).expect("configure ACP");
    assert_eq!(configured.agent.as_deref(), Some(Software::OpenCode.command()));
    assert_eq!(configured.timeout, Milliseconds::new(30_000));
}
#[test]
fn test_acp_selects_codex_profile_and_executable_override() {
    let arguments = InferenceArgs::init()
        .backend(InferenceBackendKind::Acp)
        .agent(AcpAgentKind::Codex)
        .agent_executable(PathBuf::from("custom/codex-acp"))
        .working_directory(PathBuf::from("."))
        .build();
    let configured = ConfiguredInference::from_args(&arguments, false).expect("configure Codex ACP");
    assert_eq!(configured.agent.as_deref(), Some(Software::Codex.command()));
    assert!(matches!(
        configured.backend,
        Backend::Acp(ref client) if client.config().executable() == std::path::Path::new("custom/codex-acp")
    ));
}
#[test]
fn test_openai_requires_a_model_and_rejects_acp_flags() {
    let missing = ConfiguredInference::from_args(&arguments(InferenceBackendKind::Openai), false);
    assert!(missing.is_err());
    let configured = InferenceArgs::init()
        .backend(InferenceBackendKind::Openai)
        .model("model".to_string())
        .build();
    assert!(ConfiguredInference::from_args(&configured, false).is_ok());
}
#[test]
fn test_ollama_selector_uses_local_chat_defaults_and_explicit_overrides() {
    let missing = InferenceArgs::init().backend(InferenceBackendKind::Ollama).build();
    assert!(ConfiguredInference::from_args(&missing, false)
        .expect_err("Ollama needs an explicit model")
        .to_string()
        .contains("--model is required with --backend ollama"));
    let defaults = InferenceArgs::init()
        .backend(InferenceBackendKind::Ollama)
        .model("local-model".to_string())
        .build();
    assert_eq!(endpoint_options(&defaults).domain(), "http://localhost:11434");
    assert_eq!(endpoint_mode(&defaults), EndpointMode::ChatCompletions);
    assert!(ConfiguredInference::from_args(&defaults, true).is_ok());
    let overrides = InferenceArgs::init()
        .backend(InferenceBackendKind::Ollama)
        .model("remote-model".to_string())
        .base_url("https://ollama.example.org".to_string())
        .endpoint_mode(InferenceEndpointMode::Responses)
        .build();
    assert_eq!(endpoint_options(&overrides).domain(), "https://ollama.example.org");
    assert_eq!(endpoint_mode(&overrides), EndpointMode::Responses);
    let invalid = InferenceArgs::init()
        .backend(InferenceBackendKind::Ollama)
        .model("local-model".to_string())
        .agent(AcpAgentKind::Codex)
        .build();
    assert!(ConfiguredInference::from_args(&invalid, false).is_err());
}
#[test]
fn test_resolve_prompt_prefers_the_argument() {
    let read = Cell::new(false);
    let prompt = resolve_prompt(Some(" argument "), || {
        read.set(true);
        None
    })
    .expect("argument prompt");
    assert_eq!(prompt, "argument");
    assert!(!read.get());
}
#[test]
fn test_resolve_prompt_reads_standard_input() {
    let prompt = resolve_prompt(None, || Some(" piped prompt\n".to_string())).expect("piped prompt");
    assert_eq!(prompt, "piped prompt");
}
#[test]
fn test_resolve_prompt_rejects_missing_input() {
    assert!(resolve_prompt(None, || None).is_err());
    assert!(resolve_prompt(None, || Some(" \n".to_string())).is_err());
}
#[test]
fn test_routed_input_and_default_registry_contract() {
    assert_eq!(structured_input(Some("{}")).expect("object"), Some(serde_json::json!({})));
    assert!(structured_input(Some("[]")).is_err());
    assert!(structured_input(Some("{")).is_err());
    let registry = native_routing_registry(true).expect("route registry");
    let catalog = registry
        .capability_catalog(None, acorn::io::mcp::CapabilitySelectionContext::default())
        .expect("catalog");
    assert!(catalog.contains("acorn.version"));
    assert!(catalog.contains("acorn.check"));
}
#[tokio::test]
async fn test_routed_registry_composes_configured_local_bindings() {
    let configuration = ApplicationConfiguration::parse(
        json!({
            "endpoints":[{"name":"ollama","domain":"127.0.0.1","scheme":"http","port":11434,"resources":[]}],
            "executors":[
                {"id":"acp:summary","capabilityId":"acorn.summary","executor":{"type":"acp","profile":"codex"}},
                {"id":"endpoint:summary","capabilityId":"acorn.summary","executor":{"type":"endpoint","endpoint":"ollama","model":"fixture"}},
                {"id":"sidecar:future","capabilityId":"acorn.future","executor":{"type":"sidecar","profile":"future"}}
            ]
        })
        .to_string(),
    )
    .expect("configured adapters");
    let registry = routing_registry(&configuration, false).await.expect("compose configured bindings");
    assert!(registry
        .select_capability_binding("acorn.summary", Some("acp:summary"), None, Default::default())
        .is_ok());
    assert!(registry
        .select_capability_binding("acorn.summary", Some("endpoint:summary"), None, Default::default())
        .is_ok());
    let unavailable = registry
        .select_capability_binding("acorn.future", Some("sidecar:future"), None, Default::default())
        .expect_err("future sidecar unavailable");
    assert!(unavailable.to_string().contains("not installed"));
    assert!(!remote_allowed(&configuration, false));
}
#[tokio::test]
async fn test_routed_registry_skips_mcp_discovery_offline() {
    let configuration = ApplicationConfiguration {
        executors: vec![ExecutorBindingConfiguration {
            id: "mcp:fixture".to_string(),
            capability_id: "acorn.fixture".to_string(),
            executor: ExecutorConfiguration::Mcp {
                server: "fixture".to_string(),
                tool: "summarize".to_string(),
            },
        }],
        mcp: BTreeMap::from([(
            "fixture".to_string(),
            McpServerConfiguration {
                allowed: BTreeMap::from([(
                    "summarize".to_string(),
                    McpToolConfiguration {
                        effects: Vec::new(),
                        max_output: Memory::mb(1),
                        timeout: acorn_core::time::Seconds::new(30),
                    },
                )]),
                transport: McpTransportConfiguration::ChildProcess {
                    args: Vec::new(),
                    command: "must-not-start".to_string(),
                    environment: BTreeMap::new(),
                },
            },
        )]),
        ..ApplicationConfiguration::default()
    };
    let registry = routing_registry(&configuration, true)
        .await
        .expect("offline registry skips MCP discovery");
    assert!(registry
        .select_capability_binding("acorn.version", None, None, Default::default())
        .is_ok());
    assert!(!remote_allowed(&configuration, true));
}
#[test]
fn test_remote_eligibility_requires_explicit_policy_and_online_mode() {
    let configuration = ApplicationConfiguration::parse(json!({
        "endpoints":[{"name":"remote","domain":"example.org","scheme":"https","resources":[]}],
        "executors":[{"id":"endpoint:remote","capabilityId":"acorn.remote","executor":{"type":"endpoint","endpoint":"remote","model":"fixture","data_policy":"allow-remote"}}]
    }).to_string()).expect("remote endpoint configuration");
    assert!(remote_allowed(&configuration, false));
    assert!(!remote_allowed(&configuration, true));
    assert!(!remote_allowed(&ApplicationConfiguration::default(), false));
}
#[tokio::test]
async fn test_routed_request_rejects_legacy_round_limit() {
    let parsed = Arguments::try_parse_from(["acorn", "ask", "version", "--route", "--max-rounds", "2"]).expect("parse route");
    let arguments = parsed
        .command
        .and_then(|command| match command {
            | Commands::Infer(arguments) => Some(arguments),
            | _ => None,
        })
        .expect("infer command");
    let error = run_routed(&arguments, true).await.expect_err("round limit belongs to legacy mode");
    assert!(error.to_string().contains("legacy Needle loop"));
}
#[tokio::test]
async fn test_routed_request_rejects_unknown_scope_before_starting_needle() {
    let parsed = Arguments::try_parse_from(["acorn", "ask", "check this", "--route", "--capability", "acorn.unknown"]).expect("parse route");
    let arguments = parsed
        .command
        .and_then(|command| match command {
            | Commands::Infer(arguments) => Some(arguments),
            | _ => None,
        })
        .expect("infer command");
    let error = run_routed(&arguments, true).await.expect_err("unknown capability");
    assert!(error.to_string().contains("Unknown capability scope"));
}
