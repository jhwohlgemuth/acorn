---
type: Plan
title: Move the RAD schema to standard/rads
description: Move the public Research Activity Data module to acorn_schema::standard::rads and align private filenames without changing RAD data or CLI behavior.
difficulty: high
impact: high
package: [acorn-schema, acorn-analysis, acorn-service, acorn-lib, acorn-cli, acorn-wasm, acorn-web]
tags: [migration, rust-api, schema, rads, portability]
status: planned
execution: phased
phase_budget_hours: 5
---

# Outcome

The canonical Rust path for Research Activity Data is `acorn_schema::standard::rads`. The complete current `crates/acorn-schema/src/research_activity/` tree moves to `crates/acorn-schema/src/standard/rads/`; the RAiD/DataCite crosswalk implementation directory becomes `standard/crosswalk/rads/`. The private host modules, benchmark, and fuzz source named `research_activity.rs` become `rads.rs`. Keep the `ResearchActivity` type, `Standard::ResearchActivityData` variant, RAD document shape, and user-facing `rads` CLI spelling. Remove the old Rust module path at cutover; no permanent re-export facade remains.

# Current evidence (November 27, 2026)

- At `a78440bf` plus the current worktree, `crates/acorn-schema/src/lib.rs:33` declares `pub mod research_activity`. `standard/mod.rs:130-131` already models RAD as `Standard::ResearchActivityData`, and `acorn-cli/src/cli/parse.rs:269` accepts `rad`, `rads`, and `research-activity-data`. The public schema is a directory module, not a file called `research_activity.rs`.
- `rg -l 'research_activity::' crates --glob '*.rs'` found 138 Rust files in seven crates (30 schema, 3 analysis, 3 service, 65 lib, 32 CLI, 2 WASM, 3 web). The [exact path inventory](inventory.md) groups them by owner. It also lists the 18 files under the schema directory, including three Insta snapshots, and four crosswalk files. Re-run the inventory when a phase starts because the worktree is active.
- Three private host files (`io/document/research_activity.rs`, `io/enrichment/research_activity.rs`, `io/export/research_activity.rs`) are registered in their parent `mod.rs` files. Their exports, such as `ExportContext`, are surfaced through parent modules. A source filename change alone is local; changing the public schema module path drives the broad migration.
- `crates/acorn-lib/Cargo.toml:132` declares a `research_activity` benchmark. `fuzz/Cargo.toml:33-34`, `Makefile:331,656`, and `.gitlab-ci.yml:233` name the `research_activity` fuzz target. Coverage output and old snapshot filenames record source paths, but generated coverage is not a source input.
- `research_activity/mod.rs:133-171` declares `ResearchActivity` with Serde and validation attributes. `codec.rs:179-227` decodes RAD from supported text formats. `standard/mod.rs:274-281` detects RAD from document fields. Moving Rust modules must preserve those observable contracts, generated JSON Schema meaning, crosswalk warnings, Markdown/prose snapshots, and browser/WASM behavior.
- `research_activity/mod.rs:14-17` already consumes `standard::{cff,text,zon}`; `research_activity/output.rs:4` consumes DataCite types. The move nests these references under `standard` and may alter module-cycle diagnostics, but no crate dependency changes are required. Run the portable feature matrix after the physical move.

# Scope and decisions

- **Public API:** `acorn_schema::research_activity` → `acorn_schema::standard::rads` is a breaking Rust path change. Keep public type names and serialized names. The final integration must remove `pub mod research_activity` and update first-party consumers directly, as required by the [crate boundary architecture](../../refactors/crate-boundary-architecture.md). Document the break in release-facing notes before release.
- **Staging:** Use a temporary `standard::rads` forwarding module while migrating consumers in small, buildable phases. When the source directory moves, invert the temporary bridge so `research_activity` points at the new owner. Delete that bridge in the final phase. Neither bridge is an accepted end state.
- **Names that remain:** Retain `ResearchActivity`, `ResearchActivityMetadata`, `ResearchActivityData`, RAD field names, `rad`/`rads` CLI aliases, fixture content, schema identifiers, and crosswalk warning values unless a separate compatibility decision and tests justify changing them. The title of this plan is a Rust ownership move, not a new RAD wire format.
- **Private paths:** Rename the three host source files, benchmark source/target, and fuzz source/target to `rads.rs`/`rads`. The crosswalk directory is private and moves for consistency; public `standard::crosswalk::{RaidExportOptions,RaidImportOptions}` stays in place.
- **Concurrent plans:** Serialize this migration with [structured enrichment](../../refactors/core/research-activity-enrichment-integration.md), [browser format parity](../../features/browser-format-parity.md), [research graph](../../features/browser-research-graph.md), and other owners of exact schema, host, WASM, and web files. This plan changes ownership paths only; it does not absorb those plans' behavior work.

# Inventory and exact write-set rule

Each phase's exact source write set is the named group in [inventory.md](inventory.md), plus any additional explicit files in its phase below. A group lists individual files, not an open-ended glob. For directory moves, both old paths and their corresponding new paths are claimed. Every phase may update this plan entrypoint and inventory for status and newly discovered references, but may not silently expand product-code ownership. New references found after the baseline require a bounded phase revision before editing.

# Acceptance criteria

1. `acorn_schema::standard::rads::{ResearchActivity, ResearchActivityMetadata}` and nested public modules resolve; `acorn_schema::research_activity` no longer resolves. First-party crates use the new path directly.
2. The schema directory, private crosswalk directory, three host modules, benchmark, and fuzz source/target use the planned `rads` paths. No stale source-path references remain in active manifests, Make targets, CI, tests, or documentation.
3. Existing RAD JSON, JSONC, YAML, ZON, and Markdown inputs produce equivalent typed values and diagnostics. Serialized field names, schema output, standard detection, CLI spelling, RAiD/DataCite conversions, and snapshot content are unchanged except for source/module metadata.
4. Focused tests pass per phase. Final integration passes `make lint`, `make test`, `make check-wasm`, benchmark compile, fuzz-target compile, browser/WASM RAD checks, and documentation tests when required targets are available. Report an unavailable target rather than claiming it passed.

# Execution phases

## RDS-01 — Expose the new path as a temporary bridge

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: recheck current schema declarations and exact owner lock
- Exact write set: `crates/acorn-schema/src/standard/mod.rs`, new `crates/acorn-schema/src/standard/rads.rs`, `crates/acorn-schema/src/lib.rs`, `crates/acorn-schema/src/tests/mod.rs`
- Primary integration lock: Schema roots
- Outcome: Make the new public path compile by forwarding the current schema items, with a compile-level check for nested public modules. Keep the old owner until consumer migration finishes.
- Focused validation: `cargo test -p acorn-schema` and package Clippy; check `no-default-features`/`alloc` configurations used by the schema.
- Handoff: Record the temporary bridge and its required removal in RDS-13.

## RDS-02 — Migrate schema-internal consumers

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RDS-01
- Exact write set: [Other schema consumers](inventory.md#other-schema-consumers-18)
- Primary integration lock: Schema roots
- Outcome: Point schema codecs, standards, validation, and tests directly to `standard::rads`; keep generated wire output and crosswalk behavior stable.
- Focused validation: `cargo test -p acorn-schema`, schema JSON/Markdown/crosswalk tests, package Clippy, and portable feature check.
- Handoff: Record remaining old-path uses inside the schema source and crosswalk move groups.

## RDS-03 — Migrate portable and interface consumers

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RDS-01
- Exact write set: [Portable and interface consumers](inventory.md#portable-and-interface-consumers-11)
- Primary integration lock: browser/WASM and portable consumer API
- Outcome: Update analysis, service, WASM, and web imports without changing their RAD behavior.
- Focused validation: focused `acorn-analysis`, `acorn-service`, and `acorn-wasm` tests plus `make web-check`.
- Handoff: Record compile and RAD browser contract results.

## RDS-04 — Migrate provider and intake callers

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RDS-01
- Exact write set: [Host API consumers](inventory.md#host-api-consumers-15)
- Primary integration lock: API/config roots
- Outcome: Update provider, GitLab, and Power Automate references to the new schema path.
- Focused validation: focused `acorn-lib` provider/intake tests and package Clippy.
- Handoff: Record any concurrent provider-plan collision before taking another host group.

## RDS-05 — Migrate MCP callers

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RDS-01
- Exact write set: [Host MCP consumers](inventory.md#host-mcp-consumers-10)
- Primary integration lock: Ask capability routing
- Outcome: Update native MCP import, format, export, link, report, and tests to the new path.
- Focused validation: focused `acorn-lib` MCP tests and package Clippy.
- Handoff: Record tool output and schema parity.

## RDS-06 — Migrate native analysis and integration callers

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RDS-01
- Exact write set: [Host analysis, execution, and benches](inventory.md#host-analysis-execution-and-benches-10), [Host test utilities and integration tests](inventory.md#host-test-utilities-and-integration-tests-5)
- Primary integration lock: Analyzer/check contract
- Outcome: Update native analysis and integration callers. Keep the benchmark target name until RDS-12.
- Focused validation: focused `acorn-lib` analyzer and integration tests; `cargo check -p acorn-lib --benches`.
- Handoff: Record benchmark compile status and remaining old-path imports.

## RDS-07 — Migrate host document and export callers

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RDS-01
- Exact write set: [Host document, export, and other I/O consumers](inventory.md#host-document-export-and-other-io-consumers-25)
- Primary integration lock: Export dispatch
- Outcome: Update document, enrichment, export, database, and workflow imports; keep the private filenames until RDS-11.
- Focused validation: focused `acorn-lib` document/enrichment/export/workflow tests and package Clippy.
- Handoff: Record output parity and private module declarations still to rename.

## RDS-08 — Migrate CLI consumers

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RDS-01
- Exact write set: [CLI consumers](inventory.md#cli-consumers-32)
- Primary integration lock: CLI contract
- Outcome: Update CLI commands, examples, and tests to the new path without changing flags, help text, or exit behavior.
- Focused validation: focused CLI RAD check/format/export/intake tests and `cargo check -p acorn-cli --examples`.
- Handoff: Record that parser aliases and command help remain unchanged.

## RDS-09 — Move the owning schema tree

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RDS-02–08 accepted; recheck no new old-path consumers
- Exact write set: [Schema source directory](inventory.md#schema-source-directory-move-every-path-to-standardrads-18), `crates/acorn-schema/src/lib.rs`, `crates/acorn-schema/src/standard/mod.rs`, `crates/acorn-schema/src/standard/rads.rs`
- Primary integration lock: Schema roots
- Outcome: Move the entire source and snapshot tree to `standard/rads/`. Make `standard::rads` the real owner and retain only a short-lived old-path bridge for any unmigrated internal references. Preserve Insta snapshot content; update snapshot identity/source metadata intentionally.
- Focused validation: `cargo test -p acorn-schema`, package Clippy, portable feature check, and snapshot review without automatic blanket acceptance.
- Handoff: List every remaining use of `crate::research_activity` before RDS-13.

## RDS-10 — Move the private crosswalk module

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RDS-09
- Exact write set: [Schema crosswalk directory](inventory.md#schema-crosswalk-directory-move-every-path-to-standardcrosswalkrads-4), `crates/acorn-schema/src/standard/crosswalk/mod.rs`
- Primary integration lock: Schema roots
- Outcome: Move the private crosswalk implementation to `standard/crosswalk/rads/` while preserving public `RaidExportOptions`, `RaidImportOptions`, and conversion warnings.
- Focused validation: focused RAiD/DataCite crosswalk tests, package Clippy, and portable schema check.
- Handoff: Record public export and conversion parity.

## RDS-11 — Rename the three private host modules

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RDS-07
- Exact write set: `crates/acorn-lib/src/io/document/research_activity.rs` → `crates/acorn-lib/src/io/document/rads.rs`, `crates/acorn-lib/src/io/enrichment/research_activity.rs` → `crates/acorn-lib/src/io/enrichment/rads.rs`, `crates/acorn-lib/src/io/export/research_activity.rs` → `crates/acorn-lib/src/io/export/rads.rs`, `crates/acorn-lib/src/io/document/mod.rs`, `crates/acorn-lib/src/io/enrichment/mod.rs`, `crates/acorn-lib/src/io/export/mod.rs`
- Primary integration lock: Library roots
- Outcome: Change private filenames and parent declarations while keeping parent exports and behavior stable.
- Focused validation: focused document, enrichment, and export tests plus package Clippy.
- Handoff: Record the source rename and retained parent API.

## RDS-12 — Rename benchmark and fuzz targets

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RDS-06 and RDS-09
- Exact write set: `crates/acorn-lib/benches/research_activity.rs` → `crates/acorn-lib/benches/rads.rs`, `crates/acorn-lib/Cargo.toml`, `fuzz/fuzz_targets/research_activity.rs` → `fuzz/fuzz_targets/rads.rs`, `fuzz/Cargo.toml`, `Makefile`, `.gitlab-ci.yml`
- Primary integration lock: CI/build
- Outcome: Use `rads` for the benchmark and fuzz target identities. Retain the existing fuzz corpus path and contents unless the command requires a path change, which needs a phase revision.
- Focused validation: `cargo check -p acorn-lib --benches`, fuzz-target compile, and the named Make/CI command inspection.
- Handoff: Record the target names and compile results.

## RDS-13 — Remove the old public path and accept integration

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RDS-02–12 accepted and no old-path source consumers
- Exact write set: `crates/acorn-schema/src/lib.rs`, `crates/acorn-schema/src/standard/mod.rs`, `crates/acorn-schema/src/standard/rads/mod.rs`, `README.md`, `ARCHITECTURE.md`, `docs/src/overview.md`, `.wiki/features/browser-format-parity.md`, `.wiki/features/browser-research-graph.md`, `.wiki/features/dpid-research-object-support.md`, `.wiki/features/sponsor-highlights-clarity.md`, `.wiki/refactors/core/research-activity-enrichment-integration.md`
- Primary integration lock: Library roots
- Outcome: Delete the temporary `acorn_schema::research_activity` bridge; document the breaking Rust path and new canonical path; remove stale active source-path references without changing historical records or RAD terminology.
- Focused validation: compile-fail check for old path and compile-pass check for new path; `make lint`, `make test`, `make check-wasm`, benchmark/fuzz compile, and browser/WASM RAD checks. Run final workspace gates serially after concurrent edits settle.
- Handoff: Record gate results and durable migration note, then retire this plan and its index/network/lane entries.

# Risks to check at execution

- Source move changes Rust module paths, doctest imports, Insta snapshot identity, and generated schema metadata; compare generated outputs rather than assuming a rename is behavior-neutral.
- `standard::rads` imports sibling standards, and crosswalks import RAD; the final tree may change CodeGraph/deslop module-cycle reports. Do not use a lint count as the migration acceptance criterion, but investigate a newly introduced cycle involving `rads`.
- A first-party import sweep cannot prove external users have migrated. Announce the Rust API break at release; avoid a permanent compatibility path that conceals the new owner.
