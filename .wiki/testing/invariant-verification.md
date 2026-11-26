---
okf_version: "0.1"
type: Plan
title: Invariant tests and Lean proofs for portable algorithms
description: Add property tests for untested conversion and algebra laws, add fuzz targets for untrusted parsers, and prove the Big-O dominance verdicts sound in Lean, linked to Rust through generated vectors.
difficulty: mixed
impact: high
package: [workspace, acorn-core, acorn-schema, acorn-lib, acorn-diff, acorn-cmd, acorn-service, acorn-store, acorn-web, acorn-fuzz]
tags: [testing, property-testing, proptest, fuzz, lean, verification, big-o, codec, portability]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Cover the repository's conversion, algebra, and comparison laws with generated inputs instead of hand-picked examples, and give the one user-visible mathematical claim, the Big-O `Dominance` verdict, a machine-checked soundness proof.

This plan complements [Portable PID verification](pid-verification.md), which owns Verus, Miri, and the PID checksum cores. Verus proves executable Rust with arithmetic and bounds. Lean proves mathematical models that SMT solvers handle poorly, such as asymptotics. Neither replaces the property tests, fixtures, or fuzz targets below.

# Current evidence

November 26 execution: INV-01's generated `Rational` laws now cover normalization, identities, commutativity, conditional associativity, and ordering against widened cross-products in `crates/acorn-core/src/util/math/tests.rs`. The focused suite passed at its default and `PROPTEST_CASES=4096` settings. No production code changed. Package Clippy and wiki validation are recorded in `progress.md`.

INV-02's Big-O generator covers comparison reversal, self-equivalence, transitivity of determined dominance, equivalent product rewrites, and arbitrary parser input. The `big_o` integration suite passed all 28 tests at its default and 4,096-case settings on November 26; strict `acorn-core` package Clippy passed. The generator remains in `crates/acorn-core/tests/big_o.rs` for INV-18.

INV-03's generated JSON, JSONC, and YAML value round trips, deterministic encoding, and all unsupported `MimeType` cases passed at 4,096 cases after the owner approved `serde_json/float_roundtrip`. The saved finite-float regression remains in `crates/acorn-schema/tests/proptest-regressions/codec/value.txt`. A release microbenchmark parsing five representative float strings 5 million times took 0.082 seconds per warm run without the feature and 0.157–0.158 seconds with it; this measures only that workload. Package Clippy passed. ZON property coverage remains with ZON-06.

INV-09 added `proptest` as a dev-dependency in the four target crates. `cargo check --workspace` passed and `cargo tree -e normal --invert proptest` found no production dependency path. The unrelated alloc-free `acorn-core` import and impl were gated behind `alloc`; the full `make check-wasm` matrix then passed with Rustup's installed WASM compiler. Native package Clippy and formatting passed. INV-10 and INV-11 are now ready.

INV-04's generated helper laws exposed two defects. With owner approval, `merge_unique_by` now removes duplicates from both inputs in first-seen order; `escape_ntriples_literal` emits `\uXXXX` for previously raw control characters. The saved merge regression remains under `crates/acorn-core/proptest-regressions/util/alloc/tests.txt`. All 30 focused tests passed at 4,096 generated cases, strict package Clippy passed, and the full `make check-wasm` matrix passed on November 26.

November 26 review at `3eddc62b` plus the current worktree. No tests were run for this review.

- `proptest = "1.9.0"` is a dev-dependency of `acorn-core`, `acorn-lib`, `acorn-schema`, and `acorn-diff` only. It appears in 18 test files, each with one `proptest!` block. Most blocks set 64 or 256 cases and a `FileFailurePersistence::Direct` path under `tests/proptest-regressions/`.
- **Laws already covered by generated inputs:**
  - similarity: differential edit distance and Jaro, symmetry, and the triangle inequality;
  - `Location` serde and Display round trips, and idempotent normalization;
  - base32 Crockford round trip;
  - OCI reference and remote parse/display idempotence;
  - `ResearchActivity` adapter laws;
  - candidate merge replay;
  - `ResearchOutput` merge idempotence;
  - `OneOrMany` codecs;
  - Handle, RRID, and SWHID canonicalization;
  - `Memory`, CFF, DataCite, ZON, and modelkit round trips;
  - DataCite↔DCAT crosswalk and identity-rule round trips.
- JSON patch already applies every generated `before.diff(&after)` patch through the independent `json_patch` crate and asserts the result equals `after` (`crates/acorn-diff/src/json_patch/tests.rs:67`). This plan adds no apply property there.
- `acorn-store` has no tests in its own crate. `crates/acorn-daemon/src/tests/mod.rs` exercises it about 80 times with fixed examples, covering claims, concurrent claims, pause, reload, and history bounds. There is no state-machine model.
- `Rational` (`crates/acorn-core/src/util/math/mod.rs:22`) uses `i128` with checked arithmetic and underpins `big_o::compare`. `crates/acorn-core/src/util/math/tests.rs` has five fixed-example tests and no property tests.
- Big-O has a parser and LaTeX output but no notation printer, so a parse/print round trip is impossible. `crates/acorn-core/tests/big_o.rs` lists its equivalence and transitivity cases by hand. `Dominance` is documented as a "Proven relationship" and reaches users through the CLI and WASM (`compare_big_o`, `compare_big_o_list`).
- `codec::{decode_value, encode_value}` (`crates/acorn-schema/src/codec.rs:240`) support JSON, JSONC, YAML, and ZON. Generated tests now call them directly for JSON, JSONC, and YAML; ZON remains owned by ZON-06.
- In `crates/acorn-core/src/util/alloc/mod.rs`, `split_list`, `merge_unique`, `merge_unique_by`, `redact_sensitive_text`, and `expand_brace_glob` are not named in any test file.
- CKAN, Invenio, Huwise, and OCP each have two to five fixed-example tests in `crates/acorn-schema/src/standard/tests/`. None has a generated round trip.
- `fuzz/` (`acorn-fuzz`, its own workspace) has three targets: `location_repository`, `powerpoint`, and `research_activity`. CI job `fuzz-smoke` runs each for 5 seconds.
- No Lean project, toolchain, or Make target exists. The October 27 portfolio review named Creusot for PID-06; the current PID plan replaced it with Verus.

# Project code boundaries

- **Update:** the test modules named in each phase, `fuzz/Cargo.toml`, `Makefile`, `.gitlab-ci.yml`, and `docs/src/learn/developer.md`.
- **Create:** `proofs/lean/` (a Lake project outside the Cargo workspace), new fuzz targets, the new test files named by INV-10 and INV-11, and `tests/fixtures/big_o/lean-vectors.json`.
- **Delete:** nothing. Fixed-example tests stay as readable regression anchors next to their generated counterparts.
- Production source changes are out of scope except the completed, owner-approved workspace `serde_json/float_roundtrip` dependency feature and INV-04 helper repair, and INV-11's planned single `#[cfg(test)] mod tests;` line in `crates/acorn-store/src/lib.rs`.

  A property failure that exposes a product defect becomes a repair phase in the plan that owns that code.

# Priority and parallel execution

Phases are listed in priority order. Their IDs are stable and do not encode order.

| Tier | Phases | What they touch | Runs alongside other plans |
|---|---|---|---|
| 1. Test-only, ready | INV-05–08 | Test files only, in crates that already depend on proptest | Yes. INV-08 shares one test file with low-priority Cosmos and Swift SLM phases and takes precedence over them. |
| 1. Test-only, blocked | INV-13 | `acorn-diff` unified tests | Only after DIF-05 and DIF-08, which own the same file |
| 2. Test-only, ready | INV-10 and INV-11 | New test files | Parallel-safe |
| 3. Tooling and proofs | INV-12, INV-14–18 | Fuzz package, `Makefile`, CI, `proofs/lean/` | Serialize on the CI/build lock or the Lean proof project |
| 4. Retirement | INV-20 | Docs and wiki | Last |

Parallel-safety evidence for tier 1, checked November 26 against every `.wiki` plan's write set:

- Cosmos and Swift SLM phases also list `crates/acorn-lib/src/io/tests/mod.rs`. Both plans are low priority, so INV-08 keeps the file and goes first; their write sets will be reconciled later.
- `crates/acorn-cmd/tests/execution.rs` belongs to coverage throughput, and browser format parity and browser performance own `crates/acorn-web/src/operations.rs`. INV-10 therefore adds new integration test files for both crates.
- Sponsor highlights owns `crates/acorn-schema/src/research_activity/tests/mod.rs`. That is a different file from INV-07's `standard/crosswalk/research_activity/tests/mod.rs`.
- Browser format parity edits `crates/acorn-schema/src/codec.rs`; rerun the completed codec property tests when it lands.
- PID-17 may now use the completed `Rational` properties in `crates/acorn-core/src/util/math/tests.rs`.

Each tier-1 phase runs only its focused tests. Workspace-wide `make lint` and `make test` stay serialized through one agent, as `AGENTS.md` requires.

# Dependencies

- [ZON codec hardening](../refactors/zon-codec-hardening.md) ZON-06 owns ZON property coverage. Add ZON to the completed codec generator after ZON-06 is removed.
- [Portable diff hardening](../refactors/acorn-diff-resource-and-api-hardening.md) owns `crates/acorn-diff/src/unified/tests.rs` through DIF-05 and DIF-08. INV-13 waits for both.
- [PID verification](pid-verification.md) PID-14 claims the CI/build lock first. INV-12 and INV-14 claim it afterwards and serialize with PID-05 and PID-11.
- The workspace manifest and lockfile lock is released after INV-09.
- The Verus proof of `Rational` is [PID-17](pid-verification.md#pid-17--prove-the-rational-arithmetic-core), because it changes production code. Its property-test prerequisite is satisfied.

# Decisions

- Each property test asserts one named law. The test name states the law: round trip, idempotence, symmetry, associativity, identity, order preservation, totality, or agreement with an oracle.
- Use the existing proptest configuration: 64 cases for whole-document strategies, 256 for scalar and string strategies, and a `FileFailurePersistence::Direct` path under the crate's `tests/proptest-regressions/`. Commit any regression file a failure produces.
- Generators include the empty string, Unicode outside the BMP, CRLF, and boundary integers, unless a format's documented domain excludes them. Every exclusion is a commented strategy filter, never a silent skip.
- Lean models the algorithm; it does not translate the Rust. The Lean model is executable, and a vector file it emits is replayed by a Rust test. Documentation must say that the proof covers the model, and that agreement on the vectors is what connects the model to the Rust code. Aeneas translation is out of scope until it supports `alloc` collections and `String`.
- The Lean project pins `lean-toolchain` and a Mathlib revision in `lake-manifest.json`, lives outside the Cargo workspace, and is never required by `cargo build`, `cargo test`, or `make check-wasm`.
- Lean proofs use no `sorry`, `admit`, or new `axiom` for claimed results. Any remaining trusted assumption is recorded explicitly.
- If INV-14 cannot build a trivial Mathlib-dependent module in CI within its phase, record the failure and stop INV-15 to INV-18 for replanning. Do not replace the proof with an unproved model.
- ZON grammar proofs and crosswalk lens-law proofs in Lean are deferred. Reconsider them only after INV-16 shows how much a proof phase actually costs. They are not phases in this plan.
- Do not reintroduce Creusot. Verus remains the only Rust verifier.

# Implementation sequence

1. Tier 1: add the remaining property tests in crates that already have proptest (Big-O laws, `util/alloc`, metadata standards, crosswalks, and `acorn-lib` round trips). Add unified-diff apply checks once the diff plan releases its test file.
2. Tier 2: add proptest to the remaining crates, then add wire round trips and the `acorn-store` state-machine model.
3. Tier 3: add fuzz targets for the codec and Big-O parsers. Set up Lean, model Big-O, prove its verdicts sound, and link the model to Rust through generated vectors.
4. Tier 4: publish guidance and retire the plan.

# Validation

- Each phase runs its focused `cargo nextest run` filter once with the default case count and once with `PROPTEST_CASES=4096`.
- Phases that touch manifests also run `cargo check --workspace` and `make check-wasm`.
- Lean phases run `lake build` from `proofs/lean/` and `make lean`, and must find no `sorry`, `admit`, or project `axiom`.
- The final phase runs `make lint`, `make test`, `make check-wasm`, `make lean`, and wiki validation.

# Acceptance criteria

- Every law listed under the remaining INV-05–08, INV-10, INV-11, and INV-13 phases has a generated test that passes at 4096 cases.
- Every generated test keeps a regression-file path.
- `acorn-cmd`, `acorn-service`, `acorn-store`, and `acorn-web` have proptest as a dev-dependency only. No new production dependency is introduced; the owner-approved `serde_json/float_roundtrip` feature remains enabled for correct finite-float decoding.
- `fuzz-smoke` runs the new codec and Big-O parse targets alongside the three existing targets.
- In Lean, for the modeled fragment, `Left`, `Right`, and `Equivalent` verdicts are proved sound with respect to Mathlib's `IsLittleO` and `IsTheta` at infinity. `Incomparable` is proved for the documented multivariate semantics, or the documentation is narrowed to what was proved.
- `crates/acorn-core/tests/big_o_lean.rs` replays the Lean-generated vectors and agrees on every vector.
- `docs/src/learn/developer.md` explains how to run each tool, what each proof covers, and what it does not.

# Execution phases

## INV-05 — Round-trip CKAN and Invenio documents

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-schema/src/standard/tests/ckan.rs`, `crates/acorn-schema/src/standard/tests/invenio.rs`
- Primary integration lock: Schema roots (tests only; no root edits)
- Outcome: Generate valid records for each standard. Test that every supported format decodes back to an equal value after encoding, and that validation results are unchanged by the round trip.
- Focused validation: `cargo nextest run -p acorn-schema standard::tests::ckan standard::tests::invenio`
- Handoff: Remove INV-05.

## INV-06 — Round-trip Huwise and OCP documents

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-schema/src/standard/tests/huwise.rs`, `crates/acorn-schema/src/standard/tests/ocp.rs`
- Primary integration lock: Schema roots (tests only; no root edits)
- Outcome: Same laws as INV-05 for Huwise and OCP.
- Focused validation: `cargo nextest run -p acorn-schema standard::tests::huwise standard::tests::ocp`
- Handoff: Remove INV-06.

## INV-07 — Test crosswalk round trips on supported fields

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-schema/src/standard/crosswalk/research_activity/tests/mod.rs`, `crates/acorn-schema/src/standard/crosswalk/tests/dublin_core.rs`
- Primary integration lock: Schema roots (tests only; no root edits)
- Outcome: For every crosswalk pair without a generated test, starting from `ResearchActivity` and Dublin Core, test that `A→B→A` preserves the documented supported fields. Every other field must appear in the loss report and must never be dropped without being reported.
- Focused validation: `cargo nextest run -p acorn-schema crosswalk`
- Handoff: Remove INV-07 and list the pairs covered.

## INV-08 — Use generated inputs for `acorn-lib` fixed-example round trips

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-lib/src/io/tests/mod.rs`, `crates/acorn-lib/src/io/config/tests/ron.rs`
- Primary integration lock: none (test leaf)
- Note: Cosmos (`azure-cosmos-buckets`) and Swift SLM (`swift-slm-profile`) phases also list `crates/acorn-lib/src/io/tests/mod.rs`. INV-08 has priority over them; their plans will be reconciled later, after INV-08 lands.
- Outcome:
  - Generated file trees in a temporary directory survive `archive` then `extract` for every supported archive format.
  - RON serialization is canonical and idempotent over generated configurations.
  - `Executor` serde round trips over generated values.
  - `Fingerprint::from_entries` ignores entry order and changes when any entry changes.
- Focused validation: `cargo nextest run -p acorn-lib io::tests io::config::tests::ron`
- Handoff: Remove INV-08.

## INV-13 — Check unified diffs with an apply oracle

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: DIF-05 and DIF-08 removed from the diff plan
- Exact write set: `crates/acorn-diff/src/unified/tests.rs`
- Primary integration lock: none (test leaf)
- Outcome: For generated multi-line text, including CRLF, a missing final newline, and Unicode, test that:
  - comparing a text with itself produces no diff;
  - applying `unified_diff(before, after)` with an independent unified-patch oracle yields `after` byte for byte;
  - hunk headers match the line counts.

  The oracle is a test helper written in the test file. If an external dependency is needed instead, open an INV-09-style manifest phase rather than editing manifests here.
- Focused validation: `cargo nextest run -p acorn-diff unified`
- Handoff: Remove INV-13.

## INV-10 — Round-trip service, command, and web wire types

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-service/src/job/tests.rs`, `crates/acorn-cmd/tests/properties.rs` (new), `crates/acorn-web/tests/operations.rs` (new)
- Primary integration lock: none (test leaf)
- Note: `crates/acorn-cmd/tests/execution.rs` and `crates/acorn-web/src/operations.rs` stay untouched because coverage throughput and the browser plans own them. Both new files are integration tests that use only public API.
- Outcome:
  - Generated `JobId` values within the boundary rules round trip, and values outside them are rejected.
  - `command_string` followed by `parse_sh` returns the original program and arguments for generated arguments containing quotes, backslashes, empty strings, and Unicode.
  - Generated web operation messages and worker replies round trip.
- Focused validation: `cargo nextest run -p acorn-service job`, `cargo nextest run -p acorn-cmd --test properties`, `cargo nextest run -p acorn-web --test operations`
- Handoff: Remove INV-10.

## INV-11 — Check `acorn-store` against a state-machine model

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-store/src/tests.rs`, `crates/acorn-store/src/lib.rs` (the `#[cfg(test)] mod tests;` line only)
- Primary integration lock: none (test leaf)
- Outcome: Run generated sequences of `register_jobs`, `activate_jobs`, `enqueue`, `set_paused`, `claim_next`, and `finish` against a temporary database and an in-memory model. Check that:
  - no run is claimed twice;
  - paused jobs are never claimed;
  - `history` respects its limit and returns the model's order;
  - `run` returns what `finish` recorded.
- Focused validation: `cargo nextest run -p acorn-store`
- Handoff: Remove INV-11. Any divergence becomes a repair phase in the [acornd plan](../features/acornd-service.md).

## INV-12 — Add codec and Big-O parse fuzz targets

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-14 has released the CI/build lock
- Exact write set: `fuzz/Cargo.toml`, `fuzz/fuzz_targets/codec_value.rs`, `fuzz/fuzz_targets/big_o.rs`, `fuzz/corpus/codec_value/`, `fuzz/corpus/big_o/`, `.gitlab-ci.yml`
- Primary integration lock: CI build entry points
- Outcome:
  - `codec_value` decodes arbitrary UTF-8 input in every supported format. Every value that decodes is re-encoded and decoded again, and the second decode must equal the first.
  - `big_o` parses arbitrary input, and every successful parse is compared with itself and must give `Equivalent`.

  Both targets join `fuzz-smoke` with the same time and memory limits as the existing targets.
- Focused validation: `cargo +nightly fuzz run --fuzz-dir fuzz <target> -- -max_total_time=60` for each target, plus CI lint.
- Handoff: Remove INV-12 and commit any minimized crash as a regression test in the owning crate.

## INV-14 — Set up the pinned Lean toolchain and workflow

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PID-14 has released the CI/build lock
- Exact write set: `proofs/lean/lean-toolchain`, `proofs/lean/lakefile.lean`, `proofs/lean/lake-manifest.json`, `proofs/lean/Acorn.lean`, `Makefile`, `.gitlab-ci.yml`, `docs/src/learn/developer.md`
- Primary integration lock: CI build entry points
- Outcome:
  - Pin a Lean 4 release and a Mathlib revision.
  - Build a trivial module that imports `Mathlib.Analysis.Asymptotics.Asymptotics`.
  - Add `make lean` and a path-filtered informational CI job that uses the Mathlib build cache and fails on `sorry`, `admit`, or a project `axiom`.

  Ordinary Cargo builds stay unchanged.
- Focused validation: `make lean` locally, `make help` lists the target, and one CI run each on a matching and a non-matching change.
- Handoff: Remove INV-14 after recording the pinned pair and build time. If Mathlib cannot be built or cached in CI, record the failure and stop INV-15 to INV-18.

## INV-15 — Model Big-O normal forms in Lean

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: INV-14
- Exact write set: `proofs/lean/Acorn/BigO/Model.lean`, `proofs/lean/Acorn.lean`
- Primary integration lock: Lean proof project
- Outcome: Define an executable Lean model of the normalized form that `compare.rs` produces:
  - monomials over named symbols, with rational powers of each symbol and of its logarithm;
  - `SymbolGrowth` assumptions;
  - `compare_terms`, `provably_strict`, and `maximum_total_power`.

  Give each monomial a denotation as a function `ℝ → ℝ` for one growing symbol, and as a function on `ℝ^k` for several symbols. The model mirrors the Rust structure so vectors can be compared.
- Focused validation: `lake build`, and `#eval` agreement on every fixed example in `crates/acorn-core/tests/big_o.rs`.
- Handoff: Remove INV-15 and record any Rust behavior that the model could not mirror.

## INV-16 — Prove single-symbol verdicts sound

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: INV-15
- Exact write set: `proofs/lean/Acorn/BigO/Soundness.lean`, `proofs/lean/Acorn.lean`
- Primary integration lock: Lean proof project
- Outcome: For sums of monomials in one growing symbol, prove that a `Right` verdict implies the left denotation is `IsLittleO` of the right at infinity, that `Left` implies the reverse, and that `Equivalent` implies `IsTheta`. Use no `sorry`, `admit`, or project `axiom`.
- Focused validation: `make lean`
- Handoff: Remove INV-16 and record the proof's active hours to decide on the deferred ZON and crosswalk proofs.

## INV-17 — Prove multivariate and `Incomparable` verdicts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: INV-16
- Exact write set: `proofs/lean/Acorn/BigO/Multivariate.lean`, `proofs/lean/Acorn.lean`, `docs/src/learn/developer.md`
- Primary integration lock: Lean proof project
- Outcome: State the filter that multivariate growth uses: every growing symbol tends to infinity, so the filter is the product of `atTop` filters. Extend the soundness proof to it, and prove that `Incomparable` means neither side is `IsBigO` of the other. If the current Rust semantics cannot be stated as a filter, narrow the documented meaning of `Incomparable` to what was proved, and record the gap.
- Focused validation: `make lean`
- Handoff: Remove INV-17.

## INV-18 — Connect the Lean model to Rust with generated vectors

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: INV-15
- Exact write set: `proofs/lean/Acorn/BigO/Vectors.lean`, `tests/fixtures/big_o/lean-vectors.json`, `crates/acorn-core/tests/big_o_lean.rs`
- Primary integration lock: Lean proof project
- Outcome: Lean enumerates bounded expression pairs and writes their verdicts to a JSON file, which is committed. A Rust test parses each pair, runs `compare_growth`, and requires the same verdict. `make lean` checks that the committed file is current.
- Focused validation: `make lean` and `cargo nextest run -p acorn-core --test big_o_lean`
- Handoff: Remove INV-18. A disagreement is a defect in either the model or the Rust code. Open a repair phase and do not update the vectors to hide it.

## INV-20 — Publish guidance and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: every other INV phase removed
- Exact write set: `docs/src/learn/developer.md`, `ARCHITECTURE.md`, `.wiki/index.md`, `.wiki/testing/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/testing/invariant-verification.md`
- Primary integration lock: verification documentation
- Outcome: Document how to name property tests, how to choose generators, the regression-file convention, the fuzz targets, and what the Lean proofs cover. Run the full gates, then delete this plan and every reference to it.
- Focused validation: `make lint`, `make test`, `make check-wasm`, `make lean`, and wiki validation.
- Handoff: Delete this plan and every index, network, and lane entry.
