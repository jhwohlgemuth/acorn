---
okf_version: "0.1"
type: Plan
title: Release-gated Beam inference integration
description: Evaluate Beam through an explicitly configured inference endpoint after release and serving contracts are verified.
difficulty: high
impact: medium
package: [acorn-lib, acorn-cli]
tags: [feature, inference, beam, api, evaluation]
status: planned
---

# Goal

Enable an explicitly selected Beam endpoint for advisory research-text assistance in ACORN. Start with one bounded request over user-supplied text through the existing inference command. Prefer configuration and interoperability tests over a new provider implementation when the released service matches an existing protocol.

This plan is separate from [SLM sidecars](slm-sidecars.md): Beam is a proposed externally managed inference service, while Julia provides local finite-choice decisions. No automatic fallback from Julia to Beam, tool execution, canonical RAD mutation, or background inference belongs to this scope.

# Evidence and release gates

Evidence reviewed on 2026-10-06. The [Reflection announcement](https://reflection.ai/blog/introducing-beam) describes Beam as a coding/reasoning mixture-of-experts model with 501 billion total parameters and 23 billion active. It offers early-access registration and promises weights, a model card, technical report, and developer artifacts later in October under Apache 2.0. These are announced deliverables, not verified released artifacts. The article describes controllable reasoning effort but does not establish the serving API contract needed here.

Before integration, verify the actual release, license text, immutable model revision, supported serving engine, endpoint protocol, authentication, context/output limits, reasoning controls, structured-output support, retention terms, and deployment requirements. Do not infer API compatibility from model capabilities. Record unknowns as unresolved; no invented endpoint URL, model identifier, release checksum, or pricing.

The active parameter count describes computation per token, not the total weight-storage requirement. A managed desktop sidecar is outside scope. An operator-owned service requires documented capacity and deployment evidence; ACORN will not provision hardware, download Beam weights, or manage its server lifecycle in this plan.

Local evidence: `crates/acorn-lib/src/io/api/openai/mod.rs` exposes Chat Completions and Responses endpoint modes. Its `inference.rs` client supports explicit model selection, bounded responses, deadlines, schema validation, credential checks, and remote-offline rejection. Structured output is an explicit capability; tools, streaming, and attachments default to unsupported. These existing contracts are integration candidates, not proof that Beam supports either mode.

# Decisions

- First consumer: explicit advisory summarization of supplied research prose, with human evaluation for unsupported claims and omitted key facts. Name an evaluation owner and freeze the task/corpus before promotion. Structured metadata proposals are a later consumer of the gather plan's validated boundary.
- Select endpoint, model, credentials, and supported protocol explicitly through existing configuration. Prefer the current inference path; add a Beam-specific adapter only for verified incompatibilities. No generic registry or new crate is justified by this proposal.
- Separate hosted Reflection access from operator-owned serving profiles. Each profile needs its own compatibility, data handling, latency, and availability evidence. Early-access signup alone does not pass the production gate.
- Preserve shared HTTPS, credential redaction, response-size, timeout, cancellation, and offline policy. Remote services remain unavailable offline, including privately hosted remote endpoints; retain existing loopback semantics only for an explicitly configured local service.
- Send only explicitly supplied task content. Do not upload whole repositories, cached records, secrets, or unrelated source documents. Record model/deployment identity, prompt revision, parameters, finish reason, and usage when available without logging sensitive input.
- Advertise only tested capabilities. Reject unsupported reasoning settings, attachments, tools, or structured-output requirements before sending a request. Do not silently strip requested constraints or retry through another provider.
- Treat refusals, incomplete output, schema failures, transport errors, and successful results distinctly. No retry after ambiguous completion unless duplicate execution/cost behavior is understood and bounded. Generated text cannot authorize actions or establish factuality.
- Hosted cost and self-hosted memory/throughput budgets must be set from verified deployment evidence before live evaluation. Keep ordinary tests offline; live checks are opt-in and bounded.

# Boundaries and dependencies

- **Primary candidate updates:** `crates/acorn-lib/tests/openai_interoperability.rs` for verified compatibility tests; `crates/acorn-lib/src/io/api/openai/inference.rs` only for a demonstrated reusable protocol gap; `crates/acorn-cli/src/commands/inference.rs` only if existing configuration cannot select the verified endpoint/model.
- **Proposed additions:** `docs/src/inference/beam.md` and dedicated fixtures under `tests/fixtures/inference/beam/`; enumerate exact fixture files before implementation. Update `docs/src/SUMMARY.md` under the documentation lock.
- **Conditional integration surfaces:** API/configuration roots, endpoint snapshots, CLI parsing/generated help, and manifests require an exact write set and their named locks. Do not edit all candidate files by default. No deletion is planned.
- Consume the [shared API client contract](../refactors/api/shared-client-contract.md) and coordinate changes with its owner. Existing landed transport functionality can be evaluated without waiting for the entire refactor.
- Any gather integration is owned by [gather extraction](gather-extraction.md) and follows its typed validation/provenance contract; it is not required for the first explicit inference consumer.
- Follow the [Provider/model leaves lane](../parallel-execution.md). Workspace changes, if proven necessary, follow [dependency capabilities](../refactors/portability/dependency-features-and-macos-size.md). Beam does not depend on Julia, Noul, ONNX, or Needle release validation.

# Sequence and stop gates

1. **Verify release and deployment.** Record dated primary-source evidence for artifacts, licensing, API shape, limits, retention, cost, and supported hosting. If unavailable, retain planned status and stop implementation promotion; do not create a polling automation implicitly.
2. **Prove compatibility.** Capture sanitized request/response fixtures against one authorized deployment. Test existing configuration first. Document capability differences and choose either configuration-only integration or the smallest adapter change.
3. **Freeze evaluation.** Name the task owner; freeze at least 100 held-out research-text examples, reviewer rubric, baseline, prompt/model settings, and numeric cost/latency budgets. Proposed quality gates: at least 90% reviewer acceptance and zero material unsupported claims on that set. Report omissions, refusals, and incomplete outputs separately; do not drop failed examples from the denominator.
4. **Integrate and document.** Land narrowly scoped fixtures/configuration or adapter changes, explicit invocation instructions, supported capabilities, limitations, and deployment responsibility. Add no automatic workflow adoption.
5. **Validate the supported profile.** Run offline conformance tests, then explicit live smoke and frozen quality/resource evaluation. Release only the tested deployment profile; another engine/provider needs separate evidence.

# Validation and acceptance criteria

- [ ] Published license, immutable model/deployment identity, serving contract, limits, and data-handling terms are documented from primary sources.
- [ ] Explicit endpoint/model selection works through the existing inference interface or a justified minimal adapter; unsupported capabilities fail before transport.
- [ ] Fixtures cover successful text, refusal, truncated/incomplete output, malformed envelopes, schema errors when supported, authentication failures, rate limits, timeouts, oversized responses, credential redaction, and offline rejection.
- [ ] No model download, hardware provisioning, implicit remote fallback, tool execution, or canonical record mutation is introduced.
- [ ] The frozen ACORN quality gates and predeclared latency/cost budgets pass; vendor benchmarks do not substitute for task evaluation.
- [ ] Focused interoperability/configuration tests, `make lint`, `make test`, and applicable portability/generated-help checks pass for implementation changes. Live tests stay opt-in.
- [ ] Documentation identifies the tested hosting profile and operator responsibilities. Remove this plan and its index/network/lane entries only after acceptance and durable documentation are complete.
