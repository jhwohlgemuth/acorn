//! Direct one-shot ask equivalence with the shared Needle service.
use acorn::io::{
    mcp::NeedleStatusOutput,
    sidecar::needle::{InferenceResult, Needle},
    sidecar::Sidecar,
};
use serde_json::Value;
use std::process::{Command, Stdio};
use tempfile::tempdir;

#[tokio::test]
async fn test_legacy_ask_matches_pinned_needle_service() {
    let status = NeedleStatusOutput::inspect().await.expect("pinned cache status");
    let root = tempdir().expect("isolated ask directory");
    let prompt = "What is the ACORN version?";
    let direct = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .args(["--offline", "--no-local-database", "ask", prompt])
        .current_dir(root.path())
        .stdin(Stdio::null())
        .output()
        .expect("direct ask command");
    assert_eq!(direct.status.success(), status.ready, "{}", String::from_utf8_lossy(&direct.stderr));
    if !status.ready {
        return;
    }
    let direct: InferenceResult = serde_json::from_slice(&direct.stdout).expect("direct Needle JSON");
    let shared = Needle::start(None, true)
        .await
        .expect("start pinned Needle")
        .infer(prompt.to_string())
        .await
        .expect("shared Needle inference");
    assert!(direct.response.success);
    assert!(shared.response.success);
    let direct_results = serde_json::to_value(&direct.results).expect("direct results");
    let shared_results = serde_json::to_value(&shared.results).expect("shared results");
    let version = env!("CARGO_PKG_VERSION");
    assert_eq!(direct_results, shared_results);
    let direct_version = direct_results
        .get(0)
        .and_then(|value| value.get("structured_content"))
        .and_then(|value| value.get("version"))
        .expect("structured version");
    assert_eq!(direct_version, &Value::String(version.to_string()));
    assert!(direct.withheld.is_none());
    assert!(shared.withheld.is_none());
    assert!(direct.response.function_calls.is_empty());
    assert!(shared.response.function_calls.is_empty());
    assert_eq!(std::fs::read_dir(root.path()).expect("isolated directory").count(), 0);
}

#[test]
fn test_legacy_ask_rejects_missing_prompt_without_starting_needle() {
    let result = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .args(["--offline", "--no-local-database", "ask"])
        .stdin(Stdio::null())
        .output()
        .expect("direct ask command");
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("prompt"));
}
