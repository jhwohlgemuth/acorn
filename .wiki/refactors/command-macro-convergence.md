---
type: Plan
title: Centralize Rust subprocess construction through cmd! and args!
description: Route Rust child-process creation through the ACORN command macros while preserving native process behavior and platform boundaries.
difficulty: high
impact: medium
package: [acorn-cmd, acorn-cli, acorn-lib, acorn-host, acorn-daemon, acorn-macros, acorn-web, xtask]
tags: [refactor, subprocess, command, os:linux, os:macos, os:windows]
status: planned
execution: phased
phase_budget_hours: 5
---

# Outcome

Every Rust call site that constructs or runs a child process uses `cmd!`; every explicit argument collection supplied to a command is built with `args!`. Keep one private raw process-construction backend inside `acorn-cmd` to implement those macros. Preserve command argv bytes and order, environment, working directory, standard streams, process groups, timeouts, cancellation, exit handling, and platform behavior. This is a call-site and backend refactor, with no new shell evaluation or user-facing command syntax.

The existing literal and CLI-style forms of `cmd!` remain supported: they are parsed by the macro and do not create a separate argument collection at the call site. A shell script passed to `sh`, `bash`, or `pwsh` remains one intentional shell-language argument. For structured forms, including zero arguments, construct the collection with `args![]`; do not concatenate untrusted values into a shell command string. `sudo!` remains an ACORN macro only if its callers supply `args!` collections and it still delegates execution to `cmd!`.

# Current evidence — November 27, 2026

Source inspection used the indexed tree, a current Rust-source inventory, `a78440bf`, and the dirty worktree. This is a planning audit, not a passing implementation test. The November 13 portfolio-wide currency review is current this month.

| Verdict | Source observation | Planning consequence |
|---|---|---|
| verified | A Rust-source inventory finds 76 executable `Command::new(...)` expressions plus one `ProcessCommand::new(...)` alias in `crates/acorn-cli/tests/registry/check.rs:370`; one documentation example was excluded. Thirteen calls implement `acorn-cmd` itself (`src/lib.rs:678–737`, eight; `src/runtime.rs:124,246,436,499,519`, five). The other 64 are call sites across CLI, library, daemon, browser build script, xtask, and tests. | Migrate every call site and consolidate the necessary internal constructors to one private backend. Recount against the current tree before implementation because concurrent work may add or remove calls. |
| verified | `crates/acorn-cmd/src/lib.rs:267–803` exports `cmd!` and `args!`, including captured output, status, managed execution, spawn, environment, directory, and stdin-file forms. `runtime.rs:29–130` provides `CommandSpec` and its builder. A separate scan finds 25 `CommandSpec::new` expressions: four inside `cmd!`, 21 outside. | Add macro builder forms for custom `std::process::Command`, `tokio::process::Command`, and reusable `CommandSpec`; migrate direct `CommandSpec` call sites too. Keep the existing macro forms and return types stable. |
| verified | `crates/acorn-daemon/src/runner.rs:195–229,323–334` configures null/piped streams and starts a process group. `crates/acorn-lib/src/io/mcp/client.rs:717–731` uses Tokio `Command`, `env_clear`, `kill_on_drop`, and piped transport. | These need a prepared-command macro form; replacing them with a simple output/status call would change lifecycle behavior. |
| verified | `crates/acorn-cmd/tests/execution.rs:20–46` compiles a standalone helper with `rustc`; that helper spawns a descendant in `tests/fixtures/process_helper.rs:16`. `tests/downstream-host/src/main.rs:69–76` checks the renamed `acorn-cmd` dependency and uses both `CommandSpec::new` and an inline argument array. | Move the helper into a Cargo test binary so it can import the macros, and keep downstream macro-path qualification covered. |
| verified | `crates/acorn-web/build.rs:16,22` and `xtask/src/main.rs:114,305`, `xtask/src/benchmark_history.rs:309`, and `xtask/src/application_release/mod.rs:231` construct host processes. Their manifests do not currently declare `acorn-cmd` for those compilation contexts. | Add host/build dependencies under the Workspace graph lock; verify that browser WASM runtime dependencies stay unchanged. |
| verified | `crates/acorn-daemon/src/install/macos.rs:146–312` passes arrays to `sudo!`, and `crates/acorn-macros/src/lib.rs:1556–1567` formats those arguments after delegating to `cmd!`. Other `cmd!` callers still pass inline arrays or locally built collections; examples include `acorn-lib/src/io/mod.rs:930` and `acorn-cli/src/commands/create/runner/mod.rs:170–324`. | Audit explicit argument construction beyond the 77 raw constructors. Preserve `OsStr`/`OsString` data and redaction while changing callers to `args!`. |
| unresolved | No broad native platform run or new macro builder implementation has been attempted for this plan. | Accept each process-lifecycle class with focused tests, then run the workspace and portability gates on a stable tree. |

# Priority, boundaries, and migration contract

- Portfolio rank is immediately after [netrc support](../features/netrc-support.md) (NTR), ahead of gather extraction. This is a scheduling order, not a false code dependency: external NTR platform acceptance does not block command-only work once NTR releases overlapping Workspace graph, CLI, and host locks. Begin CMD-01 after NTR's higher-priority implementation owns no overlapping file or lock.
- Add `cmd!(build program, args![...])` for a configured `std::process::Command`, `cmd!(build async program, args![...])` for Tokio child transports, and `cmd!(spec program, args![...])` for reusable `CommandSpec`. The returned builders may still set environment, directory, stdio, `kill_on_drop`, process group, and execution options; arguments must be complete before that configuration. Keep these forms macro-hygienic when `acorn-cmd` is renamed downstream. Implement the async form by converting the prepared standard command with Tokio's `Command::from_std` if parity tests confirm its lifecycle behavior; do not make Tokio mandatory for `acorn-cmd`.
- Consolidate the eight direct `Command::new` expansions in `cmd!` and five runtime constructors into one private backend where practical. Since exported macros expand in downstream crates, expose any necessary documented, hidden entry point through `$crate` while keeping the raw constructor private to the backend. The backend is the necessary exception to the call-site rule; the final audit must name its exact remaining location. A constructor hidden in another helper or a renamed alias is still a violation outside that backend.
- Preserve non-UTF-8 and path arguments as `OsString`/`OsStr`, not lossy strings. Preserve any deliberately selected shell interpreter, quoting, and the distinction between `output`, `status`, `spawn`, and managed execution. Do not change timeout, cancellation, output capture, child cleanup, stream wiring, directory, environment-clear, or failure semantics just to use a macro.
- The exact write sets below are exclusive per phase. Recheck all call sites and concurrent owners before each phase. A newly discovered call site outside a phase's write set requires a bounded plan amendment; do not silently widen that phase. New Rust code follows repository rules: no `?`, early returns, unsafe, or bare production `unwrap()`.

# Acceptance criteria

1. A source audit over the final Rust tree finds no `Command::new`, aliased equivalent, or `CommandSpec::new` outside the documented private `acorn-cmd` backend and macro implementation. Every call site that constructs an explicit argv collection uses `args!`, including direct `cmd!`, prepared-command, and `sudo!` forms; literal `cmd!` command syntax remains supported.
2. Process-lifecycle tests prove unchanged argv bytes/order, non-UTF-8 paths, cwd/env isolation, stdin/stdout/stderr, process groups, timeout/cancellation, nonzero exit handling, and descendant cleanup. The async MCP transport retains `env_clear`, `kill_on_drop`, and piped I/O.
3. The renamed downstream dependency fixture, CLI integration tests, daemon tests, xtask tests, browser host build, and portable WASM check compile or pass as applicable. `cargo fmt --all -- --check`, `cargo clippy --workspace --no-deps --all-targets`, and `make test` pass after the integration lock is free; record any unrelated pre-existing gate failure separately.
4. The final inventory records the remaining single backend constructor, a zero count outside it, and any supported shell-interpreter forms. Update command macro documentation and architecture notes to explain the boundary.

# Execution phases

## CMD-01 — Add prepared command forms and consolidate the backend

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: NTR has released overlapping command-library and Workspace graph work; verify current macro forms and exact constructor inventory.
- Exact write set: `crates/acorn-cmd/src/lib.rs`, `crates/acorn-cmd/src/runtime.rs`, `crates/acorn-cmd/src/tests.rs`.
- Primary integration lock: Command runtime API.
- Outcome: Add the `build`, `build async`, and `spec` macro forms, require `args!` for their structured argv, preserve existing forms, and route internal direct constructors through one private native backend. Cover renamed-crate hygiene in macro tests where possible without changing downstream fixtures yet.
- Focused validation: `cargo test -p acorn-cmd --lib`, `cargo clippy -p acorn-cmd --all-targets -- -D warnings`, package format check; compare output/status/spawn and managed-result behavior before and after.
- Handoff: Freeze the builder signatures and the sole remaining backend constructor for all consumer phases.

## CMD-02 — Migrate command-crate lifecycle fixtures and downstream use

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CMD-01 builder signatures accepted.
- Exact write set: `crates/acorn-cmd/Cargo.toml`, `crates/acorn-cmd/tests/execution.rs`, `crates/acorn-cmd/tests/fixtures/process_helper.rs` (delete), `crates/acorn-cmd/src/bin/process_helper.rs` (create), `tests/downstream-host/src/main.rs`.
- Primary integration lock: Command test harness.
- Outcome: Replace the standalone `rustc` helper build and the helper's raw descendant spawn with a Cargo test binary using `cmd!`/`args!`; declare the helper explicitly in the manifest, remove the stale cargo-shear fixture exception, and check its package impact. Migrate the managed `CommandSpec` cases to `cmd!(spec ...)`, including the renamed downstream crate and its inline argv.
- Focused validation: `cargo test -p acorn-cmd --test execution`; `cargo check --manifest-path tests/downstream-host/Cargo.toml`; assert cancellation, timeout, bounded capture, descendant cleanup, exit status, and exact argv cases still pass.
- Handoff: Record helper binary path and downstream macro-hygiene evidence.

## CMD-03 — Convert synchronous host and library command leaves

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CMD-01; native host/library owners have released these leaves.
- Exact write set: `crates/acorn-host/src/source.rs`, `crates/acorn-lib/src/analyzer/fix/execution.rs`, `crates/acorn-lib/src/io/acp/installation.rs`, `crates/acorn-lib/src/io/acp/tests.rs`, `crates/acorn-lib/src/io/mcp/tools/native/analysis.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/analysis.rs`.
- Primary integration lock: Native host command leaves.
- Outcome: Convert direct Git/tool calls and `CommandSpec::new` constructions to `cmd!` forms with `args!`, keeping timeout, noninteractive environment, executable availability, and failure mapping unchanged.
- Focused validation: focused `acorn-host` source tests, `acorn-lib` fix/ACP/MCP native-analysis tests under their current feature sets, and package Clippy; check the portable `acorn-lib` feature set still excludes host process code.
- Handoff: Record converted leaves, preserved error categories, and any platform-only command not yet exercised.

## CMD-04 — Preserve the asynchronous MCP child transport

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CMD-01 async builder accepted; MCP transport owner available.
- Exact write set: `crates/acorn-lib/src/io/mcp/client.rs`, `crates/acorn-lib/src/io/mcp/tests/client.rs`.
- Primary integration lock: MCP child transport.
- Outcome: Build Tokio child commands and the test helper process through `cmd!`/`args!` while retaining `env_clear`, injected secret environment, `kill_on_drop`, piped stdin/stdout, null stderr, and existing cleanup.
- Focused validation: focused `acorn-lib` MCP client tests, including launch failure, cancellation and cleanup; compile the async macro form in the consumer crate.
- Handoff: Record transport parity and any native OS behavior still requiring separate acceptance.

## CMD-05 — Preserve daemon process-group execution

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CMD-01 std builder accepted; daemon runtime owner available.
- Exact write set: `crates/acorn-daemon/src/runner.rs`, `crates/acorn-daemon/src/tests/mod.rs`.
- Primary integration lock: Daemon runtime.
- Outcome: Route both check and gather child construction through `cmd!`/`args!`; retain process-group spawn, null stdin, piped output, bounded capture, stop signal, report placement and descendant termination.
- Focused validation: focused `acorn-daemon` runner tests and native check/gather fixtures; verify cancellation and size-limit behavior before broad daemon gates.
- Handoff: Record identical argv and report outcomes for both job types.

## CMD-06 — Convert CLI production and example process calls

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CMD-01; CLI command owner available after NTR-06 finishes its shared CLI work.
- Exact write set: `crates/acorn-cli/src/commands/daemon/install.rs`, `crates/acorn-cli/src/commands/daemon/tests.rs`, `crates/acorn-cli/src/commands/exec/automation.rs`, `crates/acorn-cli/examples/metadata_chat.rs`.
- Primary integration lock: CLI command leaves.
- Outcome: Convert staged daemon actions, Rhai automation commands, and the metadata-chat example to `cmd!`/`args!` without changing optional flags, error text, permissions, or output handling.
- Focused validation: focused CLI daemon and exec tests, example compilation, and CLI package Clippy.
- Handoff: Record native command parity and release the CLI command leaf lock.

## CMD-07 — Convert export and report command tests

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CMD-01; export/report test owner available.
- Exact write set: `crates/acorn-cli/tests/registry/export.rs`, `crates/acorn-cli/src/commands/export/tests/mod.rs`, `crates/acorn-cli/src/commands/report/tests/mod.rs`.
- Primary integration lock: Export verification.
- Outcome: Replace all 27 direct CLI, `pdfimages`, and `pdftotext` constructions in these files with `cmd!` and `args!`; preserve output capture, successful/nonzero status assertions, fixture paths, and optional renderer skips.
- Focused validation: focused CLI export/report integration tests, including PDF/PPTX and failure cases where tool prerequisites are available.
- Handoff: Record which native renderer tools ran and which platform-dependent cases remain gated.

## CMD-08 — Convert the remaining CLI integration harnesses

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CMD-01; no concurrent edits to the listed tests.
- Exact write set: `crates/acorn-cli/src/io/interactive/tests.rs`, `crates/acorn-cli/tests/format.rs`, `crates/acorn-cli/tests/mcp.rs`, `crates/acorn-cli/tests/mcp_client.rs`, `crates/acorn-cli/tests/registry/check.rs`, `crates/acorn-cli/tests/registry/doctor.rs`, `crates/acorn-cli/tests/registry/download.rs`, `crates/acorn-cli/tests/registry/exec.rs`, `crates/acorn-cli/tests/registry/import.rs`, `crates/acorn-cli/tests/registry/infer.rs`, `crates/acorn-cli/tests/registry/swhid.rs`.
- Primary integration lock: CLI integration test harness.
- Outcome: Migrate the other 21 direct child constructions, including the `ProcessCommand` alias, to macro construction with `args!`; retain inherited/redirected stdio, spawned child ownership, exact request fixtures, and expected exit behavior.
- Focused validation: targeted CLI integration suites for format, MCP, check, doctor, download, exec, import, infer, SWHID, and interactive input.
- Handoff: Record the CLI direct-constructor count as zero outside the command backend.

## CMD-09 — Convert browser build and xtask host tooling

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CMD-01; NTR and other manifest owners have released Workspace graph.
- Exact write set: `Cargo.lock`, `crates/acorn-web/Cargo.toml`, `crates/acorn-web/build.rs`, `xtask/Cargo.toml`, `xtask/src/main.rs`, `xtask/src/benchmark_history.rs`, `xtask/src/application_release/mod.rs`.
- Primary integration lock: Workspace graph.
- Outcome: Give the browser build script a host-only `acorn-cmd` build dependency and xtask a command dependency; migrate their six Git/Cargo/Python process calls to `cmd!`/`args!` without adding `acorn-cmd` to the browser WASM runtime graph or changing rerun directives and release behavior.
- Focused validation: `cargo test -p xtask`, browser package build/check, `cargo check -p acorn-web --target wasm32-unknown-unknown`, and a dependency-tree check showing the command crate only in the host build context.
- Handoff: Record build-script and xtask parity plus the verified target dependency graph.

## CMD-10 — Normalize explicit library command arguments

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CMD-01; library root/sidecar owners available.
- Exact write set: `crates/acorn-lib/src/analyzer/host/runtime/mod.rs`, `crates/acorn-lib/src/io/mod.rs`, `crates/acorn-lib/src/io/sidecar/needle/process.rs`, `crates/acorn-lib/src/io/swhid.rs`, `crates/acorn-lib/src/io/tests/swhid.rs`.
- Primary integration lock: Library command adapters.
- Outcome: Replace inline arrays and independently assembled argv passed to `cmd!` with `args!`, including runtime, OCI/Git, Needle and SWHID paths. Preserve option ordering, path bytes, directory and child environment.
- Focused validation: focused analyzer-host, OCI/Git, Needle and SWHID tests, plus `acorn-lib` package Clippy with their existing feature gates.
- Handoff: Record command/argument parity and release the library root lock.

## CMD-11 — Normalize daemon installation and sudo arguments

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CMD-01; host service registration owner available.
- Exact write set: `crates/acorn-daemon/src/install/linux.rs`, `crates/acorn-daemon/src/install/macos.rs`, `crates/acorn-daemon/src/install/tests.rs`, `crates/acorn-macros/src/lib.rs`.
- Primary integration lock: Host service registration.
- Outcome: Convert inline `cmd!` and `sudo!` argument arrays to `args!`; adapt `sudo!`'s diagnostic rendering for `OsString` without changing privilege boundary, target paths, status handling, or leaking secrets.
- Focused validation: focused Linux systemd and macOS launchd tests, macro crate tests, and platform compilation checks; native host assertions only where that OS is available.
- Handoff: Record privileged command argv and error parity.

## CMD-12 — Normalize remaining CLI command arguments

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CMD-01 and CMD-06; CLI command leaves are free.
- Exact write set: `crates/acorn-cli/src/commands/create/bot/mod.rs`, `crates/acorn-cli/src/commands/create/runner/mod.rs`, `crates/acorn-cli/src/commands/daemon/mod.rs`, `crates/acorn-cli/src/commands/daemon/remote.rs`, `crates/acorn-cli/src/commands/serve/rpc.rs`, `crates/acorn-cli/src/commands/serve/mcp/mod.rs`, `crates/acorn-cli/src/commands/serve/grpc/mod.rs`.
- Primary integration lock: CLI command leaves.
- Outcome: Make every explicit argv collection passed to existing `cmd!` calls originate in `args!`, preserving SSH/container quoting, register/create option order, remote transfer stdin, and daemon action behavior.
- Focused validation: focused create/runner/bot, remote daemon, and serve transport tests; inspect rendered argv and nonzero results on controlled fixtures.
- Handoff: Record argument parity and any newly discovered call site requiring its own bounded repair phase.

## CMD-13 — Audit the final tree and close integration gates

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CMD-01–12 accepted; stable workspace tree and single integration owner.
- Exact write set: `crates/acorn-cmd/src/lib.rs`, `ARCHITECTURE.md`, `.wiki/refactors/command-macro-convergence.md`, `.wiki/refactors/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`.
- Primary integration lock: Workspace integration.
- Outcome: Document the one private constructor boundary and macro usage contract; audit the final Rust source for direct constructors, aliases, `CommandSpec::new`, and caller-built argv. Reconcile any finding through a new exact-write-set repair phase before retiring this plan.
- Focused validation: source inventory and diff audit, `cargo fmt --all -- --check`, `cargo clippy --workspace --no-deps --all-targets`, `make test`, `make check-wasm`, downstream fixture check, and the wiki validator. Preserve separate results for native platform-only and unrelated pre-existing failures.
- Handoff: Record zero out-of-boundary constructors, passing gates, and the backend location; prune the completed plan and remove its portfolio/network/lane entries.
