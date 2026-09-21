---
type: Plan
title: "Add informational Miri quality checks"
description: "Run a curated Miri integration suite over portable acorn-core and acorn-schema public APIs."
difficulty: medium
impact: medium
package: [acorn-core, acorn-schema]
tags: [testing, miri, undefined-behavior, ci, rust, pid]
status: planned
---

# Goal

Introduce a deterministic, in-memory Miri suite for the portable core and schema APIs. Exercise checksum-bearing PIDs through their public traits without exposing private arithmetic helpers, and keep host I/O, networking, processes, FFI, scheduling, and wall-clock behavior outside the target.

# Current evidence

- `acorn-core` and `acorn-schema` compile with `--no-default-features --features alloc`; their implementation is independent of the host-heavy `acorn-lib` test graph.
- Shared PID traits, helpers, and tests live under `crates/acorn-schema/src/pid/`. `ARK`, `ISBN`, `ISNI`, `ORCID`, and `ROR` expose checksum behavior through public trait implementations, while three arithmetic helpers remain private.
- `Location`, `Repository`, `ResearchActivity`, validation, and portable codecs provide deterministic allocation and parsing scenarios already represented by native tests and fuzz targets.
- No Miri target, Makefile command, or CI job exists. The workspace pins a separate nightly for fuzzing, but that pin has not been validated for Miri.

# Project code boundaries

- **Update:** `Makefile`, `.gitlab-ci.yml`, `CONTRIBUTING.md`, and focused portable test configuration.
- **Create:** `crates/acorn-schema/tests/miri.rs` as the curated integration target.
- **Delete:** host-dependent cases from the Miri target, not from native suites.

# Dependencies

Share representative vectors with [Creusot checksum verification](creusot-pid-checksums.md) without coupling either tool to production code. Miri dynamically exercises public wrappers; Creusot proves private arithmetic cores.

# Recommendation

1. Add `crates/acorn-schema/tests/miri.rs` with `#![cfg(miri)]` and run it using `--no-default-features --features alloc`.
2. Cover `Location`/`Repository` parsing and identity, `ResearchActivity` deserialize/validate/serialize round trips, and representative valid/invalid cases for ARK/NOID, ISBN, ISNI, ORCID, and ROR.
3. Keep exhaustive vectors in native PID tests and call private helpers only through public PID traits.
4. Add `make miri` using a separately validated pinned nightly, then add an informational GitLab job with an initial 20-minute timeout and path rules for `acorn-core`, `acorn-schema`, manifests, the Makefile, and CI.
5. Document toolchain setup, `cargo miri setup`, a local multi-seed run, and failure classification. Do not disable isolation, aliasing, or leak checks globally.

# Validation

- `cargo check -p acorn-core --no-default-features --features alloc`
- `cargo check -p acorn-schema --no-default-features --features alloc`
- `cargo +<pinned-nightly> miri test -p acorn-schema --test miri --no-default-features --features alloc`
- One local `MIRIFLAGS="-Zmiri-many-seeds=0..16"` pass before initial merge.
- Focused native PID/schema tests, `make lint`, and `make test`.

# Acceptance criteria

- The curated target passes under the pinned Miri toolchain without host access or blanket disable flags.
- It covers `Location`, `Repository`, `ResearchActivity`, and all five checksum-bearing PID public boundaries.
- Private checksum helpers remain private and exhaustive vectors remain in native tests.
- `make miri` is the documented local interface, and an informational CI job runs only for relevant changes.
- Ordinary workspace tests and portable feature checks remain unchanged.
