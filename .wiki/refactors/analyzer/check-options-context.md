---
type: Plan
title: "Introduce an explicit analysis context"
description: "Replace repeated CheckOptions threading with composable context-to-result functions."
difficulty: medium
impact: not-assessed
package: [acorn-lib]
tags: [refactor, analyzer, options]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/analyzer/{mod,check,service}.rs`, CLI check/gather callers, and adjacent tests; later `acorn-analysis`/`acorn-service`.
- **Create:** an immutable analysis context/value assembled at composition roots.
- **Delete:** repeated derived-option plumbing at migrated functions; retain user-facing CLI arguments.

# Dependencies

Define this before consumer-owned HTTP/filesystem ports and before completing [link resolution checks](link-resolution-checks.md).

# Evidence

`crates/acorn-lib/src/analyzer/mod.rs` repeatedly threads options and derived values through nested asynchronous closures.

# Recommendation

Create an immutable analysis context containing normalized options and derived services, then pass it by reference to focused computations.

# Acceptance criteria

Option derivation occurs once and analyzer functions declare the context they consume.
