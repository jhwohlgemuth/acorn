---
type: Plan
title: "Consolidate interpolation primitives"
description: "Use one shared abstraction for string, list, and bullet placeholders."
difficulty: low
impact: medium
package: [acorn-lib]
tags: [refactor, interpolation, export]
status: planned
---

# Project code boundaries

- **Update:** interpolation helpers in `crates/acorn-lib/src/util/`, `src/io/powerpoint/`, CLI templates, and tests; target `acorn-core` for pure primitives.
- **Create:** pure scalar/list interpolation functions with explicit missing-value semantics.
- **Delete:** duplicate regex/string interpolation implementations after compatibility tests.

# Dependencies

This prepares [export context](export-context.md), OOXML hardening, and the portable `acorn-core` boundary.

# Evidence

- `crates/acorn-lib/src/util/mod.rs`
- `crates/acorn-lib/src/io/powerpoint/mod.rs`

# Recommendation

Extend the shared interpolation abstraction for list and bullet cases, then remove the parallel PowerPoint-specific interpolation stack.

# Acceptance criteria

String and list placeholders share one tested interpolation implementation.
