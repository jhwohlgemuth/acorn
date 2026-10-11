use crate::io::vocabulary::*;
use acorn_schema::vocabulary::{VocabularyFuzzyMode, VocabularyKind, VocabularyOptions, VocabularyResolver};
use std::fs::{read_dir, write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;

fn audit_entry(applied: bool) -> VocabularyAuditEntry {
    VocabularyAuditEntry {
        applied,
        candidates: serde_json::Value::Array(Vec::new()),
        complete: true,
        edit_distance: Some(1),
        evidence: Some("fuzzy".to_string()),
        method: Some("optimal-string-alignment".to_string()),
        mode: "relaxed".to_string(),
        original: "Rst".to_string(),
        outcome: "fuzzy-accepted".to_string(),
        policy_revision: "vocabulary-v2".to_string(),
        replacement: Some("Rust".to_string()),
        runner_up_margin: None,
        score: None,
        source_locator: Some("meta.technology[0]".to_string()),
        variant: Some("restricted-optimal-string-alignment".to_string()),
        vocabulary: "technology".to_string(),
        vocabulary_revision: "technology-vocabulary-v2".to_string(),
    }
}
#[test]
fn test_audit_retention_keeps_latest_thousand_reports() {
    let directory = tempdir().expect("audit directory");
    (0..1_002).for_each(|index| {
        write(directory.path().join(format!("{index:04}.json")), "{}").expect("audit report");
    });
    retain_vocabulary_audits(directory.path()).expect("audit retention");
    assert_eq!(read_dir(directory.path()).expect("audit directory").count(), MAX_AUDIT_REPORTS);
    assert!(!directory.path().join("0000.json").exists());
    assert!(directory.path().join("1001.json").exists());
}
#[test]
fn test_audit_serialization_contains_fingerprints_evidence_and_sources() {
    let sources = vec![
        audit_source(
            "https://user:secret@example.test/first.json?token=private",
            "before",
            "after",
            vec![audit_entry(true)],
        ),
        audit_source("second.json", "old", "new", vec![audit_entry(true)]),
    ];
    let audit = VocabularyAuditLog {
        created_at_unix_millis: 1,
        schema_version: 1,
        sources,
    };
    let value = serde_json::to_value(audit).expect("audit JSON");
    let sources = value.get("sources").and_then(serde_json::Value::as_array).expect("audit sources");
    let first = sources.first().expect("first audit source");
    let first_change = first
        .get("changes")
        .and_then(serde_json::Value::as_array)
        .and_then(|changes| changes.first())
        .expect("first audit change");
    assert_eq!(sources.len(), 2);
    assert!(first.get("beforeContentFingerprint").is_some_and(serde_json::Value::is_string));
    assert_eq!(first_change.get("evidence").and_then(serde_json::Value::as_str), Some("fuzzy"));
    assert_eq!(
        first.get("sourceUri").and_then(serde_json::Value::as_str),
        Some("https://example.test/first.json")
    );
    assert!(value.get("document").is_none());
}
#[cfg(unix)]
#[test]
fn test_audit_writer_uses_owner_only_permissions() {
    let directory = tempdir().expect("audit directory");
    let path = directory.path().join("audit.json");
    write_audit_file(&path, b"{}").expect("audit report");
    let mode = path.metadata().expect("audit metadata").permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}
#[test]
fn test_cache_corruption_is_ignored_and_rebuilt() {
    let directory = tempdir().expect("cache directory");
    let path = directory.path().join("keywords.json");
    write(&path, "not-json").expect("corrupt cache");
    let resolver = VocabularyResolver::embedded(VocabularyKind::Keywords).expect("embedded vocabulary");
    assert!(read_cached(&path, VocabularyKind::Keywords).is_none());
    write_cached(&path, &resolver);
    assert!(read_cached(&path, VocabularyKind::Keywords).is_some());
}
#[test]
fn test_cache_mismatched_asset_digest_is_ignored() {
    let directory = tempdir().expect("cache directory");
    let path = directory.path().join("partners.json");
    let resolver = VocabularyResolver::embedded(VocabularyKind::Partners).expect("embedded vocabulary");
    let mut record = cache_record(resolver).expect("cache record");
    record.asset_digest = "stale".to_string();
    write(&path, serde_json::to_string(&record).expect("cache JSON")).expect("cache entry");
    assert!(read_cached(&path, VocabularyKind::Partners).is_none());
}
#[test]
fn test_cache_parity_is_identical() {
    let directory = tempdir().expect("cache directory");
    let path = directory.path().join("technology.json");
    let uncached = VocabularyResolver::embedded(VocabularyKind::Technology).expect("embedded vocabulary");
    write_cached(&path, &uncached);
    let cached = read_cached(&path, VocabularyKind::Technology).expect("cached vocabulary");
    let options = VocabularyOptions {
        fuzzy: VocabularyFuzzyMode::Relaxed,
        ..VocabularyOptions::default()
    };
    assert_eq!(
        uncached.resolve(["Rst", "Python"], options).expect("uncached report"),
        cached.resolve(["Rst", "Python"], options).expect("cached report")
    );
}
#[test]
fn test_clear_cache_is_bounded_to_supplied_namespace() {
    let directory = tempdir().expect("cache directory");
    write(directory.path().join("one.json"), "{}").expect("cache entry");
    assert_eq!(clear_vocabulary_cache_from(directory.path()).expect("clear cache"), 1);
    assert!(!directory.path().exists());
}
#[test]
fn test_unchanged_audit_does_not_create_report() {
    let source = audit_source("fixture", "before", "after", vec![audit_entry(false)]);
    assert_eq!(write_vocabulary_audit(vec![source]).expect("unchanged audit"), None);
}
