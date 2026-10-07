//! Native compatibility wrapper over the portable repository service.
use super::repository::{RepositoryChangeSet, RepositoryInput};
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
    pub fn process(&self, name: &str, input: RepositoryInput) -> ApiResult<RepositoryChangeSet> {
        match name {
            | "repository-quality" => serde_json::to_value(input)
                .map_err(|why| eyre!("Encode repository workflow request — {why}"))
                .and_then(|value| serde_json::from_value(value).map_err(|why| eyre!("Decode repository workflow request — {why}")))
                .and_then(|request| {
                    AcornService
                        .process_repository(request)
                        .map_err(|why| eyre!("Repository workflow failed — {why}"))
                })
                .and_then(|response| serde_json::to_value(response).map_err(|why| eyre!("Encode repository workflow response — {why}")))
                .and_then(|value| serde_json::from_value(value).map_err(|why| eyre!("Decode repository workflow response — {why}"))),
            | _ => Err(eyre!("Unknown repository workflow '{name}'")),
        }
    }
}
