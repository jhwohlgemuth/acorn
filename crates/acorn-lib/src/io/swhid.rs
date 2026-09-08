//! Local file-system and Git SWHID calculation
//!
//! Directory identifiers use filesystem permissions (`FilesystemPermissionsSource`) and do not follow symlinks (`follow_symlinks = false`).
//! On Unix the executable bit (`0o100755` vs `0o100644`) participates in the directory manifest, so changing it changes the SWHID.
//! Symlinks appear as `0o120000` entries whose target bytes are hashed, not followed. Special files (FIFOs, sockets, device nodes) are rejected.
//!
//! Git calculations require SHA-1 repositories;
//! SHA-256 (`extensions.objectFormat = sha256`) is rejected with an actionable diagnostic.
use crate::error::ApiResult;
use crate::{args, cmd};
use acorn_schema::pid::swhid::ObjectType;
use acorn_schema::pid::SWHID;
use color_eyre::eyre::{eyre, WrapErr};
use core::str::FromStr;
use std::fs::read;
use std::path::Path;

/// Object source used for SWHID calculation
/// ### Note
/// This is distinct from repository API entry types because it also models local-path inference and SWHID-specific Git object traversal.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CalculationKind {
    /// Infer content or directory from the local path
    #[default]
    Auto,
    /// Hash a local file as a content object
    Content,
    /// Traverse a local directory as a directory object
    Directory,
    /// Hash the Git blob selected by a revision expression
    GitBlob,
    /// Hash the Git tree selected by a revision expression
    GitTree,
    /// Traverse the Git commit selected by a revision expression
    Revision,
    /// Traverse an annotated Git tag
    Release,
    /// Traverse all local Git branches and tags as a snapshot
    Snapshot,
}
#[derive(Clone, Copy)]
struct CalculationPath<'a>(&'a Path);
impl<'a> CalculationPath<'a> {
    const fn new(path: &'a Path) -> Self {
        Self(path)
    }
    fn calculate(self, kind: CalculationKind, reference: Option<&str>) -> ApiResult<SWHID> {
        let resolved = match kind {
            | CalculationKind::Auto if self.0.is_file() => self.calculate_content(),
            | CalculationKind::Auto if self.0.is_dir() => self.calculate_directory(),
            | CalculationKind::Auto => Err(eyre!("{} is neither a file nor a directory", self.0.display())),
            | CalculationKind::Content => self.calculate_content(),
            | CalculationKind::Directory => self.calculate_directory(),
            | CalculationKind::GitBlob => self.calculate_git_oid(reference.unwrap_or("HEAD"), ObjectType::Content),
            | CalculationKind::GitTree => self.calculate_git_oid(reference.unwrap_or("HEAD^{tree}"), ObjectType::Directory),
            | CalculationKind::Revision => self.calculate_revision(reference.unwrap_or("HEAD")),
            | CalculationKind::Release => reference
                .ok_or_else(|| eyre!("a tag name is required for release calculation"))
                .and_then(|tag| self.calculate_release(tag)),
            | CalculationKind::Snapshot => self.calculate_snapshot(),
        };
        resolved.and_then(|resolved| SWHID::from_str(&resolved).map_err(|why| eyre!("calculated an invalid SWHID — {why}")))
    }
    fn calculate_content(self) -> ApiResult<String> {
        read(self.0)
            .wrap_err_with(|| format!("failed to read {}", self.0.display()))
            .map(|bytes| swhid::Content::from_bytes(bytes).swhid().to_string())
    }
    fn calculate_directory(self) -> ApiResult<String> {
        swhid::DiskDirectoryBuilder::new(self.0)
            .swhid()
            .map(|identifier| identifier.to_string())
            .map_err(|why| eyre!("failed to traverse {}: {why}", self.0.display()))
    }
    fn calculate_revision(self, reference: &str) -> ApiResult<String> {
        ensure_sha1_repository(self.0).and_then(|()| {
            swhid::git::open_repo(self.0)
                .map_err(|why| eyre!("failed to open Git repository {}: {why}", self.0.display()))
                .and_then(|repository| {
                    repository
                        .revparse_single(reference)
                        .map_err(|why| eyre!("failed to resolve Git revision `{reference}`: {why}"))
                        .and_then(|object| {
                            object
                                .peel_to_commit()
                                .map_err(|why| eyre!("Git reference `{reference}` does not resolve to a commit: {why}"))
                                .and_then(|commit| {
                                    swhid::git::revision_swhid(&repository, &commit.id())
                                        .map(|identifier| identifier.to_string())
                                        .map_err(|why| eyre!("failed to calculate Git revision SWHID: {why}"))
                                })
                        })
                })
        })
    }
    fn calculate_release(self, tag: &str) -> ApiResult<String> {
        let reference = if tag.starts_with("refs/tags/") {
            tag.to_string()
        } else {
            format!("refs/tags/{tag}")
        };
        ensure_sha1_repository(self.0).and_then(|()| {
            swhid::git::open_repo(self.0)
                .map_err(|why| eyre!("failed to open Git repository {}: {why}", self.0.display()))
                .and_then(|repository| {
                    repository
                        .refname_to_id(&reference)
                        .map_err(|why| eyre!("failed to resolve annotated Git tag `{tag}`: {why}"))
                        .and_then(|object_id| {
                            swhid::git::release_swhid(&repository, &object_id)
                                .map(|identifier| identifier.to_string())
                                .map_err(|why| {
                                    let message = why.to_string();
                                    match message.contains("Failed to find tag") {
                                        | true => eyre!("Git tag `{tag}` is a lightweight tag, not an annotated tag — release SWHIDs require annotated tags (create with `git tag -a {tag} -m \"message\"`): {why}"),
                                        | false => eyre!("failed to calculate Git release SWHID: {why}"),
                                    }
                                })
                        })
                })
        })
    }
    fn calculate_snapshot(self) -> ApiResult<String> {
        ensure_sha1_repository(self.0).and_then(|()| {
            swhid::git::open_repo(self.0)
                .map_err(|why| eyre!("failed to open Git repository {}: {why}", self.0.display()))
                .and_then(|repository| {
                    swhid::git::snapshot_swhid(&repository)
                        .map(|identifier| identifier.to_string())
                        .map_err(|why| eyre!("failed to calculate Git snapshot SWHID: {why}"))
                })
        })
    }
    fn calculate_git_oid(self, reference: &str, object_type: ObjectType) -> ApiResult<String> {
        ensure_sha1_repository(self.0).and_then(|()| {
            swhid::git::open_repo(self.0)
                .map_err(|why| eyre!("failed to open Git repository {}: {why}", self.0.display()))
                .and_then(|repository| {
                    repository
                        .revparse_single(reference)
                        .map_err(|why| eyre!("failed to resolve Git object `{reference}`: {why}"))
                        .and_then(|object| match object_type {
                            | ObjectType::Content => object
                                .peel_to_blob()
                                .map(|object| format!("swh:1:{}:{}", object_type.as_str(), object.id()))
                                .map_err(|why| eyre!("Git reference `{reference}` does not resolve to a blob: {why}")),
                            | _ => object
                                .peel_to_tree()
                                .map(|object| format!("swh:1:{}:{}", object_type.as_str(), object.id()))
                                .map_err(|why| eyre!("Git reference `{reference}` does not resolve to a tree: {why}")),
                        })
                })
        })
    }
}
/// Calculate a SWHID for a local file, directory, or Git object
///
/// Git calculations are read-only and use only objects already present in the local repository.
/// `reference` defaults to `HEAD` where applicable.
pub fn calculate(path: &Path, kind: CalculationKind, reference: Option<&str>) -> ApiResult<SWHID> {
    CalculationPath::new(path).calculate(kind, reference)
}
fn ensure_sha1_repository(path: &Path) -> ApiResult<()> {
    let output = cmd!("git", args!["config", "--get", "extensions.objectFormat"]; dir: path);
    match output {
        | Ok(result) if result.status.success() => {
            let value = String::from_utf8_lossy(&result.stdout).trim().to_string();
            match value.as_str() {
                | "sha256" => Err(eyre!("Git repository uses SHA-256 object format which is not supported for SWHID calculation — SWHIDs require SHA-1 (objectFormat=sha1); re-initialize without --object-format=sha256")),
                | _ => Ok(()),
            }
        }
        | _ => Ok(()),
    }
}
/// Verify that a local object calculates to the expected SWHID core object
pub fn verify(path: &Path, kind: CalculationKind, reference: Option<&str>, expected: &SWHID) -> ApiResult<bool> {
    calculate(path, kind, reference).map(|actual| actual.core_identifier() == expected.core_identifier())
}
