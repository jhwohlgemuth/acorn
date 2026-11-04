use super::super::report::*;
use crate::io::api::gitlab::ReportStatus;
use crate::io::mcp::tools::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome, MutationAuthority};
use crate::io::mcp::ToolRegistry;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};
use serde_json::json;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use tempfile::tempdir;

fn input() -> ReportInput {
    ReportInput {
        config: PathBuf::from("acorn.json"),
        project_id: "pilot".to_string(),
        from: None,
        through: "2026-11-01T00:00:00Z".to_string(),
        commit: "0123456789abcdef".to_string(),
        destination: PathBuf::from("reports"),
        pipeline_id: Some("100".to_string()),
        job_id: Some("200".to_string()),
        full_timestamp: false,
        overwrite: false,
    }
}

#[cfg(unix)]
#[test]
fn test_project_report_rejects_source_symlink_outside_root() {
    let root = tempdir().expect("report root");
    let outside = tempdir().expect("outside report root");
    let source = outside.path().join("index.json");
    fs::copy(crate::test::utils::fixture_path("data/valid_project/index.json"), &source).expect("outside source");
    let project = root.path().join("pilot");
    fs::create_dir(&project).expect("project directory");
    symlink(&source, project.join("index.json")).expect("source symlink");
    let result = ReportProjectRequest {
        commit: "0123456789abcdef",
        from: None,
        output: &root.path().join("reports"),
        project_id: "pilot",
        relative_path: "pilot/index.json",
        root: root.path(),
        through: "2026-11-01T00:00:00Z",
    }
    .prepare();
    assert!(result.is_err());
    assert!(!root.path().join("reports").exists());
}
#[tokio::test]
async fn test_report_local_and_publish_require_exact_authority() {
    let root = tempdir().expect("report root");
    fs::write(root.path().join("acorn.json"), "{}").expect("local configuration");
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();
    let registry = ToolRegistry::default()
        .register_report_capabilities(root.path(), true, move |input, _root, destination, _publish| {
            let calls = observed.clone();
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                fs::create_dir_all(&destination).expect("authorized report directory");
                fs::write(destination.join("manifest.json"), "{}\n").expect("authorized report artifact");
                Ok(ReportOutput {
                    project_id: input.project_id,
                    status: ReportStatus::NoChanges,
                    version: "v1".to_string(),
                    artifacts: vec![destination.join("manifest.json").display().to_string()],
                    destination,
                })
            }
        })
        .expect("register report capabilities");
    let local = registry
        .select_capability_binding("acorn.report::local", None, None, CapabilitySelectionContext::default())
        .expect("select local report");
    let publish = registry
        .select_capability_binding(
            "acorn.report::publish",
            None,
            None,
            CapabilitySelectionContext {
                offline: false,
                allow_remote: true,
            },
        )
        .expect("select publication");
    let arguments = serde_json::to_value(input()).expect("report input");
    let denied = registry
        .execute_selected(
            &local,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(denied.outcome, ExecutionOutcome::PermissionRequired);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(!root.path().join("reports").exists());
    let applied = registry
        .execute_selected(
            &local,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.report::local".to_string(),
                    binding_id: "native:acorn.report::local".to_string(),
                    arguments: arguments.clone(),
                    destination: "reports".to_string(),
                }),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(applied.outcome, ExecutionOutcome::Completed, "{applied:?}");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(fs::read(root.path().join("reports/manifest.json")).expect("local manifest"), b"{}\n");
    let publish_arguments = json!({"config":"acorn.json","project_id":"pilot","from":null,"through":"2026-11-01T00:00:00Z","commit":"0123456789abcdef","destination":"published","pipeline_id":"100","job_id":"200","full_timestamp":false,"overwrite":false});
    let denied_publish = registry
        .execute_selected(
            &publish,
            publish_arguments.clone(),
            ExecutionContext {
                grounded: true,
                selection: CapabilitySelectionContext {
                    offline: false,
                    allow_remote: true,
                },
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(denied_publish.outcome, ExecutionOutcome::PermissionRequired);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(!root.path().join("published").exists());
    let published = registry
        .execute_selected(
            &publish,
            publish_arguments.clone(),
            ExecutionContext {
                grounded: true,
                selection: CapabilitySelectionContext {
                    offline: false,
                    allow_remote: true,
                },
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.report::publish".to_string(),
                    binding_id: "native:acorn.report::publish".to_string(),
                    arguments: publish_arguments,
                    destination: "published".to_string(),
                }),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(published.outcome, ExecutionOutcome::Completed, "{published:?}");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert!(!serde_json::to_string(&published.content).expect("receipt").contains("token"));
    assert!(registry
        .select_capability_binding(
            "acorn.report::publish",
            None,
            None,
            CapabilitySelectionContext {
                offline: true,
                allow_remote: false,
            },
        )
        .is_err());
    assert!(input().validate(root.path(), true).is_err());
}
