---
okf_version: "0.1"
type: Plan
title: Deterministic and assisted gather extraction
description: Add bounded CSS extraction to gather, then an explicitly selected typed inference fallback for unresolved required fields.
difficulty: high
impact: high
package: [workspace, acorn-lib, acorn-cli, acorn-tui]
tags: [feature, gather, scraping, html, css, inference, provenance]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Give `acorn gather` one extraction pipeline with two ordered modes:

1. bounded HTTP fetching and deterministic CSS-to-RAD field extraction; and
2. an opt-in inference proposal only for required fields that deterministic extraction did not resolve.

Both modes feed the existing validated candidate boundary. Deterministic values remain authoritative, inferred values remain attributable proposals, and omission of `--inference` guarantees that no model is called.

# Current evidence

- Gather already accepts files, directories, URLs, and text and performs identifier discovery, but it has no HTML field-mapping contract.
- ACORN has shared HTTP and offline policy. The global `--threads` setting controls Rayon CPU work and is not a network-concurrency contract.
- `Sidecar` and `ModelKitManifest` provide useful concrete adapter patterns but do not establish a uniform structured-extraction provider registry.
- TUI gather duplicates loading, discovery, persistence, and offline behavior that already exists in shared source and record APIs.
- The implemented Rhai boundary does not authorize remote script access and is not part of this feature.

# Project code boundaries

- **Update:** focused gather and discovery modules, shared HTTP-policy integration, CLI/TUI gather composition, workspace and `acorn-lib` feature wiring, documentation, and generated help when syntax changes.
- **Create:** `crates/acorn-lib/src/scrape/{mod,config,extract,fetch}.rs`, then `crates/acorn-lib/src/inference/{mod,provider,request,response}.rs`; dedicated fixtures under `tests/fixtures/scrape/` and `tests/fixtures/inference/`.
- **Delete:** TUI-local loading, HTTP, and persistence paths after parity tests prove it consumes the shared pipeline. Do not delete deterministic values when inference is enabled.

# Dependencies

The deterministic slice consumes the existing HTTP/offline and source-document boundaries. The inference slice starts only after the deterministic request, validation, and provenance contracts pass. Provider selection must honor the active sidecar integration lock and treat the implemented ACP and MCP contracts as read-only.

# Decisions

## Deterministic extraction

- Add `--scrape` and repeatable `--selector FIELD=CSS` arguments. Split on the first `=` so valid CSS may contain colons and equals signs.
- Validate `FIELD` before fetching. Specify text versus attribute extraction, zero/one/many match behavior, document order, whitespace normalization, duplicate handling, and required-field behavior.
- Parse the original response. Sanitization is unnecessary because this slice neither trusts markup for rendering nor sends it to a model.
- Use a scraping-specific asynchronous concurrency limit. Bound response bytes, decompression, accepted media and encodings, redirects, destinations, timeouts, retries, private/link-local addresses, credential forwarding, and cache metadata.
- Record source URI, final URI, retrieval time, selector mapping, and each extracted value's provenance.

## Assisted extraction

- Define owned, versioned `InferenceRequest` and `InferenceResponse` DTOs before adding a provider. Include source provenance, requested schema, provider/model identity, prompt/template revision, parameters, limits, and cancellation context.
- Implement one explicit typed adapter. Add a registry only after a second implementation proves the same request, response, error, cancellation, and capability contract.
- Run inference only when `--inference PROVIDER` is present and deterministic extraction reports a required field missing.
- Keep deterministic values and proposals separate until schema and domain validation succeed. A rejected proposal cannot modify the candidate.
- Cache identity includes every behavior-affecting input and excludes credentials and sensitive content from metadata and diagnostics.

## Interface parity

The CLI and TUI call the same source, extraction, discovery, validation, persistence, HTTP, and offline services. TUI work runs off the event loop and does not retain a direct `reqwest` dependency after migration.

# Implementation sequence

1. Land the normalized selector request, pure extraction rules, fixtures, and validation tests.
2. Add bounded fetching and CLI composition, then migrate TUI gather to the shared pipeline.
3. Freeze the typed inference request/response contract and implement one fake-backed provider adapter.
4. Integrate explicit inference after deterministic extraction, add provenance and cache tests, and document the combined ordering.

Each step must pass before the next begins. The inference slice cannot weaken a deterministic transport or validation failure into a model fallback.

# Validation

- Test selector syntax, CSS containing colons and equals signs, zero/one/many matches, attribute extraction, normalization, stable ordering, and candidate validation.
- Test offline rejection, redirect and SSRF policy, oversized and decompression-bomb responses, content types, timeouts, credential stripping, and bounded concurrency with a local fake server.
- Contract-test malformed and oversized model output, cancellation, timeout, provider failure, cache identity, provenance, and deterministic merge ordering through a fake provider.
- Prove no provider call occurs without `--inference`, rejected proposals do not mutate deterministic candidates, and CLI/TUI local and remote behavior agrees.
- Run scrape-only, inference-only, combined, default, no-default, all-feature, and WASM checks; inspect feature trees; then run lint, focused tests, the full suite, help generation, and dependency/license checks.

# Acceptance criteria

- `acorn gather URL... --scrape --selector FIELD=CSS` produces deterministically ordered, validated candidates under explicit transport limits.
- Selector and field errors fail before any fetch; offline, redirect, SSRF, size, credential, and content-type policy fail closed.
- `--inference PROVIDER` invokes one explicit typed provider only for unresolved required fields after deterministic extraction.
- Accepted inferred values retain provider, model/revision, settings, schema, prompt/template revision, source, and response provenance.
- Deterministic values always win; rejected or unavailable inference never changes them.
- CLI and TUI share loading, extraction, discovery, persistence, HTTP, and offline behavior without blocking the TUI event loop.
- Scraping and provider dependencies remain absent unless their independent capabilities are enabled.
- No Rhai hook, generic sidecar registry, remote script access, workflow hook, or automatic model selection is introduced.

# Execution phases

## GAT-01 — Define normalized selector requests

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-lib/src/io/gather/extraction.rs`
- Primary integration lock: gather extraction service
- Outcome: Parse first-`=` field/CSS mappings and freeze field, text/attribute, cardinality, order, whitespace, duplicate, and required rules.
- Focused validation: Run valid/invalid selector and normalization table tests.
- Handoff: Remove GAT-01 and expose immutable selector requests.

## GAT-02 — Implement pure deterministic extraction

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: GAT-01
- Exact write set: `crates/acorn-lib/src/io/gather/extraction.rs`, `tests/fixtures/gather/extraction.html`
- Primary integration lock: gather extraction service
- Outcome: Extract ordered values and per-value provenance from supplied HTML without transport or inference.
- Focused validation: Run zero/one/many, attribute, Unicode, malformed markup, duplicate, and required-field tests.
- Handoff: Remove GAT-02 and freeze deterministic results.

## GAT-03 — Add bounded asynchronous fetching

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: GAT-02
- Exact write set: `crates/acorn-lib/src/io/gather/fetch.rs`
- Primary integration lock: gather transport
- Outcome: Enforce dedicated concurrency, bytes, decompression, media, redirects, destinations, retries, timeouts, and cache provenance.
- Focused validation: Run local-server tests for every limit, redirect, private address, encoding, and offline rejection.
- Handoff: Remove GAT-03 and expose fetched original responses to extraction.

## GAT-04 — Add CLI scraping composition

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: GAT-03
- Exact write set: `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/commands/gather/mod.rs`, `CommandLineHelp.md`, `crates/acorn-cli/src/commands/gather/tests/mod.rs`
- Primary integration lock: CLI syntax and generated help
- Outcome: Add explicit scrape/selectors and compose fetching, extraction, discovery, validation, and persistence.
- Focused validation: Run parser, command, offline, transport-failure, deterministic-output, and help tests.
- Handoff: Remove GAT-04 and freeze CLI behavior.

## GAT-05 — Migrate TUI gather to the shared pipeline

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: GAT-04
- Exact write set: `crates/acorn-tui/src/screens/gather.rs`, `crates/acorn-tui/src/screens/gather/tests.rs`
- Primary integration lock: TUI gather screen
- Outcome: Reuse shared source/extraction/discovery/persistence off the event loop and remove direct HTTP ownership.
- Focused validation: Test responsiveness, cancellation, offline behavior, persistence, and CLI/TUI parity.
- Handoff: Remove GAT-05 and record the single shared service boundary.

## GAT-06 — Freeze typed inference DTOs

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: GAT-02
- Exact write set: `crates/acorn-lib/src/io/gather/inference.rs`
- Primary integration lock: gather inference contract
- Outcome: Define versioned requests/responses with source, schema, provider/model, prompt, limits, cancellation, and provenance.
- Focused validation: Run strict serde, identity, limit, unknown-field, and fingerprint tests.
- Handoff: Remove GAT-06 and expose the provider-neutral contract.

## GAT-07 — Implement one fake-backed provider adapter

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: GAT-06
- Exact write set: `crates/acorn-lib/src/io/gather/inference.rs`
- Primary integration lock: gather inference contract
- Outcome: Prove request, response, error, cancellation, and capability behavior without a registry.
- Focused validation: Test success, refusal, malformed, timeout, cancellation, oversize, and redacted diagnostics.
- Handoff: Remove GAT-07 and retain the concrete adapter boundary.

## GAT-08 — Enforce deterministic-first fallback ordering

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: GAT-04, GAT-07
- Exact write set: `crates/acorn-lib/src/io/gather/mod.rs`
- Primary integration lock: gather extraction service
- Outcome: Invoke explicit inference only for unresolved required fields and never on deterministic transport/validation failure.
- Focused validation: Run complete, missing, failed-fetch, failed-validation, unavailable-provider, and offline cases.
- Handoff: Remove GAT-08 and freeze fallback semantics.

## GAT-09 — Validate proposals and cache identity

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: GAT-08
- Exact write set: `crates/acorn-lib/src/io/gather/mod.rs`
- Primary integration lock: gather extraction service
- Outcome: Keep proposals separate until schema/domain validation and key cache entries by every behavior-affecting input.
- Focused validation: Test rejected proposals, deterministic precedence, credential exclusion, stale fingerprints, and cache replay.
- Handoff: Remove GAT-09 and expose validated candidate merges.

## GAT-10 — Document the combined pipeline

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: GAT-05, GAT-09
- Exact write set: `docs/src/cli-reference/commands/gather.md`, `docs/src/cli-reference/commands/tui.md`, `CommandLineHelp.md`
- Primary integration lock: gather documentation
- Outcome: Document selector rules, transport limits, provenance, explicit inference, precedence, cache, and failure behavior.
- Focused validation: Run docs/help generation and example commands offline.
- Handoff: Remove GAT-10 and preserve the public contract.

## GAT-11 — Run full gates and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: GAT-10
- Exact write set: `.wiki/features/gather-extraction.md`, `.wiki/index.md`, `.wiki/features/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: gather acceptance
- Outcome: Complete feature/portable/full validation and remove the plan.
- Focused validation: Run focused CLI/TUI/library tests, portability, `make lint`, `make test`, `make doc`, and wiki validation.
- Handoff: Delete this plan and every index, network, and lane entry.
