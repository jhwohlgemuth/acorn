//! Local ROR joins for validated Research Activity Data; no registry or map effects.
use crate::session::{SuppliedFile, Workspace};
use acorn_schema::pid::{PersistentIdentifier, PersistentIdentifierParse, ROR};
use alloc::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Maximum organizations sent to the map and displayed in its equivalent table.
pub const MARKER_LIMIT: usize = 1000;
/// Geographic coordinates from the ROR dump.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Location {
    /// Country display name.
    pub country: String,
    /// Latitude in degrees.
    pub latitude: f64,
    /// Longitude in degrees.
    pub longitude: f64,
}
/// Pinned, generated location index, loaded only on the map route.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LocationIndex {
    /// ROR release version.
    pub version: String,
    /// Release date.
    pub date: String,
    /// Immutable source DOI.
    pub source: String,
    /// Digest of the complete source JSON dump.
    pub source_sha256: String,
    /// Source license.
    pub license: String,
    /// Canonical ROR URL to display name and locations.
    pub organizations: BTreeMap<String, Organization>,
}
/// Immutable map/table projection; source references remain visible without WebGL.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Marker {
    /// Canonical ROR URL and stable selection key.
    pub id: String,
    /// Name from the pinned local registry, or the unresolved identifier.
    pub name: String,
    /// Number of source references, including repeated roles.
    pub references: usize,
    /// Related relative file paths.
    pub activities: BTreeSet<String>,
    /// First deterministic location, if coordinates are available.
    pub location: Option<Location>,
    /// Whether the ROR was present in the pinned index.
    pub resolved: bool,
}
/// Minimal ROR display record, including locationless organizations.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Organization {
    /// ROR display name.
    pub name: String,
    /// Locations in deterministic country/coordinate order.
    pub locations: Vec<Location>,
}
/// Canonicalize a syntactically valid ROR identifier without a network lookup.
pub fn canonical_ror(value: &str) -> Option<String> {
    ROR::is_valid(value.trim()).then(|| format!("https://ror.org/{}", ROR::from_string(value.trim()).identifier()))
}
fn collect(value: &Value, field: &str, file: &SuppliedFile, references: &mut BTreeMap<String, (usize, BTreeSet<String>)>) {
    match value {
        | Value::Object(fields) => {
            let typed_ror = fields
                .get("scheme")
                .and_then(Value::as_str)
                .is_some_and(|scheme| scheme.eq_ignore_ascii_case("ror"));
            for (key, value) in fields {
                collect(value, if typed_ror && key == "value" { "ror" } else { key }, file, references);
            }
        }
        | Value::Array(values) => {
            for value in values {
                collect(value, field, file, references);
            }
        }
        | Value::String(text)
            if (field == "ror" || text.starts_with("https://ror.org/") || text.starts_with("http://ror.org/")) && ROR::is_valid(text.trim()) =>
        {
            let id = format!("https://ror.org/{}", ROR::from_string(text.trim()).identifier());
            let entry = references.entry(id).or_default();
            entry.0 = entry.0.saturating_add(1);
            entry.1.insert(file.path.clone());
        }
        | _ => {}
    }
}
/// Join all validated session records to the local index, without resolving identifiers.
pub fn markers(workspace: &Workspace, index: &LocationIndex) -> Vec<Marker> {
    markers_with_identifiers(workspace, index, &BTreeSet::new())
}
/// Include explicitly selected RORs without inventing activity references or changing RAD.
pub fn markers_with_identifiers(workspace: &Workspace, index: &LocationIndex, identifiers: &BTreeSet<String>) -> Vec<Marker> {
    let mut references = BTreeMap::<String, (usize, BTreeSet<String>)>::new();
    for (file, value) in workspace.activities() {
        collect(value, "", file, &mut references);
    }
    for id in identifiers.iter().filter_map(|id| canonical_ror(id)) {
        references.entry(id).or_default();
    }
    references
        .into_iter()
        .map(|(id, (count, activities))| {
            let organization = index.organizations.get(&id);
            Marker {
                name: organization.map(|org| org.name.clone()).unwrap_or_else(|| id.clone()),
                references: count,
                activities,
                location: organization
                    .and_then(|org| {
                        org.locations.iter().find(|location| {
                            location.latitude.is_finite()
                                && location.longitude.is_finite()
                                && location.latitude.abs() <= 90.0
                                && location.longitude.abs() <= 180.0
                        })
                    })
                    .cloned(),
                resolved: organization.is_some(),
                id,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Selection;
    use serde_json::json;

    #[test]
    fn test_local_join_deduplicates_and_preserves_unresolved_locationless_records() {
        let mut workspace = Workspace::default().replace(Selection {
            files: vec![SuppliedFile {
                path: "sample.json".into(),
                content: "{}".into(),
            }],
            skipped: 0,
        });
        workspace.validations.insert("sample.json".into(), json!({"valid":true,"value":{"meta":{"ror":["01qz5mb56","https://ror.org/01qz5mb56","http://ror.org/01qz5mb56","invalid","https://ror.org/04aj4c181","https://ror.org/05bnh6r87"]}}}));
        let index: LocationIndex = serde_json::from_value(json!({"version":"fixture","date":"fixture","source":"fixture","sourceSha256":"fixture","license":"CC0-1.0","organizations":{
            "https://ror.org/01qz5mb56":{"name":"Example","locations":[{"country":"US","latitude":91,"longitude":0},{"country":"US","latitude":35,"longitude":-84}]},
            "https://ror.org/04aj4c181":{"name":"Locationless","locations":[]}
        }})).unwrap();
        let rows = markers(&workspace, &index);
        let first = rows.iter().find(|marker| marker.id.ends_with("01qz5mb56")).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(first.id, "https://ror.org/01qz5mb56");
        assert!(first.resolved);
        assert_eq!(first.references, 3);
        assert_eq!(first.location.as_ref().unwrap().latitude, 35.0);
        assert_eq!(first.activities.len(), 1);
        assert!(rows.iter().any(|marker| marker.resolved && marker.location.is_none()));
        assert!(rows.iter().any(|marker| !marker.resolved));
        assert!(markers(&Workspace::default(), &index).is_empty());
        let pins = BTreeSet::from([
            "01qz5mb56".into(),
            "http://ror.org/01qz5mb56".into(),
            "https://ror.org/04aj4c181".into(),
            "https://ror.org/05bnh6r87".into(),
            "https://ror.org/01qz5mb56xxx".into(),
            "not-a-ror".into(),
        ]);
        let standalone = markers_with_identifiers(&Workspace::default(), &index, &pins);
        assert_eq!(standalone.len(), 3);
        assert!(standalone.iter().all(|marker| marker.references == 0 && marker.activities.is_empty()));
        assert_eq!(standalone.iter().filter(|marker| marker.location.is_some()).count(), 1);
        assert!(standalone.iter().any(|marker| marker.resolved && marker.location.is_none()));
        assert!(standalone.iter().any(|marker| !marker.resolved && marker.location.is_none()));
        assert_eq!(markers_with_identifiers(&workspace, &index, &pins), rows);
        assert_eq!(canonical_ror(" 01qz5mb56 "), Some("https://ror.org/01qz5mb56".into()));
        assert_eq!(canonical_ror("https://ror.org/01qz5mb56xxx"), None);
    }
}
