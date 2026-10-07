//! Typed in-process client over the canonical service contract.
use crate::{
    AcornService, CheckReadabilityRequest, CheckReadabilityResponse, ProcessRepositoryRequest, ProcessRepositoryResponse, ServiceResult,
    ValidateResearchActivityRequest, ValidateResearchActivityResponse, ValidateStandardRequest,
};

/// Small typed client usable by embedding callers and adapter conversion tests.
#[derive(Clone, Copy, Debug, Default)]
pub struct ServiceClient {
    service: AcornService,
}
impl ServiceClient {
    /// Construct a client for one service instance.
    pub const fn new(service: AcornService) -> Self {
        Self { service }
    }
    /// Validate supplied research activity data.
    pub fn validate_research_activity(&self, request: ValidateResearchActivityRequest) -> ServiceResult<ValidateResearchActivityResponse> {
        self.service.validate_research_activity(request)
    }
    /// Validate supplied data against a selected metadata standard.
    pub fn validate_standard(&self, request: ValidateStandardRequest) -> ServiceResult<ValidateResearchActivityResponse> {
        self.service.validate_standard(request)
    }
    /// Check readability of supplied text.
    pub fn check_readability(&self, request: CheckReadabilityRequest) -> ServiceResult<CheckReadabilityResponse> {
        self.service.check_readability(request)
    }
    /// Process a supplied repository snapshot.
    pub fn process_repository(&self, request: ProcessRepositoryRequest) -> ServiceResult<ProcessRepositoryResponse> {
        self.service.process_repository(request)
    }
}
