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
#[test]
fn test_ron_file_lifecycle_and_codec() {
    let directory = super::temp_resolve_dir("ron-lifecycle");
    create_dir_all(&directory).expect("create RON directory");
    let path = directory.join(".acorn.ron");
    write(&path, "// original comment\n{\"members\": \"scientist@example.org\"}\n").expect("write RON source");
    assert_eq!(ApplicationConfiguration::resolve_in(&None, &directory), Some(path.clone()));
    let loaded = ApplicationConfiguration::load(&Some(path.clone())).expect("load RON");
    assert_eq!(loaded.members.as_ref().map(OneOrMany::len), Some(1));
    let (with_path, resolved) = ApplicationConfiguration::load_with_path(&Some(path.clone())).expect("load RON with path");
    assert_eq!(resolved, Some(path.clone()));
    assert_eq!(
        serde_json::to_value(&loaded).expect("serialize"),
        serde_json::to_value(&with_path).expect("serialize")
    );
    let updated = loaded
        .with_models(&[ModelEntry::Selector("acme/added".to_string())])
        .expect("update RON model list");
    updated.write_or_preview(&path, true, true).expect("preview RON update");
    assert!(read_to_string(&path).expect("read original").contains("original comment"));
    updated.write_or_preview(&path, false, true).expect("write canonical RON");
    assert!(!read_to_string(&path).expect("read canonical").contains("original comment"));
    assert_eq!(
        ApplicationConfiguration::read(&path)
            .expect("read updated RON")
            .models
            .as_ref()
            .map(OneOrMany::len),
        Some(1)
    );
    let encoded =
        <ApplicationConfiguration as acorn_schema::codec::DocumentCodec>::encode(&updated, &acorn_core::util::MimeType::Ron).expect("encode RON");
    let decoded =
        <ApplicationConfiguration as acorn_schema::codec::DocumentCodec>::decode(&encoded, &acorn_core::util::MimeType::Ron).expect("decode RON");
    assert_eq!(
        serde_json::to_value(decoded).expect("serialize"),
        serde_json::to_value(updated).expect("serialize")
    );
    remove_dir_all(directory).expect("remove RON directory");
}
#[test]
fn test_ron_discovery_precedence_and_explicit_errors() {
    let directory = super::temp_resolve_dir("ron-precedence");
    create_dir_all(&directory).expect("create RON directory");
    let ron = directory.join(".acorn.ron");
    write(&ron, "{}\n").expect("write RON source");
    for name in DEFAULT_CONFIG_FILENAMES[..4].iter().rev() {
        let higher = directory.join(name);
        write(&higher, "{}\n").expect("write higher-priority source");
        assert_eq!(ApplicationConfiguration::resolve_in(&None, &directory), Some(higher));
    }
    assert_eq!(ApplicationConfiguration::resolve_in(&Some(ron.clone()), &directory), Some(ron));
    assert!(ApplicationConfiguration::load(&Some(directory.join("missing.ron"))).is_err());
    let malformed = directory.join("malformed.ron");
    write(&malformed, "{\"models\": [}").expect("write malformed RON");
    assert!(ApplicationConfiguration::load(&Some(malformed)).is_err());
    remove_dir_all(directory).expect("remove RON directory");
}
#[test]
fn test_ron_format_preserves_source_data_and_omissions() {
    let source = "// remove this comment\r\n{\"models\":[\"acme/example\",],\"members\":(),}\r\n";
    let formatted = ApplicationConfiguration::format_ron(source).expect("format RON data");
    assert!(!formatted.contains("comment"));
    assert!(!formatted.contains("schemaVersion"));
    assert!(!formatted.contains("\r\n"));
    assert!(formatted.ends_with('\n'));
    assert_eq!(ApplicationConfiguration::format_ron(&formatted).expect("format canonical RON"), formatted);
    let before = ApplicationConfiguration::parse_ron(source).expect("parse source RON");
    let after = ApplicationConfiguration::parse_ron(&formatted).expect("parse formatted RON");
    assert_eq!(
        serde_json::to_value(before).expect("serialize source"),
        serde_json::to_value(after).expect("serialize formatted")
    );
    assert!(ApplicationConfiguration::format_ron("{\"unknown\":true}").is_err());
    assert!(ApplicationConfiguration::format_ron("{\"models\":[}").is_err());
}
#[test]
fn test_ron_auth_profile_order_survives_format_and_write() {
    let source = read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/oauth2/config/ordered.json"))
        .expect("read ordered auth fixture");
    let configuration = ApplicationConfiguration::parse_ron(&source).expect("parse ordered RON profiles");
    assert_eq!(
        configuration.auth.as_ref().and_then(|auth| auth.profile(None)).map(|(name, _)| name),
        Some("zeta")
    );
    for formatted in [
        ApplicationConfiguration::format_ron(&source).expect("format RON"),
        configuration.to_ron().expect("write RON"),
    ] {
        let reparsed = ApplicationConfiguration::parse_ron(&formatted).expect("reparse ordered RON profiles");
        assert_eq!(
            reparsed.auth.as_ref().and_then(|auth| auth.profile(None)).map(|(name, _)| name),
            Some("zeta")
        );
        assert!(formatted.find("\"zeta\"").expect("zeta key") < formatted.find("\"alpha\"").expect("alpha key"));
    }
}
