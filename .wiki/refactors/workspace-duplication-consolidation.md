---
type: Plan
title: "Consolidate duplicated workspace behavior"
description: "Remove the production and test duplication found after the acorn-schema extraction without rebuilding compatibility facades."
difficulty: high
impact: high
package: [acorn-core, acorn-schema, acorn-lib, acorn-cli, acorn-tui]
tags: [refactor, duplication, ownership, maintainability]
status: planned
---

# Goal

Give each shared behavior one owner and make callers depend on that owner directly. Preserve provider-, format-, and interface-specific policy where the apparent similarity is intentional.

# Current evidence

- The same strict JSONC `ParseOptions` literal appears in `acorn-core::util::detect_json`, `acorn-lib::io::jsonc_parse_value`, and `acorn-lib::io::parse_jsonc_cst`.
- Handle and SWHID repeat generic `PersistentIdentifierParse` methods and identical percent-encoding validation. Their candidate punctuation rules differ and must remain explicit.
- Download dry-run, download completion, and import completion independently assemble equivalent synchronization options before calling `sync_and_update`.
- Collection validation is already consolidated: `OneOrMany<T>`, vectors, and arrays delegate to the shared sequence helpers in `acorn-schema::validation`.
- TUI screens construct repeated bordered control bars even though `widgets/status_bar.rs` provides a shared renderer that is not used by those screens.
- ROR, ORCID, and CiteAs repeat the same endpoint status invocation. CLI download and import tests repeat a tracing buffer writer, and CLI/library tests repeat workspace fixture-path resolution.
- Exact-file and asset hashing found no duplicate non-empty files. Schema source and assets are singularly owned after extraction; this plan does not recreate a schema or core facade in `acorn-lib`.

# Project code boundaries

- **Update:** `crates/acorn-schema/src/pid/{handle,swhid}/mod.rs`; `crates/acorn-core/src/util/alloc.rs`; `crates/acorn-lib/src/io/sync/` and provider API modules; `crates/acorn-cli/src/commands/{download,import}/`; `crates/acorn-tui/src/screens/` and `widgets/status_bar.rs`; adjacent focused tests.
- **Create:** dedicated shared test helpers only when at least two existing test modules consume them.
- **Delete:** screen-local status-bar renderers after the widget accepts their required theme and key-label representation.

# Dependencies and ordering

1. Make core own the JSONC parsing policy before extracting further host adapters.
2. PID and collection-validation changes serialize with other schema-lane work touching the same modules.
3. TUI, provider status, and test-helper slices may proceed independently when their exact write sets avoid shared module indexes.

This plan follows the ownership rules in [crate boundary architecture](crate-boundary-architecture.md). It must not add re-export modules to preserve displaced paths.

# Recommendation

## Portable parsing and validation

Expose one core JSONC options constructor and use it for detection, value parsing, and CST parsing. Give `PersistentIdentifierParse` default implementations where `FromStr`, `Default`, and `Display` bounds permit; share percent-encoding validation while passing identifier-specific trimming policy explicitly. Preserve the implemented shared sequence validation rather than reopening it in this plan.

## Host and interface repetition

Extract a CLI-local synchronization helper that converts resolved model identifiers and target flags into `sync::Options`. Keep each adapter's codec and upsert policy local. Route screen controls through the TUI status-bar widget. Add a shared endpoint status invocation helper and narrowly scoped test utilities.

# Validation

1. Add parity tests before consolidating each behavior, including JSONC comments/trailing commas and invalid percent escapes.
2. Run focused package tests for every slice.
3. Run `cargo check -p acorn-schema --no-default-features`, the supported alloc/std checks, and `make check-wasm` for portable changes.
4. Run `make lint` and `make test` after each integration slice.
5. Repeat the exact-file, asset-hash, and normalized source-block scan; document intentional remaining matches.

# Acceptance criteria

- All JSONC entry points obtain parser policy from one core owner.
- Handle and SWHID share generic parsing and percent-encoding behavior without merging their distinct identifier rules.
- Model synchronization option assembly has one CLI owner; adapter-specific codecs remain explicit.
- TUI screens use the shared status-bar widget.
- Provider status calls and repeated test harness utilities have one appropriate local owner.
- No consolidation introduces a public compatibility re-export or moves host constants out of `acorn-lib`.
- Focused tests, the portable feature matrix, `make lint`, and `make test` pass.

# Related plans

- [Crate boundary architecture](crate-boundary-architecture.md)
- [Portability roadmap](portability/roadmap.md)
- [API health-check helpers](api/health-check-helpers.md)
- [CLI parse and execution models](core/cli-parse-execution-models.md)
