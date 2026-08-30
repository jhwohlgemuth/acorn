---
type: Plan
title: "Add no-std and WASM CI gates"
description: "Continuously check portable feature combinations and detect dependency leakage."
difficulty: low
impact: high
package: [workspace, acorn-lib, acorn-core, acorn-schema, acorn-wasm]
tags: [refactor, ci, no-std, wasm]
status: planned
---

# Recommendation

Add CI checks for the minimal build, alloc build, and WASM composition target. CI should invoke the same `make wasm` entry point documented for developers. Add dependency-tree deny checks for host reporting, terminal, database, process, native HTTP, async-runtime, and generic embedded-asset dependencies in portable configurations.

# Current evidence

`Makefile` exposes the local WASM check, and the minimal, alloc, and WASM checks pass locally. `.gitlab-ci.yml` does not yet enforce that complete matrix or the dependency-deny policy. Future package jobs cannot be added until the corresponding crates exist.

# Project code boundaries

- **Update:** `.gitlab-ci.yml`, `Makefile`, workspace `Cargo.toml`, and `docs/src/packages/acorn-web.md` when the supported matrix changes.
- **Create:** CI scripts or `xtask` checks only if inline Cargo-tree assertions become too complex; add per-crate jobs as `acorn-core`, `acorn-schema`, and `acorn-wasm` are created.
- **Delete:** obsolete facade-only jobs after equivalent per-crate plus compatibility-facade coverage is active.

# Dependencies

- **Start now:** gate the existing facade's minimal, alloc, and WASM checks.
- **Expand after:** each extraction and the [WASM feature path](wasm-feature-path.md); this is a rolling exit gate rather than a final one-time task.

# Validation

```text
cargo check -p acorn-lib --no-default-features
cargo check -p acorn-lib --no-default-features --features alloc
make wasm
```

`make setup` installs `wasm32-unknown-unknown` for local development. CI images may install only that target before invoking `make wasm` rather than installing the complete cross-compilation target set.

# Acceptance criteria

Portable feature regressions fail CI before merge.

# Related plans

- [Plan dependency network](../../plan-network.md)
