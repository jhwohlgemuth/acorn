---
okf_version: "0.1"
type: Plan
title: Refine the outbound MCP client interface for embedders
description: Add direct server-map construction and cancellation-aware operations to the narrow outbound MCP client without changing its policy or transport contract.
difficulty: medium
impact: high
package: [acorn-lib]
tags: [refactor, mcp, api, cancellation, embedding, interoperability]
status: planned
---

# Goal

Make the outbound MCP client usable by applications that already own their
configuration and cancellation lifecycle. An embedder must be able to construct
`McpClient` from the MCP server map, call or list tools with a caller-provided
asynchronous cancellation signal, and rely on bounded session cleanup.

Preserve the current allowlist, effect, offline, timeout, output-limit,
authentication, transport, discovery, and `CallToolResult` behavior.

# Current evidence

- ACORN 0.4.0 defines public `McpServerConfiguration`,
  `McpToolConfiguration`, and `McpTransportConfiguration` types. The client
  resolves immutable server and tool state and opens a fresh child-process or
  Streamable HTTP session for each operation.
- At commit `3b0fd004f6e5`, `McpClient` only implements
  `TryFrom<&ApplicationConfiguration>`. A library consumer with an MCP-only
  configuration must fabricate ACORN's complete application root.
- `call_tool` and `list_tools` accept `InvocationContext` but no external
  cancellation signal. Tool requests use RMCP cancellable requests and internal
  deadlines. Normal paths call `close_with_timeout`, but cancellation caused by
  dropping the outer future has no documented cleanup guarantee.
- `call_tool` returns a valid `CallToolResult` as `Ok`, including results with
  `is_error == Some(true)`. The CLI converts that semantic error into its own
  command failure after printing the result. This distinction is useful to
  embedders and must remain unchanged.
- `mcp-client` now compiles independently through its narrow configuration and
  transport facade without selecting the broad `analysis` feature.
- [Xylem's tenant-tool plan](https://code.ornl.gov/research-enablement/xylem/-/blob/main/.wiki/features/mcp-tenant-tools.md)
  is the first concrete downstream consumer. It needs the direct constructor
  and bounded cancellation without an ACORN-specific runtime token or a local
  fork.

# Dependencies and ownership

The prerequisite dependency-capability work has landed. The accepted
`mcp-client` graph uses foundational `std` support while excluding `analysis`,
Polars, and OpenSSL and retaining child-process and Streamable HTTP behavior.
Workspace manifests, crate-root gates, and dependency-deny checks remain
outside this plan's write set.

This plan owns the public client methods, their implementation, focused client
tests, and MCP client documentation. It does not change manifests, application
configuration syntax, operation effects, transport options, or CLI syntax. If
implementation requires another dependency or feature edge, create a focused
prerequisite change instead of widening this plan.

# Public interface

Add direct construction from the existing ordered server map:

```rust
impl TryFrom<&BTreeMap<String, McpServerConfiguration>> for McpClient
```

The implementation validates server aliases and configurations, resolves
executables and configured secret environment values, and produces the same
immutable internal representation as the current application constructor.
`TryFrom<&ApplicationConfiguration>` delegates to this implementation so the
two entry points cannot drift.

This is a runtime constructor. Pure planning code should call validation on the
configuration types instead; it should not use `McpClient` to probe executables
or read secrets.

Add cancellation-aware variants while retaining the existing methods:

```rust
pub async fn call_tool_with_cancellation<C>(
    &self,
    server_name: &str,
    tool_name: &str,
    arguments: Value,
    context: &InvocationContext,
    cancellation: C,
) -> ApiResult<CallToolResult>
where
    C: Future<Output = ()> + Send;

pub async fn list_tools_with_cancellation<C>(
    &self,
    server_name: &str,
    context: &InvocationContext,
    cancellation: C,
) -> ApiResult<Vec<Tool>>
where
    C: Future<Output = ()> + Send;
```

`call_tool` and `list_tools` delegate with `core::future::pending()`. Existing
callers keep their source and behavior compatibility. The generic future keeps
the client independent of `acorn-cmd`, `tokio-util`, or another public token
type. Callers may adapt a notification, channel, atomic watcher, or runtime
token at their composition boundary.

The client remains immutable and safe to share. Add a compile-time assertion
that `McpClient` is `Send + Sync` because downstream schedulers may issue fresh
sessions from different worker threads.

# Cancellation contract

Pin the supplied future once and pass a mutable pinned reference through
initialization, catalog pagination, tool invocation, and shutdown. Use biased
selection so an observed cancellation wins a race with a simultaneously ready
operation result.

Apply these rules:

- Cancellation that is ready before transport work prevents executable spawn
  and HTTP connection setup.
- Cancellation during initialization drops the handshake and closes or reaps
  the transport. A child-process guard remains armed until ownership has moved
  into a session with an explicit close path.
- Cancellation during catalog or tool requests drops the pending RMCP request,
  allowing RMCP to send protocol cancellation where supported, then runs the
  same bounded session close used by normal completion.
- Cancellation during paginated discovery stops before another page request.
- After cancellation, do not return a late tool result. Return an `ApiResult`
  error identifying the server and operation as cancelled.
- Preserve cancellation as the primary error. Append bounded cleanup detail if
  graceful close also fails or times out.
- Finish cleanup before the public future resolves. Child processes must be
  reaped, and HTTP session resources must be released within the existing
  shutdown bound.
- A cancelled operation does not poison the immutable client. A later call
  starts a new session and may succeed.

Keep the existing request timeout and control timeout independent of caller
cancellation. Timeouts remain timeout failures; cancellation is reported only
when the supplied signal wins.

# Internal design

Factor the duplicated call and list lifecycle into the smallest private session
helpers needed to share cancellation and finish behavior. Keep configuration
resolution, policy authorization, transport construction, catalog filtering,
and result-size checks in their present ownership.

The normal operation flow remains:

1. Resolve the configured server and tool.
2. Enforce effects and offline transport policy before startup.
3. Start and initialize one fresh session.
4. Discover only configured tool names and require the requested tool to be
   advertised.
5. Send the bounded request.
6. Close the session before returning the result or primary error.

Do not add retries, connection pooling, background tasks that outlive the
operation, caller-selected transports, or a second cancellation abstraction.

# Exact project boundaries

- Update:
  - `crates/acorn-lib/src/io/mcp/client.rs`
  - `crates/acorn-lib/src/io/mcp/tests/client.rs`
  - `docs/src/packages/mcp-client.md`
- Create: none.
- Delete: none.

The client leaf and its focused tests are the implementation write set.
Documentation generation is a later integration lock. No source file currently
modified by another active worktree change is part of this plan.

# Validation

Constructor tests prove:

- direct server-map and application-root construction produce equivalent debug
  identity and call behavior;
- invalid aliases, invalid tool policy, missing executables, empty mapped
  environment values, and missing bearer tokens fail identically; and
- constructing from a map does not mutate the caller's configuration.

Cancellation tests use the real RMCP client against fake child-process and
Streamable HTTP servers. Cover cancellation:

- before startup;
- while initialization is stalled;
- during multi-page catalog discovery;
- while a tool request is pending;
- while the peer ignores graceful close; and
- in a race with a successful response.

Assert that no request follows cancellation, pending requests are released,
HTTP sessions close, child processes exit and are reaped, cleanup respects the
shutdown bound, and the same `McpClient` can perform a later operation.

Retain tests for allowlists, advertised tools, effects, offline mode, bearer
authentication, environment isolation, timeouts, output limits, malformed
responses, catalog ordering, and `is_error` preservation.

After the dependency prerequisite lands, run:

```shell
cargo check -p acorn-lib --no-default-features --features mcp-client
cargo test -p acorn-lib --no-default-features --features mcp-client io::mcp::tests::client
make check-portable
make lint
make test
make doc
```

Inspect `cargo tree --edges features` for the narrow build and rerun the
dependency-deny checks owned by the prerequisite plan.

# Acceptance criteria

- An embedder constructs `McpClient` from
  `BTreeMap<String, McpServerConfiguration>` without creating an
  `ApplicationConfiguration`.
- Application-root and direct-map construction use one implementation and have
  identical validation and resolution behavior.
- Call and list operations accept any `Send` cancellation future without adding
  a public cancellation-token dependency.
- Cancellation is effective during initialization, catalog discovery, and tool
  execution; the public future resolves only after bounded cleanup.
- Child-process cancellation leaves no live or unreaped child, and HTTP
  cancellation releases the session.
- Existing non-cancellable methods, CLI behavior, allowlist and effect policy,
  timeouts, output limits, ordering, and `CallToolResult.is_error` semantics do
  not change.
- The client is `Send + Sync` and remains reusable after a cancelled operation.
- The accepted narrow `mcp-client` graph excludes `analysis`, `std`, Polars,
  OpenSSL, and any cancellation dependency added solely for this interface.

# Non-goals

- No MCP server or built-in tool changes.
- No new transport, header, authentication, retry, pooling, or proxy behavior.
- No typed-error overhaul; cancellation uses the existing `ApiResult` boundary.
- No change to `ApplicationConfiguration.mcp` or serialized MCP configuration.
- No exposure of tools that are not configured, allowed, and advertised.
