//! Bounded supplied files and projections of explicitly validated activities.
use acorn_core::util::{MimeType, StringConversion};
use alloc::collections::{BTreeMap, BTreeSet};
use data_encoding::BASE64;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Maximum accepted files in a session.
pub const FILE_COUNT: usize = 500;
/// Largest accepted individual file, in source bytes.
pub const FILE_LIMIT: usize = 256 * 1024;
/// Total encoded content bytes accepted per session.
pub const TOTAL_LIMIT: usize = 16 * 1024 * 1024;
/// Result of a user-mediated browser selection.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    /// Candidate supplied files.
    pub files: Vec<SuppliedFile>,
    /// Rejections reported before content crossed the browser bridge.
    pub skipped: usize,
}
/// A selected file, with no absolute path or reusable directory authority.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuppliedFile {
    /// Normalized relative path.
    pub path: String,
    /// UTF-8 source text, or a canonical data URL for a supported raster image.
    pub content: String,
}
/// One immutable loaded session; replacement clears stale validation and selection.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Workspace {
    /// Bounded files in path order.
    pub files: Vec<SuppliedFile>,
    /// Portable validation results keyed by file path.
    pub validations: BTreeMap<String, Value>,
    /// Active source file, if any.
    pub selected: Option<String>,
    /// Repository origin for a remote snapshot, if present.
    #[serde(default)]
    pub source: Option<WorkspaceSource>,
    /// Skipped or rejected files from the last selection.
    pub skipped: usize,
}
/// Provenance of a repository snapshot without credentials or absolute file authority.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceSource {
    /// Provider identifier.
    pub kind: String,
    /// API host used for the load.
    pub host: String,
    /// Provider project path.
    pub project: String,
    /// Requested path within the repository.
    pub path: String,
    /// Immutable revision used to read files.
    pub revision: String,
}
impl Workspace {
    /// Validated records only; invalid or unvalidated files never enter the portfolio.
    pub fn activities(&self) -> Vec<(&SuppliedFile, &Value)> {
        self.files
            .iter()
            .filter_map(|file| {
                self.validations
                    .get(&file.path)
                    .filter(|result| result["valid"] == true)
                    .and_then(|result| result.get("value").filter(|value| value.is_object()))
                    .map(|value| (file, value))
            })
            .collect()
    }
    /// Total encoded content bytes.
    pub fn bytes(&self) -> usize {
        self.files.iter().map(|file| file.content.len()).sum()
    }
    /// Content of the selected source file.
    pub fn content(&self) -> String {
        self.files
            .iter()
            .find(|file| Some(&file.path) == self.selected.as_ref())
            .filter(|file| !MimeType::from(file.path.as_str()).is_image())
            .map(|file| file.content.clone())
            .unwrap_or_default()
    }
    /// Accept a selection atomically; total failure preserves the prior session.
    pub fn replace(&self, selection: Selection) -> Self {
        let mut files = selection.files;
        files.sort_by(|left, right| left.path.cmp(&right.path).then(left.content.cmp(&right.content)));
        let (accepted, _, skipped) = files
            .into_iter()
            .fold((BTreeMap::new(), 0_usize, selection.skipped), |(mut accepted, bytes, skipped), file| {
                let size = valid_path(&file.path).then(|| stored_size(&file)).flatten();
                let available = accepted.len() < FILE_COUNT && !accepted.contains_key(&file.path);
                if let Some(size) = size.filter(|size| available && bytes.saturating_add(*size) <= TOTAL_LIMIT) {
                    let bytes = bytes.saturating_add(size);
                    accepted.insert(file.path.clone(), file);
                    (accepted, bytes, skipped)
                } else {
                    (accepted, bytes, skipped.saturating_add(1))
                }
            });
        if accepted.is_empty() {
            Self { skipped, ..self.clone() }
        } else {
            Self {
                selected: accepted
                    .values()
                    .find(|file| !MimeType::from(file.path.as_str()).is_image())
                    .map(|file| file.path.clone()),
                files: accepted.into_values().collect(),
                skipped,
                source: None,
                validations: BTreeMap::new(),
            }
        }
    }
    /// Recheck all persisted paths and limits before restoring browser storage.
    pub fn restored(self) -> Option<Self> {
        let selection = Selection {
            files: self.files.clone(),
            skipped: self.skipped,
        };
        let checked = Self::default().replace(selection);
        let selected_exists = self
            .selected
            .as_ref()
            .is_none_or(|path| checked.files.iter().any(|file| &file.path == path));
        (checked.files == self.files && selected_exists).then_some(self)
    }
}
impl WorkspaceSource {
    /// Construct a source file link pinned to the revision used for loading.
    pub fn link(&self, path: &str) -> Option<String> {
        let host = self.host.as_str();
        let safe_host = !host.is_empty() && host.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'));
        let safe_revision = self.revision.len() >= 40 && self.revision.bytes().all(|byte| byte.is_ascii_hexdigit());
        (safe_host && safe_revision)
            .then(|| encoded_path(&self.project).zip(encoded_path(path)))
            .flatten()
            .and_then(|(project, path)| match self.kind.as_str() {
                | "github" => Some(format!("https://github.com/{project}/blob/{}/{path}", self.revision)),
                | "gitlab" => Some(format!("https://{host}/{project}/-/blob/{}/{path}", self.revision)),
                | _ => None,
            })
    }
}
/// Current status, without inferring one from missing metadata.
pub fn activity_status(value: &Value) -> &str {
    metadata(value, "status").as_str().unwrap_or("Not provided")
}
/// Human-readable activity kind; missing optional fields remain explicit.
pub fn activity_type(value: &Value) -> &str {
    value["@type"].as_str().unwrap_or("Research activity")
}
/// Stable distribution for charts and their text equivalents.
pub fn distribution(records: &[(&SuppliedFile, &Value)], classify: fn(&Value) -> &str) -> BTreeMap<String, usize> {
    records.iter().fold(BTreeMap::new(), |mut counts, (_, value)| {
        let count = counts.entry(classify(value).into()).or_insert(0_usize);
        *count = count.saturating_add(1);
        counts
    })
}
fn encoded_path(path: &str) -> Option<String> {
    path.split('/')
        .map(|part| (!part.is_empty() && !matches!(part, "." | "..") && !part.chars().any(char::is_control)).then(|| part.percent_encode()))
        .collect::<Option<Vec<_>>>()
        .map(|parts| parts.join("/"))
}

/// Unique normalized organization, contributor or identifier values in validated records.
pub fn entities(records: &[(&SuppliedFile, &Value)], field: &str) -> BTreeSet<String> {
    records
        .iter()
        .flat_map(|(_, value)| many(metadata(value, field)))
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    let identifiers = many(&value["identifiers"]);
                    identifiers
                        .iter()
                        .find_map(|id| id["value"].as_str())
                        .or_else(|| value["name"].as_str())
                        .map(str::to_owned)
                        .unwrap_or_else(|| value.to_string())
                })
                .trim()
                .to_lowercase()
        })
        .filter(|value| !value.is_empty())
        .collect()
}
/// Persistent identifier counts from explicit schema PID fields and typed identities.
pub fn identifiers(records: &[(&SuppliedFile, &Value)]) -> BTreeSet<String> {
    let canonical = |text: &str| {
        text.trim()
            .trim_start_matches("https://doi.org/")
            .trim_start_matches("http://doi.org/")
            .trim_start_matches("https://ror.org/")
            .to_lowercase()
    };
    records
        .iter()
        .flat_map(|(_, value)| {
            ["doi", "handle", "raid", "swhid", "ror"]
                .into_iter()
                .flat_map(move |field| many(metadata(value, field)))
                .filter_map(Value::as_str)
        })
        .chain(records.iter().flat_map(|(_, value)| {
            ["organizations", "contributors"]
                .into_iter()
                .flat_map(move |field| many(metadata(value, field)))
                .flat_map(|entity| many(&entity["identifiers"]))
                .filter_map(|identifier| identifier["value"].as_str())
        }))
        .map(canonical)
        .filter(|value| !value.is_empty())
        .collect()
}
/// Treat the schema's single-or-many representation consistently.
pub fn many(value: &Value) -> Vec<&Value> {
    match value {
        | Value::Array(values) => values.iter().collect(),
        | Value::Null => Vec::new(),
        | other => vec![other],
    }
}
fn metadata<'a>(value: &'a Value, field: &str) -> &'a Value {
    value.get("meta").and_then(|meta| meta.get(field)).unwrap_or(&Value::Null)
}
fn stored_size(file: &SuppliedFile) -> Option<usize> {
    let mime = MimeType::from(file.path.as_str());
    if mime.is_raster_image() {
        let prefix = format!("data:{mime};base64,");
        file.content
            .strip_prefix(&prefix)
            .and_then(|payload| BASE64.decode(payload.as_bytes()).ok().map(|decoded| (payload, decoded)))
            .and_then(|(payload, decoded)| {
                (mime.signature(&decoded) && decoded.len() <= FILE_LIMIT && BASE64.encode(&decoded) == payload).then_some(file.content.len())
            })
    } else {
        (file.content.len() <= FILE_LIMIT).then_some(file.content.len())
    }
}
/// Reject traversal, absolute paths, unsupported extensions and paths deeper than 20 segments.
pub fn valid_path(path: &str) -> bool {
    let parts = path.split('/').collect::<Vec<_>>();
    !path.contains(['\\', ':'])
        && !path.chars().any(char::is_control)
        && parts.len() <= 20
        && parts.iter().all(|part| !part.is_empty() && !matches!(*part, "." | ".."))
        && path.rsplit_once('.').is_some()
        && matches!(
            MimeType::from(path),
            MimeType::Json | MimeType::Yaml | MimeType::Markdown | MimeType::Text | MimeType::Svg | MimeType::Png | MimeType::Jpeg | MimeType::Webp
        )
}
