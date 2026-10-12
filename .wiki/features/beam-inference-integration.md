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
execution: phased
phase_budget_hours: 5
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
- Follow the [Provider/model leaves lane](../parallel-execution.md). Workspace changes, if proven necessary, use the workspace-graph lock defined there. Beam does not depend on Julia, Noul, ONNX, or Needle release validation.

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

# Execution phases

## BEAM-01 — Verify release, license, and hosting evidence

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: published Beam release and serving contract
- Exact write set: this plan, dated Beam evidence record
- Primary integration lock: Beam adoption decision
- Outcome: Record immutable identity, license, API, limits, retention, cost, and supported hosting from primary sources.
- Focused validation: Cross-check every release gate and stop promotion if any required evidence is absent.
- Handoff: Remove BEAM-01 or retain the plan blocked without implementation.

## BEAM-02 — Capture authorized protocol fixtures

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BEAM-01 and an authorized deployment
- Exact write set: `tests/fixtures/inference/beam/`, fixture attribution record
- Primary integration lock: Beam interoperability fixtures
- Outcome: Capture sanitized success, refusal, incomplete, malformed, auth, rate, timeout, oversize, and offline cases.
- Focused validation: Verify credentials/source content are absent and expected semantics are explicit.
- Handoff: Remove BEAM-02 and freeze the deployment profile.

## BEAM-03 — Prove existing inference compatibility

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BEAM-02
- Exact write set: `crates/acorn-lib/tests/openai_interoperability.rs`, Beam fixtures
- Primary integration lock: inference interoperability
- Outcome: Exercise explicit endpoint/model selection through the current interface before adding code.
- Focused validation: Run all sanitized fixtures, unsupported-capability rejection, redaction, and offline behavior.
- Handoff: Remove BEAM-03 and record configuration-only success or exact protocol gaps.

## BEAM-04 — Add only the proven adapter gap

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BEAM-03 reports an incompatibility
- Exact write set: `crates/acorn-lib/src/io/api/openai/inference.rs`, Beam adapter tests
- Primary integration lock: inference API
- Outcome: Implement the smallest reusable protocol difference without a registry or automatic fallback.
- Focused validation: Run existing OpenAI and Beam fixtures to prove no regression or silent capability stripping.
- Handoff: Remove BEAM-04; prune it entirely when BEAM-03 proves configuration-only integration.

## BEAM-05 — Freeze the ACORN evaluation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BEAM-03 and named evaluation owner
- Exact write set: frozen 100-example corpus, reviewer rubric, prompt/model/budget policy
- Primary integration lock: Beam evaluation
- Outcome: Predeclare baseline, 90% acceptance, zero material unsupported claims, cost, latency, and failure accounting.
- Focused validation: Review held-out separation, denominator rules, and owner approval.
- Handoff: Remove BEAM-05 and prohibit post-hoc gate changes.

## BEAM-06 — Complete offline conformance coverage

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BEAM-03
- Exact write set: Beam interoperability tests, Beam fixtures
- Primary integration lock: inference interoperability
- Outcome: Cover the supported profile entirely offline, including refusals, truncation, schema failures, redaction, and limits.
- Focused validation: Run focused interoperability/configuration tests with network disabled.
- Handoff: Remove BEAM-06 and preserve the protocol oracle.

## BEAM-07 — Add explicit invocation documentation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BEAM-04 or configuration-only result from BEAM-03
- Exact write set: `docs/src/inference/beam.md`, `docs/src/SUMMARY.md`, conditional CLI configuration docs
- Primary integration lock: book documentation
- Outcome: Document explicit endpoint/model selection, capabilities, limitations, data handling, and deployment responsibility.
- Focused validation: Run docs/help generation, internal links, and offline example configuration.
- Handoff: Remove BEAM-07 and preserve the tested hosting profile.

## BEAM-08 — Run live smoke and frozen evaluation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BEAM-05, BEAM-06, BEAM-07
- Exact write set: authorized Beam deployment, frozen evaluation results
- Primary integration lock: Beam evaluation
- Outcome: Measure the explicit profile against quality, refusal, completeness, latency, and cost gates.
- Focused validation: Run one bounded live smoke and the frozen held-out evaluation without dropping failures.
- Handoff: Remove BEAM-08 or record the failed adoption gate without automatic workflow use.

## BEAM-09 — Decide support and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BEAM-08
- Exact write set: Beam support documentation, wiki indexes, this plan
- Primary integration lock: Beam adoption decision
- Outcome: Release only the passing profile or prune implementation artifacts, then remove the plan.
- Focused validation: Run focused tests, portability/help as applicable, `make lint`, `make test`, docs, and wiki validation.
- Handoff: Delete this plan and every index, network, and lane entry.
