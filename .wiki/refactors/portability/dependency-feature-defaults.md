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

Host reporting, terminal formatting, diff, and generic embedded-asset dependencies are now optional and selected through `std`; the alloc feature enables `serde/alloc` instead of `serde/std`. A current minimal tree still includes `async-trait`, `futures`, `glob`, `shell-words`, and the full Sophia stack because they are unconditional in `crates/acorn-lib/Cargo.toml`; their actual portable ownership must be classified before extraction. `analysis`, `agentic`, `cmd`, and `shell-lint` still all imply `std`, and the broad `std` feature selects unrelated host capabilities.

# Project code boundaries

- **Update:** workspace `Cargo.toml`, `crates/acorn-lib/Cargo.toml`, every new extracted-crate manifest, and `Makefile` commands that encode supported feature combinations.
- **Create:** per-crate feature tables in manifests introduced by the [crate boundary architecture](../crate-boundary-architecture.md); dependency-deny checks in CI under [no-std and WASM CI](no-std-wasm-ci.md).
- **Delete:** unconditional portable-crate dependencies that have no portable call site; broad facade wiring only after compatibility mappings replace it.

# Dependencies

- **Prerequisite for:** every crate extraction and the WASM composition crate.
- **Can proceed with:** [std-only isolation](isolate-std-only-modules.md), provided every manifest change keeps minimal, alloc, std, and WASM checks independently green.

# Parallel execution

Dependency investigation is read-only and can run with any lane. Manifest edits acquire the workspace-graph lock and serialize with Phase 1.2 integration, crate extraction, WASM, macOS size work, acorn-py cutover, and dependency-adding features. Prefer a separate manifest-only commit so leaf agents can rebase once and continue.

# Recommendation

Prefer core- or alloc-compatible workspace defaults and enable std-dependent features from `acorn-lib`'s `std` feature.

# Acceptance criteria

No-default-feature dependency trees contain only the capabilities required by the selected build.

# Related plans

- [Reduce macOS compact-unwind pressure](macos-compact-unwind-size.md)
- [Plan dependency network](../../plan-network.md)
