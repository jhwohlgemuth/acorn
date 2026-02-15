//! Module for interacting with GitLab API
//!
use crate::io::api::TreeEntryType;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Type for GitLab API response for a tree
pub type GitlabTreeResponse = Vec<GitlabTreeEntry>;
/// Struct for GitLab tree entry
///
/// See <https://docs.gitlab.com/api/repositories/#list-repository-tree>
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitlabTreeEntry {
    /// Integer ID of GitLab project
    ///
    /// See <https://docs.gitlab.com/api/projects/#get-a-single-project> for more information
    pub id: String,
    /// Name of tree entry
    pub name: String,
    /// Type of tree entry
    #[serde(rename = "type")]
    pub entry_type: TreeEntryType,
    /// Path of tree entry
    ///
    /// The path inside the repository. Used to get content of subdirectories.
    pub path: String,
    /// Mode of tree entry
    pub mode: String,
}
impl GitlabTreeEntry {
    /// Get path of tree entry
    pub fn path(self) -> String {
        self.path
    }
    /// Whether tree entry is a blob
    pub fn is_blob(&self) -> bool {
        let Self { entry_type, .. } = self;
        entry_type.eq(&TreeEntryType::Blob)
    }
}
