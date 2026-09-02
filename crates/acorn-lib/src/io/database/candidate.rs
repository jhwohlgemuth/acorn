use super::backend::{self, BackendRow, Connection};
use super::schema::{ResearchActivityRow, Table};
use super::{Database, Row};
use crate::io::api::handle::Record;
use crate::io::ApiResult;
use acorn_core::util::to_rfc3339;
use acorn_schema::schema::discovery::RemoteEntity;
use acorn_schema::schema::pid;
use alloc::collections::BTreeSet;
use color_eyre::eyre::eyre;
use jiff::Timestamp;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub(crate) enum Provenance {
    Bucket {
        bucket: Option<String>,
        repository: String,
        relative_path: String,
        observed_at: String,
    },
    Citeas {
        title: String,
        doi: String,
        project_url: String,
    },
    Discovery {
        source: String,
        source_format: String,
        identifier: String,
        resolution_status: String,
        observed_at: String,
    },
    Handle {
        record: Record,
    },
    Osti {
        provider_identifier: String,
        entity: RemoteEntity,
        metadata: Value,
        observed_at: String,
    },
    Raid {
        records: Vec<pid::raid::Metadata>,
    },
}
/// Canonical partial RAD data and the evidence used to identify it.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResearchActivityCandidate {
    /// Partial or complete RAD-shaped JSON.
    pub rad_json: Value,
    /// Normalized project identities used for exact matching.
    pub identity_keys: Vec<String>,
    /// Source and field observations supporting the candidate.
    pub provenance: Vec<Value>,
}
impl ResearchActivityCandidate {
    /// Construct a candidate with sorted, deduplicated identity keys and provenance.
    pub fn new(rad_json: Value, identity_keys: Vec<String>, provenance: Vec<Value>) -> Self {
        Self {
            rad_json: canonicalize(rad_json),
            identity_keys: normalize_identity_keys(identity_keys),
            provenance: deduplicate_provenance(provenance),
        }
    }
}
/// Result of canonical candidate persistence.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CandidateAction {
    /// A new canonical row was created.
    Created,
    /// An existing row gained fields, identities, or provenance.
    Enriched,
    /// The candidate was already represented exactly.
    Unchanged,
    /// Conflicting fields or multiple matching rows prevented an unambiguous merge.
    Conflict,
}
/// Portable identifier and action returned by candidate persistence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CandidatePersistence {
    /// Portable row identifier, absent when several rows matched.
    pub iid: Option<String>,
    /// Persistence action.
    pub action: CandidateAction,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct MergeResult {
    rad_json: Value,
    identity_keys: Vec<String>,
    provenance: Vec<Value>,
    conflicts: Vec<Value>,
    changed: bool,
}
trait ConnectionExt {
    fn persist_candidate(&self, candidate: ResearchActivityCandidate) -> ApiResult<CandidatePersistence>;
    fn create_candidate(&self, candidate: ResearchActivityCandidate) -> ApiResult<CandidatePersistence>;
    fn merge_candidate(&self, row: &ResearchActivityRow, candidate: ResearchActivityCandidate) -> ApiResult<CandidatePersistence>;
    fn research_activities(&self) -> ApiResult<Vec<ResearchActivityRow>>;
    fn unique_research_activity_iid(&self) -> ApiResult<String>;
}
impl Database<Table> {
    /// Select a canonical research activity by portable identifier.
    pub fn research_activity_by_iid(&self, iid: &str) -> ApiResult<Option<ResearchActivityRow>> {
        self.migrate_table(Table::ResearchActivities).and_then(|()| {
            self.with_connection(|conn| {
                conn.query_row(
                    "SELECT id, iid, rad_json, identity_keys_json, provenance_json, created_at, updated_at FROM research_activities WHERE iid = ?",
                    backend::params![iid],
                    |row| Ok(ResearchActivityRow::from(row)),
                )
                .map(Some)
                .or_else(|why| match is_not_found(&why) {
                    | true => Ok(None),
                    | false => Err(eyre!("Failed to select research activity {iid} — {why}")),
                })
            })
        })
    }
    /// Select rows sharing at least one exact normalized project identity.
    pub fn research_activities_by_identity(&self, identity_keys: &[String]) -> ApiResult<Vec<ResearchActivityRow>> {
        let keys = normalize_identity_keys(identity_keys.to_vec());
        self.migrate_table(Table::ResearchActivities)
            .and_then(|()| self.with_connection(|conn| conn.research_activities().and_then(|rows| matching_rows(rows, &keys))))
    }
    /// Transactionally create a canonical candidate or enrich its unique exact match.
    pub fn create_or_enrich(&self, candidate: ResearchActivityCandidate) -> ApiResult<CandidatePersistence> {
        let candidate = ResearchActivityCandidate::new(candidate.rad_json, candidate.identity_keys, candidate.provenance);
        match candidate.identity_keys.is_empty() {
            | true => Err(eyre!("Research activity candidate has no project identity keys")),
            | false => self.migrate_table(Table::ResearchActivities).and_then(|()| {
                self.with_connection(|conn| {
                    conn.execute_batch("BEGIN TRANSACTION")
                        .map_err(|why| eyre!("Failed to begin research activity transaction — {why}"))?;
                    let result = conn.persist_candidate(candidate);
                    match result {
                        | Ok(value) => conn
                            .execute_batch("COMMIT")
                            .map_err(|why| eyre!("Failed to commit research activity transaction — {why}"))
                            .map(|()| value),
                        | Err(why) => {
                            let _ = conn.execute_batch("ROLLBACK");
                            Err(why)
                        }
                    }
                })
            }),
        }
    }
}
impl ConnectionExt for Connection {
    fn persist_candidate(&self, candidate: ResearchActivityCandidate) -> ApiResult<CandidatePersistence> {
        let matches = matching_rows(self.research_activities()?, &candidate.identity_keys)?;
        match matches.as_slice() {
            | [] => self.create_candidate(candidate),
            | [row] => self.merge_candidate(row, candidate),
            | _ => Ok(CandidatePersistence {
                iid: None,
                action: CandidateAction::Conflict,
            }),
        }
    }
    fn create_candidate(&self, candidate: ResearchActivityCandidate) -> ApiResult<CandidatePersistence> {
        let iid = self.unique_research_activity_iid()?;
        let now = Timestamp::now();
        ResearchActivityRow::init()
            .iid(iid.clone())
            .rad_json(serialize(&candidate.rad_json)?)
            .identity_keys_json(serialize(&candidate.identity_keys)?)
            .provenance_json(serialize(&candidate.provenance)?)
            .created_at(now)
            .updated_at(now)
            .build()
            .insert(self)
            .map(|_| CandidatePersistence {
                iid: Some(iid),
                action: CandidateAction::Created,
            })
    }
    fn merge_candidate(&self, row: &ResearchActivityRow, candidate: ResearchActivityCandidate) -> ApiResult<CandidatePersistence> {
        let iid = row.iid.clone().ok_or_else(|| eyre!("Research activity row is missing iid"))?;
        let merged = merge_candidate(row, candidate)?;
        let action = match (merged.conflicts.is_empty(), merged.changed) {
            | (false, _) => CandidateAction::Conflict,
            | (true, true) => CandidateAction::Enriched,
            | (true, false) => CandidateAction::Unchanged,
        };
        match merged.changed {
            | false => Ok(CandidatePersistence { iid: Some(iid), action }),
            | true => self
                .execute(
                    "UPDATE research_activities SET rad_json = ?, identity_keys_json = ?, provenance_json = ?, updated_at = ? WHERE iid = ?",
                    backend::params![
                        serialize(&merged.rad_json)?,
                        serialize(&merged.identity_keys)?,
                        serialize(&merged.provenance)?,
                        to_rfc3339(Timestamp::now()),
                        iid,
                    ],
                )
                .map_err(|why| eyre!("Failed to enrich research activity — {why}"))
                .map(|_| CandidatePersistence {
                    iid: row.iid.clone(),
                    action,
                }),
        }
    }
    fn research_activities(&self) -> ApiResult<Vec<ResearchActivityRow>> {
        self.prepare("SELECT id, iid, rad_json, identity_keys_json, provenance_json, created_at, updated_at FROM research_activities")
            .map_err(|why| eyre!("Failed to prepare research activity lookup — {why}"))
            .and_then(|mut statement| {
                statement
                    .query_map(backend::params![], |row: &BackendRow<'_>| Ok(ResearchActivityRow::from(row)))
                    .map_err(|why| eyre!("Failed to query research activities — {why}"))
                    .and_then(|rows| {
                        rows.collect::<core::result::Result<Vec<_>, _>>()
                            .map_err(|why| eyre!("Failed to read research activities — {why}"))
                    })
            })
    }
    fn unique_research_activity_iid(&self) -> ApiResult<String> {
        (0..8)
            .map(|_| nanoid!())
            .find_map(|iid| {
                self.query_row(
                    "SELECT COUNT(*) FROM research_activities WHERE iid = ?",
                    backend::params![iid.clone()],
                    |row| row.get::<_, i64>(0),
                )
                .ok()
                .filter(|count| *count == 0)
                .map(|_| iid)
            })
            .ok_or_else(|| eyre!("Failed to generate a unique research activity iid"))
    }
}
fn merge_candidate(row: &ResearchActivityRow, candidate: ResearchActivityCandidate) -> ApiResult<MergeResult> {
    let existing_rad = parse_value(row.rad_json.as_deref(), "rad_json")?;
    let existing_keys = parse_strings(row.identity_keys_json.as_deref(), "identity_keys_json")?;
    let existing_provenance = parse_values(row.provenance_json.as_deref(), "provenance_json")?;
    let (rad_json, conflicts) = merge_values(existing_rad.clone(), candidate.rad_json, String::new());
    let identity_keys = normalize_identity_keys(existing_keys.clone().into_iter().chain(candidate.identity_keys).collect());
    let conflict_provenance = conflicts.iter().cloned();
    let provenance = deduplicate_provenance(
        existing_provenance
            .clone()
            .into_iter()
            .chain(candidate.provenance)
            .chain(conflict_provenance)
            .collect(),
    );
    let changed = rad_json != existing_rad || identity_keys != existing_keys || provenance != existing_provenance;
    Ok(MergeResult {
        rad_json,
        identity_keys,
        provenance,
        conflicts,
        changed,
    })
}
fn merge_values(existing: Value, incoming: Value, path: String) -> (Value, Vec<Value>) {
    match (existing, incoming) {
        | (Value::Null, incoming) => (incoming, Vec::new()),
        | (existing, Value::Null) => (existing, Vec::new()),
        | (Value::Object(existing), Value::Object(incoming)) => {
            let (value, conflicts) = incoming
                .into_iter()
                .fold((existing, Vec::new()), |(mut merged, conflicts), (key, value)| {
                    let child_path = if path.is_empty() { key.clone() } else { format!("{path}.{key}") };
                    let (value, child_conflicts) = match merged.remove(&key) {
                        | Some(current) => merge_values(current, value, child_path),
                        | None => (value, Vec::new()),
                    };
                    merged.insert(key, value);
                    (merged, conflicts.into_iter().chain(child_conflicts).collect())
                });
            (canonicalize(Value::Object(value)), conflicts)
        }
        | (Value::Array(existing), Value::Array(incoming)) => {
            let merged = incoming.into_iter().fold(existing, |mut values, value| {
                if !values.contains(&value) {
                    values.push(value);
                }
                values
            });
            (Value::Array(merged), Vec::new())
        }
        | (existing, incoming) if existing == incoming => (existing, Vec::new()),
        | (existing, incoming) => {
            let conflict = serde_json::json!({
                "kind": "field-conflict",
                "path": path,
                "existing": existing,
                "observed": incoming,
            });
            (conflict.get("existing").cloned().unwrap_or(Value::Null), vec![conflict])
        }
    }
}
fn canonicalize(value: Value) -> Value {
    match value {
        | Value::Object(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| (key, canonicalize(value)))
                .collect::<alloc::collections::BTreeMap<_, _>>()
                .into_iter()
                .collect::<Map<_, _>>(),
        ),
        | Value::Array(values) => Value::Array(values.into_iter().map(canonicalize).collect()),
        | value => value,
    }
}
fn normalize_identity_keys(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| normalize_identity_key(&value))
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
fn normalize_identity_key(value: &str) -> String {
    let value = value.trim();
    match value.split_once(':') {
        | Some((prefix, identifier)) if matches!(prefix.to_ascii_lowercase().as_str(), "doi" | "raid" | "isbn" | "patent") => {
            format!("{}:{}", prefix.to_ascii_lowercase(), identifier.trim().to_ascii_lowercase())
        }
        | Some((prefix, identifier)) => format!("{}:{}", prefix.to_ascii_lowercase(), identifier.trim()),
        | None => value.to_string(),
    }
}
fn deduplicate_provenance(values: Vec<Value>) -> Vec<Value> {
    values.into_iter().map(canonicalize).fold(Vec::new(), |mut unique, value| {
        let fingerprint = provenance_fingerprint(&value);
        if !unique.iter().any(|existing| provenance_fingerprint(existing) == fingerprint) {
            unique.push(value);
        }
        unique
    })
}
fn provenance_fingerprint(value: &Value) -> Value {
    match value {
        | Value::Object(values) => Value::Object(
            values
                .iter()
                .filter(|(key, _)| !matches!(key.as_str(), "timestamp" | "observed_at" | "discovered_at"))
                .map(|(key, value)| (key.clone(), provenance_fingerprint(value)))
                .collect(),
        ),
        | Value::Array(values) => Value::Array(values.iter().map(provenance_fingerprint).collect()),
        | value => value.clone(),
    }
}
fn matching_rows(rows: Vec<ResearchActivityRow>, keys: &[String]) -> ApiResult<Vec<ResearchActivityRow>> {
    rows.into_iter().try_fold(Vec::new(), |mut matches, row| {
        parse_strings(row.identity_keys_json.as_deref(), "identity_keys_json").map(|row_keys| {
            if row_keys.iter().any(|key| keys.contains(key)) {
                matches.push(row);
            }
            matches
        })
    })
}
fn parse_value(value: Option<&str>, field: &str) -> ApiResult<Value> {
    value
        .ok_or_else(|| eyre!("Research activity row is missing {field}"))
        .and_then(|value| serde_json::from_str(value).map_err(|why| eyre!("Research activity {field} is invalid — {why}")))
}
fn parse_strings(value: Option<&str>, field: &str) -> ApiResult<Vec<String>> {
    value
        .ok_or_else(|| eyre!("Research activity row is missing {field}"))
        .and_then(|value| serde_json::from_str(value).map_err(|why| eyre!("Research activity {field} is invalid — {why}")))
}
fn parse_values(value: Option<&str>, field: &str) -> ApiResult<Vec<Value>> {
    value
        .ok_or_else(|| eyre!("Research activity row is missing {field}"))
        .and_then(|value| serde_json::from_str(value).map_err(|why| eyre!("Research activity {field} is invalid — {why}")))
}
fn serialize(value: &impl Serialize) -> ApiResult<String> {
    serde_json::to_string(value).map_err(|why| eyre!("Failed to serialize research activity candidate — {why}"))
}
fn is_not_found(error: &backend::Error) -> bool {
    error.to_string().contains("Query returned no rows") || error.to_string().contains("no rows")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing)]
    use super::*;
    use std::env::temp_dir;

    fn database() -> Database<Table> {
        Database::from_path(Some(temp_dir().join(format!("acorn-candidate-{}.db", nanoid!()))))
    }
    fn candidate(rad_json: Value, keys: &[&str]) -> ResearchActivityCandidate {
        ResearchActivityCandidate::new(
            rad_json,
            keys.iter().map(|value| (*value).to_string()).collect(),
            vec![serde_json::json!({"source": "test", "observed_at": "2026-01-01T00:00:00Z"})],
        )
    }
    #[test]
    fn serializes_bucket_provenance_with_provider_tag() {
        let value = serde_json::to_value(Provenance::Bucket {
            bucket: Some("science".to_string()),
            repository: "file:/bucket".to_string(),
            relative_path: "project/index.json".to_string(),
            observed_at: "2026-01-01T00:00:00Z".to_string(),
        })
        .unwrap();
        assert_eq!(value["provider"], "bucket");
        assert_eq!(value["bucket"], "science");
        assert_eq!(value["relative_path"], "project/index.json");
    }
    #[test]
    fn identity_normalization_preserves_case_sensitive_key_values() {
        let keys = normalize_identity_keys(vec![
            "DOI:10.1234/ABC".to_string(),
            "doi:10.1234/abc".to_string(),
            "ark:12345/ABC".to_string(),
            "ark:12345/abc".to_string(),
            "rad:file:/Bucket/Project".to_string(),
        ]);
        assert_eq!(keys.len(), 4);
        assert!(keys.contains(&"doi:10.1234/abc".to_string()));
        assert!(keys.contains(&"ark:12345/ABC".to_string()));
        assert!(keys.contains(&"rad:file:/Bucket/Project".to_string()));
    }
    #[test]
    fn creates_round_trips_and_replays_candidate() {
        let database = database();
        let input = candidate(serde_json::json!({"title": "Example"}), &["doi:10.1234/example"]);
        let created = database.create_or_enrich(input.clone()).unwrap();
        assert_eq!(created.action, CandidateAction::Created);
        let iid = created.iid.unwrap();
        assert_eq!(
            database.research_activity_by_iid(&iid).unwrap().unwrap().iid.as_deref(),
            Some(iid.as_str())
        );
        assert_eq!(database.create_or_enrich(input).unwrap().action, CandidateAction::Unchanged);
        assert_eq!(database.row_count(Table::ResearchActivities).unwrap(), 1);
    }
    #[test]
    fn enriches_arrays_and_retains_conflicting_values() {
        let database = database();
        database
            .create_or_enrich(candidate(
                serde_json::json!({"title": "Original", "meta": {"doi": ["10.1234/example"]}}),
                &["doi:10.1234/example"],
            ))
            .unwrap();
        let result = database
            .create_or_enrich(candidate(
                serde_json::json!({"title": "Different", "meta": {"doi": ["10.9999/related"], "raid": ["10.7777/raid"]}}),
                &["doi:10.1234/example", "raid:10.7777/raid"],
            ))
            .unwrap();
        assert_eq!(result.action, CandidateAction::Conflict);
        let row = database.research_activity_by_iid(result.iid.as_deref().unwrap()).unwrap().unwrap();
        let rad: Value = serde_json::from_str(row.rad_json.as_deref().unwrap()).unwrap();
        assert_eq!(rad["title"], "Original");
        assert_eq!(rad["meta"]["doi"], serde_json::json!(["10.1234/example", "10.9999/related"]));
        assert_eq!(rad["meta"]["raid"], serde_json::json!(["10.7777/raid"]));
    }
    #[test]
    fn rejects_ambiguous_multi_row_match_without_mutation() {
        let database = database();
        database
            .create_or_enrich(candidate(serde_json::json!({"title": "One"}), &["doi:one"]))
            .unwrap();
        database
            .create_or_enrich(candidate(serde_json::json!({"title": "Two"}), &["raid:two"]))
            .unwrap();
        let result = database
            .create_or_enrich(candidate(serde_json::json!({"title": "Ambiguous"}), &["doi:one", "raid:two"]))
            .unwrap();
        assert_eq!(
            result,
            CandidatePersistence {
                iid: None,
                action: CandidateAction::Conflict
            }
        );
        assert_eq!(database.row_count(Table::ResearchActivities).unwrap(), 2);
    }
}
