---
type: Plan
title: "Strengthen the shared API-client contract"
description: "Coordinate reusable authentication, endpoint, request, response, and provider capability boundaries."
difficulty: medium
impact: medium
package: [acorn-lib]
tags: [refactor, api, roadmap]
status: in-progress
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/api/`, `src/io/config.rs`, provider tests/snapshots, calling workflows, and feature wiring; later `crates/acorn-api/` plus host/service consumers.
- **Create:** `crates/acorn-api/` after endpoint, auth, request, response, error, and transport-port ownership is acyclic.
- **Delete:** provider protocol implementation from `acorn-lib` after facade re-exports; workflow orchestration moves to `acorn-service`, not into the new API crate.

# Dependencies

This roadmap is a boundary input to [crate extraction](../crate-boundary-architecture.md). Shared authentication builders are complete. The remaining order is endpoint/request migration -> invoke/parse -> environment/health -> errors/DTOs; provider-specific refinements follow. See the [plan dependency network](../../plan-network.md).

# Parallel execution

This roadmap is coordination-only. Shared `io/api/mod.rs` and `io/config.rs` changes serialize under one API integration owner. After a contract lands, agents may migrate distinct provider leaf files concurrently and leave shared-root wiring to that owner. Provider-specific locks and safe fan-out rules are listed in the [parallel execution guide](../../parallel-execution.md).

# Goal

Expand the common API surface for health, authentication, endpoint rendering, request validation, invocation, and error normalization without erasing provider-specific behavior.

# Implementation order

1. Complete: shared bearer/custom-token authentication builders are implemented and used by OpenAI, RAiD, Hugging Face, and GitLab.
2. Finish endpoint resolution and required/optional request-parameter migration.
3. Introduce invocation and parse helpers with explicit fallback behavior.
4. Normalize environment bootstrap and health checks.
5. Add response, validation, generation, and capability layers only where they reduce current duplication.

# Evidence

- `crates/acorn-lib/src/io/api/mod.rs`
- `crates/acorn-lib/src/io/api/openai.rs`
- `crates/acorn-lib/src/io/api/raid.rs`
- `crates/acorn-lib/src/io/api/gitlab/mod.rs`
- `Params::{with_auth,with_auth_maybe}` and their tests now own bearer and named custom-token construction; the completed standalone authentication plan was removed.
- `Params` also owns immutable optional query/template/body builders, but several provider paths still construct raw `Vec<Param>` values, so parameter composition remains in progress.

# Acceptance criteria

Shared mechanics live at the API root and provider modules retain only provider-specific policy and data shapes.

# Related plans

- [Crate boundary architecture](../crate-boundary-architecture.md)
