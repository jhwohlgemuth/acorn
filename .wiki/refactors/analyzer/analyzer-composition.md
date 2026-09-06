---
type: Plan
title: "Analyzer composition and traversal"
description: "Unify CheckBatch/CheckSummary composition, result-boundary recovery, link evidence, and standard traversal with explicit independent/dependent boundaries."
difficulty: high
impact: medium
package: [acorn-lib, acorn-cli]
tags: [refactor, analyzer, composition, category-theory]
status: in-progress
priority: next
---

# Goal

Complete analyzer composition as small, law-tested primitives: ordered `CheckBatch` combination, additive `CheckSummary` counts, pure `ValeConfig` endomorphisms, unified link evidence, standardized `Result`-to-`Check` recovery, and one shared traversal/fold over standards. Independent category checks accumulate freely; prerequisite chains (`bytes -> deserialize -> validate -> resolve`) remain ordinary `Result` flow. CLI `--exit-on-first-error` stays an outer reporting policy.

# Project code boundaries

- **Update:** `crates/acorn-lib/src/analyzer/{check,error,service}.rs`, `crates/acorn-lib/src/analyzer/host/{context,runtime,service,vale}.rs`, `crates/acorn-lib/src/analyzer/readability/**`, `crates/acorn-lib/src/analyzer/mod.rs`, `crates/acorn-cli/src/commands/{check,gather}/**`, and focused tests; later `acorn-analysis`/`acorn-service` when extraction is proven.
- **Create:** shared link evidence/evaluation values and a focused link evaluator plus concrete orchestration modules in-tree only when the existing module boundary cannot keep pure evaluation separate from I/O; explicit conversion/recovery combinators and traversal helpers only where distinct policies exist.
- **Delete:** ad hoc `Result`-to-`Check` branches, duplicated link-result construction, near-isomorphic per-standard traversal bodies, and duplicate resolution-check streams after all producers use the shared contracts.

# Dependencies

- Consumes the implemented ordered `CheckBatch` (`crates/acorn-lib/src/analyzer/check.rs:279` `empty` identity, `289` `combine` associativity) and portable ordered `ValidationReport` (`docs/src/concepts/category-theory.md:51`).
- Consumes the implemented `CheckSummary` monoid (`crates/acorn-lib/src/analyzer/check.rs:313` `zero`, `326` `combine`) and `ValeConfig` endomorphisms (`crates/acorn-lib/src/analyzer/host/vale.rs:89` `normalize` = `normalize_packages -> merge_policy -> deduplicate`).
- Follows the completed in-tree analyzer kernel/host split recorded by the [portability roadmap](../portability/roadmap.md). Preserve the CI-enforced alloc-only kernel.
- Coordinates with [thiserror adoption](../../migrations/thiserror-adoption.md); do not block portable extraction, but each new crate starts with owned portable errors.
- Keeps concrete link execution in-tree; do not introduce a generic host trait until an alternate consumer proves the seam ([host abstraction traits](../portability/host-abstraction-traits.md) deferred).
- The [crate boundary architecture](../crate-boundary-architecture.md) governs package moves; this plan does not by itself authorize `acorn-analysis` extraction.

# Evidence

- `CheckBatch` monoid laws are tested in `crates/acorn-lib/src/analyzer/tests/mod.rs:26` `check_batch_combine_is_associative_and_ordered`, `36` `check_batch_empty_is_identity_on_both_sides`, `21` insertion-order flattening. Combination is not commutative by design (`docs/src/concepts/category-theory.md:49`).
- `CheckSummary` monoid is implemented and tested: `check.rs:313` `zero` identity, `326` `combine` (`saturating_add`), `337` `counts` stable order, `348` `rows` presentation, `370` `From<&Check>`/`398` `From<&[Check]>`, and `tests/mod.rs:94` identity, `107` associativity, `123` `combined_batch_summaries_equal_flattened_summary`; `57` multi-issue counting (schema `Validator(len)`, prose `Vale(len)`). `docs/src/concepts/category-theory.md:148` still lists it as Planned though the type is present — the remaining work is rendering separation.
- Current summary rendering still scans checks for each category and immediately produces table strings in host/service render paths. The planned split is domain `CheckSummary` value + interface-owned rendering (CLI table vs WASM serialization; discovery summary counts different things and stays separate).
- `ValeConfig::normalize` is already a deterministic endomorphism pipeline with ordered, idempotent steps (`host/vale.rs:89` fold, `107` `normalize_packages`, `115` `merge_policy`, `124` `deduplicate`). Remote release discovery, `ini` materialization, directory creation, and process launch remain host effects after normalization (`host/runtime/mod.rs:217` `check_prose_for`, `728` `StaticAnalyzer<ValeConfig>`, `1005` `sync`).
- Analyzer collection is still explicit sequential loops over categories and per-standard `Analysis::check` dispatch (`host/service/mod.rs:30` `Analysis::check` match, `154` `analyze<T>`, `177` `analyze_paths` with `join_all` over `categories` then per-`Standard` `analyze::<Cff|ResearchActivity|...>`). Independence vs prerequisite boundaries are not yet explicit in types; `--exit-on-first-error` is correctly an outer policy.
- Link checks are partially unified: `check.rs` dispatches `CheckCategory::Link` to `Analysis::check_websites` (standalone `check_resolution` removed), `service.rs` groups by `CategoryChecks`/`AnalysisReport`, `DiscoveryRecords::analyze` attaches completed resolution lifecycle checks as `Link` after path analysis so `offline`/`disable_website_checks` does not discard already-available outcomes. Remote provider matches still convert completed resolution outcomes directly during report construction; no shared evidence/evaluator contract yet.
- `crates/acorn-lib/src/analyzer/mod.rs:5` now only re-exports `check::CheckSummary`; duplicate per-standard check bodies remain in `host/runtime/mod.rs` (`impl Analysis for Cff|ResearchActivity|...`).

# Recommendation

Land as ordered slices under one plan to avoid overlapping renames of `analyzer/{check,service}.rs` and `host/runtime.rs`:

1. **Result-boundary normalization:** centralize `Result`/`ValidationReport`/`ValeOutput` → `CheckBatch` conversion and recovery combinators. One policy for equivalent failures; explicit `to_checks`/`IntoChecks` at boundaries (`ConversionWarnings::to_checks` at `check.rs:490` is the existing example). Remove ad hoc branches.
2. **Link evidence and evaluator:** model minimum shared evidence `{target|identifier, locator, source_uri, optional status, optional diagnostic}`. Website code contributes unresolved URL targets; gather contributes completed `ResolutionStatus` outcomes. One evaluator applies `offline` (no new network) vs `disable_website_checks` (no URL reachability) vs `--resolve` (gated PID provider contact) before converting to `CheckCategory::Link`. Keep remote provider mapping before optional resolution.
3. **Traversal fold:** one data-driven fold over supplied schema content for the common `check_schema / check_websites / check_prose / check_readability / check_quality` orchestration. Per-standard modules retain only reading (`InputOutput::read`) and URL/prose extraction adapters; generic orchestration, `DocumentIndex` resolution, and ordered `CheckBatch` accumulation is shared. Preserve output ordering with compatibility tests before deleting near-isomorphic bodies.
4. **Applicative audit:** make collect-all explicit where branches share the same input and preserve failures from every branch; keep prerequisite chains in `Result` control flow (`bytes -> deserialize -> validate -> resolve`). Do not add concurrency for its own sake; declare order for report stability.
5. **Summary rendering split:** keep `CheckSummary` as the domain value (already `zero`/`combine`/`counts`), move table-string and serialization formatting to CLI/WASM hosts (`rows` is the current thin adapter; domain does not own table layout). Verify `AnalysisReport::checks() -> CheckSummary` homomorphism holds.

Keep changes pure where practical; host I/O (HTTP, filesystem, Vale process, DB persistence, TUI) stays behind `ExecutionOptions`/`AnalysisContext` policy and is tested separately.

# Validation

- Unit-test every pure transformation without filesystem/network: evidence → `Check` conversion (missing URL, reachable/unreachable URL, successful/failed/unsupported resolution), `CheckSummary::zero` identity, `combine` associativity, and `CheckBatch` flattening order. Keep `test_*` alphabetically ordered.
- Test `ValeConfig` endomorphism order, `merge_policy` before `deduplicate`, and `normalize` idempotence without I/O.
- Test `offline` performs no network yet retains failed resolution checks; test `disable_website_checks` does not suppress resolution-outcome checks; test gather `offline`, `file` vs `literal`, `unsupported`, `failed`, `resolved`, `--resolve` absent, CLI/env/default citation-style precedence, ORCID, OSTI, and normalized fallbacks through the unified contract.
- Verify CLI/TUI gather output, failure counts, source fields, citation formatting on resolved raw output, and database persistence remain compatible.
- Run `cargo fmt --all`, `cargo clippy --workspace --no-deps --all-targets` warnings denied, `make test` / `cargo nextest run --run-ignored all`, and the portable `alloc`/`wasm32-unknown-unknown` checks (`make check-wasm` / `make check-portable`).

# Acceptance criteria

1. `CheckBatch::empty` and `CheckSummary::zero` are identities and their `combine` operations are associative (existing tests remain green); non-commutativity is preserved for deterministic report order.
2. `ValeConfig::normalize` has documented order `normalize_packages -> merge_policy -> deduplicate`, is idempotent, and pure tests require no filesystem/network/clock; remote discovery and `ini` writes occur only in host operations.
3. One shared link entry point replaces `check_websites`/`check_resolution`; website and PID-resolution results appear as `CheckCategory::Link` in `AnalysisReport` without manual appends in gather, and policy distinctions (`offline` vs `disable_website_checks` vs `--resolve`) hold.
4. Equivalent failures map to equivalent checks via one shared conversion/recovery policy; ad hoc `Result`-to-`Check` branches are removed.
5. Per-standard implementations retain only reading and adapter extraction; common category orchestration is shared and output ordering is proven identical before bodies are deleted.
6. `AnalysisReport::checks() -> CheckSummary` homomorphism holds: `summary(a.combine(b)) == summary(a).combine(summary(b))` (covered by `tests/mod.rs:123`).
7. Domain `CheckSummary` rendering is separated from CLI/WASM presentation; no host I/O is introduced into pure normalization/combination.

# Related concepts

- [Category-inspired composition in ACORN](../../../docs/src/concepts/category-theory.md)
- [Portability roadmap](../portability/roadmap.md)
- [Crate boundary architecture](../crate-boundary-architecture.md)
- [Analyzer index](./index.md)

# Follow-on work

- Revisit `acorn-analysis` extraction only when a concrete alternate consumer proves a narrow port for the portable kernel.
