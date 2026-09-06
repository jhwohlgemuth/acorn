---
type: Plan
title: "Compose independent checks applicatively"
description: "Accumulate independent analyzer failures in one pass where computations do not depend on one another."
difficulty: medium
impact: medium
package: [acorn-lib]
tags: [refactor, analyzer, validation, category-theory]
status: planned
---

# Goal

Make the analyzer's existing collect-all behavior explicit and composable: independent checks contribute every result to one ordered report, while an operation that needs a successful predecessor still short-circuits.

# Project code boundaries

- **Update:** category orchestration in `crates/acorn-lib/src/analyzer/host/service/mod.rs`, validation-to-check conversion in `crates/acorn-lib/src/analyzer/host/runtime/mod.rs`, `CheckBatch` only if its existing collection operations are insufficient, and adjacent tests; later the owning analysis/service crate if extraction is authorized.
- **Create:** None initially. Add a private accumulation helper only if the call-site inventory proves that `CheckBatch::{empty,combine}`, `FromIterator`, and `ValidationReport` cannot express the contract directly.
- **Delete:** manual result accumulation only at the named call sites migrated by this plan; no public applicative framework or replacement check type.

# Dependencies

Build directly on the flat, ordered portable `ValidationReport` and the ordered `CheckBatch` contract. Coordinate changes to `host/service/mod.rs` with [standards traversal](standards-traversal.md) and [link resolution checks](link-resolution-checks.md).

# Evidence

- `acorn-schema::validation::ValidationReport` already collects independent schema failures in declaration order and preserves dependent `skip_on_field_errors` behavior.
- `CheckBatch` already has an empty value, ordered associative `combine`, and flattening `FromIterator` implementations with identity and associativity tests.
- `collect_validation_checks` converts every issue in a `ValidationReport` into a schema check rather than stopping at the first issue.
- `analyzer::host::service::analyze` still pushes category results through a mutable sequential loop. The behavior is collect-all, but independence, ordering, and the boundary between diagnostic values and operational errors are implicit.
- CLI `--exit-on-first-error` is applied after `analyze_paths` returns; it is a reporting policy, not currently an analyzer short-circuit.

# Recommendation

Begin with a call-site inventory. Replace manual accumulation only where every operation can run from the same input and its diagnostic result does not determine whether the next operation is valid. Preserve declared category and path order even if independent effects are later executed concurrently. Keep parsing, deserialization, analyzer synchronization, and similar prerequisite chains in ordinary `Result` control flow.

Do not add a general `Applicative` trait. Prefer the existing concrete report and batch combinators; the category-theory term describes the law and evaluation shape, not a new framework.

# Validation

Add focused analyzer service tests with two independently failing categories, a dependent operation whose prerequisite fails, and multiple standards/path groups. Run the portable analyzer tests plus the CLI check tests after any orchestration change.

# Acceptance criteria

1. Two independent failing checks both appear in the returned `AnalysisReport`.
2. A dependent stage is not invoked when its prerequisite cannot produce a valid input.
3. Categories, paths, and checks retain deterministic declared order.
4. `--exit-on-first-error` remains an outer reporting policy unless a separate CLI behavior change is approved.
5. No new generic applicative abstraction is introduced unless at least two concrete call sites require it.

# Related concepts

- [Category-inspired composition in ACORN](../../../docs/src/concepts/category-theory.md)
- [Summary monoid](summary-monoid.md)
- [Result boundary normalization](result-boundary-normalization.md)
