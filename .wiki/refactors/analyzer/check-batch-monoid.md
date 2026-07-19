---
type: Plan
title: "Introduce a monoid for Check collection"
description: "Represent batches of analyzer checks with empty identity and associative concatenation."
difficulty: low
impact: not-assessed
package: [acorn-lib]
tags: [refactor, analyzer, functional-programming]
status: planned
---

# Evidence

`acorn-lib/src/analyzer/mod.rs` and `check.rs` repeat join, flatten, and collect pipelines.

# Recommendation

Introduce a `CheckBatch` newtype whose identity is an empty batch and whose combine operation concatenates checks, then use fold-style accumulation at repeated collection boundaries.

# Acceptance criteria

Existing check ordering and results are preserved while repeated batch assembly is removed.
