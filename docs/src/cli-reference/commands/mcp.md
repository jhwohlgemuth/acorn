# MCP client

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Discover and invoke explicitly allowlisted tools on configured MCP servers with <code>acorn mcp</code>.</p>
</blockquote>

Configure a server and the tools ACORN may use:

```yaml
mcp:
  research-tools:
    transport:
      type: child-process
      command: research-mcp
      args: [serve, --stdio]
      environment:
        RESEARCH_TOKEN: RESEARCH_MCP_TOKEN
    allowed:
      resolve_identifier:
        effects: [network-read]
        max_output: 256KB
        timeout: 20
```

The `environment` mapping names a child variable on the left and a parent environment variable on the right. ACORN resolves the executable without a shell, clears the inherited child environment, and supplies only those mappings. Missing or empty variables are rejected without printing their names or values.

List the allowlisted tools the server currently advertises, then call one with a JSON object:

```sh
acorn mcp --config .acorn.yml list research-tools
acorn mcp --config .acorn.yml call research-tools resolve_identifier \
  --arguments '{"identifier":"https://doi.org/10.1000/example"}'
```

## Connect to the ACORN MCP server

The ACORN CLI can use another ACORN process as its MCP server. Configure a child-process transport that starts `acorn serve mcp`, and allowlist the built-in tools the client may use:

```yaml
mcp:
  local-acorn:
    transport:
      type: child-process
      command: acorn
      args: [serve, mcp]
    allowed:
      acorn.version:
        max_output: 4KB
        timeout: 10
```

The client starts the server automatically for each command, exchanges MCP messages over standard input and output, and closes the server when the operation finishes:

```sh
acorn mcp --config .acorn.yml list local-acorn
acorn mcp --config .acorn.yml call local-acorn acorn.version --arguments '{}'
```

The first command lists `acorn.version`. The second returns the server version in `structuredContent.version`. Replace or extend the `allowed` map to use other tools exposed by `acorn serve mcp`; tools absent from that map cannot be called.

Both commands write deterministic pretty JSON to standard output. The call output is the complete MCP result, including content, structured content, metadata, and `isError`. When a server returns `isError: true`, ACORN prints that result and exits unsuccessfully.

Streamable HTTP servers use HTTPS except for loopback development endpoints. Bearer tokens are read only from a named environment variable:

```yaml
mcp:
  institutional-tools:
    transport:
      type: streamable-http
      url: https://mcp.example.org/service
      bearer_token_env: INSTITUTIONAL_MCP_TOKEN
    allowed:
      lookup_record:
        effects: [network-read]
```

Global `--offline` rejects Streamable HTTP before any connection. Local child-process servers remain available, but tools declaring `network-read` are unavailable. Mutation, destructive, and filesystem-write effects require `--allow-mutation` on `mcp call`; credential and process effects are unavailable. Remote tool annotations are returned for display but cannot relax the configured effects policy.

Each list or call creates, initializes, and closes one MCP session. ACORN does not accept arbitrary commands, URLs, headers, or tool names from call arguments, and it does not automatically expose remote tools through its MCP server, JSON-RPC, gRPC, webhooks, or ACP.

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Embed the same policy-checked client with <a href="../../packages/mcp-client.md">the Rust MCP client API</a>.</p>
