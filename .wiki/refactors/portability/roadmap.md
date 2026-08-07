---
type: Plan
title: "Portability roadmap"
description: "Coordinate capability features, module separation, host adapters, WASM enablement, and dependency isolation."
difficulty: mixed
impact: critical
package: [workspace, acorn-lib, acorn-core, acorn-schema, acorn-host, acorn-wasm]
tags: [refactor, roadmap, no-std, wasm]
status: planned
---

# Target feature model

Retain the existing `core`, `alloc`, `std`, and `wasm` feature names on the compatibility facade. Give extracted portable crates only the core, alloc, and std capabilities they implement. Target selection must not enable browser dependencies; `acorn-wasm` owns browser bindings and adapters.

# Implementation order

1. Separate portable domain transformations from std adapters and establish capability-scoped preludes.
2. Introduce consumer-owned host traits and native implementations, then refactor selected analyzer and API call sites to accept injected capabilities.
3. Complete target-specific WASM dependency wiring and implement browser capabilities in `acorn-wasm`.
4. Add CI gates for the supported feature combinations.

# Dependency boundaries

- Keep `color-eyre` in std adapters and use a portable core error type.
- Gate `rayon` behind analysis and std.
- Isolate document/template and progress dependencies behind their export features.
- Enable `chrono/std` only through std capability wiring.
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
