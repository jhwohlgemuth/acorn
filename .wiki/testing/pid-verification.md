---
okf_version: "0.1"
type: Plan
title: Consolidate and verify portable PID algorithms
description: Gate the Verus toolchain and a portable checksum crate, prove the shared ORCID/ISNI core, then consolidate PID helpers, add Miri, and extend production-connected checksum proofs.
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
- Neither `Makefile` nor `.gitlab-ci.yml` contains a Miri or Verus target, and `verus`/`cargo-verus` are not installed locally. Toolchain and workspace compatibility remain unproven.
- `rust-toolchain.toml` pins `stable` for the whole workspace. Verus ships its own pinned nightly, so the two pins must coexist through a crate-local toolchain file rather than a workspace change.
- Verus verifies `while` loops carrying invariants, not iterator combinators, so executable cores cannot use the `try_fold` shape of the current helpers. This conflicts with the repository's functional-programming guideline and needs a recorded scoped exception before implementation, not during it.
- `vstd` is a proof-time dependency and is not part of an ordinary or WASM build. It must sit behind a non-default feature so `cargo build`, `cargo test`, and `make check-wasm` never resolve it.
- Observed at the schema boundary: `ORCID::from_string("garbage").check_digit()`, the empty string, a truncated `0000-0002`, and `ISNI::from_string("garbage").check_digit()` all answer `Some(['1'])`, the complement of the empty fold. Malformed input is not currently rejected, so a verified core cannot simply replace the helper; preserving behavior requires an explicit fallback to the existing fold.
- An ORCID identifier carries its check character inside the string the checksum normalizes: `0000-0002-2816-415X` compacts to `000000022816415X`, and `take(15)` stops before the `X`. A verified normalizer that validates the whole string rejects every ORCID whose check value is ten. A draft sketch of this plan's own normalizer had that defect.
- Width analysis from the same sketches: the MOD 11-2 intermediate is bounded at 38 (`u8`), ISBN-13 at 324 and ISBN-10 at 486 (`u16`), radix-29 at 840 (`u16`), and ROR at `32^6 * 100 = 107_374_182_400` (`u64`), where `mod_97_10_check_digit` currently uses `u128` with `checked_mul`. These bounds are the arithmetic-safety obligations, and proving them removes the checked-arithmetic chains.
- Beta-numeric reversibility is currently sampled at two points, `'0'` and `'z'` (`crates/acorn-schema/src/pid/tests.rs:103`). A lookup lemma covers all twenty-nine characters at once.
- Existing native PID tests contain the executable vectors that should remain the oracle. Their folds are: MOD 11-2 `000000031415926` and `0000000492299539` both fold to 3 (check `9`), `0000-0002-2816-415X` to 2 (check `X`); ISBN-10 `0-306-40615-2` to 130 (check `2`) and `0-8044-2957-X` to 199 (check `X`); ISBN-13 `978-0-306-40627-0` and `978-3-16-148410-0` both to 100 (check `0`); ROR `1qz5mb` decodes to 58693259 (check `56`); NOID `13030/xf93gt2` to 21 (check `q`) and `c8131/g3js3` to 25 (check `v`).
- [Verus Cargo integration](https://verus-lang.github.io/verus/guide/cargo_verus.html) supports opted-in crates alongside ordinary Cargo builds, while [its supported-feature list](https://verus-lang.github.io/verus/guide/features.html) makes direct verification of the entire schema crate an unsuitable first target.

# Project code boundaries

- **Update:** PID helper/caller modules, Crockford utilities, adjacent tests, workspace manifests, `Makefile`, CI, and contributor guidance.
- **Create:** a small `acorn-pid-checksum` crate for executable verified cores with a crate-local toolchain pin, `crates/acorn-schema/tests/miri.rs`, and pinned Verus setup guidance.
- **Delete:** duplicate generic parse and percent-validation bodies after characterization tests. Do not delete native vectors or make private checksum helpers public.

# Dependencies

Serialize PID roots with dPID or other schema PID changes. PID-15 and PID-08 claim the workspace manifest/lockfile integration lock; PID-06 no longer does, so the proof work itself does not contend with ACD-03 or SPD-24. PID-01, PID-14, and PID-16 are independent of one another and of the manifest lock. PID-14, PID-05, and PID-11 all claim the CI/build lock and run in that order, but none blocks the portability critical path or ordinary stable builds.

# Decisions

- Add bounded default implementations for the three generic parse methods while leaving identifier-specific `find_all`, trimming, punctuation, and validity rules local.
- Use one crate-private percent-escape syntax helper for Handle and SWHID.
- The Miri target is deterministic, in-memory, alloc-only, and exercises only public APIs. It excludes filesystems, network, processes, FFI, scheduling, and wall-clock behavior.
- Put the first Verus `exec` core in a small workspace crate and prove its MOD 11-2 result, bounds, overflow freedom, and termination for 15 numeric digits. PID-08 connects that compiled function to the existing ORCID/ISNI paths; a disconnected proof of copied code is insufficient.
- Prepare the toolchain and its workflow entry point before writing any proof. Installing and pinning Verus is the plan's highest-risk step and gets its own phase, so no proof phase spends its budget on setup and every later phase has `make verus` available from the start.
- Scaffold the crate, its feature gating, and its native vectors before the first proof. The manifest/lockfile lock is then held by a short phase that can be scheduled against ACD-03 and SPD-24 independently of the proof work.
- Each verified core exposes a `normalize`-style entry point whose postcondition supplies the core's preconditions, so callers in `acorn-schema` discharge no proof obligation. Normalization stops once the payload is full rather than validating the trailing check character.
- Preserve malformed-input behavior at the existing schema boundary by falling back to the current unverified fold when normalization rejects the input. Claim nothing about that branch. Explicit normalization and digit/length checks establish the core's preconditions; regex parsing stays outside the proof boundary. Do not assert that Verus has verified the regex or an unproved wrapper.
- Every core follows one skeleton: a recursive `spec fn` definition, a bound lemma that discharges arithmetic safety, a prefix step lemma that links the loop accumulator to the specification, an executable contract stating the checksum equation and result range, and a `compute_only` theorem per native vector. Reuse it rather than re-deriving a proof style per identifier.
- Use the narrowest integer width the bound lemma proves, not the width the current helper uses.
- Record the loop-versus-combinator exception in contributor guidance before implementation. Inside `verus!` blocks the loop invariant is the functional specification; wrapper and non-proof code stays functional.
- Pin the Verus release and its required Rust toolchain separately from the Miri toolchain. Ordinary stable Cargo builds must not require the verifier binary; proof dependencies may be compile-time dependencies only, with no verifier process or proof code at runtime.
- If PID-14 cannot verify a trivial crate, or PID-06 cannot produce a reproducible assumption-free proof and a passing ordinary build within its bounded phase, record the failure and stop the dependent proof phases for replanning. Do not substitute an unverified implementation or silently switch tools.
- Start Miri and Verus CI jobs as informational and path-filtered. Do not use `assume`, `external_body`, blanket ignore attributes, or solver-disable flags to claim the checksum result; [Verus documents these as trusted assumptions](https://verus-lang.github.io/verus/guide/tcb.html).

# Implementation sequence

1. Prepare the Verus toolchain: install it, pin it, wire `make verus` and an informational CI job, and record the loop exception.
2. Scaffold the portable checksum crate with feature-gated proof dependencies and a crate-local toolchain pin.
3. Prove the MOD 11-2 executable core, characterize the ORCID/ISNI domain boundary, then connect both production paths with an explicit fallback.
4. Characterize and consolidate the independent Handle/RRID/SWHID parsing helpers; add the curated Miri target for public portable PID boundaries.
5. Extend production-connected proofs in order: ISBN-13 and ISBN-10, ROR, then ARK/NOID.
6. Wire the documented `make miri` entry point and extend the PID-14 proof workflow to every verified core.

# Validation

- Run native PID suites, alloc-only checks, Miri with one ordinary and one multi-seed pass, then pinned `cargo verus verify` on the checksum crate.
- Confirm on every proof phase that ordinary `cargo build`, `cargo test`, `cargo check --workspace` and `make check-wasm` succeed on the workspace `stable` pin with no verifier present and no `vstd` in the default dependency graph.
- Require zero unsupported items, `assume`, `external_body`, blanket ignore attributes, or disabled proof obligations for claimed results; record any remaining trusted boundary explicitly.
- Run lint, the full suite, portable and WASM checks after production-helper changes.

# Acceptance criteria

- Handle, RRID, and SWHID use shared generic parse methods, and Handle/SWHID share percent validation without merging identifier-specific rules.
- The curated Miri target passes under a pinned toolchain and covers core/schema round trips plus all five checksum-bearing PID public boundaries.
- A pinned Verus release and nightly are recorded, reproduce a verification on a supported host, and leave ordinary stable builds unchanged.
- The checksum crate builds and tests on stable with no proof dependency in its default graph, and passes the WASM check.
- The shared MOD 11-2 executable core is proved first under Verus, and ORCID/ISNI production paths call it after explicit domain checks. Invalid inputs retain their current observable behavior, including the `Some(['1'])` answers frozen by PID-16, and ORCID values whose check character is `X` remain valid.
- Each verified core uses the narrowest integer width its bound lemma proves, and the checked-arithmetic chain it replaces is removed rather than retained alongside the proof.
- ISBN-10/13, ROR, and ARK/NOID subsequently call proved executable cores with contracts for overflow freedom, bounds, and their checksum equations; lookup claims cover beta-numeric and Crockford mappings where used.
- Every native checksum vector remains and has a corresponding proof example or theorem.
- Ordinary stable builds require no Verus executable or proof-time service, and informational jobs run only for relevant changes.

# Execution phases

## PID-14 — Prepare the Verus toolchain and workflow

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `scripts/pid/verus-toolchain.toml`, `Makefile`, `.gitlab-ci.yml`, `docs/src/learn/developer.md`, `AGENTS.md`
- Primary integration lock: CI build entry points
- Outcome: Install a pinned Verus release with its required nightly, verify a throwaway crate outside this repository end to end, and record the exact release/nightly pair. Add the documented `make verus` entry point and a path-filtered informational CI job so the verifier is reachable from the ordinary workflow before any proof exists. Record the scoped exception that executable code inside `verus!` blocks uses loops carrying invariants rather than iterator combinators. Confirm the pinned pair coexists with the workspace `stable` pin without changing ordinary builds.
- Focused validation: `cargo verus verify` on the throwaway crate; `make verus` resolving the pinned toolchain; `make help` listing the new target; CI lint plus one run on a matching and one on a non-matching change; `cargo check --workspace` and `make check-wasm` unchanged on `stable`.
- Handoff: Remove PID-14 after recording the pinned pair, a reproducible verification, and a green workflow entry point. If Verus cannot verify a trivial crate on a supported host, record the failure and stop PID-15, PID-06, PID-07, PID-08, PID-09, PID-10, PID-11, and PID-13 for replanning rather than substituting an unverified implementation.

## PID-15 — Scaffold the portable checksum crate

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-14; serialize workspace manifest/lockfile edits with other owners
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-pid-checksum/Cargo.toml`, `crates/acorn-pid-checksum/rust-toolchain.toml`, `crates/acorn-pid-checksum/src/lib.rs`
- Primary integration lock: workspace manifests and lockfile
- Outcome: Add a `no_std`, `forbid(unsafe_code)` crate whose proof dependencies sit behind a non-default `verify` feature, with a crate-local toolchain pin and documented public items. Ordinary builds must not resolve or compile `vstd`.
- Focused validation: `cargo test -p acorn-pid-checksum`, `cargo check -p acorn-pid-checksum --no-default-features`, `make check-wasm`, and `cargo tree -p acorn-pid-checksum` showing no proof dependency by default.
- Handoff: Remove PID-15 and release the manifest lock before PID-06 begins.

## PID-06 — Prove the first Verus checksum core

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-14, PID-15
- Exact write set: `crates/acorn-pid-checksum/src/mod11_2.rs`, `crates/acorn-pid-checksum/src/lib.rs`, `docs/src/learn/developer.md`
- Primary integration lock: checksum proof core
- Outcome: Prove an executable MOD 11-2 core over exactly 15 decimal digits: the ISO 7064 identity that the fold plus the check value is congruent to one modulo eleven, a result at or below ten, `u8` arithmetic safety derived from the accumulator bound, and loop termination. Include a verified `normalize` whose postcondition supplies the core's preconditions and stops once fifteen digits are held, and restate every `MOD_11_2_CHECK_DIGIT_CASES` vector as a theorem.
- Focused validation: Pinned `cargo verus verify -p acorn-pid-checksum --features verify`, ordinary `cargo test -p acorn-pid-checksum`, and a stable workspace check; record any unsupported dependency.
- Handoff: Remove PID-06 after a reproducible assumption-free proof, and record the reusable proof skeleton for PID-07, PID-09, PID-10, and PID-13.

## PID-16 — Characterize the ORCID and ISNI checksum boundary

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-schema/src/pid/orcid/tests.rs`, `crates/acorn-schema/src/pid/isni/tests.rs`, `crates/acorn-schema/src/pid/tests.rs`
- Primary integration lock: portable PID helpers
- Outcome: Freeze the current observable checksum behavior before any wiring: malformed, empty, and truncated values answering `Some(['1'])`; bare, spaced, and hyphenated sixteen-digit ISNI; ORCID values whose `X` check character sits inside the normalized payload string; and the nineteen- and sixteen-character `is_valid` length gates.
- Focused validation: `cargo nextest run -p acorn-schema pid::orcid pid::isni pid::tests`
- Handoff: Remove PID-16 and keep the frozen vectors as PID-08's differential oracle.

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
- Prerequisites: PID-06, PID-16
- Exact write set: `crates/acorn-schema/Cargo.toml`, `Cargo.lock`, `crates/acorn-schema/src/pid/mod.rs`, `crates/acorn-schema/src/pid/isni/tests.rs`, `crates/acorn-schema/src/pid/orcid/tests.rs`
- Primary integration lock: schema PID integration
- Outcome: Call the PID-06 core on explicitly checked 15-digit payloads in both production paths, falling back to the existing unverified fold whenever normalization rejects the input so that every vector frozen by PID-16 still answers identically. Test normalized bare, spaced, hyphenated, invalid, and `X` cases without public API expansion.
- Focused validation: Run native vectors, the PID-16 boundary vectors, and pinned proof obligations.
- Handoff: Remove PID-08 and preserve wrapper lemmas and the fallback rationale.

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

## PID-11 — Extend proof automation to every verified core

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-07, PID-09, PID-10, PID-13
- Exact write set: `Makefile`, `.gitlab-ci.yml`, `docs/src/learn/developer.md`
- Primary integration lock: CI build entry points
- Outcome: Broaden the PID-14 `make verus` target and its path filter to cover every verified core, and make the informational job fail visibly on a regressed proof obligation rather than on a missing verifier.
- Focused validation: Run all proof targets, confirm unrelated changes skip the job, and confirm a deliberately broken obligation is reported.
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
