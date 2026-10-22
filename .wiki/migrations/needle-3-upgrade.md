---
type: Plan
title: "Needle 2 to Needle 3 upgrade"
description: "Validate the shipped Needle 3 engine, weights, protocol, and policy on Windows x86_64 and Linux ARM64 release hosts."
difficulty: medium
impact: high
package: [workspace, acorn-lib, acorn-cli]
tags: [migration, inference, sidecar, needle, release-validation]
status: waiting-on-validation
execution: phased
phase_budget_hours: 5
---

# Project code boundaries

- **Update:** the existing Needle comparison harness only if a host exposes a reproducible defect, and `docs/src/inference/needle.md` with verified platform results.
- **Create:** no production module. Add only narrowly scoped host-job configuration or regression fixtures required by an observed failure.
- **Delete:** this plan and its index/network/lane entries after the two remaining release-host validations pass.

# Goal

Run the shipped Needle 3 workflow on Windows x86_64, Linux x86_64, and Linux ARM64 release hosts. Confirm the same artifact, process, protocol, policy, offline, and cleanup behavior already exercised on macOS ARM64 before declaring the migration complete. Windows ARM64 is outside this release-host validation matrix.

# Current baseline

- `crates/acorn-lib/src/util/constants/app.rs` pins `Cactus-Compute/needle3` at revision `b274efcb211a9eef48c9a88da4b43bd569696a39`, with a 0.8 confidence threshold and eight rounds.
- The model archive is pinned at 35,335,380 bytes with SHA-256 `c9d915eca282ed42d1a09b143b592adb4cc6744ffe2d294adf5cfc5548170c38`; the five supported desktop runners have immutable sizes and digests.
- `io/sidecar/needle/` validates a Kitfile-versioned ModelKit manifest, separate engine/model/license assets, sidecar identity and protocol, safe paths, sizes, hashes, symlink containment, and platform selection before startup.
- Needle process startup uses the compact ACORN tool projection, private loopback transport, bounded startup/request times, isolated sessions, output limits, telemetry opt-out, and child cleanup. Tool calls remain subject to confidence, policy, typed argument validation, offline context, and round limits.
- The checked-in comparison harness records policy, correctness, latency, memory, and installed-byte gates for the pinned Needle 2/3 inputs. macOS ARM64 passes the stricter real-engine comparison, online/offline version inference, and download/checksum path.
- Oklahoma's Linux x86_64 real-engine run passed verified acquisition, online and cached offline inference, tool handoff, policy withholding, offline network rejection, normal shutdown, and host-observed SIGTERM/SIGINT child cleanup. The [Needle guide](../../docs/src/inference/needle.md) records the 2026-10-21 platform result. Dante reports `linux/aarch64` and remains available for Linux ARM64 validation.
- The manual `validate-needle-windows-amd64` job and `scripts/needle/validate-windows-x86_64.ps1` now prepare pinned Windows x86_64 acquisition, inference, policy, offline, and normal-cleanup evidence. GitLab CI lint and 25 focused Needle tests pass. GitLab runner 55601 is tagged `windows` but reports `linux/amd64`; it has not run a job and does not establish native Windows execution. The job therefore requires a native Windows x86_64 shell runner tagged `windows-x86_64` before Windows validation can start.
- Formatting, lint, WASM, book, workspace, focused Needle, help, shell, packaging, and macOS real-engine validation have accepted results. `make doc` and local PowerShell syntax validation retain unrelated environment/baseline failures already documented outside this remaining host gate.

# Dependencies and coordination

- Lane: Agent tools and sidecars in the [parallel execution guide](../parallel-execution.md).
- Serialize edits to shared sidecar roots, manifests, OCI materialization, CLI roots, generated help, `.gitlab-ci.yml`, and `Makefile`.
- [SLM sidecars](../features/slm-sidecars.md) may proceed independently because the shared ModelKit contract is already available; only shared CI/build locks require coordination.
- [Deterministic and assisted gather extraction](../features/gather-extraction.md) owns any user-facing extraction adapter. Embedding, search, and fine-tuning remain outside this migration.

# Remaining release-host validation

For Windows x86_64 and Linux ARM64, use the Windows GitLab job or Linux Docker context:

1. Download the pinned runner, model, and license through the normal verified asset path and confirm exact size and SHA-256 values.
2. Start the real engine with the shipped flags and compact tool catalog; verify telemetry opt-out, private loopback binding, startup timeout, and session isolation.
3. Run the frozen version and policy smoke inputs, including one multi-round tool-result handoff, and require the predeclared confidence and authorization outcomes.
4. Exercise offline cache success and network rejection, timeout/cancellation, normal shutdown, and process-tree cleanup.
5. Record runner, OS, architecture, artifact revision, outcome, latency, peak memory where available, and any accepted platform-specific variance in durable inference documentation.

Do not change the shared protocol, loosen confidence or policy gates, replace immutable assets, or claim a platform based only on cross-compilation. If a host exposes a defect, add the smallest regression fixture and repair only that behavior before rerunning the affected matrix.

# Validation

- The two remaining platform validations use the repository’s documented real-engine inference path and pinned artifacts rather than synthetic protocol fixtures alone. Linux ARM64 may run directly on Dante; that run counts as final platform evidence without a CI rerun. Windows x86_64 uses the prepared GitLab job.
- Combined host evidence proves verified acquisition, startup, bounded inference, policy enforcement, offline behavior, cancellation, and child cleanup. The Windows job produces the repeatable smoke report; NDL-02 adds the native Ctrl+C observation.
- The prepared job passed GitLab CI lint and focused Needle tests. NDL-02 still requires native PowerShell execution and the affected platform job; NDL-06 runs `make lint` and `make test` after host evidence is complete.
- Documentation names platforms that actually ran the real-engine inference and does not infer runtime support from artifact availability.

# Acceptance criteria

- Real-engine smoke validation passes on Windows x86_64, Linux x86_64, and Linux ARM64 with the pinned Needle 3 engine and weights. Linux direct-context runs count without CI reruns, and durable documentation records the verified matrix.

# Completion handling

Delete this plan after all three host validations pass, then remove its migration-index, plan-network, and parallel-execution entries. Keep the pinned contract, platform matrix, measured results, rollback guidance, and operational details in code, tests, inference documentation, and release history.

# Related plans

- [SLM sidecars](../features/slm-sidecars.md)
- [Model acquisition and metadata hardening](../features/model-acquisition-hardening.md)
- [Deterministic and assisted gather extraction](../features/gather-extraction.md)

# Execution phases

## NDL-02 — Validate Windows x86_64

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: a native Windows x86_64 GitLab shell runner tagged `windows-x86_64`, with PowerShell and access to the build artifact
- Exact write set: Windows x86_64 release host, `docs/src/inference/needle.md`
- Primary integration lock: Needle host matrix
- Outcome: Run the manual Windows job, verify native Ctrl+C cancellation and child/session cleanup, and record the pinned acquisition, inference, policy, offline, and process evidence.
- Focused validation: Run `validate-needle-windows-amd64` on the native host and retain its JSON report plus the cancellation observation.
- Handoff: Runner 55601 is currently `linux/amd64`, so it cannot supply the required native Windows evidence. Install or reconfigure a Windows x86_64 shell runner with the job tag, run the job, complete the Ctrl+C cleanup check, record results in the Needle guide, then remove NDL-02.

## NDL-05 — Validate Linux ARM64

- Status: in-progress
- Budget: 5 active hours maximum
- Prerequisites: schedulable Dante Linux ARM64 Docker context and GitLab runner
- Exact write set: Dante Docker context, `docs/src/inference/needle.md`
- Primary integration lock: Needle host matrix
- Outcome: Run the real-engine smoke directly on Dante and record the verified Linux ARM64 contract. This evidence satisfies the platform gate without a later GitLab CI rerun.
- Focused validation: Run the documented real-engine inference path on Dante's Linux ARM64 Docker context.
- Handoff: Dante reports `linux/aarch64` through both its configured SSH alias and Docker context. Three additional native image-build attempts successfully installed ARM64 system dependencies but each Docker-over-SSH stream disconnected as Rust toolchain installation began. No completed ARM64 image was retained and no ARM64 inference ran. Vergil remains unavailable because SSH host-key verification fails and `ssh-askpass` is absent. NDL-05 remains active pending a stable Docker connection or corrected Vergil host-key access; then complete the real-engine checks and record platform evidence.

## NDL-06 — Publish the verified matrix and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: NDL-02, NDL-05
- Exact write set: `docs/src/inference/needle.md`, `.wiki/index.md`, `.wiki/migrations/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/migrations/needle-3-upgrade.md`
- Primary integration lock: inference documentation
- Outcome: Consolidate the four-platform matrix, run final gates, and remove the migration plan.
- Focused validation: Run focused Needle tests, applicable platform checks, `make lint`, `make test`, and wiki validation.
- Handoff: Delete this plan and its index, network, and lane entries.
