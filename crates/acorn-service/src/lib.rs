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
pub use operation::{InvocationContext, InvocationOrigin, OperationEffect, OperationEffects, Principal};
pub use registry::{Dispatch, DurableOperations, MethodName, OperationRegistry, RequestId, RpcError};
pub use service::{AcornService, Decode};

#[cfg(test)]
mod tests;
