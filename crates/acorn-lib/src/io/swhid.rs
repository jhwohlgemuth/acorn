//! Local file-system and Git SWHID calculation.
use crate::error::ApiResult;
use crate::prelude::{read, Path};
use crate::schema::pid::swhid::ObjectType;
use crate::schema::pid::SWHID;
use color_eyre::eyre::{eyre, WrapErr};
use core::str::FromStr;

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
    }
    fn calculate_release(self, tag: &str) -> ApiResult<String> {
        let reference = if tag.starts_with("refs/tags/") {
            tag.to_string()
        } else {
            format!("refs/tags/{tag}")
        };
        swhid::git::open_repo(self.0)
            .map_err(|why| eyre!("failed to open Git repository {}: {why}", self.0.display()))
            .and_then(|repository| {
                repository
                    .refname_to_id(&reference)
                    .map_err(|why| eyre!("failed to resolve annotated Git tag `{tag}`: {why}"))
                    .and_then(|object_id| {
                        swhid::git::release_swhid(&repository, &object_id)
                            .map(|identifier| identifier.to_string())
                            .map_err(|why| eyre!("failed to calculate Git release SWHID: {why}"))
                    })
            })
    }
    fn calculate_snapshot(self) -> ApiResult<String> {
        swhid::git::open_repo(self.0)
            .map_err(|why| eyre!("failed to open Git repository {}: {why}", self.0.display()))
            .and_then(|repository| {
                swhid::git::snapshot_swhid(&repository)
                    .map(|identifier| identifier.to_string())
                    .map_err(|why| eyre!("failed to calculate Git snapshot SWHID: {why}"))
            })
    }
    fn calculate_git_oid(self, reference: &str, object_type: ObjectType) -> ApiResult<String> {
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
    }
}
/// Calculate a SWHID for a local file, directory, or Git object
///
/// Git calculations are read-only and use only objects already present in the local repository.
/// `reference` defaults to `HEAD` where applicable.
pub fn calculate(path: &Path, kind: CalculationKind, reference: Option<&str>) -> ApiResult<SWHID> {
    CalculationPath::new(path).calculate(kind, reference)
}
/// Verify that a local object calculates to the expected SWHID core object
pub fn verify(path: &Path, kind: CalculationKind, reference: Option<&str>, expected: &SWHID) -> ApiResult<bool> {
    calculate(path, kind, reference).map(|actual| actual.core_identifier() == expected.core_identifier())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_calculate_content_known_vector() {
        let directory = tempfile::tempdir().expect("temporary directory should be created");
        let path = directory.path().join("hello.txt");
        std::fs::write(&path, b"hello\n").expect("fixture should be written");
        let identifier = calculate(&path, CalculationKind::Content, None).expect("content SWHID should calculate");
        assert_eq!(identifier.to_string(), "swh:1:cnt:ce013625030ba8dba906f756967f9e9ca394464a");
    }
    #[test]
    fn test_calculate_git_objects_without_mutating_repository() {
        let directory = tempfile::tempdir().expect("temporary directory should be created");
        let root = directory.path();
        git(root, &["init", "--quiet"]);
        git(root, &["config", "user.name", "ACORN Test"]);
        git(root, &["config", "user.email", "acorn@example.org"]);
        std::fs::write(root.join("hello.txt"), b"hello\n").expect("fixture should be written");
        git(root, &["add", "hello.txt"]);
        git(root, &["commit", "--quiet", "-m", "fixture"]);
        git(root, &["tag", "-a", "v1", "-m", "fixture release"]);
        let revision = calculate(root, CalculationKind::Revision, Some("HEAD")).expect("revision should calculate");
        let blob = calculate(root, CalculationKind::GitBlob, Some("HEAD:hello.txt")).expect("blob should calculate");
        let tree = calculate(root, CalculationKind::GitTree, Some("HEAD^{tree}")).expect("tree should calculate");
        let release = calculate(root, CalculationKind::Release, Some("v1")).expect("release should calculate");
        let snapshot = calculate(root, CalculationKind::Snapshot, None).expect("snapshot should calculate");
        assert!(revision.to_string().starts_with("swh:1:rev:"));
        assert_eq!(blob.object_id, git_output(root, &["rev-parse", "HEAD:hello.txt"]));
        assert_eq!(tree.object_id, git_output(root, &["rev-parse", "HEAD^{tree}"]));
        assert!(release.to_string().starts_with("swh:1:rel:"));
        assert!(snapshot.to_string().starts_with("swh:1:snp:"));
        assert!(git_output(root, &["status", "--porcelain"]).is_empty());
    }
    fn git(root: &Path, arguments: &[&str]) {
        let status = Command::new("git")
            .args(arguments)
            .current_dir(root)
            .status()
            .expect("git should execute");
        assert!(status.success(), "git command failed: {arguments:?}");
    }
    fn git_output(root: &Path, arguments: &[&str]) -> String {
        let output = Command::new("git")
            .args(arguments)
            .current_dir(root)
            .output()
            .expect("git should execute");
        assert!(output.status.success(), "git command failed: {arguments:?}");
        String::from_utf8(output.stdout).expect("git output should be UTF-8").trim().to_string()
    }
}
