//! Application configuration integration for executable hooks.
use crate::analyzer::{Check, CheckCategory};
use crate::caution;
use crate::exec::result::RhaiExt;
use crate::exec::runtime::{RhaiRuntime, Runtime};
use crate::io::config::application::{Entry, Hook};
use crate::io::workflow::WorkflowAction;
use crate::io::{read_file_snapshot, ApiResult, Bytes};
use crate::util::constants::app::APPLICATION;
use acorn_core::prelude::{String, Vec};
use acorn_core::util::constants::app::MAX_EXEC_BYTES;
use acorn_schema::validation::Validate;
use color_eyre::eyre::{eyre, Report, Result};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Origin authorized to request configured script execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionOrigin {
    /// An explicit invocation by the local command-line user.
    Local,
    /// Any TUI, transport, automation, or repository-snapshot caller.
    NonLocal,
}
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
    ast: rhai::AST,
}
impl Entry {
    /// Resolve this entry relative to the directory containing application configuration.
    pub fn resolve(&self, project_root: &Path, origin: ExecutionOrigin) -> Result<Resolved, Report> {
        self.validate().map_err(|why| eyre!("{why}")).and_then(|()| match origin {
            | ExecutionOrigin::NonLocal => Err(eyre!("configured Rhai scripts are unavailable to non-local callers")),
            | ExecutionOrigin::Local => {
                let root = project_root.join(format!(".{APPLICATION}"));
                let (candidate, is_absolute) = match self.absolute {
                    | true => (PathBuf::from(&self.path), true),
                    | false => (root.join(&self.path), false),
                };
                let candidates = match candidate.extension().is_some() {
                    | true => vec![candidate.clone()],
                    | false => vec![candidate.clone(), candidate.with_extension("rhai")],
                };
                self.select(&candidates).and_then(|selected| {
                    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.clone());
                    let canonical_location = selected.canonicalize().unwrap_or_else(|_| selected.clone());
                    let is_valid_location = is_absolute || canonical_location.starts_with(canonical_root);
                    match is_valid_location {
                        | true => read_file_snapshot(&selected, MAX_EXEC_BYTES)
                            .map_err(|why| eyre!("Failed to read script `{}` — {why}", selected.display()))
                            .and_then(|(canonical, source, metadata)| {
                                validate_permissions(&selected, &metadata, is_absolute).map(|()| (canonical, source))
                            }),
                        | false => Err(eyre!("script `{}` escapes project `.acorn/` directory", self.path)),
                    }
                    .and_then(|(canonical, source)| {
                        let digest = Bytes::from(&source).digest();
                        RhaiRuntime::compile_source(&canonical, &source).map(|ast| (canonical, digest, ast))
                    })
                    .map(|(canonical, digest, ast)| {
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
                            ast,
                        }
                    })
                })
            }
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
    /// Prepare one explicitly supplied local script using the absolute-script trust policy.
    pub fn direct(path: &Path) -> Result<Self, Report> {
        read_file_snapshot(path, MAX_EXEC_BYTES)
            .map_err(|why| eyre!("Failed to read script `{}` — {why}", path.display()))
            .and_then(|(canonical, source, metadata)| validate_permissions(path, &metadata, true).map(|()| (canonical, source)))
            .and_then(|(canonical, source)| {
                let digest = Bytes::from(&source).digest();
                RhaiRuntime::compile_source(&canonical, &source).map(|ast| {
                    caution!(
                        "Explicit script `{}` → canonical `{}` (digest {digest}) — non-portable trust exception",
                        path.display(),
                        canonical.display()
                    );
                    Self {
                        entry: Entry {
                            absolute: true,
                            hooks: vec![Hook::Check],
                            path: path.display().to_string(),
                        },
                        canonical,
                        digest,
                        is_absolute: true,
                        ast,
                    }
                })
            })
    }
    /// Evaluate this extension for one workflow document and convert its findings to checks.
    pub fn check<T>(&self, source: &str, data: &T, document_index: usize, hook: WorkflowAction) -> ApiResult<Vec<Check>>
    where
        T: serde::Serialize,
    {
        let Resolved {
            digest,
            entry: Entry { path, .. },
            ..
        } = self;
        let canonical = &self.canonical.display();
        if self.is_absolute {
            caution!("Executing absolute script `{path}` (canonical `{canonical}`, digest {digest}) for {hook} on {source}");
        }
        let mut context = rhai::Map::new();
        context.insert("hook".into(), rhai::Dynamic::from(hook.to_string()));
        context.insert("path".into(), rhai::Dynamic::from(source.to_string()));
        let proposed = serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string());
        context.insert("input".into(), rhai::Dynamic::from(proposed.clone()));
        context.insert("proposed".into(), rhai::Dynamic::from(proposed));
        context.insert("index".into(), rhai::Dynamic::from(i64::try_from(document_index).unwrap_or(i64::MAX)));
        RhaiRuntime::new()
            .check_ast(&self.ast, context)
            .and_then(|dynamic| dynamic.findings().map_err(|why| eyre!("{why}")))
            .map(|findings| {
                findings
                    .into_iter()
                    .enumerate()
                    .map(|(finding_index, finding)| {
                        let index = document_index.saturating_mul(100).saturating_add(finding_index);
                        let mut check = Check::from(finding);
                        check.category = match hook {
                            | WorkflowAction::Validate => CheckCategory::Schema,
                            | _ => CheckCategory::Quality,
                        };
                        check.with_index(index).with_uri(Some(source.to_string()))
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
