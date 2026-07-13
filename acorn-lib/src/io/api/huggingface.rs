//! Hugging Face Hub API helpers.
use super::{Endpoint, IntoHeaders, Params, RemoteResource, ResponseContent, TreeEntry};
use crate::io::{first_env_var, ApiResult};
use crate::util::constants::env::HUGGINGFACE_TOKEN_VARIABLE_NAMES;
use axum::http::HeaderMap;
use color_eyre::eyre::eyre;

/// File metadata returned by the Hugging Face repository tree API
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryFile {
    /// Repository-relative path
    pub path: String,
    /// File size in bytes, when provided by Hugging Face
    pub size: Option<u64>,
}
/// Build Hugging Face authorization headers using configured environment token values.
pub fn auth_headers() -> HeaderMap {
    Params::new()
        .with_auth(first_env_var(&HUGGINGFACE_TOKEN_VARIABLE_NAMES).unwrap_or_default().as_str(), None)
        .build()
        .into_headers()
}
/// Return whether a non-empty Hugging Face token is configured
pub fn has_auth_token() -> bool {
    first_env_var(&HUGGINGFACE_TOKEN_VARIABLE_NAMES).is_some()
}
/// List files in a Hugging Face model repository at `revision`
pub async fn repository_tree(identifier: &str, revision: &str) -> ApiResult<Vec<RepositoryFile>> {
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
