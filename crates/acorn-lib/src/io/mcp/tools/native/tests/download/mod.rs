use super::super::download::*;
#[cfg(feature = "needle")]
use super::super::sidecar::*;
use crate::io::bagit::{Bag, Save};
use crate::io::mcp::tools::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome, MutationAuthority};
use crate::io::mcp::ToolRegistry;
#[cfg(feature = "needle")]
use crate::io::sidecar::{needle::Needle, SidecarAssets};
use acorn_core::util::MimeType;
use serde_json::json;
#[cfg(feature = "needle")]
use std::env::consts::{ARCH, OS};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

mod model;
fn archive(root: &Path) -> PathBuf {
    let payload = root.join("payload");
    fs::create_dir(&payload).expect("payload directory");
    fs::copy(
        crate::test::utils::fixture_path("data/valid_project/index.json"),
        payload.join("index.json"),
    )
    .expect("copy research activity");
    let destination = root.join("bag.zip");
    Bag::init()
        .base_directory(payload.display().to_string())
        .payload(vec!["index.json".to_string()])
        .build()
        .save_as(root.join("bag"), MimeType::Zip, Some(destination.clone()))
        .expect("create verified archive");
    destination
}
#[tokio::test]
async fn test_bucket_download_rejects_unbounded_or_remote_offline_sources() {
    let root = tempdir().expect("bucket root");
    let config = root.path().join("buckets.json");
    fs::copy(crate::test::utils::fixture_path("config/with_remote_buckets.json"), &config).expect("copy bucket configuration");
    let configured = BucketDownloadInput {
        sources: Vec::new(),
        config: Some(PathBuf::from("buckets.json")),
        destination: PathBuf::from("configured"),
        filter: Vec::new(),
        ignore: Vec::new(),
        flatten: false,
        clobber: false,
        strict: false,
        canonicalize: false,
        archive_format: None,
    };
    let error = configured
        .execute(root.path(), true)
        .await
        .expect_err("remote configuration denied offline");
    assert!(error.to_string().contains("Offline mode"), "{error}");
    assert!(!root.path().join("configured").exists());
    let request = BucketDownloadInput {
        sources: vec!["https://example.org/research".to_string()],
        config: None,
        destination: PathBuf::from("out"),
        filter: Vec::new(),
        ignore: Vec::new(),
        flatten: false,
        clobber: false,
        strict: false,
        canonicalize: false,
        archive_format: None,
    };
    assert!(request.execute(root.path(), true).await.is_err());
    assert!(!root.path().join("out").exists());
    assert!(serde_json::from_value::<BucketDownloadInput>(json!({
        "sources":["https://example.org/research"],"destination":"out","shell":"true"
    }))
    .is_err());
    assert!(BucketDownloadInput {
        sources: vec!["../escape.zip".to_string()],
        config: None,
        destination: PathBuf::from("out"),
        filter: Vec::new(),
        ignore: Vec::new(),
        flatten: false,
        clobber: false,
        strict: false,
        canonicalize: false,
        archive_format: None,
    }
    .execute(root.path(), true)
    .await
    .is_err());
}
#[tokio::test]
async fn test_bucket_download_requires_exact_authority_and_preserves_source() {
    let root = tempdir().expect("bucket root");
    let archive = archive(root.path());
    let before = fs::read(&archive).expect("archive bytes");
    let registry = ToolRegistry::default()
        .register_download_bucket_capability(root.path(), true)
        .expect("register bucket");
    let selected = registry
        .select_capability_binding("acorn.download::bucket", None, None, CapabilitySelectionContext::default())
        .expect("select bucket");
    let arguments = json!({
        "sources":["bag.zip"],"config":null,"destination":"out","filter":[],"ignore":[],
        "flatten":false,"clobber":false,"strict":false,"canonicalize":false,"archive_format":null
    });
    let denied = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(denied.outcome, ExecutionOutcome::PermissionRequired);
    assert!(!root.path().join("out").exists());
    let applied = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.download::bucket".to_string(),
                    binding_id: "native:acorn.download::bucket".to_string(),
                    arguments,
                    destination: "out".to_string(),
                }),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(applied.outcome, ExecutionOutcome::Completed, "{applied:?}");
    assert!(root.path().join("out/index.json").is_file());
    assert_eq!(fs::read(&archive).expect("archive preserved"), before);
}
#[cfg(feature = "needle")]
#[tokio::test]
async fn test_needle_status_and_explicit_acquisition_policy() {
    let status = NeedleStatusOutput::inspect().await.expect("pinned status");
    assert!(status
        .destination
        .ends_with(Needle::platform(&Needle::for_target(OS, ARCH).expect("host runner"))));
    assert_eq!(status.ready, status.runner.is_some());
    let invalid = NeedleAcquireInput {
        destination: status.destination.join("other"),
    };
    assert!(invalid.acquire(true).await.is_err());
    let registry = ToolRegistry::default()
        .register_download_needle_status_capability()
        .expect("register status")
        .register_download_needle_capability(true)
        .expect("register acquisition");
    let selection = CapabilitySelectionContext {
        offline: true,
        allow_remote: false,
    };
    let selected_status = registry
        .select_capability_binding("acorn.download::needle::status", None, None, selection)
        .expect("select status offline");
    let inspected = registry
        .execute_selected(
            &selected_status,
            json!({}),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(inspected.outcome, ExecutionOutcome::Completed);
    assert!(inspected.artifacts.is_empty());
    let selected_acquire = registry
        .select_capability_binding("acorn.download::needle", None, None, selection)
        .expect("select acquisition offline");
    let arguments = json!({"destination":status.destination});
    let denied = registry
        .execute_selected(
            &selected_acquire,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(denied.outcome, ExecutionOutcome::PermissionRequired);
    let acquired = registry
        .execute_selected(
            &selected_acquire,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                selection,
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.download::needle".to_string(),
                    binding_id: "native:acorn.download::needle".to_string(),
                    destination: status.destination.display().to_string(),
                    arguments,
                }),
                ..ExecutionContext::default()
            },
        )
        .await;
    if status.ready {
        assert_eq!(acquired.outcome, ExecutionOutcome::Completed, "{acquired:?}");
        assert_eq!(
            status.runner.expect("verified runner"),
            NeedleAcquireInput {
                destination: status.destination
            }
            .acquire(true)
            .await
            .expect("cached acquisition")
            .runner
        );
    } else {
        assert_eq!(acquired.outcome, ExecutionOutcome::ToolFailed, "{acquired:?}");
    }
}
