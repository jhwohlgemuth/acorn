//! Complete ACRE reload validation before scheduler publication.
use crate::config::{DaemonConfig, JobConfig};
use acorn_service::{ServiceError, ServiceErrorKind, ServiceResult, WorkflowAction};
use std::path::Path;

/// Load one complete replacement and reject settings requiring process restart.
/// ### Errors
/// Returns a bounded, secret-safe service error for malformed configuration, unsupported jobs, or changed listener and TLS settings.
pub fn load(path: &Path, initial: &DaemonConfig) -> ServiceResult<Vec<JobConfig>> {
    DaemonConfig::load(path)
        .map_err(|_| ServiceError::new(ServiceErrorKind::InvalidInput, "daemon configuration is invalid"))
        .and_then(|candidate| {
            let restart_only = candidate.bind() != initial.bind() || candidate.tls() != initial.tls();
            match restart_only {
                | true => Err(ServiceError::new(
                    ServiceErrorKind::Unsupported,
                    "daemon listener or TLS settings require restart",
                )),
                | false => candidate
                    .jobs()
                    .iter()
                    .find(|job| !matches!(job.kind(), WorkflowAction::Check | WorkflowAction::Gather))
                    .map(|_| Err(ServiceError::new(ServiceErrorKind::InvalidInput, "daemon job action has no fixed runner")))
                    .unwrap_or_else(|| Ok(candidate.jobs().to_vec())),
            }
        })
}
