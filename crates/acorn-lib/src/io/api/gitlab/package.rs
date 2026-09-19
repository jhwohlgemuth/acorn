//! Immutable GitLab generic-package publication for generated artifacts.
use super::Options;
use crate::io::api::{require_non_empty_secret, Configuration, Endpoint, Param, Params};
use crate::io::http::{HttpResponse, HttpService, ReqwestHttpService};
use crate::io::ApiResult;
use crate::util::constants::env::GITLAB_TOKEN_VARIABLE_NAMES;
use acorn_core::prelude::{String, Vec};
use acorn_schema::research_activity::ReportingWindow;
use acorn_schema::validation::Validate;
use async_trait::async_trait;
use color_eyre::eyre::eyre;
use data_encoding::HEXLOWER;
use futures::{future, stream, StreamExt, TryStreamExt};
use ring::digest::{digest, SHA256};
use schemars::JsonSchema;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

const MAX_PACKAGE_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

/// Lifecycle for an immutable GitLab package.
#[async_trait]
pub trait PackageLifecycle {
    /// Construct a package from GitLab API and package-specific options.
    fn new(client: Options, options: PackageOptions) -> ApiResult<Self>
    where
        Self: Sized;
    /// Download the newest complete report manifest for this package.
    async fn download(&self) -> ApiResult<Option<ReportManifest>>;
    /// Publish an optional report payload followed by its final manifest.
    async fn publish(&self, manifest: &ReportManifest, pdf: Option<&[u8]>) -> ApiResult<()>;
    /// Build the authenticated URL for one immutable package file.
    fn url(&self, version: &str, file_name: &str) -> ApiResult<String>;
}
/// Generated artifact package outcome.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, strum::Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ReportStatus {
    /// A PDF and final manifest were published.
    Published,
    /// No entries were selected and only the final manifest was published.
    NoChanges,
}
/// Central abstraction for immutable GitLab package operations.
#[derive(Clone)]
pub struct Package {
    client: Options,
    options: PackageOptions,
}
/// Stable coordinates for one generated-artifact package.
#[derive(Clone, Debug, Eq, PartialEq, Validate)]
pub struct PackageOptions {
    #[validate(nonempty)]
    package_name: String,
    #[validate(nonempty)]
    project_id: String,
}
#[derive(Clone, Debug, Deserialize)]
struct PackageVersion {
    status: String,
    version: String,
}
/// Final immutable reporting ledger stored as `manifest.json`.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReportManifest {
    /// Authenticated package URL without embedded credentials.
    pub authenticated_url: String,
    /// SHA-256 checksum for `report.pdf`, omitted for no-change periods.
    pub checksum: Option<String>,
    /// Selected logbook entry identifiers in deterministic order.
    pub entry_identifiers: Vec<String>,
    /// Inclusive reporting lower bound, or `None` for full history.
    pub from: Option<String>,
    /// GitLab job identifier that published the package.
    pub job_id: String,
    /// GitLab generic-package name.
    pub package_name: String,
    /// GitLab pipeline identifier that produced the report.
    pub pipeline_id: String,
    /// Stable PowerAutomate project identifier.
    pub project_id: String,
    /// Exact merged default-branch commit used as input.
    pub source_commit: String,
    /// Publication outcome.
    pub status: ReportStatus,
    /// Exclusive reporting upper bound.
    pub through: String,
    /// Deterministic generic-package version.
    pub version: String,
}
impl Package {
    /// Download through an injected HTTP service.
    pub(super) async fn download_with(&self, service: &impl HttpService) -> ApiResult<Option<ReportManifest>> {
        let versions = {
            let project = self.client.identifier().ok_or_else(|| eyre!("GitLab project identifier is required"));
            match project {
                | Ok(project) => {
                    let identifier = urlencoding::encode(project);
                    let params = Params::new()
                        .with_template("identifier", Some(identifier.as_ref()))
                        .with_keyvalue("order_by", Some("created_at"))
                        .with_keyvalue("package_name", Some(&self.options.package_name))
                        .with_keyvalue("package_type", Some("generic"))
                        .with_keyvalue("per_page", Some("100"))
                        .with_keyvalue("sort", Some("desc"))
                        .build();
                    self.request_resource(service, "packages", params, None, None).await.and_then(|response| {
                        response.success("list GitLab report packages").and_then(|response| {
                            serde_json::from_slice::<Vec<PackageVersion>>(&response.body)
                                .map_err(|why| eyre!("Failed to decode GitLab package list — {why}"))
                        })
                    })
                }
                | Err(why) => Err(why),
            }
        };
        match versions {
            | Ok(versions) => {
                let versions = versions.into_iter().filter(|package| package.status == "default");
                let manifests = stream::iter(versions)
                    .then(|package| async move {
                        match self.file_params(&package.version, "manifest.json") {
                            | Ok(params) => match self.request_resource(service, "package-file", params, None, None).await {
                                | Ok(response) if response.status_code == 404 => Ok(None),
                                | Ok(response) => response.success("read GitLab report manifest").and_then(|response| {
                                    serde_json::from_slice::<ReportManifest>(&response.body)
                                        .map_err(|why| eyre!("Failed to decode GitLab report manifest — {why}"))
                                        .map(|manifest| self.options.contains(&manifest).then_some(manifest))
                                }),
                                | Err(why) => Err(why),
                            },
                            | Err(why) => Err(why),
                        }
                    })
                    .try_filter_map(|manifest| future::ready(Ok(manifest)));
                futures::pin_mut!(manifests);
                manifests.try_next().await
            }
            | Err(why) => Err(why),
        }
    }
    /// Publish through an injected HTTP service.
    pub(super) async fn publish_with(&self, manifest: &ReportManifest, pdf: Option<&[u8]>, service: &impl HttpService) -> ApiResult<()> {
        let expected_checksum = pdf.map(|bytes| HEXLOWER.encode(digest(&SHA256, bytes).as_ref()));
        let valid = self.options.contains(manifest)
            && match (manifest.status, pdf, manifest.checksum.as_ref()) {
                | (ReportStatus::Published, Some(_), Some(checksum)) => expected_checksum.as_ref() == Some(checksum),
                | (ReportStatus::NoChanges, None, None) => manifest.entry_identifiers.is_empty(),
                | _ => false,
            };
        match valid {
            | false => Err(eyre!("GitLab report manifest does not match its PDF publication state")),
            | true => {
                let pdf_upload = match pdf {
                    | Some(pdf) => {
                        let params = self.file_params(&manifest.version, "report.pdf");
                        match params {
                            | Ok(params) => self.upload(service, params, pdf, "application/pdf").await,
                            | Err(why) => Err(why),
                        }
                    }
                    | None => Ok(()),
                };
                match pdf_upload {
                    | Ok(()) => match serde_json::to_vec_pretty(manifest) {
                        | Ok(mut bytes) => {
                            bytes.push(b'\n');
                            match self.file_params(&manifest.version, "manifest.json") {
                                | Ok(params) => self.upload(service, params, &bytes, "application/json").await,
                                | Err(why) => Err(why),
                            }
                        }
                        | Err(why) => Err(eyre!("Failed to encode GitLab report manifest — {why}")),
                    },
                    | Err(why) => Err(why),
                }
            }
        }
    }
    fn file_url(&self, version: &str, file_name: &str) -> ApiResult<String> {
        self.file_params(version, file_name).and_then(|params| {
            Endpoint::from_template("gitlab::api")
                .map(|endpoint| endpoint.with_domain(self.client.domain()))
                .and_then(|endpoint| endpoint.resource_url("package-file", Some(params)))
        })
    }
    fn file_params(&self, version: &str, file_name: &str) -> ApiResult<Vec<Param>> {
        self.client
            .identifier()
            .ok_or_else(|| eyre!("GitLab project identifier is required"))
            .and_then(|project| self.options.file_params(project, version, file_name))
    }
    async fn request_resource(
        &self,
        service: &impl HttpService,
        resource: &str,
        params: Vec<Param>,
        body: Option<Vec<u8>>,
        content_type: Option<&str>,
    ) -> ApiResult<HttpResponse> {
        let token = require_non_empty_secret(&self.client.token, "gitlab::api::package", &GITLAB_TOKEN_VARIABLE_NAMES);
        let endpoint = Endpoint::from_template("gitlab::api").map(|endpoint| endpoint.with_domain(self.client.domain()));
        match (token, endpoint) {
            | (Ok(token), Ok(endpoint)) => {
                let params = Params::new()
                    .with_auth(ExposeSecret::expose_secret(&token), Some(self.client.token_header()))
                    .with_custom(&params)
                    .build();
                endpoint
                    .execute_resource(service, resource, Some(params), body, content_type, MAX_PACKAGE_RESPONSE_BYTES)
                    .await
            }
            | (Err(why), _) | (_, Err(why)) => Err(why),
        }
    }
    async fn upload(&self, service: &impl HttpService, params: Vec<Param>, body: &[u8], content_type: &str) -> ApiResult<()> {
        match self
            .request_resource(service, "package-file::upload", params.clone(), Some(body.to_vec()), Some(content_type))
            .await
        {
            | Ok(response) if (200..=299).contains(&response.status_code) => Ok(()),
            | Ok(response) if response.status_code == 409 => match self.request_resource(service, "package-file", params, None, None).await {
                | Ok(existing) if (200..=299).contains(&existing.status_code) && existing.body == body => Ok(()),
                | Ok(_) => Err(eyre!("GitLab package file already exists with different content")),
                | Err(why) => Err(why),
            },
            | Ok(response) => Err(eyre!("GitLab package upload failed with HTTP {}", response.status_code)),
            | Err(why) => Err(why),
        }
    }
}
#[async_trait]
impl PackageLifecycle for Package {
    fn new(client: Options, options: PackageOptions) -> ApiResult<Self> {
        options
            .validate()
            .map_err(|why| eyre!("Invalid GitLab package options — {why}"))
            .map(|()| Self { client, options })
    }
    async fn download(&self) -> ApiResult<Option<ReportManifest>> {
        self.download_with(&ReqwestHttpService::default()).await
    }
    async fn publish(&self, manifest: &ReportManifest, pdf: Option<&[u8]>) -> ApiResult<()> {
        self.publish_with(manifest, pdf, &ReqwestHttpService::default()).await
    }
    fn url(&self, version: &str, file_name: &str) -> ApiResult<String> {
        self.file_url(version, file_name)
    }
}
impl PackageOptions {
    /// Construct stable package coordinates.
    pub fn new(package_name: impl Into<String>, project_id: impl Into<String>) -> Self {
        Self {
            package_name: package_name.into(),
            project_id: project_id.into(),
        }
    }
    /// Return the GitLab generic-package name.
    pub fn package_name(&self) -> &str {
        &self.package_name
    }
    /// Return the stable automated-artifact project identifier.
    pub fn project_id(&self) -> &str {
        &self.project_id
    }
    fn contains(&self, manifest: &ReportManifest) -> bool {
        manifest.package_name == self.package_name && manifest.project_id == self.project_id
    }
    fn file_params(&self, gitlab_project: &str, version: &str, file_name: &str) -> ApiResult<Vec<Param>> {
        match [gitlab_project, self.package_name.as_str(), version, file_name]
            .iter()
            .any(|value| value.trim().is_empty())
        {
            | true => Err(eyre!("GitLab project, package name, version, and file name are required")),
            | false => {
                let file = urlencoding::encode(file_name);
                let identifier = urlencoding::encode(gitlab_project);
                let package = urlencoding::encode(&self.package_name);
                let version = urlencoding::encode(version);
                Ok(Params::new()
                    .with_template("file", Some(file.as_ref()))
                    .with_template("identifier", Some(identifier.as_ref()))
                    .with_template("package", Some(package.as_ref()))
                    .with_template("version", Some(version.as_ref()))
                    .build())
            }
        }
    }
}
impl ReportManifest {
    /// Build a final report manifest with deterministic version and checksum fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        package_name: impl Into<String>,
        project_id: impl Into<String>,
        source_commit: impl Into<String>,
        window: &ReportingWindow,
        mut entry_identifiers: Vec<String>,
        pipeline_id: impl Into<String>,
        job_id: impl Into<String>,
        pdf: Option<&[u8]>,
        authenticated_url: impl Into<String>,
    ) -> ApiResult<Self> {
        window.validate().map_err(|why| eyre!(why)).and_then(|()| {
            let package_name = package_name.into();
            let project_id = project_id.into();
            let source_commit = source_commit.into();
            match (
                package_name.trim().is_empty(),
                project_id.trim().is_empty(),
                source_commit.len() < 7 || !source_commit.chars().all(|character| character.is_ascii_hexdigit()),
            ) {
                | (true, _, _) => Err(eyre!("GitLab report package name is required")),
                | (_, true, _) => Err(eyre!("Report project identifier is required")),
                | (_, _, true) => Err(eyre!("Report source commit must be a hexadecimal Git commit identifier")),
                | _ => {
                    entry_identifiers.sort();
                    entry_identifiers.dedup();
                    let checksum = pdf.map(|bytes| HEXLOWER.encode(digest(&SHA256, bytes).as_ref()));
                    let status = match pdf {
                        | Some(_) => ReportStatus::Published,
                        | None => ReportStatus::NoChanges,
                    };
                    let commit = source_commit.chars().filter(char::is_ascii_hexdigit).take(12).collect::<String>();
                    let through = window.right.replace([':', '-'], "");
                    let project = project_id.to_ascii_lowercase();
                    Ok(Self {
                        authenticated_url: authenticated_url.into(),
                        checksum,
                        entry_identifiers,
                        from: window.left.clone(),
                        job_id: job_id.into(),
                        package_name,
                        pipeline_id: pipeline_id.into(),
                        project_id,
                        source_commit,
                        status,
                        through: window.right.clone(),
                        version: format!("{project}-{through}-{commit}"),
                    })
                }
            }
        })
    }
}
