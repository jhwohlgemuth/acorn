#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::panic)]
//! End-to-end MCP stdio protocol coverage.
#[cfg(unix)]
use acorn::io::extract;
#[cfg(unix)]
use acorn_core::util::MimeType;
use serde_json::{json, Value};
#[cfg(unix)]
use std::fs::{metadata, set_permissions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

fn assert_stdio_session(binary: &Path) {
    let mut child = Command::new(binary)
        .args(["--no-local-database", "serve", "mcp"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ACORN MCP server");
    let requests = [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"acorn-test","version":"1"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"acorn.version","arguments":{}}}),
        json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"acorn.missing","arguments":{}}}),
    ];
    let input = requests.iter().map(Value::to_string).collect::<Vec<_>>().join("\n") + "\n";
    child
        .stdin
        .take()
        .expect("MCP stdin")
        .write_all(input.as_bytes())
        .expect("write MCP requests");
    let output = child.wait_with_output().expect("wait for MCP server");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let frames = String::from_utf8(output.stdout)
        .expect("UTF-8 MCP stdout")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<Value>(line).expect("stdout contains only JSON-RPC frames"))
        .collect::<Vec<_>>();
    assert!(frames
        .iter()
        .any(|frame| frame.get("id") == Some(&json!(2)) && frame.pointer("/result/tools").is_some()));
    assert!(frames.iter().any(|frame| {
        frame.get("id") == Some(&json!(3))
            && frame.pointer("/result/structuredContent/version").and_then(Value::as_str) == Some(env!("CARGO_PKG_VERSION"))
    }));
    assert!(frames
        .iter()
        .any(|frame| frame.get("id") == Some(&json!(4)) && frame.get("error").is_some()));
}

#[test]
#[cfg(unix)]
fn test_created_bundle_runs_stdio_mcp_session() {
    let root = tempfile::tempdir().expect("temporary directory");
    let bundle = root.path().join("acorn.mcpb");
    let platform = if cfg!(target_os = "macos") { "darwin" } else { "linux" };
    let status = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .args([
            "create",
            "mcp",
            "--binary",
            env!("CARGO_BIN_EXE_acorn"),
            "--platform",
            platform,
            "--output",
        ])
        .arg(&bundle)
        .status()
        .expect("create MCP Bundle");
    assert!(status.success());
    let extracted = extract(bundle, Some(root.path().join("extracted")), Some(MimeType::Zip)).expect("extract MCP Bundle");
    let binary = extracted.join("server/acorn");
    let mut permissions = metadata(&binary).expect("bundle executable metadata").permissions();
    permissions.set_mode(0o755);
    set_permissions(&binary, permissions).expect("make bundle executable runnable");
    assert_stdio_session(&binary);
}
#[test]
fn test_stdio_mcp_session_has_protocol_only_stdout() {
    assert_stdio_session(Path::new(env!("CARGO_BIN_EXE_acorn")));
}
