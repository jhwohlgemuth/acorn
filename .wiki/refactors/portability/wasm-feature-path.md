---
type: Plan
title: "Add browser services to acorn-wasm"
description: "Extend the minimal in-memory bindings with browser-safe adapters after concrete service ports are proven."
difficulty: medium
impact: high
package: [acorn-api, acorn-export, acorn-service, acorn-wasm]
tags: [refactor, wasm, portability]
status: planned
---

# Context

`acorn-wasm` now provides a minimal JavaScript package for in-memory Research Activity validation and readability analysis. The remaining feature path adds only browser services justified by concrete workflows; portable crates remain target-neutral.

# Current evidence

- `crates/acorn-wasm` owns `wasm-bindgen` and `serde-wasm-bindgen` integration and depends only on the portable schema and analyzer feature paths.
- `make wasm-package` generates a web-target package and runs JavaScript-facing tests; `make check-portable` enforces its dependency boundary.
- The crates.io release sequence publishes and waits for `acorn-schema` and `acorn-lib` before publishing `acorn-wasm`.
- The bindings expose supplied-value validation and readability only. No browser HTTP, storage, document, process, persistence, or workflow adapter exists.
- Public build and consumption instructions live in [`docs/src/packages/acorn-web.md`](../../../docs/src/packages/acorn-web.md).

# Project code boundaries

- **Update:** `crates/acorn-wasm`, workspace build/CI wiring, the public web guide, and first-party service composition.
- **Create:** browser adapter modules and tests only for accepted service ports.
- **Delete:** no minimal in-memory binding; native adapters remain outside this crate.

# Dependencies

- **Satisfied:** portable domain crates, the analyzer kernel, minimal JS bindings, artifact generation, and package dependency gates.
- **Prerequisites:** select a concrete browser workflow, stabilize its service boundary, and introduce only the consumer-owned HTTP or storage ports demonstrated by native and browser implementations.
- **Follow-on:** expand [no-std and WASM CI](no-std-wasm-ci.md) with browser-adapter integration tests.

# Recommendation

1. Choose one browser-supported service use case before defining an adapter contract.
2. Put each narrow port with the service that consumes it and provide native, browser, and fake implementations.
3. Keep `js-sys`, `web-sys`, and browser implementations confined to `acorn-wasm`.
4. Compose the selected use case without depending on `acorn-host`, then document and test it in a browser runtime.

# Validation

```text
make check-portable
cargo tree -p acorn-wasm --target wasm32-unknown-unknown
```

# Acceptance criteria

At least one accepted service use case runs through an explicit browser adapter, browser dependencies remain confined to `acorn-wasm`, and browser integration tests cover its failure behavior.

# Related plans

- [Plan dependency network](../../plan-network.md)
