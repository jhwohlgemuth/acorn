---
type: Plan
title: "Portability roadmap"
description: "Coordinate capability features, module separation, host adapters, WASM enablement, and dependency isolation."
difficulty: mixed
impact: critical
package: [workspace, acorn-lib, acorn-core, acorn-schema, acorn-analysis, acorn-host, acorn-wasm]
tags: [refactor, roadmap, no-std, wasm]
status: in-progress
---

# Target feature model

Give extracted portable crates only the core, alloc, and std capabilities they implement. Migrate consumers directly and remove obsolete `acorn-lib` forwarding features at an explicit semver boundary. Target selection must not enable browser dependencies; `acorn-wasm` owns browser bindings and adapters.

# Current evidence

- `acorn-lib` is already `no_std` when `std` is disabled and exposes `core`, `alloc`, `wasm`, and `std` features.
- `acorn-core` is extracted under `crates/` and compiles in minimal, alloc, default, and WASM-target configurations. Root domain types, errors, and portable utilities are imported directly from `acorn-core`; `acorn-lib` does not re-export them.
- Portable string/byte codecs live on schema types, while path/file behavior lives in the std-only `io::adapter` and `io::document` modules. OpenCode I/O, model-weight persistence, contextual RAD formatting, and RAD Markdown document indexing have moved outward.
- Schema validation no longer reads `ACORN_DATABASE_PATH`; license and programming-language catalog membership is a host-service concern rather than derive-time schema policy.
- Analyzer result DTOs no longer depend on Vale runtime configuration, but analyzer path/service/rendering code remains gated behind `analysis -> std`.
- `acorn-schema` now singularly owns schema models, validation, crosswalks, codecs, vocabulary assets, geonames, and prompt templates. First-party consumers import it directly; `acorn-lib` retains only host adapters and host assets.
- The unresolved critical edge is making the analyzer model/calculation kernel compile independently under `alloc`. Extraction is not scheduled merely because source-level separation has begun.

# Project code boundaries

- **Update:** workspace `Cargo.toml`, `Makefile`, `.gitlab-ci.yml`, `crates/acorn-lib/Cargo.toml`, `crates/acorn-lib/src/lib.rs`, `error.rs`, `analyzer/`, and `io/`; `crates/acorn-core/src/prelude.rs`; first-party manifests and imports as each extraction lands.
- **Create:** in-tree analyzer model/calculation and host execution/rendering module boundaries first; future packages only after their accepted boundaries compile independently.
- **Delete:** implementation modules from `acorn-lib` after consumers use the owning crate directly; compatibility paths are not required when an explicit breaking ownership transfer is accepted.

# Dependencies

This roadmap is governed by the [crate boundary architecture](../crate-boundary-architecture.md) and detailed in the [plan dependency network](../../plan-network.md). Dependency classification and in-tree std isolation run first. Generic host traits and `acorn-analysis` extraction are deferred until the portable boundary is independently green and real alternate consumers exist.

# Parallel execution

This roadmap coordinates work and does not own a concurrent implementation branch. Active std-isolation or extraction work holds the library-root and module-index integration locks. Orthogonal leaf work may proceed in analyzer readability, remote create, Python bindings, OOXML, and a provider-specific API module when those agents avoid manifests, module indexes, CLI roots, and shared build/CI files. Use the verified batch in the [parallel execution guide](../../parallel-execution.md).

# Implementation order

1. Separate analyzer models and supplied-content transformations from document, renderer, filesystem, process, HTTP, and database services in-tree; add an alloc-only compile gate.
2. Reassess whether the resulting boundary warrants `acorn-analysis`; do not extract it or add generic host traits by default.
3. Stabilize API, store, export, and service contracts before extracting their packages one boundary at a time.
4. Complete target-specific browser wiring in `acorn-wasm`, native implementations in `acorn-host`, and CI gates for every supported feature combination.

# Dependency boundaries

- Keep `color-eyre` in std adapters and use a portable core error type.
- Gate `rayon` behind analysis and std.
- Isolate document/template and progress dependencies behind their export features.
- Enable `jiff/std` only through std capability wiring.
- Keep database, HTTP, async runtime, server, and process dependencies optional and std-only.
- Keep `wasm-bindgen`, `js-sys`, and browser implementations in `acorn-wasm`; shared services depend on port traits rather than native or browser adapters.

# Validation

Check the default, no-default, alloc, std, and WASM target configurations independently.

# Related plans

- [Crate boundary architecture](../crate-boundary-architecture.md)
- [Isolate std-only modules](isolate-std-only-modules.md)
- [Host abstraction traits](host-abstraction-traits.md)
- [WASM feature path](wasm-feature-path.md)
- [no-std and WASM CI](no-std-wasm-ci.md)
- [Plan dependency network](../../plan-network.md)
