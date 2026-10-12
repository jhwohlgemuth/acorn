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
fn test_oci_standard_serializes_to_machine_identifier() {
    assert_eq!(serde_json::to_value(super::Standard::Oci).expect("serialize OCI standard"), "oci");
}
