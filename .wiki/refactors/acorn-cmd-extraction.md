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

# Project code boundaries

- **Update:** workspace `Cargo.toml`, `crates/acorn-lib/Cargo.toml`, `src/util/{cmd,macros}.rs`, `src/util/tests/cmd.rs`, `crates/acorn-macros/src/lib.rs`, and first-party imports/features.
- **Create:** `crates/acorn-cmd/{Cargo.toml,src/lib.rs}` plus focused tests.
- **Delete:** command implementation and obsolete command features from `acorn-lib` after first-party consumers migrate directly.

# Dependencies

Extract after portable shared values have an `acorn-core` home. It can then proceed independently of analyzer/API extraction and feeds the final ownership cleanup in [crate boundary architecture](crate-boundary-architecture.md).

# Goal

Extract `cmd!`, `args!`, and `cmd_sh_words!` into a standalone `acorn-cmd` crate usable without the rest of `acorn-lib`.

# Current locations

- Proc macro `cmd_sh_words!`: `crates/acorn-macros/src/lib.rs`
- Declarative macros `cmd!` and `args!`: `crates/acorn-lib/src/util/macros.rs`
- Runtime helpers: `crates/acorn-lib/src/util/cmd.rs`
- Tests: `crates/acorn-lib/src/util/tests/cmd.rs`

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
