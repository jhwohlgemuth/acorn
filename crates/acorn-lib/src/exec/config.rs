//! Application configuration integration for executable hooks.
use crate::analyzer::Check;
use crate::caution;
use crate::exec::result::RhaiExt;
use crate::exec::runtime::{self, RhaiRuntime, Runtime};
use crate::io::config::application::{Entry, Hook};
use crate::io::workflow::WorkflowAction;
use crate::io::{read_file, ApiResult};
use crate::util::constants::app::APPLICATION;
use acorn_core::prelude::{String, Vec};
use acorn_core::util::constants::app::MAX_EXEC_BYTES;
use acorn_schema::validation::Validate;
use color_eyre::eyre::{eyre, Report, Result};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Resolved script with canonical path and content digest.
#[derive(Clone, Debug)]
pub struct Resolved {
    /// Original entry.
    pub entry: Entry,
    /// Canonical filesystem path.
    pub canonical: PathBuf,
    /// Content digest of bounded source bytes.
    pub digest: String,
    /// Whether this script was resolved via absolute trust exception.
    pub is_absolute: bool,
}
impl Entry {
    /// Resolve this entry relative to the directory containing application configuration.
    pub fn resolve(&self, project_root: &Path) -> Result<Resolved, Report> {
        self.validate().map_err(|why| eyre!("{why}")).and_then(|()| {
            let root = project_root.join(format!(".{APPLICATION}"));
            let (candidate, is_absolute) = match self.absolute {
                | true => (PathBuf::from(&self.path), true),
                | false => {
                    let path = self
                        .path
                        .strip_prefix(".acorn/")
                        .or_else(|| self.path.strip_prefix(".acorn\\"))
                        .unwrap_or(&self.path);
                    (root.join(path), false)
                }
            };
            let candidates = match candidate.extension().is_some() {
                | true => vec![candidate.clone()],
                | false => vec![candidate.clone(), candidate.with_extension("rhai")],
            };
            self.select(&candidates).and_then(|selected| {
                selected
                    .symlink_metadata()
                    .map_err(|why| eyre!("script `{}` metadata — {why}", selected.display()))
                    .and_then(|metadata| match (metadata.is_symlink(), metadata.is_file()) {
                        | (true, _) => Err(eyre!("script `{}` must not be a symbolic link", selected.display())),
                        | (_, false) => Err(eyre!("script `{}` is not a regular file", selected.display())),
                        | _ => Ok((selected, metadata)),
                    })
                    .and_then(|(selected, metadata)| {
                        let escapes = !is_absolute
                            && !selected
                                .canonicalize()
                                .unwrap_or_else(|_| selected.clone())
                                .starts_with(root.canonicalize().unwrap_or_else(|_| root.clone()));
                        match escapes {
                            | true => Err(eyre!("script `{}` escapes project `.acorn/` directory", self.path)),
                            | false => Ok((selected, metadata)),
                        }
                    })
                    .and_then(|(selected, metadata)| validate_permissions(&selected, &metadata, is_absolute).map(|()| selected))
                    .and_then(|selected| {
                        std::fs::read(&selected)
                            .map_err(|why| eyre!("Failed to read script `{}` — {why}", selected.display()))
                            .and_then(|bytes| match bytes.len() > MAX_EXEC_BYTES {
                                | true => Err(eyre!("script `{}` exceeds {} bytes", selected.display(), MAX_EXEC_BYTES)),
                                | false => Ok((selected, bytes)),
                            })
                    })
                    .map(|(selected, bytes)| {
                        let digest = runtime::digest_bytes(&bytes);
                        let canonical = selected.canonicalize().unwrap_or(selected);
                        match is_absolute {
                            | true => {
                                caution!(
                                    "Absolute script `{}` → canonical `{}` (digest {digest}) — non-portable trust exception",
                                    self.path,
                                    canonical.display()
                                );
                            }
                            | false => {
                                debug!("Resolved script {} → {} digest {}", self.path, canonical.display(), digest);
                            }
                        }
                        Resolved {
                            entry: self.clone(),
                            canonical,
                            digest,
                            is_absolute,
                        }
                    })
            })
        })
    }
    fn select(&self, candidates: &[PathBuf]) -> Result<PathBuf, Report> {
        let existing: Vec<&PathBuf> = candidates.iter().filter(|path| path.is_file()).collect();
        match existing.as_slice() {
            | [] => Err(eyre!(
                "script `{}` not found as regular file (checked: {})",
                self.path,
                candidates.iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join(", ")
            )),
            | [selected] => Ok((*selected).clone()),
            | [first, second, ..] => Err(eyre!(
                "ambiguous script `{}` — both `{}` and `{}` exist",
                self.path,
                first.display(),
                second.display()
            )),
        }
    }
}
impl PartialEq<WorkflowAction> for Hook {
    fn eq(&self, other: &WorkflowAction) -> bool {
        match other {
            | WorkflowAction::Check => self.is_check(),
            | WorkflowAction::Enrich => self.is_enrich(),
            | WorkflowAction::Format => self.is_format(),
            | WorkflowAction::Link => self.is_link(),
            | WorkflowAction::Validate => self.is_validate(),
        }
    }
}
impl Resolved {
    /// Evaluate this extension for one workflow document and convert its findings to checks.
    pub fn check<T>(&self, source: &str, data: &T, document_index: usize, hook: WorkflowAction) -> ApiResult<Vec<Check>>
    where
        T: serde::Serialize,
    {
        read_file(&self.canonical)
            .and_then(|content| match content.len() > MAX_EXEC_BYTES {
                | true => Err(eyre!("script exceeds limit")),
                | false => Ok(content),
            })
            .and_then(|script_source| {
                let Resolved {
                    digest,
                    entry: Entry { path, .. },
                    ..
                } = self;
                let canonical = &self.canonical.display();
                caution!("Executing script `{path}` (canonical `{canonical}`, digest {digest}) for {hook} on {source}");
                let mut context = rhai::Map::new();
                context.insert("hook".into(), rhai::Dynamic::from(hook.to_string()));
                context.insert("path".into(), rhai::Dynamic::from(source.to_string()));
                let proposed = serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string());
                context.insert("proposed".into(), rhai::Dynamic::from(proposed));
                context.insert("index".into(), rhai::Dynamic::from(i64::try_from(document_index).unwrap_or(i64::MAX)));
                RhaiRuntime::new().check(&script_source, context)
            })
            .and_then(|dynamic| dynamic.findings().map_err(|why| eyre!("{why}")))
            .map(|findings| {
                findings
                    .into_iter()
                    .enumerate()
                    .map(|(finding_index, finding)| {
                        let index = document_index.saturating_mul(100).saturating_add(finding_index);
                        Check::from(finding).with_index(index).with_uri(Some(source.to_string()))
                    })
                    .collect()
            })
    }
}
#[cfg(unix)]
fn validate_permissions(selected: &Path, metadata: &std::fs::Metadata, is_absolute: bool) -> Result<(), Report> {
    let mode = metadata.permissions().mode();
    match (mode & 0o022 != 0, is_absolute) {
        | (true, true) => Err(eyre!(
            "absolute script `{}` is group/world-writable (mode {:o})",
            selected.display(),
            mode & 0o777
        )),
        | (true, false) => {
            caution!("Script `{}` is group/world-writable", selected.display());
            Ok(())
        }
        | (false, _) => Ok(()),
    }
}
#[cfg(not(unix))]
fn validate_permissions(_selected: &Path, _metadata: &std::fs::Metadata, _is_absolute: bool) -> Result<(), Report> {
    Ok(())
}
