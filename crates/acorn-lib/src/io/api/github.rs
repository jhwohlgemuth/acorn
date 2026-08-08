//! Module for interacting with GitHub API
//!
use crate::io::api::{Endpoint, Params, RemoteResource, TreeEntryType};
use crate::io::ApiResult;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
/// Struct for [GitHub] tree entry
///
/// [GitHub]: https://docs.github.com/en/rest
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreeEntry {
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
/// where `"tree"` is a list of [TreeEntry].
///
/// ### Example Endpoint
/// > `https://api.github.com/repos/jhwohlgemuth/pwsh-prelude/git/trees/master?recursive=1`
///
/// See [documentation] for more information
///
/// [GitHub]: https://docs.github.com/en/rest
/// [documentation]: https://docs.github.com/en/rest/git/trees?apiVersion=2022-11-28#get-a-tree
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreeResponse {
    /// SHA1 of tree
    pub sha: String,
    /// URL of associated data API endpoint
    pub url: String,
    /// List of [TreeEntry]
    pub tree: Vec<TreeEntry>,
    /// Whether tree is truncated
    pub truncated: bool,
}
impl TreeEntry {
    /// Get path of tree entry
    pub fn path(self) -> String {
        self.path
    }
    /// Whether tree entry is a file ("blob")
    pub fn is_file(&self) -> bool {
        self.entry_type.eq(&TreeEntryType::File)
    }
}
/// Fetch repository tree blob paths for a GitHub repository and branch
pub(crate) async fn tree_paths(host: impl Into<String>, path: impl Into<String>, branch: impl Into<String>) -> ApiResult<Vec<String>> {
    let endpoint = match Endpoint::from_template("github::api").map(|e| e.with_domain(host)) {
        | Ok(endpoint) => endpoint,
        | Err(why) => return Err(why),
    };
    let path = path.into();
    let branch = branch.into();
    let recursive_response = match fetch_tree(&endpoint, path.as_str(), branch.as_str(), true).await {
        | Ok(data) => data,
        | Err(why) => return Err(why),
    };
    if !recursive_response.truncated {
        let (paths, _) = collect_tree(recursive_response.tree, "");
        return Ok(paths);
    }
    let root_response = match fetch_tree(&endpoint, path.as_str(), branch.as_str(), false).await {
        | Ok(data) if !data.truncated => data,
        | Ok(_) => return Err(eyre!("Failed to fetch complete GitHub tree for {path}")),
        | Err(why) => return Err(why),
    };
    let (mut all_paths, mut pending) = collect_tree(root_response.tree, "");
    while let Some((sha, prefix)) = pending.pop() {
        let subtree_response = match fetch_tree(&endpoint, path.as_str(), sha.as_str(), false).await {
            | Ok(data) if !data.truncated => data,
            | Ok(_) => return Err(eyre!("Failed to fetch complete GitHub tree for {path} at subtree {prefix}")),
            | Err(why) => return Err(why),
        };
        let (paths, subtrees) = collect_tree(subtree_response.tree, prefix.as_str());
        all_paths.extend(paths);
        pending.extend(subtrees);
    }
    Ok(all_paths)
}
pub(crate) fn collect_tree(entries: Vec<TreeEntry>, prefix: &str) -> (Vec<String>, Vec<(String, String)>) {
    entries.into_iter().fold((vec![], vec![]), |(mut paths, mut pending), entry| {
        let is_file = entry.is_file();
        let TreeEntry { path, sha, .. } = entry;
        let full_path = path_for(prefix, path.as_str());
        if is_file {
            paths.push(full_path);
        } else {
            pending.push((sha, full_path));
        }
        (paths, pending)
    })
}
async fn fetch_tree(endpoint: &Endpoint, path: &str, branch: &str, recursive: bool) -> ApiResult<TreeResponse> {
    let params = Params::new()
        .with_template("path", Some(path))
        .with_template("branch", Some(branch))
        .with_keyvalue("recursive", recursive.then_some("1"))
        .build();
    let response = endpoint.invoke("tree", Some(params)).await;
    endpoint.handle::<TreeResponse>(response)
}
pub(crate) fn path_for(prefix: &str, path: &str) -> String {
    if prefix.is_empty() {
        path.to_string()
    } else {
        format!("{prefix}/{path}")
    }
}
