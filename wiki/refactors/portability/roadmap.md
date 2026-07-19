---
type: Plan
title: "Portability roadmap"
description: "Coordinate capability features, module separation, host adapters, WASM enablement, and dependency isolation."
difficulty: mixed
impact: critical
package: [acorn-lib, workspace]
tags: [refactor, roadmap, no-std, wasm]
status: planned
---

# Target feature model

Retain the existing `core`, `alloc`, `std`, and `wasm` capability features. Layer domain features such as analysis, doctor, PowerPoint, agentic services, databases, and export formats on the minimum capability they require.

# Implementation order

1. Separate portable domain transformations from std adapters and establish capability-scoped preludes.
2. Introduce host traits and std implementations, then refactor selected analyzer and API call sites.
3. Complete target-specific WASM dependency wiring and adapters.
4. Add CI gates for the supported feature combinations.

# Dependency boundaries

- Keep `color-eyre` in std adapters and use a portable core error type.
- Gate `rayon` behind analysis and std.
- Isolate document/template and progress dependencies behind their export features.
- Enable `chrono/std` only through std capability wiring.
- Keep database, HTTP, async runtime, server, and process dependencies optional and std-only.

# Validation

Check the default, no-default, alloc, std, and WASM target configurations independently.

# Related plans

- [Isolate std-only modules](isolate-std-only-modules.md)
- [Host abstraction traits](host-abstraction-traits.md)
- [WASM feature path](wasm-feature-path.md)
- [no-std and WASM CI](no-std-wasm-ci.md)
