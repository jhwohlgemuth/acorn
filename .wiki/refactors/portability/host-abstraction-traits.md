---
type: Plan
title: "Introduce host abstraction traits"
description: "Represent runtime services through interfaces that can have std and WASM implementations."
difficulty: medium
impact: medium
package: [acorn-analysis, acorn-api, acorn-export, acorn-service, acorn-host, acorn-wasm]
tags: [refactor, adapters, no-std, wasm]
status: planned
---

# Recommendation

Defer this plan. Do not introduce generic file, HTTP, clock, entropy, or process traits while the portable analyzer boundary is still being discovered. First separate concrete host execution and rendering from values and calculations in-tree. Add a consumer-owned port only when at least two real implementations or a tested alternate composition require the same narrow contract.

# Current evidence

Concrete capabilities are imported directly by `analyzer/`, `io/api/`, `io/document/`, `io/download.rs`, `io/source.rs`, and `doctor/`. `AnalysisContext` now separates concrete execution policy from calculation and reporting, but `analysis` still implies `std`; a generic port family now would encode the current accidental boundary.

# Project code boundaries

- **Update:** consumer modules in future `acorn-analysis`, `acorn-api`, `acorn-export`, and `acorn-service`; current seams under `crates/acorn-lib/src/analyzer/` and `src/io/` while boundaries are prepared.
- **Create:** narrow consumer-owned port modules in those crates; native implementations in `crates/acorn-host/`; browser-supported implementations in `crates/acorn-wasm/`; fake implementations beside consumer tests.
- **Delete:** direct construction of concrete filesystem, HTTP, clock, entropy, and process services inside portable domain/service functions.

# Dependencies

- **Prerequisites:** the [portability roadmap](roadmap.md) provides an independently compiling analyzer kernel; a concrete alternate host/browser/test implementation must still demonstrate each proposed seam.
- **Follow-on:** [WASM feature path](wasm-feature-path.md) implements meaningful browser ports; native composition moves into `acorn-host`.

# Acceptance criteria

Selected analyzer and API flows consume injected host capabilities rather than concrete runtime libraries.

# Related plans

- [Portability roadmap](roadmap.md)
- [Complete the WASM feature path](wasm-feature-path.md)
- [Plan dependency network](../../plan-network.md)
