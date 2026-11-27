//! CZML Standard-tool request and schema validation coverage.
use acorn_core::util::MimeType;
use acorn_schema::{codec::DocumentCodec, standard::czml::Document, validation::Validate};
use acorn_web::operations::{Request, Tool};
use serde_json::Value;

const MALFORMED: &str = include_str!("../../../tests/fixtures/schema/czml/acorn/malformed.json");
const SCENE: &str = include_str!("../../../tests/fixtures/schema/czml/acorn/scene.czml");
fn malformed_reference() -> String {
    serde_json::from_str::<Value>(MALFORMED)
        .ok()
        .and_then(|cases| cases.get("malformedReference").cloned())
        .and_then(|value| serde_json::to_string(&value).ok())
        .unwrap_or_default()
}
fn request(input: &str) -> Request {
    Request {
        input: input.to_owned(),
        standard: "czml".to_owned(),
        ..Request::default()
    }
}

#[test]
fn test_czml_request_contract_matches_schema_diagnostics() {
    let valid = request(SCENE);
    let malformed = request(&malformed_reference());
    assert!(valid.validate(Tool::Standard).is_ok());
    assert!(malformed.validate(Tool::Standard).is_ok());
    assert_eq!(valid.validate(Tool::Fair), Err("FAIR assessment is unsupported for CZML.".into()));
    assert!(matches!(
        Document::decode(&valid.input, &MimeType::Json).map(|document| document.validate()),
        Ok(Ok(()))
    ));
    let issues = Document::decode(&malformed.input, &MimeType::Json)
        .ok()
        .and_then(|document| document.validate().err())
        .map(|report| report.into_issues())
        .unwrap_or_default();
    assert!(issues.iter().any(|issue| issue.error.code == "czml_reference"));
}
