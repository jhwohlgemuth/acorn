//! Focused Harbor v2 artifact discovery and metadata operations
use crate::io::api::{EmptyField, Endpoint, Param, RemoteResource};
use crate::io::config::RegistryProfile;
use crate::io::ApiResult;
use crate::param;
use crate::prelude::{env, String, ToString, Vec};
use color_eyre::eyre::eyre;
use core::{fmt, iter::once};
use data_encoding::BASE64;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

const PAGE_SIZE: usize = 100;
const MAX_PAGES: usize = 100;
/// Typed Harbor API failure without credential material.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HarborError {
    /// Authentication is required or invalid.
    Authentication,
    /// Authenticated identity lacks access.
    Forbidden,
    /// Project, repository, or artifact was not found.
    NotFound,
    /// Harbor rate limit was reached.
    RateLimited,
    /// Harbor or the transport returned an unexpected failure.
    Request(String),
}
/// Minimal Harbor artifact metadata used to enrich OCI model planning.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Artifact {
    /// Manifest digest.
    pub digest: String,
    /// Manifest media type.
    #[serde(default)]
    pub media_type: Option<String>,
    /// Declared artifact type.
    #[serde(default)]
    pub artifact_type: Option<String>,
    /// Artifact size in bytes.
    #[serde(default)]
    pub size: Option<u64>,
    /// Harbor push timestamp.
    #[serde(default)]
    pub push_time: Option<String>,
    /// Tags associated with the artifact.
    #[serde(default)]
    pub tags: Vec<Tag>,
    /// Harbor labels associated with the artifact.
    #[serde(default)]
    pub labels: Vec<Label>,
}
/// Harbor artifact tag metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Tag {
    /// Tag name.
    pub name: String,
}
/// Harbor artifact label metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Label {
    /// Label name.
    pub name: String,
    /// Label value, when configured.
    #[serde(default)]
    pub description: Option<String>,
}
/// Registry and artifact coordinates for Harbor service requests.
#[derive(Clone, Debug)]
pub struct Options<'a> {
    profile: &'a RegistryProfile,
    pub(crate) project: &'a str,
    pub(crate) repository: &'a str,
    reference: Option<&'a str>,
}
impl fmt::Display for HarborError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::Authentication => write!(formatter, "Harbor authentication failed"),
            | Self::Forbidden => write!(formatter, "Harbor access is forbidden"),
            | Self::NotFound => write!(formatter, "Harbor artifact was not found"),
            | Self::RateLimited => write!(formatter, "Harbor request was rate limited"),
            | Self::Request(message) => write!(formatter, "Harbor request failed — {message}"),
        }
    }
}
impl core::error::Error for HarborError {}
impl<'a> Options<'a> {
    /// Parse a project-qualified repository for Harbor service requests.
    pub fn from_repository(profile: &'a RegistryProfile, repository: &'a str) -> ApiResult<Self> {
        split_repository(repository).map(|(project, repository)| Self {
            profile,
            project,
            repository,
            reference: None,
        })
    }
    /// Attach a tag or digest for a single-artifact request.
    pub fn with_reference(self, reference: &'a str) -> Self {
        Self {
            reference: Some(reference),
            ..self
        }
    }
}
/// Probe a Harbor deployment's health endpoint.
pub async fn health(profile: &RegistryProfile) -> ApiResult<()> {
    match (endpoint(profile), params(profile, Vec::new())) {
        | (Ok(endpoint), Ok(params)) => endpoint
            .invoke_with::<EmptyField, EmptyField>("health", Some(params))
            .await
            .map(|_| ())
            .map_err(|why| eyre!(HarborError::Request(why.to_string()))),
        | (Err(why), _) | (_, Err(why)) => Err(why),
    }
}
/// Retrieve one Harbor artifact by tag or digest.
pub async fn artifact(options: &Options<'_>) -> ApiResult<Artifact> {
    match options.reference {
        | Some(reference) => {
            let project = encode(options.project);
            let repository = encode(options.repository);
            let reference = encode(reference);
            let values = vec![
                param!(TemplateValue, "project", project.as_ref()),
                param!(TemplateValue, "repository", repository.as_ref()),
                param!(TemplateValue, "reference", reference.as_ref()),
            ];
            match (endpoint(options.profile), params(options.profile, values)) {
                | (Ok(endpoint), Ok(params)) => {
                    let response = endpoint.invoke_with::<EmptyField, EmptyField>("artifact", Some(params)).await;
                    endpoint
                        .handle::<Artifact>(response)
                        .map_err(|why| eyre!(HarborError::Request(format!("Invalid response for 'artifact' — {why}"))))
                }
                | (Err(why), _) | (_, Err(why)) => Err(why),
            }
        }
        | None => Err(eyre!("Harbor artifact request requires a tag or digest reference")),
    }
}
/// List all Harbor artifacts with deterministic bounded pagination.
pub async fn artifacts(options: &Options<'_>) -> ApiResult<Vec<Artifact>> {
    let mut artifacts = Vec::new();
    for page in 1..=MAX_PAGES {
        let page_value = page.to_string();
        let page_size = PAGE_SIZE.to_string();
        let project_value = encode(options.project);
        let repository_value = encode(options.repository);
        let values = vec![
            param!(TemplateValue, "project", project_value.as_ref()),
            param!(TemplateValue, "repository", repository_value.as_ref()),
            param!(KeyValuePair, "page", page_value.as_str()),
            param!(KeyValuePair, "page_size", page_size.as_str()),
            param!(KeyValuePair, "with_tag", "true"),
            param!(KeyValuePair, "with_label", "true"),
        ];
        let page_artifacts = match (endpoint(options.profile), params(options.profile, values)) {
            | (Ok(endpoint), Ok(params)) => {
                let response = endpoint.invoke_with::<EmptyField, EmptyField>("artifacts", Some(params)).await;
                endpoint
                    .handle::<Vec<Artifact>>(response)
                    .map_err(|why| eyre!(HarborError::Request(format!("Invalid response for 'artifacts' — {why}"))))
            }
            | (Err(why), _) | (_, Err(why)) => Err(why),
        };
        match page_artifacts {
            | Ok(values) => {
                let is_last_page = values.len() < PAGE_SIZE;
                artifacts.extend(values);
                if is_last_page {
                    return Ok(artifacts);
                }
            }
            | Err(why) => return Err(why),
        }
    }
    Err(eyre!("Harbor artifact pagination exceeded {MAX_PAGES} pages"))
}
pub(crate) fn endpoint(profile: &RegistryProfile) -> ApiResult<Endpoint> {
    let endpoint_uri = profile.endpoint.trim_end_matches('/');
    match endpoint_uri.starts_with("https://") || profile.plain_http && endpoint_uri.starts_with("http://") {
        | true => Endpoint::from_template("harbor::api").map(|endpoint| endpoint.with_domain(endpoint_uri)),
        | false => Err(eyre!("Harbor endpoint must use HTTPS (plain HTTP is restricted to loopback development)")),
    }
}
pub(crate) fn params(profile: &RegistryProfile, params: Vec<Param>) -> ApiResult<Vec<Param>> {
    match (&profile.credential_env, &profile.username) {
        | (Some(name), username) => match env::var(name) {
            | Ok(secret) => {
                let authorization = match username {
                    | Some(username) => {
                        let credentials = format!("{username}:{secret}");
                        format!("Basic {}", BASE64.encode(credentials.as_bytes()))
                    }
                    | None => format!("Bearer {secret}"),
                };
                Ok(params
                    .into_iter()
                    .chain(once(param!(Header, "Authorization", authorization.as_str())))
                    .collect())
            }
            | Err(_) => Err(eyre!("Harbor credential environment variable '{name}' is not set")),
        },
        | (None, _) => Ok(params),
    }
}
fn split_repository(repository: &str) -> ApiResult<(&str, &str)> {
    repository
        .split_once('/')
        .filter(|(project, repository)| !project.is_empty() && !repository.is_empty())
        .ok_or_else(|| eyre!("Harbor repository must include a project and repository path"))
}
