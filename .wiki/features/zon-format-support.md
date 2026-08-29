---
type: Plan
title: "Add first-class ZON serialization support"
description: "Implement a pinned strict ZON codec and integrate .zonf with RAD I/O, CLI workflows, crosswalks, discovery, and GitLab review."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli, acorn-schema, acorn-host]
tags: [feature, serialization, zon, zonf, rad, cli, crosswalk, gitlab]
status: planned
---

# Project code boundaries

- **Update:** portable MIME/format values, ResearchActivity codecs, standard crosswalk parsing/serialization, CLI discovery/export/format/check dispatch, GitLab review classification, fixtures, benchmarks, docs, and help.
- **Create:** a pure ZON codec under the schema boundary (initially in-tree, later `acorn-schema`), conformance fixtures, property tests, and std-only path adapters through the existing I/O compatibility surface.
- **Delete:** no existing format; do not add or later retain a second std-only codec under `io`.

# Dependencies

Implement after the [schema I/O validation handoff](../refactors/core/schema-io-validation-handoff.md) defines codec versus path ownership. ZON must not reintroduce schema-to-host coupling while `acorn-schema` is being extracted.

# Goal

Add dependency-free support for the ZON JSON-data-model serialization format using the `.zonf` extension and provisional `text/zonf` media type. ACORN must read, write, format, check, export, crosswalk, discover, and review ZON research activity data while preserving existing schema diagnostics and offline single-binary behavior.

# Current evidence

- ACORN currently recognizes JSON, JSONC, YAML, and Markdown RAD inputs through `MimeType`, `InputOutput`, `ResearchActivity`, CLI path discovery, and structured export dispatch, but has no ZON MIME type or codec.
- Crosswalk parsing, serialization, and source-standard inference currently accept only JSON and YAML.
- GitLab merge-request review has an explicit supported-file list that does not include `.zonf`.
- `serde_json` is already an alloc-capable `acorn-lib` dependency, so a `serde_json::Value` codec requires no new runtime dependency and can remain in the portable schema/WASM surface.
- The published ZON page labels itself version 1.3.0, describes its normative status as v1.1.0, and includes labeled v2 syntax. ACORN will therefore pin conformance to upstream commit [`1b5917dbbf254ba11fb217a378d8ea05a17598d2`](https://github.com/ZON-Format/zon-TS/commit/1b5917dbbf254ba11fb217a378d8ea05a17598d2) rather than claim broad "ZON 1.x" compatibility.

# Recommendation

## Codec and public interface

- Add public schema-codec functions for `encode`, `decode`, and `decode_value`, returning portable typed results with generic Serde bounds and conversion through `serde_json::Value`; retain any `acorn::io::zon` path only as a facade re-export.
- Build the encoder and decoder from small pure functions. Keep the initial implementation inside the portable `acorn-lib` schema boundary and move it mechanically to `acorn-schema`; do not shell out or add a runtime dependency.
- Implement the pinned strict core profile: JSON primitives, canonical numbers, flat and grouped nested objects, root and nested arrays, compound irregular arrays, and uniform-object tables.
- Emit `T`/`F`, `null`, literal UTF-8, specification quoting and escaping, CSV-aware table cells, alphabetically sorted object keys and columns, LF line endings, no trailing whitespace, and no final newline. Preserve array order and produce deterministic bytes.
- Accept LF and CRLF input. Reject BOMs, malformed syntax and numbers, invalid escapes, unterminated strings, malformed table headers, and row or field count mismatches. Preserve upstream `E001` through `E304` identifiers in contextual diagnostics.
- Enforce the published limits on encoding and decoding: 100 MB documents, 1 MB lines, depth 100, one million items per array, and 100,000 keys per object.
- Explicitly reject sparse-table, dictionary-compression, binary, streaming, schema-validation, alternate encoding-mode, and labeled v2 syntax. Strict decoding is always enabled; do not add a permissive public option.

## Format and RAD integration

- Add `MimeType::Zon` with display `text/zonf`, `.zonf` inference, and `file_type() == "zonf"`. Do not recognize `.zon`.
- Add default `InputOutput::read_zon` and `write_zon` methods that report "not implemented," avoiding required changes to unrelated implementations.
- Implement ZON dispatch, reading, writing, and `serialize_as` for `ResearchActivity`. Decode RAD input to a value, convert it to JSON text, and pass it to `eserde::json::from_str` to retain multi-error schema diagnostics.
- Add `FileFormat::Zon`, display it as `ZON`, map it to `MimeType::Zon`, and classify it as structured. Derive structured-export extensions from `MimeType::file_type()` so `--format zon` writes `.zonf` rather than `.zon`.
- Route ZON through structured export and add it to CLI path discovery and `SUPPORTED_RAD_FILETYPES`, enabling `check`, `format`, export, and bucket-related RAD discovery.

## Crosswalk, review, and documentation

- Extend `OneOrMany::parse` and `serialize` plus CLI source-standard inference to handle ZON objects and arrays. Update public errors and docs from "JSON or YAML" to "JSON, YAML, or ZON."
- Add `.zonf` to GitLab merge-request review input recognition; the existing analysis path must then consume it through `ResearchActivity::read`.
- Update curated export and format documentation, examples, module rustdocs, and generated `CommandLineHelp.md`.
- Extend the existing RAD Criterion benchmark with ZON encode/decode timing and byte throughput. Record fixture byte-size comparisons against compact and pretty JSON without making tokenizer-specific claims.

# Validation

- Add golden codec tests for primitives, canonical numbers, quoting, Unicode, escapes, nesting, root and nested tables, compound arrays, empty values, CRLF normalization, and deterministic ordering.
- Add property tests asserting `decode_value(encode(value)) == value` for recursive JSON values constrained to supported resource limits.
- Check in attributed fixtures produced by the official TypeScript implementation at the pinned commit and test compatible decoding and canonical encoding without invoking Node at runtime.
- Cover invalid headers, numbers, escapes, unterminated strings, BOMs, count mismatches, and every resource limit with negative tests.
- Add RAD tests for JSON-to-ZON equivalence, `.zonf` read/write, deterministic serialization, and multiple `eserde` schema errors.
- Add CLI tests for discovery, check, format, regular export, `.zonf` output naming, crosswalk input/output with explicit and inferred standards, and GitLab supported-file classification.
- Regenerate help and run focused tests, `cargo fmt --all --check`, `make lint`, `make test`, `make book`, `cargo check -p acorn-lib --no-default-features`, and `make wasm`.

# Acceptance criteria

- The public codec round-trips every supported JSON value deterministically and rejects input outside the pinned strict profile with actionable diagnostics.
- Resource limits, row counts, field counts, quoting, escaping, line endings, and UTF-8/BOM behavior match the documented ACORN profile.
- `acorn export --format zon activity.json` writes `activity.zonf`; `acorn check activity.zonf` and `acorn format activity.zonf` succeed for valid RAD data.
- Crosswalks accept and emit ZON with both explicit and inferred source standards.
- Automatic local, bucket, and GitLab review discovery recognizes `.zonf` without recognizing `.zon`.
- ZON RAD schema failures retain multi-error `eserde` diagnostics.
- No new runtime dependency is added, and default, minimal, and WASM validation remains green.

# Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Upstream syntax and version labels continue to change. | High | Pin the exact upstream commit, document exclusions, and use attributed conformance fixtures. |
| Ambiguous compound and table syntax causes lossy decoding. | High | Parse into `serde_json::Value` and require golden plus property-based round trips before integration. |
| Large or deeply nested input exhausts memory or stack. | High | Enforce the published limits before allocation or recursion wherever possible. |
| Display name `ZON` accidentally produces a `.zon` file. | Medium | Derive output extensions from `MimeType::file_type()` and assert `.zonf` in CLI tests. |

# Related concepts

- [ACORN work item 16](https://code.ornl.gov/research-enablement/acorn/-/work_items/16)
