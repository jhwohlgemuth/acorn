---
type: Plan
title: "Add no-std and WASM CI gates"
description: "Continuously check portable feature combinations and detect dependency leakage."
difficulty: low
impact: low
package: [acorn-lib, workspace]
tags: [refactor, ci, no-std, wasm]
status: planned
---

# Recommendation

Add CI checks for the minimal build, alloc build, and WASM target, plus targeted dependency-tree inspection when feature leakage is suspected.

# Validation

```text
cargo check -p acorn-lib --no-default-features
cargo check -p acorn-lib --no-default-features --features alloc
cargo check -p acorn-lib --no-default-features --target wasm32-unknown-unknown --features wasm
```

# Acceptance criteria

Portable feature regressions fail CI before merge.
