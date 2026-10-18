---
okf_version: "0.1"
type: Plan
title: Close high-value CLI coverage gaps
description: Add deterministic coverage for link, serve, SWHID, and stable binary error boundaries while consolidating crate-local test support.
difficulty: medium
impact: medium
package: [acorn-cli]
tags: [testing, coverage, cli, integration, assert_cmd]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Close the remaining high-value command and binary gaps in the current crate-local test layout. Favor behavior and failure boundaries that protect users over a broad line-count target.

# Current evidence

- Download, check, doctor, export, sync, format, import, create, and gather have substantial command coverage.
- The outbound MCP command has dedicated compiled-binary interoperability and policy/error tests; it is not a remaining command gap.
- Link has one compiled-binary gate test but no direct success, dry-run, invalid-input, ignore, or multi-file suite.
- Serve and SWHID lack direct command coverage. Skill and TUI gaps are lower value unless another change touches them.
- Binary coverage exists across 43 `cargo_bin` call sites plus the dedicated MCP binary tests, but several important missing-input and invalid-structured-input exits are not asserted consistently.
- Download, import, and format tests repeat a tracing buffer writer. Fixture-root helpers are intentionally crate-local and should remain so.

# Project code boundaries

- **Update:** link, serve, SWHID, and selected binary integration tests; shared support under `crates/acorn-cli/src/test/`; existing workspace fixtures.
- **Create:** focused crate-local integration files only when behavior requires a compiled-binary boundary.
- **Delete:** repeated tracing/log-capture harnesses after all current users migrate. Do not recreate the superseded root `tests/` layout.

# Dependencies

Leaf command tests may proceed while avoiding shared CLI parser/help files. Tests for normalized startup or exit behavior follow the CLI composition boundary and should consume its public seams rather than duplicate setup.

# Work

1. Cover link success for JSON/YAML, dry run, missing/invalid input, ignores, deterministic multi-file ordering, naming, and the existing configured gate.
2. Cover serve target validation and SWHID success/error paths through stable command seams.
3. Add the highest-value compiled-binary missing-input, invalid-input, exit-status, and diagnostic assertions for check, export, format, link, serve, and SWHID.
4. Move the tracing writer and a result-preserving `capture_logs` helper into crate-local test support.
5. Add skill or TUI tests only when a concrete regression risk or implementation plan names the behavior.

# Validation

- Run focused command and integration tests, then `make test-ci` and `make test`.
- Assert both exit status and stable diagnostic content without depending on color or platform-specific absolute paths.
- Keep network access disabled unless a test is explicitly ignored for a named external dependency.

# Acceptance criteria

- Link has deterministic success, dry-run, invalid-input, ignore, naming, multi-file, and configured-gate coverage.
- Serve and SWHID have direct success and failure tests for their supported local contracts.
- High-value binary failures return tested nonzero statuses and actionable diagnostics.
- Log capture has one crate-local owner, while fixture-root helpers remain local to their crates.
- No new ignored test is added without a named external dependency.
- New tests use the current crate-local layout and pass both CI and full workspace test commands.

# Execution phases

## COV-01 — Consolidate crate-local log capture

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-cli/src/test.rs`
- Primary integration lock: CLI test support
- Outcome: Add one result-preserving tracing capture helper and remove only duplicated writers.
- Focused validation: Run the migrated focused tests and verify errors/results survive capture.
- Handoff: Remove COV-01 and expose the helper to later coverage phases.

## COV-02 — Cover link success and dry-run behavior

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COV-01
- Exact write set: `crates/acorn-cli/src/commands/link/tests.rs`, `tests/fixtures/cli/link/valid.json`
- Primary integration lock: link command tests
- Outcome: Cover JSON/YAML success, dry run, output naming, and configured-gate behavior.
- Focused validation: Run focused link command tests with network disabled.
- Handoff: Remove COV-02 and retain deterministic success fixtures.

## COV-03 — Cover link failures and ordering

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COV-02
- Exact write set: `crates/acorn-cli/src/commands/link/tests.rs`, `tests/fixtures/cli/link/invalid.json`
- Primary integration lock: link command tests
- Outcome: Cover missing/invalid input, ignores, multi-file order, collisions, and actionable errors.
- Focused validation: Run focused link tests and assert stable status/diagnostic content.
- Handoff: Remove COV-03 and record the complete link command matrix.

## COV-04 — Cover serve target behavior

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COV-01
- Exact write set: `crates/acorn-cli/src/commands/serve/mod.rs`, `crates/acorn-cli/src/commands/serve/rpc.rs`, `crates/acorn-cli/src/commands/serve/grpc/mod.rs`
- Primary integration lock: serve command tests
- Outcome: Cover local, SSH/context validation, detach, spawn, and nonzero-exit seams.
- Focused validation: Run focused RPC, gRPC, and MCP serve tests without external hosts.
- Handoff: Remove COV-04 and retain stable target diagnostics.

## COV-05 — Cover the SWHID command

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COV-01
- Exact write set: `crates/acorn-cli/src/commands/swhid/mod.rs`, `tests/fixtures/swhid/valid-repository.json`
- Primary integration lock: SWHID command tests
- Outcome: Cover supported success, missing input, invalid repositories, and stable error paths.
- Focused validation: Run focused SWHID command tests offline.
- Handoff: Remove COV-05 and record the command matrix.

## COV-06 — Add high-value binary failure coverage

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COV-03, COV-04, COV-05
- Exact write set: `crates/acorn-cli/tests/errors.rs`, `tests/fixtures/cli/errors/invalid.json`
- Primary integration lock: CLI binary tests
- Outcome: Assert status and stable diagnostics for check, export, format, link, serve, and SWHID failures.
- Focused validation: Run the compiled-binary test target on supported platforms without color/path coupling.
- Handoff: Remove COV-06 and preserve the shared failure oracle.

## COV-07 — Run CI and retire coverage work

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COV-06
- Exact write set: `.wiki/testing/cli-coverage.md`, `.wiki/index.md`, `.wiki/testing/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: CLI test integration
- Outcome: Run CI/full suites, confirm no unjustified ignored tests, and remove the plan.
- Focused validation: Run focused command tests, `make test-ci`, `make test`, and wiki validation.
- Handoff: Delete this plan and every index, network, and lane entry.
