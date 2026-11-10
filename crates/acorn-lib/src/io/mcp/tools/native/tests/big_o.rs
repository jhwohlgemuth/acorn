use crate::io::mcp::tools::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome};
use crate::io::mcp::ToolRegistry;
use acorn_core::big_o::{compare_notation, ComparisonInput, ComparisonOutput, Dominance};
use acorn_core::util::SemanticVersion;
use serde_json::json;

const CAPABILITY: &str = "acorn.diff::bigo";
const BINDING: &str = "native:acorn.diff::bigo";

fn offline() -> CapabilitySelectionContext {
    CapabilitySelectionContext {
        offline: true,
        allow_remote: false,
    }
}

#[tokio::test]
async fn test_big_o_offline_discovery_and_explicit_dispatch_match_core() {
    let registry = ToolRegistry::default().register_big_o_capability().expect("register Big-O");
    let catalog = registry
        .capability_catalog(Some(&[CAPABILITY.to_string()]), offline())
        .expect("offline catalog");
    let entries: serde_json::Value = serde_json::from_str(&catalog).expect("catalog JSON");
    assert_eq!(entries.as_array().map(Vec::len), Some(1));
    let selected = registry
        .select_capability_binding(CAPABILITY, Some(BINDING), None, offline())
        .expect("explicit offline binding");
    assert_eq!(selected.capability.id, CAPABILITY);
    assert_eq!(selected.binding.id, BINDING);
    assert_eq!(selected.capability.version, SemanticVersion::from((1, 0, 0)));
    assert_eq!(selected.binding.version, selected.capability.version);
    assert_eq!(selected.binding.input_schema, selected.capability.input_schema);
    assert_eq!(selected.binding.output_schema, selected.capability.output_schema);
    assert_eq!(
        selected.capability.input_schema,
        serde_json::to_value(schemars::schema_for!(ComparisonInput)).expect("input schema")
    );
    assert_eq!(
        selected.capability.output_schema,
        serde_json::to_value(schemars::schema_for!(ComparisonOutput)).expect("output schema")
    );
    assert!(selected.capability.effects.read_only);
    assert!(selected.capability.effects.idempotent);
    assert!(!selected.capability.effects.destructive);
    assert!(!selected.capability.effects.open_world);
    assert_eq!(selected.binding.effects, selected.capability.effects);
    assert!(selected.binding.offline_safe);
    let arguments = json!({"left":"O(n^2)","right":"O(n log n)"});
    let expected = compare_notation(serde_json::from_value(arguments.clone()).expect("typed input")).expect("core comparison");
    assert_eq!(expected.dominance, Dominance::Left);
    let result = registry
        .execute_selected(
            &selected,
            arguments,
            ExecutionContext {
                selection: offline(),
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(result.outcome, ExecutionOutcome::Completed);
    assert_eq!(result.content, Some(serde_json::to_value(expected).expect("core result JSON")));
    assert!(result.artifacts.is_empty());
    assert_eq!(result.provenance.capability_id, CAPABILITY);
    assert_eq!(result.provenance.binding_id, BINDING);
    assert!(registry
        .select_capability_binding(CAPABILITY, Some("remote:acorn.diff::bigo"), None, offline())
        .is_err());
}
#[tokio::test]
async fn test_big_o_invalid_requests_and_cancellation_have_no_artifacts() {
    let registry = ToolRegistry::default().register_big_o_capability().expect("register Big-O");
    let selected = registry
        .select_capability_binding(CAPABILITY, Some(BINDING), None, offline())
        .expect("explicit offline binding");
    for (arguments, outcome, detail) in [
        (
            json!({"left":"O(n - m)","right":"O(n)"}),
            ExecutionOutcome::ToolFailed,
            Some("invalid left Big-O expression"),
        ),
        (
            json!({"left":"O(n)","right":"O(n - m)"}),
            ExecutionOutcome::ToolFailed,
            Some("invalid right Big-O expression"),
        ),
        (
            json!({"left":"O(n)","right":"O(1)","assumptions":{"missing":"growing"}}),
            ExecutionOutcome::ToolFailed,
            Some("growth assumption `missing`"),
        ),
        (json!({"left":"O(n^2)"}), ExecutionOutcome::InvalidInput, None),
        (json!({"left":"O(n)","right":"O(1)","extra":true}), ExecutionOutcome::InvalidInput, None),
    ] {
        let result = registry
            .execute_selected(
                &selected,
                arguments,
                ExecutionContext {
                    selection: offline(),
                    grounded: true,
                    ..ExecutionContext::default()
                },
            )
            .await;
        assert_eq!(result.outcome, outcome);
        assert_eq!(
            result.error.as_deref().map(|message| message.contains(detail.unwrap_or_default())),
            detail.map(|_| true)
        );
        assert_eq!(
            serde_json::to_value(&result).expect("routed result").get("error").is_some(),
            detail.is_some()
        );
        assert!(result.content.is_none());
        assert!(result.artifacts.is_empty());
    }
    let cancelled = registry
        .execute_selected(
            &selected,
            json!({"left":"O(n^2)","right":"O(n log n)"}),
            ExecutionContext {
                selection: offline(),
                grounded: true,
                cancellation: Some(Box::pin(async {})),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(cancelled.outcome, ExecutionOutcome::Cancelled);
    assert!(cancelled.content.is_none());
    assert!(cancelled.artifacts.is_empty());
}
