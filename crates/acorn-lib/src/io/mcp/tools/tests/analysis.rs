use super::*;
use crate::io::mcp::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome, InputReference, ToolRegistry};
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}
fn input(path: &str) -> AnalysisInput {
    AnalysisInput {
        input: InputReference::File(PathBuf::from(path)),
        standard: None,
        format: None,
        skip: Vec::new(),
        filter: Vec::new(),
        ignore: Vec::new(),
        disable_website_checks: false,
    }
}

#[tokio::test]
async fn test_category_selection_and_invalid_selection_are_explicit() {
    let mut selected = input("CITATION.cff");
    selected.skip = vec!["schema".to_string()];
    let result = selected.analyze(&fixture_root(), true).await.expect("analysis");
    assert_eq!(result.skipped_categories, vec!["schema"]);
    let mut invalid = input("CITATION.cff");
    invalid.skip = vec!["not-a-category".to_string()];
    assert!(invalid.analyze(&fixture_root(), true).await.is_err());
}
#[tokio::test]
async fn test_directory_results_are_stable_and_input_failures_are_reported() {
    let request = AnalysisInput {
        input: InputReference::Directory(PathBuf::from("data/valid_project")),
        standard: None,
        format: None,
        skip: Vec::new(),
        filter: Vec::new(),
        ignore: Vec::new(),
        disable_website_checks: true,
    };
    let first = request.clone().analyze(&fixture_root(), true).await.expect("first");
    let second = request.analyze(&fixture_root(), true).await.expect("second");
    assert_eq!(serde_json::to_value(first).unwrap(), serde_json::to_value(second).unwrap());
}
#[tokio::test]
async fn test_filter_and_ignore_select_only_confirmed_files() {
    let root = fixture_root();
    let mut request = AnalysisInput {
        input: InputReference::Directory(PathBuf::from("data/valid_project")),
        standard: None,
        format: None,
        skip: Vec::new(),
        filter: vec!["index[.]json$".to_string()],
        ignore: Vec::new(),
        disable_website_checks: true,
    };
    let selected = request.clone().analyze(&root, true).await.expect("filtered analysis");
    assert!(selected
        .batches
        .iter()
        .flat_map(|batch| &batch.paths)
        .all(|path| path.ends_with("index.json")));
    request.ignore = vec!["index[.]json$".to_string()];
    assert!(request.clone().analyze(&root, true).await.is_err());
    request.ignore = vec!["[".to_string()];
    assert!(request.analyze(&root, true).await.is_err());
}
#[tokio::test]
async fn test_full_local_check_reports_structured_categories_offline() {
    let result = input("data/valid_project/index.json")
        .analyze(&fixture_root(), true)
        .await
        .expect("analysis");
    assert_eq!(result.batches.len(), 1);
    let batch = result.batches.first().expect("one batch");
    assert!(!batch.categories.is_empty());
    assert!(batch.categories.iter().any(|category| category.category == "schema"));
    assert!(batch
        .categories
        .iter()
        .any(|category| category.category == "link" && category.status == AnalysisCategoryStatus::Skipped));
}
#[tokio::test]
async fn test_git_commit_and_branch_select_changed_metadata_under_the_root() {
    let root = tempdir().expect("temporary Git repository");
    let run_git = |arguments: &[&str]| {
        let output = Command::new("git").arg("-C").arg(root.path()).args(arguments).output().expect("run Git");
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8(output.stdout).expect("Git output")
    };
    run_git(&["init", "-b", "main"]);
    run_git(&["config", "user.email", "acorn@example.org"]);
    run_git(&["config", "user.name", "ACORN test"]);
    let file = root.path().join("metadata.json");
    let original = include_str!("../../../../../../../tests/fixtures/analyzer/fair/metadata-minimal-rad.json");
    std::fs::write(&file, original).expect("initial metadata");
    run_git(&["add", "metadata.json"]);
    run_git(&["commit", "-m", "Initial metadata"]);
    run_git(&["checkout", "-b", "feature"]);
    std::fs::write(&file, format!("{original}\n")).expect("changed metadata bytes");
    run_git(&["add", "metadata.json"]);
    run_git(&["commit", "-m", "Change metadata"]);
    let commit = run_git(&["rev-parse", "HEAD"]);
    for reference in [
        InputReference::GitCommit(commit.trim().to_string()),
        InputReference::GitBranch("feature".to_string()),
    ] {
        let mut request = input("metadata.json");
        request.input = reference;
        request.disable_website_checks = true;
        let result = request.analyze(root.path(), true).await.expect("Git analysis");
        assert_eq!(result.batches.len(), 1);
        assert_eq!(
            result.batches.first().expect("one batch").paths,
            vec![file.canonicalize().expect("canonical metadata path")]
        );
    }
}
#[tokio::test]
async fn test_invalid_remote_and_git_references_do_not_fall_back_to_local_input() {
    let root = fixture_root();
    let mut request = input("CITATION.cff");
    request.input = InputReference::Url("file:///tmp/CITATION.cff".to_string());
    assert!(request.clone().analyze(&root, false).await.is_err());
    request.input = InputReference::GitCommit("nonexistent-commit".to_string());
    assert!(request.analyze(&root, true).await.is_err());
}
#[tokio::test]
async fn test_native_registry_executes_the_same_analysis_offline() {
    let root = fixture_root();
    let expected = input("data/valid_project/index.json")
        .analyze(&root, true)
        .await
        .expect("direct analysis");
    let registry = ToolRegistry::default().register_analysis_capability(root, true).expect("registry");
    let selection = registry
        .select_capability_binding(
            "acorn.check",
            None,
            None,
            CapabilitySelectionContext {
                offline: true,
                allow_remote: false,
            },
        )
        .expect("native selection");
    let result = registry
        .execute_selected(
            &selection,
            json!({"input":{"kind":"file","value":"data/valid_project/index.json"},"standard":null,"skip":[],"disable_website_checks":false}),
            ExecutionContext {
                grounded: true,
                selection: CapabilitySelectionContext {
                    offline: true,
                    allow_remote: false,
                },
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(result.outcome, ExecutionOutcome::Completed);
    assert_eq!(result.content.expect("output"), serde_json::to_value(expected).unwrap());
}
#[test]
fn test_online_binding_cannot_be_reused_under_an_offline_policy() {
    let registry = ToolRegistry::default()
        .register_analysis_capability(fixture_root(), false)
        .expect("registry");
    let offline = CapabilitySelectionContext {
        offline: true,
        allow_remote: false,
    };
    assert!(registry.select_capability_binding("acorn.check", None, None, offline).is_err());
}
#[tokio::test]
async fn test_online_remote_source_uses_the_native_analysis_service() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("loopback listener");
    let address = listener.local_addr().expect("loopback address");
    let content = include_str!("../../../../../../../tests/fixtures/analyzer/fair/metadata-minimal-rad.json");
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{content}",
        content.len()
    );
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accepted request");
        let mut request = [0_u8; 2048];
        let count = stream.read(&mut request).await.expect("read request");
        assert!(count > 0);
        stream.write_all(response.as_bytes()).await.expect("write response");
    });
    let mut request = input("CITATION.cff");
    request.input = InputReference::Url(format!("http://{address}/metadata.json"));
    request.disable_website_checks = true;
    let result = request.analyze(&fixture_root(), false).await.expect("remote analysis");
    assert_eq!(result.batches.len(), 1);
    server.await.expect("server finished");
}
#[tokio::test]
async fn test_remote_and_oci_sources_fail_closed_offline() {
    let root = fixture_root();
    let mut request = input("CITATION.cff");
    request.input = InputReference::Url("https://example.org/metadata.json".to_string());
    assert!(request.clone().analyze(&root, true).await.is_err());
    request.input = InputReference::Oci("oci://registry.example.org/example:latest".to_string());
    request.standard = Some(Standard::Oci);
    assert!(request.analyze(&root, true).await.is_err());
    let mut merge_request = input("CITATION.cff");
    merge_request.input = InputReference::GitMergeRequest;
    assert!(merge_request.analyze(&root, true).await.is_err());
}
