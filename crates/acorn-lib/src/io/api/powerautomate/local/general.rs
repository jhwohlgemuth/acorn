//! Standalone JSON logbook storage for report-wide submissions.
use super::MAX_INDEX_BYTES;
use crate::io::config::application::GeneralLogbookConfig;
use crate::io::{read_file_snapshot, ApiResult};
use acorn_core::validation::ValidationError;
use acorn_host::fs::SafePath;
use acorn_schema::research_activity::{Logbook, LogbookEntry, ReportingWindow};
use acorn_schema::validation::{Validate, ValidationReport};
use color_eyre::eyre::eyre;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const EMPTY_LOGBOOK: &str = "{\"entries\":[]}\n";
struct GeneralLogbook<'a>(&'a Logbook);
impl Validate for GeneralLogbook<'_> {
    fn validate(&self) -> Result<(), ValidationReport> {
        if self
            .0
            .entries
            .iter()
            .any(|entry| entry.project_id != "general" || !entry.global || entry.archived || entry.validate().is_err())
        {
            Err(ValidationReport::from_error(
                "entries",
                ValidationError::new("general_logbook").with_message("General logbook contains an invalid or non-general entry"),
            ))
        } else if self.0.entries.is_empty() {
            Ok(())
        } else {
            self.0.validate()
        }
    }
}
/// Resolve the configured path without creating or modifying the logbook.
pub fn destination(config: &GeneralLogbookConfig, config_path: &Path) -> ApiResult<PathBuf> {
    config
        .validate()
        .map_err(|why| eyre!("Invalid general logbook configuration — {why}"))
        .and_then(|()| {
            config_path
                .parent()
                .ok_or_else(|| eyre!("Configuration has no parent directory"))
                .and_then(|parent| {
                    SafePath::new(&config.path)
                        .and_then(|safe| safe.materialize_under(parent))
                        .map_err(|why| eyre!("Unsafe general logbook path — {why}"))
                })
        })
        .and_then(|path| match path.symlink_metadata() {
            | Ok(metadata) if metadata.is_symlink() || !metadata.is_file() => Err(eyre!("General logbook must be a regular file")),
            | Ok(_) | Err(_) => Ok(path),
        })
}
pub(super) fn read_content(path: &Path) -> ApiResult<String> {
    match path.symlink_metadata() {
        | Ok(metadata) if metadata.is_symlink() || !metadata.is_file() => Err(eyre!("General logbook must be a regular file")),
        | Ok(_) => read_file_snapshot(path, MAX_INDEX_BYTES).map(|(_, text, _)| text),
        | Err(why) if why.kind() == ErrorKind::NotFound => Ok(EMPTY_LOGBOOK.to_string()),
        | Err(why) => Err(eyre!("Cannot inspect general logbook — {why}")),
    }
}
pub(super) fn propose(content: &str, entry: &LogbookEntry, replaces: Option<&LogbookEntry>) -> ApiResult<String> {
    serde_json::from_str::<Logbook>(content)
        .map_err(|why| eyre!("Invalid general logbook — {why}"))
        .and_then(|logbook| {
            GeneralLogbook(&logbook)
                .validate()
                .map_err(|why| eyre!("Invalid general logbook — {why}"))
                .map(|()| logbook)
        })
        .and_then(|logbook| {
            let existing = logbook.entries.iter().find(|candidate| candidate.identifier == entry.identifier);
            match (existing, replaces) {
                | (Some(previous), Some(expected))
                    if serde_json::to_value(previous).ok() == serde_json::to_value(expected).ok() && !previous.archived && !entry.archived =>
                {
                    Ok((
                        logbook
                            .entries
                            .into_iter()
                            .map(|candidate| {
                                if candidate.identifier == entry.identifier {
                                    entry.clone()
                                } else {
                                    candidate
                                }
                            })
                            .collect(),
                        true,
                    ))
                }
                | (None, None) => Ok((logbook.entries.into_iter().chain(core::iter::once(entry.clone())).collect(), true)),
                | (Some(previous), None) if serde_json::to_value(previous).ok() == serde_json::to_value(entry).ok() => Ok((logbook.entries, false)),
                | _ => Err(eyre!("General logbook correction or replay conflicts with existing content")),
            }
            .and_then(|(entries, changed)| {
                let updated = Logbook { entries, ..logbook };
                GeneralLogbook(&updated)
                    .validate()
                    .map_err(|why| eyre!("Invalid general logbook — {why}"))
                    .and_then(|()| match changed {
                        | true => serde_json::to_string_pretty(&updated)
                            .map_err(|why| eyre!("Cannot encode general logbook — {why}"))
                            .map(|text| format!("{text}\n")),
                        | false => Ok(content.to_string()),
                    })
            })
        })
}
/// Read report-wide entries inside the requested reporting window.
pub fn entries_within(path: &Path, window: &ReportingWindow) -> ApiResult<Vec<LogbookEntry>> {
    read_content(path)
        .and_then(|content| serde_json::from_str::<Logbook>(&content).map_err(|why| eyre!("Invalid general logbook — {why}")))
        .and_then(|logbook| {
            GeneralLogbook(&logbook)
                .validate()
                .map_err(|why| eyre!("Invalid general logbook — {why}"))
                .map(|()| logbook)
        })
        .and_then(|logbook| logbook.entries_within(window).map_err(|why| eyre!(why)))
}
