---
type: Plan
title: "Consolidate duplicated workspace behavior"
description: "Give repeated production and test behavior one explicit owner without rebuilding compatibility facades."
difficulty: high
impact: high
package: [acorn-core, acorn-schema, acorn-lib, acorn-cli, acorn-tui]
tags: [refactor, duplication, ownership, maintainability]
status: planned
---

# Goal

Give each shared behavior one owner and make callers depend on that owner directly. Preserve provider-, format-, persistence-, and interface-specific policy where the apparent similarity is intentional.

# Current evidence

- The same strict JSONC `ParseOptions` literal appears in `acorn-core::util::detect_json`, `acorn-lib::io::jsonc_parse_value`, and `acorn-lib::io::parse_jsonc_cst`.
- Handle, RRID, and SWHID repeat identical `PersistentIdentifierParse::{format,from_string,is_valid}` implementations. Handle and SWHID also repeat percent-encoding validation, while their discovery punctuation and identifier rules differ.
- Download dry-run, download completion, and import completion independently assemble equivalent target flags and synchronization options before calling `sync_and_update`.
- The TUI gather screen reimplements input classification, local and remote loading, identifier discovery, persistence, and offline rejection already available through `SourceDocument::load`, `Records`, and `Records::persist`. It also performs blocking HTTP through its own `reqwest` dependency.
- Check, doctor, gather, GGUF-picker, and theme-picker screens construct repeated bordered control bars even though `widgets/status_bar.rs` provides an unused shared renderer that currently accepts only single-character keys and fixed colors.
- RPC, gRPC, and MCP serve commands repeat runtime validation, unsupported-context errors, detached-SSH guidance, foreground SSH invocation, and subprocess failure handling.
- Capability-free and capability-authorized Rhai hook paths repeat the conversion of findings into categorized, document-indexed, source-qualified checks.
- CLI download, import, and format tests repeat the same tracing buffer writer. Fixture-root helpers are intentionally crate-local because sharing them would require a public or cross-crate test-support API.
- Production source and embedded assets have no identical non-empty copies. Identical tracked files are confined to intentional BagIt fixture payloads, fuzz seeds, equivalent snapshots, and other test data that must remain independently addressable.
- Collection validation is already consolidated through `acorn-schema::validation`. Provider health calls are owned by [API health-check helpers](api/health-check-helpers.md); active source-reader and terminal-label consolidation remains under the [portability roadmap](portability/roadmap.md).

# Project code boundaries

- **Update:** `crates/acorn-core/src/util/alloc.rs`; `crates/acorn-schema/src/pid/{mod,handle,rrid,swhid}/`; `crates/acorn-lib/src/{io/mod.rs,exec/}`; `crates/acorn-cli/src/{commands/download,commands/import,commands/sync,commands/serve,test/}`; `crates/acorn-tui/{Cargo.toml,src/screens/,src/widgets/status_bar.rs}`; adjacent focused tests.
- **Create:** crate-private helpers inside the owning modules only when at least two current consumers migrate in the same slice.
- **Delete:** repeated JSONC option literals, redundant PID method bodies and percent validators, TUI-local gather loading/persistence code, screen-local control-bar renderers, repeated serve dispatch branches, repeated Rhai check projection, repeated CLI tracing writers, and the TUI `reqwest` dependency when unused.

# Dependencies and ordering

1. Land the JSONC owner after the active `acorn-core::util::alloc` portability work and before that host boundary moves again.
2. Serialize PID defaults and percent validation with dPID, Creusot, Miri, and other work touching `acorn-schema::pid`.
3. Migrate TUI gather after the deterministic source-reader cutover so it consumes the final shared offline and HTTP policy. Coordinate discovery changes with gather scraping.
4. Serialize remote serve dispatch with the highest-priority RPC/MCP lane and CLI composition; do not change CLI syntax or generated help in this slice.
5. Keep API health/status consolidation in its existing API plan. TUI status bars, Rhai projection, model synchronization, and CLI test utilities may proceed independently when their exact files do not overlap another lane.

This plan follows the ownership rules in [crate boundary architecture](crate-boundary-architecture.md). It must not add compatibility re-exports or a cross-crate test-support API.

# Recommendation

## Portable parsing and validation

Expose `acorn_core::util::strict_jsonc_options() -> jsonc_parser::ParseOptions` and use it for JSON detection, value parsing, and CST parsing. Add bounded default implementations for `PersistentIdentifierParse::{format,from_string,is_valid}` while leaving `find_all` required. Move percent-encoding syntax validation to one crate-private PID helper used by Handle and SWHID; keep their trimming, representation, and validation policies local.

## Services and interfaces

Add one CLI-local model-target options helper for download and import synchronization without changing the richer standalone sync command. Make TUI gather call the shared document/discovery/persistence APIs in a background worker rather than blocking the event loop. Extend the shared status-bar widget to accept the active theme and string key labels such as `Esc`, `Ctrl+A`, and `Up/Down`, then migrate all five screens.

Add one CLI serve helper that returns whether a validated request was handled remotely or should continue to the protocol-specific local server. Keep token, TLS, database, and local service startup in their protocol modules. Add one crate-private Rhai helper that projects findings into checks with the existing category, `document_index * 100 + finding_index`, and URI semantics.

## Test support and intentional repetition

Move the tracing writer and a result-preserving `capture_logs` helper into CLI test utilities and migrate download, import, and format tests. Retain fixture-root helpers within each crate and retain intentional fixture, fuzz, and snapshot copies. Keep synchronization codecs/upsert rules, runtime versus persisted OCI models, and interface-specific rendering separate.

# Validation

1. Add JSONC parity tests for comments, trailing commas, loose names, missing commas, single quotes, hexadecimal numbers, and unary-plus numbers.
2. Run Handle, RRID, and SWHID contract tests, including malformed percent escapes and type-specific discovery punctuation.
3. Test synchronization parity for target selection, force, dry-run, and model-directory overrides.
4. Test TUI local, PID, document, persistence-disabled, and offline-remote gathering plus themed multi-character control bars and compact rendering.
5. Test local, SSH, context, detached, validation-failure, spawn-failure, and nonzero-exit serve behavior for RPC, gRPC, and MCP.
6. Prove capability-free and authorized Rhai hooks produce identical check category, index, and URI metadata, and retain all log-capture assertions.
7. Run focused package tests, `cargo check -p acorn-schema --no-default-features`, the supported alloc/std checks, `make check-wasm`, `make lint`, and `make test`.
8. Repeat tracked-file hashing and normalized production-source scanning; classify intentional fixture, fuzz, and snapshot matches.

# Acceptance criteria

- JSONC entry points obtain parsing policy from one core owner without changing accepted syntax.
- Handle, RRID, and SWHID use the shared generic parse methods; Handle and SWHID share percent validation without merging identifier-specific rules.
- Model synchronization option assembly has one CLI owner while adapter codecs and the standalone sync command retain their policy.
- TUI gather uses shared loading, discovery, persistence, HTTP, and offline behavior and no longer needs direct `reqwest`.
- All five TUI screens use one theme-aware status-bar widget with string key labels.
- RPC, gRPC, and MCP share remote dispatch while retaining protocol-specific local startup and diagnostics.
- Rhai execution paths share one finding-to-check projection, and CLI log-capture tests share one crate-local harness.
- API health, deterministic source/terminal work, intentional test-data copies, fixture-root helpers, OCI boundary types, and format-specific adapters remain with their established owners.
- Focused tests, the portable feature matrix, `make lint`, and `make test` pass.

# Related plans

- [API health-check helpers](api/health-check-helpers.md)
- [CLI parse and execution models](core/cli-parse-execution-models.md)
- [Crate boundary architecture](crate-boundary-architecture.md)
- [Portability roadmap](portability/roadmap.md)
