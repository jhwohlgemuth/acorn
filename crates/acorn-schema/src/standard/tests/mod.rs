mod cff;
#[cfg(feature = "std")]
mod ckan;
mod datacite;
mod dublin_core;
#[cfg(feature = "std")]
mod huwise;
mod invenio;
mod ocp;
mod text;

fn canonical_json(value: serde_json::Value) -> serde_json::Value {
    match value {
        | serde_json::Value::Array(values) => serde_json::Value::Array(values.into_iter().map(canonical_json).collect()),
        | serde_json::Value::Object(values) => serde_json::Value::Object(
            values
                .into_iter()
                .map(|(key, value)| (key, canonical_json(value)))
                .collect::<std::collections::BTreeMap<_, _>>()
                .into_iter()
                .collect(),
        ),
        | value => value,
    }
}

#[test]
fn test_czml_standard_detects_only_document_first_arrays() {
    use super::Standard;
    use serde_json::json;

    assert_eq!(serde_json::to_value(Standard::Czml).expect("serialize CZML standard"), "czml");
    assert_eq!(
        Standard::detect_value(&json!([{"id": "document"}, {"id": "entity"}])),
        Some(Standard::Czml)
    );
    assert_eq!(Standard::detect_value(&json!([])), None);
    assert_eq!(Standard::detect_value(&json!([{"id": "entity"}, {"id": "document"}])), None);
    assert_eq!(Standard::detect_value(&json!([{"version": "1.0"}])), None);
    assert_eq!(
        Standard::detect_value(&json!({"contact": {}, "meta": {}, "title": "RAD"})),
        Some(Standard::ResearchActivityData)
    );
}
#[test]
fn test_oci_standard_serializes_to_machine_identifier() {
    assert_eq!(serde_json::to_value(super::Standard::Oci).expect("serialize OCI standard"), "oci");
}
