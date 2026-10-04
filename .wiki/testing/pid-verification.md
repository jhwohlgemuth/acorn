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
