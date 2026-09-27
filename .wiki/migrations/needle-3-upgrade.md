---
type: Plan
title: "Needle 2 to Needle 3 upgrade"
description: "Validate the shipped Needle 3 engine, weights, protocol, and policy on the remaining Windows and Linux release hosts."
difficulty: medium
impact: high
package: [workspace, acorn-lib, acorn-cli]
tags: [migration, inference, sidecar, needle, release-validation]
status: in-progress
---

# Project code boundaries

- **Update:** `.gitlab-ci.yml` or the release-host job definitions, the existing Needle comparison/smoke harness only if a host exposes a reproducible defect, and `docs/src/inference/needle.md` with verified platform results.
- **Create:** no production module. Add only narrowly scoped host-job configuration or regression fixtures required by an observed failure.
- **Delete:** this plan and its index/network/lane entries after all four remaining release-host jobs pass.

# Goal

Run the shipped Needle 3 workflow on Windows x86_64, Windows ARM64, Linux x86_64, and Linux ARM64 release hosts. Confirm the same artifact, process, protocol, policy, offline, and cleanup behavior already exercised on macOS ARM64 before declaring the migration complete.

# Current baseline

- `crates/acorn-lib/src/util/constants/app.rs` pins `Cactus-Compute/needle3` at revision `b274efcb211a9eef48c9a88da4b43bd569696a39`, with a 0.8 confidence threshold and eight rounds.
- The model archive is pinned at 35,335,380 bytes with SHA-256 `c9d915eca282ed42d1a09b143b592adb4cc6744ffe2d294adf5cfc5548170c38`; the five supported desktop runners have immutable sizes and digests.
- `io/sidecar/needle/` validates a Kitfile-versioned ModelKit manifest, separate engine/model/license assets, sidecar identity and protocol, safe paths, sizes, hashes, symlink containment, and platform selection before startup.
- Needle process startup uses the compact ACORN tool projection, private loopback transport, bounded startup/request times, isolated sessions, output limits, telemetry opt-out, and child cleanup. Tool calls remain subject to confidence, policy, typed argument validation, offline context, and round limits.
- The checked-in comparison harness records policy, correctness, latency, memory, and installed-byte gates for the pinned Needle 2/3 inputs. macOS ARM64 passes the stricter real-engine comparison, online/offline version inference, and download/checksum path.
- Formatting, lint, WASM, book, workspace, focused Needle, help, shell, packaging, and macOS real-engine validation have accepted results. `make doc` and local PowerShell syntax validation retain unrelated environment/baseline failures already documented outside this remaining host gate.

# Dependencies and coordination

- Lane: Agent tools and sidecars in the [parallel execution guide](../parallel-execution.md).
- Serialize edits to shared sidecar roots, manifests, OCI materialization, CLI roots, generated help, `.gitlab-ci.yml`, and `Makefile`.
- [SLM sidecars](../features/slm-sidecars.md) may proceed independently because the shared ModelKit contract is already available; only shared CI/build locks require coordination.
- [Structured extraction inference](../features/structured-extraction-inference.md) owns any user-facing extraction adapter. Embedding, search, and fine-tuning remain outside this migration.

# Remaining release-host validation

For each of Windows x86_64, Windows ARM64, Linux x86_64, and Linux ARM64:

1. Download the pinned runner, model, and license through the normal verified asset path and confirm exact size and SHA-256 values.
2. Start the real engine with the shipped flags and compact tool catalog; verify telemetry opt-out, private loopback binding, startup timeout, and session isolation.
3. Run the frozen version and policy smoke inputs, including one multi-round tool-result handoff, and require the predeclared confidence and authorization outcomes.
4. Exercise offline cache success and network rejection, timeout/cancellation, normal shutdown, and process-tree cleanup.
5. Record runner, OS, architecture, artifact revision, outcome, latency, peak memory where available, and any accepted platform-specific variance in durable inference documentation.

Do not change the shared protocol, loosen confidence or policy gates, replace immutable assets, or claim a platform based only on cross-compilation. If a host exposes a defect, add the smallest regression fixture and repair only that behavior before rerunning the affected matrix.

# Validation

- All four release-host jobs use the repository’s real-engine smoke harness and pinned artifacts rather than synthetic protocol fixtures alone.
- Each job proves verified acquisition, startup, one bounded inference, policy enforcement, offline behavior, cancellation, and child cleanup.
- Any code or job-definition change passes focused Needle tests, `make lint`, `make test`, applicable shell/PowerShell validation, and the affected platform job.
- Documentation names platforms that actually ran the smoke harness and does not infer runtime support from artifact availability.

# Acceptance criteria

- Real-engine smoke jobs pass on Windows x86_64, Windows ARM64, Linux x86_64, and Linux ARM64 with the pinned Needle 3 engine and weights, and durable documentation records the verified matrix.

# Completion handling

Delete this plan after the four host jobs pass, then remove its migration-index, plan-network, and parallel-execution entries. Keep the pinned contract, platform matrix, measured results, rollback guidance, and operational details in code, tests, inference documentation, and release history.

# Related plans

- [SLM sidecars](../features/slm-sidecars.md)
- [ModelKit and ModelPack OCI interoperability](../features/oci-model-artifact-downloads.md)
- [Structured extraction inference](../features/structured-extraction-inference.md)
