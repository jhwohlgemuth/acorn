---
type: Plan
title: "Extract acorn-cmd as a standalone crate"
description: "Move command macros, compile-time parsing, and process runtime helpers into focused crates independent of acorn-lib."
difficulty: medium
impact: medium
package: [workspace, acorn-cli, acorn-lib, acorn-macros, acorn-cmd, acorn-cmd-macros]
tags: [refactor, commands, macros, crate-extraction]
status: planned
---

# Goal

Extract the existing command surface into `acorn-cmd`, backed by a focused `acorn-cmd-macros` proc-macro companion, so first-party consumers can construct and execute commands without depending on `acorn-lib`.

# Current evidence

- `cmd!` and `args!` are defined in `crates/acorn-lib/src/util/macros.rs`; their runtime helpers are in `crates/acorn-lib/src/util/cmd.rs`; `cmd_sh_words!` and its parser are mixed into `crates/acorn-macros/src/lib.rs` with unrelated derives.
- The command surface now includes literal interpolation and splats, dynamic shell-word parsing, direct/bash/PowerShell execution, status and string-return forms, working directories, child environment overrides, detached spawning, redaction, and command rendering. Freeze this grammar for the extraction; do not add forms while ownership moves.
- The `cmd` feature exports macros, but the runtime module is gated by broader `analysis` or `std` capabilities and runtime parsing obtains `shell-words` through `cli-util`. The extracted crate needs a self-contained feature contract.
- Runtime logging depends on `acorn-lib::util::Label`, and shell-lint configuration depends on an `acorn-lib` environment constant. Both are presentation/configuration details rather than architectural dependencies.
- `acorn-lib` calls runtime helpers directly from OCI and synchronization code. `acorn-cli` imports `cmd!`, `args!`, and `command_string` through `acorn-lib`; all must migrate to the owning crate.
- The current baseline passes 35 command/runtime tests and 7 command-word parser tests.

# Project code boundaries

- **Update:** workspace and affected crate manifests; command definitions and tests under `crates/acorn-lib/src/util/`; command-using `acorn-lib` I/O/analyzer modules; `crates/acorn-macros/src/lib.rs`; and direct imports in `acorn-cli` commands and tests.
- **Create:** `crates/acorn-cmd/{Cargo.toml,src/,tests/}` and `crates/acorn-cmd-macros/{Cargo.toml,src/,tests/}`.
- **Delete:** command parsing from `acorn-macros`; command implementation and tests from `acorn-lib`; the `acorn-lib` `cmd` feature and command re-exports after all consumers migrate directly.

# Dependencies

- The earlier `acorn-core` prerequisite is satisfied, but neither new crate should depend on `acorn-core` or `acorn-lib`.
- Coordinate the new manifests and explicit `shell-lint` capability with [dependency feature defaults](portability/dependency-feature-defaults.md). The extraction can otherwise proceed independently of analyzer, API, and host-trait work.
- Preserve current roadmap priority and serialize workspace manifests, `acorn-macros`, `acorn-lib` roots, and CLI import changes with their current owners.

# Public interface and feature contract

- `acorn-cmd` owns and exports `cmd!` and `args!`, and re-exports `cmd_sh_words!` from `acorn-cmd-macros`. Macro expansions resolve only through `$crate` paths owned by `acorn-cmd`, including when the package is renamed by a downstream consumer.
- `acorn-cmd::runtime` owns the process helpers required by macro expansion and the directly used `command_string`, `redact`, `run_output`, and `run_output_with_stdin` functions. Keep implementation-only helpers documented but hidden where downstream macro expansion requires public visibility.
- `acorn-cmd-macros` is a `proc-macro` crate containing only command literal parsing and expansion. It must not depend on `acorn-lib`, `acorn-core`, or the existing multi-purpose `acorn-macros` crate.
- `acorn-cmd` is a native `std` crate. `shell-words` and `tracing` are direct runtime dependencies. `shell-lint` is the sole optional capability and enables `shuck-linter` plus `shuck-parser`; it is not a default feature.
- First-party `std` consumers that currently receive shell linting enable `acorn-cmd/shell-lint` explicitly; narrower analysis-only consumers retain the current no-lint behavior.
- Preserve `ACORN_SHELL_LINT_MIN_SEVERITY`, redaction rules, tracing fields, and the visible run label inside `acorn-cmd` without importing ACORN-wide constants or terminal utilities.
- First-party crates depend on `acorn-cmd` directly. Do not retain `acorn-lib` macro, module, or feature forwarding after the cutover.

# Implementation sequence

1. Record the current macro forms and runtime functions as the compatibility baseline; accept no grammar additions during extraction.
2. Create `acorn-cmd-macros`, move `CmdWord`, command-word parsing, `cmd_sh_words!`, and their parser tests, and add compile-pass/fail coverage for interpolation boundaries.
3. Create `acorn-cmd`, move `cmd!`, `args!`, runtime execution, rendering, redaction, logging, and shell linting, then rewrite macro support paths to `acorn_cmd` ownership.
4. Define the minimal public runtime module and explicit feature wiring, preserving current behavior with and without `shell-lint`.
5. Move command/runtime tests into `acorn-cmd`, including cross-platform direct, shell, status, try, environment, directory, spawn, stdin, rendering, and redaction cases.
6. Migrate `acorn-lib`, `acorn-cli`, and their tests to direct `acorn-cmd` imports; remove the old implementation, proc-macro code, re-exports, and feature wiring.
7. Run focused crate tests, supported feature checks, full workspace lint/tests, and package dry-runs for both new crates.

# Validation

- `cargo test -p acorn-cmd-macros` passes parser and UI compile tests.
- `cargo test -p acorn-cmd` passes the migrated behavior suite on supported platforms; `cargo check -p acorn-cmd --no-default-features` and `cargo check -p acorn-cmd --features shell-lint` both pass.
- `cargo tree` confirms neither new crate depends on `acorn-lib` or `acorn-core`, and the non-lint build excludes `shuck-linter` and `shuck-parser`.
- Searches find no command implementation or `cmd_sh_words!` parser in `acorn-lib`/`acorn-macros`, and no first-party imports through `acorn::{cmd, args}` or `acorn::util::cmd`.
- `make lint`, `make test`, and `cargo package --workspace` or focused package dry-runs pass after migration.

# Acceptance criteria

- Existing command forms retain their argument boundaries, return types, working-directory and environment behavior, shell selection, exit-status handling, redaction, and diagnostics.
- `acorn-cmd` provides the complete command capability under its own dependencies and features; enabling a command macro never relies on an unrelated `acorn-lib` feature.
- `acorn-cmd-macros` owns only command parsing, while `acorn-macros` retains only its remaining schema/database/general derive responsibilities.
- First-party consumers import the owning crate directly, with no `acorn-lib` compatibility facade or obsolete `cmd` feature.
- Both new packages pass focused tests, feature checks, packaging checks, and the full workspace validation suite.

# Related plans

- [Crate boundary architecture](crate-boundary-architecture.md)
- [Dependency feature defaults](portability/dependency-feature-defaults.md)
