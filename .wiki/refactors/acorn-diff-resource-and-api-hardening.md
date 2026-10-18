---
type: Plan
title: "Bound and extend portable diff operations"
description: "Add deterministic resource limits, safe output boundaries, complete patch construction, configurable unified rendering, and bounded sequence-aware array diffs."
difficulty: high
impact: high
package: [workspace, acorn-diff, acorn-lib, acorn-cli]
tags: [refactor, diff, security, performance, portability, api]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Keep `acorn-diff` pure, alloc-only, and deterministic while making its cost and output trust boundaries explicit. Add bounded entry points for untrusted inputs, complete the public output-oriented patch API, support configurable sink-based unified rendering, and offer sequence-aware arrays without changing the positional V1 default.

# Project code boundaries

- **Update:** `crates/acorn-diff/Cargo.toml`; `crates/acorn-diff/src/{lib.rs,json_patch/mod.rs,json_patch/tests.rs,unified/mod.rs,unified/tests.rs}`; `crates/acorn-cli/src/commands/diff/{mod.rs,tests.rs}`; `crates/acorn-lib/src/analyzer/diff/{mod.rs,tests.rs}`; `crates/acorn-lib/src/analyzer/fix/{preview.rs,tests.rs}`; `docs/src/packages/acorn-diff.md`; and focused `Makefile` or portability-script wiring only when required by the benchmark and feature matrix.
- **Create:** `crates/acorn-diff/benches/diff.rs` for representative resource and array-strategy measurements.
- **Delete:** None. Retain the current convenience functions and positional array behavior as compatibility defaults.

# Dependencies and ordering

The crate boundary, alloc/WASM matrix, pointer conformance coverage, package documentation, and low-cost allocation fixes are implemented prerequisites. No external plan blocks the portable core work.

Land the work in this order: bounded error/options contracts, JSON and unified bounded entry points, first-party consumer migration, header and terminal-output handling, patch-container ergonomics, sink-based rendering, then sequence-aware arrays. Sequence alignment must consume the established work limits rather than introduce an unbounded alternate path.

The crate manifest and CI/build entry points are single-writer locks. Changes to `analyzer::fix` serialize with inference work; changes to the CLI diff command remain a leaf and must not take the shared CLI argument or generated-help lock unless its syntax changes.

# Current evidence

- `DiffValue::diff` recursively traverses supplied `serde_json::Value` instances and returns an infallible owned patch with no depth, visited-node, operation, or output ceiling.
- `unified_diff` invokes the default Myers line algorithm with no input or output bound and always allocates the complete output `String`.
- The native `acorn diff` command validates regular files but reads both completely before comparison. Fix previews also retain complete original and proposed documents.
- Unified labels and changed content are emitted verbatim. Newlines or control characters in a label can forge patch headers, while raw changed content is not safe for direct terminal rendering.
- `JsonPatch` exposes only `is_empty` and borrowed operations. The public constructors create individual operations, but consumers cannot assemble or consume a patch without serializing it.
- Arrays are compared by shared index. This is deterministic and correct, but a front insertion can replace every shared element and append one value even though one indexed add would express the change.

# Recommendation

## Priority (difficulty ascending, then impact descending)

| Order | Recommendation | Difficulty | Impact |
|---:|---|---|---|
| 1 | Escape unified header labels and sanitize host-owned terminal presentation. | Medium | High |
| 2 | Complete patch construction, access, and owned/borrowed iteration APIs. | Medium | Medium |
| 3 | Add configurable `core::fmt::Write`-based unified rendering. | Medium | Medium |
| 4 | Bound input, traversal, operation, alignment, and output work with typed errors. | High | High |
| 5 | Add an opt-in, bounded sequence-aware array strategy with positional fallback. | High | Medium |

Implementation follows the dependency order below rather than this assessment order: the shared limit contract must exist before first-party output handling and sequence alignment can consume it safely.

## Bounded portable operations

Add a public `DiffError`, `JsonDiffOptions`, and `UnifiedDiffOptions`. JSON options bound traversal depth, visited nodes, emitted operations, and sequence-alignment work. Unified options bound input bytes, line count, and output bytes while configuring context radius and the missing-newline hint. Limits are deterministic under both `std` and alloc-only builds; do not use a clock-only timeout as the portable safety contract.

Add fallible bounded methods alongside the compatibility helpers. Migrate every first-party CLI, semantic adapter, and fix-preview caller to the bounded entry points. File consumers must enforce the byte limit before allocating the complete file; canonicalized values must remain subject to node, depth, operation, and output limits after parsing.

Preflight JSON values iteratively before recursive comparison so excessive programmatically constructed nesting returns `DiffError` before `Value::PartialEq` or traversal can exhaust the stack. Treat zero limits as rejecting non-empty work rather than silently substituting defaults.

## Output and API boundaries

Encode unified header labels onto one physical line by escaping backslash, CR, LF, C0 controls, escape, and DEL with deterministic printable sequences. Preserve ordinary Unicode. Keep changed content byte-for-byte raw in machine output and document that raw patches are not terminal-safe. Host renderers escape control characters only when writing human output to a terminal; redirected `acorn diff` output remains a valid raw patch.

Add inherent constructors and accessors to `Operation`, plus `JsonPatch::from_operations`, `JsonPatch::into_operations`, `JsonPatch::len`, borrowed iteration, and owned iteration. If `Rfc6902` is already part of a released API, retain it as a delegating compatibility trait and deprecate it in a later semver-compatible release rather than removing it here.

Add a `core::fmt::Write`-based unified renderer using `UnifiedDiffOptions`; retain `unified_diff` as the default three-context-line `String` convenience wrapper. The bounded writer counts emitted bytes before forwarding each segment and returns `DiffError` without producing bytes beyond the configured ceiling.

## Sequence-aware arrays

Add `ArrayDiffStrategy::{Positional, Sequence}` to JSON options and keep `Positional` as the default. The sequence strategy uses the existing portable `similar` dependency over `serde_json::Value` equality and hashing, emits only add/remove/replace operations, and never infers move or copy.

Translate the aligned edit script with one current target index: equal elements recurse semantically, deletions remove at the current index without advancing it, and insertions add at the current index and advance it. When the configured alignment-work ceiling is exceeded, deterministically fall back to positional comparison. Test repeated equal values and mixed insert/delete runs explicitly because their index bookkeeping is the highest-risk part of the change.

# Validation

1. Add boundary tests at, below, and above every byte, line, depth, node, operation, output, and array-alignment limit under `std` and alloc-only builds.
2. Prove that limit failures emit no partial returned patch and that sink writers never exceed their output ceiling.
3. Test header labels containing backslash, tab, CR, LF, escape, NUL, DEL, Unicode, and their literal escaped spellings; verify redirected machine output remains applicable.
4. Test patch construction, borrowed/owned iteration, inherent constructors, and compatibility-trait parity for all six RFC 6902 shapes.
5. Continue applying generated positional and sequence patches through the independent `json-patch` oracle, including front/middle/end insertions, removals, repeated values, nested arrays, and fallback boundaries.
6. Benchmark one-field object changes, array append, array front insertion, repetitive multiline text, and maximum accepted inputs. Record allocation count, peak bytes, output size, and elapsed time before changing defaults.
7. Run `cargo test -p acorn-diff` with default, alloc-only, and `serde_json/preserve_order` features; Clippy and rustdoc tests; native and WASM portability checks; focused CLI/analyzer tests; `make lint`; and `make test`.

# Acceptance criteria

- Every first-party diff path uses explicit deterministic limits before processing untrusted or repository-controlled content.
- Excessive input, traversal, output, or alignment work returns a typed error without panic, stack exhaustion, or over-limit output.
- Unified headers remain one physical line, machine output preserves changed content, and terminal presentation cannot execute embedded control sequences.
- Public operations can be assembled into, inspected through, and consumed from `JsonPatch` without a serialization round trip.
- Existing convenience APIs retain three context lines, missing-newline hints, deterministic object ordering, and positional array output.
- Sequence-aware arrays produce valid deterministic patches and fall back to positional behavior at the configured work ceiling.
- The normal and alloc-only dependency graphs remain free of ACORN and host-service dependencies and continue to compile for `wasm32-unknown-unknown`.

# Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Limits reject previously accepted large local files. | High | Keep compatibility helpers, expose exact limit diagnostics, and migrate first-party callers with documented policy. |
| Terminal sanitization corrupts machine-readable patches. | High | Sanitize only host-owned terminal presentation; preserve redirected/raw output and test both paths. |
| Sequence edits apply at the wrong shifted index. | High | Use one documented target cursor and verify every generated patch with the independent oracle. |
| Option types expand a deliberately small API. | Medium | Keep one options type per algorithm and retain the current convenience wrappers. |
| Benchmark or feature wiring collides with active CI work. | Medium | Land leaf code first and serialize manifest, Makefile, and CI changes through their existing locks. |

# Related plans

- [Crate boundary architecture](crate-boundary-architecture.md)

# Execution phases

## DIF-01 — Characterize cost and output behavior

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-diff/benches/diff.rs`
- Primary integration lock: acorn-diff public API
- Outcome: Freeze compatibility output and baseline representative resource cases.
- Focused validation: Run current default/alloc/preserve-order tests and benchmark seeds.
- Handoff: Remove DIF-01 and use the baselines for option defaults.

## DIF-02 — Add typed errors and option contracts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DIF-01
- Exact write set: `crates/acorn-diff/src/lib.rs`
- Primary integration lock: acorn-diff public API
- Outcome: Add `DiffError`, JSON options, unified options, limits, context, and positional default.
- Focused validation: Test defaults, zero limits, builders, display, and alloc-only compilation.
- Handoff: Remove DIF-02 and freeze the limit semantics.

## DIF-03 — Preflight JSON depth and nodes iteratively

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DIF-02
- Exact write set: `crates/acorn-diff/src/json_patch/mod.rs`, `crates/acorn-diff/src/json_patch/tests.rs`
- Primary integration lock: acorn-diff public API
- Outcome: Reject excessive programmatic values before recursive equality or traversal.
- Focused validation: Test at/below/above depth and node bounds without panic or stack exhaustion.
- Handoff: Remove DIF-03 and pass accepted values to bounded comparison.

## DIF-04 — Bound JSON operations and traversal

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DIF-03
- Exact write set: `crates/acorn-diff/src/json_patch/mod.rs`, `crates/acorn-diff/src/json_patch/tests.rs`
- Primary integration lock: acorn-diff public API
- Outcome: Enforce deterministic visited-node, operation, output, and alignment-work ceilings with no partial patch.
- Focused validation: Run every limit boundary and independent patch-application oracle.
- Handoff: Remove DIF-04 and expose the bounded JSON entry point.

## DIF-05 — Add bounded sink-based unified rendering

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DIF-02
- Exact write set: `crates/acorn-diff/src/unified/mod.rs`, `crates/acorn-diff/src/unified/tests.rs`
- Primary integration lock: acorn-diff public API
- Outcome: Render through `core::fmt::Write` with input, line, context, and output limits.
- Focused validation: Test byte/line/output boundaries and prove writers never exceed the ceiling.
- Handoff: Remove DIF-05 and retain the compatibility string wrapper.

## DIF-06 — Migrate library and fix-preview consumers

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DIF-04, DIF-05
- Exact write set: `crates/acorn-lib/src/analyzer/diff/mod.rs`, `crates/acorn-lib/src/analyzer/fix/preview.rs`, `crates/acorn-lib/src/analyzer/diff/tests.rs`, `crates/acorn-lib/src/analyzer/fix/tests.rs`
- Primary integration lock: analyzer diff integration
- Outcome: Use explicit bounded entry points for semantic adapters and fix previews.
- Focused validation: Run analyzer semantic/fix tests for accepted and rejected limits.
- Handoff: Remove DIF-06 and record first-party limit policy.

## DIF-07 — Bound CLI file and terminal output handling

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DIF-06
- Exact write set: `crates/acorn-cli/src/commands/diff/mod.rs`, `crates/acorn-cli/src/commands/diff/tests.rs`
- Primary integration lock: CLI diff command
- Outcome: Enforce byte limits before allocation and sanitize only human terminal presentation.
- Focused validation: Test files, redirects, terminals, oversize input, raw patch applicability, and exits.
- Handoff: Remove DIF-07 and preserve raw-versus-terminal behavior.

## DIF-08 — Escape unified header labels

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DIF-05
- Exact write set: `crates/acorn-diff/src/unified/mod.rs`, `crates/acorn-diff/src/unified/tests.rs`
- Primary integration lock: acorn-diff public API
- Outcome: Keep labels on one physical line while preserving ordinary Unicode.
- Focused validation: Test backslash, tab, CR, LF, escape, NUL, DEL, Unicode, and literal escaped forms.
- Handoff: Remove DIF-08 and document the machine-output contract.

## DIF-09 — Complete patch construction and iteration

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DIF-02
- Exact write set: `crates/acorn-diff/src/json_patch/mod.rs`, `crates/acorn-diff/src/json_patch/tests.rs`
- Primary integration lock: acorn-diff public API
- Outcome: Add operation constructors/accessors plus patch construction, length, and owned/borrowed iteration.
- Focused validation: Test all six RFC 6902 operation shapes and compatibility-trait parity.
- Handoff: Remove DIF-09 and record any retained compatibility trait.

## DIF-10 — Add bounded sequence-aware arrays

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DIF-04
- Exact write set: `crates/acorn-diff/src/json_patch/mod.rs`, `crates/acorn-diff/src/json_patch/tests.rs`
- Primary integration lock: acorn-diff public API
- Outcome: Add opt-in deterministic sequence alignment with positional fallback at the work ceiling.
- Focused validation: Apply generated patches for insert/remove/replace, repeats, nesting, and fallback boundaries.
- Handoff: Remove DIF-10 and keep positional output as the default.

## DIF-11 — Run fuzz, benchmark, and portability gates

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DIF-07, DIF-08, DIF-09, DIF-10
- Exact write set: `crates/acorn-diff/benches/diff.rs`, `fuzz/fuzz_targets/diff.rs`, `docs/src/packages/acorn-diff.md`
- Primary integration lock: diff validation
- Outcome: Measure limits/sequence behavior and prove normal, alloc-only, preserve-order, and WASM compatibility.
- Focused validation: Run bounded fuzz seeds, benchmarks, crate tests, clippy, rustdoc, and WASM checks.
- Handoff: Remove DIF-11 and retain measured defaults and regression seeds.

## DIF-12 — Document and retire diff hardening

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DIF-11
- Exact write set: `docs/src/packages/acorn-diff.md`, `.wiki/index.md`, `.wiki/refactors/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/refactors/acorn-diff-resource-and-api-hardening.md`
- Primary integration lock: diff documentation
- Outcome: Document limits, raw output, patch API, array strategies, and remove the completed plan.
- Focused validation: Run package docs, focused consumers, `make lint`, `make test`, and wiki validation.
- Handoff: Delete this plan and every index, network, and lane entry.
