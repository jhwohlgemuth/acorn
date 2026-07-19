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

# Evidence

`acorn-lib/src/analyzer/mod.rs` repeatedly threads options and derived values through nested asynchronous closures.

# Recommendation

Create an immutable analysis context containing normalized options and derived services, then pass it by reference to focused computations.

# Acceptance criteria

Option derivation occurs once and analyzer functions declare the context they consume.
