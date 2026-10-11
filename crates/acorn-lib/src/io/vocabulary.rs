//! Native persistence for disposable prepared-vocabulary state.
use crate::io::Fingerprint;
use crate::util::constants::app::{APPLICATION, ORGANIZATION, QUALIFIER};
use acorn_analysis::VocabularyAdvisory;
use acorn_host::fs::{project_cache_directory, project_data_directory, SafePath};
use acorn_schema::vocabulary::{VocabularyError, VocabularyKind, VocabularyReport, VocabularyResolver};
use alloc::collections::BTreeSet;
use serde::{Deserialize, Serialize};
use std::{
    fs::{read_dir, read_to_string, remove_dir_all, remove_file, rename, write, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const CACHE_SCHEMA_VERSION: u8 = 2;
pub(super) const MAX_AUDIT_REPORTS: usize = 1_000;
const UNICODE_DATA_VERSION: &str = "icu4x-2.2.0";
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct CachedVocabulary {
    pub(super) asset_digest: String,
    pub(super) cache_schema_version: u8,
    pub(super) content_digest: String,
    pub(super) kind: VocabularyKind,
    pub(super) policy_revision: String,
    pub(super) resolver: VocabularyResolver,
    pub(super) unicode_data_version: String,
    pub(super) vocabulary_revision: String,
}
#[derive(Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct VocabularyAuditEntry {
    pub(super) applied: bool,
    pub(super) candidates: serde_json::Value,
    pub(super) complete: bool,
    pub(super) edit_distance: Option<usize>,
    pub(super) evidence: Option<String>,
    pub(super) method: Option<String>,
    pub(super) mode: String,
    pub(super) original: String,
    pub(super) outcome: String,
    pub(super) policy_revision: String,
    pub(super) replacement: Option<String>,
    pub(super) runner_up_margin: Option<acorn_core::Noul>,
    pub(super) score: Option<acorn_core::Noul>,
    pub(super) source_locator: Option<String>,
    pub(super) variant: Option<String>,
    pub(super) vocabulary: String,
    pub(super) vocabulary_revision: String,
}
#[derive(Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct VocabularyAuditLog {
    pub(super) created_at_unix_millis: u128,
    pub(super) schema_version: u8,
    pub(super) sources: Vec<VocabularyAuditSource>,
}
#[derive(Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct VocabularyAuditSource {
    pub(super) after_content_fingerprint: String,
    pub(super) before_content_fingerprint: String,
    pub(super) changes: Vec<VocabularyAuditEntry>,
    pub(super) deduplicated_scalar_values: bool,
    pub(super) source_id: String,
    pub(super) source_uri: String,
    pub(super) status: &'static str,
}
pub(super) fn audit_source(source: &str, before: &str, after: &str, changes: Vec<VocabularyAuditEntry>) -> VocabularyAuditSource {
    let deduplicated_scalar_values = changes
        .iter()
        .filter(|change| matches!(change.vocabulary.as_str(), "keywords" | "partners" | "technology"))
        .filter_map(|change| {
            change
                .replacement
                .as_ref()
                .map(|replacement| (change.vocabulary.as_str(), replacement.as_str()))
        })
        .try_fold(BTreeSet::new(), |mut values, value| values.insert(value).then_some(values))
        .is_none();
    VocabularyAuditSource {
        after_content_fingerprint: Fingerprint::from_bytes(after.as_bytes()).to_string(),
        before_content_fingerprint: Fingerprint::from_bytes(before.as_bytes()).to_string(),
        changes,
        deduplicated_scalar_values,
        source_id: Fingerprint::from_bytes(source.as_bytes()).to_string(),
        source_uri: redacted_source_uri(source),
        status: "applied",
    }
}
pub(super) fn cache_record(resolver: VocabularyResolver) -> Option<CachedVocabulary> {
    VocabularyResolver::embedded_asset_digest(resolver.kind())
        .ok()
        .map(|asset_digest| CachedVocabulary {
            asset_digest,
            cache_schema_version: CACHE_SCHEMA_VERSION,
            content_digest: resolver.content_digest(),
            kind: resolver.kind(),
            policy_revision: resolver.policy_revision().to_string(),
            unicode_data_version: UNICODE_DATA_VERSION.to_string(),
            vocabulary_revision: resolver.revision().to_string(),
            resolver,
        })
}
/// Load an equivalent cached resolver when available, otherwise rebuild in memory.
pub fn cached_vocabulary_resolver(kind: VocabularyKind) -> Result<VocabularyResolver, VocabularyError> {
    vocabulary_cache_directory().map_or_else(
        |_| VocabularyResolver::embedded(kind),
        |directory| cached_vocabulary_resolver_in(kind, &directory),
    )
}
/// Load a versioned cached resolver from an explicit cache namespace.
///
/// This variant supports deterministic lifecycle tests and benchmarks without
/// reading or mutating the user's application cache.
pub fn cached_vocabulary_resolver_in(kind: VocabularyKind, directory: &Path) -> Result<VocabularyResolver, VocabularyError> {
    let path = directory.join(format!("{kind}.json"));
    read_cached(&path, kind).map_or_else(
        || {
            VocabularyResolver::embedded(kind).inspect(|embedded| {
                write_cached(&path, embedded);
            })
        },
        Ok,
    )
}
/// Clear the ACORN-managed prepared-vocabulary cache namespace.
pub fn clear_vocabulary_cache() -> Result<usize, String> {
    vocabulary_cache_directory()
        .map_err(|why| why.to_string())
        .and_then(|directory| clear_vocabulary_cache_from(&directory))
}
pub(super) fn clear_vocabulary_cache_from(directory: &Path) -> Result<usize, String> {
    match directory.exists() {
        | false => Ok(0),
        | true => read_dir(directory)
            .map_err(|why| format!("Read vocabulary cache {} — {why}", directory.display()))
            .map(|entries| entries.filter_map(Result::ok).filter(|entry| entry.path().is_file()).count())
            .and_then(|count| {
                remove_dir_all(directory)
                    .map(|()| count)
                    .map_err(|why| format!("Clear vocabulary cache {} — {why}", directory.display()))
            }),
    }
}
/// Clear only the ACORN-managed vocabulary audit-log namespace.
pub fn clear_vocabulary_logs() -> Result<usize, String> {
    vocabulary_log_directory()
        .map_err(|why| why.to_string())
        .and_then(|directory| clear_vocabulary_logs_from(&directory))
}
fn clear_vocabulary_logs_from(directory: &Path) -> Result<usize, String> {
    match directory.exists() {
        | false => Ok(0),
        | true => read_dir(directory)
            .map_err(|why| format!("Read vocabulary logs {} — {why}", directory.display()))
            .map(|entries| entries.filter_map(Result::ok).filter(|entry| entry.path().is_file()).count())
            .and_then(|count| {
                remove_dir_all(directory)
                    .map(|()| count)
                    .map_err(|why| format!("Clear vocabulary logs {} — {why}", directory.display()))
            }),
    }
}
pub(super) fn read_cached(path: &Path, kind: VocabularyKind) -> Option<VocabularyResolver> {
    VocabularyResolver::embedded_asset_digest(kind).ok().and_then(|expected_asset_digest| {
        read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str::<CachedVocabulary>(&content).ok())
            .filter(|cached| {
                cached.asset_digest == expected_asset_digest
                    && cached.cache_schema_version == CACHE_SCHEMA_VERSION
                    && cached.kind == kind
                    && cached.policy_revision == cached.resolver.policy_revision()
                    && cached.unicode_data_version == UNICODE_DATA_VERSION
                    && cached.vocabulary_revision == cached.resolver.revision()
                    && cached.resolver.content_digest() == cached.content_digest
                    && cached.resolver.kind() == cached.kind
            })
            .map(|cached| cached.resolver)
    })
}
fn redacted_source_uri(source: &str) -> String {
    let base = source.split(['?', '#']).next().unwrap_or(source);
    base.split_once("://").map_or_else(
        || base.to_string(),
        |(scheme, remainder)| {
            let (authority, path) = remainder.split_once('/').map_or((remainder, ""), |(authority, path)| (authority, path));
            let authority = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
            match path.is_empty() {
                | true => format!("{scheme}://{authority}"),
                | false => format!("{scheme}://{authority}/{path}"),
            }
        },
    )
}
pub(super) fn retain_vocabulary_audits(directory: &Path) -> Result<(), String> {
    read_dir(directory)
        .map_err(|why| format!("Read vocabulary audit directory {} — {why}", directory.display()))
        .map(|entries| {
            let mut paths = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|extension| extension == "json"))
                .collect::<Vec<_>>();
            paths.sort();
            paths
        })
        .and_then(|paths| {
            let excess = paths.len().saturating_sub(MAX_AUDIT_REPORTS);
            paths
                .into_iter()
                .take(excess)
                .try_for_each(|path| remove_file(&path).map_err(|why| format!("Prune vocabulary audit {} — {why}", path.display())))
        })
}
fn vocabulary_cache_directory() -> Result<PathBuf, acorn_host::fs::FsError> {
    SafePath::new("standards/vocabulary").and_then(|namespace| project_cache_directory(QUALIFIER, ORGANIZATION, APPLICATION, &namespace))
}
fn vocabulary_log_directory() -> Result<PathBuf, acorn_host::fs::FsError> {
    SafePath::new("logs/vocabulary").and_then(|namespace| project_data_directory(QUALIFIER, ORGANIZATION, APPLICATION, &namespace))
}
pub(super) fn write_audit_file(path: &Path, content: &[u8]) -> Result<(), String> {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(path)
        .map_err(|why| format!("Create vocabulary audit {} — {why}", path.display()))
        .and_then(|mut file| {
            file.write_all(content)
                .and_then(|()| file.sync_all())
                .map_err(|why| format!("Write vocabulary audit {} — {why}", path.display()))
        })
}
pub(super) fn write_cached(path: &Path, resolver: &VocabularyResolver) {
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    cache_record(resolver.clone())
        .and_then(|record| serde_json::to_string(&record).ok())
        .and_then(|content| write(&temporary, content).ok())
        .and_then(|()| rename(&temporary, path).ok());
    if temporary.exists() {
        let _ = remove_file(temporary);
    }
}
/// Persist one combined log for deterministic vocabulary fixes after they apply.
pub fn write_vocabulary_advisory_audit<'a>(
    source: &str,
    before: &str,
    after: &str,
    advisories: impl IntoIterator<Item = &'a VocabularyAdvisory>,
) -> Result<Option<PathBuf>, String> {
    let changes = advisories
        .into_iter()
        .map(|advisory| VocabularyAuditEntry {
            applied: advisory
                .replacement
                .as_deref()
                .is_some_and(|replacement| replacement != advisory.original),
            candidates: serde_json::to_value(&advisory.candidates).unwrap_or_default(),
            complete: advisory.complete,
            edit_distance: advisory.edit_distance,
            evidence: advisory.evidence.clone(),
            method: advisory.method.clone(),
            mode: advisory.mode.to_string(),
            original: advisory.original.clone(),
            outcome: advisory.outcome.to_string(),
            policy_revision: advisory.policy_revision.clone(),
            replacement: advisory.replacement.clone(),
            runner_up_margin: advisory.runner_up_margin,
            score: advisory.score,
            source_locator: Some(advisory.source_locator.clone()),
            variant: advisory.variant.clone(),
            vocabulary: advisory.vocabulary.clone(),
            vocabulary_revision: advisory.vocabulary_revision.clone(),
        })
        .collect::<Vec<_>>();
    write_vocabulary_audit(vec![audit_source(source, before, after, changes)])
}
pub(super) fn write_vocabulary_audit(sources: Vec<VocabularyAuditSource>) -> Result<Option<PathBuf>, String> {
    match sources.iter().flat_map(|source| &source.changes).any(|change| change.applied) {
        | false => Ok(None),
        | true => SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|why| format!("Resolve vocabulary audit time — {why}"))
            .and_then(|elapsed| {
                let created_at_unix_millis = elapsed.as_millis();
                let source_id =
                    Fingerprint::from_entries(sources.iter().map(|source| (source.source_id.clone(), source.source_id.as_bytes()))).to_string();
                let audit = VocabularyAuditLog {
                    created_at_unix_millis,
                    schema_version: 1,
                    sources,
                };
                vocabulary_log_directory().map_err(|why| why.to_string()).and_then(|directory| {
                    serde_json::to_vec(&audit)
                        .map_err(|why| format!("Serialize vocabulary audit — {why}"))
                        .and_then(|content| {
                            (content.len() <= 1024 * 1024)
                                .then_some(content)
                                .ok_or_else(|| "Vocabulary audit exceeds the 1 MiB report limit".to_string())
                        })
                        .and_then(|content| {
                            let short_source = source_id.get(..12).unwrap_or(source_id.as_str());
                            let filename = format!("{}-{}-{short_source}.json", elapsed.as_nanos(), std::process::id());
                            let final_path = directory.join(filename);
                            let temporary = final_path.with_extension("json.tmp");
                            write_audit_file(&temporary, &content)
                                .and_then(|()| {
                                    rename(&temporary, &final_path)
                                        .map_err(|why| format!("Publish vocabulary audit {} — {why}", final_path.display()))
                                })
                                .and_then(|()| {
                                    retain_vocabulary_audits(&directory).map(|()| Some(final_path.clone())).inspect_err(|_| {
                                        remove_file(&final_path).ok();
                                    })
                                })
                                .inspect_err(|_| {
                                    remove_file(&temporary).ok();
                                })
                        })
                })
            }),
    }
}
/// Persist one combined invocation log for report-bearing vocabulary formatting operations.
pub fn write_vocabulary_report_audits<'a>(
    sources: impl IntoIterator<Item = (&'a str, &'a str, &'a str, &'a [VocabularyReport])>,
) -> Result<Option<PathBuf>, String> {
    let sources = sources
        .into_iter()
        .map(|(source, before, after, reports)| {
            let changes = reports
                .iter()
                .flat_map(|report| {
                    report.resolutions.iter().map(|resolution| VocabularyAuditEntry {
                        applied: resolution
                            .canonical
                            .as_deref()
                            .is_some_and(|replacement| replacement != resolution.input && resolution.outcome.is_accepted()),
                        candidates: serde_json::to_value(&resolution.candidates).unwrap_or_default(),
                        complete: resolution.complete,
                        edit_distance: resolution.edit_distance,
                        evidence: resolution.match_kind.map(|value| value.to_string()),
                        method: resolution.method.clone(),
                        mode: report.fuzzy.to_string(),
                        original: resolution.input.clone(),
                        outcome: resolution.outcome.to_string(),
                        policy_revision: report.policy_revision.clone(),
                        replacement: resolution.canonical.clone(),
                        runner_up_margin: resolution.runner_up_margin,
                        score: resolution.score,
                        source_locator: resolution.source_locator.clone(),
                        variant: None,
                        vocabulary: report.vocabulary.to_string(),
                        vocabulary_revision: report.vocabulary_revision.clone(),
                    })
                })
                .collect::<Vec<_>>();
            audit_source(source, before, after, changes)
        })
        .collect::<Vec<_>>();
    write_vocabulary_audit(sources)
}
