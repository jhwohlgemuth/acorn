//! Call an allowlisted MCP tool selected by an explicit ACRE executor binding.
//!
//! From the repository root, first build the ACORN MCP server and put it on PATH:
//! `cargo build -p acorn-cli --bin acorn`
//! `PATH="$PWD/target/debug:$PATH" cargo run -p acorn-cli --example mcp_tools -- crates/acorn-cli/examples/mcp_tools.json mcp:version '{}'`
//!
//! This example uses the outbound MCP client directly. Routed MCP capability registration
//! belongs to ADP-02 and is not performed here.

use acorn::io::api::operation::InvocationContext;
use acorn::io::config::ApplicationConfiguration;
use acorn::io::mcp::client::McpClient;
use acorn::io::mcp::ExecutorConfiguration;
use acorn_schema::validation::Validate;
use color_eyre::eyre::{eyre, Result};
use serde_json::Value;
use std::path::PathBuf;

fn main() -> Result<()> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        | [config, binding_id, json_arguments] => tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(Into::into)
            .and_then(|runtime| runtime.block_on(run(config, binding_id, json_arguments))),
        | _ => Err(eyre!("Usage: mcp_tools <config.json> <binding-id> <json-arguments>")),
    }
}

async fn run(path: &str, binding_id: &str, json_arguments: &str) -> Result<()> {
    let prepared = ApplicationConfiguration::load(&Some(PathBuf::from(path)))
        .and_then(|config| {
            config
                .validate()
                .map_err(|why| eyre!("Invalid executor configuration — {why}"))
                .map(|()| config)
        })
        .and_then(|config| {
            config
                .executors
                .iter()
                .find(|binding| binding.id == binding_id)
                .ok_or_else(|| eyre!("Executor binding '{binding_id}' is not configured"))
                .and_then(|binding| match &binding.executor {
                    | ExecutorConfiguration::Mcp { server, tool } => Ok((server.clone(), tool.clone())),
                    | _ => Err(eyre!("Executor binding '{binding_id}' is not an MCP tool")),
                })
                .and_then(|selector| McpClient::try_from(&config).map(|client| (client, selector)))
        })
        .and_then(|(client, selector)| {
            serde_json::from_str::<Value>(json_arguments)
                .map_err(|why| eyre!("Invalid JSON arguments — {why}"))
                .and_then(|arguments| match arguments {
                    | Value::Object(_) => Ok((client, selector, arguments)),
                    | _ => Err(eyre!("MCP tool arguments must be a JSON object")),
                })
        });
    match prepared {
        | Err(why) => Err(why),
        | Ok((client, (server, tool), arguments)) => match client.call_tool(&server, &tool, arguments, &InvocationContext::default()).await {
            | Err(why) => Err(why),
            | Ok(result) if result.is_error == Some(true) => Err(eyre!("MCP tool '{server}/{tool}' reported an error: {result:?}")),
            | Ok(result) => serde_json::to_string_pretty(&result)
                .map_err(Into::into)
                .map(|output| println!("{output}")),
        },
    }
}
