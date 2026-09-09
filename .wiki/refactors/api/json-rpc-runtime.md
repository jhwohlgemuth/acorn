---
type: Plan
title: "Add a typed operation registry and JSON-RPC runtime"
description: "Provide shared operation dispatch, JSON-RPC 2.0 messages, policy enforcement, and authenticated HTTP transport."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [feature, api, rpc, json-rpc, service, transport]
status: planned
priority: next
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/api/mod.rs`, `src/io/mcp/tools.rs`, feature wiring, CLI serve parsing/dispatch/tests, generated help, and serve documentation.
- **Create:** `crates/acorn-lib/src/io/api/json_rpc/{mod,http,registry,tests}.rs` and `crates/acorn-cli/src/commands/serve/rpc/mod.rs`.
- **Delete:** duplicate `Value -> Future<Value>` handler aliases after consumers use the shared operation handler. Do not replace `rmcp` or the official ACP SDK's protocol types.

# Dependencies

Consume the shared HTTP service, secret handling, and API error boundaries. This plan precedes the [webhook runtime](webhook-runtime.md), [Protobuf/gRPC transport](protobuf-grpc-transport.md), [GitLab automation](gitlab-webhook-bot-workflows.md), [ACP inference](../../features/opencode-acp-inference.md), and [MCP client](../../features/mcp-client.md).

# Goal

Implement JSON-RPC 2.0 as a transport-neutral invocation boundary for ACORN services. Support requests, notifications, success and error responses, and batches through an immutable, allowlisted method registry. Provide an authenticated bounded HTTP endpoint without reimplementing MCP or ACP lifecycle behavior.

# Decisions

- ACORN methods use `acorn.*`; reject the JSON-RPC-reserved `rpc.*` namespace.
- JSON-RPC request IDs are correlation values, never durable operation, webhook delivery, ACP session, or MCP request identities.
- Method definitions declare parameter/result schemas and read-only, destructive, idempotent, mutation, and open-world effects.
- The registry validates parameters, authorizes effects, invokes the handler, validates results, and records redacted provenance independently of transport.
- Direct HTTP uses `POST /rpc` and a dedicated `ACORN_RPC_TOKEN`; it does not reuse GitLab or webhook credentials.
- Notifications produce no JSON-RPC response. An all-notification HTTP batch returns `204 No Content`.
- Read-only operations may complete inline. Durable operations return an accepted operation reference.

# Public interfaces

```rust
pub struct OperationDefinition {
    pub effects: OperationEffects,
    pub name: String,
    pub parameters_schema: Value,
    pub result_schema: Option<Value>,
}

pub struct InvocationContext {
    pub correlation_id: Option<String>,
    pub offline: bool,
    pub origin: InvocationOrigin,
    pub principal: Principal,
}

pub type OperationFuture = Pin<Box<dyn Future<Output = ApiResult<Value>> + Send>>;
pub type OperationHandler = Arc<dyn Fn(Value, InvocationContext) -> OperationFuture + Send + Sync>;
```

```json
{
  "jsonrpc": "2.0",
  "id": "request-42",
  "method": "acorn.inference.run",
  "params": {"backend": "opencode", "prompt": "Summarize this research activity"}
}
```

# Implementation

1. Add strict JSON-RPC request, notification, response, error, ID, and batch types. Reject duplicate fields, trailing input, invalid versions, empty batches, fractional IDs, and invalid parameter shapes.
2. Add the immutable operation registry and shared handler signature. Register `acorn.version` as the baseline read-only method.
3. Add protocol error mapping for parse error, invalid request, method not found, invalid parameters, internal error, and policy denial.
4. Add a framework-neutral HTTP service plus a thin Axum adapter and `acorn serve rpc` composition root.
5. Adapt the MCP tool registry to reuse handler/policy primitives where behavior is genuinely shared; retain `rmcp` request, response, and lifecycle types.

# Plan conflicts

| Plan or lane | Conflict | Resolution |
|---|---|---|
| API roadmap | `io/api/mod.rs`, HTTP errors, and request handling | One API integration owner; reuse the existing HTTP service. |
| ACP and MCP | Handler types, `io/mcp/tools.rs`, manifests, and CLI roots | Share operation handlers below transports; protocol SDKs retain their wire contracts. |
| Portability | Axum and Tokio are native dependencies | Keep message and registry contracts transport-neutral and gate HTTP serving. |
| CLI composition | Serve command registration and generated help | Land through the CLI integration owner. |

# Validation

- Cover official JSON-RPC request, notification, error, and batch examples.
- Test string, numeric, absent, and invalid IDs; mixed and all-notification batches; deterministic response order; unknown methods; invalid parameters; handler failure; and policy denial.
- Test authentication, content type, body limits, malformed JSON, duplicate fields, trailing data, and credential-safe diagnostics.
- Run focused tests, `make lint`, `make test`, independent feature checks, portable checks, and `git diff --check`.

# Acceptance criteria

- JSON-RPC, gRPC, webhook, ACP, and MCP composition roots can invoke the same registered operation handlers without depending on one another's transports.
- Direct JSON-RPC HTTP behavior conforms to JSON-RPC 2.0, and notifications never receive JSON-RPC responses.
- Transport credentials and transport-specific identifiers never enter operation parameters, results, or provenance.

# Next plan

After this plan's acceptance criteria pass, continue with the [extensible webhook runtime](webhook-runtime.md). Do not begin its shared lifecycle cutover before the operation registry and notification semantics are stable.

# Citations

- [JSON-RPC 2.0 specification](https://www.jsonrpc.org/specification)
