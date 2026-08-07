---
type: Plan
title: "Extract acorn-cmd as a standalone crate"
description: "Move command macros and runtime helpers into a reusable crate independent of acorn-lib."
difficulty: not-assessed
impact: not-assessed
package: [acorn-lib, acorn-macros, acorn-cmd]
tags: [refactor, commands, macros, crate-extraction]
status: planned
---

# Goal

Extract `cmd!`, `args!`, and `cmd_sh_words!` into a standalone `acorn-cmd` crate usable without the rest of `acorn-lib`.

# Current locations

- Proc macro `cmd_sh_words!`: `acorn-macros/src/lib.rs`
- Declarative macros `cmd!` and `args!`: `acorn-lib/src/util/macros.rs`
- Runtime helpers: `acorn-lib/src/util/cmd.rs`
- Tests: `acorn-lib/src/util/tests/cmd.rs`

# Recommendation

| Step | Action | Dependency |
|---|---|---|
| 1 | Create the `acorn-cmd` package and required macro/library structure | None |
| 2 | Move command-word parsing and proc-macro expansion from `acorn-macros` | Step 1 |
| 3 | Move declarative macros and runtime helpers from `acorn-lib` | Step 2 |
| 4 | Add the command-line word parsing dependency | Step 3 |
| 5 | Add integration tests matching current behavior | Step 4 |
| 6 | Re-export `cmd_sh_words!` from `acorn-cmd` through `acorn-lib` as needed | Step 5 |
| 7 | Replace the `acorn-lib` implementation feature with the new package dependency | Step 6 |

# Acceptance criteria

The extracted crate passes the existing macro and runtime-helper coverage, and `acorn-lib` consumers retain compatible imports during migration.

# Related plans

- [Crate boundary architecture](crate-boundary-architecture.md)
