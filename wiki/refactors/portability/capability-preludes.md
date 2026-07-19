---
type: Plan
title: "Create strict capability preludes"
description: "Make core, alloc, and std imports explicit so modules cannot acquire host APIs accidentally."
difficulty: low
impact: medium
package: [acorn-lib]
tags: [refactor, prelude, no-std]
status: planned
---

# Evidence

`acorn-lib/src/prelude.rs` has capability-dependent exports, while analyzer, utility, and I/O call sites still assume host APIs.

# Recommendation

Expose explicit `prelude::core`, `prelude::alloc`, and `prelude::std` modules and restrict imports according to each module's capability level.

# Acceptance criteria

Portable modules cannot compile when they accidentally import std-only prelude items.
