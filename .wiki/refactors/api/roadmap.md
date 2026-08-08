---
type: Plan
title: "Strengthen the shared API-client contract"
description: "Coordinate reusable authentication, endpoint, request, response, and provider capability boundaries."
difficulty: medium
impact: medium
package: [acorn-lib]
tags: [refactor, api, roadmap]
status: planned
---

# Goal

Expand the common API surface for health, authentication, endpoint rendering, request validation, invocation, and error normalization without erasing provider-specific behavior.

# Implementation order

1. Extract endpoint resolution and authentication builders.
2. Normalize required and optional request parameters.
3. Introduce invocation and parse helpers with explicit fallback behavior.
4. Normalize environment bootstrap and health checks.
5. Add response, validation, generation, and capability layers only where they reduce current duplication.

# Evidence

- `crates/acorn-lib/src/io/api/mod.rs`
- `crates/acorn-lib/src/io/api/openai.rs`
- `crates/acorn-lib/src/io/api/raid.rs`
- `crates/acorn-lib/src/io/api/gitlab/mod.rs`

# Acceptance criteria

Shared mechanics live at the API root and provider modules retain only provider-specific policy and data shapes.

# Related plans

- [Crate boundary architecture](../crate-boundary-architecture.md)
