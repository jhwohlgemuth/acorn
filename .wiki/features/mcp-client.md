---
type: Plan
title: "Add an allowlisted MCP client capability"
description: "Connect to configured MCP servers through rmcp and expose approved remote tools to ACORN operation handlers."
difficulty: high
impact: medium
package: [acorn-lib, acorn-cli]
tags: [feature, mcp, json-rpc, agent, tools, client]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/mcp/{mod,tools}.rs`, workspace dependencies/features, MCP CLI commands/tests, documentation, and generated help.
- **Create:** `crates/acorn-lib/src/io/mcp/{client,config}.rs` plus fake MCP server and interoperability fixtures.
- **Delete:** none. Retain `rmcp` as the MCP wire and lifecycle implementation.

# Dependencies

This is third and final among the remaining plans in the [highest-priority RPC and automation lane](../parallel-execution.md#highest-priority-lane-execution-order). It consumes the implemented operation registry in `io::api::json_rpc` and follows [ACP inference](opencode-acp-inference.md), but neither protocol depends on the other's wire types.

# Goal

Allow ACORN to initialize configured MCP servers, negotiate capabilities, list tools, and invoke explicitly approved tools through `rmcp`. Make approved remote tools available to operation handlers without automatically exposing them to JSON-RPC, gRPC, webhooks, ACP, or other MCP servers.

# Decisions

- Enable only required `rmcp` client, child-process, and Streamable HTTP features behind a non-default MCP-client capability.
- Require explicit server and tool allowlists plus declared effects, timeout, output bounds, offline policy, and mutation policy.
- Support configured child-process and Streamable HTTP servers.
- A GitLab workflow may call an MCP tool only when deployment configuration names the server/tool and authorizes its effects and data boundary.
- Keep credentials, authorization headers, raw environments, and unbounded tool output out of persistence and diagnostics.

# Configuration and invocation example

```yaml
mcp_clients:
  research-tools:
    transport:
      command: "research-mcp"
      args: ["serve", "--stdio"]
    allow_tools:
      resolve_identifier:
        effects: [network-read]
        timeout_seconds: 20
        max_output_bytes: 262144
```

```rust
let result = client
    .call_tool(
        "research-tools",
        "resolve_identifier",
        json!({"identifier": "https://doi.org/10.1000/example"}),
        &invocation_context,
    )
    .await?;
```

The configuration key, server, tool, effects, timeout, and output bound are all resolved before RMCP sends the request. The caller cannot supply a command, transport URL, credential, or undeclared tool through operation parameters.

# Plan conflicts

| Plan or lane | Conflict | Resolution |
|---|---|---|
| ACP inference | `io/mcp`, agent configuration, features, and CLI roots | One agent-tools owner; ACP retains the official ACP SDK and MCP retains `rmcp`. |
| JSON-RPC | Shared handlers and error concepts | Share operation policy below transports, not protocol messages. |
| Dependency defaults | Tokio/process/HTTP/TLS feature growth | Add an independent non-default capability and inspect feature trees. |
| GitLab automation | Repository workflows may consume remote tools | Integrate only after allowlist/effects policy passes. |

# Validation

- Fake-server tests cover initialize, version/capability negotiation, list/call, structured errors, cancellation, timeout, malformed messages, and early exit.
- Test child cleanup, Streamable HTTP auth, offline rejection, mutation denial, output bounds, and redaction.
- Verify advertised but unallowlisted tools cannot run and are not re-exported automatically.
- Run independent MCP server/client/combined feature checks, lint, tests, audit, and portable graphs.

# Acceptance criteria

- ACORN can call allowlisted tools on configured MCP servers through supported transports.
- Remote tool invocation receives the same operation-effects policy as local handlers.
- Existing MCP server and MCPB behavior remains compatible.
- MCP client dependencies remain absent from portable and default-library graphs unless explicitly enabled.

# Next plan

This plan completes the highest-priority RPC and automation lane. After its acceptance criteria pass, return to the wiki's next scheduled work with [analyzer composition and traversal](../refactors/analyzer/analyzer-composition.md).

# Citations

- [MCP base protocol](https://modelcontextprotocol.io/specification/2025-06-18/basic)
- [MCP transports](https://modelcontextprotocol.io/specification/2025-06-18/basic/transports)
