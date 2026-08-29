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

Define focused traits for file storage, HTTP, clock, entropy, and process execution. Provide std implementations and WASM implementations where the capability is meaningful.

# Current evidence

Concrete capabilities are imported directly by `analyzer/`, `io/api/`, `io/document/`, `io/download.rs`, `io/source.rs`, and `doctor/`. No shared host-port module exists, and creating one before separating domain from adapters would preserve the current coupling behind traits.

# Project code boundaries

- **Update:** consumer modules in future `acorn-analysis`, `acorn-api`, `acorn-export`, and `acorn-service`; current seams under `crates/acorn-lib/src/analyzer/` and `src/io/` while boundaries are prepared.
- **Create:** narrow consumer-owned port modules in those crates; native implementations in `crates/acorn-host/`; browser-supported implementations in `crates/acorn-wasm/`; fake implementations beside consumer tests.
- **Delete:** direct construction of concrete filesystem, HTTP, clock, entropy, and process services inside portable domain/service functions.

# Dependencies

- **Prerequisites:** [std-only isolation](isolate-std-only-modules.md) identifies real capability seams, and `acorn-core`/`acorn-schema` extraction establishes portable ownership.
- **Follow-on:** [WASM feature path](wasm-feature-path.md) implements meaningful browser ports; native composition moves into `acorn-host`.

# Acceptance criteria

Selected analyzer and API flows consume injected host capabilities rather than concrete runtime libraries.

# Related plans

- [Isolate std-only modules](isolate-std-only-modules.md)
- [Complete the WASM feature path](wasm-feature-path.md)
- [Plan dependency network](../../plan-network.md)
