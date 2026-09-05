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

The dedicated `test-portability` job runs `make check-portable`, covering the core, alloc, minimal, analyzer-only native, and analyzer-only WASM configurations. `scripts/check-portable-dependencies.sh` rejects host reporting, terminal, database, process, native HTTP, async-runtime, and embedded-asset dependencies from the analyzer-only native and WASM graphs.

# Current evidence

`Makefile` exposes `check-portable` as the shared local/CI contract and retains `wasm` as the target-specific subset. `.gitlab-ci.yml` enforces the complete current-package matrix in a distinct job. Future `acorn-wasm` artifact and adapter jobs cannot be added until that package exists.

# Project code boundaries

- **Update:** `.gitlab-ci.yml`, `Makefile`, workspace `Cargo.toml`, and `docs/src/packages/acorn-web.md` when the supported matrix changes.
- **Create:** CI scripts or `xtask` checks only if inline Cargo-tree assertions become too complex; add per-crate jobs as `acorn-core`, `acorn-schema`, and `acorn-wasm` are created.
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

`make setup` installs `wasm32-unknown-unknown` for local development. CI images may install only that target before invoking `make wasm` rather than installing the complete cross-compilation target set.

# Acceptance criteria

Portable feature regressions in current packages fail CI before merge; future extracted and browser-composition packages receive equivalent gates when they exist.

# Related plans

- [Plan dependency network](../../plan-network.md)
