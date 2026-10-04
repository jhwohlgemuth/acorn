//! Outbound MCP client interoperability tests using ACORN's stdio MCP server.
use serde_json::{json, Value};
use std::fs::write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::{tempdir, TempDir};

fn configuration() -> Result<(TempDir, PathBuf), Box<dyn core::error::Error>> {
    let directory = tempdir()?;
    let config = directory.path().join(".acorn.yml");
    let document = json!({
        "mcp": {
            "local": {
                "transport": {
                    "type": "child-process",
                    "command": env!("CARGO_BIN_EXE_acorn"),
                    "args": ["serve", "mcp"]
                },
                "allowed": {
                    "acorn.version": {
                        "timeout": 10,
                        "max_output": "4KB"
                    }
                }
            }
        }
    });
    write(&config, serde_norway::to_string(&document)?)?;
    Ok((directory, config))
}
fn invoke(config: &Path, arguments: &[&str]) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_acorn"))
        .args(["mcp", "--config"])
        .arg(config)
        .args(arguments)
        .output()
}

#[test]
fn test_acorn_mcp_client_communicates_with_acorn_mcp_server() -> Result<(), Box<dyn core::error::Error>> {
    let (_directory, config) = configuration()?;
    let listed = invoke(&config, &["list", "local"])?;
    assert!(listed.status.success(), "{}", String::from_utf8_lossy(&listed.stderr));
    let tools = serde_json::from_slice::<Vec<Value>>(&listed.stdout)?;
    assert_eq!(tools.len(), 1);
    assert_eq!(
        tools.first().and_then(|tool| tool.get("name")).and_then(Value::as_str),
        Some("acorn.version")
    );
    let called = invoke(&config, &["call", "local", "acorn.version", "--arguments", "{}"])?;
    assert!(called.status.success(), "{}", String::from_utf8_lossy(&called.stderr));
    let result = serde_json::from_slice::<Value>(&called.stdout)?;
    assert_eq!(
        result.pointer("/structuredContent/version").and_then(Value::as_str),
        Some(env!("CARGO_PKG_VERSION"))
    );
    Ok(())
}
#[test]
fn test_acorn_mcp_client_enforces_policy_and_surfaces_server_errors() -> Result<(), Box<dyn core::error::Error>> {
    let (_directory, config) = configuration()?;
    let blocked = invoke(&config, &["call", "local", "acorn.hash", "--arguments", "{}"])?;
    assert!(!blocked.status.success());
    assert!(blocked.stdout.is_empty());
    assert!(String::from_utf8_lossy(&blocked.stderr).contains("not allowlisted"));
    let failed = invoke(&config, &["call", "local", "acorn.version", "--arguments", r#"{"unexpected":true}"#])?;
    assert!(!failed.status.success());
    let result = serde_json::from_slice::<Value>(&failed.stdout)?;
    assert_eq!(result.get("isError").and_then(Value::as_bool), Some(true));
    Ok(())
}
