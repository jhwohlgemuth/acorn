---
type: Plan
title: "Tighten dependency feature defaults"
description: "Move std-dependent dependency features into explicit crate capability wiring."
difficulty: medium
impact: low
package: [acorn-lib, workspace]
tags: [refactor, dependencies, no-std]
status: in-progress
---

# Evidence

Host reporting, terminal formatting, diff, and generic embedded-asset dependencies are now optional and selected through `std`; the alloc feature enables `serde/alloc` instead of `serde/std`. The remaining dependency tree still needs classification before extraction.

# Recommendation

Prefer core- or alloc-compatible workspace defaults and enable std-dependent features from `acorn-lib`'s `std` feature.

# Acceptance criteria

No-default-feature dependency trees contain only the capabilities required by the selected build.

# Related plans

- [Reduce macOS compact-unwind pressure](macos-compact-unwind-size.md)
