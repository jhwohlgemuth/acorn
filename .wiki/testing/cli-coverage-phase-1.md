---
type: Plan
title: "CLI test coverage phase 1"
description: "Cover the untested link command and close remaining high-value binary and error-path gaps."
difficulty: medium
impact: high
package: [acorn-cli]
tags: [testing, coverage, cli, link, assert_cmd, error-paths]
status: planned
---

# Goal

Finish the remaining high-priority CLI coverage rather than recreating the superseded root-test layout. The workspace now has substantial inline command coverage, crate-local integration tests in `crates/acorn-cli/tests/`, and 27 `Command::cargo_bin` call sites. The link command remains the only mature user-facing command with no focused tests.

# Current evidence

- `crates/acorn-cli/src/commands/link/mod.rs` has no `#[test]` or `#[tokio::test]` coverage.
- `crates/acorn-cli/tests/check.rs` contains six active binary tests and `crates/acorn-cli/tests/help.rs` contains two.
- Download and gather suites also exercise the binary; command-local totals have grown to 103 download, 18 gather, 17 check, 17 export, 17 import, 11 sync, 9 doctor, 8 create, and 6 format tests.
- The former planned root files `tests/check.rs`, `tests/export.rs`, `tests/format.rs`, and `tests/doctor.rs` are not the current crate layout and must not be recreated.
- Four ignored export tests require external visual/platform tooling; this phase does not relabel those as ordinary regressions.

# Project code boundaries

- **Update:** `crates/acorn-cli/src/commands/link/mod.rs`, `crates/acorn-cli/tests/{check,help}.rs`, shared test support under `crates/acorn-cli/src/test/`, and existing workspace fixtures.
- **Create:** `crates/acorn-cli/src/commands/link/tests/mod.rs` if the command follows the adjacent module convention; add a crate-local integration file only for behavior that requires the compiled binary.
- **Delete:** no current tests. Do not recreate or delete tests through a root `tests/` directory.

# Dependencies

The link tests may use the implemented shared link-evidence and evaluation seams, but deterministic offline parsing, missing-path, dry-run, and output tests can proceed independently.

# Work

1. Add link-command unit/integration coverage for valid JSON and YAML inputs, dry-run output, missing paths, invalid structured data, ignore filtering, deterministic multi-file behavior, and output naming.
2. Audit current binary tests for the highest-value uncovered error boundaries: missing input, invalid structured input, and exit-status/message stability for check, export, format, and link.
3. Reuse existing fixtures and crate-local test helpers. Add only the smallest malformed/valid fixtures not already present.
4. Keep network access disabled unless a test is explicitly ignored with a concrete external dependency.

# Validation

- Run focused `acorn-cli` link and integration tests.
- Run `make test-ci` and `make test`.
- Confirm new binary tests assert both exit status and stable diagnostic content without depending on color or platform-specific paths.

# Acceptance criteria

- The link command has deterministic coverage for successful write, dry run, missing input, invalid input, ignore filtering, and multiple files.
- High-value compiled-binary error paths return tested non-zero exit statuses and actionable diagnostics.
- All new tests live in the current crate-local layout, use existing helpers/fixtures, and require no network access.
- No new ignored test is introduced without a named external dependency.
