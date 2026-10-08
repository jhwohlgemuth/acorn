# Feature Plans

## In progress

- [Model acquisition and metadata hardening](model-acquisition-hardening.md) - Complete identity-safe Hugging Face requests, exact GGUF metadata, and ModelKit/ModelPack interoperability evidence.
- [Small language model (SLM) sidecars](slm-sidecars.md) - Ship explicitly selected Tongue, Redact, and Title sidecars; plan a separately gated [Julia-1 decision sidecar](slm-sidecars.md#julia-1-decision-sidecar-planned-follow-on) with offline CPU packaging and ACORN quality gates.

## Waiting on validation

- [Power Automate logbook reporting deployment](powerapps-gitlab-logbook-reporting.md) - Clear the access gates, configure one pilot, and run the intake, graduation, publication, callback, and idempotency smoke test.

## Planned

- [Portable similarity measures, vocabulary coercion, and explicit comparison](similarity-measures.md) - Add bounded Rust primitives and an early vocabulary-resolution slice with explicit aliases, ambiguity reporting, portable parity, and loss-aware formatting/enrichment; expose pairwise comparison before evaluating duplicate suggestions.

- [WebMCP browser validation and readability tools](webmcp-browser-tools.md) - Expose the existing WASM bindings through an optional browser adapter with bounded structured results, a demonstration, and native browser verification.

- [Release-gated Beam inference integration](beam-inference-integration.md) - Verify released serving contracts, then evaluate an explicitly configured hosted or operator-owned endpoint independently of local SLM sidecars.

- [Database-backed artifact generation](database-backed-artifact-generation.md) - Generate reproducible current or historical artifacts from canonical database state through existing typed renderers.
- [CEL metadata validation-profile pilot](cel-validation-profile-pilot.md) - Evaluate bounded, optional institutional rules against existing Rust/Rhai behavior before adopting a CEL runtime; see the [OPA/CEL review](../assets/opa-cel-review.md).
- [dPID research-object support](dpid-research-object-support.md) - Recognize and resolve versioned dPIDs, import resolver RO-Crate metadata into RAD, and preserve manifest provenance through a read-only first release.
- [Deterministic and assisted gather extraction](gather-extraction.md) - Add bounded CSS extraction first, then an explicitly selected typed inference fallback for unresolved required fields.
- [Azure Cosmos DB bucket downloads](azure-cosmos-buckets.md) - After the provider-neutral source prerequisite, add a Cosmos DB for NoSQL container or logical partition as a read-only bucket source.
