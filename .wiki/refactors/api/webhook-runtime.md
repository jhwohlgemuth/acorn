---
type: Plan
title: "Finish webhook runtime operations"
description: "Expose webhook readiness and outcome state, and add graceful worker draining."
difficulty: high
impact: high
package: [acorn-lib]
tags: [refactor, api, webhook, json-rpc, runtime, queue, callback]
status: in-progress
priority: next
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/api/webhooks/{mod,store}.rs`, webhook worker composition roots, health/state endpoints, and focused lifecycle tests.
- **Create:** A provider-neutral runtime-state DTO only if the existing API status model cannot express readiness and drain state.
- **Delete:** None.

# Dependencies

Uses the implemented `io::api::json_rpc` operation registry. The remaining operational state must land before the [GitLab automation plan](gitlab-webhook-bot-workflows.md) finishes its readiness and drain integration.

# Goal

Finish the operational surface of the provider-neutral webhook runtime: expose shared last-success and last-error timestamps, distinguish liveness from readiness, and drain claimed work gracefully during shutdown.

# Proposed state contract

```rust
pub struct WebhookRuntimeState {
    pub accepting: bool,
    pub active_claims: u64,
    pub last_error_at: Option<Timestamp>,
    pub last_success_at: Option<Timestamp>,
    pub ready: bool,
}
```

# Remaining behavior

- Record shared success/error timestamps when operations and callbacks settle without persisting secrets or raw unauthenticated bodies.
- Stop accepting new claims when shutdown begins, allow active claims to settle within a bounded grace period, and leave unfinished claims recoverable after timeout.
- Report liveness independently from readiness; readiness fails while draining or when store initialization/recovery has not completed.
- Keep state and drain coordination provider-neutral and framework-neutral.

# Plan conflicts

| Plan or lane | Conflict | Resolution |
|---|---|---|
| GitLab automation | Both plans need readiness and shutdown composition | Define state and drain behavior here; GitLab only exposes the shared result. |
| API health work | Health endpoints may otherwise create a second readiness model | Reuse one API status DTO or add one webhook DTO with an explicit conversion. |
| Crate extraction | Runtime state may later divide across API/service/store/host | Keep persistence facts in the store and drain orchestration in the host composition root. |

# Validation

- Test success/error timestamp updates for operation and callback settlement, including retries and terminal failures.
- Test graceful draining with no work, active work that settles, and active work that exceeds the grace period and remains recoverable.
- Test liveness/readiness transitions during initialization, normal operation, drain, and store failure.
- Run focused webhook tests, lint, the full suite, applicable portable checks, and `git diff --check`.

# Acceptance criteria

- [ ] Shared last-success/error timestamps and readiness state are observable without exposing credentials or provider-specific fields.
- [ ] Shutdown stops new claims, settles active work within a bounded grace period, and preserves timed-out work for recovery.

# Next plan

After this plan's acceptance criteria pass, continue with [GitLab repository automation](gitlab-webhook-bot-workflows.md), which must expose rather than recreate the shared readiness and drain state.
