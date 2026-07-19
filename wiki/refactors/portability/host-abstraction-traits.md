---
type: Plan
title: "Introduce host abstraction traits"
description: "Represent runtime services through interfaces that can have std and WASM implementations."
difficulty: medium
impact: medium
package: [acorn-lib]
tags: [refactor, adapters, no-std, wasm]
status: planned
---

# Recommendation

Define focused traits for file storage, HTTP, clock, entropy, and process execution. Provide std implementations and WASM implementations where the capability is meaningful.

# Acceptance criteria

Selected analyzer and API flows consume injected host capabilities rather than concrete runtime libraries.

# Related plans

- [Isolate std-only modules](isolate-std-only-modules.md)
- [Complete the WASM feature path](wasm-feature-path.md)
