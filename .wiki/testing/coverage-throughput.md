---
okf_version: "0.1"
type: Plan
title: Speed up the local test and coverage loop
description: Remove forced rebuilds, fixture-crate scope, and unbounded sleep-bound test time from `make cover` and `make test` so iteration uses incremental scoped runs.
difficulty: medium
impact: medium
package: [acorn-lib, acorn-cli, acorn-cmd]
tags: [testing, coverage, tarpaulin, nextest, developer-workflow, throughput]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Make the local verification loop fast enough to run after every change. Coverage is a developer-feedback tool; today a single `make cover` is dominated by a forced clean rebuild of the whole workspace, so developers either skip it or scope it informally.

# Current evidence

November 12 targeted currency check at `6c2a51b8`: the forced-clean recipes and missing nextest configuration remain; the workspace now has 17 members and GitLab CI runs coverage. Historical suite/package counts below have been removed where not remeasured. The separate [Blaze pilot](../refactors/blaze-development-pilot/index.md) evaluates native incremental builds and does not replace this plan's tarpaulin or slow-test work.

- `Makefile` `cover`, `cover-cli`, and `cover-lib` all pass `--force-clean` to `cargo tarpaulin`. `cargo tarpaulin --help` describes that flag as "adds a clean stage", so every run discards instrumented artifacts and recompiles the workspace from scratch.
- The workspace has 17 members, including heavy optional dependencies (`duckdb`, `chromiumoxide`, `tonic`, `lychee-lib`) owned by `acorn-cli`, `acorn-lib`, `acorn-schema`, and `acorn-server`.
- Unscoped `make cover` includes workspace test targets, including the fixture-only `tests/downstream-host` crate and `xtask`. Measure actual instrumented files before changing coverage scope; maintenance-tool tests still provide release-tool correctness value.
- Wall-clock-bound tests exist in several places and are the most likely test-time cost once rebuilds are removed: `crates/acorn-cmd/tests/execution.rs` sleeps 700ms and 75ms, `crates/acorn-lib/src/io/mcp/tests/client.rs` has 30-second retry sleeps, `crates/acorn-cli/src/commands/download/tests/mod.rs` polls with 150ms sleeps, and `crates/acorn-lib/src/io/api/webhooks/mod.rs` polls at 10ms.
- No `[profile.coverage]` tuning exists beyond `inherits = "test"` plus `debug = 1`. `[profile.test]` sets `"*".opt-level = 2`, so a coverage build pays optimized codegen for all dependencies even though coverage does not need it.
- There is no `.config/nextest.toml`. `make test` runs `cargo nextest run` with library defaults: no per-test timeout, so a hung sleep- or network-bound test can stall a run with no diagnostic.
- `.gitlab-ci.yml` has a `coverage` job calling `make cover` for merge requests and the default branch. Changes to this shared target therefore affect both local and CI coverage; preserve its reporting contract and inspect resulting lcov file totals.
- Earlier `cargo tarpaulin --help` inspection reports `-j, --jobs <N>` defaults to the CPU count; concurrency tuning still requires measurement. Tarpaulin does not run doctests unless `--doc` is passed, so doctests are not part of `make cover`; `make doctest` is already separate.
- Tarpaulin supplies its own `--test-threads` unless `--implicit-test-threads` is passed. Whether that helps is an empirical question, not an assumption.

# Project code boundaries

- **Update:** `Makefile` coverage and test targets; `.config/nextest.toml`; `[profile.coverage]` in the root `Cargo.toml`; `AGENTS.md` inner-loop guidance.
- **Create:** `.config/nextest.toml` and a named slow-test gate target.
- **Delete:** none. No test may be removed or silently skipped to gain time.

# Dependencies

Independent of every feature and refactor plan; it owns only developer-workflow surfaces. It shares the `CI/build` lock and the workspace graph lock, so it must not run concurrently with Podman migration, runner-pool, PID verification, or export fuzzing phases that own `Makefile` or `.gitlab-ci.yml`. Test-source edits in EXP-03 must observe the exact write sets of any concurrent CLI or MCP phase touching the same test modules.

# Work

1. Remove the forced clean stage from the default coverage targets and keep the clean build available under an explicit target.
2. Scope default coverage to product crates and document the scoped inner loop.
3. Give every sleep- or network-bound test a bounded timeout and a named slow gate, so no test can consume minutes silently.
4. Stop paying optimized codegen for dependencies in the coverage profile, provided line mapping is unchanged.

# Validation

- Record cold and warm wall clock and compile time for the coverage target before and after each phase; a phase that does not reduce a measured number is reverted rather than kept.
- Compare lcov line and file totals before and after scoping so any coverage reduction is recorded, not concealed.
- Run `make test` and `make test-all` after every test-source change and confirm no test silently stops running by default.
- Run `make check-dependencies` after any manifest or profile change.

# Acceptance criteria

- A second consecutive `make cover` run reuses instrumented artifacts instead of rebuilding the workspace.
- The default local coverage target excludes fixture-only members while an unscoped full-workspace run remains available.
- No test is skipped by default without a named gate that runs it.
- Coverage line mapping is unchanged by profile tuning.
- The recommended inner loop is documented where developers already look.
- The measured before/after wall clock and recommended inner loop are recorded in durable developer documentation before this plan is retired.

# Out of scope

Splitting `acorn-lib` is a package-ownership decision owned by the [crate boundary architecture](../refactors/crate-boundary-architecture.md) coordination record, not a test-throughput change; its performance effect needs measurement. Adding new CI coverage thresholds is outside this plan. Preserve the existing CI coverage job and reporting when changing shared Make targets.

# Execution phases

## COVT-01 — Remove the forced clean stage and record the baseline

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `Makefile`, `AGENTS.md`, `.wiki/testing/coverage-throughput.md`
- Primary integration lock: CI/build (`Makefile`)
- Outcome: Default coverage targets build incrementally, a clean-stage target remains available, and cold versus warm wall clock is recorded in this plan's evidence.
- Focused validation: Two back-to-back `make cover-lib` runs; confirm the second reuses artifacts and emits an equivalent lcov file list.
- Handoff: Remove COVT-01 and carry the measured numbers into COVT-02.

## COVT-02 — Scope default coverage to product crates

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COVT-01
- Exact write set: `Makefile`, `.wiki/testing/coverage-throughput.md`
- Primary integration lock: CI/build (`Makefile`)
- Outcome: The default local coverage target excludes `tests/downstream-host` and `xtask`, an unscoped full-workspace run stays available, and the lcov delta is recorded.
- Focused validation: `make cover-lib` and the full-scope target; compare lcov line and file totals and record the difference.
- Handoff: Remove COVT-02 and record the scope boundary for later phases.

## COVT-03 — Bound sleep- and network-bound test time

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COVT-01
- Exact write set: `.config/nextest.toml`, `Makefile`, `crates/acorn-cmd/tests/execution.rs`, `crates/acorn-lib/src/io/mcp/tests/client.rs`, `crates/acorn-cli/src/commands/download/tests/mod.rs`, `.wiki/testing/coverage-throughput.md`
- Primary integration lock: CLI test support
- Outcome: Per-test timeouts apply to the default run, wall-clock-bound tests have a named slow gate that still runs them, and no test is dropped from default coverage.
- Focused validation: `make test`, `make test-all`, and one scoped coverage run; assert the slow gate actually executes the gated tests.
- Handoff: Remove COVT-03 and record the gated test list and timeout policy.

## COVT-04 — Trim the coverage build profile

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COVT-01, COVT-02
- Exact write set: `Cargo.toml`, `.wiki/testing/coverage-throughput.md`
- Primary integration lock: workspace graph (root manifest)
- Outcome: The coverage profile stops paying optimized codegen for dependencies while preserving coverage line mapping.
- Focused validation: Scoped coverage run before and after; compare lcov totals and run `make check-dependencies`.
- Handoff: Remove COVT-04 and pass the profile and line-mapping evidence to COVT-05.

## COVT-05 — Publish the measured loop and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COVT-03, COVT-04
- Exact write set: `docs/src/learn/developer.md`, `.wiki/testing/coverage-throughput.md`, `.wiki/testing/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: developer verification guidance
- Outcome: Publish cold/warm measurements, coverage scope and line-mapping evidence, the named slow gate, and the recommended inner loop; retire this plan.
- Focused validation: Run the full local loop once, verify the documented targets and links, and validate the wiki.
- Handoff: Delete this plan and its index, network, and lane entries after measurements survive in developer documentation.
