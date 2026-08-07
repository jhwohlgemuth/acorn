---
type: Plan
title: "Introduce a portable core error surface"
description: "Keep rich color-eyre reports in std adapters while portable code uses a core-compatible error type."
difficulty: medium
impact: high
package: [acorn-lib]
tags: [refactor, no-std, errors]
status: complete
---

# Evidence

`AcornError` and `AcornResult` provide the core-compatible error contract. `ApiResult` is std-only, `color-eyre` is enabled only by the `std` feature, and minimal/alloc dependency trees no longer resolve it.

# Recommendation

1. Introduce a core-compatible `AcornError` using `core::fmt` and an appropriate error derive.
2. Convert it to `color-eyre` reports only in std adapters.
3. Make result aliases capability-aware without changing user-facing CLI diagnostics.

# Acceptance criteria

Core builds expose typed errors without pulling in terminal-oriented reporting dependencies.
