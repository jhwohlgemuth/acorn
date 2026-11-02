---
type: Plan
title: Julia decision sidecar profile
description: Release an advisory Julia Choice and Noul sidecar behind the shared ask router after parity and quality gates.
difficulty: high
impact: high
package: [acorn-lib, acorn-cli, acorn-core]
tags: [feature, inference, slm, julia, sidecar, decision]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Add an explicitly invoked, advisory finite-choice decision capability. It does not block Tongue, Redact, or Title and has independent runtime and release gates. Native routing and generic adapter contracts are tracked in their own plans.

## Evidence and intended use

- [Supersonic Labs' introduction](https://supersoniclabs.ai/julia-1/) describes Julia-1 as a CPU-capable, 144.3M-parameter decision model with 2–20 supplied answers. The launch evaluation uses a 1,024-token budget and reports weaknesses when shortlisting large label sets. These results do not establish ACORN task quality.
- The [current model card](https://huggingface.co/SupersonicLabs/Julia-1) specifies Python 3.11+, the custom Julia runtime, Apache-2.0 artifacts, and 550.5 MiB weights. It documents named `choice`, `score`, and `noul` questions, strict encoding, and an 8,192-token runtime limit; 8k task accuracy is unproven. `score` is an expected rubric index, and `noul` is the probability of true. The legacy list API rounds probabilities. A separate ONNX/WebGPU export is linked. Pin and inspect runtime source before relying on these moving interfaces.
- Local evidence rechecked on 2026-10-06: `crates/acorn-lib/src/io/sidecar/mod.rs` exposes `SidecarAssets`, `SidecarDistribution`, and consuming `Sidecar::infer`, with an associated `ToolIndex`. Only the Needle child module exists. Julia requires a new concrete adapter; none is shipped. Existing ModelKit resolution verifies platform, protocol, paths, sizes, and hashes.

The first consumer is explicit research-text triage: given a supplied abstract and a small caller-defined topic rubric, return an advisory category or abstention for human review. This is a proposed pilot, not an established user requirement or automatic RAD field mapping. Do not use the result to validate factual claims, approve tools, modify records, or replace deterministic schema validation. A maintained ACORN corpus and a named evaluation owner are prerequisites for promotion.

## Contract and architectural decisions

- Register the decision capability behind `acorn ask` and retain `acorn download slm decide [--status]`. Freeze structured request attachment and explicit backend selection under the CLI lock; do not introduce `infer decide` or reserve a new prompt token. Use the existing custom-location mechanism. Download is explicit; inference requires verified local assets and cannot silently acquire missing files.
- Implement a concrete `JuliaSidecar` behind the opt-in host `slm` feature. Preserve `SupersonicLabs/Julia-1`, runtime/tokenizer revisions, artifact hashes, and adapter protocol identity in provenance. Needle may select the registered Julia capability; Julia executes the final typed decision without recursive tool selection, a chat-completions provider, or a GGUF loader.
- Include ONNX Runtime CPU inference in this slice behind the ACORN-owned launcher and bounded JSON-lines protocol. Retain pinned upstream Python/PyTorch as the reference backend for parity testing. Initially use Python ONNX Runtime bindings to keep the existing runner boundary; a native Rust runner is a later packaging decision. Package a locked runtime and all dependencies; never run pip or fetch tokenizer files during inference. ONNX becomes the default only after CPU compatibility, parity, packaging, and quality gates pass; a Python-only result does not complete the ONNX scope.
- Reuse `SidecarAssets` and `SidecarDistribution`; resolve the required `ToolIndex` with a documented no-op implementation or a narrowly scoped shared-contract change approved under the library-root lock. Do not create a fake tool catalog. Use the shared capability registry without a Julia-specific registry or broad sidecar refactor.
- One process loads once, handles one bounded request containing multiple independent questions, then exits, matching consuming `infer`. Persistent sessions require a separate measured justification; warm upstream throughput is not a claim about this cold-start lifecycle.
- Versioned input contains a request ID, source text, and ordered questions with stable unique IDs. Model questions and answers as tagged Rust enums with `Choice` and `Noul` variants, serialized as `choice` and `noul`. Choice accepts 2–20 unique option IDs and nonempty descriptions. Preserve caller order in an explicit sequence; reject duplicate IDs and unknown fields. Typed `score` remains deferred.
- Return the selected caller ID, ordered full-precision probabilities, explicit abstention reason, and model/runtime/prompt provenance plus source fingerprint. Reject mismatched IDs, missing/extra probabilities, non-finite or out-of-range values, and distributions outside a documented sum tolerance. Use the named API or verified logits path, not rounded legacy output. Scores are conditional model estimates, not calibrated certainty.
- ACORN owns abstention using thresholds frozen on development data; distinguish low confidence/insufficient evidence from invalid input and technical failure. Never turn a crash or timeout into a successful decision. Disable hierarchical routing: more than 20 options is an input error.
- Proposed pilot budgets: 64 KiB UTF-8 per request, at most 16 questions, 1,024 combined tokens per encoded question, 120-second startup, 60-second inference, and 1 MiB response. Enforce strict tokenizer limits without silent truncation; measure and freeze question/option sub-budgets against the pinned runtime. Larger context is a later evaluated change.
- Keep diagnostics off stdout and redact source text from errors. Verify network-free execution, timeout/cancellation, child-process cleanup, and bounded output. Use runtime-appropriate offline controls; the Desert Ant-specific `DAL_USAGE_DISABLED` flag is not evidence of Julia offline behavior.

## Noul type modeling

Treat `Noul` as a distinct probabilistic Boolean result, not an alias for `bool` or a nullable choice. Reuse the existing `acorn_core::Noul` probability type where its invariants fit; add sidecar-local `NoulCriteria`, `NoulQuestion`, and `NoulOutcome`. Do not introduce an incompatible duplicate probability type.

- `NoulCriteria` has named false/true descriptions, both nonempty when supplied. An omitted criteria object uses literal false/true; a partial, empty, or extra-key object fails validation. Encode exactly two alternatives in false-then-true order regardless of JSON key order. Preserve supplied descriptions: the [upstream model card](https://huggingface.co/SupersonicLabs/Julia-1) reports a corrected API bug that previously discarded Boolean descriptions.
- `Noul` stores a private, validated full-precision `probability_true` with fallible construction and deserialization. Reject NaN, infinity, and values outside inclusive `[0, 1]`; expose `probability_false` as the complement. Validate any upstream two-value distribution and its agreement with `probability_true` before construction. No implicit Boolean conversion or display rounding enters the inference path.
- `NoulOutcome` explicitly represents `Answered { value: bool }` or `Abstained { reason }`, alongside the unchanged probability and common provenance. For a frozen confidence threshold `t` with `0.5 < t <= 1`, answer true when `p >= t`, false when `p <= 1 - t`, otherwise abstain. Missing evidence can independently require abstention; technical failures remain errors. Record the threshold/policy revision so decisions are reproducible.
- Cover serde round trips, unknown fields, omitted versus descriptive criteria, swapped key order, exact endpoints, midpoint ties, threshold boundaries, and invalid values. Evaluate a separate held-out Boolean evidence-triage corpus, such as whether supplied prose explicitly describes a dataset; the result remains advisory. Freeze at least 200 labeled Boolean cases and apply the same accuracy/coverage gates as choice separately, reporting false-positive/false-negative rates and calibration.

## ONNX inference and parity

The [official ONNX export](https://huggingface.co/SupersonicLabs/Julia-1-ONNX) supplies `model.onnx`, external `model.onnx.data`, tokenizer assets, and a browser WebGPU adapter. Its documented 100-request agreement test is browser evidence, not proof of ACORN CPU execution. Native ONNX Runtime CPU support is an implementation feasibility gate; browser/WebGPU integration remains outside this slice.

- Pin export and source-checkpoint revisions together and verify their relationship. Package the graph, every referenced external tensor file, tokenizer/configuration, encoding adapter, and ONNX Runtime libraries as separately hashed ModelKit assets. Validate external-data paths for containment before session creation; reject missing, altered, or escaping references.
- Inspect and freeze graph opset, input/output names, tensor dtypes/shapes, dynamic dimensions, masks, option markers, and valid-option slicing. Reproduce reference tokenization and strict overflow behavior for both `Choice` and `Noul`. Compute stable softmax over valid logits only; never consume rounded display probabilities or padded-option logits.
- Start with the CPU execution provider and record its actual selection, runtime version, thread settings, and optimization settings. Fail clearly on unsupported operators or provider initialization; do not silently switch backends. Propose explicit `--backend <onnx|python>` selection for inference and download, with backend-specific artifact identity/cache selection and provenance. Python is an explicit reference option, not an automatic fallback.
- Compare identical encoded tensors, raw logits, full-precision probabilities, selected answers, and abstention outcomes against the pinned Python CPU reference. Freeze numeric tolerances on development fixtures before the held-out run; proposed initial gates are maximum absolute probability error `1e-4` and identical decisions on cases outside that tolerance of a tie or policy threshold. Report every near-boundary difference separately, and require both backends to pass the independent ACORN quality gates. Any tolerance change requires recorded evidence and a plan revision.
- Test single/mixed-kind batches, padding, option counts 2 and 20, descriptive/default Noul criteria, Unicode, limit/overflow inputs, and repeated deterministic execution. Quantization and alternative execution providers need separate parity/resource evidence. Measure the existing resource ceilings for ONNX itself on every advertised target.

## Boundaries, dependencies, and sequence

1. **Pin and prove feasibility.** Record immutable Julia code/weight and tokenizer revisions, complete hashes, dependency lock, licenses/notices, and build instructions. Inspect strict encoding and named API behavior at that revision. Establish a reproducible Linux x86_64 CPU baseline first. Linux ARM64/macOS ARM64 need their own packaging and smoke evidence before advertisement; Windows remains outside this concept's current scope. Stop promotion if redistribution or offline packaging cannot be proven.
2. **Freeze evaluation and protocol.** Name the triage owner and approve the input/output schema, abstention behavior, resource limits, held-out corpus, and numeric gates before implementation integration. Compare with a deterministic keyword baseline; SemIf/Open-Jev/Needle and GLiDE comparisons use equivalent tasks only where available and deployment-approved. Their availability is not a Julia prerequisite.
3. **Build and test the owned runner.** Proposed leaf write set: `scripts/slm/julia/runner.py`, `scripts/slm/julia/onnx_backend.py`, `scripts/slm/julia/test_onnx_parity.py`, `scripts/slm/julia/pyproject.toml`, `scripts/slm/julia/uv.lock`, `scripts/slm/julia/README.md`, and `scripts/slm/julia/test_runner.py`. Lock dependency tooling during feasibility. Implement both Choice and Noul encoding/results, prove ONNX CPU parity against the reference backend, and package graph/external weights, tokenizer, runtime, and notices as verified ModelKit assets; publish immutable artifacts before pinning release checksums.
4. **Integrate typed Rust support.** Proposed new files: `crates/acorn-lib/src/io/sidecar/slm/decision.rs` and `crates/acorn-lib/src/io/sidecar/slm/decision/tests.rs`. Reuse the parent's asset/process/protocol leaves only after their contract is landed; otherwise implement the minimum Julia-owned leaves without assuming those planned files exist. Claim exact changed files before work starts. Shared `slm/mod.rs`, `sidecar/mod.rs`, feature manifests, and embedded asset registration require their owners' integration locks.
5. **Wire explicit CLI and docs.** Claim the current inference/download command files, parser/dispatch roots, generated help, `docs/src/inference/slm.md`, and book index under the parent locks. Add dedicated `tests/fixtures/slm/decision/` protocol and evaluation fixtures, listing exact files at implementation handoff. No research-activity schema, analyzer/check, gather, or database changes belong to this slice.
6. **Validate and release separately.** Run protocol tests without model downloads, then opt-in real-model offline/package tests on every advertised target. Record the measured decision quality and resource report. Julia can ship only after its own gates pass, independent of the three Swift adapters.

The [model acquisition plan](model-acquisition-hardening.md) owns shared Hugging Face/OCI/materialization changes; consume its landed contracts and coordinate any missing capability there. Workspace feature changes use the workspace-graph lock in the [parallel execution guide](../parallel-execution.md). The Agent tools lane serializes shared sidecar, CLI, artifact, and CI/build changes with Needle and the other SLM adapters. No dependency on Title's macOS runner is imposed on the Linux-only Julia pilot.

## Julia validation and acceptance

Proposed release gates must be accepted and frozen before scoring: at least 200 held-out, human-labeled ACORN triage examples; at least 90% accuracy among answered examples at at least 80% coverage; and no lower full-corpus accuracy than the deterministic baseline when abstentions count as incorrect. Report per-label precision/recall, confusion matrix, calibration, and accuracy-versus-coverage. Include missing-evidence, ambiguous, scientific-abbreviation, reordered-option, Unicode, and supported-language cases. Keep development/calibration and final test sets disjoint; failed gates return the slice to evaluation rather than justify post-hoc threshold tuning.

Measure cold/warm latency, p50/p95, peak process-tree RSS, installed/download bytes, CPU threads, hardware, and runtime versions. Proposed Linux x86_64 pilot ceilings are 120 seconds cold readiness, 5 seconds p95 for one question after readiness, and 2 GiB peak process-tree RSS. Confirm these budgets with the evaluation owner before the final run; exceeding them requires a documented plan revision before release.

- [ ] Immutable code, weights, tokenizer, runtime dependencies, notices, sizes, and hashes are recorded; clean-machine installation and inference work offline.
- [ ] Strict 2–20 option handling, token overflow, stable order/IDs, malformed/non-finite probabilities, abstention, and provenance have deterministic fixtures.
- [ ] Typed Noul construction/deserialization, false/true descriptions and ordering, probability semantics, threshold boundaries, explicit abstention, and independent Boolean quality gates pass.
- [ ] ONNX CPU graph loading, external-data containment/integrity, exact encoding, output parity, explicit backend selection, offline packaging, and per-target resource gates pass against pinned reference artifacts; Python-only completion is insufficient.
- [ ] Protocol tests cover readiness, request IDs, oversized input/output, startup failure, timeout, cancellation, cleanup, and redacted diagnostics without downloading weights.
- [ ] Explicit download/inference, missing-cache errors, custom locations, unsupported hosts, and legacy Needle parser compatibility are tested.
- [ ] Frozen ACORN quality and resource gates pass on every advertised target; upstream benchmark numbers alone cannot satisfy this gate.
- [ ] Focused tests, `make lint`, `make test`, applicable feature/WASM checks, dependency/license checks, generated help, and real-model platform smoke jobs pass for implementation changes.
- [ ] Documentation presents advisory decisions and limitations; no automatic RAD mutation, authorization, remote fallback, or background inference is introduced.

# Execution phases

## JUL-01 — Pin Julia feasibility inputs

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: named Julia pilot owner
- Exact write set: `scripts/slm/julia/README.md`, `.wiki/features/julia-decision-sidecar.md`
- Primary integration lock: Julia adoption decision
- Outcome: Pin code, weights, tokenizer, APIs, licenses, build, and offline packaging for Linux x86_64.
- Focused validation: Reproduce one CPU reference inference from locked inputs or stop promotion.
- Handoff: Remove JUL-01 or prune the Julia section if feasibility fails.

## JUL-02 — Freeze protocol, corpus, and gates

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-01; named evaluation owner supplies and reviews the Choice and Noul corpora, each with at least 200 labeled cases and disjoint development/test splits
- Exact write set: `tests/fixtures/slm/decision/contract.json`, `tests/fixtures/slm/decision/choice-evaluation.jsonl`, `tests/fixtures/slm/decision/noul-evaluation.jsonl`, `docs/src/inference/julia-evaluation.md`
- Primary integration lock: Julia pilot contract
- Outcome: Validate and freeze the owner-supplied corpora, schemas, abstention, limits, baseline, quality, parity, and resource gates without authoring the full evaluation set inside this phase.
- Focused validation: Check schema, count, duplicate IDs, development/test separation, and an independent label sample against the owner's reviewed source.
- Handoff: Remove JUL-02 and prohibit post-hoc gate changes.

## JUL-03 — Implement the pinned Python reference runner

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-02
- Exact write set: `scripts/slm/julia/runner.py`, `scripts/slm/julia/pyproject.toml`, `scripts/slm/julia/uv.lock`, `scripts/slm/julia/test_runner.py`
- Primary integration lock: Julia runner
- Outcome: Run strict Choice and Noul inference through the locked upstream API with one bounded JSON-lines request.
- Focused validation: Test encoding, IDs, limits, invalid probabilities, timeout, cleanup, and offline operation.
- Handoff: Remove JUL-03 and freeze reference tensors/results.

## JUL-04 — Implement and test Noul types

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-02
- Exact write set: `crates/acorn-lib/src/io/sidecar/slm/mod.rs`, `crates/acorn-lib/src/io/sidecar/slm/decision.rs`, `crates/acorn-lib/src/io/sidecar/slm/decision/tests.rs`
- Primary integration lock: shared sidecar root
- Outcome: Model criteria, validated probability, complement, threshold answer/abstention, and policy revision explicitly.
- Focused validation: Run serde, ordering, endpoints, midpoint, threshold, missing evidence, and invalid-number tests.
- Handoff: Remove JUL-04 and expose the stable Boolean contract.

## JUL-05 — Validate ONNX graph assets

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-01
- Exact write set: `scripts/slm/julia/onnx_backend.py`, `scripts/slm/julia/test_onnx_backend.py`
- Primary integration lock: Julia runner
- Outcome: Pin graph/external data/tokenizer and freeze containment, hashes, opset, names, dtypes, shapes, and dimensions.
- Focused validation: Test missing, altered, escaping, unsupported-op, and provider-initialization failures.
- Handoff: Remove JUL-05 and publish the accepted graph contract.

## JUL-06 — Reproduce strict Julia encoding

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-03, JUL-05
- Exact write set: `scripts/slm/julia/onnx_backend.py`, `scripts/slm/julia/test_onnx_backend.py`
- Primary integration lock: Julia runner
- Outcome: Match reference Choice/Noul tokenization, option order, masks, padding, and overflow without truncation.
- Focused validation: Compare encoded tensors for 2/20 options, default/descriptive Noul, Unicode, mixed batches, and limits.
- Handoff: Remove JUL-06 and freeze encoded fixtures.

## JUL-07 — Implement ONNX CPU inference

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-06
- Exact write set: `scripts/slm/julia/onnx_backend.py`, `scripts/slm/julia/test_onnx_backend.py`
- Primary integration lock: Julia runner
- Outcome: Run explicit CPU provider inference and stable softmax over valid logits with recorded settings.
- Focused validation: Test provider selection, unsupported operators, repeated determinism, malformed outputs, and no fallback.
- Handoff: Remove JUL-07 and expose full-precision ONNX results.

## JUL-08 — Prove Python/ONNX parity

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-03, JUL-07
- Exact write set: `scripts/slm/julia/test_onnx_parity.py`, `docs/src/inference/julia-evaluation.md`
- Primary integration lock: Julia evaluation
- Outcome: Meet frozen tensor/logit/probability/decision tolerances and report every near-boundary difference.
- Focused validation: Run the complete development parity corpus without adjusting tolerances afterward.
- Handoff: Remove JUL-08 or create a bounded parity repair phase.

## JUL-09 — Package immutable Julia artifacts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-08
- Exact write set: Julia artifact registry identified in JUL-01, `tests/fixtures/slm/decision/modelkit.json`, `docs/src/inference/julia-artifacts.md`
- Primary integration lock: SLM artifact publication
- Outcome: Package graph, external weights, tokenizer, runtime, launcher, dependencies, and notices with hashes.
- Focused validation: Install on a clean Linux x86_64 host and run both explicit backends offline.
- Handoff: Remove JUL-09 and provide immutable asset facts to Rust.

## JUL-10 — Add JuliaSidecar

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-04, JUL-09
- Exact write set: `crates/acorn-lib/src/io/sidecar/slm/decision.rs`
- Primary integration lock: shared sidecar root
- Outcome: Add typed Choice/Noul requests/results, assets, limits, provenance, and consuming lifecycle behind `slm`.
- Focused validation: Run fixture, asset, offline, timeout, cancellation, cleanup, redaction, and real-model smoke tests.
- Handoff: Remove JUL-10 and freeze the typed Rust interface.

## JUL-11 — Register decision routing and backend selection

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-10
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/needle.rs`, `crates/acorn-lib/src/io/mcp/tests/server.rs`
- Primary integration lock: Ask capability routing
- Outcome: Register Julia decisions behind `acorn ask` with explicit backend identity; Needle selects Julia and the sidecar returns the final advisory result.
- Focused validation: Fake and real Needle-to-Julia routing, backend availability, option preservation, and abstention.
- Handoff: Remove JUL-11 and freeze the registry contract for CLI composition.

## JUL-16 — Expose decision backend selection in the CLI

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-11
- Exact write set: `crates/acorn-cli/src/commands/infer/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`, `crates/acorn-cli/src/commands/download/mod.rs`, `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/cli/tests/mod.rs`, `CommandLineHelp.md`
- Primary integration lock: CLI syntax and generated help
- Outcome: Add explicit `download slm decide` and `onnx|python` selection while preserving existing inference syntax.
- Focused validation: Parser, input, option-count, backend/cache, custom-location, unsupported-host, and generated help.
- Handoff: Remove JUL-16 and freeze public syntax.

## JUL-12 — Complete decision lifecycle tests

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-16
- Exact write set: `tests/fixtures/slm/decision/contract.json`, `crates/acorn-lib/src/io/sidecar/slm/decision/tests.rs`
- Primary integration lock: Julia acceptance
- Outcome: Cover readiness, IDs, sizes, startup, timeout, cancellation, cleanup, diagnostics, and explicit download/inference.
- Focused validation: Run all model-independent protocol tests without downloading weights.
- Handoff: Remove JUL-12 and preserve regression fixtures.

## JUL-13 — Evaluate Choice decisions

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-12
- Exact write set: `tests/fixtures/slm/decision/choice-evaluation.jsonl`, `docs/src/inference/julia-evaluation.md`
- Primary integration lock: Julia evaluation
- Outcome: Meet 90% answered accuracy, 80% coverage, and full-corpus baseline requirements.
- Focused validation: Run the held-out corpus once and report per-label metrics, confusion, calibration, and abstentions.
- Handoff: Remove JUL-13 or create a bounded evaluation repair phase.

## JUL-14 — Evaluate Noul and resource gates

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-12
- Exact write set: `tests/fixtures/slm/decision/noul-evaluation.jsonl`, `docs/src/inference/julia-evaluation.md`
- Primary integration lock: Julia evaluation
- Outcome: Pass independent Boolean metrics plus frozen latency, memory, bytes, threads, and parity ceilings.
- Focused validation: Run the held-out Noul corpus and measured cold/warm Linux x86_64 profile.
- Handoff: Remove JUL-14 or create a bounded evaluation repair phase.

## JUL-15 — Document and release the independent Julia profile

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-07, JUL-13, JUL-14
- Exact write set: `docs/src/inference/slm.md`, `docs/src/SUMMARY.md`, `.wiki/index.md`, `.wiki/features/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/features/julia-decision-sidecar.md`
- Primary integration lock: SLM release gate
- Outcome: Publish the advisory decision profile after its own gates pass; do not wait for Swift release.
- Focused validation: Run focused tests, lint/full suite, feature/WASM, license/dependency, help/docs, real-model smoke, and wiki validation.
- Handoff: Remove JUL-15 after documenting its release; retire this plan and its index, network, and lane entries when its own acceptance gates pass.
