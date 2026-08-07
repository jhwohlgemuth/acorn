---
type: Plan
title: "Add no-std and WASM CI gates"
description: "Continuously check portable feature combinations and detect dependency leakage."
difficulty: low
impact: low
package: [workspace, acorn-lib, acorn-core, acorn-schema, acorn-wasm]
tags: [refactor, ci, no-std, wasm]
status: planned
---

# Recommendation

Add CI checks for the minimal build, alloc build, and WASM composition target. CI should invoke the same `make wasm` entry point documented for developers. Add dependency-tree deny checks for host reporting, terminal, database, process, native HTTP, async-runtime, and generic embedded-asset dependencies in portable configurations.

# Validation

```text
cargo check -p acorn-lib --no-default-features
cargo check -p acorn-lib --no-default-features --features alloc
make wasm
```

`make setup` installs `wasm32-unknown-unknown` for local development. CI images may install only that target before invoking `make wasm` rather than installing the complete cross-compilation target set.

# Acceptance criteria

Portable feature regressions fail CI before merge.
