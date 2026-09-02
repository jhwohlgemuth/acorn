---
type: Plan
title: "Tighten dependency feature defaults"
description: "Move std-dependent dependency features into explicit crate capability wiring."
difficulty: medium
impact: high
package: [acorn-lib, workspace]
tags: [refactor, dependencies, no-std]
status: in-progress
priority: next
---

# Evidence

Host reporting (`color-eyre`), terminal formatting (`console`, `owo-colors`), diff (`similar`) and generic embedded-asset (`rust-embed`, `anydoc`/`tera` asset handling) dependencies are optional and selected through `std`; the `alloc` feature enables `serde/alloc` instead of `serde/std` and `acorn-core/alloc`.

Verified 2026-09-01 against `cargo tree -p acorn-lib --no-default-features` (commit `d05ad1b9` + `28615bac`): `async-trait`, `futures`, `glob` and `shell-words` are now `optional = true` in `crates/acorn-lib/Cargo.toml:35,51-52,77` and wired behind `std` (`Cargo.toml:148-189` `dep:async-trait`, `dep:futures`, `dep:glob`, `dep:shell-words`). The minimal and `alloc` trees no longer resolve them; only `sophia` remains unconditional (`Cargo.toml:97`, `default-features = false`) — deliberately retained because `schema/namespaces.rs:5` uses `sophia::api::ns::Namespace` in portable RDF namespace handling with no host service.

Remaining broad wiring is unresolved rather than a compatibility contract: `analysis`, `agentic`, `cmd`, and `shell-lint` still imply `std`, and the `std` feature still selects unrelated host capabilities. Narrow the in-tree analyzer dependency cut first. Do not add compatibility mappings or extract a crate merely to hide this feature coupling.

# Project code boundaries

- **Update:** workspace `Cargo.toml`, `crates/acorn-lib/Cargo.toml`, every new extracted-crate manifest, and `Makefile` commands that encode supported feature combinations.
- **Create:** per-crate feature tables in manifests introduced by the [crate boundary architecture](../crate-boundary-architecture.md); dependency-deny checks in CI under [no-std and WASM CI](no-std-wasm-ci.md).
- **Delete:** unconditional portable-crate dependencies that have no portable call site; broad feature wiring once an independently compiling in-tree capability boundary proves the narrower set.

# Dependencies

- **Prerequisite for:** every crate extraction and the WASM composition crate.
- **Can proceed with:** [std-only isolation](isolate-std-only-modules.md), provided every manifest change keeps minimal, alloc, std, and WASM checks independently green.

# Parallel execution

Dependency investigation is read-only and can run with any lane. Manifest edits acquire the workspace-graph lock and serialize with std isolation, crate extraction, WASM, macOS size work, acorn-py cutover, and dependency-adding features. Prefer a separate manifest-only commit so leaf agents can rebase once and continue.

# Recommendation

Prefer core- or alloc-compatible workspace defaults and enable std-dependent features from `acorn-lib`'s `std` feature.

# Acceptance criteria

No-default-feature dependency trees contain only the capabilities required by the selected build.

# Related plans

- [Reduce macOS compact-unwind pressure](macos-compact-unwind-size.md)
- [Plan dependency network](../../plan-network.md)
