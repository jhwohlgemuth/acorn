use crate::standard::{ckan::Package, crosswalk::Crosswalk, datacite::RelationType, dcat::Dataset};
use acorn_core::util::MimeType;

#[test]
fn converts_relation_type_aliases() {
    assert_eq!(RelationType::from("parent_of"), RelationType::HasPart);
    assert_eq!(RelationType::from("unknown"), RelationType::Other);
}
#[test]
fn parses_action_response_and_crosswalks_to_dcat() {
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
fn parses_json_lines_in_source_order() {
    let source = concat!(
        "{\"id\":\"one\",\"name\":\"first\",\"title\":\"First\",\"resources\":[]}\n",
        "{\"id\":\"two\",\"name\":\"second\",\"title\":\"Second\",\"resources\":[]}\n",
    );
    let packages = Package::parse(source, MimeType::JsonLines).expect("JSON Lines packages should parse");
    let identifiers = packages.iter().map(|package| package.id.as_str()).collect::<Vec<_>>();
    assert_eq!(identifiers, vec!["one", "two"]);
}
#[test]
fn rejects_unsuccessful_action_response() {
    let source = r#"{"success":false,"error":{"message":"not found"}}"#;
    assert!(Package::parse(source, MimeType::Json).is_err());
}
#[test]
fn reports_multiple_required_field_errors() {
    let error = Package::parse(r#"{"resources":[]}"#, MimeType::Json).expect_err("required fields should be reported");
    let message = error.to_string();
    assert!(message.contains("id"));
    assert!(message.contains("name"));
    assert!(message.contains("title"));
}
