//! Integration tests for terminal formatting behavior.
use std::process::{Command, Stdio};

#[test]
fn test_interactive_format_rejects_non_terminal_streams() {
    let output = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .args(["--no-local-database", "format", "--interactive"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run acorn with non-terminal streams");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "interactive format should reject non-terminal streams");
    assert!(stderr.contains("terminal input and output"), "unexpected error: {stderr}");
}
