---
type: Plan
title: "Isolate std-only modules from domain modules"
description: "Complete the boundary between portable transformations and filesystem, process, and network services."
difficulty: medium
impact: critical
package: [acorn-lib, acorn-cli]
tags: [refactor, no-std, portability]
status: in-progress
priority: next
---

# Evidence

- `acorn-core::prelude::{core, alloc}` exposes the portable capability boundary. Schema and portable utility modules import allocation-backed types from that owner, while host code imports operating-system services directly from `std`; `acorn-lib` no longer publishes an ownership-masking prelude.
- `acorn-schema::validation` provides alloc-compatible contracts, ordered flat reports, built-in scalar/PID/version rules, and indexed collection validation; the derive emits direct owning-crate paths.
- Portable string/byte codecs remain on schema types, while path/file `InputOutput` implementations, migration machinery, RAiD read/conversion, and `format_with` live in the std-only `io::adapter` module.
- `acorn-core` owns portable errors, location/repository values, and core/alloc utility implementations. `acorn-lib` supplies native `LocationExt` and `RepositoryExt` adapters.
- OpenCode `InputOutput`, model-weight persistence, RAD contextual formatting, and RAD Markdown document indexing have moved out of schema modules. License/programming-language schema validation is deterministic and database-independent.
- Schema models and deterministic prompt rendering now live in `acorn-schema`; network/file resolution, OpenCode CST/path preservation, persistence, and presentation remain explicit host extensions in `acorn-lib`.
- `Check` no longer stores a host `DocumentIndex`. `AnalysisReport` owns document indexes once and supplies them to detailed or terse rendering through `RenderContext`.
- `AnalysisContext` explicitly separates calculation inputs (`AnalysisOptions`), concrete host execution (`ExecutionOptions`), and rendering/failure policy (`ReportOptions`).
- Readability calculation consumes an explicit metric and optional threshold; the CLI composition root owns `.env` resolution.
- `analyzer/error.rs:5` `DocumentPath`/`DocumentQuery` and `DocumentTarget::query:13` gated `std`; `ValidatorIssue::query:209` gated.
- `analyzer/mod.rs:7` `SourceDocument`/`InputOutput`/`command_exists`/`Path`/`PathBuf` now `#[cfg(feature="std")]`, `Arc`/`HashMap` kept for `alloc`.
- `analyzer` is still selected only by the `analysis` feature, and `analysis` still implies `std`. Individual source sections have portable characteristics, but an independently compiling portable analyzer boundary has not yet been established.
- Vale output DTOs now have analyzer-model ownership and are re-exported from the existing `analyzer::vale` path; Vale runtime/configuration remains host-owned.

# Project code boundaries

- **Update:** `crates/acorn-lib/Cargo.toml`; `src/lib.rs`, `error.rs`, `src/analyzer/`, `src/io/`, and `src/util/`; `crates/acorn-core/src/prelude.rs`; focused tests adjacent to each moved boundary.
- **Create:** in-tree portable model/calculation modules and explicit host rendering/execution modules only after their dependency cuts compile independently.
- **Delete:** remaining analyzer portable-to-host imports after explicit adapters exist.

# Dependencies

- **Prerequisite:** keep [dependency feature defaults](dependency-feature-defaults.md) green for minimal and alloc builds.
- **Follow-on:** prove an alloc-compiling analyzer kernel in-tree. Reassess package extraction and consumer-owned ports only after concrete host consumers demonstrate stable seams.

# Parallel execution

Analyzer isolation is a separate batch and owns `analyzer/mod.rs`, `check.rs`, and `service.rs`. Schema isolation owns its remaining std-gated signatures and host-coupled modules. Leaf work outside those files may proceed according to the [parallel execution guide](../../parallel-execution.md).

# Recommendation

1. Completed in this phase: separate check values from retained document indexes and split calculation, execution, and report policy.
2. Move `Render`, ariadne/terminal output, `Analysis` path traversal, and `AnalysisReport` filesystem indexing into explicit in-tree host modules.
3. Make the remaining check/error/readability model kernel compile under `alloc` without enabling the current `analysis -> std` feature bundle.
4. Keep Vale, HTTP, filesystem, process, and database code concrete until repeated consumers prove a narrow port. Do not introduce generic host traits yet.
5. Do not create `acorn-analysis` until the in-tree portable boundary has its own green feature check and no host imports.

# Validation

Run `cargo check -p acorn-lib --no-default-features`, `cargo check -p acorn-lib --no-default-features --features alloc`, `make wasm`, and the capability-specific checks in the [portability roadmap](roadmap.md).

Verified 2026-09-02: `cargo check --workspace --all-targets`, `cargo check -p acorn-lib --no-default-features`, `cargo check -p acorn-lib --no-default-features --features alloc`, `make lint`, `cargo test -p acorn-lib analyzer::tests --lib`, and `cargo test -p acorn-lib analyzer::service::tests --lib`. The alloc check remains a library baseline and does not yet select analyzer modules; establishing that narrower analyzer gate is the next acceptance step.

# Acceptance criteria

`acorn-schema` compiles without concrete host services; analyzer model modules compile independently of concrete document, renderer, filesystem, network, process, and database services.

# Related plans

- [Plan dependency network](../../plan-network.md)
