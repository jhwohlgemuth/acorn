use super::super::script::*;
use crate::io::config::application::Hook;
use crate::io::mcp::tools::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome};
use crate::io::mcp::ToolRegistry;
use crate::io::Bytes;
use serde_json::json;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[cfg(not(unix))]
fn secure_file(_path: &Path) {}
#[cfg(unix)]
fn secure_file(path: &Path) {
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("secure script fixture permissions");
}

#[tokio::test]
async fn test_direct_script_check_returns_findings_without_artifacts() {
    let root = tempdir().expect("script root");
    let script = root.path().join("valid.rhai");
    fs::copy(crate::test::utils::fixture_path("exec/valid.rhai"), &script).expect("copy valid script");
    secure_file(&script);
    fs::write(root.path().join("input.txt"), "hello").expect("input document");
    let registry = ToolRegistry::default()
        .register_exec_check_capability(root.path())
        .expect("register script check");
    let selected = registry
        .select_capability_binding("acorn.exec::check", None, None, CapabilitySelectionContext::default())
        .expect("select script check");
    let result = registry
        .execute_selected(
            &selected,
            json!({"script":"valid.rhai","input":"input.txt"}),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(result.outcome, ExecutionOutcome::Completed, "{result:?}");
    assert!(result.artifacts.is_empty());
    let output = ScriptCheckInput {
        script: PathBuf::from("valid.rhai"),
        input: PathBuf::from("input.txt"),
    }
    .execute(root.path())
    .expect("direct check operation");
    assert_eq!(output.failures, 1);
    assert_eq!(output.findings.len(), 1);
    assert_eq!(output.findings.first().expect("finding").message, "example");
    assert_eq!(fs::read_to_string(root.path().join("input.txt")).expect("unchanged input"), "hello");
    assert!(ScriptCheckInput {
        script: PathBuf::from("valid.rhai"),
        input: PathBuf::from("../escape.txt"),
    }
    .execute(root.path())
    .is_err());
}
#[tokio::test]
async fn test_hook_check_requires_digest_authorization_and_applies_no_proposals() {
    let root = tempdir().expect("hook root");
    let script = root.path().join("valid.rhai");
    fs::copy(crate::test::utils::fixture_path("exec/valid.rhai"), &script).expect("copy valid script");
    secure_file(&script);
    fs::write(root.path().join("input.txt"), "hello").expect("input document");
    let source = fs::read_to_string(&script).expect("script source");
    let digest = Bytes::from(&source).digest();
    let manifest = json!({
        "version":"1.0.0",
        "scripts":[{
            "id":"quality","version":"0.2.0","entry":"evaluate","digest":digest.clone(),
            "hooks":["check"],"capabilities":{"read":null,"write":null,"network":null}
        }]
    });
    let manifest_path = root.path().join("manifest.json");
    fs::write(&manifest_path, serde_json::to_vec(&manifest).expect("manifest JSON")).expect("write manifest");
    secure_file(&manifest_path);
    let registry = ToolRegistry::default()
        .register_exec_hook_check_capability(root.path(), true)
        .expect("register hook check");
    let selected = registry
        .select_capability_binding("acorn.exec::hook::check", None, None, CapabilitySelectionContext::default())
        .expect("select hook check");
    let result = registry
        .execute_selected(
            &selected,
            json!({"script":"valid.rhai","manifest":"manifest.json","hook":"check","input":"input.txt"}),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(result.outcome, ExecutionOutcome::Completed, "{result:?}");
    assert!(result.artifacts.is_empty());
    let output = ScriptHookInput {
        script: PathBuf::from("valid.rhai"),
        manifest: PathBuf::from("manifest.json"),
        hook: Hook::Check,
        input: PathBuf::from("input.txt"),
    }
    .execute(root.path(), true)
    .expect("native hook evaluation");
    assert_eq!(output.failures, 1);
    assert_eq!(output.findings.first().expect("finding").message, "example");
    assert!(output.files.is_empty());
    assert!(output.used_effects.is_empty());
    assert_eq!(fs::read_to_string(root.path().join("input.txt")).expect("unchanged input"), "hello");
    let outside = tempdir().expect("outside root");
    let escaped = json!({"version":"1.0.0","scripts":[{
        "id":"quality","version":"0.2.0","entry":"evaluate","digest":digest,
        "hooks":["check"],"capabilities":{"read":{"roots":{"outside":outside.path()}},"write":null,"network":null}
    }]});
    fs::write(
        root.path().join("manifest.json"),
        serde_json::to_vec(&escaped).expect("escaped manifest JSON"),
    )
    .expect("replace manifest");
    let escaped_error = ScriptHookInput {
        script: PathBuf::from("valid.rhai"),
        manifest: PathBuf::from("manifest.json"),
        hook: Hook::Check,
        input: PathBuf::from("input.txt"),
    }
    .execute(root.path(), true)
    .expect_err("capability root must be contained");
    assert!(escaped_error.to_string().contains("escapes the permitted root"));
    let invalid = json!({"version":"1.0.0","scripts":[{"id":"quality","version":"0.2.0","entry":"evaluate","digest":"sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad","hooks":["check"]}]});
    fs::write(
        root.path().join("manifest.json"),
        serde_json::to_vec(&invalid).expect("invalid manifest JSON"),
    )
    .expect("replace manifest");
    assert!(ScriptHookInput {
        script: PathBuf::from("valid.rhai"),
        manifest: PathBuf::from("manifest.json"),
        hook: Hook::Check,
        input: PathBuf::from("input.txt"),
    }
    .execute(root.path(), true)
    .is_err());
    let network_source = "// @acorn-exec-v0.2.0\nfn evaluate(context) { http_get(\"https://example.org\"); #{findings: []} }";
    fs::write(&script, network_source).expect("network script");
    let network_manifest = json!({"version":"1.0.0","scripts":[{
        "id":"network","version":"0.2.0","entry":"evaluate","digest":Bytes::from(network_source).digest(),
        "hooks":["check"],"capabilities":{"read":null,"write":null,"network":{"hosts":"example.org"}}
    }]});
    fs::write(
        root.path().join("manifest.json"),
        serde_json::to_vec(&network_manifest).expect("network manifest JSON"),
    )
    .expect("replace manifest");
    let offline_error = ScriptHookInput {
        script: PathBuf::from("valid.rhai"),
        manifest: PathBuf::from("manifest.json"),
        hook: Hook::Check,
        input: PathBuf::from("input.txt"),
    }
    .execute(root.path(), true)
    .expect_err("offline network call must fail");
    assert!(offline_error.to_string().contains("offline"), "{offline_error}");
}
#[tokio::test]
async fn test_static_script_routes_are_read_only_and_bounded() {
    let root = tempdir().expect("script root");
    let valid_script = root.path().join("valid.rhai");
    fs::copy(crate::test::utils::fixture_path("exec/valid.rhai"), &valid_script).expect("copy valid script");
    secure_file(&valid_script);
    let invalid_script = root.path().join("invalid.rhai");
    fs::copy(crate::test::utils::fixture_path("exec/invalid_no_header.rhai"), &invalid_script).expect("copy invalid script");
    secure_file(&invalid_script);
    let manifest_path = root.path().join("manifest.json");
    fs::write(&manifest_path, r#"{"version":"1.0.0","scripts":[]}"#).expect("manifest");
    secure_file(&manifest_path);
    let registry = ToolRegistry::default()
        .register_exec_check_safe_capability(root.path())
        .and_then(|registry| registry.register_exec_manifest_check_capability(root.path()))
        .expect("register static checks");
    let safe = registry
        .select_capability_binding("acorn.exec::check::safe", None, None, CapabilitySelectionContext::default())
        .expect("select safe check");
    let checked = registry
        .execute_selected(
            &safe,
            json!({"script":"valid.rhai"}),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(checked.outcome, ExecutionOutcome::Completed);
    assert!(checked.artifacts.is_empty());
    let invalid = registry
        .execute_selected(
            &safe,
            json!({"script":"invalid.rhai"}),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_ne!(invalid.outcome, ExecutionOutcome::Completed);
    assert!(ScriptSafeInput {
        script: PathBuf::from("../escape.rhai")
    }
    .execute(root.path())
    .is_err());
    let manifest = registry
        .select_capability_binding("acorn.exec::manifest::check", None, None, CapabilitySelectionContext::default())
        .expect("select manifest check");
    let checked_manifest = registry
        .execute_selected(
            &manifest,
            json!({"manifest":"manifest.json"}),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(checked_manifest.outcome, ExecutionOutcome::Completed);
    assert!(checked_manifest.artifacts.is_empty());
    assert!(ScriptManifestInput {
        manifest: PathBuf::from("../escape.json")
    }
    .execute(root.path())
    .is_err());
}
