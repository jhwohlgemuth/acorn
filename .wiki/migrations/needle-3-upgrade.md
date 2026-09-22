---
type: Plan
title: "Needle 2 to Needle 3 upgrade"
description: "Migrate the managed Needle sidecar to pinned Needle 3 engine and weight assets while preserving tool policy, offline operation, and custom bundles."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [migration, inference, sidecar, needle, modelkit]
status: planned
---

# Goal

Make Needle 3 the supported default for ACORN's local tool-calling sidecar after compatibility and quality gates pass. Preserve policy-controlled tool dispatch and offline deployment, and establish explicit handling of older custom bundles. This is an engine, artifact, and protocol migration, not only a repository-name update.

# Current evidence

Evidence checked on 2026-09-22; record immutable upstream revisions during implementation.

- `crates/acorn-lib/src/util/constants/app.rs` pins `Cactus-Compute/needle2` at `98fbd955b0347e78059be0c253cc1ffa09b87bc7`, with a default confidence threshold of 0.8 and eight rounds.
- `io/sidecar/needle/assets.rs` pins executable checksums/sizes for Windows x86_64/ARM64, Linux x86_64/ARM64, and macOS ARM64. Its default download contains one runner per platform.
- `io/sidecar/needle/process.rs` launches the runner with tools, tool index, serve, and port arguments; it does not pass a separate model path. Inspect shared command environment behavior before concluding how telemetry is currently controlled.
- `io/api/needle.rs` sends an `input` request to `/complete`. `io/sidecar/needle/mod.rs` consumes function calls, checks confidence, dispatches through `ToolRegistry` with the offline context, and feeds serialized tool results into subsequent rounds.
- `io/sidecar/mod.rs` currently accepts schema-version-1 sidecar manifests containing runner metadata and optional tools. `scripts/needle/create-oci-artifact.{sh,ps1}` and `docs/src/inference/needle.md` package this layout.
- [Needle 3's model card](https://huggingface.co/Cactus-Compute/needle3) describes a separate engine and `.cact` weights, schema-constrained tool calls, extraction, embeddings, and deployable depth variants. These capabilities require contract tests against the selected revision.

# Reasons to upgrade

1. **A broader local inference foundation:** extraction and embeddings provide candidates for future ACORN metadata extraction and tool retrieval without adding another runtime. Exposing these workflows remains owned by their feature plans.
2. **Capacity can match the workload:** upstream documents subnetworks from 2 to 20 layers and a fine-tuning/export workflow. This creates a path to smaller ACORN-specific deployments after evaluation; custom training is not required for this migration.
3. **Separate engine and weights:** the new packaging supports independent artifact identities and reproducible deployment. It also makes missing or incompatible weights a new failure mode that ACORN must handle explicitly.
4. **Evaluate the current upstream generation:** compare its tool selection, argument accuracy, refusal behavior, latency, and memory with ACORN's pinned Needle 2 baseline rather than assuming public benchmark gains transfer to ACORN.

The model card advertises 8–29 MB deployment variants; actual selected artifact bytes, supporting assets, and runtime memory must be measured. Local fine-tuning exports at four bits while upstream's shipped two-bit process uses its platform. These are planning opportunities, not measured ACORN improvements. [Upstream model and customization documentation](https://huggingface.co/Cactus-Compute/needle3)

Grammar-constrained output and confidence already exist in ACORN's Needle 2 integration; do not present them as newly introduced benefits. The [upstream README](https://github.com/cactus-compute/needle) documents telemetry enabled by default and the opt-out environment variables `NEEDLE_TELEMETRY=0` and `DO_NOT_TRACK=1`. Set and verify these for the managed process. Apache-2.0 is the published Needle 3 model license; retain the pinned artifact's license and notices.

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/sidecar/mod.rs`; `io/sidecar/needle/{assets,mod,process}.rs` and tests; `io/api/needle.rs` and `io/api/tests/needle.rs`; `util/constants/{app,oci}.rs` as needed; CLI download/infer/export consumers and CLI definitions where their contracts change; `scripts/needle/create-oci-artifact.{sh,ps1}`; `docs/src/inference/needle.md`; generated help if changed.
- **Create:** Dedicated Needle 3 protocol/bundle fixtures under `tests/fixtures/needle/` and an explicit reproducible comparison harness under `scripts/needle/`.
- **Delete:** None initially. Remove obsolete v2-only implementation only after migration behavior and rollback documentation are validated; do not delete user caches automatically.

Shared locks include `io/sidecar/mod.rs`, API/module roots if changed, CLI definitions/generated help, manifests, and OCI packaging contracts. Narrow each slice to exact files before implementation.

# Dependencies and ownership

- Lane: Agent tools and sidecars in the [parallel execution guide](../parallel-execution.md). Respect the existing RPC/ACP/MCP priority and shared integration locks.
- Coordinate with [OCI interoperability](../features/oci-model-artifact-downloads.md): preserve its materialization boundary and distinguish explicit executable-sidecar `.cact` assets from its selected-GGUF inference policy.
- Land multi-file asset validation before the shared packaging slice of [SLM sidecars](../features/slm-sidecars.md). Independent model evaluation can proceed before either integration.
- [Structured extraction inference](../features/structured-extraction-inference.md) owns any user-facing extraction adapter. Embedding/search features and fine-tuning are follow-on work, not prerequisites or migration acceptance requirements.

# Recommendation and implementation slices

1. Capture the Needle 2 baseline and pin a Needle 3 release/commit. Inventory real engines and weights, hashes, supported platforms, licenses, request/response examples, CLI flags, context limits, and failure codes. Verify all five currently supported host targets; do not infer artifact names from v2. Record download-size discrepancies rather than promising the headline range.
2. Extend resolved sidecar assets to represent and validate the required model separately from the runner. Version the custom bundle contract and inventory all required assets with safe relative paths and hashes. Keep it narrowly sufficient for the two concrete sidecar consumers. Namespace caches and retrieval indexes by engine/model revision, quantization/depth, protocol, and tool-catalog identity as applicable. Validate cached artifacts offline and stage downloads atomically.
3. Update startup for the verified v3 flags and explicit model path. Enforce telemetry opt-out, private loopback, bounded startup/request times, cancellation, isolated session directories, and cleanup. Check process-tree behavior on Windows and Unix and prevent shared session files from colliding.
4. Contract-test `/complete` and multi-round state against the pinned engine before adapting DTOs. Preserve response/refusal semantics, empty-call termination, sequential calls, tool-result handoff, output limits, and max rounds. Missing, non-finite, out-of-range, or insufficient confidence cannot authorize calls. Recalibrate the threshold from held-out data; do not assume the v2 value transfers. Schema correctness never bypasses tool permissions or argument validation.
5. Update local/remote/OCI bundle scripts to include weights and required metadata. Read old manifest versions explicitly: preserve a v2 bundle only through a tested version-aware path, or reject it with an actionable migration diagnostic. Never interpret a v1 runner as v3 implicitly. Ship a documented conversion/repackaging path; reject mixed engine/weight generations. Keep a tested rollback procedure to the prior ACORN release and pinned v2 assets.
6. Run side-by-side evaluations, then switch the default constants and docs only when exit gates pass. Preserve normal `infer`, download, and tool-export behavior wherever compatible. Explain any CLI or custom-bundle changes, regenerate help, and document the exact platform matrix and asset footprint.

# Validation

- Deterministic protocol fixtures cover valid calls, malformed arguments, unknown tools, off-topic/refused input, empty responses, multiple calls, missing/invalid confidence, oversized output, context overflow, tool errors, and round exhaustion.
- Asset tests cover separate missing/corrupt engine and weights, incompatible revisions, old/new manifest versions, path traversal/symlink escape, unsupported platform, partial download recovery, offline cache hits/misses, and OCI/local parity.
- Process tests verify flags/model path, telemetry environment, timeouts, cancellation, concurrent sessions, child cleanup, and no unexpected network during offline inference.
- Compare both generations on the same held-out ACORN catalog/prompts: exact tool and argument match, unauthorized-call rejection, refusal quality, confidence/abstention behavior, multi-round completion, cold/warm latency, peak RAM, and total installed bytes. Define numeric regression tolerances before evaluating; require zero policy escapes and no unexplained correctness regressions. Keep measured results in durable inference documentation.
- Run real-engine smoke tests on each claimed host target; mark untested targets explicitly instead of declaring parity. Ordinary unit tests remain offline and model-independent.
- Run formatting, lint, focused tests, `make test`, feature checks, and help/documentation validation appropriate to changed files.

# Acceptance criteria

- [ ] Default Needle assets are an immutable, verified v3 engine/weight pair on every claimed platform.
- [ ] Existing ACORN tool policy, confidence gating, result handoff, round limits, offline behavior, and process isolation pass regression tests.
- [ ] Custom bundles carry all required assets, distinguish generations, and provide tested migration diagnostics and rollback instructions.
- [ ] Telemetry is explicitly disabled and offline inference performs no unexpected network access.
- [ ] A reproducible v2/v3 comparison meets predeclared quality/resource gates; upstream claims are distinguished from measured ACORN gains.
- [ ] Documentation and packaging scripts agree with the implemented CLI, protocol, bundle version, and support matrix. New extraction/embedding workflows remain separately scoped.
