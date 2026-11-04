use super::super::swhid::*;
use crate::io::mcp::tools::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome};
use crate::io::mcp::ToolRegistry;
use crate::io::swhid::CalculationKind;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_swhid_file_directory_and_verify_are_bounded() {
    let root = tempdir().expect("temporary SWHID root");
    fs::write(root.path().join("item.txt"), "bounded content").expect("write source");
    let registry = ToolRegistry::default().register_swhid_capability(root.path()).expect("register SWHID");
    let selected = registry
        .select_capability_binding("acorn.swhid", None, None, CapabilitySelectionContext::default())
        .expect("select SWHID");
    let file = json!({"path":"item.txt","kind":"content","reference":null,"verify":null});
    let result = registry
        .execute_selected(
            &selected,
            file.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(result.outcome, ExecutionOutcome::Completed);
    let output = SwhidInput {
        path: PathBuf::from("item.txt"),
        kind: CalculationKind::Content,
        reference: None,
        verify: None,
    }
    .execute(root.path())
    .expect("calculate file");
    assert!(output.identifier.starts_with("swh:1:cnt:"));
    let verified = SwhidInput {
        verify: Some(output.identifier.clone()),
        ..SwhidInput {
            path: PathBuf::from("item.txt"),
            kind: CalculationKind::Content,
            reference: None,
            verify: None,
        }
    }
    .execute(root.path())
    .expect("verify file");
    assert!(verified.verified);
    assert!(SwhidInput {
        path: PathBuf::from("../outside"),
        kind: CalculationKind::Auto,
        reference: None,
        verify: None
    }
    .execute(root.path())
    .is_err());
    let directory = SwhidInput {
        path: PathBuf::from("."),
        kind: CalculationKind::Directory,
        reference: None,
        verify: None,
    }
    .execute(root.path())
    .expect("calculate directory");
    assert!(directory.identifier.starts_with("swh:1:dir:"));
    assert!(SwhidInput {
        path: PathBuf::from("item.txt"),
        kind: CalculationKind::Content,
        reference: None,
        verify: Some(directory.identifier),
    }
    .execute(root.path())
    .is_err());
}
