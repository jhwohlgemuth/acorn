---
type: Plan
title: "Introduce a monoid for analyzer summaries"
description: "Fold checks directly into an additive summary record."
difficulty: low
impact: medium
package: [acorn-lib, acorn-cli, acorn-wasm]
tags: [refactor, analyzer, summary, category-theory]
status: planned
---

# Goal

Separate analyzer counting from presentation so summaries can be calculated once, merged across batches, and rendered consistently by every interface.

# Project code boundaries

- **Update:** `crates/acorn-lib/src/analyzer/check.rs` and its tests; consumers in `crates/acorn-cli/src/commands/{check,export}/mod.rs` and `crates/acorn-wasm/src/lib.rs`; later the owning analysis crate if extraction is authorized.
- **Create:** None as a file. Add one structured summary value beside `Check`/`CheckBatch`, with a zero identity, field-wise combination, and a separate conversion to presentation rows or interface DTOs.
- **Delete:** the current `summary(&[Check]) -> Vec<Vec<String>>` calculation/presentation function and any duplicate category-count folds after all callers migrate.

# Dependencies

Consumes stable `CheckCategory` and `Check::issue_count` semantics. It is otherwise independent, but changes to `check.rs` serialize with [applicative validation](applicative-validation.md), [standards traversal](standards-traversal.md), and other plans holding the analyzer model lock.

# Evidence

- `summary` currently scans the same check slice once for each of six categories and returns table-ready strings rather than domain data.
- The function is called directly by the CLI check and export commands and by the WASM binding, so presentation shape is coupled across interfaces.
- `CheckBatch` already demonstrates the intended algebra: `empty` is an identity, `combine` preserves left-to-right order, and tests cover identity and associativity.
- Discovery owns a separate `Summary` type for gather results. It has different semantics and should not be folded into this analyzer summary merely because the names overlap.

# Recommendation

Represent analyzer counts as a record whose fields correspond to stable `CheckCategory` values. Convert a `Check` using its existing `issue_count`, fold checks in one pass, and combine partial summaries by field-wise addition. Keep table labels, string conversion, serialization DTOs, and visibility filtering in their owning interface layers.

Use one explicit `combine` implementation; implementing `Add` or `Sum` is optional and justified only by clearer call sites. Do not introduce a generic algebra crate or merge the unrelated discovery summary.

# Validation

Add law-oriented unit tests for left/right identity and associativity, category coverage tests including multi-issue schema and Vale checks, and interface tests proving that CLI rows and WASM output remain unchanged.

# Acceptance criteria

1. Every `CheckCategory` contributes exactly `Check::issue_count()` to its corresponding field in one pass over the input.
2. The zero summary is a left and right identity, and combination is associative.
3. Combining per-batch summaries produces the same value as summarizing the flattened checks.
4. CLI check/export output and WASM summary serialization remain unchanged.
5. Summary calculation contains no table labels, color, or string formatting.

# Related concepts

- [Category-inspired composition in ACORN](../../../docs/src/concepts/category-theory.md)
- [Applicative validation](applicative-validation.md)
