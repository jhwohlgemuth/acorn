---
okf_version: "0.1"
type: Plan
title: JSON Lines recognition and export crosswalk
description: Recognize `.jsonl` and `.ndjson` as one structured batch format and support deterministic JSON Lines export and standards crosswalks.
difficulty: medium
impact: medium
package:
  - acorn-core
  - acorn-schema
  - acorn-lib
  - acorn-cli
tags:
  - codec
  - crosswalk
  - export
  - jsonl
  - ndjson
status: planned
---

# Project code boundaries

- **Update, portable codec slice:** `crates/acorn-core/src/util/alloc.rs` and `crates/acorn-schema/src/types.rs`, including their adjacent tests.
- **Update, crosswalk slice:** `crates/acorn-lib/src/analyzer/check.rs` and `crates/acorn-lib/src/analyzer/tests/mod.rs`.
- **Update, CLI/export integration slice:** `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/cli/parse.rs`, `crates/acorn-cli/src/cli/tests/mod.rs`, `crates/acorn-cli/src/commands/export/mod.rs`, and `crates/acorn-cli/src/commands/export/tests/mod.rs`.
- **Update, documentation slice:** `docs/src/cli-reference/commands/export.md` and generated `CommandLineHelp.md`.
- **Create:** `tests/fixtures/jsonl/rad-batch.jsonl`, `tests/fixtures/jsonl/datacite-batch.ndjson`, and `tests/fixtures/jsonl/malformed.jsonl`.
- **Delete:** None.

Do not update `crates/acorn-lib/src/io/adapter/mod.rs`, `ResearchActivity`'s generic `InputOutput` implementation, or workflow serialization. The export command can read record batches through `OneOrMany<T>` without making `check`, `format`, enrichment, hooks, or other generic RAD consumers claim JSON Lines support.

# Dependencies

This plan consumes the existing `MimeType`, `OneOrMany<T>`, standard inference, standards crosswalk, and structured-export boundaries. It adds no third-party dependency: the existing `serde_json` codec is sufficient for record-level parsing and compact serialization.

Coordinate portable codec-law coverage with [semantic transformation laws](../testing/semantic-transformation-laws.md) and ownership of `crates/acorn-schema/src/types.rs` with [ZON codec hardening](../refactors/zon-codec-hardening.md). The portable codec can land before the CLI integration, but CLI argument registration, export dispatch, generated help, and shared fixtures remain single-writer integration locks.

Treating a JSON Lines document as a bucket source is a separate follow-on. It must consume the [provider-neutral bucket source abstraction](../refactors/bucket-source-abstraction.md) and the transfer contract in [unified import and download with BagIt archives](unified-import-download-bagit.md), rather than making this export codec impersonate a repository.

# Goal

Make JSON Lines a named structured interchange format for ACORN export and standards crosswalks. Within those boundaries, accept `.jsonl` and `.ndjson` inputs, emit canonical `.jsonl` output, preserve record order, and report malformed records with source line numbers.

# Current evidence

- `MimeType` recognizes JSON, JSONC, and JSON-LD but currently maps both `.jsonl` and `.ndjson` filenames to `MimeType::Vendor`.
- `MimeType::is_repository_file`, `MimeType::infer_from`, and the shared CLI path resolver intentionally do not recognize JSON Lines. Leaving those generic surfaces unchanged prevents repository automation, `check`, and `format` from claiming support accidentally.
- `OneOrMany<T>` already represents single-record and batch-shaped metadata and centralizes JSON, YAML, and ZON parsing and serialization, but has no record-delimited codec. JSON and YAML/ZON batches already preserve their input order.
- `FileFormat` exposes JSON, YAML, and ZON structured exports but no JSON Lines value or NDJSON alias. Its `is_structured` and `From<&FileFormat> for MimeType` matches require explicit integration.
- Standards crosswalks already parse, map, and serialize `OneOrMany<T>`, so a JSON Lines codec can reuse the existing ordered conversion path rather than introduce a second crosswalk implementation. Batch conversion failures currently report a zero-based record index, and field-loss warnings use `record[index]` prefixes.
- `infer_standard_from_content` currently accepts JSON, YAML, and ZON and infers array input from its first object. A JSON Lines branch can apply the same first-record rule while checking later records against the inferred standard.
- Non-crosswalk structured export currently reads one `ResearchActivity` per source file through `InputOutput`. Batch export therefore needs an export-local reader instead of broadening the singleton adapter contract for unrelated commands.
- `resolve_paths` uses one shared default extension list. JSON Lines directory, branch, commit, and merge-request discovery needs an export-specific resolver path; adding JSON Lines to that default list would broaden other commands.
- `gather --raw --enrich` already prints compact enriched RAD values one per line, but that command-specific output does not provide MIME recognition, reusable parsing, export selection, crosswalk support, or `.ndjson` compatibility.

# Decisions

## Format identity

- Add one `MimeType::JsonLines` variant. Its canonical file type is `jsonl`; both `jsonl` and `ndjson` resolve to the same variant.
- Display the variant as `application/x-ndjson`, the media type recommended by the [NDJSON 1.0 specification](https://github.com/ndjson/ndjson-spec). This media type is not registered in the [IANA media type registry](https://www.iana.org/assignments/media-types), and ACORN must not use `application/json-seq`: [RFC 7464](https://www.rfc-editor.org/rfc/rfc7464.html) defines different record-separator framing.
- Expose `jsonl` as the canonical CLI format name and accept `ndjson` as an alias. Do not create separate JSONL and NDJSON variants with divergent behavior.
- Keep JSON Lines distinct from JSON and JSONC. Comments, trailing commas, and multi-line pretty-printed records are not valid JSON Lines input.
- Do not infer JSON Lines from content alone or include it in `MimeType::is_repository_file`; recognition in this plan is extension- and export-format-driven.

## Record codec

- Decode exactly one value of `T` from each record line and collect records in source order as `OneOrMany::Many`, including a one-record JSON Lines document.
- Accept LF and CRLF input and an optional final newline. Reject an empty document and interior whitespace-only records so every reported record has an unambiguous physical line.
- Fail the whole parse when any record is invalid. Diagnostics identify the one-based physical line and retain the underlying deserialization details.
- Serialize every record with compact `serde_json` encoding, one record per line, in stable input order, with one final newline.
- Keep the codec pure over strings. Filesystem reads, writes, progress, and CLI policy remain in their existing host and command layers.

## Export behavior

- Add JSON Lines to structured export without changing PDF, PowerPoint, CFF, Markdown, JSON, YAML, or ZON behavior.
- Exporting one RAD document to JSON Lines produces one record line. Exporting a JSON array or JSON Lines batch produces one line per RAD record.
- Converting JSON Lines to JSON, YAML, or ZON emits the existing batch representation for that target codec; it does not discard records or select only the first record.
- Read batches in the export command through `OneOrMany<ResearchActivity>`. Keep JSONC and Markdown on the existing singleton `ResearchActivity::read` path and wrap their result as `OneOrMany::One` when JSON Lines is the target.
- Directory export retains the current one-output-per-source behavior. This plan does not aggregate unrelated source files into one JSON Lines destination.
- Add an export-specific path-resolution entry point that extends the existing supported extension set with `JsonLines` for directory, branch, commit, and merge-request discovery. Keep `resolve_paths` and its shared default set unchanged.
- Skip a canonical `.jsonl` source only when non-crosswalk export would overwrite the same target path. Treat `.ndjson` as an alias input that can be rewritten to the canonical `.jsonl` suffix.

## Crosswalk behavior

- Infer an unspecified source standard from the first JSON Lines record, then require every record to decode as that same standard. Mixed-standard files fail with the offending line.
- Convert records in order through the existing `OneOrMany<T>` crosswalk. Preserve record cardinality and attach conversion failures or warnings to the corresponding record index and source line when available.
- Support JSON Lines as both source and target for every standard pair already supported by the existing crosswalk matrix. This plan does not add new standard mappings.
- For JSON Lines, a zero-based batch index maps directly to one-based physical line because empty and whitespace-only records are rejected. Reuse that invariant instead of adding source-location state to `OneOrMany<T>`.

## Scope boundary

- Do not add JSON Lines input to `check`, `format`, `gather`, enrichment, workflow hooks, application configuration, or MCP tools.
- Do not add JSON Lines as a bucket source, repository type, archive format, or database backend.
- Do not add streaming partial success, malformed-record skipping, cross-file aggregation, random access, or JSONC-per-line behavior.
- Preserve the existing `gather --raw --enrich` behavior; it may reuse the codec only if doing so does not change its output or failure contract.
- Do not add JSON Lines to generic `InputOutput`, `MimeType::infer_from`, `MimeType::is_repository_file`, or the shared default CLI path set.

# Recommendation

Land the portable format identity and `OneOrMany<T>` codec first, with its alias and error contract frozen by tests. Integrate that codec into the library crosswalk next. Add source inference, export-specific discovery, the CLI `FileFormat`, and structured-export dispatch last so shared CLI and export locks are held only after the lower-level contract is stable.

# Implementation readiness

Evidence was reconciled with the current code on 2026-09-24. No JSON Lines implementation slice is present, so the plan remains `planned` until the first accepted code slice lands.

1. **Portable codec slice:** Own only `crates/acorn-core/src/util/alloc.rs` and `crates/acorn-schema/src/types.rs`. Freeze extension aliases, media display, serde representation, record framing, final-newline output, cardinality, and line-numbered parse errors with inline tests.
2. **Library crosswalk slice:** Own only `crates/acorn-lib/src/analyzer/check.rs` and `crates/acorn-lib/src/analyzer/tests/mod.rs`. Reuse `OneOrMany<T>` parsing and serialization, preserve order, and render JSON Lines failures and warnings with both record index and one-based line where applicable.
3. **CLI/export integration slice:** Acquire the CLI contract and export-dispatch locks for the exact CLI/export files and three dedicated fixtures listed above. Add source inference, export-only discovery, `jsonl`/`ndjson` selection, batch-aware structured export, canonical naming, and regression tests without changing generic ingestion.
4. **Documentation slice:** Update the export guide and regenerate `CommandLineHelp.md` once the CLI contract is final.

The portable codec slice is independent of the CLI and export locks. The library crosswalk slice depends on that codec. CLI/export integration depends on both lower slices and must yield to the highest-priority RPC lane whenever it owns the CLI contract or generated-help lock.

# Implementation sequence

1. Add `MimeType::JsonLines`, `application/x-ndjson` display, canonical `.jsonl` rendering, `.jsonl`/`.ndjson` recognition, and focused core tests without changing content inference or repository-file policy.
2. Add ordered JSON Lines parsing and serialization to `OneOrMany<T>`, including physical-line diagnostics and focused codec tests.
3. Extend existing library crosswalk entry points to accept JSON Lines without duplicating standard mappings, and map JSON Lines batch indices to physical lines in diagnostics.
4. Add JSON Lines source-standard inference, export-only path discovery, the `jsonl` format with an `ndjson` alias, and batch-capable structured-export dispatch.
5. Add the named fixtures, CLI and crosswalk regression tests, export documentation, and regenerated CLI help.

# Validation

- Test `MimeType` parsing, file-type rendering, serde representation, and case-insensitive `.jsonl` and `.ndjson` filenames.
- Test one and many records, LF and CRLF, Unicode, a final newline, missing final newline, empty input, interior blank lines, malformed JSON, schema-invalid records, and stable round trips.
- Test line-numbered failures without leaking record contents into unrelated diagnostics.
- Crosswalk at least two JSON Lines RAD records to an existing target standard and back; assert order, cardinality, compact lines, and indexed conversion failures.
- Test JSON object and array input to JSON Lines, JSON Lines input to JSON/YAML/ZON batch output, canonical `.jsonl` output naming, and the `ndjson` CLI alias.
- Test that `.ndjson` input is not skipped when canonical `.jsonl` output is requested, while a same-path `.jsonl` no-op remains protected from overwrite.
- Verify directory export produces one output per source and does not aggregate files.
- Verify directory-, branch-, commit-, and merge-request-based export discovery accepts JSON Lines while directory-based `check` and `format` do not begin selecting it as a side effect.
- Run focused core, schema, crosswalk, and export tests, then `make lint`, `make test`, `make help`, `make check-portable`, and `make check-wasm`.

# Acceptance criteria

- `MimeType` recognizes `.jsonl` and `.ndjson` as one JSON Lines format, displays `application/x-ndjson`, and renders `jsonl` as its canonical file type.
- `OneOrMany<T>` round-trips ordered JSON Lines records with compact one-record-per-line output and actionable physical-line errors.
- `acorn export --format jsonl` emits canonical `.jsonl` output, and `--format ndjson` selects the same behavior as an alias.
- Existing supported standards can use JSON Lines as crosswalk input or output without new mapping logic, record loss, or reordering.
- Existing JSON, YAML, ZON, gather raw output, and non-export command behavior remain unchanged.
- JSON Lines bucket sources and broad command ingestion remain explicitly outside this plan.
