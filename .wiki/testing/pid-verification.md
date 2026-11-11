---
okf_version: "0.1"
type: Plan
title: Consolidate and verify portable PID algorithms
description: Prove the shared ORCID/ISNI checksum with Verus first, then consolidate PID helpers, add Miri, and extend production-connected checksum proofs.
difficulty: high
impact: medium
package: [workspace, acorn-core, acorn-pid-checksum, acorn-schema]
tags: [testing, pid, miri, verus, verification, checksum, portability]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Give portable PID behavior one implementation and two complementary verification layers, starting with the shared ORCID/ISNI checksum:

- native characterization plus shared parsing/percent-validation helpers;
- Miri over public alloc-only APIs; and
- Verus proofs for executable check-digit cores that production PID paths call.

The tools complement ordinary tests. They do not replace parsing, formatting, fixture, fuzz, or wrapper-level regression coverage.

# Current evidence

- Handle, RRID, and SWHID repeat generic `PersistentIdentifierParse::{format,from_string,is_valid}` bodies. Handle and SWHID also repeat percent-escape validation while retaining different discovery and identifier rules.
- ARK, ISBN, ISNI, ORCID, and ROR expose checksum behavior through public traits and share four arithmetic algorithms plus beta-numeric/Crockford helpers.
- `mod_11_2_check_digit` is shared by ORCID and ISNI. It folds the first 15 normalized characters, treating a non-decimal character as zero; the current parsing and validity paths enforce identifier shape separately. The proof must state its 15-decimal-digit domain and cannot establish parser correctness by itself.
- `acorn-core` and `acorn-schema` compile with `--no-default-features --features alloc`.
- No Miri or Verus job exists, and `verus`/`cargo-verus` are not installed locally. Toolchain and workspace compatibility remain unproven.
- Existing native PID tests contain the executable vectors that should remain the oracle.
- [Verus Cargo integration](https://verus-lang.github.io/verus/guide/cargo_verus.html) supports opted-in crates alongside ordinary Cargo builds, while [its supported-feature list](https://verus-lang.github.io/verus/guide/features.html) makes direct verification of the entire schema crate an unsuitable first target.

# Project code boundaries

- **Update:** PID helper/caller modules, Crockford utilities, adjacent tests, workspace manifests, `Makefile`, CI, and contributor guidance.
- **Create:** a small `acorn-pid-checksum` crate for executable verified cores, `crates/acorn-schema/tests/miri.rs`, and pinned Verus setup guidance.
- **Delete:** duplicate generic parse and percent-validation bodies after characterization tests. Do not delete native vectors or make private checksum helpers public.

# Dependencies

Serialize PID roots with dPID or other schema PID changes. PID-06 claims the workspace manifest/lockfile integration lock and may proceed before PID-01. Miri and Verus automation share the CI/build lock, but neither blocks the portability critical path or ordinary stable builds.

# Decisions

- Add bounded default implementations for the three generic parse methods while leaving identifier-specific `find_all`, trimming, punctuation, and validity rules local.
- Use one crate-private percent-escape syntax helper for Handle and SWHID.
- The Miri target is deterministic, in-memory, alloc-only, and exercises only public APIs. It excludes filesystems, network, processes, FFI, scheduling, and wall-clock behavior.
- Put the first Verus `exec` core in a small workspace crate and prove its MOD 11-2 result, bounds, overflow freedom, and termination for 15 numeric digits. PID-08 connects that compiled function to the existing ORCID/ISNI paths; a disconnected proof of copied code is insufficient.
- Preserve malformed-input behavior at the existing schema boundary. Explicit normalization and digit/length checks establish the core's preconditions; regex parsing stays outside the proof boundary. Do not assert that Verus has verified the regex or an unproved wrapper.
- Pin the Verus release and its required Rust toolchain separately from the Miri toolchain. Ordinary stable Cargo builds must not require the verifier binary; proof dependencies may be compile-time dependencies only, with no verifier process or proof code at runtime.
- If PID-06 cannot produce a reproducible assumption-free proof and a passing ordinary build within its bounded phase, record the failure and stop the dependent proof phases for replanning. Do not substitute an unverified implementation or silently switch tools.
- Start Miri and Verus CI jobs as informational and path-filtered. Do not use `assume`, `external_body`, blanket ignore attributes, or solver-disable flags to claim the checksum result; [Verus documents these as trusted assumptions](https://verus-lang.github.io/verus/guide/tcb.html).

# Implementation sequence

1. Implement and prove the MOD 11-2 executable core with pinned Verus tooling, then connect it to ORCID and ISNI with explicit wrapper preconditions.
2. Characterize and consolidate the independent Handle/RRID/SWHID parsing helpers; add the curated Miri target for public portable PID boundaries.
3. Extend production-connected proofs in order: ISBN-13 and ISBN-10, ROR, then ARK/NOID.
4. Wire documented `make miri` and `make verus` entry points plus informational CI jobs.

# Validation

- Run native PID suites, alloc-only checks, Miri with one ordinary and one multi-seed pass, then pinned `cargo verus verify` on the checksum crate.
- Require zero unsupported items, `assume`, `external_body`, blanket ignore attributes, or disabled proof obligations for claimed results; record any remaining trusted boundary explicitly.
- Run lint, the full suite, portable and WASM checks after production-helper changes.

# Acceptance criteria

- Handle, RRID, and SWHID use shared generic parse methods, and Handle/SWHID share percent validation without merging identifier-specific rules.
- The curated Miri target passes under a pinned toolchain and covers core/schema round trips plus all five checksum-bearing PID public boundaries.
- The shared MOD 11-2 executable core is proved first under Verus, and ORCID/ISNI production paths call it after explicit domain checks. Invalid inputs retain their current observable behavior.
- ISBN-10/13, ROR, and ARK/NOID subsequently call proved executable cores with contracts for overflow freedom, bounds, and their checksum equations; lookup claims cover beta-numeric and Crockford mappings where used.
- Every native checksum vector remains and has a corresponding proof example or theorem.
- Ordinary stable builds require no Verus executable or proof-time service, and informational jobs run only for relevant changes.

# Execution phases

## PID-06 — Implement the first Verus checksum core

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none; serialize workspace manifest/lockfile edits with other owners
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-pid-checksum/Cargo.toml`, `crates/acorn-pid-checksum/src/lib.rs`, `crates/acorn-pid-checksum/src/mod11_2.rs`, `docs/src/learn/developer.md`
- Primary integration lock: workspace manifests and lockfile
- Outcome: Pin Verus and implement a verified executable MOD 11-2 core over exactly 15 numeric digits, including its checksum equation, result range, arithmetic safety, and termination. Keep the new crate portable and document the local verifier invocation.
- Focused validation: Run pinned `cargo verus verify -p acorn-pid-checksum`, ordinary `cargo test -p acorn-pid-checksum`, and a stable workspace check; record any unsupported dependency or toolchain failure.
- Handoff: Remove PID-06 after a reproducible proof and build; otherwise record the stop decision and prune or replan dependent proof phases.

## PID-01 — Characterize repeated PID parsing

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-schema/src/pid/handle/tests.rs`, `crates/acorn-schema/src/pid/rrid/tests.rs`, `crates/acorn-schema/src/pid/swhid/tests.rs`, `.wiki/testing/pid-verification.md`
- Primary integration lock: portable PID helpers
- Outcome: Freeze format/from-string/validity, discovery, punctuation, and percent behavior before consolidation.
- Focused validation: Run all existing and added Handle, RRID, and SWHID vectors.
- Handoff: Remove PID-01 and preserve behavior differences explicitly.

## PID-02 — Add shared parse defaults

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-01
- Exact write set: `crates/acorn-schema/src/pid/mod.rs`
- Primary integration lock: portable PID helpers
- Outcome: Use bounded default parse methods while leaving identifier-specific discovery and validity local.
- Focused validation: Run focused PID plus alloc/no-default/WASM checks.
- Handoff: Remove PID-02 and record the shared/default boundary.

## PID-03 — Share percent-escape syntax validation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-01
- Exact write set: `crates/acorn-schema/src/pid/handle/mod.rs`, `crates/acorn-schema/src/pid/swhid/mod.rs`, `crates/acorn-schema/src/pid/tests.rs`
- Primary integration lock: portable PID helpers
- Outcome: Reuse one crate-private syntax helper without merging identifier-specific rules.
- Focused validation: Run valid/invalid/truncated/case/Unicode percent vectors for both identifiers.
- Handoff: Remove PID-03 and freeze helper semantics.

## PID-04 — Add the curated Miri target

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-02, PID-03
- Exact write set: `crates/acorn-schema/tests/miri.rs`, `tests/fixtures/pid/miri.json`
- Primary integration lock: verification test target
- Outcome: Exercise public Location, Repository, ResearchActivity, codec, and checksum PID boundaries in memory.
- Focused validation: Run the curated target under Miri with default strict checks.
- Handoff: Remove PID-04 and publish the deterministic target command.

## PID-05 — Wire Miri automation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-04
- Exact write set: `scripts/pid/miri-toolchain.toml`, `Makefile`, `.gitlab-ci.yml`, `docs/src/learn/developer.md`, `.wiki/testing/pid-verification.md`
- Primary integration lock: CI build entry points
- Outcome: Add documented path-filtered informational Miri execution without weakening checks.
- Focused validation: Dry-run Make, lint CI, and run the job once on relevant and irrelevant changes.
- Handoff: Remove PID-05 and preserve the pinned invocation.

## PID-07 — Verify the ISBN-13 checksum core

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-08
- Exact write set: `crates/acorn-pid-checksum/src/isbn.rs`, `crates/acorn-pid-checksum/src/lib.rs`, `crates/acorn-schema/src/pid/mod.rs`, `crates/acorn-schema/src/pid/isbn/tests.rs`
- Primary integration lock: checksum proof core
- Outcome: Prove ISBN-13's alternating-weight MOD 10 rule and route valid production inputs through its verified executable core.
- Focused validation: Run native vectors and the pinned proof obligations.
- Handoff: Remove PID-07 and preserve wrapper lemmas.

## PID-08 — Connect the verified ORCID and ISNI core

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-06
- Exact write set: `crates/acorn-schema/Cargo.toml`, `Cargo.lock`, `crates/acorn-schema/src/pid/mod.rs`, `crates/acorn-schema/src/pid/isni/tests.rs`, `crates/acorn-schema/src/pid/orcid/tests.rs`
- Primary integration lock: schema PID integration
- Outcome: Call the PID-06 core on explicitly checked 15-digit payloads in both production paths, preserve malformed-input behavior, and test normalized bare, spaced, hyphenated, invalid, and `X` cases without public API expansion.
- Focused validation: Run native vectors and pinned proof obligations.
- Handoff: Remove PID-08 and preserve wrapper lemmas.

## PID-09 — Verify the ROR checksum core

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-08
- Exact write set: `crates/acorn-pid-checksum/src/ror.rs`, `crates/acorn-pid-checksum/src/lib.rs`, `crates/acorn-schema/src/pid/mod.rs`, `crates/acorn-schema/src/pid/ror/tests.rs`
- Primary integration lock: checksum proof core
- Outcome: Prove the ROR MOD 97-10 equation and Crockford decoding boundary, then call the verified executable core from valid ROR inputs.
- Focused validation: Run native vectors and pinned proof obligations.
- Handoff: Remove PID-09 and preserve wrapper lemmas.

## PID-10 — Verify ARK and NOID checksum cores

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-08
- Exact write set: `crates/acorn-pid-checksum/src/radix29.rs`, `crates/acorn-pid-checksum/src/lib.rs`, `crates/acorn-schema/src/pid/mod.rs`, `crates/acorn-schema/src/pid/ark/tests.rs`, `crates/acorn-schema/src/pid/tests.rs`
- Primary integration lock: checksum proof core
- Outcome: Prove beta-numeric lookup reversibility, bounds, and the radix-29 checksum equation, then call the verified executable core from valid ARK/NOID inputs.
- Focused validation: Run native vectors and pinned proof obligations.
- Handoff: Remove PID-10 and preserve wrapper lemmas.

## PID-11 — Wire proof automation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-07, PID-09, PID-10, PID-13
- Exact write set: `Makefile`, `.gitlab-ci.yml`, `docs/src/learn/developer.md`
- Primary integration lock: CI build entry points
- Outcome: Add a documented `verus` target plus path-filtered informational CI for the pinned proof crate.
- Focused validation: Run all proof targets and confirm unrelated changes skip the job.
- Handoff: Remove PID-11 and preserve the pinned proof workflow.

## PID-13 — Verify the ISBN-10 checksum core

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-07
- Exact write set: `crates/acorn-pid-checksum/src/isbn.rs`, `crates/acorn-schema/src/pid/mod.rs`, `crates/acorn-schema/src/pid/isbn/tests.rs`
- Primary integration lock: checksum proof core
- Outcome: Prove the ISBN-10 MOD 11 rule, including the `X` result, and route valid production inputs through its verified executable core.
- Focused validation: Run ISBN-10 native vectors, alloc-only checks, and the pinned Verus proof.
- Handoff: Remove PID-13 and preserve the production-connected checksum contract.

## PID-12 — Publish verification guidance and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-05, PID-11
- Exact write set: `docs/src/learn/developer.md`, `.wiki/index.md`, `.wiki/testing/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/testing/pid-verification.md`
- Primary integration lock: verification documentation
- Outcome: Document supported targets/toolchains, run native/portable/full gates, and remove the plan.
- Focused validation: Run PID tests, Miri, Verus proofs, portability, `make lint`, `make test`, docs, and wiki validation.
- Handoff: Delete this plan and every index, network, and lane entry.
