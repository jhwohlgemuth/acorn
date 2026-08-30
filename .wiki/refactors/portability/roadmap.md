---
type: Plan
title: "Portability roadmap"
description: "Coordinate capability features, module separation, host adapters, WASM enablement, and dependency isolation."
difficulty: mixed
impact: critical
package: [workspace, acorn-lib, acorn-core, acorn-schema, acorn-host, acorn-wasm]
tags: [refactor, roadmap, no-std, wasm]
status: in-progress
priority: next
---

# Target feature model

Retain the existing `core`, `alloc`, `std`, and `wasm` feature names on the compatibility facade. Give extracted portable crates only the core, alloc, and std capabilities they implement. Target selection must not enable browser dependencies; `acorn-wasm` owns browser bindings and adapters.

# Current evidence

- `acorn-lib` is already `no_std` when `std` is disabled and exposes `core`, `alloc`, `wasm`, and `std` features.
- Portable string/byte codecs live on schema types, while path/file behavior lives in the std-only `io::adapter` module. Minimal, alloc, and WASM checks pass. `schema/mod.rs` still uses `prelude::std::PathBuf` in a handful of std-gated signatures, and `analyzer` remains entirely gated behind `analysis -> std`.
- Only the existing product crates (`acorn-lib`, `acorn-cli`, `acorn-macros`, `acorn-py`, and `acorn-tui`) exist. None of the target boundary crates has been created.
- The unresolved critical edge in the [crate boundary architecture](../crate-boundary-architecture.md) is isolating the remaining std-only schema and analyzer capabilities before extracting portable crates.

# Project code boundaries

- **Update:** workspace `Cargo.toml`, `Makefile`, `.gitlab-ci.yml`, `crates/acorn-lib/Cargo.toml`, `crates/acorn-lib/src/lib.rs`, `prelude.rs`, `error.rs`, `schema/`, `validation/`, `analyzer/`, and `io/`; first-party manifests and imports as each extraction lands.
- **Create:** `crates/acorn-core/`, `crates/acorn-schema/`, `crates/acorn-analysis/`, `crates/acorn-api/`, `crates/acorn-store/`, `crates/acorn-export/`, `crates/acorn-host/`, `crates/acorn-wasm/`, `crates/acorn-service/`, and `crates/acorn-cmd/`, one accepted boundary at a time.
- **Delete:** implementation modules from `acorn-lib` only after their compatibility paths re-export extracted code; no public facade path is deleted by this plan.

# Dependencies

This roadmap is governed by the [crate boundary architecture](../crate-boundary-architecture.md) and detailed in the [plan dependency network](../../plan-network.md). Dependency classification and std isolation run first; host traits follow proven seams; WASM adapters follow portable domains; CI expands continuously as each package appears.

# Parallel execution

This roadmap coordinates work and does not own a concurrent implementation branch. Active std-isolation or extraction work holds the library-root and module-index integration locks. Orthogonal leaf work may proceed in analyzer readability, remote create, Python bindings, OOXML, and a provider-specific API module when those agents avoid manifests, module indexes, CLI roots, and shared build/CI files. Use the verified batch in the [parallel execution guide](../../parallel-execution.md).

# Implementation order

1. Isolate the remaining std-gated schema and analyzer capabilities, introduce consumer-owned host traits and native implementations, and refactor selected analyzer and API call sites to accept injected capabilities.
2. Complete target-specific WASM dependency wiring and implement browser capabilities in `acorn-wasm`.
3. Add CI gates for the supported feature combinations.

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
