use crate::standard::zon;
use crate::standard::{ckan::Package, crosswalk::Crosswalk, datacite::RelationType, dcat::Dataset};
use crate::validation::Validate;
use acorn_core::util::MimeType;
use proptest::{
    prelude::*,
    test_runner::{Config as ProptestConfig, FileFailurePersistence},
};

proptest! {
    #![proptest_config(ProptestConfig {
        cases: std::env::var("PROPTEST_CASES").ok().and_then(|value| value.parse().ok()).unwrap_or(64),
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct("tests/proptest-regressions/standard/ckan.txt"))),
        ..ProptestConfig::default()
    })]
    #[test]
    fn test_package_round_trips_in_supported_formats(
        id in "[a-z0-9]{1,24}",
        name in "[a-z][a-z0-9-]{0,23}",
        title in prop_oneof![Just("𝄞 dataset".to_string()), Just("Alpha\r\nBeta".to_string()), "[A-Za-z0-9][A-Za-z0-9 ]{0,39}"],
        notes in any::<String>(),
    ) {
        let source = serde_json::json!({"id": id, "name": name, "title": title, "notes": notes, "resources": [], "tags": []});
        let package: Package = serde_json::from_value(source).expect("generated CKAN package");
        prop_assert!(package.validate().is_ok());
        let expected = serde_json::to_value(&package).expect("serialize CKAN package");
        let encodings = [
            (MimeType::Json, serde_json::to_string(&package).expect("JSON encoding")),
            (MimeType::JsonLines, serde_json::to_string(&package).expect("JSON Lines encoding")),
            (MimeType::Yaml, serde_norway::to_string(&package).expect("YAML encoding")),
            (MimeType::Zon, zon::encode(&package).expect("ZON encoding")),
        ];
        for (format, encoded) in encodings {
            let decoded = Package::parse(&encoded, format).expect("encoded CKAN package parses");
            let actual = decoded.first().expect("one package");
            let value = serde_json::to_value(actual).expect("serialize decoded package");
            prop_assert_eq!(&value, &expected);
            prop_assert_eq!(actual.validate().is_ok(), package.validate().is_ok());
        }
    }
}
#[test]
fn test_converts_relation_type_aliases() {
    assert_eq!(RelationType::from("parent_of"), RelationType::HasPart);
    assert_eq!(RelationType::from("unknown"), RelationType::Other);
}
#[test]
fn test_parses_action_response_and_crosswalks_to_dcat() {
    let source =
        r#"{"success":true,"result":{"id":"123","name":"sample","title":"Sample","notes":"<p>Hello &amp; goodbye</p>","resources":[],"tags":[]}}"#;
    let packages = Package::parse(source, MimeType::Json).expect("action response should parse");
    let package = packages.first().expect("one package");
    let (dataset, warnings) = <Package as Crosswalk<Dataset>>::crosswalk(package).expect("package should convert");
    assert_eq!(dataset.jsonld_type.as_deref(), Some("dcat:Dataset"));
    assert_eq!(
        dataset.description.as_ref().and_then(|values| values.first()).map(String::as_str),
        Some("Hello & goodbye")
    );
    assert!(warnings.iter().any(|warning| warning.field == "notes"));
}
#[test]
fn test_parses_json_lines_in_source_order() {
    let source = concat!(
        "{\"id\":\"one\",\"name\":\"first\",\"title\":\"First\",\"resources\":[]}\n",
        "{\"id\":\"two\",\"name\":\"second\",\"title\":\"Second\",\"resources\":[]}\n",
    );
    let packages = Package::parse(source, MimeType::JsonLines).expect("JSON Lines packages should parse");
    let identifiers = packages.iter().map(|package| package.id.as_str()).collect::<Vec<_>>();
    assert_eq!(identifiers, vec!["one", "two"]);
}
#[test]
fn test_rejects_unsuccessful_action_response() {
    let source = r#"{"success":false,"error":{"message":"not found"}}"#;
    assert!(Package::parse(source, MimeType::Json).is_err());
}
#[test]
fn test_reports_multiple_required_field_errors() {
    let error = Package::parse(r#"{"resources":[]}"#, MimeType::Json).expect_err("required fields should be reported");
    let message = error.to_string();
    assert!(message.contains("id"));
    assert!(message.contains("name"));
    assert!(message.contains("title"));
}
