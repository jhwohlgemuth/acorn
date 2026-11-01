---
type: Plan
title: "Unify development toolchains with mise"
description: "Adopt a locked mise tool inventory for local development and CI while keeping Make as ACORN's task interface and rust-toolchain.toml as the stable Rust contract."
difficulty: medium
impact: medium
package: [workspace]
tags: [migration, mise, tooling, make, ci, reproducibility, portability]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Use one reviewed, multiplatform tool inventory for ACORN development and CI without duplicating tasks in mise or replacing the Makefile as the authoritative workflow surface.

The migration must make local and CI tool selection reproducible, preserve the Rust components and targets declared in `rust-toolchain.toml`, and distinguish mise-managed developer executables from operating-system packages and release-host prerequisites.

# Current evidence

- Mise support is disabled: the repository has no `mise.toml` or `mise.lock`, and Make, CI, and contributor guidance contain no mise invocation.
- `Makefile` is the authoritative task catalog. Its `setup` target updates stable Rust, installs `DEV_DEPENDENCIES` with Cargo, and adds the cross-compilation targets in `TARGETS`.
- `rust-toolchain.toml` selects the rolling `stable` channel with the minimal profile, `rust-analyzer`, `rustfmt`, Clippy, and four release/portable targets.
- `DEV_DEPENDENCIES` mixes exact requests (`cargo-deb@3.8.0`, `cargo-shear@1.14.0`) with rolling Cargo installs for audit, coverage, fuzzing, snapshots, testing, documentation, SQL linting, and WebAssembly packaging.
- `Dockerfile.rust` separately pins Rust 1.98.0, nightly 2026-08-01, Buf 1.73.0, cargo-deb 3.8.0, cargo-fuzz 0.13.1, cargo-shear 1.14.0, mdBook 0.5.2, mdbook-mermaid 0.17.0, mdbook-pikchr 0.2.0, sqruff 0.40.0, and wasm-pack 0.15.0. Node/npm and ShellCheck come from unpinned distribution packages.
- `.gitlab-ci.yml` repeats several image pins or fallback installs for Buf, cargo-shear, mdBook, cargo-fuzz, nightly Rust, and wasm-pack.
- `CONTRIBUTING.md` names only a subset of the tools installed by `make setup` and separately describes Buf and cargo-insta installation.
- Mise recommends using the same project configuration in development and CI, committing a lockfile, and installing with `mise install --locked`.[^mise-ci]
- Mise can read the channel, profile, components, and targets from `rust-toolchain.toml`; the migration should preserve that file rather than create a second stable-Rust declaration.[^mise-rust]
- Mise lockfiles record exact top-level versions for Cargo-backed tools, while Cargo remains responsible for their dependency resolution. Review must therefore retain `--locked` Cargo installation semantics and not claim artifact-level locking that the backend does not provide.[^mise-lock]

# Project code boundaries

- **Create:** `mise.toml` and `mise.lock`.
- **Update:** `Makefile`, `Dockerfile.rust`, `.gitlab-ci.yml`, `CONTRIBUTING.md`, and focused developer setup documentation.
- **Preserve:** `rust-toolchain.toml` remains the stable Rust toolchain, component, profile, and target source. Make remains the task interface; do not add duplicate `[tasks]` workflows to `mise.toml`.
- **Delete:** No repository file. Remove direct installation commands only after the locked mise path provides and validates the same executable on every affected platform.
- **Exclude:** Product dependencies, runtime configuration, container-engine selection, release artifact formats, user-facing CLI syntax, and editor-specific configuration.

# Decisions

1. Mise owns version selection for cross-platform developer executables that it can install reproducibly; operating-system libraries, compilers, PowerShell, browser dependencies, container engines, and native release-host utilities remain explicit system prerequisites.
2. `mise.toml` declares tools and settings only. `Makefile` remains the sole task catalog and invokes mise-managed tools without requiring shell activation.
3. `mise.lock` is committed with entries for every supported developer and CI platform. CI uses locked mode and fails when a required entry is absent.
4. The mise executable itself is pinned independently in the CI image; an unpinned bootstrap script is not an accepted supply-chain boundary.
5. Stable Rust remains configured by `rust-toolchain.toml`. Nightly Rust uses one exact dated request shared by local fuzz/audit setup and CI.
6. Rolling requests such as `latest` or `lts` are acceptable in `mise.toml` only when `mise.lock` fixes the installed version and updates are explicit, reviewed lockfile changes.
7. Direct CI or Dockerfile installers remain until their mise-managed replacement passes on the corresponding image architecture. Removal and replacement land together per tool group.
8. Cargo-backed tools retain locked crate installation. The acceptance claim is consistent top-level tool versions, not complete transitive binary reproducibility.

# Dependencies and coordination

- Lane: Build and operations in the [parallel execution guide](../parallel-execution.md).
- MSE-01 is an isolated configuration/lockfile phase. MSE-02 through MSE-04 and MSE-07 take the CI/build lock and must serialize with [coverage throughput](../testing/coverage-throughput.md), [Podman Makefile migration](podman-makefile.md), [runner-pool isolation](../refactors/ci-runner-pools.md), PowerPoint visual-test CI phases, and any other owner of `Makefile`, `Dockerfile.rust`, or `.gitlab-ci.yml`.
- MSE-05 and MSE-06 are native-host acceptance phases. They may run independently after MSE-02 when they do not edit shared documentation concurrently.
- Native Windows development acceptance does not depend on the blocked GitLab shell runner. Any decision to require a Windows CI job must wait for [GSR-02](../testing/gitlab-shell-runner-readiness.md#gsr-02--prove-a-native-windows-shell-runner).
- Tool-version updates discovered during implementation are part of this plan only when required for parity. Product dependency upgrades remain separate work.

# Validation

## Configuration and locking

1. Validate `mise.toml` and generate lock entries for Linux AMD64, Linux ARM64, macOS ARM64, and Windows AMD64.
2. Run `mise install --locked` from an empty mise data directory on each available platform and require zero unlocked resolution.
3. Record `mise ls --current`, `rustc --version`, `cargo --version`, and every Make-invoked managed tool version.
4. Confirm `mise exec -- rustup show active-toolchain` reflects `rust-toolchain.toml` and includes its components and targets.

## Make and local workflow

1. Dry-run every affected Make target and confirm Make remains the only task catalog.
2. Run `make setup`, `make lint`, `make test`, `make book`, `make check-portable`, and one bounded fuzz smoke test from a shell without mise activation.
3. Confirm missing or stale locked tools fail with an actionable setup message rather than silently skipping lint or validation.
4. Verify existing target names, arguments, environment overrides, and generated output locations remain unchanged.

## CI and image workflow

1. Build the Rust CI image for Linux AMD64 and ARM64 with a pinned mise executable and `mise install --locked`.
2. Compare the installed tool manifest between both images and the committed lockfile.
3. Run the test, documentation, portability, fuzz-smoke, benchmark, and release-build job paths that consume affected Make targets.
4. Confirm CI contains no fallback installer or version pin for a mise-managed tool and performs no unlocked registry lookup.
5. Verify cache reuse does not weaken lockfile validation on a clean runner.

# Acceptance criteria

1. `mise.toml` and `mise.lock` are the reviewed version contract for every mise-managed developer executable.
2. Make remains the authoritative task interface; `mise.toml` contains no duplicate task definitions.
3. Stable Rust continues to derive its channel, profile, components, and targets from `rust-toolchain.toml`.
4. Local and CI fuzz/audit workflows use the same exact dated nightly toolchain.
5. Linux AMD64, Linux ARM64, macOS ARM64, and Windows AMD64 locked installs resolve the same top-level tool versions.
6. `make setup` and affected Make targets work without shell activation on supported developer platforms.
7. The Rust CI image installs a pinned mise release and installs project tools in locked mode.
8. `Dockerfile.rust` and `.gitlab-ci.yml` contain no duplicate installer or version request for a mise-managed tool.
9. System dependencies that remain outside mise are explicit in the Dockerfile or contributor documentation.
10. The affected test, documentation, portability, fuzz, benchmark, and release paths pass with the locked tool inventory.

# Completion handling

After every acceptance criterion passes, preserve the supported platform matrix, update procedure, system prerequisites, and troubleshooting guidance in durable developer documentation. Delete this plan and remove its migration-index, dependency-network, and parallel-lane entries in the same change.

# Execution phases

## MSE-01 — Freeze the tool inventory and lock contract

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `mise.toml`, `mise.lock`, `.wiki/migrations/mise-toolchain-integration.md`
- Primary integration lock: development tool version contract
- Outcome: Declare the mise-managed tool boundary, preserve `rust-toolchain.toml` ownership, pin the dated nightly policy, and commit complete lock entries for the supported platform matrix.
- Focused validation: Validate the config; inspect every locked tool/platform entry; perform a clean locked install on the implementation host; compare the resulting version manifest with current Make, Dockerfile, and CI requirements.
- Handoff: Remove MSE-01, preserve the accepted inventory and update policy as current evidence, and make MSE-02 ready.

## MSE-02 — Adopt locked mise tools behind Make

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: MSE-01
- Exact write set: `Makefile`, `CONTRIBUTING.md`, `.wiki/migrations/mise-toolchain-integration.md`
- Primary integration lock: CI/build (`Makefile`)
- Outcome: Make setup installs locked tools, Make recipes activate them without shell setup, and contributor guidance names mise as the supported bootstrap while retaining every existing task contract.
- Focused validation: Dry-run affected targets; run setup, lint, tests, book, portable checks, and bounded fuzzing from a non-activated shell; confirm `mise.toml` defines no tasks.
- Handoff: Remove MSE-02, record the local version manifest and command evidence, and make MSE-03, MSE-05, and MSE-06 eligible subject to host availability.

## MSE-03 — Build locked Linux CI images

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: MSE-02
- Exact write set: `Dockerfile.rust`, `mise.toml`, `mise.lock`, `.wiki/migrations/mise-toolchain-integration.md`
- Primary integration lock: CI/build (`Dockerfile.rust`)
- Outcome: Pin mise in the Rust image, install project tools from the committed lockfile, retain required system packages, and produce equivalent AMD64 and ARM64 tool manifests.
- Focused validation: Build both image architectures; run locked-install verification in a clean layer; compare tool manifests; smoke-test the Make targets used by test, docs, portability, fuzz, benchmark, and release jobs.
- Handoff: Remove MSE-03, preserve image/version evidence, and make MSE-04 ready.

## MSE-04 — Cut GitLab CI over to locked mise

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: MSE-03
- Exact write set: `.gitlab-ci.yml`, `Dockerfile.rust`, `.wiki/migrations/mise-toolchain-integration.md`
- Primary integration lock: CI/build (`.gitlab-ci.yml`)
- Outcome: CI uses the image's pinned mise and locked project tools, removes redundant fallback installers and pins, and triggers relevant jobs when mise configuration or locks change.
- Focused validation: Validate the merged GitLab configuration and run test, test-docs, portability, fuzz-smoke, benchmark, and release-build pipelines on clean runners without unlocked tool resolution.
- Handoff: Remove MSE-04 and preserve pipeline IDs, image digests, and tool manifests as current evidence.

## MSE-05 — Accept the macOS developer workflow

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: MSE-02; an available supported macOS ARM64 host
- Exact write set: `.wiki/migrations/mise-toolchain-integration.md`; external system: macOS ARM64 development host
- Primary integration lock: macOS toolchain acceptance
- Outcome: Prove clean locked setup and the representative Make workflow on macOS without relying on globally installed project tools.
- Focused validation: Clean locked install; setup, lint, tests, book, portable checks, and bounded fuzzing; record OS, architecture, mise version, and tool manifest.
- Handoff: Remove MSE-05 and preserve accepted macOS prerequisites as current evidence for MSE-07.

## MSE-06 — Accept the Windows developer workflow

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: MSE-02; an available supported Windows AMD64 host with the repository's existing GNU Make environment
- Exact write set: `.wiki/migrations/mise-toolchain-integration.md`; external system: Windows AMD64 development host
- Primary integration lock: Windows toolchain acceptance
- Outcome: Prove clean locked setup and the representative Make workflow on Windows while retaining the existing OpenSSL and GNU Make prerequisites.
- Focused validation: Clean locked install; setup, lint, tests, book, portable checks, and bounded fuzzing where supported; record OS, shell, architecture, mise version, and tool manifest.
- Handoff: Remove MSE-06 and preserve accepted Windows prerequisites as current evidence for MSE-07.

## MSE-07 — Publish the update workflow and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: MSE-04, MSE-05, MSE-06
- Exact write set: `CONTRIBUTING.md`, `docs/src/getting-started.md`, `.wiki/migrations/mise-toolchain-integration.md`, `.wiki/migrations/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: developer setup guidance
- Outcome: Publish installation, locked-update, system-prerequisite, cache, and troubleshooting guidance; run final workspace gates; retire the plan.
- Focused validation: Repeat the documented clean setup, run `make lint`, `make test`, `make book`, `make check-portable`, validate documentation links, and validate the wiki.
- Handoff: Delete this plan and every index, network, and lane reference after durable guidance and acceptance evidence remain outside `.wiki`.

[^mise-ci]: [mise continuous-integration guidance](https://mise.jdx.dev/continuous-integration.html)
[^mise-rust]: [mise Rust version-file support](https://mise.jdx.dev/lang/rust.html#version-files)
[^mise-lock]: [mise lockfile backend support](https://mise.jdx.dev/dev-tools/mise-lock.html#backend-support)
