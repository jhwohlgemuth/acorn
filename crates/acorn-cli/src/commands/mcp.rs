//! CLI adapter for the policy-checked outbound MCP client.
use crate::cli::{McpCommands, Void};
use acorn::io::api::operation::InvocationContext;
use acorn::io::config::ApplicationConfiguration;
use acorn::io::mcp::client::McpClient;
use color_eyre::eyre::eyre;
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;

enum Command<'a> {
    Call {
        allow_mutation: bool,
        arguments: Value,
        client: McpClient,
        server: &'a str,
        tool: &'a str,
    },
    List {
        client: McpClient,
        server: &'a str,
    },
}
impl Command<'_> {
    async fn execute(self, offline: bool) -> Void {
        match self {
            | Self::Call {
                allow_mutation,
                arguments,
                client,
                server,
                tool,
            } => {
                let context = InvocationContext {
                    allow_mutation,
                    offline,
                    ..InvocationContext::default()
                };
                match client.call_tool(server, tool, arguments, &context).await {
                    | Err(why) => Err(why),
                    | Ok(result) => print_json(&result).and_then(|()| match result.is_error {
                        | Some(true) => Err(eyre!("MCP tool '{server}/{tool}' reported an error")),
                        | _ => Ok(()),
                    }),
                }
            }
            | Self::List { client, server } => match client
                .list_tools(
                    server,
                    &InvocationContext {
                        offline,
                        ..InvocationContext::default()
                    },
                )
                .await
            {
                | Ok(tools) => print_json(&tools),
                | Err(why) => Err(why),
            },
        }
    }
}
fn print_json(value: &impl Serialize) -> Void {
    serde_json::to_string_pretty(value)
        .map_err(|why| eyre!("Failed to serialize MCP output — {why}"))
        .map(|output| println!("{output}"))
}
pub(crate) async fn run(config: &Option<PathBuf>, command: &McpCommands, offline: bool) -> Void {
    let prepared = ApplicationConfiguration::load(config)
        .and_then(|configuration| McpClient::try_from(&configuration))
        .and_then(|client| match command {
            | McpCommands::Call {
                allow_mutation,
                arguments,
                server,
                tool,
            } => serde_json::from_str::<Value>(arguments)
                .map_err(|why| eyre!("MCP tool arguments are not valid JSON — {why}"))
                .map(|arguments| Command::Call {
                    allow_mutation: *allow_mutation,
                    arguments,
                    client,
                    server,
                    tool,
                }),
            | McpCommands::List { server } => Ok(Command::List { client, server }),
        });
    match prepared {
        | Ok(command) => command.execute(offline).await,
        | Err(why) => Err(why),
    }
}
