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

# Project code boundaries

- **Update:** `crates/acorn-lib/src/analyzer/{mod,check,service}.rs` and adjacent analyzer tests; later `acorn-analysis`.
- **Create:** one additive `Check` collection newtype/helper if the laws and async call sites justify it.
- **Delete:** repeated mutable `Vec<Check>` assembly only at migrated sites; no check payload fields.

# Dependencies

This is a prerequisite for [standards traversal](standards-traversal.md) and simplifies [link resolution](link-resolution-checks.md); it is independent of host adapter extraction.

# Evidence

`crates/acorn-lib/src/analyzer/mod.rs` and `check.rs` repeat join, flatten, and collect pipelines.

# Recommendation

Introduce a `CheckBatch` newtype whose identity is an empty batch and whose combine operation concatenates checks, then use fold-style accumulation at repeated collection boundaries.

# Acceptance criteria

Existing check ordering and results are preserved while repeated batch assembly is removed.
