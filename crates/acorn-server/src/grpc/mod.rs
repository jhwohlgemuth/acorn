//! Authenticated typed gRPC adapter for the canonical application service.
use crate::proto;
use acorn_analysis::readability::ReadabilityType;
use acorn_analysis::CheckSeverity;
use acorn_core::util::{constant_time_eq, MimeType};
use acorn_service::{
    AcornService, CheckReadabilityRequest, CheckReadabilityResponse, Diagnostic, FileChange, FileChangeKind, ProcessRepositoryRequest,
    ProcessRepositoryResponse, RepositoryFileInput, ServiceError, ServiceErrorKind, ValidateResearchActivityRequest,
    ValidateResearchActivityResponse, VocabularyFuzzyMode, WorkflowAction, WorkflowFileReport,
};
use alloc::sync::Arc;
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, Endpoint};
use tonic::{Request, Response, Status};

const AUTHORIZATION: &str = "authorization";
/// Failures returned by the typed remote gRPC client.
#[derive(Debug, thiserror::Error)]
pub enum GrpcClientError {
    /// Endpoint or bearer metadata was invalid.
    #[error("Invalid gRPC client configuration: {0}")]
    Configuration(String),
    /// The transport could not connect or exchange messages.
    #[error("gRPC transport failed: {0}")]
    Transport(#[from] tonic::transport::Error),
    /// The remote service returned a typed gRPC status.
    #[error("gRPC request failed: {0}")]
    Status(#[from] Status),
}
/// Authenticated gRPC client that accepts and returns canonical service models.
#[derive(Clone)]
pub struct GrpcClient {
    inner: crate::proto::acorn_service_client::AcornServiceClient<Channel>,
    authorization: MetadataValue<tonic::metadata::Ascii>,
}
/// Typed gRPC adapter sharing one portable service instance.
#[derive(Clone)]
pub struct GrpcService {
    service: AcornService,
    token: Arc<str>,
}
impl GrpcClient {
    /// Connect to an HTTP/2 endpoint with bounded encoding and decoding.
    pub async fn connect(endpoint: String, token: &str, timeout: core::time::Duration, max_message_bytes: usize) -> Result<Self, GrpcClientError> {
        let configuration = (max_message_bytes > 0)
            .then_some(())
            .ok_or_else(|| GrpcClientError::Configuration("message bound must be positive".to_string()))
            .and_then(|()| reqwest::Url::parse(&endpoint).map_err(|error| GrpcClientError::Configuration(error.to_string())))
            .and_then(|parsed| {
                let loopback = parsed.host_str().is_some_and(|host| {
                    host.eq_ignore_ascii_case("localhost") || host.parse::<core::net::IpAddr>().is_ok_and(|address| address.is_loopback())
                });
                (matches!(parsed.scheme(), "http" | "https") && (parsed.scheme() != "http" || loopback))
                    .then_some(())
                    .ok_or_else(|| GrpcClientError::Configuration("use HTTPS, or HTTP with an explicit loopback host".to_string()))
            })
            .and_then(|()| {
                format!("Bearer {token}")
                    .parse()
                    .map_err(|error: tonic::metadata::errors::InvalidMetadataValue| GrpcClientError::Configuration(error.to_string()))
            });
        match configuration {
            | Err(error) => Err(error),
            | Ok(authorization) => match Endpoint::from_shared(endpoint).map_err(|error| GrpcClientError::Configuration(error.to_string())) {
                | Err(error) => Err(error),
                | Ok(endpoint) => endpoint.timeout(timeout).connect().await.map_err(Into::into).map(|channel| {
                    let inner = proto::acorn_service_client::AcornServiceClient::new(channel)
                        .max_decoding_message_size(max_message_bytes)
                        .max_encoding_message_size(max_message_bytes);
                    Self { inner, authorization }
                }),
            },
        }
    }
    /// Validate supplied research activity data.
    pub async fn validate_research_activity(
        &mut self,
        request: ValidateResearchActivityRequest,
    ) -> Result<ValidateResearchActivityResponse, GrpcClientError> {
        let mut request = Request::new(proto::ValidateResearchActivityRequest {
            content: request.content,
            format: wire_mime(&request.format) as i32,
            source_uri: request.source_uri,
            vocabulary_fuzzy: wire_vocabulary_fuzzy(request.vocabulary_fuzzy) as i32,
        });
        self.authenticate(&mut request);
        self.inner
            .validate_research_activity(request)
            .await
            .map_err(Into::into)
            .and_then(|response| {
                let response = response.into_inner();
                response
                    .diagnostics
                    .into_iter()
                    .map(|diagnostic| {
                        check_severity(diagnostic.severity).map(|severity| Diagnostic {
                            code: diagnostic.code,
                            message: diagnostic.message,
                            path: diagnostic.path,
                            severity,
                        })
                    })
                    .collect::<Result<Vec<_>, GrpcClientError>>()
                    .and_then(|diagnostics| {
                        response
                            .vocabulary_fuzzy
                            .map(response_vocabulary_fuzzy)
                            .transpose()
                            .map(|vocabulary_fuzzy| ValidateResearchActivityResponse {
                                diagnostics,
                                valid: response.valid,
                                vocabulary_fuzzy,
                            })
                    })
            })
    }
    /// Check readability of supplied text.
    pub async fn check_readability(&mut self, request: CheckReadabilityRequest) -> Result<CheckReadabilityResponse, GrpcClientError> {
        let mut request = Request::new(proto::CheckReadabilityRequest {
            metric: wire_metric(request.metric) as i32,
            source_uri: request.source_uri,
            text: request.text,
            threshold: request.threshold,
        });
        self.authenticate(&mut request);
        self.inner.check_readability(request).await.map_err(Into::into).and_then(|response| {
            let response = response.into_inner();
            metric(response.metric)
                .map_err(|error| GrpcClientError::Configuration(error.message().to_string()))
                .map(|metric| CheckReadabilityResponse {
                    acceptable: response.acceptable,
                    metric,
                    source_uri: response.source_uri,
                    threshold: response.threshold,
                    value: response.value,
                })
        })
    }
    /// Process a supplied repository snapshot.
    pub async fn process_repository(&mut self, request: ProcessRepositoryRequest) -> Result<ProcessRepositoryResponse, GrpcClientError> {
        let mut request = Request::new(proto::ProcessRepositoryRequest {
            actions: request.actions.into_iter().map(wire_action).map(|value| value as i32).collect(),
            files: request
                .files
                .into_iter()
                .map(|file| proto::RepositoryFileInput {
                    content: file.content,
                    format: file.format.as_ref().map(wire_mime).map(|value| value as i32),
                    path: file.path,
                })
                .collect(),
            project: request.project,
            revision: request.revision,
        });
        self.authenticate(&mut request);
        self.inner.process_repository(request).await.map_err(Into::into).and_then(|response| {
            let response = response.into_inner();
            let files = response
                .files
                .into_iter()
                .map(|file| {
                    file_change_kind(file.kind).map(|kind| FileChange {
                        content: file.content,
                        kind,
                        path: file.path,
                    })
                })
                .collect::<Result<Vec<_>, GrpcClientError>>();
            let reports = response
                .reports
                .into_iter()
                .map(|report| {
                    mime(report.format)
                        .map_err(|error| GrpcClientError::Configuration(error.message().to_string()))
                        .map(|format| WorkflowFileReport {
                            changed: report.changed,
                            format,
                            path: report.path,
                        })
                })
                .collect::<Result<Vec<_>, GrpcClientError>>();
            files.and_then(|files| {
                reports.map(|reports| ProcessRepositoryResponse {
                    base_revision: response.base_revision,
                    files,
                    reports,
                })
            })
        })
    }
    fn authenticate<T>(&self, request: &mut Request<T>) {
        request.metadata_mut().insert(AUTHORIZATION, self.authorization.clone());
    }
}
impl GrpcService {
    /// Construct a typed authenticated adapter.
    pub fn new(service: AcornService, token: impl Into<Arc<str>>) -> Self {
        Self {
            service,
            token: token.into(),
        }
    }
    fn authenticate<T>(&self, request: &Request<T>) -> Result<(), Status> {
        let authenticated = request
            .metadata()
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .is_some_and(|received| constant_time_eq(received.as_bytes(), self.token.as_bytes()));
        authenticated
            .then_some(())
            .ok_or_else(|| Status::unauthenticated("A valid gRPC bearer token is required"))
    }
}
#[tonic::async_trait]
impl proto::acorn_service_server::AcornService for GrpcService {
    #[tracing::instrument(name = "acorn.grpc", skip_all, fields(protocol = "grpc", operation = "ValidateResearchActivity"))]
    async fn validate_research_activity(
        &self,
        request: Request<proto::ValidateResearchActivityRequest>,
    ) -> Result<Response<proto::ValidateResearchActivityResponse>, Status> {
        self.authenticate(&request).and_then(|()| {
            let request = request.into_inner();
            mime(request.format).and_then(|format| {
                vocabulary_fuzzy(request.vocabulary_fuzzy).and_then(|vocabulary_fuzzy| {
                    self.service
                        .validate_research_activity(ValidateResearchActivityRequest {
                            content: request.content,
                            format,
                            source_uri: request.source_uri,
                            vocabulary_fuzzy,
                        })
                        .map(|response| {
                            Response::new(proto::ValidateResearchActivityResponse {
                                diagnostics: response
                                    .diagnostics
                                    .into_iter()
                                    .map(|diagnostic| proto::Diagnostic {
                                        code: diagnostic.code,
                                        message: diagnostic.message,
                                        path: diagnostic.path,
                                        severity: severity(diagnostic.severity) as i32,
                                    })
                                    .collect(),
                                valid: response.valid,
                                vocabulary_fuzzy: response.vocabulary_fuzzy.map(wire_vocabulary_fuzzy).map(|value| value as i32),
                            })
                        })
                        .map_err(status)
                })
            })
        })
    }
    #[tracing::instrument(name = "acorn.grpc", skip_all, fields(protocol = "grpc", operation = "CheckReadability"))]
    async fn check_readability(&self, request: Request<proto::CheckReadabilityRequest>) -> Result<Response<proto::CheckReadabilityResponse>, Status> {
        self.authenticate(&request).and_then(|()| {
            let request = request.into_inner();
            metric(request.metric).and_then(|metric| {
                self.service
                    .check_readability(CheckReadabilityRequest {
                        metric,
                        source_uri: request.source_uri,
                        text: request.text,
                        threshold: request.threshold,
                    })
                    .map(|response| {
                        Response::new(proto::CheckReadabilityResponse {
                            acceptable: response.acceptable,
                            metric: wire_metric(response.metric) as i32,
                            source_uri: response.source_uri,
                            threshold: response.threshold,
                            value: response.value,
                        })
                    })
                    .map_err(status)
            })
        })
    }
    #[tracing::instrument(name = "acorn.grpc", skip_all, fields(protocol = "grpc", operation = "ProcessRepository"))]
    async fn process_repository(
        &self,
        request: Request<proto::ProcessRepositoryRequest>,
    ) -> Result<Response<proto::ProcessRepositoryResponse>, Status> {
        self.authenticate(&request).and_then(|()| {
            let request = request.into_inner();
            request
                .actions
                .into_iter()
                .map(action)
                .collect::<Result<Vec<_>, _>>()
                .and_then(|actions| {
                    request
                        .files
                        .into_iter()
                        .map(|file| {
                            file.format.map(mime).transpose().map(|format| RepositoryFileInput {
                                content: file.content,
                                format,
                                path: file.path,
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .map(|files| (actions, files))
                })
                .and_then(|(actions, files)| {
                    self.service
                        .process_repository(ProcessRepositoryRequest {
                            actions,
                            files,
                            project: request.project,
                            revision: request.revision,
                        })
                        .map_err(status)
                })
                .map(|response| {
                    Response::new(proto::ProcessRepositoryResponse {
                        base_revision: response.base_revision,
                        files: response
                            .files
                            .into_iter()
                            .map(|file| proto::FileChange {
                                content: file.content,
                                kind: match file.kind {
                                    | FileChangeKind::Create => proto::FileChangeKind::Create,
                                    | FileChangeKind::Update => proto::FileChangeKind::Update,
                                } as i32,
                                path: file.path,
                            })
                            .collect(),
                        reports: response
                            .reports
                            .into_iter()
                            .map(|report| proto::WorkflowFileReport {
                                changed: report.changed,
                                format: wire_mime(&report.format) as i32,
                                path: report.path,
                            })
                            .collect(),
                    })
                })
        })
    }
}
fn action(value: i32) -> Result<WorkflowAction, Status> {
    match proto::RepositoryAction::try_from(value).unwrap_or(proto::RepositoryAction::Unspecified) {
        | proto::RepositoryAction::Check => Ok(WorkflowAction::Check),
        | proto::RepositoryAction::Enrich => Ok(WorkflowAction::Enrich),
        | proto::RepositoryAction::Format => Ok(WorkflowAction::Format),
        | proto::RepositoryAction::Link => Ok(WorkflowAction::Link),
        | proto::RepositoryAction::Validate => Ok(WorkflowAction::Validate),
        | proto::RepositoryAction::Unspecified => Err(Status::invalid_argument("Repository action is required")),
    }
}
fn check_severity(value: i32) -> Result<CheckSeverity, GrpcClientError> {
    match proto::Severity::try_from(value).unwrap_or(proto::Severity::Unspecified) {
        | proto::Severity::Error => Ok(CheckSeverity::Error),
        | proto::Severity::Info => Ok(CheckSeverity::Info),
        | proto::Severity::Suggestion => Ok(CheckSeverity::Suggestion),
        | proto::Severity::Warning => Ok(CheckSeverity::Warning),
        | proto::Severity::Unspecified => Err(GrpcClientError::Configuration("response severity is unspecified".to_string())),
    }
}
fn file_change_kind(value: i32) -> Result<FileChangeKind, GrpcClientError> {
    match proto::FileChangeKind::try_from(value).unwrap_or(proto::FileChangeKind::Unspecified) {
        | proto::FileChangeKind::Create => Ok(FileChangeKind::Create),
        | proto::FileChangeKind::Update => Ok(FileChangeKind::Update),
        | proto::FileChangeKind::Unspecified => Err(GrpcClientError::Configuration("response file-change kind is unspecified".to_string())),
    }
}
fn metric(value: i32) -> Result<ReadabilityType, Status> {
    match proto::ReadabilityMetric::try_from(value).unwrap_or(proto::ReadabilityMetric::Unspecified) {
        | proto::ReadabilityMetric::Ari => Ok(ReadabilityType::ARI),
        | proto::ReadabilityMetric::Cli => Ok(ReadabilityType::CLI),
        | proto::ReadabilityMetric::Fkgl => Ok(ReadabilityType::FKGL),
        | proto::ReadabilityMetric::Fres => Ok(ReadabilityType::FRES),
        | proto::ReadabilityMetric::Gfi => Ok(ReadabilityType::GFI),
        | proto::ReadabilityMetric::Lix => Ok(ReadabilityType::Lix),
        | proto::ReadabilityMetric::Smog => Ok(ReadabilityType::SMOG),
        | proto::ReadabilityMetric::Unspecified => Err(Status::invalid_argument("Readability metric is required")),
    }
}
fn mime(value: i32) -> Result<MimeType, Status> {
    match proto::DataFormat::try_from(value).unwrap_or(proto::DataFormat::Unspecified) {
        | proto::DataFormat::Cff => Ok(MimeType::Cff),
        | proto::DataFormat::Json => Ok(MimeType::Json),
        | proto::DataFormat::Jsonc => Ok(MimeType::Jsonc),
        | proto::DataFormat::Markdown => Ok(MimeType::Markdown),
        | proto::DataFormat::Yaml => Ok(MimeType::Yaml),
        | proto::DataFormat::Zon => Ok(MimeType::Zon),
        | proto::DataFormat::Unspecified => Err(Status::invalid_argument("Data format is required")),
    }
}
fn response_vocabulary_fuzzy(value: i32) -> Result<VocabularyFuzzyMode, GrpcClientError> {
    proto::VocabularyFuzzyMode::try_from(value)
        .map_err(|_| GrpcClientError::Configuration("response vocabulary-fuzzy mode is unknown".to_string()))
        .and_then(|value| match value {
            | proto::VocabularyFuzzyMode::Off => Ok(VocabularyFuzzyMode::Off),
            | proto::VocabularyFuzzyMode::Relaxed => Ok(VocabularyFuzzyMode::Relaxed),
            | proto::VocabularyFuzzyMode::Strict => Ok(VocabularyFuzzyMode::Strict),
            | proto::VocabularyFuzzyMode::Unspecified => Err(GrpcClientError::Configuration(
                "response vocabulary-fuzzy mode is unspecified".to_string(),
            )),
        })
}
fn severity(value: CheckSeverity) -> proto::Severity {
    match value {
        | CheckSeverity::Error => proto::Severity::Error,
        | CheckSeverity::Info => proto::Severity::Info,
        | CheckSeverity::Suggestion => proto::Severity::Suggestion,
        | CheckSeverity::Warning => proto::Severity::Warning,
    }
}
fn status(error: ServiceError) -> Status {
    match error.kind {
        | ServiceErrorKind::InvalidInput | ServiceErrorKind::Unsupported => Status::invalid_argument(error.to_string()),
        | ServiceErrorKind::PolicyDenied => Status::permission_denied(error.to_string()),
        | ServiceErrorKind::PayloadBounds => Status::resource_exhausted(error.to_string()),
        | ServiceErrorKind::DurableOperationsUnavailable => Status::unavailable(error.to_string()),
        | ServiceErrorKind::Internal => Status::internal("Internal service failure"),
    }
}
fn vocabulary_fuzzy(value: i32) -> Result<VocabularyFuzzyMode, Status> {
    proto::VocabularyFuzzyMode::try_from(value)
        .map_err(|_| Status::invalid_argument("Vocabulary fuzzy mode is unknown"))
        .map(|value| match value {
            | proto::VocabularyFuzzyMode::Off => VocabularyFuzzyMode::Off,
            | proto::VocabularyFuzzyMode::Relaxed | proto::VocabularyFuzzyMode::Unspecified => VocabularyFuzzyMode::Relaxed,
            | proto::VocabularyFuzzyMode::Strict => VocabularyFuzzyMode::Strict,
        })
}
fn wire_action(value: WorkflowAction) -> proto::RepositoryAction {
    match value {
        | WorkflowAction::Check => proto::RepositoryAction::Check,
        | WorkflowAction::Enrich => proto::RepositoryAction::Enrich,
        | WorkflowAction::Format => proto::RepositoryAction::Format,
        | WorkflowAction::Link => proto::RepositoryAction::Link,
        | WorkflowAction::Validate => proto::RepositoryAction::Validate,
    }
}
fn wire_metric(value: ReadabilityType) -> proto::ReadabilityMetric {
    match value {
        | ReadabilityType::ARI => proto::ReadabilityMetric::Ari,
        | ReadabilityType::CLI => proto::ReadabilityMetric::Cli,
        | ReadabilityType::FKGL => proto::ReadabilityMetric::Fkgl,
        | ReadabilityType::FRES => proto::ReadabilityMetric::Fres,
        | ReadabilityType::GFI => proto::ReadabilityMetric::Gfi,
        | ReadabilityType::Lix => proto::ReadabilityMetric::Lix,
        | ReadabilityType::SMOG => proto::ReadabilityMetric::Smog,
    }
}
fn wire_mime(value: &MimeType) -> proto::DataFormat {
    match value {
        | MimeType::Cff => proto::DataFormat::Cff,
        | MimeType::Json => proto::DataFormat::Json,
        | MimeType::Jsonc => proto::DataFormat::Jsonc,
        | MimeType::Markdown => proto::DataFormat::Markdown,
        | MimeType::Yaml => proto::DataFormat::Yaml,
        | MimeType::Zon => proto::DataFormat::Zon,
        | _ => proto::DataFormat::Unspecified,
    }
}
fn wire_vocabulary_fuzzy(value: VocabularyFuzzyMode) -> proto::VocabularyFuzzyMode {
    match value {
        | VocabularyFuzzyMode::Off => proto::VocabularyFuzzyMode::Off,
        | VocabularyFuzzyMode::Relaxed => proto::VocabularyFuzzyMode::Relaxed,
        | VocabularyFuzzyMode::Strict => proto::VocabularyFuzzyMode::Strict,
    }
}

#[cfg(test)]
mod tests;
