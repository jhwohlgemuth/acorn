//! Bounded, in-memory supplied files and projections of explicitly validated activities.
use alloc::collections::{BTreeMap, BTreeSet};
use serde::Deserialize;
use serde_json::Value;

/// Largest accepted individual file, in UTF-8 bytes.
pub const FILE_LIMIT: usize = 256 * 1024;
/// Maximum accepted files in a session.
pub const FILE_COUNT: usize = 500;
/// Total text bytes accepted per session.
pub const TOTAL_LIMIT: usize = 16 * 1024 * 1024;
/// A selected file, with no absolute path or reusable directory authority.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SuppliedFile {
    /// Normalized relative path.
    pub path: String,
    /// UTF-8 source text.
    pub content: String,
}
/// Result of a user-mediated browser selection.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    /// Candidate supplied files.
    pub files: Vec<SuppliedFile>,
    /// Rejections reported before content crossed the browser bridge.
    pub skipped: usize,
}
/// One immutable loaded session; replacement clears stale validation and selection.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Workspace {
    /// Bounded files in path order.
    pub files: Vec<SuppliedFile>,
    /// Portable validation results keyed by file path.
    pub validations: BTreeMap<String, Value>,
    /// Active source file, if any.
    pub selected: Option<String>,
    /// Skipped or rejected files from the last selection.
    pub skipped: usize,
}
impl Workspace {
    /// Accept a selection atomically; total failure preserves the prior session.
    pub fn replace(&self, selection: Selection) -> Self {
        let mut files = selection.files;
        files.sort_by(|left, right| left.path.cmp(&right.path).then(left.content.cmp(&right.content)));
        let (accepted, _, skipped) = files
            .into_iter()
            .fold((BTreeMap::new(), 0_usize, selection.skipped), |(mut accepted, bytes, skipped), file| {
                if valid_path(&file.path)
                    && file.content.len() <= FILE_LIMIT
                    && bytes.saturating_add(file.content.len()) <= TOTAL_LIMIT
                    && accepted.len() < FILE_COUNT
                    && !accepted.contains_key(&file.path)
                {
                    let bytes = bytes.saturating_add(file.content.len());
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
                selected: accepted.keys().next().cloned(),
                files: accepted.into_values().collect(),
                skipped,
                validations: BTreeMap::new(),
            }
        }
    }
    /// Total loaded UTF-8 bytes.
    pub fn bytes(&self) -> usize {
        self.files.iter().map(|file| file.content.len()).sum()
    }
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
    /// Content of the selected source file.
    pub fn content(&self) -> String {
        self.files
            .iter()
            .find(|file| Some(&file.path) == self.selected.as_ref())
            .map(|file| file.content.clone())
            .unwrap_or_default()
    }
}
/// Reject traversal, absolute paths, unsupported extensions and paths deeper than 20 segments.
pub fn valid_path(path: &str) -> bool {
    let parts = path.split('/').collect::<Vec<_>>();
    !path.contains(['\\', ':'])
        && !path.chars().any(char::is_control)
        && parts.len() <= 20
        && parts.iter().all(|part| !part.is_empty() && !matches!(*part, "." | ".."))
        && path
            .rsplit_once('.')
            .is_some_and(|(_, extension)| ["json", "yaml", "yml", "md", "txt"].contains(&extension.to_ascii_lowercase().as_str()))
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
/// Human-readable activity kind; missing optional fields remain explicit.
pub fn activity_type(value: &Value) -> &str {
    value["@type"].as_str().unwrap_or("Research activity")
}
/// Current status, without inferring one from missing metadata.
pub fn activity_status(value: &Value) -> &str {
    metadata(value, "status").as_str().unwrap_or("Not provided")
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
/// Stable distribution for charts and their text equivalents.
pub fn distribution(records: &[(&SuppliedFile, &Value)], classify: fn(&Value) -> &str) -> BTreeMap<String, usize> {
    records.iter().fold(BTreeMap::new(), |mut counts, (_, value)| {
        let count = counts.entry(classify(value).into()).or_insert(0_usize);
        *count = count.saturating_add(1);
        counts
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn file(path: &str, content: &str) -> SuppliedFile {
        SuppliedFile {
            path: path.into(),
            content: content.into(),
        }
    }
    #[test]
    fn test_failed_selection_preserves_content_and_validation() {
        let original = Workspace::default().replace(Selection {
            files: vec![file("ok.json", "{}")],
            skipped: 0,
        });
        let next = original.replace(Selection {
            files: vec![file("../bad.json", "bad")],
            skipped: 2,
        });
        assert_eq!(next.files, original.files);
        assert_eq!(next.selected, original.selected);
        assert_eq!(next.skipped, 3);
    }
    #[test]
    fn test_limits_sorting_duplicates_and_traversal() {
        let workspace = Workspace::default().replace(Selection {
            files: vec![
                file("b/x.json", "{}"),
                file("a.txt", "text"),
                file("a.txt", "text"),
                file("/bad.json", ""),
                file("huge.txt", &"é".repeat(FILE_LIMIT)),
            ],
            skipped: 0,
        });
        assert_eq!(
            workspace.files.iter().map(|file| file.path.as_str()).collect::<Vec<_>>(),
            ["a.txt", "b/x.json"]
        );
        assert_eq!(workspace.skipped, 3);
        assert_eq!(workspace.bytes(), 6);
        for path in ["../x.json", "a/../x.json", "a//b.json", "C:/x.json", "a\\b.json", "x.exe", "\n.json"] {
            assert!(!valid_path(path), "{path}");
        }
        assert!(!valid_path(&format!("{}x.json", "a/".repeat(20))));
        assert!(valid_path("a/é.JSON"));
        let many = Workspace::default().replace(Selection {
            files: (0..510).map(|i| file(&format!("{i:03}.txt"), "x")).collect(),
            skipped: 0,
        });
        assert_eq!(many.files.len(), 500);
        assert_eq!(many.skipped, 10);
        let large = Workspace::default().replace(Selection {
            files: (0..70).map(|i| file(&format!("{i:03}.txt"), &"x".repeat(FILE_LIMIT))).collect(),
            skipped: 0,
        });
        assert_eq!(large.bytes(), TOTAL_LIMIT);
        assert_eq!(large.skipped, 6);
    }
    #[test]
    fn test_portfolio_uses_only_validated_records_and_deduplicates_entities() {
        let mut workspace = Workspace::default().replace(Selection {
            files: vec![file("a.json", "{}"), file("b.json", "{}"), file("invalid.json", "{}")],
            skipped: 0,
        });
        let value = json!({"title":"A","@type":"Project","meta":{"status":"active","doi":"https://doi.org/10.1/x","organizations":[{"name":"Example","identifiers":[{"value":"https://ror.org/example"}]}]}});
        for name in ["a.json", "b.json"] {
            workspace.validations.insert(name.into(), json!({"valid":true,"value":value}));
        }
        workspace.validations.insert("invalid.json".into(), json!({"valid":false,"value":value}));
        let records = workspace.activities();
        assert_eq!(records.len(), 2);
        assert_eq!(entities(&records, "organizations").len(), 1);
        assert_eq!(identifiers(&records).len(), 2);
        assert_eq!(distribution(&records, activity_type).get("Project"), Some(&2));
        let replaced = workspace.replace(Selection {
            files: vec![file("new.json", "{}")],
            skipped: 0,
        });
        assert!(replaced.activities().is_empty());
        assert_eq!(replaced.selected.as_deref(), Some("new.json"));
    }
}
