---
type: Plan
title: "Add no-std and WASM CI gates"
description: "Continuously check portable feature combinations and detect dependency leakage."
difficulty: low
impact: high
package: [workspace, acorn-lib, acorn-core, acorn-schema, acorn-wasm]
tags: [refactor, ci, no-std, wasm]
status: in-progress
---

# Current slice

The dedicated `test-portability` job runs `make check-portable`, covering the core, alloc, minimal, analyzer-only native, analyzer-only WASM, and `acorn-wasm` configurations. It also generates the web package and runs JavaScript-facing tests. `scripts/check-portable-dependencies.sh` rejects host reporting, terminal, database, process, native HTTP, async-runtime, and embedded-asset dependencies from the analyzer-only and `acorn-wasm` graphs.

# Current evidence

`Makefile` exposes `check-portable` as the shared local/CI contract, `wasm` as the target-specific compile subset, and `wasm-package` as the artifact and JavaScript-test gate. `.gitlab-ci.yml` installs the pinned package tool when necessary and enforces the complete matrix in a distinct job. Browser-adapter cases remain pending until a concrete browser service is added.

# Project code boundaries

- **Update:** `.gitlab-ci.yml`, `Makefile`, workspace `Cargo.toml`, and `docs/src/packages/acorn-web.md` when the supported matrix changes.
- **Create:** CI scripts or `xtask` checks only if inline Cargo-tree assertions become too complex; add browser-runtime jobs as concrete adapters appear.
- **Delete:** obsolete in-tree package jobs after equivalent direct owning-crate coverage is active.

# Dependencies

- **Completed:** gate the existing crates' minimal, alloc, analyzer-only native, and analyzer-only WASM checks and deny host-only analyzer dependencies.
- **Expand after:** each extraction and the [WASM feature path](wasm-feature-path.md); this remains a rolling exit gate rather than a final one-time task.

# Validation

```text
cargo check -p acorn-lib --no-default-features
cargo check -p acorn-lib --no-default-features --features alloc
make check-portable
```

`make setup` installs `wasm32-unknown-unknown` for local development. CI images may install only that target before invoking `make check-wasm` rather than installing the complete cross-compilation target set.

# Acceptance criteria

Portable feature regressions in current packages fail CI before merge; `acorn-wasm` must compile, generate a package, pass JavaScript-facing tests, and exclude host dependencies. Future browser adapters receive equivalent runtime gates when they exist.

# Related plans

- [Plan dependency network](../../plan-network.md)
