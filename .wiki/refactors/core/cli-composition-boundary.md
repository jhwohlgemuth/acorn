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
- RPC, gRPC, and MCP serve commands repeat target validation, unsupported-context errors, foreground SSH invocation, detached guidance, and subprocess failure mapping.
- `color-eyre` is the correct application reporter. The workspace already has direct `thiserror` dependencies and several owned typed errors, so a separate workspace-wide derive migration is unnecessary.

# Project code boundaries

- **Update:** `crates/acorn-cli/src/main.rs`, `src/cli/`, command dispatch/result aliases, logging/runtime/database bootstrap helpers, serve command composition, and binary tests.
- **Create:** normalized execution request values, focused startup functions/modules, one reusable remote-serve dispatcher, and one application failure-to-report/exit mapping.
- **Delete:** command-layer clap dependencies after migration, repeated remote-serve branches, per-command process exits, and duplicate top-level report setup.

# Dependencies

Schedule after any active CLI syntax owner releases `src/cli/` and generated help. Provider and service extraction may proceed independently, but this plan consumes their final request and error types rather than inventing interface-owned duplicates.

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
