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

`util` now has explicit core, alloc, and std ownership, while `crates/acorn-lib/src/prelude.rs` still combines capability-dependent exports and analyzer and I/O call sites still assume the flat prelude.

# Recommendation

Mirror the utility ownership with explicit `prelude::core`, `prelude::alloc`, and `prelude::std` modules, preserve flat compatibility re-exports, and restrict new module imports according to capability level.

# Acceptance criteria

Portable modules cannot compile when they accidentally import std-only prelude items.
