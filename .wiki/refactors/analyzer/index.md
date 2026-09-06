# Analyzer Refactor Plans

These plans improve the in-tree analysis boundary defined by the [crate boundary architecture](../crate-boundary-architecture.md). They do not by themselves authorize crate extraction.

## Active plan

- [Analyzer composition and traversal](analyzer-composition.md) - Unified `CheckBatch`/`CheckSummary` composition, `ValeConfig` endomorphisms, link evidence, result-boundary recovery, and shared traversal over standards. Replaces `link-resolution-checks`, `result-boundary-normalization`, and `standards-traversal`.

The single plan applies the small, law-driven composition patterns described in [the category-inspired composition concept](../../../docs/src/concepts/category-theory.md); it does not introduce a category-theory framework. Prior leaf plans were consolidated to avoid overlapping renames of `analyzer/{check,service}.rs` and `host/runtime.rs`.

## Implementation slices

Land in order while keeping pure calculation separate from host I/O:

1. Result-boundary recovery combinators (`check`, `error`, `service` boundaries).
2. Shared link evidence/evaluator with correct `offline` vs `disable_website_checks` vs `--resolve` policy.
3. Shared traversal/fold over standards (per-standard adapters remain).
4. Explicit applicative (collect-all) vs dependent (`Result` chain) boundaries; `--exit-on-first-error` stays outer policy.
5. Domain `CheckSummary` vs CLI/WASM rendering split.
