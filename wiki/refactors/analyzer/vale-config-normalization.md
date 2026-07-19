---
type: Plan
title: "Compose Vale configuration normalization"
description: "Model Vale configuration changes as composable endomorphisms applied once."
difficulty: low
impact: not-assessed
package: [acorn-lib]
tags: [refactor, analyzer, vale]
status: planned
---

# Evidence

`acorn-lib/src/analyzer/mod.rs` resolves packages and mutates `ValeConfig` through multiple steps.

# Recommendation

Express each normalization as a `ValeConfig -> ValeConfig` transformation, compose the transformations, and apply the pipeline once.

# Acceptance criteria

Configuration resolution preserves current behavior and each transformation can be tested independently.
