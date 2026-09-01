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

Retain the existing `core`, `alloc`, `std`, and `wasm` feature names on the compatibility facade. Give extracted portable crates only the core, alloc, and std capabilities they implement. Target selection must not enable browser dependencies; `acorn-wasm` owns browser bindings and adapters.

# Current evidence

- `acorn-lib` is already `no_std` when `std` is disabled and exposes `core`, `alloc`, `wasm`, and `std` features.
- `acorn-core` is extracted under `crates/` and compiles in minimal, alloc, default, and facade WASM configurations. `acorn-lib` maps its legacy features to the new package and preserves root/error/utility paths through re-exports.
- Portable string/byte codecs live on schema types, while path/file behavior lives in the std-only `io::adapter` and `io::document` modules. OpenCode I/O, model-weight persistence, contextual RAD formatting, and RAD Markdown document indexing have moved outward.
- Schema validation no longer reads `ACORN_DATABASE_PATH`; license and programming-language catalog membership is a host-service concern rather than derive-time schema policy.
- Analyzer result DTOs no longer depend on Vale runtime configuration, but analyzer path/service/rendering code remains gated behind `analysis -> std`.
- The unresolved critical edge is completing the remaining schema and analyzer isolation before `acorn-schema` and `acorn-analysis` extraction.

# Project code boundaries

- **Update:** workspace `Cargo.toml`, `Makefile`, `.gitlab-ci.yml`, `crates/acorn-lib/Cargo.toml`, `crates/acorn-lib/src/lib.rs`, `prelude.rs`, `error.rs`, `schema/`, `validation/`, `analyzer/`, and `io/`; first-party manifests and imports as each extraction lands.
- **Create:** `crates/acorn-schema/`, `crates/acorn-analysis/`, `crates/acorn-api/`, `crates/acorn-store/`, `crates/acorn-export/`, `crates/acorn-host/`, `crates/acorn-wasm/`, `crates/acorn-service/`, and `crates/acorn-cmd/`, one accepted boundary at a time.
- **Delete:** implementation modules from `acorn-lib` only after their compatibility paths re-export extracted code; no public facade path is deleted by this plan.

# Dependencies

This roadmap is governed by the [crate boundary architecture](../crate-boundary-architecture.md) and detailed in the [plan dependency network](../../plan-network.md). Dependency classification and std isolation run first; host traits follow proven seams; WASM adapters follow portable domains; CI expands continuously as each package appears.

# Parallel execution

This roadmap coordinates work and does not own a concurrent implementation branch. Active std-isolation or extraction work holds the library-root and module-index integration locks. Orthogonal leaf work may proceed in analyzer readability, remote create, Python bindings, OOXML, and a provider-specific API module when those agents avoid manifests, module indexes, CLI roots, and shared build/CI files. Use the verified batch in the [parallel execution guide](../../parallel-execution.md).

# Implementation order

1. Finish the named schema std-isolation blockers and execute the [`acorn-schema` extraction](acorn-schema-extraction.md).
2. Separate analyzer models and supplied-content transformations from document, renderer, filesystem, process, HTTP, and database services; extract `acorn-analysis`.
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
- [Extract acorn-schema](acorn-schema-extraction.md)
- [Host abstraction traits](host-abstraction-traits.md)
- [WASM feature path](wasm-feature-path.md)
- [no-std and WASM CI](no-std-wasm-ci.md)
- [Plan dependency network](../../plan-network.md)
