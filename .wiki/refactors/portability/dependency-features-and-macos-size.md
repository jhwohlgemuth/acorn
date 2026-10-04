---
okf_version: "0.1"
type: Plan
title: Narrow dependency capabilities and macOS unwind size
description: Finish explicit host-capability wiring and use measured dependency/profile changes to keep Darwin debug and test binaries below the compact-unwind limit.
difficulty: high
impact: critical
package: [workspace, acorn-lib, acorn-cli, acorn-cmd, acorn-host]
tags: [refactor, dependencies, features, portability, no-std, macos, binary-size]
status: in-progress
priority: next
---

# Goal

Complete two coupled outcomes:

1. every supported `acorn-lib` feature graph contains only the capabilities it names; and
2. the default Apple Silicon debug/test CLI keeps `__eh_frame` at or below 14 MiB without suppressing linker diagnostics or weakening unwind behavior.

The feature work is justified independently by portability and extraction. macOS section measurements decide how far profile or dependency-size work proceeds.

# Current evidence

- Minimal and alloc graphs no longer resolve earlier host-reporting, terminal, diff, generic asset, async, glob, or shell parsing leakage. The native `analysis` capability compiles independently of the full `std` umbrella and has dependency-deny checks.
- `acorn-core`, `acorn-schema`, `acorn-cmd`, and initial `acorn-host` capabilities are extracted and used directly.
- `acorn-lib::std` still enables unrelated `agentic`, `analysis`, `shell-lint`, PowerPoint, DuckDB, network, archive, template, progress, CLI utility, database, chart, and system-diagnostic capabilities.
- The measured Apple Silicon baseline was a roughly 317 MiB debug binary with `__eh_frame` of 21,230,536 bytes, 4,453,320 bytes above Apple's 16 MiB compact-unwind encoding limit.
- The workspace already optimizes `insta` and `similar` in development builds, but it has not measured a general dependency optimization profile against the unwind section.
- Cargo test artifacts require unwind behavior, so release-only settings or `panic = "abort"` do not solve the developer/test warning.

# Project code boundaries

- **Update:** workspace and crate manifests, capability gates and module exports, supported feature-check commands, size measurement tooling, and CI measurement jobs.
- **Create:** per-crate feature tables for accepted extractions and a stable machine-readable Mach-O section measurement helper only if manual comparison is error-prone.
- **Delete:** broad or unconditional dependency wiring after narrower graphs compile independently. Do not delete diagnostics, default CLI capabilities, or test coverage merely to meet the size target.

# Dependencies

This plan is a prerequisite for new crate extraction. Manifest, lockfile, Makefile, and CI changes hold the workspace-graph and CI/build locks. A profile or feature change is accepted only after default, minimal, alloc, analyzer, analysis, std, WASM, and CLI behavior remain green.

# Decisions

- Keep `std` as standard-library availability, not an implicit bundle of every host service.
- Keep network/model download, process discovery, link checking, document/template/PDF/PowerPoint/chart, databases, servers, analysis, and shell linting behind coherent named capabilities.
- Configure interface crates with `default-features = false` plus an explicit capability union once the equivalent graph passes.
- Do not add one-use aliases that merely rename dependencies, and do not extract a crate just to hide feature coupling.
- Measure `__eh_frame` directly with `size -m` and `otool -l`; total binary or dSYM size is not a substitute.
- Do not allow `linker_messages`, pass `-no_compact_unwind`, remove backtraces, or rely on release-only builds.
- Stop at the first measured phase that reaches 14 MiB with acceptable build/debug cost and all behavior preserved.

# Implementation sequence

## 1. Reproduce and automate measurement

Record commit/dirty state, Rust and linker versions, architecture, feature set, profile, command, `__text`, `__gcc_except_tab`, `__unwind_info`, `__eh_frame`, total size, and clean/incremental build time. Require two materially consistent baseline runs.

## 2. Finish capability inventory and enforcement

Map every optional dependency to its consuming module and command. Establish independent `agentic`, command/shell-lint, and remaining host cuts; update dependency-deny checks with each supported graph.

## 3. Test the smallest profile change

Measure `[profile.dev.package."*"] opt-level = 1`, then `2` or `s` only if required. Keep workspace crates debuggable and change one profile dimension per experiment.

## 4. Attribute by capability

Build a controlled default/PDF/TUI/minimal-library/capability matrix and verify resolved features with `cargo tree --edges features`. Keep only feature changes that express a stable boundary; use unwind deltas as supporting evidence.

## 5. Reduce one measured contributor if needed

Disable unused dependency features, remove proven duplicate stacks, or reduce measured monomorphization. A dependency replacement or helper-binary design exceeding this plan's atomic boundary requires a new reviewed plan.

# Validation

- Run all supported feature checks, dependency-deny scripts, `make check-portable`, WASM package tests, lint, and the full suite.
- On macOS, build the default debug CLI and test artifact, confirm the linker warning is absent, and record sections with both `size -m` and `otool -l`.
- Exercise download, analysis/checking, export/PDF, doctor, TUI, and agentic commands affected by feature changes.
- Build and smoke-test release packaging with existing release semantics.
- Run normal Linux CI and inspect resolved graphs for every supported combination.

# Acceptance criteria

- Every supported no-default or named-capability graph contains only its required dependencies, and `std` no longer selects unrelated host services by itself.
- Interface crates request explicit capability unions and unavailable capabilities fail at a clear parse or execution boundary.
- A default Apple Silicon debug/test `acorn` artifact has `__eh_frame <= 14,680,064` bytes.
- The Apple linker emits no compact-unwind size warning, compact unwind stays enabled, and no linker diagnostic is suppressed.
- Default CLI capabilities, panic/backtrace assumptions, foreign-exception behavior, and release packaging remain supported.
- The final evidence records before/after sections, toolchain, feature graph, build-time tradeoffs, and validation commands.
- New extracted crates begin with explicit feature and owned-error contracts; no compatibility feature is added solely to conceal old ownership.
