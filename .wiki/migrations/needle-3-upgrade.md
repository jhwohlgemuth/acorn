---
type: Plan
title: "Needle 2 to Needle 3 upgrade"
description: "Validate the shipped Needle 3 engine, weights, protocol, and policy on the remaining Windows and Linux release hosts."
difficulty: medium
impact: high
package: [workspace, acorn-lib, acorn-cli]
tags: [migration, inference, sidecar, needle, release-validation]
status: waiting-on-validation
execution: phased
phase_budget_hours: 5
---

# Project code boundaries

- **Update:** `.gitlab-ci.yml` or release-host job definitions, the existing Needle comparison harness only if a host exposes a reproducible defect, and `docs/src/inference/needle.md` with verified platform results.
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
- Oklahoma reports `linux/x86_64`; Dante reports `linux/aarch64`. Both contexts have schedulable GitLab runners, and direct real-engine runs on them count as Linux platform validation evidence.
- Formatting, lint, WASM, book, workspace, focused Needle, help, shell, packaging, and macOS real-engine validation have accepted results. `make doc` and local PowerShell syntax validation retain unrelated environment/baseline failures already documented outside this remaining host gate.

# Dependencies and coordination

- Lane: Agent tools and sidecars in the [parallel execution guide](../parallel-execution.md).
- Serialize edits to shared sidecar roots, manifests, OCI materialization, CLI roots, generated help, `.gitlab-ci.yml`, and `Makefile`.
- [SLM sidecars](../features/slm-sidecars.md) may proceed independently because the shared ModelKit contract is already available; only shared CI/build locks require coordination.
- [Deterministic and assisted gather extraction](../features/gather-extraction.md) owns any user-facing extraction adapter. Embedding, search, and fine-tuning remain outside this migration.

# Remaining release-host validation

For each of Windows x86_64, Windows ARM64, Linux x86_64, and Linux ARM64, use the applicable Windows GitLab job or Linux Docker context:

1. Download the pinned runner, model, and license through the normal verified asset path and confirm exact size and SHA-256 values.
2. Start the real engine with the shipped flags and compact tool catalog; verify telemetry opt-out, private loopback binding, startup timeout, and session isolation.
3. Run the frozen version and policy smoke inputs, including one multi-round tool-result handoff, and require the predeclared confidence and authorization outcomes.
4. Exercise offline cache success and network rejection, timeout/cancellation, normal shutdown, and process-tree cleanup.
5. Record runner, OS, architecture, artifact revision, outcome, latency, peak memory where available, and any accepted platform-specific variance in durable inference documentation.

Do not change the shared protocol, loosen confidence or policy gates, replace immutable assets, or claim a platform based only on cross-compilation. If a host exposes a defect, add the smallest regression fixture and repair only that behavior before rerunning the affected matrix.

# Validation

- All four remaining platform validations use the repository’s documented real-engine inference path and pinned artifacts rather than synthetic protocol fixtures alone. Linux x86_64 and ARM64 may run directly on Oklahoma and Dante; those runs count as final platform evidence without a CI rerun. Windows x86_64 and ARM64 use the prepared GitLab jobs.
- Each run proves verified acquisition, startup, one bounded inference, policy enforcement, offline behavior, cancellation, and child cleanup.
- Any code or job-definition change passes focused Needle tests, `make lint`, `make test`, applicable shell/PowerShell validation, and the affected platform job.
- Documentation names platforms that actually ran the real-engine inference and does not infer runtime support from artifact availability.

# Acceptance criteria

- Real-engine smoke validation passes on Windows x86_64, Windows ARM64, Linux x86_64, and Linux ARM64 with the pinned Needle 3 engine and weights. Linux direct-context runs count without CI reruns, and durable documentation records the verified matrix.

# Completion handling

Delete this plan after all four host validations pass, then remove its migration-index, plan-network, and parallel-execution entries. Keep the pinned contract, platform matrix, measured results, rollback guidance, and operational details in code, tests, inference documentation, and release history.

# Related plans

- [SLM sidecars](../features/slm-sidecars.md)
- [Model acquisition and metadata hardening](../features/model-acquisition-hardening.md)
- [Deterministic and assisted gather extraction](../features/gather-extraction.md)

# Execution phases

## NDL-01 — Prepare Windows release-host jobs

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: schedulable Windows x86_64 and Windows ARM64 release hosts
- Exact write set: `.gitlab-ci.yml`
- Primary integration lock: CI build entry points
- Outcome: Define the two Windows real-engine jobs that use the pinned artifacts and common evidence schema.
- Focused validation: GitLab CI lint and focused synthetic Needle tests.
- Handoff: Remove NDL-01 and make the two Windows host phases ready.

## NDL-02 — Validate Windows x86_64

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: NDL-01 and a schedulable Windows x86_64 release host
- Exact write set: Windows x86_64 release host, `docs/src/inference/needle.md`
- Primary integration lock: Needle host matrix
- Outcome: Record verified acquisition, inference, policy, offline, cancellation, and cleanup evidence.
- Focused validation: Run the documented real-engine inference path on Windows x86_64.
- Handoff: Remove NDL-02 and retain only the durable platform result.

## NDL-03 — Validate Windows ARM64

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: NDL-01 and a schedulable Windows ARM64 release host
- Exact write set: Windows ARM64 release host, `docs/src/inference/needle.md`
- Primary integration lock: Needle host matrix
- Outcome: Record the pinned real-engine contract on Windows ARM64.
- Focused validation: Run the documented real-engine inference path on Windows ARM64.
- Handoff: Remove NDL-03 and retain only the durable platform result.

## NDL-04 — Validate Linux x86_64

- Status: in-progress
- Budget: 5 active hours maximum
- Prerequisites: schedulable Oklahoma Linux x86_64 Docker context and GitLab runner
- Exact write set: Oklahoma Docker context, `docs/src/inference/needle.md`
- Primary integration lock: Needle host matrix
- Outcome: Run the real-engine smoke directly on Oklahoma and record the verified Linux x86_64 contract. This evidence satisfies the platform gate without a later GitLab CI rerun.
- Focused validation: Run the documented real-engine inference path on Oklahoma's Linux x86_64 Docker context.
- Handoff: Oklahoma reports `linux/x86_64`; the CLI image built successfully from committed source (`sha256:23390858c8e67f2ef55191f251a62eac000b9d0d558d3315e0bcb6a0496031e9`). Online inference, cached offline inference, version-tool round trip, unsupported-weather abstention, low-confidence tool withholding, and offline URL rejection passed. Three additional online runs took 4.0, 3.9, and 3.8 seconds wall time, reporting 77.8–78.0 MB peak RAM. Cancellation/process-tree cleanup remain unverified: inference completed before the container could be inspected, and the image is distroless, preventing an in-container process probe. A detached CLI invocation also completed before inspection; a model-only invocation exited 2 before inspection. The configured Oklahoma SSH alias and Docker context both respond as `linux/x86_64`. NDL-04 remains active for a controlled cancellation/process-tree check and durable documentation.

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
- Prerequisites: NDL-02, NDL-03, NDL-04, NDL-05
- Exact write set: `docs/src/inference/needle.md`, `.wiki/index.md`, `.wiki/migrations/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/migrations/needle-3-upgrade.md`
- Primary integration lock: inference documentation
- Outcome: Consolidate the five-platform matrix, run final gates, and remove the migration plan.
- Focused validation: Run focused Needle tests, applicable platform checks, `make lint`, `make test`, and wiki validation.
- Handoff: Delete this plan and its index, network, and lane entries.
