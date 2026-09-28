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
