---
type: Plan
title: "Normalize analyzer result boundaries"
description: "Use one conversion and recovery pipeline for analyzer and Vale failures."
difficulty: medium
impact: not-assessed
package: [acorn-lib]
tags: [refactor, analyzer, errors]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/analyzer/{mod,check,error,service}.rs`, boundary callers, and tests; later `acorn-analysis`/`acorn-service`.
- **Create:** explicit conversion functions/newtypes only where distinct recovery policies exist.
- **Delete:** ad hoc `Result`-to-check recovery branches at migrated boundaries.

# Dependencies

Coordinate with [thiserror adoption](../../migrations/thiserror-adoption.md); stabilize and compile the in-tree portable kernel before reconsidering `acorn-analysis`.

# Evidence

`crates/acorn-lib/src/analyzer/mod.rs` and `vale.rs` contain repeated matches that convert failures into defaults or checks.

# Recommendation

Standardize error mapping and explicit recovery combinators at conversion boundaries.

# Acceptance criteria

Equivalent failures produce equivalent checks or propagated errors from one shared policy.
