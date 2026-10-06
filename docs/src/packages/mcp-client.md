# MCP client

The non-default `mcp-client` feature lets `acorn-lib` initialize configured MCP servers, discover approved tools, and invoke them through child-process or Streamable HTTP transports. The ACORN CLI enables this feature for [`acorn mcp`](../cli-reference/commands/mcp.md).

```sh
cargo add acorn-lib --no-default-features --features mcp-client
```

Application configuration owns every server transport, environment-variable reference, tool allowlist, declared effect, timeout, and output limit. The top-level `mcp` map names servers, each server's `allowed` map names tools, `timeout` is measured in seconds, and `max_output` uses a unit-bearing memory value such as `256KB`. Callers select only a configured server alias, tool name, and JSON-object arguments:

An MCP-only build exposes the `mcp` projection of `ApplicationConfiguration` at the same path and ignores unrelated top-level fields while deserializing. Enabling `analysis` exposes the complete application configuration without changing the client constructor.

```rust
use acorn::io::api::operation::InvocationContext;
use acorn::io::config::ApplicationConfiguration;
use acorn::io::mcp::client::McpClient;
use serde_json::json;

# async fn example(configuration: &ApplicationConfiguration) -> color_eyre::Result<()> {
    let client = McpClient::try_from(configuration)?;
    let result = client
        .call_tool(
            "research-tools",
            "resolve_identifier",
            json!({"identifier": "https://doi.org/10.1000/example"}),
            &InvocationContext::default(),
        )
        .await?;
    if result.is_error == Some(true) {
        // The MCP exchange succeeded and the remote tool reported an error.
    }
    Ok(())
 }
```

`list_tools` returns only tools that are configured, currently advertised, and authorized by the supplied invocation context. `call_tool` preserves the complete typed `rmcp::model::CallToolResult`; a tool-level `isError` value is not converted into a transport error.

The client and ACORN's JSON-RPC operation registry share the same transport-neutral effects policy. Credentials and process effects are unavailable; inference requires explicit inference authorization; mutations require mutation authorization; and network reads are denied offline. Streamable HTTP itself is unavailable offline. The effective result limit is the smaller of the tool configuration and caller context limits.

Sessions are intentionally one operation long. Initialization, paginated catalog discovery, requests, cancellation, output bounds, and shutdown all have fixed limits. Configuration stores environment-variable names, never resolved bearer tokens or child values. Child processes run without a shell or inherited environment, and remote HTTP requires TLS except on loopback.

The client is not a general MCP proxy. Register an ACORN operation handler explicitly when an application should call a remote tool; enabling `mcp-client` does not add remote tools to any inbound catalog or transport.
