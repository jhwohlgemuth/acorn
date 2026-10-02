#[cfg(feature = "std")]
mod ckan;
mod datacite;
#[cfg(feature = "std")]
mod huwise;
mod invenio;
mod ocp;
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
