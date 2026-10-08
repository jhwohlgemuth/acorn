//! Native compatibility wrapper over the portable repository service.
use super::repository::{ProcessRepositoryRequest, ProcessRepositoryResponse};
use crate::io::ApiResult;
use acorn_service::AcornService;
use color_eyre::eyre::eyre;

/// Registry wrapper retained for native GitLab and workflow composition.
#[derive(Clone, Copy, Debug, Default)]
pub struct RepositoryWorkflowRegistry;
impl RepositoryWorkflowRegistry {
    /// Build the conservative built-in repository workflow registry.
    pub fn acorn() -> ApiResult<Self> {
        Ok(Self)
    }
    /// Return registered workflow names in deterministic order.
    pub fn names(&self) -> Vec<String> {
        vec!["repository-quality".to_string()]
    }
    /// Process a repository snapshot through the portable service.
    pub fn process(&self, name: &str, input: ProcessRepositoryRequest) -> ApiResult<ProcessRepositoryResponse> {
        match name {
            | "repository-quality" => AcornService
                .process_repository(input)
                .map_err(|why| eyre!("Repository workflow failed — {why}")),
            | _ => Err(eyre!("Unknown repository workflow '{name}'")),
        }
    }
}
