---
type: Plan
title: "Add informational Miri quality checks"
description: "Use a curated, isolated Miri suite to detect undefined behavior in acorn-lib and exercised dependencies."
difficulty: medium
impact: medium
package: [acorn-lib]
tags: [testing, miri, undefined-behavior, ci, rust]
status: planned
---

# Goal

Introduce a focused Miri suite for `acorn-lib` to detect undefined behavior in exercised code and dependencies. Keep the initial suite deterministic, in-memory, and informational in GitLab CI.

# Current evidence

- The workspace forbids unsafe code by default, while `acorn-tui` explicitly allows it. Miri therefore adds value by checking unsafe behavior reached through dependencies and validating allocation, aliasing, and concurrency assumptions.
- The repository uses stable Rust for normal development and pins `nightly-2026-08-01` in GitLab CI for fuzzing. Miri is distributed as a nightly Rust component.
- Existing unit tests mix pure logic with filesystem, networking, subprocess, database, and FFI behavior. Miri documents limited support for host operations and recommends `cargo miri test` for compatible test suites.
- The `acorn-lib` unit-test target does not currently compile with only the `alloc` feature because `cfg(test)` modules import `std`-only APIs. A dedicated integration target compiles `acorn-lib` as a dependency and avoids those internal test modules.
- Miri is a single-threaded interpreter and can be substantially slower than native tests, so a curated suite provides a more predictable initial CI signal than retrofitting the full test suite.

See the [official Miri documentation](https://github.com/rust-lang/miri) for installation, supported checks, isolation behavior, CI setup, and execution-model limitations.

# Recommendation

## Curated integration suite

Add `crates/acorn-lib/tests/miri.rs` with `#![cfg(miri)]`. Run it with `--no-default-features --features alloc` so normal test runs remain unchanged and Miri exercises the portable library surface without enabling network, subprocess, database, TUI, or FFI-heavy features.

Cover representative public behavior already exercised by the fuzz targets:

1. Parse and inspect local and remote `Location` and `Repository` values, including URI and Windows-path cases.
2. Exercise repository cloning, hashing, identifiers, locality checks, and raw URL generation.
3. Verify valid and invalid PID/check-digit inputs for ORCID, ISNI, ROR, ISBN, and NOID helpers.
4. Deserialize an embedded `ResearchActivity` JSON fixture, validate it, format it to prose, serialize it, and deserialize it again.

Keep every Miri test free of runtime filesystem, environment, network, subprocess, FFI, and wall-clock dependencies. Leave Miri isolation, aliasing checks, and leak checks enabled. Do not add broad `cfg_attr(miri, ignore)` annotations or disable checks to make the suite pass.

## Developer command

Add a `make miri` target that runs:

```text
cargo +$(NIGHTLY_TOOLCHAIN) miri test -p acorn-lib --test miri --no-default-features --features alloc
```

Default `NIGHTLY_TOOLCHAIN` to `nightly` for local development. Allow CI to supply its existing pinned nightly value. Document installation, `cargo miri setup`, local execution, and an optional deeper run with `MIRIFLAGS="-Zmiri-many-seeds=0..16"` in `CONTRIBUTING.md`.

## Informational GitLab job

Add a `miri` job to the test stage with these policies:

- Reuse `NIGHTLY_TOOLCHAIN: nightly-2026-08-01`.
- Install the pinned toolchain with the `miri` component and run `cargo miri setup`.
- Run `make miri` on the existing Rust runner and shared test resource group with a 20-minute timeout.
- Trigger for merge requests and default-branch changes that affect `acorn-lib`, Cargo manifests or lockfile, the Makefile, or GitLab CI configuration.
- Set `allow_failure: true` so Miri remains informational while the suite matures.

# Maintenance policy

- Update the pinned nightly alongside the existing fuzz nightly pin and validate Miri before merging the update.
- Add deterministic Miri cases when pure allocation, parsing, identifier, serialization, or concurrency-sensitive behavior is introduced.
- Classify failures as undefined behavior, unsupported host behavior, dependency regression, or nightly/Miri regression.
- Fix confirmed undefined behavior and add a regression case. For confirmed upstream limitations, use the narrowest documented exclusion and link the upstream issue; never disable a class of Miri checks globally.
- Keep Miri complementary to Clippy, native tests, coverage, and fuzzing rather than treating it as a replacement.

# Validation

1. Install the pinned nightly and Miri component, run `cargo miri setup`, and confirm `make miri NIGHTLY_TOOLCHAIN=nightly-2026-08-01` passes with isolation enabled.
2. Run one local multi-seed pass over `0..16` before the initial merge.
3. Run `make lint` and the existing Rust test workflow to confirm the dedicated target does not affect normal tests.
4. Verify a qualifying merge request starts the GitLab Miri job, exposes its logs, and does not block the pipeline on failure.
5. Verify unrelated changes do not start the job.

# Acceptance criteria

1. The curated `acorn-lib` suite passes under the pinned Miri toolchain with no blanket disable flags.
2. `make miri` is the documented local interface and ordinary `make test` behavior is unchanged.
3. The informational GitLab job appears on relevant merge-request and default-branch pipelines and completes within its timeout.
4. The suite covers `Location`, `Repository`, PID helpers, and `ResearchActivity` validation and serialization without runtime host access.
5. Maintenance and failure-triage expectations are documented in `CONTRIBUTING.md`.

