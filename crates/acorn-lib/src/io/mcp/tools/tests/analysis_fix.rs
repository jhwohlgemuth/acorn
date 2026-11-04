use super::*;
use crate::agent::{InMemoryBackend, InferenceFinishReason, InferenceProvenance, InferenceResult};
use crate::analyzer::fix::FixOptions;
use crate::io::mcp::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

#[tokio::test]
async fn test_configured_fix_proposal_builds_reviewable_diff_without_writing() {
    let root = tempfile::tempdir().expect("temporary check root");
    let path = root.path().join("CITATION.cff");
    let original = include_str!("../../../../../../../tests/fixtures/CITATION.cff");
    fs::write(&path, original).expect("copy CFF fixture");
    let analysis = AnalysisInput {
        input: InputReference::File(PathBuf::from("CITATION.cff")),
        standard: None,
        format: None,
        skip: ["prose", "link", "quality", "readability", "fair", "crosswalk"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        filter: Vec::new(),
        ignore: Vec::new(),
        disable_website_checks: true,
    };
    let options = FixOptions::default();
    let (report, _, _) = analysis.clone().analyze_report(root.path(), true).await.expect("analyze fixture");
    let reference = report
        .normalize_checks(&options)
        .expect("normalized findings")
        .into_iter()
        .find(|reference| reference.vocabulary_advisory.is_none() && reference.uri.is_some())
        .expect("schema finding with a file target");
    let target = reference.uri.clone().expect("file target");
    let backend = InMemoryBackend::new(InferenceResult {
        finish_reason: InferenceFinishReason::Stop,
        provenance: InferenceProvenance {
            agent: None,
            backend: "memory".to_string(),
            model: Some("fixture".to_string()),
            request_id: Some("fixture-request".to_string()),
        },
        structured: Some(json!({
            "fixes": [{
                "actions": [{
                    "description": "Replace invalid email",
                    "expected": "invalid email",
                    "kind": "document-change",
                    "replacement": "valid@example.org"
                }],
                "automation": "safe-automatic",
                "confidence": 0.95,
                "effects": {
                    "elevation": false,
                    "filesystem_write": true,
                    "network": false,
                    "package_manager": false,
                    "process": false,
                    "restart": false,
                    "reversible": true
                },
                "finding_id": reference.id,
                "id": "fix-0001",
                "rationale": "Use a valid email address",
                "risk": "low",
                "source_fingerprint": reference.fingerprint,
                "status": "proposed",
                "summary": "Replace invalid email",
                "target": target
            }],
            "version": "1"
        })),
        text: String::new(),
        usage: None,
    });
    let output = AnalysisFixInput { analysis: analysis.clone() }
        .analyze(root.path(), true, &backend, &options)
        .await
        .expect("nonexecuting fix preview");
    assert!(output.fixes.diagnostics.is_empty(), "{:?}", output.fixes.diagnostics);
    assert!(output.preview_diagnostics.is_empty(), "{:?}", output.preview_diagnostics);
    assert_eq!(output.differences.len(), 1);
    assert!(output
        .differences
        .first()
        .is_some_and(|difference| difference.unified.contains("valid@example.org")));
    let registry = ToolRegistry::default()
        .register_analysis_fixes_capability(root.path(), true, Arc::new(backend), options)
        .expect("registered fixes");
    let selected = registry
        .select_capability_binding(
            "acorn.check::fixes",
            None,
            None,
            CapabilitySelectionContext {
                offline: true,
                allow_remote: false,
            },
        )
        .expect("selected fixes");
    let routed = registry
        .execute_selected(
            &selected,
            serde_json::to_value(AnalysisFixInput { analysis }).expect("fix input JSON"),
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
    assert_eq!(routed.outcome, ExecutionOutcome::Completed);
    assert_eq!(routed.content, Some(serde_json::to_value(output).expect("fix output JSON")));
    assert_eq!(fs::read_to_string(&path).expect("original remains"), original);
}
