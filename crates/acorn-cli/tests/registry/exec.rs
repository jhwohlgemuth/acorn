//! Direct CLI equivalence for read-only static script checks.
#![cfg(feature = "exec")]
use acorn::io::config::application::Hook;
use acorn::io::mcp::{ScriptCheckInput, ScriptHookInput, ScriptManifestInput, ScriptSafeInput};
use acorn::io::Bytes;
use serde_json::json;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_direct_script_findings_match_native_check() {
    let root = tempdir().expect("script root");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/exec/valid.rhai");
    fs::copy(fixture, root.path().join("valid.rhai")).expect("copy valid script");
    fs::write(root.path().join("input.txt"), "hello").expect("input document");
    let direct = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .current_dir(root.path())
        .args(["--offline", "--no-local-database", "exec", "check", "valid.rhai", "input.txt"])
        .output()
        .expect("run direct script check");
    let native = ScriptCheckInput {
        script: PathBuf::from("valid.rhai"),
        input: PathBuf::from("input.txt"),
    }
    .execute(root.path())
    .expect("native script check");
    assert_eq!(native.failures, 1);
    assert!(!direct.status.success());
    assert!(String::from_utf8_lossy(&direct.stderr).contains("exec check found 1 failure(s)"));
    assert!(String::from_utf8_lossy(&direct.stdout).contains(&native.findings.first().expect("finding").message));
    assert_eq!(fs::read_to_string(root.path().join("input.txt")).expect("unchanged input"), "hello");
}
#[test]
fn test_manifest_authorized_hook_matches_direct_command() {
    let root = tempdir().expect("hook root");
    let script = root.path().join("valid.rhai");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/exec/valid.rhai");
    fs::copy(fixture, &script).expect("copy valid script");
    let input = root.path().join("input.txt");
    fs::write(&input, "hello").expect("input document");
    let source = fs::read_to_string(&script).expect("script source");
    let manifest = root.path().join("manifest.json");
    let authorization = json!({"version":"1.0.0","scripts":[{
        "id":"quality","version":"0.2.0","entry":"evaluate","digest":Bytes::from(&source).digest(),
        "hooks":["check"],"capabilities":{"read":null,"write":null,"network":null}
    }]});
    fs::write(&manifest, serde_json::to_vec(&authorization).expect("manifest JSON")).expect("write manifest");
    let direct = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .args(["--offline", "--no-local-database", "exec", "hook", "check"])
        .arg(&script)
        .arg("--manifest")
        .arg(&manifest)
        .args(["--hook", "check", "--input"])
        .arg(&input)
        .output()
        .expect("run direct hook check");
    let native = ScriptHookInput {
        script: PathBuf::from("valid.rhai"),
        manifest: PathBuf::from("manifest.json"),
        hook: Hook::Check,
        input: PathBuf::from("input.txt"),
    }
    .execute(root.path(), true)
    .expect("native hook check");
    assert_eq!(native.failures, 1);
    assert!(!direct.status.success());
    assert!(String::from_utf8_lossy(&direct.stderr).contains("exec hook check found 1 failure(s)"));
    assert!(String::from_utf8_lossy(&direct.stdout).contains(&native.findings.first().expect("finding").message));
    assert_eq!(fs::read_to_string(&input).expect("unchanged input"), "hello");
}
#[test]
fn test_static_script_and_manifest_match_direct_commands() {
    let root = tempdir().expect("script root");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/exec/valid.rhai");
    fs::copy(fixture, root.path().join("valid.rhai")).expect("copy valid script");
    fs::write(root.path().join("manifest.json"), r#"{"version":"1.0.0","scripts":[]}"#).expect("write manifest");
    let direct_safe = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .current_dir(root.path())
        .args(["--offline", "--no-local-database", "exec", "check-safe", "valid.rhai", "--format", "json"])
        .output()
        .expect("run direct static check");
    assert!(direct_safe.status.success(), "{}", String::from_utf8_lossy(&direct_safe.stderr));
    let native_safe = ScriptSafeInput {
        script: PathBuf::from("valid.rhai"),
    }
    .execute(root.path())
    .expect("native static check");
    let direct_json: Value = serde_json::from_slice(&direct_safe.stdout).expect("direct JSON status");
    assert_eq!(serde_json::to_value(native_safe).expect("native JSON status"), direct_json);
    let direct_manifest = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .current_dir(root.path())
        .args(["--offline", "--no-local-database", "exec", "manifest", "check", "manifest.json"])
        .output()
        .expect("run direct manifest check");
    assert!(direct_manifest.status.success(), "{}", String::from_utf8_lossy(&direct_manifest.stderr));
    let native_manifest = ScriptManifestInput {
        manifest: PathBuf::from("manifest.json"),
    }
    .execute(root.path())
    .expect("native manifest check");
    let text = String::from_utf8_lossy(&direct_manifest.stdout);
    assert!(text.contains(&format!(
        "Manifest `{}` authorizes {} script(s)",
        native_manifest.version, native_manifest.scripts
    )));
}
