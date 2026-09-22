---
type: Plan
title: "Small language model (SLM) sidecars"
description: "Add optional language and enrichment sidecars alongside Needle, with a gated semantic-decision evaluation using SemIf and comparison candidates."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [feature, inference, sidecar, slm, enrichment, privacy, semantic-decision]
status: planned
---

# Goal

Add explicitly selected local small language model (SLM) sidecars alongside Needle: Tongue for language detection, Redact for sensitive-content findings, Title for title/subtitle enrichment, and Schemer for source-backed structured extraction when a supported release is available. Download and start only the selected capability. Preserve deterministic validation and authored RAD values.

Use SLM as the vendor-neutral integration category for these compact language-processing models, including classifiers and extractors as well as generative models. Name ACORN modules, configuration, documentation, and user-facing capabilities by task or SLM rather than vendor. Preserve exact upstream identities in adapters, artifact provenance, citations, and license notices. The initial model selection remains unchanged; this naming decision does not require a generic provider registry.

Evaluate an optional semantic-decision capability for source-grounded enrichment review. Start with SemIf and compare Open-Jev and Needle 3 categorical output on the same RAD fixtures. This experiment does not block the initial sidecars or commit ACORN to shipping both projects. Promote a runtime adapter only after its quality, calibration, resource, and platform gates pass.

# Current evidence

Evidence checked on 2026-09-22; upstream availability and artifact revisions must be rechecked at implementation.

- `crates/acorn-lib/src/io/sidecar/mod.rs` exposes a `Sized` lifecycle trait with an associated output and consuming string-input inference method. It is not a general typed provider registry. Its current manifest validates a runner and optional tools, but does not inventory all model assets.
- `io/sidecar/needle/` already owns managed process startup, asset resolution, and tool dispatch. Reuse proven lifecycle/download primitives without sending analyzer or enrichment requests through the tool-execution loop.
- The RAD schema in `crates/acorn-schema/src/research_activity/mod.rs` validates title and subtitle; subtitle currently has a 75-character maximum. Use canonical validators rather than duplicating limits.
- [Tongue](https://huggingface.co/desert-ant-labs/tongue) is a roughly 2 MB language detector with reliability output and a 512-character input limit. Mixed languages and acronym-heavy input need explicit unknown handling.
- [Redact](https://huggingface.co/desert-ant-labs/redact) combines learned spans with structured-identifier rules. Published recall is imperfect; a clean result cannot certify absence of sensitive content.
- [Title](https://huggingface.co/desert-ant-labs/title) currently ships for Apple silicon through MLX, uses a fixed trained prompt, and returns a short title plus a longer description. Its card has no completed independent quality evaluation. Its description is not inherently a valid ACORN subtitle.
- [Schemer](https://huggingface.co/desert-ant-labs/schemer) is private pre-release. It extracts typed values and source spans; schema and input share a bounded context. Its SDK/harness availability is a release gate.
- The [SDK](https://github.com/Desert-Ant-Labs/desert-ant-core) offers Swift, Kotlin, and JavaScript integrations, not a documented Rust SDK. Platform support varies by model and runtime; browser support does not establish a native Windows runner.
- The [license notice](https://raw.githubusercontent.com/Desert-Ant-Labs/desert-ant-core/main/LICENSE.md) requires attribution and states usage limits and restrictions on training competing models from models, outputs, or logs. Record full authoritative terms and redistribution requirements before packaging; do not assume MIT or Apache terms apply.
- [SemIf](https://github.com/TheoLeeCJ/SemIf) scores supplied alternatives directly with existing models and documents CPU/GGUF, Apple, and GPU paths plus workload-specific calibration. It is an independent project formerly named OpenJev, distinct from the Open-Jev repository below. Model footprints and optimized-path numerical differences require measurement; it is not a small standalone weight artifact.
- [Open-Jev](https://github.com/Zefan-Cai/Open-Jev) provides typed decision serving and released adapters with a trained scalar head and calibration data. Its loader also requires pinned base weights; its published GPU workflow targets Linux. Prefix caching remains experimental. Neither project's published results establish correctness or calibrated confidence for research metadata.

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/sidecar/mod.rs`; focused analyzer/discovery consumers under `crates/acorn-lib/src/analyzer/host/`; `crates/acorn-cli/src/commands/{download,infer}/`; CLI definitions and generated help; relevant manifests; `docs/src/SUMMARY.md`. Canonical enrichment persistence consumes the canonical database interfaces rather than adding a competing store.
- **Create:** `crates/acorn-lib/src/io/sidecar/slm/` with typed request/response, asset, process, and per-capability modules/tests; `scripts/slm/` for the minimal pinned SDK-backed runner and packaging; dedicated `tests/fixtures/slm/`; `docs/src/inference/slm.md`.
- **Semantic-decision evaluation boundary:** Create dedicated fixtures under `tests/fixtures/slm/semantic-decision/` and an opt-in comparison harness under `scripts/slm/`. Keep experimental dependencies isolated from normal builds. Only a successful promotion gate authorizes a task-specific adapter within `io/sidecar/slm/` and focused enrichment-review integration; it does not authorize new search services or tool execution.
- **Delete:** None.

Before implementation, expand these directory boundaries into exact files for each slice. Shared locks include `io/sidecar/mod.rs`, CLI roots, manifests/lockfiles, book index/help, and any analyzer or database integration files.

# Dependencies and ownership

- Lane: Agent tools and sidecars in the [parallel execution guide](../parallel-execution.md); model evaluation can precede integration, but shared writes follow the existing highest-priority lane.
- Schedule the shared asset-contract slice of [Needle 3 migration](../migrations/needle-3-upgrade.md) first, then reuse its reviewed multi-file asset validation where appropriate. Full Needle 3 benchmark completion is not a prerequisite for independent SLM evaluation.
- Semantic-decision fixtures and SemIf/Open-Jev experiments can run independently. The Needle 3 comparison requires its pinned runnable baseline; production enrichment review consumes the proposal boundary in this plan. Semantic content checks and discovery ranking remain follow-on applications, not additional delivery requirements.
- [Structured extraction inference](structured-extraction-inference.md) owns the versioned extraction request, candidate validation, provenance, and gather merge contract. Schemer implements an adapter to that contract; this plan must not duplicate it. Gather use also follows that plan's deterministic scraping prerequisite.
- The canonical research activity database boundary in `crates/acorn-lib/src/io/database/` owns durable enrichment proposals and reconciliation. Evaluation and proposal-only output can ship before persistence integration.
- Coordinate packaging with [OCI interoperability](oci-model-artifact-downloads.md). Explicit sidecar assets are not generic GGUF inference inputs; do not weaken that plan's GGUF selection policy to load arbitrary model files.

# Recommendation and implementation slices

1. Establish a per-model support and licensing matrix. Pin SDK, runtime, model, tokenizer, harness, and prompt revisions; record every required file, checksum, download size, and measured resident memory. Prototype real executable runners on supported hosts. Title begins on Apple silicon; unsupported platforms return a clear capability error without remote fallback. Keep Schemer marked unavailable until a supported harness can be packaged and tested.
2. Add one minimal SDK-backed subprocess adapter with typed task envelopes and bounded input/output. Choose stdio or private loopback after inspecting the actual SDK/CLI contract; do not invent an upstream wire API. Include readiness, timeouts, cancellation, process-tree cleanup, isolated sessions, concurrency limits, and redacted diagnostics. Prefer explicit adapters over changing the current trait into a speculative dynamic registry.
3. Add Tongue and Redact. Detect language from prose windows, preserve uncertainty, and let explicit language metadata win. Redact reports spans and labels or produces an explicitly requested redacted copy. Contact fields are expected to contain names/emails: distinguish field intent and never silently redact canonical RAD.
4. Add Title enrichment. Construct bounded source text from mission and relevant descriptive sections, retaining source identity. Use the pinned trained prompt/template and parse `TITLE`/`DESC` tolerantly while rejecting incomplete or unusable outputs. Validate proposed titles and subtitles with RAD validators. Reject an overlong description as a subtitle; do not truncate blindly or add an untested rewrite pass. Return a partial proposal when only the title is valid.
5. Integrate proposals into the enrichment workflow. Missing or known-placeholder fields are eligible for explicit fill-missing application; existing authored text receives reviewable alternatives. Record source fingerprint/fields, model and prompt revisions, generation settings, validation, and accepted/rejected status. Recheck the source fingerprint before applying. Insufficient evidence leaves the field unresolved. Generated language must not invent scientific claims, expand ambiguous acronyms, or remove essential qualifiers.
6. Add Schemer once its release gate passes, using the extraction plan's contract. Enforce schema-plus-document limits, preserve source offsets, and validate extracted values and absence handling before any merge. Chunking must preserve attribution and expose conflicting candidates. Compare against Needle 3 on identical RAD fixtures before selecting defaults.
7. Expose explicit capability selection, download/status reporting, and diagnostics through existing CLI conventions. Final flag names follow the CLI integration review. No model download or inference occurs merely because a normal check/gather command runs. Preseeded assets support offline use; missing assets fail without network. Gist, audio, vision, training, and browser integration are outside this plan.

## Optional semantic-decision evaluation

1. Freeze human-labeled source/proposal pairs for title/subtitle review before comparing models. Include supported, unsupported, and insufficient-evidence alternatives. Test whether a proposed subtitle changes planned work into completed work, exaggerates maturity, invents a capability, or expands an ambiguous acronym. Keep calibration and held-out evaluation records separate. Review output-use license restrictions before building or redistributing fixtures from generated text; no model training is added by this slice.
2. Define a bounded task-specific request carrying source text and fingerprint, proposed text, criterion revision, and stable option IDs/descriptions. Preserve model/runtime revisions, quantization, prompt hash, raw scores, calibration identity when available, selected alternative, and abstention in results. Model scores are conditional on supplied options, not verified truth probabilities. Do not invent evidence spans or explanations when a backend returns only scores.
3. Evaluate SemIf first, Open-Jev as a trained comparison, and Needle 3 with equivalent categorical choices. Keep categorical accuracy comparable while reporting probability/calibration metrics only for backends that actually expose suitable scores; Needle's execution confidence is not automatically an option distribution. Pin all base weights, adapters, heads, tokenizers, and runtime assets, and verify their individual licenses. Disable experimental prefix reuse initially; enable it only after measured decision and probability parity tests pass.
4. Measure value against Title proposals without a semantic reviewer. Predeclare numeric gates for unsupported-claim detection, false rejection, abstention coverage, calibration, human reviewer acceptance, cold/warm latency, peak RAM, and total installed bytes on each claimed platform. Include negation, scientific terminology, option-order permutations, paraphrased criteria, distractors, and multilingual inputs where support is claimed. A larger model's success does not establish a viable compact deployment.
5. Record a promote/defer/reject decision with evidence in durable inference documentation. If promoted, ship one explicitly selected semantic-decision adapter using the existing bounded lifecycle and offline policies. Return advisory review findings; keep deterministic RAD validators and existing application policy authoritative. Agreement with Title cannot silently overwrite authored metadata, prove factuality, or authorize a tool call. A deferred experiment leaves the initial sidecars deliverable.

# Validation

- Fixture tests cover language ambiguity, scientific abbreviations, mixed-language prose, expected contact PII, missed/false-positive redactions, absent fields, malformed output, source offsets, title/subtitle limits, Unicode, unsupported claims, and stale proposals.
- Lifecycle tests cover bad checksums, missing supporting files, unsupported platforms, malformed/oversized responses, startup failure, timeout, cancellation, concurrent sessions, cleanup, and no network in offline mode.
- Run held-out RAD evaluations per capability: language accuracy/abstention; redaction precision/recall by label; Title factuality, specificity, reviewer acceptance, and length compliance; Schemer field correctness, unsupported-value rate, and provenance correctness. Define numeric deployment thresholds before scoring and retain results in durable documentation. Do not claim benchmark parity from unrelated upstream datasets.
- For semantic decisions, validate option identity, finite normalized probabilities where applicable, malformed/overlength requests, absent evidence, stale source fingerprints, and backend failures. Preserve insufficient evidence and technical failure as distinct outcomes. Verify that advisory results cannot bypass deterministic validation or mutate records. Report held-out calibration and option-order sensitivity, and test reviewer benefit against the no-reviewer baseline before promotion.
- Measure cold/warm latency, total download, peak memory, and batch behavior on each claimed platform. Keep heavyweight real-model tests explicit and separate from ordinary offline unit tests.
- Run formatting, lint, focused tests, `make test`, applicable feature/WASM checks, license/dependency checks, and generated-help validation for implementation changes.

# Acceptance criteria

- [ ] Tongue, Redact, and Title have explicit supported-platform sidecar paths with reproducible assets; Schemer remains visibly release-gated until its runnable adapter is validated.
- [ ] Selected capabilities run locally, independently of Needle's tool-execution loop, with bounded resources and tested offline behavior.
- [ ] Title produces validated title/subtitle proposals and preserves authored values, source evidence, and application policy; overlong descriptions cannot silently become truncated subtitles.
- [ ] Language uncertainty and redaction limitations remain visible; neither model silently rewrites canonical records.
- [ ] Schemer integration consumes the existing extraction contract and passes shared RAD comparisons when available.
- [ ] If the optional semantic-decision slice is undertaken, its comparison and promote/defer/reject outcome are documented against predeclared gates. Only a passing candidate becomes a shipped adapter; initial delivery does not require SemIf or Open-Jev installation.
- [ ] License/attribution, platform matrix, evaluation results, packaging, and operational instructions are documented. Unreleased or unsupported capabilities are not presented as shipped.
