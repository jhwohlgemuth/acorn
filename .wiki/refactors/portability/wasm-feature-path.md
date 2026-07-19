---
type: Plan
title: "Complete the WASM feature path"
description: "Build browser-safe adapters on top of the existing wasm feature without leaking them into other builds."
difficulty: medium
impact: high
package: [acorn-lib]
tags: [refactor, wasm, portability]
status: planned
---

# Context

`acorn-lib` already defines `core`, `alloc`, and `wasm` features. The remaining work is to make the WASM path useful and independently buildable.

# Recommendation

1. Add target-specific `wasm-bindgen` and `js-sys` adapter wiring where required.
2. Keep browser dependencies out of non-WASM no-std builds.
3. Implement WASM services for the host traits that can be supported in a browser.

# Acceptance criteria

The WASM target compiles with only its intended dependencies and exposes supported domain operations through explicit adapters.
