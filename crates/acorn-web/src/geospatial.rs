//! Local ROR joins for validated Research Activity Data; no registry or map effects.
use crate::research_index::{Project, ResearchIndex};
use crate::session::{SuppliedFile, Workspace};
use acorn_schema::pid::{PersistentIdentifier, PersistentIdentifierParse, ROR};
use alloc::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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
    /// Stable identifiers of validated projects associated with this organization.
    pub project_ids: BTreeSet<String>,
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
/// Serialize the bounded Cesium bridge view without repeating activity paths or registry records.
pub fn map_payload(markers: &[Marker], projects: &[Project], selected: Option<&str>) -> Value {
    json!({
        "version": 1,
        "projects": projects.iter().map(|project| json!({
            "id": project.id,
            "title": project.title,
            "activity_count": project.paths.len(),
        })).collect::<Vec<_>>(),
        "markers": markers.iter().map(|marker| json!({
            "id": marker.id,
            "name": marker.name,
            "references": marker.references,
            "project_ids": marker.project_ids,
            "location": marker.location,
        })).collect::<Vec<_>>(),
        "selected": selected,
    })
}
/// Join all validated session records to the local index, without resolving identifiers.
pub fn markers(workspace: &Workspace, index: &LocationIndex) -> Vec<Marker> {
    markers_with_identifiers(workspace, index, &BTreeSet::new())
}
/// Include explicitly selected RORs without inventing activity references or changing RAD.
pub fn markers_with_identifiers(workspace: &Workspace, index: &LocationIndex, identifiers: &BTreeSet<String>) -> Vec<Marker> {
    let catalog = ResearchIndex::from(workspace);
    let projects_by_path = catalog
        .activities
        .iter()
        .map(|activity| (activity.path.as_str(), activity.project_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    references(workspace, identifiers)
        .into_iter()
        .map(|(id, (count, activities))| {
            let organization = index.organizations.get(&id);
            Marker {
                name: organization.map(|org| org.name.clone()).unwrap_or_else(|| id.clone()),
                references: count,
                project_ids: activities
                    .iter()
                    .filter_map(|path| projects_by_path.get(path.as_str()).copied())
                    .map(str::to_owned)
                    .collect(),
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
/// Canonical ROR records needed to resolve the current validated workspace and added locations.
pub fn referenced_identifiers(workspace: &Workspace, identifiers: &BTreeSet<String>) -> BTreeSet<String> {
    references(workspace, identifiers).into_keys().collect()
}
fn references(workspace: &Workspace, identifiers: &BTreeSet<String>) -> BTreeMap<String, (usize, BTreeSet<String>)> {
    let mut references = BTreeMap::new();
    for (file, value) in workspace.activities() {
        collect(value, "", file, &mut references);
    }
    for id in identifiers.iter().filter_map(|id| canonical_ror(id)) {
        references.entry(id).or_default();
    }
    references
}
