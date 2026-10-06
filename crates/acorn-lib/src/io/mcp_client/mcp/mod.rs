//! Model Context Protocol configuration and outbound client adapter.

#[path = "../../mcp/client.rs"]
pub mod client;
#[path = "../../mcp/config.rs"]
pub mod config;

#[cfg(test)]
#[path = "../../mcp/tests/mod.rs"]
mod tests;
