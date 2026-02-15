//! Module for interacting with GitHub API
//!
use crate::io::api::TreeEntryType;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
/// Struct for [GitHub] tree entry
///
/// [GitHub]: https://docs.github.com/en/rest
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GithubTreeEntry {
    /// Path of tree entry
    ///
    /// The path inside the repository. Used to get content of subdirectories.
    pub path: String,
    /// Mode of tree entry
    pub mode: String,
    /// Type of tree entry
    #[serde(rename = "type")]
    pub entry_type: TreeEntryType,
    /// [SHA1] of entry
    ///
    /// [SHA1]: https://en.wikipedia.org/wiki/SHA-1
    pub sha: String,
    /// Size of associated data
    /// ### Note
    /// > Not included for "tree" type entries
    pub size: Option<u64>,
    /// URL of associated data API endpoint
    ///
    /// Basically, a combination of the API endpoint and the SHA
    pub url: String,
}
/// Struct for [GitHub] tree API response
///
/// GitHub API endpoint for trees returns
/// ```json
/// {
///   "sha": "...",
///   "url": "<endpoint>/repos/<owner>/<repo>/git/trees/<sha>",
///   "tree": [...],
///   "truncated": false
/// }
/// ```
/// where `"tree"` is a list of [GithubTreeEntry].
///
/// ### Example Endpoint
/// > `https://api.github.com/repos/jhwohlgemuth/pwsh-prelude/git/trees/master?recursive=1`
///
/// See [documentation] for more information
///
/// [GitHub]: https://docs.github.com/en/rest
/// [documentation]: https://docs.github.com/en/rest/git/trees?apiVersion=2022-11-28#get-a-tree
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GithubTreeResponse {
    /// SHA1 of tree
    pub sha: String,
    /// URL of associated data API endpoint
    pub url: String,
    /// List of [GithubTreeEntry]
    pub tree: Vec<GithubTreeEntry>,
    /// Whether tree is truncated
    pub truncated: bool,
}
impl GithubTreeEntry {
    /// Get path of tree entry
    pub fn path(self) -> String {
        self.path
    }
    /// Whether tree entry is a blob
    pub fn is_blob(&self) -> bool {
        self.entry_type.eq(&TreeEntryType::Blob)
    }
}
