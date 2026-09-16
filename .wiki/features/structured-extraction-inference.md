---
type: Plan
title: "Structured extraction inference"
description: "Add typed, provenance-preserving inference fallback for incomplete deterministic gather extraction."
difficulty: high
impact: medium
package: [workspace, acorn-lib, acorn-cli]
tags: [feature, gather, inference, ai, sidecar, provider]
status: planned
---

# Project code boundaries

- **Update:** Workspace and `acorn-lib` manifests, focused gather/scrape integration, one existing provider adapter selected during implementation, CLI arguments, dedicated tests, documentation, and generated help.
- **Create:** `crates/acorn-lib/src/inference/{mod,request,response,provider}.rs` and deterministic provider fixtures.
- **Delete:** None.

# Dependencies

This plan follows [deterministic gather scraping](deterministic-gather-scraping.md) and consumes its normalized extraction request, validated candidate boundary, transport limits, and provenance model. It must coordinate with the highest-priority ACP work before selecting an inference transport and must not take `io/sidecar`, JSON-RPC, MCP, or shared API integration locks during an active owner slice.

# Goal

When deterministic extraction leaves explicitly required fields unresolved, allow an explicitly selected inference provider to propose structured values that are schema-validated and clearly identified as non-deterministic output.

# Current evidence

- The existing `Sidecar` trait is `Sized`, has an associated output type, and consumes itself during inference; it is not a uniform object-safe provider registry.
- `ModelKitManifest` describes model execution metadata but does not define a general structured-inference wire protocol.
- Existing remote API configuration and Needle execution provide useful adapter patterns but do not prove a common extraction interface across arbitrary sidecars or providers.

# Decisions

- Define owned, versioned `InferenceRequest` and `InferenceResponse` DTOs before adding providers. Include input provenance, requested schema, prompt/template revision, provider/model identity, parameters, capabilities, token/output limits, and cancellation context.
- Keep the provider trait asynchronous and typed. Validate its response against the requested schema and the RAD candidate validators before merging any value.
- Implement one explicit adapter first, chosen to avoid conflict with active ACP and sidecar ownership. Do not promise `Anthropic`, `Auto`, arbitrary OpenAI-compatible endpoints, or `sidecar:<path|oci>` in the initial slice.
- Add a provider registry only after two concrete implementations demonstrate the same request, response, error, cancellation, and capability contract.
- Inference is explicitly enabled with `--inference PROVIDER`; omission guarantees no model call. Deterministic extraction always runs first.
- Describe the ordering as deterministic extraction followed by best-effort inference, not as deterministic output.
- Preserve raw deterministic values and inference proposals separately until validation succeeds. Record provider, model/revision, settings, schema, prompt/template revision, timestamps, and response provenance.
- Define cache keys from all behavior-affecting inputs and prevent credentials or sensitive content from entering cache metadata or diagnostics.

# Validation

- Contract-test schema validation, incomplete deterministic input, malformed and oversized model output, cancellation, timeout, provider errors, provenance, cache identity, and deterministic merge ordering through a fake provider.
- Test that no provider is called without explicit `--inference`, offline rejects remote providers, and rejected proposals do not modify deterministic candidates.
- Run independent provider-feature checks, lint, focused and full tests, audit, license checks, and feature-tree inspection.

# Acceptance criteria

- An explicitly selected provider receives a bounded, versioned extraction request only after deterministic extraction reports required fields missing.
- Provider output is treated as a proposal and cannot enter a candidate until schema and domain validation pass.
- Every accepted inferred value records provider, model/revision, settings, schema, prompt/template revision, and source provenance.
- The first implementation uses an explicit adapter; arbitrary sidecars and a generic registry are not claimed without a second proven implementation.
- Offline, cancellation, timeout, output-size, redaction, and cache policies fail closed.
- No Rhai execution or workflow hook is added by this plan.

# Follow-on work

Additional providers and automatic selection require their own evidence-backed slices after the common contract is demonstrated. Remote script access remains excluded by the implemented [Rhai exec policy](../../docs/src/packages/exec.md), not governed by this provider interface.
