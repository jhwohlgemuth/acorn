---
type: Plan
title: RON support for ACRE configuration
description: Add RON map-based ACRE loading, discovery, validation, serialization, and explicit canonical formatting while preserving the existing configuration contract.
difficulty: medium
impact: medium
package: [acorn-core, acorn-lib, acorn-cli]
tags: [feature, acre, configuration, ron, serialization]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Accept `.acorn.ron` and explicitly selected `.ron` files as ACRE configuration alongside JSON, JSONC, and YAML. The user selected RON maps that preserve ACRE's existing field names and enum values. Keep schema version `2.0.0`, defaults, aliases, validation, configuration precedence, and execution policy unchanged.

The intended authoring form is a RON map:

```ron
{
  // Existing ACRE names and values retain their meaning.
  "schemaVersion": "2.0.0",
  "buckets": ["./science"],
  "models": ["./models/research.gguf"],
}
```

# Current evidence

RON-01 through RON-05 are implemented. RON-06 documentation and RON-specific checks are complete, but portfolio retirement waits for the workspace security audit gate. Unrelated Power Platform wiki edits remain outside this implementation's ownership.

- RON 0.12.2 is pinned as an optional `acorn-lib` dependency enabled only by `analysis`. Feature-tree checks keep it absent from default, no-default, `mcp-client`, and `acp` selections. [`MimeType::Ron`](../../crates/acorn-core/src/util/alloc/mod.rs) maps `.ron` to the ACORN-convention label `application/ron` without joining repository-file allowlists.
- [`ApplicationConfiguration::parse_ron`](../../crates/acorn-lib/src/io/config/application.rs) and `to_ron` use a private string-keyed data-tree adapter. The corpus covers every top-level ACRE section, aliases, defaulting, `OneOrMany`, tagged MCP transports, workflow/script values, explicit null, nested duplicates, numeric loss, schema paths, trailing content, recursion limits, canonical output, and typed round trips under `analysis` and `analysis,exec`.
- [`ApplicationConfiguration`](../../crates/acorn-lib/src/io/config/application.rs) now routes extension-based reads/writes, rendering, model-update previews, and `DocumentCodec` through the RON adapter. RON lifecycle tests cover dry-run and write behavior, codec round trips, precedence, and explicit failures.
- [`DEFAULT_CONFIG_FILENAMES`](../../crates/acorn-lib/src/util/constants/app.rs) appends `.acorn.ron` after the existing four names. [`StandardHostExt::from_path`](../../crates/acorn-lib/src/analyzer/host/service/mod.rs) recognizes that exact name; custom filenames require an explicit standard.
- [`parse_check_format`](../../crates/acorn-cli/src/cli/parse.rs) accepts `ron` for piped or clipboard check input. Check-specific discovery accepts `.acorn.ron` without adding all RON files to shared export discovery. CLI parser and check integration tests pass.
- [`format/acre.rs`](../../crates/acorn-cli/src/commands/format/acre.rs) selects canonical `.acorn.ron` files and formats explicit RON through the validated data tree. RON formatting removes comments and preserves omitted fields. The formatter suite and RON library suite pass; [`ACRE_SPEC.md`](../../ACRE_SPEC.md) and the book now document the behavior.
- The adapter preserves `auth.profiles` declaration order through typed loading, formatting, and writing; a regression fixture starts with `zeta` before `alpha`. The final `make test` run passed 3,032 tests with 18 skipped; read-only all-features Clippy, `cargo fmt --check`, `make check-portable` (including WASM package and tests), and `mdbook build docs` passed. `make audit` completed dependency-usage analysis but failed on 38 advisory findings in the unchanged workspace lockfile. That repository-wide security gate remains outside the RON write set.
- RON 0.12.2 documents comments, trailing commas, and map syntax, but limits tagged/untagged/flattened Serde representations and does not guarantee arbitrary round trips through `ron::Value`. This supports using an ACRE data adapter instead of changing shared schema types. [RON documentation](https://docs.rs/ron/0.12.2/ron/#limitations)

# Decisions and boundaries

1. **Preserve the ACRE data model.** Decode RON string-keyed maps, sequences, and compatible scalar values into the existing JSON data model, then use the existing eserde JSON entry point to construct `ApplicationConfiguration`. Encode by projecting the typed configuration to that same data model before RON serialization. Keep the adapter private to ACRE; do not duplicate ACRE fields in a second public schema or call `ron::Value::into_rust::<ApplicationConfiguration>`.
2. **Define the supported representation in fixtures.** Keep enum values as strings, including `"2.0.0"`, `"child-process"`, and effect names. Preserve numeric types and ranges. Omitted optional fields retain current defaults; use RON unit `()` for an explicit JSON null. Named Rust structs and enum constructors are outside the supported authoring contract. Cover every top-level ACRE section, concise/detailed source forms, single/list values, nested maps, and null handling before connecting file loading.
3. **Reject lossy decoding.** Detect duplicate map keys before inserting into an intermediate map. Reject non-string keys, non-finite or unrepresentable numbers, trailing documents, invalid schema versions, and values that cannot retain their ACRE meaning. Reuse typed validation after decoding. RON syntax errors retain their source span; typed errors retain ACRE field paths without claiming positions in the original RON text. Keep JSON's existing multi-error behavior. [RON error API](https://docs.rs/ron/latest/ron/error/struct.SpannedError.html)
4. **Select RON explicitly.** Add `MimeType::Ron` and `.ron` extension handling. Append `.acorn.ron` after `.acorn.yml` in discovery; an explicit `--config settings.ron` still wins. Missing or unreadable explicit RON files and malformed content fail without falling back to another format or a default configuration. Leave the existing format-sniffing `ApplicationConfiguration::parse` behavior intact; provide an explicit RON entry point and route selected RON through it.
5. **Keep the dependency local.** Declare the shared dependency at the workspace root and enable the optional `acorn-lib` dependency through `analysis`, where configuration currently lives. Start with the reviewed 0.12.2 release and verify its MSRV/license/features when implementing. Keep RON out of portable core/schema dependencies and narrow MCP/ACP paths. Preserve the finite parser recursion limit; pretty-print depth is a layout setting, not a parser safety limit. [RON features](https://docs.rs/crate/ron/latest/features), [RON options](https://docs.rs/ron/latest/ron/options/struct.Options.html)
6. **Support the complete configuration lifecycle.** Wire `read`, `write`, `render`, `DocumentCodec::decode/encode`, discovery, and model-update preview/write through one adapter. Parsing remains free of environment reads, network requests, script execution, and model invocation. Keep `.env` loading, relative-path resolution, CLI overrides, and mutation policy at their existing boundaries.
7. **Make formatting behavior explicit.** RON reading accepts comments. RON writing and `acorn format --standard acre` produce canonical serialization with two-space indentation, LF, a final newline, and stable map ordering; source comments and lexical layout are not retained. Format the validated input data tree so omitted `schemaVersion` and other absent fields stay absent. `--dry-run` exposes the exact rewrite, including removed comments, without writing. Document this RON-specific behavior alongside examples; existing JSON/JSONC/YAML comment preservation remains unchanged. A comment-preserving RON formatter is outside this plan. [RON pretty serializer](https://docs.rs/ron/0.12.2/ron/ser/struct.PrettyConfig.html)
8. **Limit format adoption to ACRE.** Recognize `.acorn.ron` in mixed check discovery and accept `check settings.ron --standard acre` and piped/clipboard content with `--format ron --standard acre`. Select canonical RON files for directory ACRE formatting. Do not advertise RON for RAD, metadata crosswalks, schema export, check reports, MCP payloads, or the separately planned daemon configuration. JSON Schema output remains JSON/YAML.

# Dependencies and ownership

There is no prerequisite implementation plan. RON-01 through RON-05 and the RON-06 documentation are landed. RON-06 remains blocked on the repository security audit gate before portfolio retirement. No phase authorizes edits to adjacent subsystems.

- Serialize manifest changes with every workspace-graph owner, including core format changes that touch `acorn-core/src/util/`.
- Serialize ACRE roots with [shared API configuration](../refactors/api/shared-client-contract.md), model configuration, and other active configuration work. These are ownership constraints, not dependencies on those plans finishing.
- Serialize CLI discovery, parser/help changes, and formatter integration with [CLI composition](../refactors/core/cli-composition-boundary.md) and [interactive format](interactive-metadata-chat.md). Avoid expanding shared export discovery to all `.ron` files.
- The [acornd plan](acornd-service.md) owns a separate daemon configuration and requires no RON change. Preserve its current plan and index entries.
- At each phase close, update this plan's evidence and remove the completed phase. The final phase owns portfolio retirement. Unexpected implementation defects require a new bounded repair phase and an updated exact write set.

# Validation and acceptance criteria

| Boundary | Required evidence |
|---|---|
| Data parity | Equivalent JSON, JSONC, YAML, and RON configurations produce equal serialized ACRE values and validation outcomes, including version defaulting, aliases, `OneOrMany`, tagged transports, and feature-dependent sections. |
| Parsing failures | Malformed RON, duplicate keys at multiple depths, unknown fields, bad types/versions, non-finite numbers, out-of-range integers, trailing content, and excessive nesting fail with useful diagnostics. No silent coercion or default configuration. |
| Loading and precedence | RON-only discovery works; every existing canonical filename outranks `.acorn.ron`; explicit RON wins; missing explicit paths fail; config-relative resolution and `.env` precedence remain unchanged. |
| Lifecycle | Read/write and codec round trips preserve meaning. Model-update dry runs leave bytes unchanged, and real writes remain readable RON. Syntax comments are accepted but canonical writes have the documented comment-loss behavior. |
| Checking and discovery | Canonical, custom, mixed-directory, filtered/git-selected, and piped RON paths reach ACRE validation. Other `.ron` documents are not silently treated as valid ACRE or added to general export/repository processing. |
| Formatting | Canonical output is idempotent and semantically equal; absent fields stay absent. Dry runs, unchanged-file handling, and failures preserve source bytes. Existing JSONC/YAML comment tests still pass. |
| Portability and regression | Default/no-default, portable/WASM, and narrow MCP/ACP builds do not acquire RON. Focused suites and final workspace checks pass. Documentation examples match tested fixtures. |

# Execution phases

## RON-06 — Retire after the security audit gate

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: `make audit` passes; its current run reports 38 vulnerabilities from the unchanged workspace lockfile.
- Exact write set: `.wiki/features/acre-ron-configuration.md`, `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`.
- Primary integration lock: Documentation generation
- Outcome: Retire the plan after the repository audit passes without changing the accepted RON contract.
- Focused validation: Run `make audit` after dependency remediation, then validate the wiki and inspect the retirement diff.
- Handoff: Keep durable behavior in tests and documentation; delete this plan and its index/network/lane entries after the remaining gate passes.
