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

# Evidence

- `acorn-lib/src/util/mod.rs`
- `acorn-lib/src/io/powerpoint/mod.rs`

# Recommendation

Extend the shared interpolation abstraction for list and bullet cases, then remove the parallel PowerPoint-specific interpolation stack.

# Acceptance criteria

String and list placeholders share one tested interpolation implementation.
