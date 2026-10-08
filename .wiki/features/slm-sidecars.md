---
type: Plan
title: "Small language model (SLM) sidecars"
description: "Ship explicitly selected, ACORN-owned Tongue, Redact, and Title sidecars, with a gated Julia-1 decision sidecar and Schemer/GLiDE follow-ons."
difficulty: high
impact: high
package: [workspace, acorn-lib, acorn-cli]
tags: [feature, inference, sidecar, slm, enrichment, privacy, semantic-decision]
status: in-progress
---

# Goal

Ship three stable, explicitly invoked local capabilities alongside Needle:

- Tongue detects language and exposes uncertainty.
- Redact reports sensitive spans or produces a user-requested redacted copy.
- Title proposes a title and optional subtitle from bounded source text.

ACORN owns the native runner adapters, download metadata, process protocol, and Rust integration. The first release never runs these models from ordinary check or gather commands, never mutates canonical RAD records, and never routes them through Needle's tool loop. Schemer, durable proposal persistence, and semantic-decision review remain later gated slices.

Use SLM as the vendor-neutral integration category, including classifiers and extractors as well as generative models. Name public ACORN capabilities by task. Preserve exact upstream identities in adapter code, artifact provenance, citations, and license notices; this plan does not add a generic provider registry.

# Current evidence

Evidence checked on 2026-09-22. Recheck upstream availability, licenses, and immutable artifact revisions before publishing a release.

- `crates/acorn-lib/src/io/sidecar/mod.rs` separates the complete shared contract into `SidecarAssets` and `Sidecar`. The asset trait owns the typed pinned manifest and runner asset, stable kind/protocol identity, target selection, cache root, ModelKit validation, and verified acquisition; the runtime trait adds typed input/output, verified-asset startup, and required consuming inference. Its landed, Kitfile-versioned ModelKit manifest validates the runner and every named supporting asset, including paths, byte sizes, and SHA-256 digests.
- `io/sidecar/needle/` already owns managed process startup and asset resolution. Reuse its lifecycle and download primitives without reusing its HTTP tool-execution protocol.
- The official Desert Ant command-line target does not expose Tongue, Redact, or Title as runnable commands. ACORN therefore needs owned, thin SDK adapters rather than shelling out to an undocumented CLI contract.
- [Desert Ant Core v3.3.0](https://github.com/Desert-Ant-Labs/desert-ant-core/tree/0711f8907188ca1ee85b2a0fa37ad7aa64b9a750) is the integration baseline. Its source-available terms and every bundled or downloaded model license require explicit redistribution approval before release.
- [Tongue](https://github.com/Desert-Ant-Labs/desert-ant-core/blob/0711f8907188ca1ee85b2a0fa37ad7aa64b9a750/docs/models/tongue.md) accepts at most 512 characters and returns language reliability. Pin model revision [`fd7e0dd2336c2a9f222dd4c0e5ac235d0edb708d`](https://huggingface.co/desert-ant-labs/tongue/tree/fd7e0dd2336c2a9f222dd4c0e5ac235d0edb708d) for evaluation.
- [Redact](https://github.com/Desert-Ant-Labs/desert-ant-core/blob/0711f8907188ca1ee85b2a0fa37ad7aa64b9a750/docs/models/redact.md) combines learned spans and structured identifier rules. Its documented defaults include a 0.6 minimum score, 256-token windows, and a 64-token stride; pin model revision [`3ed5033aeb544f454ebde77dc76fd7b7e76cc572`](https://huggingface.co/desert-ant-labs/redact/tree/3ed5033aeb544f454ebde77dc76fd7b7e76cc572).
- [Title](https://github.com/Desert-Ant-Labs/desert-ant-core/blob/0711f8907188ca1ee85b2a0fa37ad7aa64b9a750/docs/models/title.md) is Apple/MLX-only and lacks an independent published quality evaluation. Pin model revision [`72375893bb8d1b96f50c7f32dbd1bc124f20d0e6`](https://huggingface.co/desert-ant-labs/title/tree/72375893bb8d1b96f50c7f32dbd1bc124f20d0e6). Its generated description is not inherently a valid ACORN subtitle.
- The RAD schema validates title and subtitle; subtitle currently has a 75-character maximum. Sidecar output must use canonical validators rather than copy those limits.
- The macOS GitLab runner remains blocked and cannot provide a reliable host gate. `scripts/slm/check-macos-host.sh` can verify macOS ARM64, Xcode 26, Swift 6.2 or newer, and native Swift compilation manually. A schedulable macOS host and legal approval for SDK and weight redistribution remain hard prerequisites for release implementation.
- Fastino GLiDE provides typed, probabilistic decisions over caller-defined choices through the hosted `fastino/glide` System One API. Its decision contract is relevant to the retained semantic-decision experiment, but the documented integration is hosted rather than an ACORN-owned offline runner. GLiDE therefore does not replace Needle's local tool-selection loop; it is a candidate policy/approval decision adapter that requires a separate offline or approved self-hosting path before production consideration. [GLiDE announcement](https://fastino.ai/blog/introducing-glide-the-first-thinking-decision-model)

# Decisions locked for the first release

- **Capabilities:** Tongue, Redact, and Title only. Schemer remains visibly unavailable until a supported public release, pinned harness, and shared extraction contract exist.
- **Runtime ownership:** ACORN maintains one pinned Swift package with a separate executable for each capability. Do not depend on an unversioned upstream command-line interface.
- **Platforms:** Tongue and Redact target Linux x86_64, Linux ARM64, and macOS ARM64. Title targets macOS ARM64 only. Windows is unsupported in this slice and returns an actionable capability error.
- **Stability:** all three commands remain unreleased until their functional, quality, licensing, packaging, and platform gates pass. Do not label only a subset experimental to work around a missing gate.
- **Invocation:** inference and download are explicit. There is no automatic model download, background inference, remote fallback, record mutation, or default integration with check, gather, export, or validation.
- **Licensing:** legal/redistribution approval for the pinned SDK and weights is a hard release gate, not a documentation follow-up.
- **GLiDE placement:** evaluate GLiDE only as a deferred semantic-decision sidecar or comparison adapter. Do not include it in the first Tongue/Redact/Title release, expose it as a generic provider, or present it as a Needle replacement. Needle remains the bounded local tool selector; GLiDE would make task-specific decisions over predefined options.

# Exact implementation boundaries

## Owned runner and artifact files

- **Create:** `scripts/slm/runner/Package.swift`, `scripts/slm/runner/Package.resolved`, shared protocol/process helpers under `scripts/slm/runner/Sources/SlmProtocol/`, and `TongueRunner`, `RedactRunner`, and `TitleRunner` executable targets.
- **Create:** `scripts/slm/create-oci-artifact.sh` and `scripts/slm/upload-runners.sh` to build one capability/platform artifact at a time and emit its immutable URL, byte size, and SHA-256.
- **Update:** `.gitlab-ci.yml` and `Makefile` through the CI/build integration lock. Linux jobs build and smoke-test only supported target/capability pairs. Keep macOS validation manual until a runner is proven schedulable; a passing macOS gate is required before release.
- Build and publish runner/model artifacts first. Pin the resulting immutable URLs, sizes, and hashes in a second Rust integration change. Rust source must not predict release checksums.

## Rust and CLI files

- **Update:** `crates/acorn-lib/Cargo.toml` and `crates/acorn-lib/src/io/mod.rs` to add an opt-in `slm` feature and expose the existing sidecar module for either `needle` or `slm`. Extend `crates/acorn-lib/src/io/sidecar/mod.rs` only when a second concrete implementation proves another lifecycle primitive is shared.
- **Create:** `crates/acorn-lib/src/io/sidecar/slm/{mod,assets,process,protocol,language,redact,title}.rs` with concrete `LanguageSidecar`, `RedactSidecar`, and `TitleSidecar` implementations. Each sidecar implements `SidecarAssets` for acquisition and `Sidecar` for its typed request, result, and process lifecycle. Keep capability-specific result types and validation explicit; do not introduce a registry or erase them behind `serde_json::Value`.
- **Update:** the existing CLI inference/download command modules, CLI argument definitions, and dispatch roots to add the commands below. Regenerate `CommandLineHelp.md` once under the CLI integration lock.
- **Create:** `tests/fixtures/slm/{language,redact,title}/` and `docs/src/inference/slm.md`; update `docs/src/SUMMARY.md`.
- **Delete:** none.

# Runner protocol and resource contract

Each executable writes one JSON-lines readiness envelope, reads exactly one typed request from stdin, writes one typed response to stdout, and exits. Diagnostics go to stderr. Every envelope includes protocol version, capability, and request ID. Responses distinguish success, unsupported input, validation failure, model failure, and timeout; error text must not echo sensitive source content.

Set `DAL_USAGE_DISABLED=1` for every managed process and verify offline execution. Use isolated temporary/session directories, bounded startup and request timers, cancellation, process-tree cleanup, and a 1 MiB maximum response. The initial input budgets are:

| Capability | Input limit | Request timeout |
|---|---:|---:|
| Tongue | 512 Unicode scalar values | 10 seconds |
| Redact | 64 KiB UTF-8 | 60 seconds |
| Title | 16 KiB UTF-8 | 120 seconds |

`LanguageResult` preserves the detected tag, upstream reliability, and an explicit uncertain/abstained state. `RedactResult` preserves validated, non-overlapping UTF-8 source spans, labels, scores where provided, and an optional copy produced only when requested. `TitleResult` preserves the proposed title, optional subtitle, validation findings, model/prompt revisions, and source fingerprint. Unsupported, insufficient-evidence, and technical-failure outcomes stay distinct.

# CLI contract

Expose explicit subcommands:

```text
acorn infer needle <PROMPT> [legacy Needle options]
acorn infer language [TEXT]
acorn infer redact [TEXT] [redaction options]
acorn infer title [TEXT]
acorn download slm <language|redact|title> [--status]
```

Language, redact, and title accept one positional value or stdin, never both. Redact flags select report-only versus copy output and documented thresholds; report-only is the default. Each command accepts the existing custom sidecar location mechanism. There is intentionally no `download slm all` in the first release.

Preserve `acorn infer <PROMPT>` as the legacy Needle form. Because `needle`, `language`, `redact`, and `title` become reserved first tokens, prompts beginning with one of them use `acorn infer needle -- <PROMPT>`. Cover this compatibility rule in parser and generated-help tests.

# Implementation sequence and exit gates

1. **Prove licensing and hosts.** Provision a schedulable macOS ARM64 host with Xcode 26 and Swift 6.2 or newer, record one passing host check, and obtain approval for SDK/weight redistribution before runner release work proceeds; retain local evaluation notes if either gate fails.
2. **Build the runners.** Implement `SidecarAssets` and `Sidecar` for every capability, package every runner and supporting file through the existing semantic-versioned manifest, pin the SDK and model commits above, implement the one-request protocol, and add protocol-only fixture tests plus real-model smoke tests on every claimed platform.
3. **Publish immutable artifacts.** Produce capability/platform artifacts, capture exact bytes/hashes/licenses, verify clean offline installation, then pin them in Rust.
4. **Integrate the library.** Add concrete sidecars, typed results, limits, telemetry opt-out, asset/download behavior, and lifecycle tests behind `slm`.
5. **Integrate the CLI and docs.** Add the explicit inference/download commands, preserve legacy Needle parsing, regenerate help, and document unsupported targets and limitations.
6. **Run held-out evaluation.** Freeze fixtures and thresholds before scoring. Release all three commands as stable only when every gate below passes.

# Quality and release gates

- **Tongue:** at least 95% top-1 accuracy on the frozen unambiguous ACORN prose set; every predesignated mixed-language, acronym-only, or otherwise ambiguous case abstains instead of asserting a language.
- **Redact:** at least 85% macro recall across the frozen sensitive-label set and at least 95% precision. Documentation states that a clean result does not certify absence of sensitive material.
- **Title:** at least 90% blinded reviewer acceptance, zero unsupported scientific claims in the frozen evaluation set, and at least 95% canonical title/subtitle validation compliance. Invalid subtitles are omitted with findings; do not truncate or silently rewrite them.
- **All capabilities:** immutable reproducible artifacts, approved licenses/notices, clean offline execution, no unexpected network calls, bounded lifecycle behavior, and passing real-model smoke tests on every advertised platform.

# Deferred retained scope

- **Schemer:** wait for a supported release and implement it only through [deterministic and assisted gather extraction](gather-extraction.md), preserving source offsets, typed absence, conflicting candidates, and deterministic validation.
- **Persistence/application:** proposal-only output may precede persistence. Durable enrichment proposals and reconciliation belong to the canonical database boundary; authored RAD values always win unless a user explicitly accepts a validated proposal.
- **Semantic decisions:** SemIf/Open-Jev/Needle comparison remains an opt-in experiment over frozen source/proposal fixtures. It must predeclare accuracy, unsupported-claim, false-rejection, abstention, calibration, latency, memory, and installed-size gates. Scores are advisory and conditional on supplied options. Promotion would add one task-specific adapter; it cannot authorize mutation, establish factuality, or block the first three sidecars.
- **GLiDE decision adapter:** include Fastino GLiDE in the semantic-decision comparison only after verifying an immutable revision, licensing/redistribution terms, data-handling requirements, and an offline or approved self-hosted deployment path. Compare its typed choice, confidence, probability, abstention, latency, memory, and failure behavior against SemIf, Open-Jev, and the existing Needle baseline. A successful evaluation may justify a separate policy/approval sidecar; it must not replace Needle's tool-selection protocol or broaden tool execution authority.
- **Out of scope:** automatic enrichment, generic providers, remote fallback, browser integration, audio/vision, training, and Windows runners.

# Julia-1 decision sidecar (planned follow-on)

This slice was added on 2026-10-06 and remains **planned**; the parent status reflects the existing shared sidecar work. Its outcome is an explicitly invoked, advisory finite-choice decision capability. It does not block Tongue, Redact, or Title and does not inherit their Swift runtime or their all-three release gate. Keep this work in the sidecar concept rather than creating an overlapping model-provider plan.

## Evidence and intended use

- [Supersonic Labs' introduction](https://supersoniclabs.ai/julia-1/) describes Julia-1 as a CPU-capable, 144.3M-parameter decision model with 2–20 supplied answers. The launch evaluation uses a 1,024-token budget and reports weaknesses when shortlisting large label sets. These results do not establish ACORN task quality.
- The [current model card](https://huggingface.co/SupersonicLabs/Julia-1) specifies Python 3.11+, the custom Julia runtime, Apache-2.0 artifacts, and 550.5 MiB weights. It documents named `choice`, `score`, and `noul` questions, strict encoding, and an 8,192-token runtime limit; 8k task accuracy is unproven. `score` is an expected rubric index, and `noul` is the probability of true. The legacy list API rounds probabilities. A separate ONNX/WebGPU export is linked. Pin and inspect runtime source before relying on these moving interfaces.
- Local evidence rechecked on 2026-10-06: `crates/acorn-lib/src/io/sidecar/mod.rs` exposes `SidecarAssets`, `SidecarDistribution`, and consuming `Sidecar::infer`, with an associated `ToolIndex`. Only the Needle child module exists. Julia requires a new concrete adapter; none is shipped. Existing ModelKit resolution verifies platform, protocol, paths, sizes, and hashes.

The first consumer is explicit research-text triage: given a supplied abstract and a small caller-defined topic rubric, return an advisory category or abstention for human review. This is a proposed pilot, not an established user requirement or automatic RAD field mapping. Do not use the result to validate factual claims, approve tools, modify records, or replace deterministic schema validation. A maintained ACORN corpus and a named evaluation owner are prerequisites for promotion.

## Contract and architectural decisions

- Propose task-named `acorn infer decide --input <request.json>` (stdin when omitted) and `acorn download slm decide [--status]`. Freeze syntax under the CLI lock; preserve the legacy Needle escape for the newly reserved `decide` token. Use the existing custom-location mechanism. Download is explicit; inference requires verified local assets and cannot silently acquire missing files.
- Implement a concrete `JuliaSidecar` behind the opt-in host `slm` feature. Preserve `SupersonicLabs/Julia-1`, runtime/tokenizer revisions, artifact hashes, and adapter protocol identity in provenance. Do not send Julia through a chat-completions provider, GGUF loader, or Needle tool loop.
- Include ONNX Runtime CPU inference in this slice behind the ACORN-owned launcher and bounded JSON-lines protocol. Retain pinned upstream Python/PyTorch as the reference backend for parity testing. Initially use Python ONNX Runtime bindings to keep the existing runner boundary; a native Rust runner is a later packaging decision. Package a locked runtime and all dependencies; never run pip or fetch tokenizer files during inference. ONNX becomes the default only after CPU compatibility, parity, packaging, and quality gates pass; a Python-only result does not complete the ONNX scope.
- Reuse `SidecarAssets` and `SidecarDistribution`; resolve the required `ToolIndex` with a documented no-op implementation or a narrowly scoped shared-contract change approved under the library-root lock. Do not create a fake tool catalog. Avoid a generic registry or broad sidecar refactor.
- One process loads once, handles one bounded request containing multiple independent questions, then exits, matching consuming `infer`. Persistent sessions require a separate measured justification; warm upstream throughput is not a claim about this cold-start lifecycle.
- Versioned input contains a request ID, source text, and ordered questions with stable unique IDs. Model questions and answers as tagged Rust enums with `Choice` and `Noul` variants, serialized as `choice` and `noul`. Choice accepts 2–20 unique option IDs and nonempty descriptions. Preserve caller order in an explicit sequence; reject duplicate IDs and unknown fields. Typed `score` remains deferred.
- Return the selected caller ID, ordered full-precision probabilities, explicit abstention reason, and model/runtime/prompt provenance plus source fingerprint. Reject mismatched IDs, missing/extra probabilities, non-finite or out-of-range values, and distributions outside a documented sum tolerance. Use the named API or verified logits path, not rounded legacy output. Scores are conditional model estimates, not calibrated certainty.
- ACORN owns abstention using thresholds frozen on development data; distinguish low confidence/insufficient evidence from invalid input and technical failure. Never turn a crash or timeout into a successful decision. Disable hierarchical routing: more than 20 options is an input error.
- Proposed pilot budgets: 64 KiB UTF-8 per request, at most 16 questions, 1,024 combined tokens per encoded question, 120-second startup, 60-second inference, and 1 MiB response. Enforce strict tokenizer limits without silent truncation; measure and freeze question/option sub-budgets against the pinned runtime. Larger context is a later evaluated change.
- Keep diagnostics off stdout and redact source text from errors. Verify network-free execution, timeout/cancellation, child-process cleanup, and bounded output. Use runtime-appropriate offline controls; the Desert Ant-specific `DAL_USAGE_DISABLED` flag is not evidence of Julia offline behavior.

## Noul type modeling

Treat `Noul` as a distinct probabilistic Boolean result, not an alias for `bool` or a nullable choice. Proposed sidecar-local types are `NoulCriteria`, `NoulQuestion`, `Noul`, and `NoulOutcome`; keep them in the decision module until a second consumer justifies a portable shared type.

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

The [model acquisition plan](model-acquisition-hardening.md) owns shared Hugging Face/OCI/materialization changes; consume its landed contracts and coordinate any missing capability there. Workspace feature changes follow [dependency capabilities](../refactors/portability/dependency-features-and-macos-size.md). The [Agent tools lane](../parallel-execution.md) serializes shared sidecar, CLI, artifact, and CI/build changes with Needle and the other SLM adapters. No dependency on Title's macOS runner is imposed on the Linux-only Julia pilot.

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

# Validation

- Compile-time contract tests assert that every concrete adapter declares typed input/output, stable identity/protocol values, asset resolution, verified-asset startup, and required inference.
- Model-independent fixtures cover readiness and typed envelopes, malformed/oversized responses, request ID mismatch, Unicode and byte limits, timeouts, cancellation, concurrent sessions, startup failure, cleanup, stderr separation, and redacted diagnostics.
- Asset tests cover the supported semantic manifest version, unsupported and legacy shapes, duplicate or unsafe asset names, bad sizes/digests, partial downloads, unsupported targets, custom locations, offline cache hits/misses, and OCI/local parity.
- Capability fixtures cover scientific abbreviations, ambiguous and mixed-language prose, expected contact PII, missed and false-positive redactions, UTF-8 span boundaries, malformed Title output, absent fields, subtitle limits, unsupported claims, and stale source fingerprints.
- Measure cold/warm latency, total download, peak memory, and batch behavior on every claimed platform. Keep heavyweight real-model tests explicit and separate from ordinary offline unit tests.
- Run formatting, lint, focused tests, `make test`, applicable feature/WASM checks, license/dependency checks, generated-help checks, Linux CI smoke jobs, and manual macOS host and real-model checks for implementation changes.

# Acceptance criteria

- [ ] Pinned, ACORN-owned Tongue, Redact, and Title runners implement the bounded one-request protocol and run offline on every advertised target.
- [ ] Explicit library and CLI paths return typed results without using Needle's tool loop, mutating RAD records, or downloading during unrelated commands.
- [ ] Legacy Needle inference parsing remains supported with a documented escape for reserved first tokens.
- [ ] Tongue, Redact, and Title meet their frozen numeric quality gates; all three also pass license, artifact, lifecycle, documentation, and platform gates before stable release.
- [ ] Title is clearly macOS ARM64-only, Windows is clearly unsupported, and unavailable capabilities fail with actionable errors rather than remote fallback.
- [ ] Schemer, persistence, and semantic-decision work remain gated follow-ons and are not represented as shipped.
