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

Finish the remaining high-priority CLI coverage rather than recreating the superseded root-test layout. The workspace now has substantial inline command coverage, crate-local integration tests in `crates/acorn-cli/tests/`, and 41 `cargo_bin` call sites. The link command has one configured-gate binary test but no direct success, dry-run, parsing, or multi-file coverage.

# Current evidence

- `crates/acorn-cli/src/commands/link/tests.rs` has one compiled-binary test proving that a configured failing link hook prevents source and JSON-LD writes.
- `crates/acorn-cli/tests/check.rs` contains seven tests; `help.rs` and `mcp.rs` contain two each, and `create.rs` contains one.
- Command-local totals are 106 download, 25 check, 24 export, 24 gather, 17 import, 15 create, 12 format, 12 sync, and 9 doctor tests.
- The former planned root files `tests/check.rs`, `tests/export.rs`, `tests/format.rs`, and `tests/doctor.rs` are not the current crate layout and must not be recreated.
- Four ignored export tests require external visual/platform tooling; this phase does not relabel those as ordinary regressions.

# Project code boundaries

- **Update:** `crates/acorn-cli/src/commands/link/mod.rs`, `crates/acorn-cli/tests/{check,help}.rs`, shared test support under `crates/acorn-cli/src/test/`, and existing workspace fixtures.
- **Create:** additional focused files only if `crates/acorn-cli/src/commands/link/tests.rs` becomes too large; add a crate-local integration file only for behavior that requires a separate compiled-binary boundary.
- **Delete:** no current tests. Do not recreate or delete tests through a root `tests/` directory.

# Dependencies

The link tests may use the implemented shared link-evidence and evaluation seams, but deterministic offline parsing, missing-path, dry-run, and output tests can proceed independently.

# Work

1. Extend link-command coverage for valid JSON and YAML inputs, dry-run output, missing paths, invalid structured data, ignore filtering, deterministic multi-file behavior, and output naming while preserving the existing configured-gate case.
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
