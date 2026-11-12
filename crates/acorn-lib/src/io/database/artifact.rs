//! Validated source selection for database-backed artifacts
use crate::io::ApiResult;
use acorn_schema::validation::is_identifier;
use color_eyre::eyre::eyre;
use std::path::{Path, PathBuf};

/// One file, current activity, or immutable revision selected for artifact generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactSource(Source);
#[derive(Clone, Debug, Eq, PartialEq)]
enum Source {
    File(PathBuf),
    CurrentActivity(String),
    Revision(String),
}
impl ArtifactSource {
    /// Select a local file without opening it
    pub fn file(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let path = path.into();
        (!path.as_os_str().is_empty())
            .then_some(Self(Source::File(path)))
            .ok_or_else(|| eyre!("Artifact file path cannot be empty"))
    }
    /// Select the current canonical activity by its portable database identifier
    pub fn current_activity(iid: impl Into<String>) -> ApiResult<Self> {
        let iid = iid.into();
        validate_iid(&iid).map(|()| Self(Source::CurrentActivity(iid)))
    }
    /// Select an immutable activity revision by its portable database identifier
    pub fn revision(iid: impl Into<String>) -> ApiResult<Self> {
        let iid = iid.into();
        validate_iid(&iid).map(|()| Self(Source::Revision(iid)))
    }
    /// Select exactly one of a file, current activity, or immutable revision
    pub fn from_inputs(file: Option<PathBuf>, activity_iid: Option<String>, revision_iid: Option<String>) -> ApiResult<Self> {
        match (file, activity_iid, revision_iid) {
            | (Some(path), None, None) => Self::file(path),
            | (None, Some(iid), None) => Self::current_activity(iid),
            | (None, None, Some(iid)) => Self::revision(iid),
            | (None, None, None) => Err(eyre!("Select one artifact source: file, activity, or revision")),
            | _ => Err(eyre!("Artifact source must be exactly one of file, activity, or revision")),
        }
    }
    /// Return the file path when this source selects a file
    pub fn as_file(&self) -> Option<&Path> {
        match &self.0 {
            | Source::File(path) => Some(path),
            | _ => None,
        }
    }
    /// Return the portable identifier when this source selects current activity data
    pub fn as_current_activity(&self) -> Option<&str> {
        match &self.0 {
            | Source::CurrentActivity(iid) => Some(iid),
            | _ => None,
        }
    }
    /// Return the portable identifier when this source selects an immutable revision
    pub fn as_revision(&self) -> Option<&str> {
        match &self.0 {
            | Source::Revision(iid) => Some(iid),
            | _ => None,
        }
    }
}
// Database keys use NanoID; UUID/digest rules would reject existing rows.
fn validate_iid(iid: &str) -> ApiResult<()> {
    (is_identifier(iid) && iid.len() <= 128 && !iid.contains('.'))
        .then_some(())
        .ok_or_else(|| eyre!("Artifact database identifier must contain 1-128 ASCII letters, digits, '_' or '-'"))
}
