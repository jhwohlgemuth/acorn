//! Narrow I/O surface for outbound MCP client builds.

#[cfg(feature = "acp")]
#[path = "../acp/mod.rs"]
pub mod acp;
pub mod api;
pub mod config;
pub mod mcp;

pub use crate::error::ApiResult;
