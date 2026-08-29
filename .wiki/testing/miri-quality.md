---
type: Plan
title: "Add informational Miri quality checks"
description: "Run a curated Miri integration suite over acorn-lib's portable public APIs, including the refactored PID module boundaries."
difficulty: medium
impact: medium
package: [acorn-lib]
tags: [testing, miri, undefined-behavior, ci, rust, pid]
status: planned
---

# Project code boundaries

- **Update:** portable tests under `crates/acorn-lib/src/schema/pid/`, `Makefile`, and `.gitlab-ci.yml`; later the owning extracted crate.
- **Create:** `crates/acorn-lib/tests/miri.rs` or an equivalent curated integration target only if inline public-API tests cannot be selected reliably.
- **Delete:** unsupported std/network/process cases from the Miri target, not from the native suite.

# Dependencies

Track module moves into `acorn-core`/`acorn-schema`; share vectors with [Creusot](creusot-pid-checksums.md) without coupling either tool to production code.

# Goal

Introduce a focused Miri suite for `acorn-lib` to detect undefined behavior in exercised code and dependencies. Keep the initial suite deterministic, in-memory, portable, and informational in GitLab CI. Exercise checksum-bearing PIDs through their public trait implementations without exposing private helpers introduced by the PID refactor.

# Current evidence

## Portable test boundary (verified 2026-08-23 against `acorn-lib` `0.1.75`)

- The workspace forbids unsafe code by default, while `acorn-tui` explicitly allows it. Miri still adds value by checking unsafe behavior reached through dependencies and validating allocation, aliasing, and interpreter-sensitive assumptions.
- Stable Rust is pinned to `1.95.0`; GitLab CI currently defines `NIGHTLY_TOOLCHAIN: nightly-2026-08-01` for fuzzing. No Miri target, Makefile command, CI job, or `crates/acorn-lib/tests/` directory exists yet.
- `cargo check -p acorn-lib --no-default-features --features alloc` passes, confirming that the library's portable dependency surface builds.
- `cargo test -p acorn-lib --lib --no-run --no-default-features --features alloc` currently fails because internal `cfg(test)` modules import `std`-gated I/O, paths, Tokio, macros, and test-only dependencies. The failure persists after the PID refactor; adding imports to every internal suite is not part of this plan.
- A dedicated integration target compiles `acorn-lib` as a dependency, does not enable its internal `cfg(test)` modules, and remains the correct isolation boundary for Miri.
- Miri is an interpreter and is substantially slower than native tests. A curated suite gives a predictable initial signal while leaving host-dependent coverage to native tests and fuzzing. See the [official Miri documentation](https://github.com/rust-lang/miri).

## PID state after the refactor

- Shared PID traits and free functions live in `crates/acorn-lib/src/schema/pid/mod.rs`.
- Concrete implementations and their native tests now live in paired `pid/<kind>/mod.rs` and `pid/<kind>/tests.rs` modules.
- Checksum-bearing public boundaries are `ARK`, `ISBN`, `ISNI`, `ORCID`, and `ROR` through `PersistentIdentifier::check_digit` and `PersistentIdentifierParse::is_valid`; `noid_check_digit` is also public.
- `mod_10_or_11_check_digit`, `mod_11_2_check_digit`, and `mod_97_10_check_digit` are intentionally private shared helpers. An integration suite cannot and should not call them directly.
- Direct shared-helper vectors live in `crates/acorn-lib/src/schema/pid/tests.rs`; adjacent PID suites cover parsing, formatting, invalid inputs, and trait contracts. Miri should reuse representative values through public APIs rather than duplicate the complete native matrix.

## Related work

- [Creusot verification for PID checksum calculations](creusot-pid-checksums.md) proves arithmetic properties of the private shared cores. Miri dynamically checks reachable behavior through public wrappers and dependencies.
- Existing fuzz targets cover `Location`/`Repository` and `ResearchActivity`; their pure, deterministic operations provide the starting Miri scenarios.

# Recommendation

## Curated integration target

Add `crates/acorn-lib/tests/miri.rs` with `#![cfg(miri)]` and run it using `--no-default-features --features alloc`. Normal test runs may compile an empty integration target, while Miri receives only deterministic test cases.

Cover these public behaviors:

1. Parse and inspect local and remote `Location` and `Repository` values, including URI and Windows-path cases already represented by `fuzz/fuzz_targets/location_repository.rs`.
2. Exercise repository cloning, hashing, identifiers, locality checks, and raw URL generation without filesystem or network access.
3. Exercise checksum-bearing PID boundaries:
   - ARK/NOID: call public `noid_check_digit`, parse an ARK with a NOID check character, and compare `ARK::check_digit`.
   - ISBN: cover ISBN-10 `X`, ISBN-13 decimal output, invalid length, and invalid payload through `ISBN::{from_string,is_valid}` and `check_digit`.
   - ISNI and ORCID: cover decimal and `X` MOD 11-2 results plus invalid identifiers through their public types.
   - ROR: cover valid MOD 97-10 output and invalid Crockford payload through `ROR` public methods.
4. Add one lightweight parse/format/trait-contract case for each non-checksum PID module only if it stays deterministic and materially exercises a distinct allocation or dependency path. Do not reproduce every adjacent native test under Miri.
5. Deserialize an embedded `ResearchActivity` JSON fixture, validate it, format it to prose, serialize it, and deserialize it again without runtime file access.

Keep every Miri test free of runtime filesystem, environment, network, subprocess, FFI, thread scheduling, and wall-clock dependencies. Leave Miri isolation, aliasing checks, and leak checks enabled. Do not add broad `cfg_attr(miri, ignore)` annotations or disable checks to make the suite pass.

## Developer command

Add a `make miri` target that runs:

```text
cargo +$(NIGHTLY_TOOLCHAIN) miri test -p acorn-lib --test miri --no-default-features --features alloc
```

Default `NIGHTLY_TOOLCHAIN` to `nightly` for local development. Allow CI to supply a separately validated pinned nightly. Document installation, `cargo miri setup`, local execution, and an optional deeper run with `MIRIFLAGS="-Zmiri-many-seeds=0..16"` in `CONTRIBUTING.md`.

Before invoking Miri, retain `cargo check -p acorn-lib --no-default-features --features alloc` as the portable compile baseline. Do not use the failing `--lib` unit-test command as the Miri entry point.

## Informational GitLab job

Add a `miri` job to the test stage with these policies:

- Pin a nightly known to provide a working Miri component for the current dependency graph. Reuse `nightly-2026-08-01` only after validation; do not assume the fuzz pin is automatically Miri-compatible.
- Install the pinned toolchain with the Miri component and run `cargo miri setup`.
- Run `make miri` on the existing Rust runner and shared test resource group with an initial 20-minute timeout.
- Trigger for merge requests and default-branch changes affecting `crates/acorn-lib/src/**`, `crates/acorn-lib/tests/miri.rs`, Cargo manifests or lockfile, `Makefile`, or GitLab CI configuration.
- Ensure changes under `crates/acorn-lib/src/schema/pid/**` trigger the job, including adjacent PID modules/tests and shared `pid/mod.rs` changes.
- Set `allow_failure: true` while Miri remains informational.

# Maintenance policy

- Update the Miri nightly deliberately alongside, but not necessarily identically to, the fuzz and Creusot pins.
- Add deterministic Miri cases when portable allocation, parsing, identifier, serialization, or concurrency-sensitive behavior is introduced.
- For PID changes, add or adjust Miri coverage at the public PID boundary; keep exhaustive case tables in `pid/tests.rs` and each `pid/<kind>/tests.rs`.
- Classify failures as undefined behavior, unsupported host behavior, dependency regression, or nightly/Miri regression.
- Fix confirmed undefined behavior and add a regression case. For upstream limitations, use the narrowest documented exclusion and link the upstream issue; never disable a class of Miri checks globally.
- Keep Miri complementary to Clippy, native tests, coverage, fuzzing, and Creusot rather than treating it as a replacement.

# Validation

1. `cargo check -p acorn-lib --no-default-features --features alloc` passes before and after adding the integration target.
2. Install the pinned nightly and Miri component, run `cargo miri setup`, and confirm `make miri NIGHTLY_TOOLCHAIN=<pinned-nightly>` passes with isolation, aliasing, and leak checks enabled.
3. Run one local multi-seed pass over `0..16` before the initial merge.
4. Run `cargo test -p acorn-lib schema::pid`, `make lint`, and `make test` to confirm the dedicated target does not change normal PID or workspace behavior.
5. Verify a PID-only merge request starts the GitLab Miri job, exposes logs, and does not block the pipeline on failure; verify unrelated documentation-only changes do not start it.

# Acceptance criteria

1. The curated `acorn-lib` integration suite passes under the pinned Miri toolchain with no blanket disable flags and no runtime host access.
2. `make miri` is the documented local interface; ordinary `make test` behavior and the refactored per-PID native suites remain unchanged.
3. The informational GitLab job appears on relevant merge-request and default-branch pipelines and completes within its timeout.
4. The suite covers `Location`, `Repository`, `ResearchActivity`, and all five checksum-bearing PID public boundaries: ARK/NOID, ISBN, ISNI, ORCID, and ROR.
5. Private shared PID check-digit helpers remain private; Miri reaches them through public trait implementations while Creusot covers their direct arithmetic contracts.
6. Maintenance and failure-triage expectations are documented in `CONTRIBUTING.md`.

# Risks and tradeoffs

- **Internal unit tests remain incompatible with `alloc`-only builds:** use the dedicated integration target rather than broad test-module gating changes.
- **Private checksum helpers are not directly callable:** public PID wrappers provide realistic reachability; Creusot supplies direct core proofs.
- **Interpreter cost:** keep the initial matrix representative and use multi-seed runs locally before expanding CI workload.
- **Dependency support:** isolate or narrowly exclude only confirmed unsupported dependency behavior, with an upstream reference.
- **Nightly drift:** validate the pinned Miri component whenever Rust or dependencies change.
