//! Dual-protocol composition root for the portable ACORN service.
extern crate alloc;

pub mod client;
pub mod config;
pub mod durable;
pub mod grpc;
pub mod health;
pub mod http;
pub mod runtime;

/// Generated protobuf wire model for the typed ACORN service.
pub mod proto {
    #![allow(clippy::std_instead_of_core)]
    tonic::include_proto!("acorn.service");
}
/// Generated standard gRPC health types.
pub mod health_proto {
    #![allow(clippy::std_instead_of_core)]
    tonic::include_proto!("grpc.health.v1");
}

pub use config::{ServerArgs, ServerConfig};
pub use runtime::{serve, serve_with_durable};

/// Server result type with thread-safe error context.
pub type ServerResult<T> = Result<T, Box<dyn core::error::Error + Send + Sync>>;
