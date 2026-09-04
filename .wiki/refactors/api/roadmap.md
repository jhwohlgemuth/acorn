---
type: Plan
title: "Strengthen the shared API-client contract"
description: "Coordinate reusable authentication, endpoint, request, response, webhook, and provider capability boundaries."
difficulty: medium
impact: medium
package: [acorn-lib]
tags: [refactor, api, roadmap]
status: in-progress
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/api/`, `src/io/config/{application,bucket,mod}.rs`, provider tests/snapshots, calling workflows, and feature wiring; later `crates/acorn-api/` plus host/service consumers.
- **Create:** `crates/acorn-api/` after endpoint, auth, request, response, error, and transport-port ownership is acyclic.
- **Delete:** provider protocol implementation from `acorn-lib` after consumers migrate directly; workflow orchestration moves to `acorn-service`, not into the new API crate.

# Dependencies

This roadmap is a boundary input to [crate extraction](../crate-boundary-architecture.md). Shared authentication builders and reusable webhook contracts are complete. The remaining order is endpoint/request migration -> invoke/parse -> environment/health -> errors/DTOs, followed by provider-specific refinements. See the [plan dependency network](../../plan-network.md).

# Parallel execution

This roadmap is coordination-only. Shared `io/api/mod.rs` and `io/config/mod.rs` changes serialize under one API integration owner. Endpoint slices may own `io/config/application.rs`; environment slices must claim `application.rs` or `bucket.rs` explicitly because bucket token discovery overlaps transfer work. After a contract lands, agents may migrate distinct provider leaf files concurrently and leave shared-root wiring to the integration owner. Provider-specific locks and safe fan-out rules are listed in the [parallel execution guide](../../parallel-execution.md).

# Goal

Expand the common API surface for health, authentication, endpoint rendering, request validation, invocation, webhook delivery, and error normalization without erasing provider-specific behavior.

# Implementation order

1. Complete: shared bearer/custom-token authentication builders are implemented and used by OpenAI, RAiD, Hugging Face, and GitLab.
2. Finish endpoint resolution and required/optional request-parameter migration.
3. Introduce invocation and parse helpers with explicit fallback behavior.
4. Normalize environment bootstrap and health checks.
5. Complete: reusable webhook verification, typed delivery, safe callback dispatch, transport errors, and guarded GitLab queue settlement are implemented.
6. Add response, validation, generation, and capability layers only where they reduce current duplication.

# Evidence

- `crates/acorn-lib/src/io/api/mod.rs`
- `crates/acorn-lib/src/io/api/openai.rs`
- `crates/acorn-lib/src/io/api/raid.rs`
- `crates/acorn-lib/src/io/api/gitlab/mod.rs`
- `crates/acorn-lib/src/io/config/application.rs` now owns application-file discovery, endpoint lists, rendering, and synchronization; `bucket.rs` owns bucket repository/authentication orchestration; `mod.rs` retains shared model/registry types and stable public re-exports.
- `Params::{with_auth,with_auth_maybe}` and their tests now own bearer and named custom-token construction; the completed standalone authentication plan was removed.
- `api::Secret` aliases `secrecy::SecretString`; API credentials, provider options, webhook credentials, and persisted runner configuration use secret containers with explicit exposure only at request construction and redacted outer diagnostics through `veil`.
- `Params` also owns immutable optional query/template/body builders, but several provider paths still construct raw `Vec<Param>` values, so parameter composition remains in progress.
- `io::api::webhooks` owns provider-neutral delivery, verification, and one-shot HTTPS callback dispatch; `io::http` enforces request-level redirect refusal, response bounds, anonymous-fallback policy, and sensitive-URL diagnostics.

# Validation

- Keep the config compatibility surface covered by `cargo test -p acorn-lib io::config::tests --lib` while API contracts move.
- Run provider-focused tests, `make lint`, and `make test` after a shared API or config integration slice.
- The config split passed `make lint`, `make test`, and `make cover` on 2026-08-31; coverage reported 66.71% (19,208/28,795 lines).

# Acceptance criteria

Shared mechanics live at the API root, provider modules retain only provider-specific policy and data shapes, and application configuration changes claim the exact config leaf without recreating a monolithic shared file.

# Related plans

- [Crate boundary architecture](../crate-boundary-architecture.md)
