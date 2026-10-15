---
type: Plan
title: "Needle router and capability registry parity"
description: "Build native capability parity first, then route acorn ask through reusable SLM, MCP, ACP, and configured inference adapters."
difficulty: high
impact: high
package: [workspace, acorn-lib, acorn-cli, acorn-schema]
tags: [feature, inference, routing, tools, parity, sidecar, mcp, acp]
status: in-progress
execution: phased
phase_budget_hours: 5
---

# Goal

Provide one `acorn ask` interface that selects the right available capability and executor. Establish near registry parity with existing bounded ACORN workflows first, then add reusable bindings for local SLMs, configured MCP tools, ACP agents, and existing inference endpoints. Retain this filename and unfinished SLM/JUL phase IDs for link and assignment stability.

```text
question/command + explicit inputs + application policy
    -> available capability catalog
    -> Needle selects capability and grounded arguments
    -> validate route, arguments, authority, and availability
    -> deterministic executor binding
       native Rust | local SLM | configured MCP tool | ACP agent | inference endpoint
    -> execute once -> typed result / artifacts / provenance
```

Native operations execute existing Rust services; models and agents implement capabilities that require them. The framework is extensible through registered descriptors and adapters, with concrete contracts at operation boundaries. It does not generate shell commands or autonomously orchestrate multiple agents.

# Framework evidence

Local evidence checked on 2026-10-14:

- `cli/mod.rs` exposes `ask` as an alias of `infer`; explicit `infer run` already supports configured prompt backends.
- `io/mcp/tools/mod.rs` contains nine built-in definitions with analysis enabled. Check and validate currently use the same schema validator, so names alone overstate behavioral coverage. Graduation apply is not Needle-exposed.
- `ToolRegistry` already supports typed handlers, Needle schema projections, and effect/exposure/output policy. Extend it as the source of truth; its current MCP directory does not require a duplicate registry or crate extraction.
- `io/sidecar/needle/mod.rs` constructs `ToolRegistry::acorn()` during startup, uses a bounded multi-round loop, and caps the catalog at 32 KiB. It already withholds low-confidence, uncalibrated, negated, and ungrounded calls. Caller-owned registry injection, terminal execution, and catalog-size handling are required.
- `io/mcp/client.rs` exposes `McpClientBackend`, configured discovery/calls, allowlists, context, cancellation, and bounded cleanup. A protocol-successful result can still contain `is_error`.
- `agent/mod.rs` supplies `InferenceBackend`, typed requests/results, capabilities, provenance, and `InferencePermissionPolicy::DenyMutation`; ACP and OpenAI-compatible implementations exist. ACP mutation remains unsupported in this plan.
- Analysis has reusable services in `analyzer/host/service/mod.rs`. Other CLI leaves sometimes combine parsing, operation, and presentation; extract only the operation seam required for parity, coordinating with the CLI composition plan.
- Shared sidecar asset/distribution contracts exist. The four planned SLM adapters remain unimplemented; their external gates do not block native parity or generic adapters.

# Shared framework contract

## Capabilities and executor bindings

A capability declares a stable namespaced ID, versioned input/output schemas, task description/examples, effects, and resource limits. An executor binding declares implementation kind, configured model/server/agent identity, supported contract version, availability, and provenance. Multiple implementations may bind to one capability only when they satisfy the same contract. Arbitrary MCP tools or general-purpose agents are not automatically equivalent to native operations.

Extend the existing `ToolRegistry` with narrow registration/adapter interfaces; reuse `Sidecar`, `McpClientBackend`, and `InferenceBackend`. Keep Rust types canonical for built-ins and validate dynamic MCP JSON at the protocol boundary. Do not force tool calls into a chat API. Registration and policy evaluation are pure; discovery/readiness/execution are explicit effects.

## Routing, selection, and execution

1. Build an immutable request-scoped catalog filtered by features, host, verified assets, allowlists, permissions, network/data policy, and caller constraints.
2. Needle selects a capability and grounded arguments. Retain confidence/negation/grounding gates. Missing inputs, uncertainty, or multiple incompatible actions return clarification/withheld with no execution. Preserve source data through typed references or verbatim projection.
3. Bind deterministically: valid explicit caller selection, then configured preference, then an equivalent native implementation, then a compatible local implementation. Equally preferred external bindings require clarification. Do not invent cost/quality rankings or use routing confidence as model-quality evidence.
4. Revalidate arguments, policy, and availability before dispatch. Stale catalogs or schemas fail explicitly. Configured remote services are eligible only under network/data policy; no implicit remote fallback, provider substitution, or model download. Credentials remain outside prompts/catalogs.
5. Execute one terminal capability and return its final result. No recursive `ask` or registry changes from provider output. Preserve the legacy Needle loop through its existing API while giving routed capability execution a distinct single-execution contract.

Honor the 32-KiB catalog limit: deduplicate executor alternatives behind capabilities, bound descriptions/schema projections, and measure serialized size. If needed, shortlist deterministically using explicit scope and declared task metadata, with retained-candidate recall measured against the routing corpus. Never silently truncate tools; request narrower scope when a valid catalog cannot fit. Catalog/schema/binding changes invalidate session tool-index caches.

Use a normalized context for offline/data policy, permitted paths, mutation authority, deadline, cancellation, and input/output budgets. Adapters may narrow authority, never broaden it. Existing CLI writes remain available through explicit typed policy; model-generated arguments are not authorization. Default writes yield a proposal or permission-required outcome, and apply authority binds the exact operation, arguments, and destination. Never automatically retry non-idempotent effects.

Distinguish completed, needs-input/withheld, unavailable, permission-required, invalid-input, cancelled, timed-out, transport-failed, and tool-failed outcomes. Preserve structured/text output, bounded artifact references, and secret-free provenance. Transport success does not mean task success. Verify cancellation cleanup per adapter and report already-completed effects accurately.

# Early near-parity milestone

PAR-00 freezes a machine-readable scenario matrix against the current CLI. Each row records code owner, input forms/options, outputs/artifacts, effects, feature/platform requirements, registry binding, phase, and equivalence test or explicit exclusion. Count behavior, not aliases or duplicate handlers. Report covered/total bounded scenarios and covered/total CLI scenarios including exclusions; do not claim a percentage without a frozen denominator.

All mandatory bounded scenarios must pass before new external-adapter/model work. This milestone covers existing behavior, not every shell convenience or planned ACORN feature.

| Capability family | Mandatory behavior | Phase |
|---|---|---|
| Existing registry/logbook | Preserve tools and explicit graduation policy; separate schema validation from analysis | SLM-00, RTR-02 |
| Check | Existing analysis categories, findings, offline and unavailable-category behavior | PAR-02 |
| Diff/similarity | Existing semantic diff/patch and pairwise metrics | PAR-03 |
| Gather | Existing PID extraction from text/documents/permitted URLs | PAR-04 |
| Format/crosswalk | Supported standards, normalization/enrichment, loss warnings, preview/apply | PAR-05 |
| Linked data | Existing transformation and authorized output | PAR-06 |
| SWHID | Current calculate/verify over permitted inputs | PAR-07 |
| Doctor | Structured diagnostics/readiness; no automatic repair | PAR-08 |
| Export | Existing formats/targets, bounded batches, artifact output | PAR-09 |
| Import | Current spec/model import, preview and explicit apply | PAR-10 |
| Download | Existing bucket/model/Needle acquisition/status, explicitly requested | PAR-11 |
| Sync | Current target configuration preview/apply | PAR-12 |
| Report | Existing artifact generation and explicit publication | PAR-13 |
| Script checks | Existing check/check-safe; no unrestricted automation | PAR-14 |
| Prompt/MCP commands | Preserve direct commands; routed integration follows native parity | ADP-02 through ADP-04 |

Exclude interactive TUI/clipboard UX, persistent servers, infrastructure/secret creation, automatic repair, and unrestricted automation from the bounded milestone. Keep them visible in the total CLI inventory. Unsupported flags/scenarios need explicit entries rather than disappearing from the denominator. Native-handler equivalence tests require no model weights; real Needle routing is a separate gate.

# CLI, ownership, and ordering

`acorn ask [QUESTION_OR_COMMAND]` remains the primary interface. Preserve positional/stdin exclusivity, current flags/aliases/escaping, ordinary direct commands, and `infer run`. RTR-03 freezes structured input, binding selection, route preview, and apply-policy options. Do not add per-provider inference subcommands.

Registry/adapter leaves initially live under `crates/acorn-lib/src/io/mcp/tools/` with preserved public exports and optional feature gates. CLI business logic needed by both interfaces moves into a small library operation seam; the CLI keeps normalization/presentation. Do not invoke the ACORN binary or duplicate algorithms to simulate parity.

Scheduling order:

1. SLM-00 caller-owned registry; PAR-00 parity inventory; RTR-01 through RTR-03 shared binding/context/CLI.
2. PAR-01 through PAR-14 native capability coverage; PAR-15 real Needle routing; PAR-16 native parity release.
3. ADP-01 configuration/features; ADP-02 through ADP-05 MCP, ACP, endpoint, and sidecar adapters; ADP-06/07 framework validation/release.
4. Existing Swift and Julia phases consume this framework and release independently on their own gates.

Each phase has at most five active hours and an exact product write set. Reconcile this plan and its four index/network/lane files after each phase. PAR-00 must split missing reusable service seams into bounded extraction phases before an adapter begins; a cross-crate rewrite cannot hide inside a registration phase.

The [CLI composition plan](../refactors/core/cli-composition-boundary.md) owns broad parser/startup normalization; coordinate command leaves and reuse landed request types. The [export plan](../refactors/core/export-pipeline-hardening.md) owns renderer correctness; consume its supported services and report failing output gates. The [gather plan](gather-extraction.md) owns future extraction. The [model acquisition plan](model-acquisition-hardening.md) owns shared acquisition/OCI changes. Existing [MCP embedder contracts](../refactors/api/outbound-mcp-client-interface.md) are usable before that plan's documentation closeout. Needle release-host validation remains independently owned.

All registered native operations should support the same typed execution contract for `ask` and explicitly enabled MCP exposure; catalog membership and execution policy still differ by interface. Routability does not grant inbound MCP exposure. Remote tools get collision-safe configured-server/tool names; local allowlists/effects control exposure. Treat MCP descriptions/results as data. ACP launch profiles, endpoints, and credentials come from trusted configuration, never generated executable paths or URLs.

# Framework acceptance

- Two compatible executor bindings for the same capability work without router changes; mismatched schemas/effects are rejected.
- All mandatory parity scenarios pass CLI/service-versus-registry tests for success, failure, offline behavior, effects, and artifacts, with exclusions reported.
- Freeze at least 200 labeled routing cases before scoring. Require 95% correct capability selection on unambiguous supported cases and zero executions on designated forbidden/negated/ungrounded cases. Report abstentions, coverage, latency, and shortlist recall; do not tune on the held-out set.
- Adapters prove availability changes, semantic errors, resource limits, cancellation/cleanup, provenance, and no implicit fallback or recursive routing.
- Existing CLI, MCP, Needle, and explicit inference compatibility survives; narrow feature and portable builds do not acquire unwanted host dependencies.
- Run focused tests per phase; serialize final lint/full-suite, feature/WASM, help/docs, and explicit real-backend gates after shared files stabilize.

# SLM adapter evidence

The following retained upstream evidence was checked on 2026-09-22; it is model-profile planning evidence, not a current availability guarantee. Recheck upstream availability, licenses, and immutable artifact revisions before publishing a release.

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

# Swift adapter profile decisions

- **Swift profile:** Tongue, Redact, and Title. Native, MCP, ACP, and Julia profiles have independent release gates. Schemer remains visibly unavailable until a supported public release, pinned harness, and shared extraction contract exist.
- **Runtime ownership:** ACORN maintains one pinned Swift package with a separate executable for each capability. Do not depend on an unversioned upstream command-line interface.
- **Platforms:** Tongue and Redact target Linux x86_64, Linux ARM64, and macOS ARM64. Title targets macOS ARM64 only. Windows model runners are unsupported in this Swift profile; the generic framework retains existing host support.
- **Stability:** all three capabilities remain unreleased until their functional, quality, licensing, packaging, and platform gates pass. Do not label only a subset experimental to work around a missing gate.
- **Routing:** The shared SLM-00/RTR/PAR framework comes first. Register concrete sidecars as executor bindings after ADP-05; Needle selects the capability and its bound sidecar returns the terminal result.
- **Invocation:** `acorn ask` is the single user-facing question/command interface; download remains explicit. There is no automatic model download, background inference, remote fallback, model-driven record mutation, or default integration with check, gather, export, or validation.
- **Licensing:** legal/redistribution approval for the pinned SDK and weights is a hard release gate, not a documentation follow-up.
- **GLiDE placement:** evaluate GLiDE only as a deferred semantic-decision sidecar or comparison adapter. Do not include it in the first Tongue/Redact/Title release, expose it as a generic provider, or present it as a Needle replacement. Needle remains the bounded local tool selector; GLiDE would make task-specific decisions over predefined options.

# SLM implementation boundaries

## Owned runner and artifact files

- **Create:** `scripts/slm/runner/Package.swift`, `scripts/slm/runner/Package.resolved`, shared protocol/process helpers under `scripts/slm/runner/Sources/SlmProtocol/`, and `TongueRunner`, `RedactRunner`, and `TitleRunner` executable targets.
- **Create:** `scripts/slm/create-oci-artifact.sh` and `scripts/slm/upload-runners.sh` to build one capability/platform artifact at a time and emit its immutable URL, byte size, and SHA-256.
- **Update:** `.gitlab-ci.yml` and `Makefile` through the CI/build integration lock. Linux jobs build and smoke-test only supported target/capability pairs. Keep macOS validation manual until a runner is proven schedulable; a passing macOS gate is required before release.
- Build and publish runner/model artifacts first. Pin the resulting immutable URLs, sizes, and hashes in a second Rust integration change. Rust source must not predict release checksums.

## Rust and CLI files

- **Update:** `crates/acorn-lib/Cargo.toml` and `crates/acorn-lib/src/io/mod.rs` to consume ADP-01's opt-in `slm` feature/module wiring and add only the concrete profile registrations. Extend `crates/acorn-lib/src/io/sidecar/mod.rs` only when a second concrete implementation proves another lifecycle primitive is shared.
- **Create:** `crates/acorn-lib/src/io/sidecar/slm/{mod,assets,process,protocol,language,redact,title}.rs` with concrete `LanguageSidecar`, `RedactSidecar`, and `TitleSidecar` implementations. Each sidecar implements `SidecarAssets` for acquisition and `Sidecar` for its typed request, result, and process lifecycle. Keep capability-specific result types and validation explicit; register typed handlers in the existing tools registry; do not introduce a parallel registry or erase capability contracts behind `serde_json::Value`.
- **Update:** the existing CLI inference/download command modules, CLI argument definitions, and dispatch roots to compose configured executor bindings and explicit model downloads. Regenerate `CommandLineHelp.md` once under the CLI integration lock.
- **Create:** `tests/fixtures/slm/{language,redact,title}/` and `docs/src/inference/slm.md`; update `docs/src/SUMMARY.md`.
- **Delete:** none.

# SLM runner protocol and resource contract

Each executable writes one JSON-lines readiness envelope, reads exactly one typed request from stdin, writes one typed response to stdout, and exits. Diagnostics go to stderr. Every envelope includes protocol version, capability, and request ID. Responses distinguish success, unsupported input, validation failure, model failure, and timeout; error text must not echo sensitive source content.

Set `DAL_USAGE_DISABLED=1` for every managed process and verify offline execution. Use isolated temporary/session directories, bounded startup and request timers, cancellation, process-tree cleanup, and a 1 MiB maximum response. The initial input budgets are:

| Capability | Input limit | Request timeout |
|---|---:|---:|
| Tongue | 512 Unicode scalar values | 10 seconds |
| Redact | 64 KiB UTF-8 | 60 seconds |
| Title | 16 KiB UTF-8 | 120 seconds |

`LanguageResult` preserves the detected tag, upstream reliability, and an explicit uncertain/abstained state. `RedactResult` preserves validated, non-overlapping UTF-8 source spans, labels, scores where provided, and an optional copy produced only when requested. `TitleResult` preserves the proposed title, optional subtitle, validation findings, model/prompt revisions, and source fingerprint. Unsupported, insufficient-evidence, and technical-failure outcomes stay distinct.

# SLM implementation sequence

After native parity and the shared adapter contract, retain SLM-01 through SLM-22 for Swift and JUL-01 through JUL-15 for Julia. Publish immutable assets before pinning hashes in Rust. Register each model as a capability executor behind `acorn ask`; retain explicit `acorn download slm <language|redact|title|decide> [--status]` and custom locations. Licensing and host prerequisites apply only to their model profiles.

# Swift quality and release gates

- **Tongue:** at least 95% top-1 accuracy on the frozen unambiguous ACORN prose set; every predesignated mixed-language, acronym-only, or otherwise ambiguous case abstains instead of asserting a language.
- **Redact:** at least 85% macro recall across the frozen sensitive-label set and at least 95% precision. Documentation states that a clean result does not certify absence of sensitive material.
- **Title:** at least 90% blinded reviewer acceptance, zero unsupported scientific claims in the frozen evaluation set, and at least 95% canonical title/subtitle validation compliance. Invalid subtitles are omitted with findings; do not truncate or silently rewrite them.
- **All capabilities:** immutable reproducible artifacts, approved licenses/notices, clean offline execution, no unexpected network calls, bounded lifecycle behavior, and passing real-model smoke tests on every advertised platform.

# Deferred retained scope

- **Schemer:** wait for a supported release and implement it only through [deterministic and assisted gather extraction](gather-extraction.md), preserving source offsets, typed absence, conflicting candidates, and deterministic validation.
- **Persistence/application:** proposal-only output may precede persistence. Durable enrichment proposals and reconciliation belong to the canonical database boundary; authored RAD values always win unless a user explicitly accepts a validated proposal.
- **Semantic decisions:** SemIf/Open-Jev/Needle comparison remains an opt-in experiment over frozen source/proposal fixtures. It must predeclare accuracy, unsupported-claim, false-rejection, abstention, calibration, latency, memory, and installed-size gates. Scores are advisory and conditional on supplied options. Promotion would add one task-specific adapter; it cannot authorize mutation, establish factuality, or block the first three sidecars.
- **GLiDE decision adapter:** include Fastino GLiDE in the semantic-decision comparison only after verifying an immutable revision, licensing/redistribution terms, data-handling requirements, and an offline or approved self-hosted deployment path. Compare its typed choice, confidence, probability, abstention, latency, memory, and failure behavior against SemIf, Open-Jev, and the existing Needle baseline. A successful evaluation may justify a separate policy/approval sidecar; it must not replace Needle's tool-selection protocol or broaden tool execution authority.
- **Deferred model scope:** automatic enrichment, implicit remote fallback, browser integration, audio/vision, training, and Windows model runners. Generic executor adapters, configured MCP, ACP, and inference endpoints are in scope for the framework.

# Julia-1 decision sidecar (planned follow-on)

This slice was added on 2026-10-06 and remains **planned**; the parent status reflects the existing registry, inference, and sidecar foundations. Its outcome is an explicitly invoked, advisory finite-choice decision capability. It does not block Tongue, Redact, or Title and does not inherit their Swift runtime or their all-three release gate. Keep this work in the router concept rather than creating an overlapping model-provider plan.

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

# SLM profile validation

- Compile-time contract tests assert that every concrete adapter declares typed input/output, stable identity/protocol values, asset resolution, verified-asset startup, and required inference.
- Model-independent fixtures cover readiness and typed envelopes, malformed/oversized responses, request ID mismatch, Unicode and byte limits, timeouts, cancellation, concurrent sessions, startup failure, cleanup, stderr separation, and redacted diagnostics.
- Asset tests cover the supported semantic manifest version, unsupported and legacy shapes, duplicate or unsafe asset names, bad sizes/digests, partial downloads, unsupported targets, custom locations, offline cache hits/misses, and OCI/local parity.
- Capability fixtures cover scientific abbreviations, ambiguous and mixed-language prose, expected contact PII, missed and false-positive redactions, UTF-8 span boundaries, malformed Title output, absent fields, subtitle limits, unsupported claims, and stale source fingerprints.
- Measure cold/warm latency, total download, peak memory, and batch behavior on every claimed platform. Keep heavyweight real-model tests explicit and separate from ordinary offline unit tests.
- Run formatting, lint, focused tests, `make test`, applicable feature/WASM checks, license/dependency checks, generated-help checks, Linux CI smoke jobs, and manual macOS host and real-model checks for implementation changes.

# SLM profile acceptance criteria

- [ ] `acorn ask` routes questions/commands through the existing Needle tools registry to exactly one eligible capability sidecar; invalid, unavailable, or withheld routes execute no model.
- [ ] Pinned, ACORN-owned Tongue, Redact, and Title runners implement the bounded one-request protocol and run offline on every advertised target.
- [ ] Explicit library and CLI paths return typed results through Needle-selected capability handlers without mutating RAD records or downloading during inference.
- [ ] The single `acorn ask` interface, legacy inference parsing, explicit `infer run`, and prompt escaping remain supported; no per-capability inference commands are required.
- [ ] Tongue, Redact, and Title meet their frozen numeric quality gates; all three also pass license, artifact, lifecycle, documentation, and platform gates before stable release.
- [ ] Title is clearly macOS ARM64-only, Windows is clearly unsupported, and unavailable capabilities fail with actionable errors rather than remote fallback.
- [ ] Schemer, persistence, and semantic-decision work remain gated follow-ons and are not represented as shipped.

# Execution phases

SLM-00 remains first. RTR/PAR/ADP phases extend the plan without renumbering model phases. Native parity precedes external-adapter and model implementation. Split any missing operation extraction identified by PAR-00 into its own bounded phase before work begins.

## SLM-00 — Augment the registry and inject it into Needle

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: landed ToolRegistry and Needle contracts; no external model prerequisite
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/needle.rs`, `crates/acorn-lib/src/io/sidecar/needle/mod.rs`, `crates/acorn-lib/src/io/sidecar/needle/tests/mod.rs`, `crates/acorn-lib/src/io/mcp/tests/server.rs`
- Primary integration lock: Ask capability routing
- Outcome: Add caller-owned registry/catalog injection while preserving the default constructor and legacy loop. Establish availability-filtered typed terminal handlers without new providers or flags.
- Focused validation: Fixture-test injected catalog, default behavior, withheld routes, and exactly one terminal handler call.
- Handoff: Remove SLM-00 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-00 — Freeze the capability parity matrix

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-00
- Exact write set: `tests/fixtures/routing/parity.json`, `tests/fixtures/routing/cases.json`, `docs/src/inference/capability-parity.md`
- Primary integration lock: Capability parity contract
- Outcome: Inventory all current CLI scenarios, mandatory bounded scope, exclusions, code owners, exact operation-extraction needs, and routing/equivalence cases. Freeze separate bounded and total-CLI denominators.
- Focused validation: Check unique IDs, command coverage, scenario/phase ownership, at least 200 routing cases with disjoint calibration/held-out splits, and no duplicate-handler coverage inflation.
- Handoff: Remove PAR-00 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## RTR-01 — Define capabilities and executor bindings

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-00
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/capability.rs`, `crates/acorn-lib/src/io/mcp/tools/capability_tests.rs`
- Primary integration lock: Ask capability routing
- Outcome: Add capability identity/schema versions, typed binding registration, availability reasons, and deterministic selection in one registry. Preserve the catalog byte cap immediately, including deterministic explicit-scope filtering and clear narrowing errors; later parity additions must not silently break startup.
- Focused validation: Test two compatible implementations, ID collisions, schema/effect mismatch, feature/host filtering, explicit override, configured preference, ambiguity, and oversized-catalog rejection/scoping.
- Handoff: Remove RTR-01 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## RTR-02 — Carry execution context and terminal outcomes

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RTR-01
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/execution.rs`, `crates/acorn-lib/src/io/mcp/tools/execution_tests.rs`, `crates/acorn-lib/src/io/sidecar/needle/mod.rs`, `crates/acorn-lib/src/io/sidecar/needle/tests/mod.rs`
- Primary integration lock: Ask capability routing
- Outcome: Propagate policy, deadline/cancellation, typed result/artifact envelope, exact mutation authority, and final availability checks. Separate routed terminal execution from legacy looping.
- Focused validation: Test denial before effects, stale bindings, invalid/ungrounded input, no repeated non-idempotent calls, cancellation, bounds, and secret-free results.
- Handoff: Remove RTR-02 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## RTR-03 — Compose routed acorn ask

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RTR-02
- Exact write set: `crates/acorn-cli/src/commands/infer/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`, `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/cli/tests/mod.rs`, `CommandLineHelp.md`
- Primary integration lock: CLI contract
- Outcome: Build the caller-owned registry at the CLI boundary; freeze structured inputs, binding override, route preview, and apply-policy syntax while retaining existing aliases/flags and infer run.
- Focused validation: Parser, stdin conflicts, legacy compatibility, preview-no-execution, authorization, result rendering, and generated-help tests.
- Handoff: Remove RTR-03 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-01 — Share bounded inputs and artifact handling

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RTR-03
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/input.rs`, `crates/acorn-lib/src/io/mcp/tools/input_tests.rs`
- Primary integration lock: Ask capability routing
- Outcome: Normalize text/file/directory inputs and artifact destinations with limits, deterministic batches, and explicit write policy. Establish the native leaf module root.
- Focused validation: Path containment, traversal limits, invalid input, batch ordering/partial failures, destination collisions, and no unintended writes.
- Handoff: Remove PAR-01 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-02 — Expose full analysis checks

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/analysis.rs`, `crates/acorn-lib/src/io/mcp/tools/native/analysis_tests.rs`, `crates/acorn-cli/src/commands/check/mod.rs`, `crates/acorn-cli/src/commands/check/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Existing category selection and structured analysis reports; preserve schema-only validation as distinct. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Category selection, offline links, unavailable dependencies, schema/prose fixtures, and matching findings. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-02 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-03 — Expose semantic diff and similarity

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/diff.rs`, `crates/acorn-lib/src/io/mcp/tools/native/diff_tests.rs`, `crates/acorn-cli/src/commands/diff/mod.rs`, `crates/acorn-cli/src/commands/diff/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current semantic diff, JSON Patch, and supported similarity metrics. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Equal/changed inputs, metric options, limits, and matching patch/score outputs. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-03 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-04 — Expose current gather operations

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/gather.rs`, `crates/acorn-lib/src/io/mcp/tools/native/gather_tests.rs`, `crates/acorn-cli/src/commands/gather/mod.rs`, `crates/acorn-cli/src/commands/gather/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current PID extraction from text/documents/permitted URLs and its provenance; no future extraction feature. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Input types, offline rejection, duplicates, bounds, and matching extracted identifiers. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-04 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-05 — Align format and crosswalk behavior

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/format.rs`, `crates/acorn-lib/src/io/mcp/tools/native/format_tests.rs`, `crates/acorn-cli/src/commands/format/mod.rs`, `crates/acorn-cli/src/commands/format/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Supported standards, normalization/enrichment, loss warnings, preview and authorized output. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Format/standard matrix, offline enrichment, field-loss warnings, preview/apply, and invalid-input no-write. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-05 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-06 — Expose linked-data operations

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/link.rs`, `crates/acorn-lib/src/io/mcp/tools/native/link_tests.rs`, `crates/acorn-cli/src/commands/link/mod.rs`, `crates/acorn-cli/src/commands/link/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Existing linked-data transformation with typed output and destination policy. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Transformation fixtures, repeated operation behavior, preview/apply, and schema failures. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-06 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-07 — Expose SWHID calculate and verify

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/swhid.rs`, `crates/acorn-lib/src/io/mcp/tools/native/swhid_tests.rs`, `crates/acorn-cli/src/commands/swhid/mod.rs`, `crates/acorn-cli/src/commands/swhid/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current local object calculation/verification through existing library behavior. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: File/directory/revision fixtures, mismatches, permitted paths, and stable identifiers. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-07 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-08 — Expose structured diagnostics

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/doctor.rs`, `crates/acorn-lib/src/io/mcp/tools/native/doctor_tests.rs`, `crates/acorn-cli/src/commands/doctor/mod.rs`, `crates/acorn-cli/src/commands/doctor/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Report-only diagnostics and readiness; keep automatic repair unavailable. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Platform/dependency fixtures, missing tools, no mutations, redaction, and matching report data. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-08 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-09 — Align exports and artifact results

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/export.rs`, `crates/acorn-lib/src/io/mcp/tools/native/export_tests.rs`, `crates/acorn-cli/src/commands/export/mod.rs`, `crates/acorn-cli/src/commands/export/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Existing formats/targets and bounded batch inputs through shared renderers; preserve update export. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Format/target fixtures, output bytes/artifacts, failed renderers, and authorized destinations. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-09 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-10 — Expose bounded imports

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/import.rs`, `crates/acorn-lib/src/io/mcp/tools/native/import_tests.rs`, `crates/acorn-cli/src/commands/import/mod.rs`, `crates/acorn-cli/src/commands/import/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current spec/model import previews and explicit apply using existing services. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Local/remote inputs, invalid specs, offline behavior, persistence policy, and preview/apply equivalence. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-10 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-11 — Expose explicit downloads and status

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/download.rs`, `crates/acorn-lib/src/io/mcp/tools/native/download_tests.rs`, `crates/acorn-cli/src/commands/download/mod.rs`, `crates/acorn-cli/src/commands/download/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Existing bucket/model/Needle acquisition/status through current planners; only explicit acquisition requests. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Dry-run/cache/status, offline/missing cache, clobber policy, cancellation, and no inference-triggered downloads. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-11 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-12 — Expose configuration synchronization

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/sync.rs`, `crates/acorn-lib/src/io/mcp/tools/native/sync_tests.rs`, `crates/acorn-cli/src/commands/sync/mod.rs`, `crates/acorn-cli/src/commands/sync/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current target selection, configuration previews, and explicit apply. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Supported targets, unchanged configuration, proposed diffs, exact apply policy, and CLI equivalence. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-12 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-13 — Expose reporting and explicit publication

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/report.rs`, `crates/acorn-lib/src/io/mcp/tools/native/report_tests.rs`, `crates/acorn-cli/src/commands/report/mod.rs`, `crates/acorn-cli/src/commands/report/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Existing report artifact generation and publication through current destination/credential policy. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Deterministic artifacts, denied/default publish, authorized mock publication, retries, and secret-free results. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-13 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-14 — Expose constrained script checking

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/script.rs`, `crates/acorn-lib/src/io/mcp/tools/native/script_tests.rs`, `crates/acorn-cli/src/commands/exec/mod.rs`, `crates/acorn-cli/src/commands/exec/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current check/check-safe behavior; do not register unrestricted automation. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Static/evaluation fixtures, forbidden operations, fuel/size bounds, and matching findings. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-14 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-15 — Prove routing at near-parity catalog size

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-02 through PAR-14; pinned Needle runner installed on a supported host
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/needle.rs`, `crates/acorn-lib/src/io/sidecar/needle/mod.rs`, `crates/acorn-lib/src/io/sidecar/needle/assets.rs`, `crates/acorn-lib/src/io/sidecar/needle/tests/mod.rs`, `tests/fixtures/routing/results.json`, `docs/src/inference/routing-evaluation.md`
- Primary integration lock: Needle routing evaluation
- Outcome: Enforce catalog budgets/cache invalidation and score the frozen corpus against the actual native catalog. Add measured deterministic scoping only if needed.
- Focused validation: Declared routing gates, no silent truncation, retained-candidate recall, negative-case nonexecution, abstention, latency, and index/schema cache isolation.
- Handoff: Remove PAR-15 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-16 — Release native near parity

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-15
- Exact write set: `tests/fixtures/routing/parity.json`, `docs/src/inference/capability-parity.md`, `docs/src/inference/router.md`, `docs/src/SUMMARY.md`, `CommandLineHelp.md`
- Primary integration lock: Router release gate
- Outcome: Publish measured coverage, exclusions, compatibility, and examples before external adapters. Failed scenarios remain remaining work.
- Focused validation: Serialize make lint, make test, applicable feature/WASM checks, help/book checks, and matrix/equivalence audit.
- Handoff: Remove PAR-16 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## ADP-01 — Wire optional executor configuration and features

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-16
- Exact write set: `crates/acorn-lib/Cargo.toml`, `crates/acorn-lib/src/io/mod.rs`, `crates/acorn-lib/src/io/sidecar/mod.rs`, `crates/acorn-lib/src/io/sidecar/slm/mod.rs`, `crates/acorn-cli/src/commands/infer/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`, `crates/acorn-cli/Cargo.toml`, `crates/acorn-lib/src/io/config/application.rs`, `crates/acorn-lib/src/io/config/tests/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/configuration.rs`, `crates/acorn-lib/src/io/mcp/tools/configuration_tests.rs`
- Primary integration lock: Workspace graph
- Outcome: Validate configured capability bindings to existing MCP server aliases, trusted ACP profiles, endpoints, and verified sidecars. Reuse transport configuration and isolate optional features; no new transport implementation. Wire composition from the CLI and create the optional SLM module independently of Swift artifacts so Julia can compile and release independently.
- Focused validation: Minimal/combined feature builds, duplicate/invalid bindings, unavailable executor kinds, secret-free descriptors, and no discovery during configuration parsing.
- Handoff: Remove ADP-01 after preserving contract/evidence in code, tests, and documentation.

## ADP-02 — Adapt configured MCP tools

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-01
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/mcp.rs`, `crates/acorn-lib/src/io/mcp/tools/mcp_tests.rs`
- Primary integration lock: Ask capability routing
- Outcome: Use McpClientBackend discovery/calls with collision-safe IDs, local policy, schemas, cancellation, and bounded content. Preserve semantic tool errors; ask eligibility does not imply inbound MCP re-export.
- Focused validation: Mock pagination/schema drift, name collisions, untrusted metadata, offline/allowlist rules, is_error, limits, cancellation cleanup, and no provider fallback.
- Handoff: Remove ADP-02 after preserving contract/evidence in code, tests, and documentation.

## ADP-03 — Adapt trusted ACP agent capabilities

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-01
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/acp.rs`, `crates/acorn-lib/src/io/mcp/tools/acp_tests.rs`
- Primary integration lock: Ask capability routing
- Outcome: Bind named tasks to trusted ACP profiles through InferenceBackend and prompt hydration; preserve deny-mutation policy, configured model/agent, schema support, and provenance. Do not grant arbitrary agent authority.
- Focused validation: Fake sessions for capability mismatch, permission requests, selected model, refusal, timeout/cancellation cleanup, malformed structured output, and final result mapping.
- Handoff: Remove ADP-03 after preserving contract/evidence in code, tests, and documentation.

## ADP-04 — Adapt configured inference endpoints

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-01
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/inference.rs`, `crates/acorn-lib/src/io/mcp/tools/inference_tests.rs`
- Primary integration lock: Ask capability routing
- Outcome: Bind named tasks to existing OpenAI-compatible InferenceBackend with configured endpoint/model, explicit data policy, and validated outputs. No new provider SDK or implicit fallback.
- Focused validation: Mock refusal/failure, malformed output, unsupported schema/attachments, offline rejection, cancellation, and binding preference.
- Handoff: Remove ADP-04 after preserving contract/evidence in code, tests, and documentation.

## ADP-05 — Prove reusable local sidecar registration

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-01
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/sidecar.rs`, `crates/acorn-lib/src/io/mcp/tools/sidecar_tests.rs`
- Primary integration lock: Ask capability routing
- Outcome: Adapt SidecarAssets/Sidecar into registered handlers with verified installed assets, target availability, bounded lifecycle, and typed final output. Use deterministic fixture runners; real SLMs remain model-profile work.
- Focused validation: Readiness, input/output contracts, missing/altered assets, unsupported hosts, timeout/cancellation/process cleanup, exactly one invocation, and no implicit acquisition.
- Handoff: Remove ADP-05 after preserving contract/evidence in code, tests, and documentation.

## ADP-06 — Validate heterogeneous executor selection

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-02, ADP-03, ADP-04, ADP-05
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/capability_tests.rs`, `crates/acorn-lib/src/io/mcp/tools/execution_tests.rs`, `tests/fixtures/routing/executors.json`, `docs/src/inference/routing-evaluation.md`
- Primary integration lock: Router release gate
- Outcome: Prove compatible task bindings across native/SLM/MCP/ACP/endpoint fixtures without router changes. Compare task results and explainable selection; no recursive delegation.
- Focused validation: Cross-adapter schema/effect rejection, explicit override, preference/ambiguity, stale/unavailable bindings, data policy, and no repeated side effects.
- Handoff: Remove ADP-06 after preserving contract/evidence in code, tests, and documentation.

## ADP-07 — Document and release the generic router

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-06
- Exact write set: `docs/src/inference/router.md`, `docs/src/inference/capability-parity.md`, `docs/src/SUMMARY.md`, `CommandLineHelp.md`
- Primary integration lock: Router release gate
- Outcome: Publish registration examples, configuration, selection rules, supported policies, and measured coverage. Release independently of Swift and Julia availability.
- Focused validation: Serialized lint/full suite, narrow/combined features, WASM, help/docs, and explicit available-transport smoke tests. Record unsupported/unvalidated profiles; a failed required gate keeps this phase open.
- Handoff: Remove ADP-07 after preserving contract/evidence in code, tests, and documentation.

## SLM-01 — Obtain redistribution approval

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: legal owner availability
- Exact write set: Desert Ant SDK/weight approval record, this plan
- Primary integration lock: SLM release gate
- Outcome: Record approval or rejection for the pinned SDK and Tongue, Redact, and Title weights.
- Focused validation: Verify every artifact and required notice is covered by the dated decision.
- Handoff: Remove SLM-01 or stop the Swift release path without speculative packaging.

## SLM-02 — Prove the macOS build host

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: schedulable macOS ARM64 host
- Exact write set: macOS ARM64 runner configuration, `scripts/slm/check-macos-host.sh`, this plan
- Primary integration lock: SLM release gate
- Outcome: Record one schedulable Xcode 26 and Swift 6.2+ native build host.
- Focused validation: Run the host check and one pinned Swift package compilation.
- Handoff: Remove SLM-02 and publish non-secret host evidence.

## SLM-03 — Freeze the one-request protocol fixtures

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-05, SLM-01
- Exact write set: `tests/fixtures/slm/protocol/`, protocol contract
- Primary integration lock: SLM protocol
- Outcome: Freeze readiness, request, response, error, timeout, and request-ID envelopes for three capabilities.
- Focused validation: Review strict JSON-lines, byte limits, stdout/stderr, and redaction cases.
- Handoff: Remove SLM-03 and use the fixtures for every runner and Rust adapter.

## SLM-04 — Implement shared Swift protocol support

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-02, SLM-03
- Exact write set: `scripts/slm/runner/Package.swift`, `scripts/slm/runner/Package.resolved`, `scripts/slm/runner/Sources/SlmProtocol/Protocol.swift`
- Primary integration lock: Swift SLM package
- Outcome: Read one bounded request, emit readiness/typed response, isolate diagnostics, and exit deterministically.
- Focused validation: Run protocol-only Swift tests for malformed, oversize, timeout, and request-ID cases.
- Handoff: Remove SLM-04 and freeze the shared runner API.

## SLM-05 — Implement TongueRunner

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-04
- Exact write set: `scripts/slm/runner/Sources/TongueRunner/main.swift`, Tongue runner tests
- Primary integration lock: Swift SLM package
- Outcome: Produce typed language, reliability, and explicit abstention within 512 scalars.
- Focused validation: Run protocol fixtures and pinned-model smoke tests on a supported host.
- Handoff: Remove SLM-05 and preserve runner/model revisions.

## SLM-06 — Implement RedactRunner

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-04
- Exact write set: `scripts/slm/runner/Sources/RedactRunner/main.swift`, Redact runner tests
- Primary integration lock: Swift SLM package
- Outcome: Return validated non-overlapping UTF-8 spans and an optional explicitly requested copy.
- Focused validation: Run structured-rule/model, boundary, malformed, and 64-KiB limit cases.
- Handoff: Remove SLM-06 and preserve runner/model revisions.

## SLM-07 — Implement TitleRunner

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-04
- Exact write set: `scripts/slm/runner/Sources/TitleRunner/main.swift`, Title runner tests
- Primary integration lock: Swift SLM package
- Outcome: Return title/subtitle proposals, findings, revisions, and source fingerprint without silent truncation.
- Focused validation: Run malformed, unsupported-claim, subtitle-validation, and 16-KiB input cases on macOS ARM64.
- Handoff: Remove SLM-07 and preserve runner/model revisions.

## SLM-08 — Build Linux x86_64 runner artifacts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-05, SLM-06
- Exact write set: `.gitlab-ci.yml`, `scripts/slm/create-oci-artifact.sh`, Linux x86_64 build record
- Primary integration lock: CI build entry points
- Outcome: Build and smoke Tongue/Redact artifacts for Linux x86_64.
- Focused validation: Verify clean build, protocol, real model, offline execution, bytes, hashes, and notices.
- Handoff: Remove SLM-08 and retain immutable artifact facts.

## SLM-09 — Build Linux ARM64 runner artifacts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-05, SLM-06
- Exact write set: `.gitlab-ci.yml`, `scripts/slm/create-oci-artifact.sh`, Linux ARM64 build record
- Primary integration lock: CI build entry points
- Outcome: Build and smoke Tongue/Redact artifacts for Linux ARM64.
- Focused validation: Verify clean build, protocol, real model, offline execution, bytes, hashes, and notices.
- Handoff: Remove SLM-09 and retain immutable artifact facts.

## SLM-10 — Build macOS ARM64 runner artifacts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-02, SLM-05, SLM-06, SLM-07
- Exact write set: macOS ARM64 build job, `scripts/slm/create-oci-artifact.sh`, macOS artifact record
- Primary integration lock: CI build entry points
- Outcome: Build and smoke Tongue, Redact, and Title artifacts for macOS ARM64.
- Focused validation: Verify clean build, protocol, real models, offline execution, bytes, hashes, and notices.
- Handoff: Remove SLM-10 and retain immutable artifact facts.

## SLM-11 — Publish immutable Swift artifacts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-08, SLM-09, SLM-10
- Exact write set: `scripts/slm/upload-runners.sh`, versioned artifact registry, artifact manifest record
- Primary integration lock: SLM artifact publication
- Outcome: Publish each capability/platform artifact and record exact URL, size, SHA-256, and license.
- Focused validation: Install each artifact into a clean cache and rerun offline protocol/model smoke tests.
- Handoff: Remove SLM-11 and provide immutable facts to Rust integration.

## SLM-12 — Pin Swift profile manifests and registration

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-01, SLM-11
- Exact write set: `crates/acorn-lib/Cargo.toml`, `crates/acorn-lib/src/io/mod.rs`, `crates/acorn-lib/src/io/sidecar/slm/assets.rs`, `crates/acorn-lib/src/io/sidecar/slm/mod.rs`
- Primary integration lock: library feature graph
- Outcome: Consume the existing opt-in `slm` wiring and pin validated Swift capability/platform assets without predicting checksums.
- Focused validation: Run feature/no-feature, dependency, manifest, custom-location, and unsupported-target tests.
- Handoff: Remove SLM-12 and freeze asset identity.

## SLM-13 — Implement bounded SLM process lifecycle

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-03, SLM-12
- Exact write set: `crates/acorn-lib/src/io/sidecar/slm/process.rs`, `crates/acorn-lib/src/io/sidecar/slm/protocol.rs`, lifecycle tests
- Primary integration lock: shared sidecar root
- Outcome: Start verified assets with telemetry off, bounded I/O, cancellation, isolation, and process-tree cleanup.
- Focused validation: Run fixture-only startup, timeout, cancellation, concurrency, cleanup, stderr, and redaction tests.
- Handoff: Remove SLM-13 and expose a typed consuming lifecycle.

## SLM-14 — Add LanguageSidecar

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-13
- Exact write set: `crates/acorn-lib/src/io/sidecar/slm/language.rs`, language adapter tests
- Primary integration lock: shared sidecar root
- Outcome: Validate and return language results without remote fallback or mutation.
- Focused validation: Run fixtures, limits, uncertain outcomes, asset, offline, and real-model smoke tests.
- Handoff: Remove SLM-14 and freeze the typed result.

## SLM-15 — Add RedactSidecar

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-13
- Exact write set: `crates/acorn-lib/src/io/sidecar/slm/redact.rs`, redact adapter tests
- Primary integration lock: shared sidecar root
- Outcome: Validate source spans, scores, modes, and optional copies without claiming complete detection.
- Focused validation: Run UTF-8, overlap, threshold, limit, asset, offline, and real-model smoke tests.
- Handoff: Remove SLM-15 and freeze the typed result.

## SLM-16 — Add TitleSidecar

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-13
- Exact write set: `crates/acorn-lib/src/io/sidecar/slm/title.rs`, title adapter tests
- Primary integration lock: shared sidecar root
- Outcome: Validate title/subtitle proposals and preserve findings, model/prompt identity, and source fingerprint.
- Focused validation: Run schema, subtitle, stale-source, limit, asset, offline, and real-model smoke tests.
- Handoff: Remove SLM-16 and freeze the typed result.

## SLM-17 — Register Swift capabilities behind acorn ask

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-05, SLM-14, SLM-15, SLM-16
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/needle.rs`, `crates/acorn-lib/src/io/mcp/tests/server.rs`, `crates/acorn-cli/src/commands/infer/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`, `crates/acorn-cli/src/commands/download/mod.rs`, `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/cli/tests/mod.rs`, `CommandLineHelp.md`
- Primary integration lock: CLI syntax and generated help
- Outcome: Register language/redact/title handlers behind `acorn ask` and add explicit status/download support while preserving existing parsing. Needle routes and the selected sidecar executes; return its result without another routing round.
- Focused validation: Run parser, stdin/argument conflict, unavailable platform, missing cache, custom location, and help tests; run actual Needle-to-sidecar smoke tests for all three capabilities and frozen request-routing cases, including ambiguity/abstention.
- Handoff: Remove SLM-17 and freeze public syntax.

## SLM-18 — Document supported SLM behavior

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-17
- Exact write set: `docs/src/inference/slm.md`, `docs/src/SUMMARY.md`
- Primary integration lock: book documentation
- Outcome: Document explicit invocation, supported targets, limits, typed outcomes, quality caveats, and no automatic mutation/fallback.
- Focused validation: Run `make book`, `make doc`, internal links, and example commands.
- Handoff: Remove SLM-18 and preserve the public support contract.

## SLM-19 — Evaluate Tongue quality

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-14
- Exact write set: frozen Tongue evaluation corpus, Tongue quality report
- Primary integration lock: SLM evaluation
- Outcome: Measure at least 95% top-1 accuracy and required abstention on predeclared ambiguous cases.
- Focused validation: Run the frozen corpus once and report every failure without post-hoc threshold changes.
- Handoff: Remove SLM-19 or create a bounded repair/evaluation phase.

## SLM-20 — Evaluate Redact quality

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-15
- Exact write set: frozen Redact evaluation corpus, Redact quality report
- Primary integration lock: SLM evaluation
- Outcome: Measure at least 85% macro recall and 95% precision across the frozen labels.
- Focused validation: Run the frozen corpus and report misses, false positives, latency, memory, and bytes.
- Handoff: Remove SLM-20 or create a bounded repair/evaluation phase.

## SLM-21 — Evaluate Title quality

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-16
- Exact write set: frozen Title evaluation corpus, Title quality report
- Primary integration lock: SLM evaluation
- Outcome: Measure 90% reviewer acceptance, zero unsupported claims, and 95% canonical validation compliance.
- Focused validation: Run blinded review and report rejected, invalid, omitted, and resource cases.
- Handoff: Remove SLM-21 or create a bounded repair/evaluation phase.

## SLM-22 — Release the Swift profile

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-18, SLM-19, SLM-20, SLM-21
- Exact write set: SLM support documentation, release configuration, this plan
- Primary integration lock: SLM release gate
- Outcome: Run all platform/lifecycle/license/quality gates and mark only the tested three-capability profile stable.
- Focused validation: Run focused tests, feature/WASM checks, dependency/license checks, help/docs, Linux CI, and macOS real-model jobs.
- Handoff: Prune completed Swift phases; retain the plan while any router, parity, adapter, or Julia work remains.

## JUL-01 — Pin Julia feasibility inputs

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-16, named Julia pilot owner
- Exact write set: Julia revision/license/dependency record, `scripts/slm/julia/README.md`, this plan
- Primary integration lock: Julia adoption decision
- Outcome: Pin code, weights, tokenizer, APIs, licenses, build, and offline packaging for Linux x86_64.
- Focused validation: Reproduce one CPU reference inference from locked inputs or stop promotion.
- Handoff: Remove JUL-01 or prune the Julia section if feasibility fails.

## JUL-02 — Freeze protocol, corpus, and gates

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-01
- Exact write set: `tests/fixtures/slm/decision/contract.json`, frozen choice/Noul corpora, evaluation policy
- Primary integration lock: Julia pilot contract
- Outcome: Approve schemas, abstention, limits, baseline, at least 200 cases, quality, parity, and resource gates.
- Focused validation: Review development/test separation and every expected outcome with the named owner.
- Handoff: Remove JUL-02 and prohibit post-hoc gate changes.

## JUL-03 — Implement the pinned Python reference runner

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-02
- Exact write set: `scripts/slm/julia/runner.py`, `scripts/slm/julia/pyproject.toml`, `scripts/slm/julia/uv.lock`, reference tests
- Primary integration lock: Julia runner
- Outcome: Run strict Choice and Noul inference through the locked upstream API with one bounded JSON-lines request.
- Focused validation: Test encoding, IDs, limits, invalid probabilities, timeout, cleanup, and offline operation.
- Handoff: Remove JUL-03 and freeze reference tensors/results.

## JUL-04 — Implement and test Noul types

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-01, JUL-02
- Exact write set: `crates/acorn-lib/src/io/sidecar/slm/mod.rs`, `crates/acorn-lib/src/io/sidecar/slm/decision.rs`, `crates/acorn-lib/src/io/sidecar/slm/decision/tests.rs`
- Primary integration lock: shared sidecar root
- Outcome: Model criteria, validated probability, complement, threshold answer/abstention, and policy revision explicitly.
- Focused validation: Run serde, ordering, endpoints, midpoint, threshold, missing evidence, and invalid-number tests.
- Handoff: Remove JUL-04 and expose the stable Boolean contract.

## JUL-05 — Validate ONNX graph assets

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-01
- Exact write set: `scripts/slm/julia/onnx_backend.py`, ONNX asset inspection tests
- Primary integration lock: Julia runner
- Outcome: Pin graph/external data/tokenizer and freeze containment, hashes, opset, names, dtypes, shapes, and dimensions.
- Focused validation: Test missing, altered, escaping, unsupported-op, and provider-initialization failures.
- Handoff: Remove JUL-05 and publish the accepted graph contract.

## JUL-06 — Reproduce strict Julia encoding

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-03, JUL-05
- Exact write set: `scripts/slm/julia/onnx_backend.py`, encoding parity tests
- Primary integration lock: Julia runner
- Outcome: Match reference Choice/Noul tokenization, option order, masks, padding, and overflow without truncation.
- Focused validation: Compare encoded tensors for 2/20 options, default/descriptive Noul, Unicode, mixed batches, and limits.
- Handoff: Remove JUL-06 and freeze encoded fixtures.

## JUL-07 — Implement ONNX CPU inference

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-06
- Exact write set: `scripts/slm/julia/onnx_backend.py`, ONNX inference tests
- Primary integration lock: Julia runner
- Outcome: Run explicit CPU provider inference and stable softmax over valid logits with recorded settings.
- Focused validation: Test provider selection, unsupported operators, repeated determinism, malformed outputs, and no fallback.
- Handoff: Remove JUL-07 and expose full-precision ONNX results.

## JUL-08 — Prove Python/ONNX parity

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-03, JUL-07
- Exact write set: `scripts/slm/julia/test_onnx_parity.py`, parity report
- Primary integration lock: Julia evaluation
- Outcome: Meet frozen tensor/logit/probability/decision tolerances and report every near-boundary difference.
- Focused validation: Run the complete development parity corpus without adjusting tolerances afterward.
- Handoff: Remove JUL-08 or create a bounded parity repair phase.

## JUL-09 — Package immutable Julia artifacts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-08
- Exact write set: Julia ModelKit manifest, versioned artifact registry, artifact record
- Primary integration lock: SLM artifact publication
- Outcome: Package graph, external weights, tokenizer, runtime, launcher, dependencies, and notices with hashes.
- Focused validation: Install on a clean Linux x86_64 host and run both explicit backends offline.
- Handoff: Remove JUL-09 and provide immutable asset facts to Rust.

## JUL-10 — Add JuliaSidecar

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-04, JUL-09
- Exact write set: `crates/acorn-lib/src/io/sidecar/slm/decision.rs`, decision sidecar tests
- Primary integration lock: shared sidecar root
- Outcome: Add typed Choice/Noul requests/results, assets, limits, provenance, and consuming lifecycle behind `slm`.
- Focused validation: Run fixture, asset, offline, timeout, cancellation, cleanup, redaction, and real-model smoke tests.
- Handoff: Remove JUL-10 and freeze the typed Rust interface.

## JUL-11 — Register decision routing and backend selection

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-05, JUL-10
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/needle.rs`, `crates/acorn-lib/src/io/mcp/tests/server.rs`, `crates/acorn-cli/src/commands/infer/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`, `crates/acorn-cli/src/commands/download/mod.rs`, `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/cli/tests/mod.rs`, `CommandLineHelp.md`
- Primary integration lock: CLI syntax and generated help
- Outcome: Register Julia decisions behind `acorn ask` and add `download slm decide` with explicit `onnx|python` selection. Preserve supplied questions/options and existing inference compatibility; Needle selects Julia and Julia returns the final advisory result.
- Focused validation: Run parser, input, option-count, backend/cache, custom-location, unsupported-host, and help tests plus actual Needle-to-Julia smoke and frozen routing/abstention cases.
- Handoff: Remove JUL-11 and freeze public syntax.

## JUL-12 — Complete decision lifecycle tests

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-11
- Exact write set: `tests/fixtures/slm/decision/`, decision protocol integration tests
- Primary integration lock: Julia acceptance
- Outcome: Cover readiness, IDs, sizes, startup, timeout, cancellation, cleanup, diagnostics, and explicit download/inference.
- Focused validation: Run all model-independent protocol tests without downloading weights.
- Handoff: Remove JUL-12 and preserve regression fixtures.

## JUL-13 — Evaluate Choice decisions

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-12
- Exact write set: frozen Choice corpus, Choice quality/calibration report
- Primary integration lock: Julia evaluation
- Outcome: Meet 90% answered accuracy, 80% coverage, and full-corpus baseline requirements.
- Focused validation: Run the held-out corpus once and report per-label metrics, confusion, calibration, and abstentions.
- Handoff: Remove JUL-13 or create a bounded evaluation repair phase.

## JUL-14 — Evaluate Noul and resource gates

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-12
- Exact write set: frozen Noul corpus, Julia resource report
- Primary integration lock: Julia evaluation
- Outcome: Pass independent Boolean metrics plus frozen latency, memory, bytes, threads, and parity ceilings.
- Focused validation: Run the held-out Noul corpus and measured cold/warm Linux x86_64 profile.
- Handoff: Remove JUL-14 or create a bounded evaluation repair phase.

## JUL-15 — Document and release the independent Julia profile

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: JUL-13, JUL-14
- Exact write set: `docs/src/inference/slm.md`, `docs/src/SUMMARY.md`, wiki indexes, this plan
- Primary integration lock: SLM release gate
- Outcome: Publish the advisory decision profile after its own gates pass; do not wait for Swift release.
- Focused validation: Run focused tests, lint/full suite, feature/WASM, license/dependency, help/docs, real-model smoke, and wiki validation.
- Handoff: Remove JUL-15 after documenting its release. Retire the plan and all index/network/lane entries only when every router, parity, adapter, and model phase is complete.
