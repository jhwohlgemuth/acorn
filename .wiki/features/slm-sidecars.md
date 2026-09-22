---
type: Plan
title: "Small language model (SLM) sidecars"
description: "Add optional Tongue, Redact, Title, and release-gated Schemer sidecars with validated enrichment proposals alongside Needle."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [feature, inference, sidecar, slm, enrichment, privacy]
status: planned
---

# Goal

Add explicitly selected local small language model (SLM) sidecars alongside Needle: Tongue for language detection, Redact for sensitive-content findings, Title for title/subtitle enrichment, and Schemer for source-backed structured extraction when a supported release is available. Download and start only the selected capability. Preserve deterministic validation and authored RAD values.

Use SLM as the vendor-neutral integration category for these compact language-processing models, including classifiers and extractors as well as generative models. Name ACORN modules, configuration, documentation, and user-facing capabilities by task or SLM rather than vendor. Preserve exact upstream identities in adapters, artifact provenance, citations, and license notices. The initial model selection remains unchanged; this naming decision does not require a generic provider registry.

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

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/sidecar/mod.rs`; focused analyzer/discovery consumers under `crates/acorn-lib/src/analyzer/host/`; `crates/acorn-cli/src/commands/{download,infer}/`; CLI definitions and generated help; relevant manifests; `docs/src/SUMMARY.md`. Canonical enrichment persistence consumes the database plan's interfaces rather than adding a competing store.
- **Create:** `crates/acorn-lib/src/io/sidecar/slm/` with typed request/response, asset, process, and per-capability modules/tests; `scripts/slm/` for the minimal pinned SDK-backed runner and packaging; dedicated `tests/fixtures/slm/`; `docs/src/inference/slm.md`.
- **Delete:** None.

Before implementation, expand these directory boundaries into exact files for each slice. Shared locks include `io/sidecar/mod.rs`, CLI roots, manifests/lockfiles, book index/help, and any analyzer or database integration files.

# Dependencies and ownership

- Lane: Agent tools and sidecars in the [parallel execution guide](../parallel-execution.md); model evaluation can precede integration, but shared writes follow the existing highest-priority lane.
- Schedule the shared asset-contract slice of [Needle 3 migration](../migrations/needle-3-upgrade.md) first, then reuse its reviewed multi-file asset validation where appropriate. Full Needle 3 benchmark completion is not a prerequisite for independent SLM evaluation.
- [Structured extraction inference](structured-extraction-inference.md) owns the versioned extraction request, candidate validation, provenance, and gather merge contract. Schemer implements an adapter to that contract; this plan must not duplicate it. Gather use also follows that plan's deterministic scraping prerequisite.
- [Canonical research activity database](research-activity-database.md) owns durable enrichment proposals and reconciliation. Evaluation and proposal-only output can ship before persistence integration.
- Coordinate packaging with [OCI interoperability](oci-model-artifact-downloads.md). Explicit sidecar assets are not generic GGUF inference inputs; do not weaken that plan's GGUF selection policy to load arbitrary model files.

# Recommendation and implementation slices

1. Establish a per-model support and licensing matrix. Pin SDK, runtime, model, tokenizer, harness, and prompt revisions; record every required file, checksum, download size, and measured resident memory. Prototype real executable runners on supported hosts. Title begins on Apple silicon; unsupported platforms return a clear capability error without remote fallback. Keep Schemer marked unavailable until a supported harness can be packaged and tested.
2. Add one minimal SDK-backed subprocess adapter with typed task envelopes and bounded input/output. Choose stdio or private loopback after inspecting the actual SDK/CLI contract; do not invent an upstream wire API. Include readiness, timeouts, cancellation, process-tree cleanup, isolated sessions, concurrency limits, and redacted diagnostics. Prefer explicit adapters over changing the current trait into a speculative dynamic registry.
3. Add Tongue and Redact. Detect language from prose windows, preserve uncertainty, and let explicit language metadata win. Redact reports spans and labels or produces an explicitly requested redacted copy. Contact fields are expected to contain names/emails: distinguish field intent and never silently redact canonical RAD.
4. Add Title enrichment. Construct bounded source text from mission and relevant descriptive sections, retaining source identity. Use the pinned trained prompt/template and parse `TITLE`/`DESC` tolerantly while rejecting incomplete or unusable outputs. Validate proposed titles and subtitles with RAD validators. Reject an overlong description as a subtitle; do not truncate blindly or add an untested rewrite pass. Return a partial proposal when only the title is valid.
5. Integrate proposals into the enrichment workflow. Missing or known-placeholder fields are eligible for explicit fill-missing application; existing authored text receives reviewable alternatives. Record source fingerprint/fields, model and prompt revisions, generation settings, validation, and accepted/rejected status. Recheck the source fingerprint before applying. Insufficient evidence leaves the field unresolved. Generated language must not invent scientific claims, expand ambiguous acronyms, or remove essential qualifiers.
6. Add Schemer once its release gate passes, using the extraction plan's contract. Enforce schema-plus-document limits, preserve source offsets, and validate extracted values and absence handling before any merge. Chunking must preserve attribution and expose conflicting candidates. Compare against Needle 3 on identical RAD fixtures before selecting defaults.
7. Expose explicit capability selection, download/status reporting, and diagnostics through existing CLI conventions. Final flag names follow the CLI integration review. No model download or inference occurs merely because a normal check/gather command runs. Preseeded assets support offline use; missing assets fail without network. Gist, audio, vision, training, and browser integration are outside this plan.

# Validation

- Fixture tests cover language ambiguity, scientific abbreviations, mixed-language prose, expected contact PII, missed/false-positive redactions, absent fields, malformed output, source offsets, title/subtitle limits, Unicode, unsupported claims, and stale proposals.
- Lifecycle tests cover bad checksums, missing supporting files, unsupported platforms, malformed/oversized responses, startup failure, timeout, cancellation, concurrent sessions, cleanup, and no network in offline mode.
- Run held-out RAD evaluations per capability: language accuracy/abstention; redaction precision/recall by label; Title factuality, specificity, reviewer acceptance, and length compliance; Schemer field correctness, unsupported-value rate, and provenance correctness. Define numeric deployment thresholds before scoring and retain results in durable documentation. Do not claim benchmark parity from unrelated upstream datasets.
- Measure cold/warm latency, total download, peak memory, and batch behavior on each claimed platform. Keep heavyweight real-model tests explicit and separate from ordinary offline unit tests.
- Run formatting, lint, focused tests, `make test`, applicable feature/WASM checks, license/dependency checks, and generated-help validation for implementation changes.

# Acceptance criteria

- [ ] Tongue, Redact, and Title have explicit supported-platform sidecar paths with reproducible assets; Schemer remains visibly release-gated until its runnable adapter is validated.
- [ ] Selected capabilities run locally, independently of Needle's tool-execution loop, with bounded resources and tested offline behavior.
- [ ] Title produces validated title/subtitle proposals and preserves authored values, source evidence, and application policy; overlong descriptions cannot silently become truncated subtitles.
- [ ] Language uncertainty and redaction limitations remain visible; neither model silently rewrites canonical records.
- [ ] Schemer integration consumes the existing extraction contract and passes shared RAD comparisons when available.
- [ ] License/attribution, platform matrix, evaluation results, packaging, and operational instructions are documented. Unreleased or unsupported capabilities are not presented as shipped.
