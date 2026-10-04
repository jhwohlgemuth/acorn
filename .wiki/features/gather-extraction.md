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
