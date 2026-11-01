use super::*;

fn fixture(name: &str) -> String {
    read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/config/ron").join(name)).expect("read RON configuration fixture")
}

#[test]
fn test_ron_adapter_matches_equivalent_json_across_acre_sections() {
    let json = ApplicationConfiguration::parse(fixture("equivalent.json")).expect("parse equivalent JSON configuration");
    let ron = ApplicationConfiguration::parse_ron(fixture("equivalent.ron")).expect("parse equivalent RON configuration");
    assert_eq!(
        serde_json::to_value(&ron).expect("serialize RON configuration"),
        serde_json::to_value(&json).expect("serialize JSON configuration")
    );
    json.validate().expect("JSON fixture validates");
    ron.validate().expect("RON fixture validates");
}
#[test]
fn test_ron_adapter_preserves_defaults_nulls_and_one_or_many_values() {
    let configuration = ApplicationConfiguration::parse_ron(
        r#"{
            "models": (),
            "members": "scientist@example.org",
            "folders": [{"id": "pilot", "aliases": ["science", "research"]}],
        }"#,
    )
    .expect("parse RON defaults and unions");
    assert_eq!(configuration.schema_version, AcreSchemaVersion::V2_0_0);
    assert!(configuration.models.is_none());
    assert_eq!(configuration.members.as_ref().map(OneOrMany::len), Some(1));
    assert_eq!(
        configuration.folder("pilot").expect("folder").aliases.as_ref().map(OneOrMany::len),
        Some(2)
    );
}
#[test]
fn test_ron_adapter_rejects_duplicate_keys_at_every_depth() {
    for content in [
        r#"{"models": [], "models": []}"#,
        r#"{"variables": {"custom": {"catalog": "ONE", "catalog": "TWO"}}}"#,
    ] {
        let message = ApplicationConfiguration::parse_ron(content)
            .expect_err("duplicate RON key should fail")
            .to_string();
        assert!(message.contains("duplicate RON map key"), "unexpected error: {message}");
        assert!(message.contains("1:"), "missing source position: {message}");
    }
}
#[test]
fn test_ron_adapter_rejects_lossy_or_unsupported_data_values() {
    for (content, expected) in [
        (r#"{1: "value"}"#, "string"),
        (r#"{"schemaVersion": 340282366920938463463374607431768211455}"#, "range"),
        (r#"{"options": {"scripts": {"enabled": NaN}}}"#, "non-finite"),
        (r#"{"members": [b"x"]}"#, "JSON-compatible"),
    ] {
        let message = ApplicationConfiguration::parse_ron(content)
            .expect_err("unsupported RON value should fail")
            .to_string();
        assert!(message.contains(expected), "expected '{expected}' in: {message}");
    }
}
#[test]
fn test_ron_adapter_reports_syntax_schema_and_trailing_errors() {
    for (content, expected) in [
        (r#"{"models": [}"#, "1:"),
        (r#"{"schemaVersion": "3.0.0"}"#, "schemaVersion"),
        (r#"{"unknown": true}"#, "unknown"),
        (r#"{} {}"#, "trailing"),
    ] {
        let message = ApplicationConfiguration::parse_ron(content)
            .expect_err("invalid RON configuration should fail")
            .to_string();
        assert!(
            message.to_lowercase().contains(&expected.to_lowercase()),
            "expected '{expected}' in: {message}"
        );
    }
}
#[test]
fn test_ron_adapter_retains_finite_recursion_limit() {
    let nested = format!("{{\"unknown\": {}0{}}}", "[".repeat(140), "]".repeat(140));
    let message = ApplicationConfiguration::parse_ron(nested)
        .expect_err("excessively nested RON should fail")
        .to_string();
    assert!(message.contains("recursion limit"), "unexpected error: {message}");
}
#[test]
fn test_ron_serialization_is_canonical_and_round_trips() {
    let configuration = ApplicationConfiguration::parse_ron(fixture("equivalent.ron")).expect("parse RON fixture");
    let serialized = configuration.to_ron().expect("serialize RON configuration");
    assert!(serialized.ends_with('\n'));
    assert!(!serialized.contains("\r\n"));
    assert!(serialized.lines().any(|line| line.starts_with("  \"auth\"")));
    let reparsed = ApplicationConfiguration::parse_ron(&serialized).expect("reparse canonical RON");
    assert_eq!(
        serde_json::to_value(reparsed).expect("serialize reparsed configuration"),
        serde_json::to_value(configuration).expect("serialize original configuration")
    );
}
