# MCP client

The non-default `mcp-client` feature lets `acorn-lib` initialize configured MCP servers, discover approved tools, and invoke them through child-process or Streamable HTTP transports. The ACORN CLI enables this feature for [`acorn mcp`](../cli-reference/commands/mcp.md).

```sh
cargo add acorn-lib --no-default-features --features mcp-client
cargo add serde_json
```

Application configuration owns every server transport, environment-variable reference, tool allowlist, declared effect, timeout, and output limit. The top-level `mcp` map names servers, each server's `allowed` map names tools, `timeout` is measured in seconds, and `max_output` uses a unit-bearing memory value such as `256KB`. Callers select only a configured server alias, tool name, and JSON-object arguments.

An MCP-only build exposes the `mcp` projection of `ApplicationConfiguration` at the same path and ignores unrelated top-level fields while deserializing. Enabling `analysis` exposes the complete application configuration without changing the client constructor.

```rust
use acorn::error::ApiResult;
use acorn::io::api::operation::InvocationContext;
use acorn::io::config::ApplicationConfiguration;
use acorn::io::mcp::client::McpClient;
use serde_json::json;

# async fn call_from_application(configuration: &ApplicationConfiguration) -> ApiResult<()> {
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
# }
```

Embedders that already own server configuration do not need to construct an `ApplicationConfiguration`. The client also accepts a borrowed `BTreeMap<String, McpServerConfiguration>` directly:

```rust
use acorn::error::ApiResult;
use acorn::io::mcp::client::McpClient;
use acorn::io::mcp::config::McpServerConfiguration;
use std::collections::BTreeMap;

# fn direct_client() -> ApiResult<McpClient> {
    let servers: BTreeMap<String, McpServerConfiguration> =
        serde_json::from_value(serde_json::json!({
            "research-tools": {
                "allowed": {
                    "resolve_identifier": {
                        "effects": ["network-read"],
                        "max_output": "256KB",
                        "timeout": 30
                    }
                },
                "transport": {
                    "type": "streamable-http",
                    "url": "https://research.example.org/mcp"
                }
            }
        }))?;
    McpClient::try_from(&servers)
# }
```

`McpClientBackend` is the static-dispatch contract for applications that accept the concrete client or another backend without a trait object. Its associated types preserve each backend's context, tool, result, error, cancellation, and future types:

```rust
use acorn::io::mcp::client::McpClientBackend;

async fn discover<B>(
    backend: &B,
    server: &str,
    context: &B::Context,
) -> Result<Vec<B::Tool>, B::Error>
where
    B: McpClientBackend,
{
    backend.list_tools(server, context, None).await
}
```

The inherent methods omit cancellation by default. To stop an operation when a caller-owned future resolves, use `list_tools_with_cancellation` or `call_tool_with_cancellation`:

```rust
use acorn::error::ApiResult;
use acorn::io::api::operation::InvocationContext;
use acorn::io::mcp::client::McpClient;
use core::future::Future;

async fn cancellable_call(
    client: &McpClient,
    cancelled: impl Future<Output = ()> + Send,
) -> ApiResult<()> {
    let result = client
        .call_tool_with_cancellation(
            "research-tools",
            "resolve_identifier",
            serde_json::json!({"identifier": "https://doi.org/10.1000/example"}),
            &InvocationContext::default(),
            cancelled,
        )
        .await?;
    if result.is_error == Some(true) {
        // The MCP exchange succeeded and the remote tool reported an error.
    }
    Ok(())
}
```

Cancellation covers initialization, discovery, and tool calls. Before the future resolves, the client bounds protocol shutdown and reaps a child-process transport, killing it if graceful exit does not finish in time. Streamable HTTP requests are dropped during the same bounded cleanup path, and the immutable client remains reusable for later operations.

`list_tools` returns only tools that are configured, currently advertised, and authorized by the supplied invocation context. `call_tool` preserves the complete typed `rmcp::model::CallToolResult`; a tool-level `isError` value remains a successful MCP exchange and is not converted into a transport error. Inspect `CallToolResult.is_error` separately from handling the outer `Result`.

The client and ACORN's JSON-RPC operation registry share the same transport-neutral effects policy. Credentials and process effects are unavailable; inference requires explicit inference authorization; mutations require mutation authorization; and network reads are denied offline. Streamable HTTP itself is unavailable offline. The effective result limit is the smaller of the tool configuration and caller context limits.

Sessions are intentionally one operation long. Initialization, paginated catalog discovery, requests, cancellation, output bounds, and shutdown all have fixed limits. Configuration stores environment-variable names, never resolved bearer tokens or child values. Child processes run without a shell or inherited environment, and remote HTTP requires TLS except on loopback.

The client is not a general MCP proxy. Register an ACORN operation handler explicitly when an application should call a remote tool; enabling `mcp-client` does not add remote tools to any inbound catalog or transport.
