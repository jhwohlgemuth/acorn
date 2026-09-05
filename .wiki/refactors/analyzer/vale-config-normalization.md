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

# Project code boundaries

- **Update:** Vale configuration logic in `crates/acorn-lib/src/analyzer/{mod,vale}.rs`, native adapter callers, and tests.
- **Create:** pure normalization functions and separate native file/process materialization modules in-tree; reassess package ownership only after the pure subset compiles independently.
- **Delete:** mutation-heavy duplicated normalization branches at migrated call sites.

# Dependencies

Separate pure configuration values from Vale process/file execution under the [portability roadmap](../portability/roadmap.md).

# Evidence

`crates/acorn-lib/src/analyzer/mod.rs` resolves packages and mutates `ValeConfig` through multiple steps.

# Recommendation

Express each normalization as a `ValeConfig -> ValeConfig` transformation, compose the transformations, and apply the pipeline once.

# Acceptance criteria

Configuration resolution preserves current behavior and each transformation can be tested independently.
