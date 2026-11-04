use super::super::doctor::*;
use crate::analyzer::host::discovery::Records;
use crate::io::mcp::tools::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome};
use crate::io::mcp::ToolRegistry;
use serde_json::{json, Value};

#[tokio::test]
async fn test_report_is_read_only_and_matches_direct_diagnostic_shape() {
    let registry = ToolRegistry::default().register_doctor_capability().expect("register doctor");
    let selected = registry
        .select_capability_binding("acorn.doctor", None, None, CapabilitySelectionContext::default())
        .expect("select doctor");
    let result = registry
        .execute_selected(
            &selected,
            json!({"categories":["system"]}),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(result.outcome, ExecutionOutcome::Completed);
    let report = result.content.expect("doctor output");
    let findings = report.as_array().expect("array report");
    assert_eq!(findings.len(), 1);
    let finding = findings.first().expect("doctor finding");
    assert_eq!(finding.get("record").expect("record"), "finding");
    assert_eq!(finding.get("category").expect("category"), "system");
    assert!(matches!(
        finding.get("status").and_then(Value::as_str),
        Some("healthy" | "attention" | "unavailable")
    ));
    assert_eq!(result.artifacts.len(), 0);
    let direct = serde_json::to_value(Records::diagnose_host().findings_in(["system"])).expect("direct report");
    let direct_finding = direct.as_array().and_then(|values| values.first()).expect("direct finding");
    assert_eq!(finding.get("subject"), direct_finding.get("subject"));
    assert_eq!(finding.get("expected"), direct_finding.get("expected"));
    assert_eq!(finding.get("status"), direct_finding.get("status"));
    assert!(DoctorInput {
        categories: vec!["repair".to_string()]
    }
    .execute()
    .is_err());
    let malformed = registry
        .execute_selected(
            &selected,
            json!({"categories":["system"],"fix":true}),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_ne!(malformed.outcome, ExecutionOutcome::Completed);
    assert_eq!(
        serde_json::to_value(DoctorOutput(Vec::new())).expect("empty report"),
        Value::Array(Vec::new())
    );
}
