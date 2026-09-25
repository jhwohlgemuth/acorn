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

- **Update:** `crates/acorn-core/src/util/alloc.rs`; `crates/acorn-schema/src/types.rs` and focused schema tests; `crates/acorn-lib/src/analyzer/check.rs` and focused crosswalk tests; `crates/acorn-lib/src/io/adapter/mod.rs` only where structured export needs batch-aware RAD serialization; `crates/acorn-cli/src/cli/{arguments/mod.rs,mod.rs,parse.rs}`; `crates/acorn-cli/src/commands/export/{mod.rs,tests/mod.rs}`; export documentation; and `CommandLineHelp.md` when the CLI format list changes.
- **Create:** Dedicated JSON Lines fixtures under `tests/fixtures/jsonl/`.
- **Delete:** None.

# Dependencies

This plan consumes the existing `MimeType`, `OneOrMany<T>`, standard inference, standards crosswalk, and structured-export boundaries. It adds no third-party dependency: the existing `serde_json` codec is sufficient for record-level parsing and compact serialization.

Coordinate portable codec-law coverage with [semantic transformation laws](../testing/semantic-transformation-laws.md) and ownership of `crates/acorn-schema/src/types.rs` with [ZON codec hardening](../refactors/zon-codec-hardening.md). The portable codec can land before the CLI integration, but CLI argument registration, export dispatch, generated help, and shared fixtures remain single-writer integration locks.

Treating a JSON Lines document as a bucket source is a separate follow-on. It must consume the [provider-neutral bucket source abstraction](../refactors/bucket-source-abstraction.md) and the transfer contract in [unified import and download with BagIt archives](unified-import-download-bagit.md), rather than making this export codec impersonate a repository.

# Goal

Make JSON Lines a named structured interchange format for ACORN export and standards crosswalks. Accept `.jsonl` and `.ndjson` inputs, emit canonical `.jsonl` output, preserve record order, and report malformed records with source line numbers.

# Current evidence

- `MimeType` recognizes JSON, JSONC, and JSON-LD but maps `.jsonl` and `.ndjson` to vendor or unknown formats.
- `OneOrMany<T>` already represents single-record and batch-shaped metadata and centralizes JSON, YAML, and ZON parsing and serialization, but has no record-delimited codec.
- `FileFormat` exposes JSON, YAML, and ZON structured exports but no JSON Lines value or alias.
- Standards crosswalks already operate over `OneOrMany<T>`, so a JSON Lines codec can reuse the existing ordered conversion path rather than introduce a second crosswalk implementation.
- Structured export currently reads one `ResearchActivity` per source file. Batch export therefore needs an explicit path instead of broadening the singleton `InputOutput` contract for unrelated commands.
- `gather --raw --enrich` already prints compact enriched RAD values one per line, but that command-specific output does not provide MIME recognition, reusable parsing, export selection, crosswalk support, or `.ndjson` compatibility.

# Decisions

## Format identity

- Add one `MimeType::JsonLines` variant. Its canonical file type is `jsonl`; both `jsonl` and `ndjson` resolve to the same variant.
- Expose `jsonl` as the canonical CLI format name and accept `ndjson` as an alias. Do not create separate JSONL and NDJSON variants with divergent behavior.
- Keep JSON Lines distinct from JSON and JSONC. Comments, trailing commas, and multi-line pretty-printed records are not valid JSON Lines input.

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
- Directory export retains the current one-output-per-source behavior. This plan does not aggregate unrelated source files into one JSON Lines destination.
- Include JSON Lines in export-specific path discovery. Do not add it to the shared default path set in a way that makes `check` or `format` claim support accidentally.

## Crosswalk behavior

- Infer an unspecified source standard from the first JSON Lines record, then require every record to decode as that same standard. Mixed-standard files fail with the offending line.
- Convert records in order through the existing `OneOrMany<T>` crosswalk. Preserve record cardinality and attach conversion failures or warnings to the corresponding record index and source line when available.
- Support JSON Lines as both source and target for every standard pair already supported by the existing crosswalk matrix. This plan does not add new standard mappings.

## Scope boundary

- Do not add JSON Lines input to `check`, `format`, `gather`, enrichment, workflow hooks, application configuration, or MCP tools.
- Do not add JSON Lines as a bucket source, repository type, archive format, or database backend.
- Do not add streaming partial success, malformed-record skipping, cross-file aggregation, random access, or JSONC-per-line behavior.
- Preserve the existing `gather --raw --enrich` behavior; it may reuse the codec only if doing so does not change its output or failure contract.

# Recommendation

Land the portable format identity and `OneOrMany<T>` codec first, with its alias and error contract frozen by tests. Integrate that codec into standard inference and crosswalk next. Add the CLI `FileFormat` and structured-export dispatch last so shared CLI and export locks are held only after the lower-level contract is stable.

# Implementation sequence

1. Add `MimeType::JsonLines`, canonical `.jsonl` rendering, `.jsonl`/`.ndjson` recognition, and focused core tests.
2. Add ordered JSON Lines parsing and serialization to `OneOrMany<T>`, including physical-line diagnostics and focused codec tests.
3. Extend source-standard inference and existing crosswalk entry points to accept JSON Lines without duplicating standard mappings.
4. Add the JSON Lines export format and NDJSON alias, then route batch-capable structured export through the shared codec.
5. Add fixtures, command and crosswalk regression tests, export documentation, and regenerated CLI help.

# Validation

- Test `MimeType` parsing, file-type rendering, serde representation, and case-insensitive `.jsonl` and `.ndjson` filenames.
- Test one and many records, LF and CRLF, Unicode, a final newline, missing final newline, empty input, interior blank lines, malformed JSON, schema-invalid records, and stable round trips.
- Test line-numbered failures without leaking record contents into unrelated diagnostics.
- Crosswalk at least two JSON Lines RAD records to an existing target standard and back; assert order, cardinality, compact lines, and indexed conversion failures.
- Test JSON object and array input to JSON Lines, JSON Lines input to JSON/YAML/ZON batch output, canonical `.jsonl` output naming, and the `ndjson` CLI alias.
- Verify directory export produces one output per source and does not aggregate files.
- Verify directory-based `check` and `format` do not begin selecting JSON Lines files as a side effect.
- Run focused core, schema, crosswalk, and export tests, then `make lint`, `make test`, `make help`, `make check-portable`, and `make check-wasm`.

# Acceptance criteria

- `MimeType` recognizes `.jsonl` and `.ndjson` as one JSON Lines format and renders `jsonl` as its canonical file type.
- `OneOrMany<T>` round-trips ordered JSON Lines records with compact one-record-per-line output and actionable physical-line errors.
- `acorn export --format jsonl` emits canonical `.jsonl` output, and `--format ndjson` selects the same behavior as an alias.
- Existing supported standards can use JSON Lines as crosswalk input or output without new mapping logic, record loss, or reordering.
- Existing JSON, YAML, ZON, gather raw output, and non-export command behavior remain unchanged.
- JSON Lines bucket sources and broad command ingestion remain explicitly outside this plan.
