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
