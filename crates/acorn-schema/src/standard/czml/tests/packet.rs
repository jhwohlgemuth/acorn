use super::super::{Document, Packet};
use crate::codec::DocumentCodec;
use crate::validation::Validate;
use acorn_core::util::MimeType;

#[test]
fn test_codec_reports_nested_paths_and_rejects_trailing_json() {
    let invalid = r#"[{"id":"document","version":"1.0"},{"id":"track","position":{"notAProperty":true}}]"#;
    let diagnostics = Document::decode(invalid, &MimeType::Json).expect_err("unknown position property");
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.path.as_deref().is_some_and(|path| path.contains("position"))));
    assert!(Document::decode(r#"[{"id":"document","version":"1.0"}] true"#, &MimeType::Json).is_err());
    assert!(Document::decode("[]", &MimeType::Yaml).is_err());
}
#[test]
fn test_document_packets_allow_updates_and_preserve_extensions() {
    let source = r#"[{"id":"document","version":"1.0","clock":{"interval":"2016-01-01T00:00:00Z/2017-01-01T00:00:00Z"}},{"id":"facility","position":{"cartographicDegrees":[-75,40,100]},"x_alpha":{"value":1},"agi_conicSensor":{"show":true},"x_beta":2},{"id":"facility","delete":true}]"#;
    let document = Document::decode(source, &MimeType::Json).expect("valid CZML document");
    assert!(document.validate().is_ok());
    let extensions = &document.0.get(1).expect("facility packet").extensions.0;
    assert_eq!(
        extensions.iter().map(|(key, _)| key.as_str()).collect::<Vec<_>>(),
        ["x_alpha", "agi_conicSensor", "x_beta"]
    );
    let encoded = document.encode(&MimeType::Json).expect("encode document");
    let again = Document::decode(&encoded, &MimeType::Json).expect("decode encoded document");
    assert_eq!(again, document);
    assert!(encoded.find("x_alpha").expect("alpha key") < encoded.find("x_beta").expect("beta key"));
}
#[test]
fn test_document_requires_first_document_packet_and_valid_version() {
    let first: Document = serde_json::from_str(r#"[{"id":"facility"},{"id":"document","version":"1.0"}]"#).expect("decode packet array");
    let report = first.validate().expect_err("document packet must come first");
    assert!(report.into_issues().iter().any(|issue| issue.error.code == "czml_document_first"));
    let missing: Document = serde_json::from_str(r#"[{"id":"document"}]"#).expect("decode missing version");
    let report = missing.validate().expect_err("version is required");
    assert!(report.into_issues().iter().any(|issue| issue.error.code == "czml_version"));
    let bad: Document = serde_json::from_str(r#"[{"id":"document","version":"2.0"}]"#).expect("decode unsupported version");
    assert!(bad.validate().is_err());
}
#[test]
fn test_packet_rejects_misplaced_document_fields_and_missing_delete_id() {
    let misplaced: Packet = serde_json::from_str(r#"{"id":"entity","version":"1.0","clock":{}}"#).expect("decode misplaced fields");
    let report = misplaced.validate().expect_err("document fields misplaced");
    let issues = report.into_issues();
    assert!(issues
        .iter()
        .any(|issue| issue.error.code == "czml_document_field" && issue.path == "clock"));
    assert!(issues
        .iter()
        .any(|issue| issue.error.code == "czml_document_field" && issue.path == "version"));
    let deletion: Packet = serde_json::from_str(r#"{"delete":true}"#).expect("decode anonymous deletion");
    let report = deletion.validate().expect_err("deletion requires id");
    assert!(report.into_issues().iter().any(|issue| issue.error.code == "czml_delete_id"));
}
#[test]
fn test_sample_times_must_increase_within_a_property() {
    let source =
        r#"[{"id":"document","version":"1.0"},{"id":"track","position":{"cartesian":["2016-01-02T00:00:00Z",1,2,3,"2016-01-01T00:00:00Z",4,5,6]}}]"#;
    let document: Document = serde_json::from_str(source).expect("decode sampled track");
    let report = document.validate().expect_err("descending sample times");
    assert!(report.into_issues().iter().any(|issue| issue.error.code == "czml_sample_order"));
}
