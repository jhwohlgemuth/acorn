use crate::io::api::operation::OperationEffect;
use crate::io::mcp::config::{McpServerConfiguration, McpToolConfiguration, McpTransportConfiguration};
use acorn_core::time::{Seconds, UnitOfTime};
use acorn_schema::hardware::Memory;
use acorn_schema::validation::Validate;
use alloc::collections::BTreeMap;

fn child_server() -> McpServerConfiguration {
    McpServerConfiguration {
        allowed: BTreeMap::from([("version".to_string(), tool_configuration())]),
        transport: McpTransportConfiguration::ChildProcess {
            args: Vec::new(),
            command: "server".to_string(),
            environment: BTreeMap::new(),
        },
    }
}
fn tool_configuration() -> McpToolConfiguration {
    McpToolConfiguration {
        effects: Vec::new(),
        max_output: Memory::mb(1),
        timeout: Seconds::new(30),
    }
}

#[test]
fn test_configuration_rejects_duplicate_effects_and_zero_limits() {
    let tool = McpToolConfiguration {
        effects: vec![OperationEffect::Mutation, OperationEffect::Mutation],
        max_output: Memory::mb(0),
        timeout: Seconds::new(0),
    };
    assert_eq!(tool.validate().expect_err("invalid tool").len(), 3);
}
#[test]
fn test_configuration_transport_debug_is_redacted() {
    let transport = McpTransportConfiguration::ChildProcess {
        args: vec!["--token".to_string(), "literal-secret".to_string()],
        command: "secret-command".to_string(),
        environment: BTreeMap::from([("TOKEN".to_string(), "SECRET_ENV".to_string())]),
    };
    let debug = format!("{transport:?}");
    ["SECRET_ENV", "literal-secret", "secret-command"]
        .into_iter()
        .for_each(|secret| assert!(!debug.contains(secret)));
}
#[test]
fn test_server_name_validates_separately() {
    assert!(McpServerConfiguration::validate_name("").is_err());
    assert!(McpServerConfiguration::validate_name("research").is_ok());
    assert!(child_server().validate().is_ok());
}
#[test]
fn test_server_requires_an_allowlist() {
    let server = McpServerConfiguration {
        allowed: BTreeMap::new(),
        ..child_server()
    };
    assert!(server.validate().is_err());
}
#[test]
fn test_streamable_http_requires_secure_remote_urls() {
    ["http://example.org/mcp", "file:///tmp/mcp", "https://user:secret@example.org/mcp"]
        .into_iter()
        .for_each(|url| {
            let transport = McpTransportConfiguration::StreamableHttp {
                bearer_token_env: None,
                url: url.to_string(),
            };
            assert!(transport.validate().is_err(), "accepted {url}");
        });
    ["https://example.org/mcp", "http://localhost:3000/mcp", "http://127.0.0.1/mcp"]
        .into_iter()
        .for_each(|url| {
            let transport = McpTransportConfiguration::StreamableHttp {
                bearer_token_env: None,
                url: url.to_string(),
            };
            assert!(transport.validate().is_ok(), "rejected {url}");
        });
}
#[test]
fn test_tagged_transport_rejects_unknown_fields() {
    let valid = serde_json::from_str::<McpTransportConfiguration>(r#"{"type":"child-process","command":"server"}"#);
    let invalid = serde_json::from_str::<McpTransportConfiguration>(r#"{"type":"child-process","command":"server","url":"secret"}"#);
    assert!(valid.is_ok());
    assert!(invalid.is_err());
}
