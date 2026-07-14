//! Hugging Face Hub API helpers.
use super::{Endpoint, IntoHeaders, Params, RemoteResource, ResponseContent, TreeEntry};
use crate::io::config::FilterSet;
use crate::io::{first_env_var, ApiResult};
use crate::util::constants::env::HUGGINGFACE_TOKEN_VARIABLE_NAMES;
use crate::util::{glob_matches, regex_to_glob};
use axum::http::HeaderMap;
use color_eyre::eyre::eyre;

/// Repository files returned by the Hugging Face repository tree API
pub type RepositoryFiles = Vec<RepositoryFile>;
/// Trait for selecting and filtering files from a Hugging Face repository.
pub trait HuggingFaceRepository {
    /// Filter repository files using glob patterns for include/exclude.
    fn filter(&self, filter: Option<&str>, ignore: Option<&str>) -> ApiResult<Self>
    where
        Self: Sized;
    /// Select preferred files based on a marker pattern, erroring on ambiguity.
    fn select(&self, policy: &FileSelectionPolicy<'_>) -> ApiResult<Self>
    where
        Self: Sized;
}
/// Policy for selecting a preferred file when multiple candidates exist.
#[derive(Clone, Debug)]
pub struct FileSelectionPolicy<'a> {
    /// Substring to match in the filename (case-insensitive) to identify the preferred file.
    pub preferred_marker: &'a str,
    /// Error message to display when no files match.
    pub no_match_message: &'a str,
}
/// File metadata returned by the Hugging Face repository tree API
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryFile {
    /// Repository-relative path
    pub path: String,
    /// File size in bytes, when provided by Hugging Face
    pub size: Option<u64>,
}
impl HuggingFaceRepository for RepositoryFiles {
    fn filter(&self, filter: Option<&str>, ignore: Option<&str>) -> ApiResult<Self> {
        let filter_vec = filter.into_iter().map(String::from).collect::<Vec<_>>();
        let ignore_vec = ignore.into_iter().map(String::from).collect::<Vec<_>>();
        let filtered = match try_glob_filter(self, filter, ignore) {
            | Some(result) => Ok(result),
            | None => filter_by_patterns(
                self.clone(),
                &filter_vec,
                &ignore_vec,
                |file: &RepositoryFile| file.path.clone(),
                |_| true,
            ),
        };
        filtered.and_then(|files| {
            if filter.is_some() {
                match files.is_empty() {
                    | true => Err(eyre!("No model files matched --filter/--ignore")),
                    | false => Ok(files),
                }
            } else {
                files.select(&FileSelectionPolicy {
                    preferred_marker: "Q4_K_M",
                    no_match_message: "No GGUF model files found; use a GGUF repository or provide --filter for another format",
                })
            }
        })
    }
    fn select(&self, policy: &FileSelectionPolicy<'_>) -> ApiResult<Vec<RepositoryFile>> {
        let gguf_files: Vec<&RepositoryFile> = self.iter().filter(|file| file.path.to_ascii_lowercase().ends_with(".gguf")).collect();
        match gguf_files.len() {
            | 0 => Err(eyre!("{}", policy.no_match_message)),
            | 1 => Ok(gguf_files.into_iter().cloned().collect()),
            | _ => {
                let preferred: Vec<&RepositoryFile> = gguf_files
                    .into_iter()
                    .filter(|file| file.path.to_ascii_uppercase().contains(policy.preferred_marker))
                    .collect();
                match preferred.len() {
                    | 1 => Ok(preferred.into_iter().cloned().collect()),
                    | _ => Err(eyre!(
                        "Multiple model file candidates found; use --filter to choose one: {}",
                        self.iter().map(|file| file.path.clone()).collect::<Vec<_>>().join(", ")
                    )),
                }
            }
        }
    }
}
/// Build Hugging Face authorization headers using configured environment token values.
pub fn auth_headers() -> HeaderMap {
    Params::new()
        .with_auth(first_env_var(&HUGGINGFACE_TOKEN_VARIABLE_NAMES).unwrap_or_default().as_str(), None)
        .build()
        .into_headers()
}
pub(crate) fn filter_by_patterns<T>(
    items: Vec<T>,
    filter: &[String],
    ignore: &[String],
    value: impl Fn(&T) -> String,
    keep: impl Fn(&T) -> bool,
) -> ApiResult<Vec<T>> {
    match FilterSet::compile(ignore, filter) {
        | Ok(FilterSet { ignore, filter }) => Ok(items
            .into_iter()
            .filter(|item| {
                let value = value(item);
                let ignored = ignore.iter().any(|pattern| pattern.is_match(&value).unwrap_or(false));
                let filtered = filter.is_empty() || filter.iter().any(|pattern| pattern.is_match(&value).unwrap_or(false));
                !ignored && filtered && keep(item)
            })
            .collect()),
        | Err(why) => Err(why),
    }
}
/// Return whether a non-empty Hugging Face token is configured
pub fn has_auth_token() -> bool {
    first_env_var(&HUGGINGFACE_TOKEN_VARIABLE_NAMES).is_some()
}
/// Try to filter HuggingFace files using glob patterns
/// ### Note
/// Returns `Some(filtered)` if all patterns converted to globs successfully, `None` to signal regex fallback.
pub(crate) fn try_glob_filter(files: &[RepositoryFile], filter: Option<&str>, ignore: Option<&str>) -> Option<Vec<RepositoryFile>> {
    match (filter.map(regex_to_glob), ignore.map(regex_to_glob)) {
        | (Some(None), _) | (_, Some(None)) => None,
        | (filter_opt, ignore_opt) => {
            let filter_glob = filter_opt.flatten();
            let ignore_glob = ignore_opt.flatten();
            let filtered = files
                .iter()
                .filter(|file| {
                    let ignored = ignore_glob.as_ref().is_some_and(|pattern| glob_matches(&file.path, pattern));
                    let kept = filter_glob.as_ref().is_none_or(|pattern| glob_matches(&file.path, pattern));
                    !ignored && kept
                })
                .cloned()
                .collect();
            Some(filtered)
        }
    }
}
/// List files in a Hugging Face model repository at `revision`
pub async fn repository_tree(identifier: &str, revision: &str) -> ApiResult<RepositoryFiles> {
    let params = Params::new()
        .with_auth(first_env_var(&HUGGINGFACE_TOKEN_VARIABLE_NAMES).unwrap_or_default().as_str(), None)
        .with_template("identifier", Some(identifier))
        .with_template("revision", Some(revision))
        .with_keyvalue("recursive", Some("1"))
        .build();
    match Endpoint::from_template("huggingface::api") {
        | Ok(endpoint) => match endpoint.invoke("tree", Some(params)).await {
            | Ok(ResponseContent::Json(content)) => serde_json::from_str::<Vec<TreeEntry>>(&content)
                .map(|entries| {
                    entries
                        .into_iter()
                        .filter(TreeEntry::is_file)
                        .map(|entry| RepositoryFile {
                            path: entry.path,
                            size: entry.size,
                        })
                        .collect()
                })
                .map_err(|why| eyre!("Failed to parse Hugging Face repository file list — {why}")),
            | Ok(_) => Err(eyre!("Failed to list Hugging Face model files — response was not JSON")),
            | Err(why) => Err(eyre!("Failed to list Hugging Face model files — {why}")),
        },
        | Err(why) => Err(eyre!("Failed to configure Hugging Face API — {why}")),
    }
}
