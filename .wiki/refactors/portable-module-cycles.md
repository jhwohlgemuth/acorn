---
type: Plan
title: Remove three portable module cycles
description: Break three small, source-verified sibling-module cycles without changing public behavior or serialized data.
difficulty: medium
impact: medium
package: [acorn-analysis, acorn-core, acorn-schema]
tags: [refactor, architecture, module-cycles, portability]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Make the `acorn-analysis` `check`/`fair`, `acorn-core` `alloc`/`constants`, and `acorn-schema` `vocabulary::types`/`matching` dependencies one-way. Preserve public paths, wire shapes, and portable feature builds.

# Current evidence (November 27, 2026)

- `deslop-please --json` at `a78440bf` plus the worktree reported 279 diagnostics: 18 module cycles, 22 tangles, 214 field leaks, 9 shallow modules, 8 wide directories, and 8 long files. Its nonzero exit reflects the reported diagnostics, not a failed Cargo check. This is one host/default-feature configuration; other targets and features were not scanned.
- `crates/acorn-analysis/src/check.rs:172` stores `FairDetails`; `fair.rs:2` imports `Check`, `CheckCategory`, `CheckSeverity`, and `Finding` from its sibling. The checker reports both directions.
- `crates/acorn-core/src/util/constants/app.rs:78` constructs `SemanticVersion`, defined at `util/alloc/mod.rs:570`; `alloc/mod.rs:1344` uses constants from `constants`. The checker reports both directions. Keep the exported `SUPPORTED_EXEC_VERSION` path stable while changing its internal representation or dependency.
- `crates/acorn-schema/src/vocabulary/types.rs:286` calls `stable_unique` in `matching.rs:174`; `matching.rs:2` imports vocabulary types. The helper uses only strings, vectors, and a set, so it can live below both modules without introducing a new public API.
- These three cycles have narrow source paths and can be validated per package. The much larger `acorn-lib` and `acorn-schema` root cycles overlap the [crate boundary architecture](crate-boundary-architecture.md), provider, analyzer, and schema ownership plans; a checker count alone is not grounds for a cross-cutting extraction.

# Triage decisions

- Prioritize the three small cycles above because each has a clear back edge and a bounded owner. Recheck each diagnostic against current source before editing; the worktree is actively changing.
- Treat the 214 field leaks as review leads, not a bulk encapsulation task. Many are intentional public DTO/schema fields (for example `ResearchActivityMetadata` and `Cff`), and changing them would affect builders, serialization, and consumers. Review a field only when an existing plan changes its contract or a concrete invariant is violated.
- Do not move files solely to satisfy the 8 long-file, 8 wide-directory, or 9 shallow-module thresholds. Split a file when a cohesive owner and focused tests are identified during related work. The 22 tangles can be revisited after these cycles and existing extraction plans land; record a new atomic plan only for a source-verified remaining boundary.
- Do not add a workspace-wide allowlist or make `deslop-please` a CI gate from this baseline. Such a gate needs a stable feature/target matrix, diagnostic ownership, and a reviewed debt baseline.

# Boundaries and dependencies

No schema, CLI, or feature contract changes. Coordinate `DES-01` with vocabulary/schema work, `DES-02` with core/version work, and `DES-03` with analyzer/check work through the [parallel execution guide](../parallel-execution.md). The phases are independent, but each holds only its named package lock. Existing dirty worktree files outside these exact sets remain untouched.

# Validation and acceptance

Each phase preserves relevant package tests and portable checks and makes its named `module-cycle` diagnostic disappear under the same host/default-feature checker configuration. The remaining diagnostic total may change with concurrent work; compare by lint key and module members, not only total count. After all three phases, run workspace format/Clippy and the portable WASM gate when integration locks are free. Record any newly observed behavior or compatibility change before marking a phase accepted.

# Execution phases

## DES-01 — Make vocabulary matching depend on types only

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: confirm current vocabulary sources and no concurrent schema edit
- Exact write set: `crates/acorn-schema/src/vocabulary/types.rs`, `crates/acorn-schema/src/vocabulary/matching.rs`, `crates/acorn-schema/src/vocabulary/mod.rs`
- Primary integration lock: Schema roots
- Outcome: Move or locally express the order-preserving deduplication helper so `types` no longer imports `matching`, while keeping accepted/lossless vocabulary output unchanged.
- Focused validation: vocabulary tests in `acorn-schema`, package Clippy and format, portable `acorn-schema` check, and `deslop-please acorn-schema --json` filtered to the `matching`/`types` cycle.
- Handoff: Record the removed edge, behavior checks, and any remaining schema diagnostic keys.

## DES-02 — Remove the core version-constant back edge

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: confirm `SUPPORTED_EXEC_VERSION` consumers and no concurrent core edit
- Exact write set: `crates/acorn-core/src/util/constants/app.rs`, `crates/acorn-core/src/util/alloc/mod.rs`, `crates/acorn-core/src/util/constants/mod.rs`
- Primary integration lock: Library roots
- Outcome: Keep the public version constant and its value, while removing `constants`' dependency on `alloc` or the reverse dependency without duplicating version policy.
- Focused validation: `acorn-core` tests under default and portable feature sets, package Clippy and format, and `deslop-please acorn-core --json` filtered to the `alloc`/`constants` cycle.
- Handoff: Record the preserved public path and the removed edge.

## DES-03 — Separate FAIR evidence from check conversion

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: confirm FAIR/check call paths and no concurrent analyzer edit
- Exact write set: `crates/acorn-analysis/src/check.rs`, `crates/acorn-analysis/src/fair.rs`, `crates/acorn-analysis/src/lib.rs`
- Primary integration lock: Analyzer/check contract
- Outcome: Give FAIR evidence and check conversion one dependency direction while preserving serialized check details and the 41-indicator assessment behavior.
- Focused validation: `acorn-analysis` FAIR/check tests, package Clippy and format, portable `acorn-analysis` check, and `deslop-please acorn-analysis --json` filtered to the `check`/`fair` cycle.
- Handoff: Record the dependency direction and output parity evidence; close this plan after all three cycle keys are absent.
