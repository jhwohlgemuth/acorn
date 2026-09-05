---
type: Plan
title: "Introduce the portable CheckBatch contract"
description: "Replace the Checks alias with an ordered, portable check batch with empty identity and associative concatenation."
difficulty: low
impact: not-assessed
package: [acorn-lib, acorn-cli]
tags: [refactor, analyzer, functional-programming]
status: in-progress
priority: next
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/analyzer/{mod,check}.rs`, `crates/acorn-lib/src/analyzer/host/{service,runtime,discovery}/`, analyzer consumers in `crates/acorn-lib/src/io/api/gitlab/review.rs` and `crates/acorn-cli/src/commands/{check,export,gather}/`, and adjacent tests.
- **Create:** `crates/acorn-lib/src/analyzer/tests/mod.rs` for portable contract tests.
- **Delete:** the `Checks = Vec<Check>` alias and repeated `Vec<Check>` batch assembly at migrated sites; no check payload fields.

# Dependencies

This is a prerequisite for [standards traversal](standards-traversal.md) and simplifies [link resolution](link-resolution-checks.md); it is independent of host adapter extraction.

# Evidence

`crates/acorn-lib/src/analyzer/check.rs` exposes `Checks` as an unconstrained vector alias. Host service, runtime, discovery, and CLI consumers repeat join, flatten, and collect pipelines whose ordering semantics are implicit.

# Recommendation

Introduce a portable `CheckBatch` newtype whose identity is an empty batch and whose combine operation concatenates checks from left to right. Provide explicit vector conversions, ordered collection and iteration, and slice access without a compatibility alias, dereference adapter, crate extraction, or host traits.

# Validation

Run analyzer-focused tests, `make check-portable`, `make check-host`, `make lint`, and the full workspace suite.

# Acceptance criteria

- Empty batches are identity values on both sides and `combine` is associative.
- Collection, nested flattening, borrowed iteration, owned iteration, and vector round trips preserve insertion order.
- Analyzer batch-producing APIs, reports, category groups, readability results, discovery failures, and CLI consumers use `CheckBatch`.
- Summary calculation accepts `&[Check]` and remains independent of batch ownership.
- Category, path, future, and discovery ordering and existing check results are preserved.
