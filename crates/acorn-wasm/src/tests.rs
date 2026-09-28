use super::*;

const VALID_RESEARCH_ACTIVITY: &str = include_str!("../../../tests/fixtures/data/valid_project/index.json");

#[test]
fn check_readability_accepts_every_metric() {
    assert!(["ari", "cli", "fkgl", "fres", "gfi", "lix", "smog"]
        .into_iter()
        .all(|metric| analyze_readability("This is a short sentence.", metric, None).is_ok()));
}
#[test]
fn check_readability_rejects_unknown_metrics() {
    assert!(analyze_readability("Readable text.", "unknown", None).is_err());
}
#[test]
fn check_readability_returns_checks_and_summary() {
    let result = analyze_readability("This is a short sentence.", "fkgl", Some("memory://sample".to_string())).unwrap();
    assert_eq!(result.checks.len(), 1);
    assert_eq!(result.checks.first().and_then(|check| check.uri.as_deref()), Some("memory://sample"));
    assert_eq!(
        result.summary,
        vec![
            vec!["Schema items found", "0"],
            vec!["Link items found", "0"],
            vec!["Prose items found", "0"],
            vec!["Quality items found", "0"],
            vec!["Fair items found", "0"],
            vec!["Readability items found", "1"],
            vec!["Crosswalk items found", "0"],
        ]
    );
}
#[test]
fn validate_research_activity_collects_parse_diagnostics() {
    let result = validate_json(r#"{"title": 1, "unknown": true}"#);
    assert!(!result.valid);
    assert!(!result.diagnostics.is_empty());
    assert!(result.value.is_none());
}
#[test]
fn validate_research_activity_preserves_validation_order() {
    let invalid = VALID_RESEARCH_ACTIVITY
        .replace(r#""title": "Valid Project""#, r#""title": "x""#)
        .replace("wohlgemuthjh@ornl.gov", "invalid-email");
    let result = validate_json(&invalid);
    let paths = result
        .diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic.path.as_deref())
        .collect::<Vec<_>>();
    assert!(!result.valid);
    assert_eq!(paths.first().copied(), Some("title"));
    assert!(paths.contains(&"contact.email"));
}
#[test]
fn validate_research_activity_returns_valid_value() {
    let result = validate_json(VALID_RESEARCH_ACTIVITY);
    assert!(result.valid);
    assert!(result.diagnostics.is_empty());
    assert!(result.value.is_some());
}
