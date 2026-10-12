---
okf_version: "0.1"
type: Plan
title: Consolidate and verify portable PID algorithms
description: Remove duplicate PID parsing helpers, add a curated Miri suite, and formally verify shared checksum cores with pinned Creusot tooling.
difficulty: high
impact: medium
package: [workspace, acorn-core, acorn-schema]
tags: [testing, pid, miri, creusot, verification, checksum, portability]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Give portable PID behavior one implementation and two complementary verification layers:

- native characterization plus shared parsing/percent-validation helpers;
- Miri over public alloc-only APIs; and
- Creusot/Why3 proofs for the shared check-digit arithmetic.

The tools complement ordinary tests. They do not replace parsing, formatting, fixture, fuzz, or wrapper-level regression coverage.

# Current evidence

- Handle, RRID, and SWHID repeat generic `PersistentIdentifierParse::{format,from_string,is_valid}` bodies. Handle and SWHID also repeat percent-escape validation while retaining different discovery and identifier rules.
- ARK, ISBN, ISNI, ORCID, and ROR expose checksum behavior through public traits and share four arithmetic algorithms plus beta-numeric/Crockford helpers.
- `acorn-core` and `acorn-schema` compile with `--no-default-features --features alloc`.
- No Miri, Creusot, Why3, or solver job exists. The fuzz nightly is not evidence of compatibility with either tool.
- Existing native PID tests contain the executable vectors that should remain the oracle.

# Project code boundaries

- **Update:** PID helper/caller modules, Crockford utilities, adjacent tests, `Makefile`, CI, and contributor guidance.
- **Create:** `crates/acorn-schema/tests/miri.rs`, private proof-friendly cores only where existing wrappers cannot be modeled, and pinned Creusot/Why3/solver configuration outside runtime dependencies.
- **Delete:** duplicate generic parse and percent-validation bodies after characterization tests. Do not delete native vectors or make private checksum helpers public.

# Dependencies

Serialize PID roots with dPID or other schema PID changes. Miri and Creusot wiring share the CI/build lock, but neither blocks the portability critical path or ordinary stable builds.

# Decisions

- Add bounded default implementations for the three generic parse methods while leaving identifier-specific `find_all`, trimming, punctuation, and validity rules local.
- Use one crate-private percent-escape syntax helper for Handle and SWHID.
- The Miri target is deterministic, in-memory, alloc-only, and exercises only public APIs. It excludes filesystems, network, processes, FFI, scheduling, and wall-clock behavior.
- Creusot contracts stay on private shared helpers or private extracted arithmetic cores. Regex parsing remains outside the proof boundary; wrapper lemmas state the exact normalized ASCII and length preconditions.
- Pin a Creusot-compatible nightly, Creusot, Why3, and two SMT solvers separately from the fuzz/Miri nightly.
- Start both CI jobs as informational and path-filtered. Do not disable isolation, aliasing, leak checks, or proof obligations globally.

# Implementation sequence

1. Characterize current Handle/RRID/SWHID behavior, consolidate generic parse methods and percent validation, and pass portable checks.
2. Add the curated Miri target for `Location`, `Repository`, `ResearchActivity`, codecs, and all five checksum-bearing PID public boundaries.
3. Pin proof tools and define invariant-friendly private cores only where current allocation/iterator wrappers block verification.
4. Prove overflow freedom, bounds, lookup reversibility, and functional equations for ISBN MOD 10/11, ISNI/ORCID MOD 11-2, ROR MOD 97-10, and ARK/NOID radix-29 checks.
5. Wire documented `make miri`, `make creusot`, and `make creusot-prove` entry points plus informational CI jobs.

# Validation

- Run native PID suites, alloc-only checks, Miri with one ordinary and one multi-seed pass, then the pinned Creusot translation and two solvers.
- Require zero unsupported items, `admit`, `assume`, blanket ignore attributes, or solver-disable flags in the proof boundary.
- Run lint, the full suite, portable and WASM checks after production-helper changes.

# Acceptance criteria

- Handle, RRID, and SWHID use shared generic parse methods, and Handle/SWHID share percent validation without merging identifier-specific rules.
- The curated Miri target passes under a pinned toolchain and covers core/schema round trips plus all five checksum-bearing PID public boundaries.
- All four shared algorithms and beta-numeric/Crockford lookup have contracts for overflow freedom, bounds, and functional correctness.
- Wrapper lemmas cover ARK/NOID, ISBN-10/13, ISNI/ORCID, and ROR without making private helpers public.
- Every native checksum vector remains and has a corresponding proof example or theorem.
- Stable builds gain no proof runtime dependency, and informational jobs run only for relevant changes.

# Execution phases

## PID-01 — Characterize repeated PID parsing

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: Handle/RRID/SWHID parser tests, this plan
- Primary integration lock: portable PID helpers
- Outcome: Freeze format/from-string/validity, discovery, punctuation, and percent behavior before consolidation.
- Focused validation: Run all existing and added Handle, RRID, and SWHID vectors.
- Handoff: Remove PID-01 and preserve behavior differences explicitly.

## PID-02 — Add shared parse defaults

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-01
- Exact write set: `crates/acorn-schema/src/pid/mod.rs`, Handle/RRID/SWHID modules and tests
- Primary integration lock: portable PID helpers
- Outcome: Use bounded default parse methods while leaving identifier-specific discovery and validity local.
- Focused validation: Run focused PID plus alloc/no-default/WASM checks.
- Handoff: Remove PID-02 and record the shared/default boundary.

## PID-03 — Share percent-escape syntax validation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-01
- Exact write set: Handle/SWHID modules, percent helper tests
- Primary integration lock: portable PID helpers
- Outcome: Reuse one crate-private syntax helper without merging identifier-specific rules.
- Focused validation: Run valid/invalid/truncated/case/Unicode percent vectors for both identifiers.
- Handoff: Remove PID-03 and freeze helper semantics.

## PID-04 — Add the curated Miri target

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-02, PID-03
- Exact write set: Miri integration test target, Miri fixture module
- Primary integration lock: verification test target
- Outcome: Exercise public Location, Repository, ResearchActivity, codec, and checksum PID boundaries in memory.
- Focused validation: Run the curated target under Miri with default strict checks.
- Handoff: Remove PID-04 and publish the deterministic target command.

## PID-05 — Wire Miri automation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-04
- Exact write set: `Makefile`, `.gitlab-ci.yml`, Miri toolchain pin
- Primary integration lock: CI build entry points
- Outcome: Add documented path-filtered informational Miri execution without weakening checks.
- Focused validation: Dry-run Make, lint CI, and run the job once on relevant and irrelevant changes.
- Handoff: Remove PID-05 and preserve the pinned invocation.

## PID-06 — Prove the Creusot toolchain

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-01
- Exact write set: Creusot/Why3/solver toolchain pins, proof feasibility record
- Primary integration lock: proof toolchain
- Outcome: Demonstrate compatible pinned tools and private-core contracts without stable runtime dependencies.
- Focused validation: Prove one representative arithmetic helper or record the stop decision.
- Handoff: Remove PID-06 or prune proof phases if feasibility fails.

## PID-07 — Verify ISBN checksum cores

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-06
- Exact write set: private ISBN proof core, ISBN proof examples
- Primary integration lock: checksum proof core
- Outcome: Prove MOD 10/11 bounds, overflow freedom, and functional correctness.
- Focused validation: Run native vectors and the pinned proof obligations.
- Handoff: Remove PID-07 and preserve wrapper lemmas.

## PID-08 — Verify ISNI and ORCID checksum cores

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-06
- Exact write set: private MOD 11-2 proof core, ISNI/ORCID proof examples
- Primary integration lock: checksum proof core
- Outcome: Prove shared arithmetic and both wrapper preconditions without public API expansion.
- Focused validation: Run native vectors and pinned proof obligations.
- Handoff: Remove PID-08 and preserve wrapper lemmas.

## PID-09 — Verify the ROR checksum core

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-06
- Exact write set: private MOD 97-10 proof core, ROR proof examples
- Primary integration lock: checksum proof core
- Outcome: Prove bounds, lookup behavior, overflow freedom, and functional correctness.
- Focused validation: Run native vectors and pinned proof obligations.
- Handoff: Remove PID-09 and preserve wrapper lemmas.

## PID-10 — Verify ARK and NOID checksum cores

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-06
- Exact write set: private radix-29 proof core, ARK/NOID proof examples
- Primary integration lock: checksum proof core
- Outcome: Prove Crockford/beta-numeric lookup reversibility, bounds, and checksum equations.
- Focused validation: Run native vectors and pinned proof obligations.
- Handoff: Remove PID-10 and preserve wrapper lemmas.

## PID-11 — Wire proof automation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-07, PID-08, PID-09, PID-10
- Exact write set: `Makefile`, `.gitlab-ci.yml`, proof runner documentation
- Primary integration lock: CI build entry points
- Outcome: Add documented `creusot` and `creusot-prove` targets plus path-filtered informational CI.
- Focused validation: Run all proof targets and confirm unrelated changes skip the job.
- Handoff: Remove PID-11 and preserve the pinned proof workflow.

## PID-12 — Publish verification guidance and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-05, PID-11
- Exact write set: verification documentation, wiki indexes, this plan
- Primary integration lock: verification documentation
- Outcome: Document supported targets/toolchains, run native/portable/full gates, and remove the plan.
- Focused validation: Run PID tests, Miri, proofs, portability, `make lint`, `make test`, docs, and wiki validation.
- Handoff: Delete this plan and every index, network, and lane entry.
