use super::*;
use dioxus::core::Mutation;
use serde_json::json;

#[test]
fn test_complexity_samples_plot_submitted_expressions_and_assumptions() {
    let assumptions = json!({"n": "growing", "k": "fixed-positive"});
    let samples = complexity_samples("O(n log n + k)", "O(n^2)", &assumptions).expect("supported expressions should produce samples");
    assert_eq!(samples.len(), 13);
    assert_eq!(samples.first().copied(), Some(ComplexitySample { n: 1, left: 2.0, right: 1.0 }));
    assert_eq!(
        samples.last().copied(),
        Some(ComplexitySample {
            n: 4096,
            left: 4096.0 * 4096.0_f64.ln() + 2.0,
            right: 16_777_216.0
        })
    );
    let path = complexity_path(
        &samples,
        samples.iter().map(|sample| sample.right.ln_1p()).fold(0.0_f64, f64::max),
        |sample| sample.left,
    );
    assert!(path.starts_with('M'), "path must begin with a move: {path}");
    assert_eq!(path.matches('L').count(), 12, "path must connect every sample: {path}");
    let fast = complexity_samples("O(n^n)", "O(n^2)", &assumptions).expect("finite prefix should remain graphable");
    assert_eq!(fast.last().map(|sample| sample.n), Some(128));
    assert!(complexity_samples("O(polylog(n))", "O(n)", &assumptions).is_none());
}
#[test]
fn test_json_patch_hierarchy_preserves_operation_details() {
    let value = json!({
        "changed": true,
        "mode": "json-patch",
        "operations": [
            {"op": "add", "path": "/authors/0", "value": {"name": "Ada"}},
            {"op": "move", "from": "/draft", "path": "/published"}
        ]
    });
    let mut dom = VirtualDom::new_with_props(Difference, DifferenceProps { value });
    let mutations = dom.rebuild_to_vec();
    let text = mutations
        .edits
        .iter()
        .filter_map(|edit| match edit {
            | Mutation::CreateTextNode { value, .. } | Mutation::SetText { value, .. } => Some(value.as_str()),
            | _ => None,
        })
        .collect::<Vec<_>>();
    for expected in ["1", "add", "/authors/0", "Ada", "2", "move", "/draft", "/published"] {
        assert!(text.contains(&expected), "missing {expected:?}: {text:?}");
    }
}
#[test]
fn test_validation_places_unlocated_checks_before_source_groups() {
    let value = Rc::new(json!({"valid": false, "diagnostics": [
        {"category": "schema", "path": "$.title", "message": "Located title", "line": 1, "character": 2},
        {"category": "schema", "path": "$.missing", "message": "Missing field"},
    ]}));
    let mut dom = VirtualDom::new_with_props(
        Validation,
        ValidationProps {
            value,
            source: None,
            tool: Tool::Activity,
        },
    );
    let mutations = dom.rebuild_to_vec();
    let text = mutations
        .edits
        .iter()
        .filter_map(|edit| match edit {
            | Mutation::CreateTextNode { value, .. } | Mutation::SetText { value, .. } => Some(value.as_str()),
            | _ => None,
        })
        .collect::<Vec<_>>();
    let document = text
        .iter()
        .position(|value| *value == "Document checks")
        .expect("document group should render");
    let schema = text.iter().position(|value| *value == "Schema").expect("source group should render");
    assert!(document < schema, "document checks should precede source groups: {text:?}");
    for message in ["Missing field", "Located title"] {
        assert_eq!(
            text.iter().filter(|value| **value == message).count(),
            1,
            "{message} should render once: {text:?}"
        );
    }
}
