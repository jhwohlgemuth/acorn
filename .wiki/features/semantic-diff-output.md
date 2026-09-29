---
type: Plan
title: "Portable semantic diff and inference output"
description: "Add a portable diff engine, semantically informed file comparison, and diff-formatted inference suggestions."
difficulty: high
impact: medium
package: [workspace, acorn-diff, acorn-lib, acorn-cli]
tags: [feature, diff, json-patch, inference, portability, cli]
status: planned
---

# Project code boundaries

- **Update:** Workspace manifests and lockfile; focused `acorn-lib` analysis/fix integration; CLI arguments, command registration, check rendering, tests, architecture/package documentation, generated help, and portable dependency gates.
- **Create:** `crates/acorn-diff/`, an in-memory ACORN semantic-diff adapter under `crates/acorn-lib/src/analyzer/diff/`, `crates/acorn-cli/src/commands/diff/`, and dedicated fixtures and documentation.
- **Delete:** None. Retain the existing `acorn_lib::util::terminal` helpers and their current dry-run behavior until a separate compatibility decision authorizes migration.

# Dependencies and coordination

This plan follows [dependency feature defaults](../refactors/portability/dependency-feature-defaults.md) for the new manifest contract and the highest-priority [MCP client](mcp-client.md) for shared workspace, inference, CLI, and generated-help locks. Its contract-first landing order is `acorn-diff`, then the ACORN semantic adapter, then CLI and suggestion integration.

The implementation owns the workspace graph, CLI contract, analyzer module index, documentation-generation, and portable-gate integration locks only during their named slices. Before implementation, replace the directory boundaries above with exact files and coordinate the active `analyzer::fix`, inference, and CLI owners recorded in the [parallel execution guide](../parallel-execution.md).

# Goal

Provide one reusable diff capability for standard unified text and [RFC 6902 JSON Patch](https://datatracker.ietf.org/doc/html/rfc6902), then use it for a standalone `acorn diff` command and for non-mutating previews of inferred fixes.

# Current evidence

- `acorn_lib::util::terminal` uses `similar::TextDiff` to return or print every changed line with `-`, `+`, or context prefixes. Format, link, enrichment, application-configuration, and model-sync dry runs consume it, but it has no file headers or hunk ranges.
- Git helpers request changed filenames only. GitLab review fetches complete head files and does not retain the response's textual patch.
- `FixAction::DocumentChange` already carries exact `expected` and `replacement` strings, and trusted execution applies the first matching occurrence. `FixReport` groups these proposals with non-fatal diagnostics, but human output currently renders summaries rather than patches.
- `InferenceOutputFormat` is shared by `infer run` and `check --suggest-fixes`, although only the latter has a concrete before/after document pair.
- ACORN has typed standard detection and JSON, JSONC, YAML, CFF, ZON, and supported Markdown codecs. Some complete parsing paths are currently `std`-gated, so V1 cannot truthfully promise that semantic normalization is available in an alloc-only build.
- `similar` supports an alloc-only configuration. The available `json-patch` implementation uses `std`, so it is unsuitable as the runtime foundation for the required portable boundary.

# Architecture and portability decisions

## `acorn-diff`: portable transformation owner

- Add a publishable workspace crate that owns only in-memory diff algorithms and wire types. It does not depend on any ACORN crate or on filesystem, process, network, terminal, tracing, async, or CLI libraries.
- Use `default = ["std"]`, an `alloc` minimum capability, and `#![cfg_attr(not(feature = "std"), no_std)]`. The `std` feature enables only dependency-level standard-library support and does not alter results or add host capabilities.
- Depend on `similar` with default features disabled and portable `serde`/`serde_json`. Do not add `schemars` without a demonstrated schema consumer.
- Expose pure functions that accept supplied values and return owned results:
  - unified comparison accepts before/after labels and UTF-8 strings and returns a standard uncolored patch with `---`, `+++`, `@@`, and three context lines;
  - JSON comparison accepts two `serde_json::Value` values and returns an RFC 6902 patch;
  - equal unified inputs return an empty string and equal JSON values return an empty patch array.
- Model all six RFC operations (`add`, `remove`, `replace`, `move`, `copy`, and `test`) with output-focused serializable public types and validated constructors. Generated patches use deterministic `add`, `remove`, and `replace` operations only; move/copy inference and patch application are outside V1.
- Generate JSON Patch locally so alloc-only and WASM builds have identical behavior. Escape JSON Pointer tokens per [RFC 6901](https://datatracker.ietf.org/doc/html/rfc6901), use the empty pointer for the root, sort object keys explicitly, recurse over shared array indices, remove surplus elements from the end, and append additions in target order. Correctness and determinism are contractual; minimum operation count is not.
- Consumers depend directly on `acorn-diff`. Do not add an ownership-masking `acorn-lib` re-export.

## ACORN semantic adapter: pure but native in V1

- Add an `acorn-lib` adapter under the existing native `analysis` capability. It accepts supplied content, labels, media types, and an optional common `Standard`; it performs no file access and delegates final patch construction to `acorn-diff`.
- Parse both sides as the same standard and normalize both to the left input's supported representation. Do not invoke `Standard::crosswalk` or perform a lossy cross-standard comparison.
- Without an explicit standard, plain text, unsupported formats, mismatched standards, or inconclusive detection cause unified output to fall back to the original text. An explicit standard makes parse failure an error. JSON Patch returns a typed diagnostic whenever no trustworthy canonical JSON document pair exists.
- Keep this boundary separate from `analyzer` and `acorn-wasm` features in V1. The generic `acorn-diff` crate must pass alloc-only/WASM checks, while a later portable semantic adapter requires portable codec evidence and a separate consumer-driven plan.

## Host and CLI composition

- Add `acorn diff BEFORE AFTER --format diff|json-patch`; accept `unified` as an alias for `diff` and an optional `--standard` override applied to both inputs.
- V1 accepts two local regular UTF-8 files. The CLI owns path validation and reads; directories, stdin, URLs, and binary inputs fail with normal CLI diagnostics.
- Write only the requested machine-readable result to stdout. Unified labels use the supplied paths with `a/` and `b/` prefixes; output never contains ANSI styling.
- Equal and different inputs both return success. Reads, explicit-standard validation/parsing, and unsupported JSON-Patch comparisons return the normal error exit; automatic unified comparison retains the documented raw-text fallback.
- Split the shared output enum: `infer run` retains `human|json`, while `check` supports `human|json|diff|json-patch`. Reject the two diff formats unless `--suggest-fixes` is present.

## Inference suggestion projection

- Group `DocumentChange` actions by a target that resolves to exactly one checked local file. Ambiguous, non-file, unsafe, or unmatched targets become diagnostics.
- Read each target once, then apply proposals in deterministic report order to an immutable in-memory progression using the existing first-occurrence replacement rule. If an expected value is absent after earlier accepted changes, skip that fix, record its diagnostic, and continue with unrelated fixes and targets.
- Emit one unified patch per changed target from original to final proposed content. Send patches to stdout and manual actions, registered recipes, and skipped-fix summaries to stderr.
- For JSON Patch, semantically normalize each original/final pair and emit one stable JSON envelope:
  - `targets[]` contains `target`, `standard`, `source_format`, successful `fix_ids`, and `patch`;
  - `diagnostics[]` contains optional `target`, related `fix_ids`, `summary`, and `message`.
- Preserve existing `check` pass/fail semantics and existing human/JSON output. Do not write files, execute recipes, or change the inference request/response schema.

# Validation

- Unit-test unified headers, hunk ranges, context radius, additions, removals, missing final newlines, labels, and equality.
- Test every RFC operation's serialization and constructor validation, plus nested values, root replacement, empty structures, pointer escaping, deterministic ordering, and arrays.
- Use a native `json-patch` dev dependency as an independent oracle: property tests apply every generated patch and assert exact equality with the target value. Include the RFC examples as conformance fixtures.
- Run `cargo check -p acorn-diff --no-default-features --features alloc` and the equivalent `wasm32-unknown-unknown` check. Add a dedicated dependency-tree gate that permits `similar` only in the `acorn-diff` graph while rejecting filesystem, process, HTTP, terminal, async-runtime, and ACORN dependencies; leave the existing analyzer/WASM prohibition unchanged.
- Test same-standard JSON, JSONC, YAML, CFF, ZON, and supported Markdown normalization; left-format unified serialization; raw-text fallback; JSON-Patch diagnostics; and proof that crosswalks are not invoked.
- Add CLI and suggestion tests for equal/different exits, override handling, failures, multiple targets, duplicate expected text, overlapping/stale replacements, target ambiguity, manual/recipe diagnostics, deterministic ordering, and stdout/stderr separation.
- Preserve regression coverage for current terminal diff helpers and human/JSON suggestion output. Finish with focused tests, `make check-portable`, `make check-wasm`, generated-help drift checks, `make lint`, and `make test`.

# Acceptance criteria

- `acorn-diff` creates deterministic standard unified diffs and valid RFC 6902 patches from supplied in-memory values under both `std` and alloc-only builds.
- The alloc-only dependency graph has no host capability, and the crate compiles for `wasm32-unknown-unknown` without introducing a JavaScript binding.
- `acorn diff` performs same-standard semantic comparison when supported, falls back only for unified text, and never performs an implicit crosswalk.
- `check --suggest-fixes --output diff|json-patch` presents every applicable document proposal without mutating inputs and preserves all unrenderable proposals as actionable diagnostics.
- Existing inference output, check exit behavior, and dry-run diff consumers remain compatible.
- Architecture and package documentation identify `acorn-diff` as the generic transform owner, `acorn-lib` as the ACORN semantic owner, and CLI/host code as the I/O and presentation owner.

# Out of scope

Directory comparison, stdin and remote inputs, binary patches, cross-standard transformations, patch application, automatic fix execution, colored patches, a JS/WASM binding, and migration or removal of existing terminal diff helpers require separate evidence-backed work.
