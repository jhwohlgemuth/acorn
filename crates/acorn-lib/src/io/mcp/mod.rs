//! Model Context Protocol (MCP) configuration and optional client/server adapters.

#[cfg(feature = "agentic")]
pub mod bundle;
#[cfg(feature = "mcp-client")]
pub mod client;
pub mod config;
#[cfg(feature = "agentic")]
mod server;
#[cfg(feature = "agentic")]
mod tools;

#[cfg(feature = "agentic")]
pub use server::{run, Server};
#[cfg(feature = "agentic")]
pub use tools::{ToolCallContext, ToolRegistry};

#[cfg(test)]
mod tests;
