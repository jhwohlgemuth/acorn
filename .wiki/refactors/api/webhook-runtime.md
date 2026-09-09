---
type: Plan
title: "Build a provider-neutral webhook runtime"
description: "Extend shared webhook verification into typed ingress, JSON-RPC normalization, durable processing, and outbound delivery."
difficulty: high
impact: high
package: [acorn-lib]
tags: [refactor, api, webhook, json-rpc, runtime, queue, callback]
status: in-progress
priority: next
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/api/webhooks/`, `src/io/http/mod.rs`, `src/io/database/mod.rs`, `src/io/api/gitlab/{bot,database,mod,webhook}.rs`, GitLab tests, and shared runtime constants.
- **Create:** `io/api/webhooks/{auth,callback,delivery,ingress,router,store,worker,axum}.rs`, `io/database/webhooks.rs`, and fake-provider/store fixtures.
- **Delete:** GitLab-local receipt, queue, retry, settlement, worker, and counter contracts after migration. Replace experimental `bot_*` tables; no compatibility migration is required because there are no deployed users.

# Dependencies

Requires the [typed operation and JSON-RPC runtime](json-rpc-runtime.md). The [GitLab automation plan](gitlab-webhook-bot-workflows.md) consumes this runtime. Protobuf/gRPC and MCP are independent consumers, not webhook prerequisites.

# Goal

Make `io::api::webhooks` the authoritative provider-neutral webhook capability. Provider adapters authenticate and decode native payloads into typed events, then classify actionable events into durable JSON-RPC notifications handled by the shared operation registry.

# Public interfaces

```rust
pub trait WebhookProvider: Send + Sync {
    type Event: Clone + DeserializeOwned + Send + Serialize + Sync + 'static;

    fn name(&self) -> &'static str;
    fn receive(&self, request: InboundRequest<'_>, now: Timestamp) -> Result<Delivery<Self::Event>, WebhookError>;
    fn classify(&self, delivery: &Delivery<Self::Event>) -> Result<Vec<OperationSpec>, WebhookError>;
}

pub struct OperationSpec {
    pub idempotency_key: String,
    pub message: json_rpc::Notification,
    pub provider: String,
}
```

The JSON-RPC notification has no request ID. Its provider-namespaced idempotency key controls durable deduplication.

# Persistence and behavior

- Create generic inbound-delivery, operation, outbound-delivery, and outbound-attempt tables.
- Persist authenticated normalized payloads, never raw unauthenticated bodies or credentials.
- Enforce a shared body bound before parsing, persist before returning `202 Accepted`, and recover expired claims on startup.
- Keep the core framework-neutral; Axum only translates HTTP request/response types.
- Support provider-namespaced deduplication, bounded retries, terminal failure, replay, cancellation, graceful draining, and counters.
- Outbound callbacks are at-least-once with stable idempotency identity. Persist a deployment destination name and resolve URL/secrets at attempt time. Document ambiguous timeout outcomes and receiver-side deduplication.
- Prove extensibility with GitLab plus a deterministic fake provider.

# Plan conflicts

| Plan or lane | Conflict | Resolution |
|---|---|---|
| JSON-RPC | Notification and operation dispatch types | JSON-RPC lands first; do not define another envelope. |
| API/HTTP | `io/api/webhooks`, `io/http`, and errors | Webhook policy stays here; reuse HTTP transport contracts. |
| Research store | `io/database/mod.rs` and backend wiring | One store owner registers generic webhook tables; no RAD persistence changes. |
| GitLab | Existing queue/worker code moves | Migrate GitLab as the first adapter before repository automation. |
| Crate extraction | Contracts later divide across API/service/store/host | Freeze and test in-tree boundaries before physical movement. |

# Validation

- Run authentication and exact-byte signature tests, a shared conformance suite against GitLab and a fake provider, and multi-operation classification tests.
- Cover namespaced collisions, retries, recovery, replay, cancellation, double settlement, graceful draining, and state reporting.
- Cover outbound success, rejection, ambiguous transport failure, retry, terminal state, and secret-safe persistence.
- Run SQLite/DuckDB paths, focused tests, lint, full tests, audit, portable checks, and whitespace validation.

# Acceptance criteria

- A new provider requires only a typed adapter and a thin route, not copied receipt, queue, worker, or callback code.
- Inbound work is durable before acceptance; outbound callbacks are durably attempted at least once.
- The core lifecycle does not depend on Axum or GitLab.
- Queue state, retries, failures, replay, deduplication, and last success/error are observable.

# Next plan

After this plan's acceptance criteria pass, continue with [GitLab repository automation](gitlab-webhook-bot-workflows.md). GitLab is the first real provider adapter and must consume, rather than recreate, the shared lifecycle.
