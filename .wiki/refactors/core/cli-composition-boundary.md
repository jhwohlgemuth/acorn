---
okf_version: "0.1"
type: Plan
title: Establish the CLI composition boundary
description: Normalize parser inputs, isolate startup services, share remote serve dispatch, and map command failures at one outer boundary.
difficulty: high
impact: high
package: [acorn-cli]
tags: [refactor, cli, clap, startup, errors, composition]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Turn `acorn-cli` into a thin composition root:

```text
clap parser -> normalized execution request -> startup services -> command dispatch -> one error/exit boundary
```

Command implementations receive immutable execution values rather than clap-only wrappers, startup units are independently testable, and no command terminates the process or chooses its own top-level exit mapping.

# Current evidence

- `main.rs` installs reporting, loads dotenv, parses help, configures tracing and Rayon, mutates cache environment, constructs database configuration, validates conflicting flags, dispatches, special-cases silent discovery failures, prints reports, and calls `process::exit`.
- Command run signatures still expose parser-derived types in several paths.
- The outbound MCP command is another current example: its adapter receives `McpCommands` directly and performs configuration loading, request normalization, execution, and presentation in one leaf.
- RPC, gRPC, and MCP serve commands repeat target validation, unsupported-context errors, foreground SSH invocation, detached guidance, and subprocess failure mapping.
- `color-eyre` is the correct application reporter. The workspace already has direct `thiserror` dependencies and several owned typed errors, so a separate workspace-wide derive migration is unnecessary.

# Project code boundaries

- **Update:** `crates/acorn-cli/src/main.rs`, `src/cli/`, command dispatch/result aliases, logging/runtime/database bootstrap helpers, serve command composition, and binary tests.
- **Create:** normalized execution request values, focused startup functions/modules, one reusable remote-serve dispatcher, and one application failure-to-report/exit mapping.
- **Delete:** command-layer clap dependencies after migration, repeated remote-serve branches, per-command process exits, and duplicate top-level report setup.

# Dependencies

Schedule after any active CLI syntax owner releases `src/cli/` and generated help. Provider and service extraction may proceed independently, but this plan consumes their final request and error types rather than inventing interface-owned duplicates.

The [Needle router and registry parity plan](../../features/slm-sidecars.md) owns capability descriptors, executor bindings, and CLI/service equivalence adapters. Its PAR-00 inventories reusable operation seams; coordinate each PAR command leaf with this plan before editing. Share normalized operation requests/results and keep parsing/presentation in the CLI. Neither plan owns a duplicate implementation or requires completion of the other entire plan; a missing service seam gets a separately bounded owner phase.

# Decisions

- Clap structs exist only at the parse boundary. One fallible normalization step produces execution requests with defaults, paths, feature availability, and mutually exclusive inputs resolved.
- Startup functions own tracing, runtime/thread-pool setup, database bootstrap, cache policy, and command dispatch. They return values or typed errors and never exit.
- `main` installs the reporter, parses and normalizes once, calls startup/dispatch, renders one report, maps one named exit code, and returns control to the process boundary.
- Silent-output behavior becomes an explicit application error classification rather than a downcast embedded in `main`.
- Local protocol startup stays in each serve module. A CLI-local helper owns only validated remote dispatch and its common SSH/subprocess diagnostics.
- Use `thiserror` for newly touched owned reusable errors where variants help callers. Do not mechanically convert unrelated errors or replace `color-eyre` at the application boundary.

# Implementation sequence

1. Inventory parser types crossing into commands and introduce normalized request values without changing public syntax.
2. Migrate command leaves in small groups and keep generated help stable.
3. Extract tracing, thread-pool, database/cache, preflight, and dispatch units with order-sensitive tests.
4. Consolidate remote serve dispatch against normalized targets.
5. Add the outer application error/exit boundary and remove direct command exits and the `main` downcast special case.

# Validation

- Parser snapshot and generated-help tests prove syntax and defaults remain compatible.
- Unit tests cover normalization errors, startup order, duplicate reporter/runtime initialization, database-disabled conflicts, and thread-pool failures.
- Serve tests cover local, SSH, context, detached, validation, spawn, and nonzero-exit behavior for RPC, gRPC, and MCP.
- Binary tests assert stable exit status and diagnostic content for representative parse, setup, command, silent-output, and internal failures.
- Run focused CLI tests, no-default and feature combinations, lint, the full suite, and documentation generation.

# Acceptance criteria

- Command implementations receive normalized execution requests without depending on clap types.
- Startup units are independently testable and `main` contains only reporter installation, parse/normalize, orchestration, and final result handling.
- Commands do not call `process::exit` or duplicate exit-code mapping.
- Silent failures, usage failures, expected operation failures, and internal failures have explicit tested report and exit behavior.
- RPC, gRPC, and MCP share remote dispatch while retaining protocol-specific local startup and policy.
- Public CLI syntax, generated help, offline behavior, terminal output, and existing successful command behavior remain compatible.

# Execution phases

## CLI-01 — Freeze parser-to-command crossings

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: no active CLI syntax owner
- Exact write set: `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/cli/arguments/mod.rs`, CLI characterization tests
- Primary integration lock: CLI syntax and generated help
- Outcome: Inventory parser types crossing command boundaries and define immutable normalized request conventions.
- Focused validation: Run parser snapshots and generated-help comparison without syntax changes.
- Handoff: Remove CLI-01 and publish the migration groups used below.

## CLI-02 — Normalize check and doctor requests

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CLI-01
- Exact write set: `crates/acorn-cli/src/commands/check/mod.rs`, `crates/acorn-cli/src/commands/doctor/mod.rs`, focused tests
- Primary integration lock: CLI command signatures
- Outcome: Remove clap-only wrappers from check/doctor command execution values.
- Focused validation: Run focused command, parser, offline, and output tests.
- Handoff: Remove CLI-02 and record the reusable normalization pattern.

## CLI-03 — Normalize transform and export requests

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CLI-01
- Exact write set: `crates/acorn-cli/src/commands/diff/mod.rs`, `crates/acorn-cli/src/commands/format/mod.rs`, `crates/acorn-cli/src/commands/export/mod.rs`, `crates/acorn-cli/src/commands/link/mod.rs`
- Primary integration lock: CLI command signatures
- Outcome: Pass immutable normalized values to diff, format, export, and link leaves.
- Focused validation: Run focused command tests and confirm generated help is unchanged.
- Handoff: Remove CLI-03 and preserve stable output/exit behavior.

## CLI-04 — Normalize gather and import requests

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CLI-01
- Exact write set: `crates/acorn-cli/src/commands/gather/mod.rs`, `crates/acorn-cli/src/commands/import/mod.rs`, focused tests
- Primary integration lock: CLI command signatures
- Outcome: Resolve mutually exclusive sources/defaults at the parse boundary for gather/import.
- Focused validation: Run focused parser, command, offline, and collision tests.
- Handoff: Remove CLI-04 and provide normalized requests to later gather work.

## CLI-05 — Normalize download and sync requests

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CLI-01
- Exact write set: `crates/acorn-cli/src/commands/download/mod.rs`, `crates/acorn-cli/src/commands/sync/mod.rs`, focused tests
- Primary integration lock: CLI command signatures
- Outcome: Resolve acquisition options and feature availability before command execution.
- Focused validation: Run focused request, offline, missing-feature, and dry-run tests.
- Handoff: Remove CLI-05 and record the acquisition request contract.

## CLI-06 — Normalize create and inference requests

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CLI-01
- Exact write set: `crates/acorn-cli/src/commands/create/mod.rs`, `crates/acorn-cli/src/commands/infer/mod.rs`, focused tests
- Primary integration lock: CLI command signatures
- Outcome: Move parser-specific defaults and conflicts out of create/inference leaves.
- Focused validation: Run focused parser, command, feature, and failure tests.
- Handoff: Remove CLI-06 and record normalized creation/inference requests.

## CLI-07 — Normalize exec and outbound MCP requests

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CLI-01, MCP-02
- Exact write set: `crates/acorn-cli/src/commands/exec/mod.rs`, `crates/acorn-cli/src/commands/mcp.rs`, focused tests
- Primary integration lock: CLI command signatures
- Outcome: Separate configuration loading, request normalization, execution, and presentation.
- Focused validation: Run exec and compiled-binary MCP interoperability tests.
- Handoff: Remove CLI-07 and preserve the narrow MCP client boundary.

## CLI-08 — Consolidate remote serve dispatch

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CLI-01
- Exact write set: `crates/acorn-cli/src/commands/serve/mod.rs`, RPC/gRPC/MCP serve tests
- Primary integration lock: serve composition
- Outcome: Share target validation and SSH/subprocess diagnostics while retaining protocol-local startup.
- Focused validation: Test local, SSH, context, detached, spawn, and nonzero-exit cases.
- Handoff: Remove CLI-08 and record the shared remote dispatcher.

## CLI-09 — Extract startup services

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CLI-02, CLI-03, CLI-04, CLI-05, CLI-06, CLI-07, CLI-08
- Exact write set: `crates/acorn-cli/src/main.rs`, `crates/acorn-cli/src/startup.rs`, startup tests
- Primary integration lock: CLI composition root
- Outcome: Isolate tracing, thread pool, database/cache, preflight, runtime, and dispatch units without exiting.
- Focused validation: Test ordering, duplicate initialization, disabled database, and thread-pool failures.
- Handoff: Remove CLI-09 and expose one result-returning application entry point.

## CLI-10 — Establish the outer error and exit boundary

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CLI-09
- Exact write set: `crates/acorn-cli/src/main.rs`, binary integration tests, `CommandLineHelp.md`
- Primary integration lock: CLI composition root
- Outcome: Render one report and map parse/setup/command/silent/internal failures to explicit tested exits.
- Focused validation: Run binary failure tests, feature combinations, help generation, `make lint`, and `make test`.
- Handoff: Remove this completed plan and all index, network, and lane entries.
