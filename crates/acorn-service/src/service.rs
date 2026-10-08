//! Canonical application service implementation.
use crate::model::{
    CheckReadabilityRequest, CheckReadabilityResponse, Diagnostic, ProcessRepositoryRequest, ProcessRepositoryResponse,
    ValidateResearchActivityRequest, ValidateResearchActivityResponse, ValidateStandardRequest,
};
use crate::repository::process_repository;
use crate::{ServiceError, ServiceErrorKind, ServiceResult};
use acorn_analysis::readability::score::ReadabilityScore;
use acorn_analysis::CheckSeverity;
use acorn_core::prelude::{format, ToString, Vec};
use acorn_core::util::MimeType;
use acorn_schema::codec::{CodecDiagnostic, DocumentCodec};
use acorn_schema::modelkit::kitfile::Kitfile;
use acorn_schema::pid::raid;
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::standard::{cff::Cff, ckan, datacite, invenio, text, Standard};
#[cfg(feature = "std")]
use acorn_schema::standard::{dcat, huwise};
use acorn_schema::validation::{Validate, ValidationReport};

/// Stateless portable ACORN application service.
#[derive(Clone, Copy, Debug, Default)]
pub struct AcornService;
impl AcornService {
    /// Validate caller-supplied research activity data and return every diagnostic.
    pub fn validate_research_activity(&self, request: ValidateResearchActivityRequest) -> ServiceResult<ValidateResearchActivityResponse> {
        validate_decoded::<ResearchActivity>(&request.content, &request.format, request.source_uri.as_deref())
    }
    /// Validate caller-supplied data against a selected portable metadata standard.
    pub fn validate_standard(&self, request: ValidateStandardRequest) -> ServiceResult<ValidateResearchActivityResponse> {
        let source = request.source_uri.as_deref();
        match request.standard {
            | Standard::CitationFileFormat => validate_decoded::<Cff>(&request.content, &request.format, source),
            | Standard::Ckan => validate_decoded::<ckan::Package>(&request.content, &request.format, source),
            | Standard::Datacite => validate_decoded::<datacite::Record>(&request.content, &request.format, source),
            #[cfg(feature = "std")]
            | Standard::Dcat => validate_decoded::<dcat::Dataset>(&request.content, &request.format, source),
            | Standard::Document | Standard::Text => validate_decoded::<text::Text>(&request.content, &request.format, source),
            #[cfg(feature = "std")]
            | Standard::Huwise => validate_decoded::<huwise::Dataset>(&request.content, &request.format, source),
            | Standard::Invenio => validate_decoded::<invenio::Record>(&request.content, &request.format, source),
            | Standard::Kitfile | Standard::ModelKit => validate_decoded::<Kitfile>(&request.content, &request.format, source),
            | Standard::Raid => validate_decoded::<raid::Metadata>(&request.content, &request.format, source),
            | Standard::ResearchActivityData => validate_decoded::<ResearchActivity>(&request.content, &request.format, source),
            | standard => Err(ServiceError::new(
                ServiceErrorKind::Unsupported,
                format!("Validation is not available for '{standard}' at the portable service boundary"),
            )),
        }
    }
    /// Calculate and assess one readability metric over supplied text.
    pub fn check_readability(&self, request: CheckReadabilityRequest) -> ServiceResult<CheckReadabilityResponse> {
        let threshold = request.threshold.unwrap_or_else(|| request.metric.maximum_allowed());
        match (request.text.trim().is_empty(), threshold.is_finite()) {
            | (true, _) => Err(ServiceError::new(ServiceErrorKind::InvalidInput, "Readability text cannot be empty").with_path("text")),
            | (false, false) => Err(ServiceError::new(ServiceErrorKind::InvalidInput, "Readability threshold must be finite").with_path("threshold")),
            | (false, true) => {
                let score = ReadabilityScore::new(request.metric, request.metric.calculate(&request.text), threshold);
                Ok(CheckReadabilityResponse {
                    acceptable: !score.is_excessive(),
                    metric: score.metric(),
                    source_uri: request.source_uri,
                    threshold: score.threshold(),
                    value: score.value(),
                })
            }
        }
    }
    /// Process a supplied repository snapshot without reading or writing the host filesystem.
    pub fn process_repository(&self, request: ProcessRepositoryRequest) -> ServiceResult<ProcessRepositoryResponse> {
        process_repository(request)
    }
}
impl From<CodecDiagnostic> for Diagnostic {
    fn from(diagnostic: CodecDiagnostic) -> Self {
        Diagnostic {
            code: diagnostic.code,
            message: diagnostic.message,
            path: diagnostic.path,
            severity: CheckSeverity::Error,
        }
    }
}
fn validate_decoded<T: DocumentCodec + Validate>(
    content: &str,
    format: &MimeType,
    source: Option<&str>,
) -> ServiceResult<ValidateResearchActivityResponse> {
    T::decode(content, format)
        .map_err(|diagnostics| diagnostics.into_iter().map(Diagnostic::from).collect())
        .map_err(|diagnostics| ValidateResearchActivityResponse {
            diagnostics: with_source(diagnostics, source),
            valid: false,
        })
        .and_then(|value| {
            value.validate().map_err(|report| ValidateResearchActivityResponse {
                diagnostics: with_source(validation_diagnostics(report), source),
                valid: false,
            })
        })
        .map(|()| ValidateResearchActivityResponse {
            diagnostics: Vec::new(),
            valid: true,
        })
        .or_else(Ok)
}
fn validation_diagnostics(report: ValidationReport) -> Vec<Diagnostic> {
    report
        .into_issues()
        .into_iter()
        .map(|issue| Diagnostic {
            code: issue.error.code,
            message: issue.error.message.unwrap_or_else(|| "Validation failed".to_string()),
            path: (!issue.path.is_empty()).then_some(issue.path),
            severity: CheckSeverity::Error,
        })
        .collect()
}
fn with_source(diagnostics: Vec<Diagnostic>, source: Option<&str>) -> Vec<Diagnostic> {
    diagnostics
        .into_iter()
        .map(|diagnostic| Diagnostic {
            path: match (source, diagnostic.path) {
                | (Some(source), Some(path)) => Some(format!("{source}:{path}")),
                | (Some(source), None) => Some(source.to_string()),
                | (None, path) => path,
            },
            ..diagnostic
        })
        .collect()
}
