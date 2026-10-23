//! Bounded, deterministic local inputs and proposed artifact destinations
use crate::error::ApiResult;
use acorn_core::prelude::{String, Vec};
use acorn_host::fs::SafePath;
use acorn_schema::hardware::Memory;
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Caller-supplied text or one local path form.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind", content = "value")]
pub enum InputReference {
    /// Verbatim text input.
    Text(String),
    /// One local file.
    File(PathBuf),
    /// A recursively discovered local directory.
    Directory(PathBuf),
}
/// Destination boundary used to propose, then revalidate, artifact writes.
#[derive(Clone, Debug)]
pub struct ArtifactPolicy {
    root: PathBuf,
    allow_write: bool,
    allow_overwrite: bool,
}
/// Nonfatal directory entry failure; successful siblings remain available.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct InputFailure {
    /// Entry path encountered while walking the requested directory.
    pub path: PathBuf,
    /// Stable failure category, without raw filesystem error details.
    pub reason: String,
}
/// Hard bounds for one input resolution request.
#[derive(Clone, Debug, PartialEq)]
pub struct InputLimits {
    /// Maximum number of files accepted in one batch.
    pub max_files: usize,
    /// Maximum combined file or text size.
    pub max_total: Memory,
    /// Maximum nested directory depth below the selected directory.
    pub max_depth: usize,
}
/// Resolved input references in deterministic path order.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedInput {
    /// Verbatim text for a text reference.
    pub text: Option<String>,
    /// Confirmed files for a file or directory reference.
    pub files: Vec<ResolvedInputFile>,
    /// Per-entry failures for a directory reference.
    pub failures: Vec<InputFailure>,
}
/// One confirmed local file and its observed size.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedInputFile {
    /// Canonical path inside the allowed root.
    pub path: PathBuf,
    /// Size observed during input resolution.
    pub size: Memory,
}
/// Caller path boundary used before handing references to a native leaf.
#[derive(Clone, Debug)]
pub struct RoutedInputPolicy {
    root: PathBuf,
    limits: InputLimits,
}
impl ArtifactPolicy {
    /// Establish an existing artifact root and explicit write policy.
    pub fn new(root: &Path, allow_write: bool, allow_overwrite: bool) -> ApiResult<Self> {
        root.canonicalize()
            .map_err(|why| eyre!("Invalid artifact root — {why}"))
            .and_then(|root| {
                if root.is_dir() {
                    Ok(Self {
                        root,
                        allow_write,
                        allow_overwrite,
                    })
                } else {
                    Err(eyre!("Artifact root must be a directory"))
                }
            })
    }
    /// Validate an exact relative destination without writing or creating directories.
    /// Call this again immediately before a native leaf performs the write.
    pub fn destination(&self, relative: &Path, sources: &[PathBuf]) -> ApiResult<PathBuf> {
        match (self.allow_write, SafePath::new(relative)) {
            | (false, _) => Err(eyre!("Artifact write policy is not enabled")),
            | (true, Err(_)) => Err(eyre!("Artifact destination contains traversal or unsupported components")),
            | (true, Ok(relative)) => {
                let destination = self.root.join(relative.as_path());
                destination
                    .parent()
                    .ok_or_else(|| eyre!("Artifact destination has no parent"))
                    .and_then(|parent| {
                        core::iter::successors(Some(parent), |ancestor| ancestor.parent())
                            .find(|ancestor| ancestor.exists())
                            .ok_or_else(|| eyre!("Artifact destination has no existing ancestor"))
                    })
                    .and_then(|ancestor| {
                        ancestor
                            .canonicalize()
                            .map_err(|why| eyre!("Cannot resolve artifact destination parent — {why}"))
                    })
                    .and_then(|canonical_ancestor| {
                        if !canonical_ancestor.starts_with(&self.root) {
                            Err(eyre!("Artifact destination escapes the permitted root"))
                        } else if destination.exists() {
                            destination
                                .canonicalize()
                                .map_err(|why| eyre!("Cannot resolve artifact destination — {why}"))
                                .and_then(|canonical| {
                                    if canonical.starts_with(&self.root) && self.allow_overwrite {
                                        Ok(())
                                    } else {
                                        Err(eyre!("Artifact destination collides with an existing path"))
                                    }
                                })
                        } else {
                            Ok(())
                        }
                    })
                    .and_then(|()| {
                        if sources.iter().any(|source| source == &destination) && !self.allow_overwrite {
                            Err(eyre!("Artifact destination collides with an input"))
                        } else {
                            Ok(destination)
                        }
                    })
            }
        }
    }
}
impl Default for InputLimits {
    fn default() -> Self {
        Self {
            max_files: 256,
            max_total: Memory::mb(32),
            max_depth: 8,
        }
    }
}
impl From<PathBuf> for InputReference {
    fn from(path: PathBuf) -> Self {
        if path.is_dir() {
            Self::Directory(path)
        } else {
            Self::File(path)
        }
    }
}
impl RoutedInputPolicy {
    /// Canonicalize an existing root and install resource limits.
    pub fn new(root: &Path, limits: InputLimits) -> ApiResult<Self> {
        if limits.max_files == 0 || limits.max_total.checked_bytes().is_none_or(|bytes| bytes == 0) {
            Err(eyre!("Input limits must allow at least one file and one byte"))
        } else {
            root.canonicalize().map_err(|why| eyre!("Invalid input root — {why}")).and_then(|root| {
                if root.is_dir() {
                    Ok(Self { root, limits })
                } else {
                    Err(eyre!("Input root must be a directory"))
                }
            })
        }
    }
    /// Resolve a caller reference without reading file contents or performing writes.
    pub fn resolve(&self, reference: &InputReference) -> ApiResult<ResolvedInput> {
        match reference {
            | InputReference::Text(text) => {
                if text.is_empty()
                    || !self
                        .limits
                        .max_total
                        .can_contain(u64::try_from(text.len()).unwrap_or(u64::MAX))
                        .unwrap_or(false)
                {
                    Err(eyre!("Text input is empty or exceeds the byte limit"))
                } else {
                    Ok(ResolvedInput {
                        text: Some(text.clone()),
                        files: Vec::new(),
                        failures: Vec::new(),
                    })
                }
            }
            | InputReference::File(path) => self.resolve_path(path).and_then(|path| {
                fs::metadata(&path)
                    .map_err(|why| eyre!("Cannot inspect input file — {why}"))
                    .and_then(|metadata| {
                        if metadata.is_file() && self.limits.max_total.can_contain(metadata.len()) == Some(true) {
                            Ok(ResolvedInput {
                                text: None,
                                files: vec![ResolvedInputFile {
                                    path,
                                    size: Memory::from(metadata.len()),
                                }],
                                failures: Vec::new(),
                            })
                        } else {
                            Err(eyre!("Input file is unsupported or exceeds the byte limit"))
                        }
                    })
            }),
            | InputReference::Directory(path) => self.resolve_path(path).and_then(|path| {
                if path.is_dir() {
                    let mut resolved = ResolvedInput {
                        text: None,
                        files: Vec::new(),
                        failures: Vec::new(),
                    };
                    let mut bytes = 0_u64;
                    self.walk(&path, 0, &mut bytes, &mut resolved);
                    resolved.files.sort_by(|left, right| left.path.cmp(&right.path));
                    resolved
                        .failures
                        .sort_by(|left, right| left.path.cmp(&right.path).then(left.reason.cmp(&right.reason)));
                    Ok(resolved)
                } else {
                    Err(eyre!("Directory input must be a directory"))
                }
            }),
        }
    }
    fn resolve_path(&self, requested: &Path) -> ApiResult<PathBuf> {
        if requested.components().any(|component| matches!(component, Component::ParentDir)) {
            Err(eyre!("Input path contains traversal"))
        } else {
            let candidate = if requested == Path::new(".") {
                Ok(self.root.clone())
            } else if requested.is_absolute() {
                Ok(requested.to_path_buf())
            } else if let Ok(safe) = SafePath::new(requested) {
                Ok(self.root.join(safe.as_path()))
            } else {
                Err(eyre!("Input path contains traversal or unsupported components"))
            };
            candidate
                .and_then(|candidate| candidate.canonicalize().map_err(|why| eyre!("Cannot resolve input path — {why}")))
                .and_then(|path| {
                    if path.starts_with(&self.root) {
                        Ok(path)
                    } else {
                        Err(eyre!("Input path escapes the permitted root"))
                    }
                })
        }
    }
    fn walk(&self, directory: &Path, depth: usize, total_bytes: &mut u64, output: &mut ResolvedInput) {
        match fs::read_dir(directory) {
            | Err(_) => {
                output.failures.push(InputFailure {
                    path: directory.to_path_buf(),
                    reason: "unreadable-directory".to_string(),
                });
            }
            | Ok(read_dir) => {
                let mut entries = Vec::new();
                for entry in read_dir {
                    match entry {
                        | Ok(entry) => entries.push(entry.path()),
                        | Err(_) => output.failures.push(InputFailure {
                            path: directory.to_path_buf(),
                            reason: "unreadable-entry".to_string(),
                        }),
                    }
                }
                entries.sort();
                for path in entries {
                    let metadata = match fs::symlink_metadata(&path) {
                        | Ok(metadata) => metadata,
                        | Err(_) => {
                            output.failures.push(InputFailure {
                                path,
                                reason: "unreadable-entry".to_string(),
                            });
                            continue;
                        }
                    };
                    if metadata.file_type().is_symlink() {
                        output.failures.push(InputFailure {
                            path,
                            reason: "symlink".to_string(),
                        });
                    } else if metadata.is_dir() {
                        if depth >= self.limits.max_depth {
                            output.failures.push(InputFailure {
                                path,
                                reason: "depth-limit".to_string(),
                            });
                        } else {
                            self.walk(&path, depth.saturating_add(1), total_bytes, output);
                        }
                    } else if metadata.is_file() {
                        if output.files.len() >= self.limits.max_files {
                            output.failures.push(InputFailure {
                                path,
                                reason: "file-limit".to_string(),
                            });
                        } else if self.limits.max_total.can_contain(total_bytes.saturating_add(metadata.len())) != Some(true) {
                            output.failures.push(InputFailure {
                                path,
                                reason: "byte-limit".to_string(),
                            });
                        } else {
                            *total_bytes = total_bytes.saturating_add(metadata.len());
                            output.files.push(ResolvedInputFile {
                                path,
                                size: Memory::from(metadata.len()),
                            });
                        }
                    } else {
                        output.failures.push(InputFailure {
                            path,
                            reason: "unsupported-entry".to_string(),
                        });
                    }
                }
            }
        }
    }
}
