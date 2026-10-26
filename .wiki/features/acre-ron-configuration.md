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

Reviewed on 2026-10-25 against `7a7c15f2` and the current worktree. Existing IID edits and the separate acornd planning changes are outside this implementation's ownership. This is a source and documentation review; no RON implementation or compatibility tests have run.

- [`ApplicationConfiguration`](../../crates/acorn-lib/src/io/config/application.rs) owns parsing, extension-based reads/writes, discovery, `render`, model-update previews, and the `DocumentCodec` implementation. Each format dispatch currently accepts only JSON, JSONC, or YAML. Changing only `read` would leave encoding and `sync_and_update` incomplete.
- `AcreSchemaVersion` serializes its enum variant as `"2.0.0"`. Configuration also contains `OneOrMany`, untagged shorthand values, tagged MCP transports, and custom deserializers. Direct RON serialization of the Rust structs would expose a different representation and encounter RON identifier/Serde constraints.
- [`DEFAULT_CONFIG_FILENAMES`](../../crates/acorn-lib/src/util/constants/app.rs) orders `.acorn.json`, `.acorn.jsonc`, `.acorn.yaml`, and `.acorn.yml`. Configuration tests explicitly pin that list and its precedence.
- [`StandardHostExt::from_path`](../../crates/acorn-lib/src/analyzer/host/service/mod.rs) recognizes the same four exact ACRE names. Custom filenames require an explicit standard. ACRE uses schema checks without prose, link, or crosswalk analysis.
- [`MimeType`](../../crates/acorn-core/src/util/alloc.rs) has no RON variant; the workspace manifests and lockfile have no `ron` dependency. The full `acorn-lib::io::config` surface is enabled by `analysis`; default and narrow MCP/ACP builds need no RON parser.
- [`parse_check_format`](../../crates/acorn-cli/src/cli/parse.rs) lacks `ron`. CLI path discovery shares an extension list with other commands, including export, so globally adding RON there would broaden unrelated input discovery.
- [`format/acre.rs`](../../crates/acorn-cli/src/commands/format/acre.rs) has its own canonical-name list and uses source-preserving JSON/YAML formatters. [`ACRE_SPEC.md`](../../ACRE_SPEC.md) and the book currently document only the three existing formats.
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

There is no prerequisite implementation plan. RON-01 through RON-06 land in order. Claim the exact files for the active phase before starting; no phase authorizes edits to adjacent subsystems.

- Serialize manifest changes with every workspace-graph owner and core format changes with the [export pipeline](../refactors/core/export-pipeline-hardening.md) owner when it touches `acorn-core/src/util/`.
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

## RON-01 — Register the dependency and format identifier

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: workspace graph and core utility files available for exclusive edits.
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-lib/Cargo.toml`, `crates/acorn-core/src/util/alloc.rs`, `crates/acorn-core/src/util/alloc/tests.rs`.
- Primary integration lock: Workspace graph
- Outcome: Add the optional analysis-only dependency and a tested `MimeType::Ron` extension/string representation. Document its media-type label as an ACORN convention, without claiming registration. Leave repository-file and generic document-format allowlists unchanged.
- Focused validation: `cargo test -p acorn-core --features std mime_type`; `cargo check -p acorn-lib --features analysis`; inspect `cargo tree` for default, no-default, `mcp-client`, and `acp` feature selections.
- Handoff: Freeze the dependency/features and format identifier; RON-02 can use them without manifest edits.

## RON-02 — Implement and prove the ACRE RON adapter

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RON-01.
- Exact write set: `crates/acorn-lib/src/io/config/mod.rs`, `crates/acorn-lib/src/io/config/application.rs`, new `crates/acorn-lib/src/io/config/ron.rs`, `crates/acorn-lib/src/io/config/tests/mod.rs`, new `crates/acorn-lib/src/io/config/tests/ron.rs`, new `tests/fixtures/config/ron/equivalent.json`, new `tests/fixtures/config/ron/equivalent.ron`.
- Primary integration lock: API/config roots
- Outcome: Provide documented explicit RON parsing/serialization methods backed by the private data adapter. Prove existing schema compatibility and duplicate/numeric/error handling without enabling filesystem discovery. Use existing schema deserialization and validation; do not change shared enum or union representations.
- Focused validation: `cargo test -p acorn-lib --features analysis io::config`; run the RON corpus with `--features analysis,exec` for script/workflow representations. Compare normalized typed values and field-path errors across formats.
- Handoff: Preserve the supported RON representation and parser limitations in tests/rustdoc. If any ACRE field cannot round-trip, retain this phase and add a bounded repair before RON-03.

## RON-03 — Complete configuration loading and persistence

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RON-02 compatibility corpus passes.
- Exact write set: `crates/acorn-lib/src/io/config/application.rs`, `crates/acorn-lib/src/util/constants/app.rs`, `crates/acorn-lib/src/io/config/tests/mod.rs`, `crates/acorn-lib/src/io/config/tests/ron.rs`.
- Primary integration lock: API/config roots
- Outcome: Append the canonical name, route extension-based reads/writes, codec operations and preview rendering through the adapter, and preserve model-update and load-with-path behavior. Library consumers can use RON without CLI discovery changes.
- Focused validation: `cargo test -p acorn-lib --features analysis io::config`; cover every discovery precedence pair, explicit-file failures, config-relative paths, load/load-with-path, source selection, read/write, and model-update dry-run/write parity using local fixtures.
- Handoff: Freeze the loader/writer contract and preserve existing JSONC comment behavior; RON-04 can expose it through check commands.

## RON-04 — Add check input and canonical-file discovery

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RON-03; CLI/check files available for exclusive integration.
- Exact write set: `crates/acorn-lib/src/analyzer/host/service/mod.rs`, `crates/acorn-lib/src/analyzer/host/service/tests/mod.rs`, `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/cli/parse.rs`, `crates/acorn-cli/src/cli/tests/mod.rs`, `crates/acorn-cli/src/commands/check/mod.rs`, `crates/acorn-cli/tests/check.rs`, `CommandLineHelp.md`.
- Primary integration lock: CLI contract
- Outcome: Recognize the exact `.acorn.ron` name, add explicit `--format ron`, and preserve standard selection and offline schema-only behavior. Extend check-specific path selection without adding all RON files to shared export discovery. Update config/check help in the same phase.
- Focused validation: `cargo test -p acorn-lib --features analysis analyzer::host::service`; `cargo test -p acorn-cli --lib cli::tests`; `cargo test -p acorn-cli --test check`; verify stdin, explicit file, mixed directory, filters, and git-selected inputs plus unchanged non-ACRE discovery; `make help`.
- Handoff: Record accepted command behavior and release parser/help ownership before formatter integration.

## RON-05 — Add canonical ACRE RON formatting

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RON-04.
- Exact write set: `crates/acorn-lib/src/io/config/application.rs`, `crates/acorn-lib/src/io/config/ron.rs`, `crates/acorn-lib/src/io/config/tests/ron.rs`, `crates/acorn-cli/src/commands/format/acre.rs`, `crates/acorn-cli/src/commands/format/tests/mod.rs`, `docs/src/cli-reference/commands/format.md`.
- Primary integration lock: CLI contract
- Outcome: Format explicit RON files and canonical directory entries through the validated data tree. Use the existing diff/write machinery, preserve omissions and semantic values, and document canonicalization and comment removal when the behavior becomes available.
- Focused validation: `cargo test -p acorn-cli --lib commands::format`; RON codec tests; assert idempotence, semantic equality, unchanged omitted version, exact dry-run/no-write behavior, invalid-input preservation, and retained JSONC/YAML comments.
- Handoff: Preserve formatting examples and guarantees in command documentation; RON-06 can close cross-surface acceptance.

## RON-06 — Document, verify, and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RON-05; other implementations stable for serialized workspace gates.
- Exact write set: `ACRE_SPEC.md`, `README.md`, `docs/src/concepts/acre.md`, `docs/src/concepts/standards.md`, `docs/src/cli-reference/configuration.md`, `docs/src/cli-reference/commands/check.md`, `docs/src/cli-reference/commands/format.md`, `.wiki/features/acre-ron-configuration.md`, `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`.
- Primary integration lock: Documentation generation
- Outcome: Publish the RON mapping contract, examples, discovery order, explicit selection, diagnostics, and formatting limits. Run final gates without expanding implementation scope, then retire the plan.
- Focused validation: `make lint`, `make test`, `make check-portable`, `make check-wasm`, `make audit`, `mdbook build docs`; repeat feature-tree checks from RON-01; verify documented examples against fixtures and run `python3 .agents/skills/planning/scripts/validate_wiki.py .`. Record any pre-existing gate failure separately from RON regressions.
- Handoff: Keep durable behavior in tests and documentation; delete this plan and its index/network/lane entries after every acceptance criterion passes.
