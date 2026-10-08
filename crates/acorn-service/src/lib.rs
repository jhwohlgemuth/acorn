//! Transport-neutral ACORN application operations over caller-supplied data.
#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod client;
pub mod error;
pub mod model;
pub mod operation;
pub mod registry;
pub mod repository;
pub mod service;

pub use client::ServiceClient;
pub use error::{ServiceError, ServiceErrorKind, ServiceResult};
pub use model::{
    CheckReadabilityRequest, CheckReadabilityResponse, Diagnostic, FileChange, FileChangeKind, ProcessRepositoryRequest, ProcessRepositoryResponse,
    RepositoryFileInput, ValidateResearchActivityRequest, ValidateResearchActivityResponse, ValidateStandardRequest, WorkflowAction,
    WorkflowFileReport,
};
pub use operation::{InvocationContext, InvocationOrigin, Memory, OperationEffect, OperationEffects, Principal};
pub use registry::{
    Dispatch, DurableOperations, MethodName, Notification, OperationDefinition, OperationRegistry, RequestId, RpcError, INTERNAL_ERROR,
    INVALID_PARAMS, INVALID_REQUEST, JSON_RPC_VERSION, METHOD_NOT_FOUND, PARSE_ERROR, POLICY_DENIED,
};
pub use service::AcornService;

#[cfg(test)]
mod tests;
