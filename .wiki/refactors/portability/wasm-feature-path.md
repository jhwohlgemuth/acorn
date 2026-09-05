---
type: Plan
title: "Complete the WASM feature path"
description: "Build browser-safe adapters on top of the existing wasm feature without leaking them into other builds."
difficulty: medium
impact: high
package: [acorn-lib, acorn-core, acorn-schema, acorn-service, acorn-wasm]
tags: [refactor, wasm, portability]
status: planned
---

# Context

`acorn-lib` already defines legacy `core`, `alloc`, and `wasm` features. Portable crates should remain target-neutral; browser bindings and concrete browser services need a dedicated `acorn-wasm` adapter and composition crate, and consumers should migrate to it directly.

# Current evidence

- `make wasm` runs `cargo check -p acorn-lib --no-default-features --features wasm --target wasm32-unknown-unknown`.
- The current legacy `wasm` feature selects the allocation-capable portable surface. It is a temporary build check, not a compatibility contract for the extracted browser package.
- The check does not link a `.wasm` artifact, generate JavaScript bindings, or provide browser HTTP and storage adapters.
- Public build instructions live in [`docs/src/packages/acorn-web.md`](../../../docs/src/packages/acorn-web.md); this plan remains authoritative for the unfinished architecture.
- No `crates/acorn-wasm/` package, `wasm-bindgen` dependency, JavaScript bindings, or browser adapter implementation exists today.

# Project code boundaries

- **Update:** workspace `Cargo.toml`, `Makefile`, `.gitlab-ci.yml`, `docs/src/packages/acorn-web.md`, and first-party imports/features.
- **Create:** `crates/acorn-wasm/Cargo.toml`, `crates/acorn-wasm/src/lib.rs`, browser HTTP/storage adapter modules, JS-facing DTO conversions, integration tests, and package-generation configuration.
- **Delete:** the documentation caveat that the target check is the complete web surface once a consumable artifact exists; no native adapter is moved into this crate.

# Dependencies

- **Prerequisites:** portable `acorn-core` and `acorn-schema` boundaries plus meaningful consumer-owned traits from [host abstraction traits](host-abstraction-traits.md).
- **Follow-on:** expand [no-std and WASM CI](no-std-wasm-ci.md) from current-package compilation to artifact and browser-adapter checks.

# Recommendation

1. Add target-specific `wasm-bindgen` and `js-sys` wiring only in `acorn-wasm`.
2. Keep browser dependencies out of non-WASM no-std builds.
3. Implement browser services for consumer-owned port traits and compose them with shared service use cases without depending on `acorn-host`.
4. Replace the public guide's portability-check caveat with artifact generation and integration instructions when `acorn-wasm` becomes consumable.

# Validation

```text
rustup target add wasm32-unknown-unknown
make wasm
cargo tree -p acorn-lib --no-default-features --features wasm --target wasm32-unknown-unknown
```

# Acceptance criteria

The WASM target compiles with only its intended dependencies, `acorn-wasm` is the only browser-binding owner, supported service use cases run through explicit browser adapters, and the public developer guide documents how to build and consume the resulting package.

# Related plans

- [Plan dependency network](../../plan-network.md)
