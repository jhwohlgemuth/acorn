---
type: Plan
title: "Add a contract-gated SciCrunch RRID provider"
description: "Add typed SciCrunch RRID lookup and optional bounded registry search only after a reviewed machine-API contract satisfies ACORN's lifecycle, security, and offline requirements."
difficulty: high
impact: high
package: [acorn-schema, acorn-lib, acorn-cli, acorn-tui]
tags: [feature, rrid, scicrunch, api, provider, resolution, search, security]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-schema/src/discovery.rs` for `RemoteEntity::Resource`; `crates/acorn-lib/src/io/api/mod.rs`, `crates/acorn-lib/src/io/config/application.rs`, `crates/acorn-lib/assets/constants/application.json`, focused HTTP/redaction tests, and `crates/acorn-lib/src/analyzer/host/discovery/{mod.rs,tests/}`; optional bounded search integration in focused `crates/acorn-cli` and `crates/acorn-tui` files only after lookup is stable, with shared roots and generated help owned by their integration lanes.
- **Create:** `.wiki/assets/rrid-scicrunch-contract-matrix.md`; pinned synthetic and reviewed response fixtures under `tests/fixtures/rrid/scicrunch/`; `crates/acorn-lib/src/io/api/rrid.rs` and focused API tests; provider-specific CLI/TUI leaf tests only if optional search is approved.
- **Delete:** None. Do not replace generic discovery lifecycle or persistence contracts, and do not remove normalized local RRID evidence on provider failure.

# Dependencies

- [Portable RRID research-resource support](rrid-research-resource-support.md) has landed its strict parser, canonical identifier, `PID::RRID`, non-project identity policy, and local discovery behavior. Lookup integration depends on those contracts remaining stable.
- Reuse the current `api::Secret`, endpoint override, request redaction, offline rejection, retry bounds, and `ResolutionOutcome`/`ResolutionStatus` lifecycle contracts. Coordinate shared API files with the [API roadmap](../refactors/api/roadmap.md) and [provider capability matrix](../refactors/api/provider-capability-matrix.md).
- `RemoteProvider::Rrid` belongs to the host discovery provider enum; `RemoteEntity::Resource` belongs to the portable provider-neutral discovery schema. Their shared enum roots are integration locks.
- Registry search is downstream of stable exact lookup and an approved pagination/ranking contract. CLI and TUI search are optional bounded consumers, not prerequisites for resolution.

# Goal

Confirm and enrich locally parsed RRIDs through a typed SciCrunch machine API, then optionally expose bounded registry search without scraping HTML, leaking credentials, treating resources as projects, or making local RRID support depend on network access.

# Contract gate (2026-09-07)

No production implementation may begin until a reviewed matrix pins all of the following:

1. A supported production endpoint and revision or dated contract source.
2. Authentication placement and handling that satisfies URL, trace, error, fixture, and persistence redaction.
3. Stable request fields for exact RRID lookup and, separately, free-text search.
4. Stable response fields, identifier canonicalization, resource lifecycle/availability semantics, and unknown-field policy.
5. Pagination, provider ranking, maximum page size, total-result semantics, and hard client-side request/result bounds.
6. Status behavior for known, absent, unauthorized, forbidden, rate-limited, unavailable, malformed, and server-error responses.
7. Quotas, `Retry-After` behavior, acceptable-use terms, licensing/provenance, and permission to retain the planned metadata.
8. Credential environment naming, supported header or query placement, endpoint overrides, and operational ownership.

Current public material documents an Elasticsearch pass-through, production aliases, `size`/`from` pagination examples, exact `rrid.curie` queries, and a query-string API key. It does not establish the complete stable-field, quota/terms, pagination-bound, lifecycle, or credential-handling contract required above ([SciCrunch RIN overview](https://github.com/SciCrunch/API-Handbook/blob/master/resources-information-network/overview.md), [RIN search examples](https://github.com/SciCrunch/API-Handbook/blob/master/resources-information-network/basic-rin-search-examples.md)). The RRID system points API users toward SciCrunch but does not close those gaps ([RRID system](https://www.rrids.org/rrid-system)). Therefore this plan remains gated and `planned`.

If the gate cannot pass, do not add experimental production DTOs, portal scraping, or an unreviewed credential flow. Retain portable RRID support and document the external portal as the search workflow.

# Current evidence

- `crates/acorn-lib/src/io/api/mod.rs` exposes `api::Secret` as `secrecy::SecretString` and central API request/configuration contracts. Existing provider leaves demonstrate endpoint overrides and fixture-backed request/response testing.
- `crates/acorn-lib/src/analyzer/host/discovery/mod.rs` currently has `RemoteProvider::Osti` only. Its resolution model distinguishes `NotRequested`, `Unsupported`, `Resolved`, and `Failed`; RRID must preserve those lifecycle states.
- `crates/acorn-schema/src/discovery.rs` has provider-neutral `Project`, `Person`, `Organization`, and `Repository` entities but no `Resource` variant.
- Generic discovery persistence already stores optional serialized metadata and lifecycle status. Provider work should enrich that record, not migrate it or create a research-activity candidate.
- SWHID resolution at baseline `50e319e1` is the precedent for keeping strict parsing in `acorn-schema`, provider I/O in `acorn-lib`, retaining the normalized PID on failure, and covering status behavior with local fixtures rather than default live tests.
- DataCite and JATS interchange are owned by the portable plan and standards alignment; this provider does not author relations or silently mutate canonical RAD from registry metadata.

# Recommendation

Fail the provider gate closed. Approve exact lookup and search separately, implement lookup first from pinned typed fixtures, and add resource search consumers only after pagination, ranking, quota/terms, and credential handling are documented. If those contracts remain incomplete, ship no provider code and keep the portable RRID baseline unchanged.

# Provider contract

| Concern | Required behavior |
|---|---|
| Exact lookup | Query only the reviewed exact-identifier field. Accept success only when a typed response identifies the requested or contract-equivalent canonical RRID. |
| DTO boundary | Model only reviewed stable fields in typed request/response DTOs; preserve a lossless reviewed raw payload where provenance requires it. |
| Normalized result | Expose canonical RRID, name, resource type, proper citation, catalog number, canonical/alternate URLs and identifiers, availability, and alerts only when the contract defines them. |
| Lifecycle | Map success, absence, authentication, rate limit, transport, malformed payload, and identifier mismatch to explicit typed outcomes while retaining the local RRID. |
| Secrets | Reuse `api::Secret`; redact credentials from URL strings, debug output, errors, traces, fixtures, and persisted metadata. Query-string authentication is unacceptable unless shared transport proves complete redaction and no safer supported form exists. |
| Endpoint | Use the existing endpoint model and local mock overrides. Do not hard-code test endpoints or contact SciCrunch in the default suite. |
| Offline | Reject or mark unsupported before constructing or sending a request. Local parsing and discovery remain available. |
| Retry | Reuse bounded transport retry, honor reviewed `Retry-After` behavior, and prohibit busy loops or unbounded all-pages traversal. |
| Search | Build fixed typed queries over allowlisted fields. Never expose raw Elasticsearch DSL; preserve provider ranking and reviewed pagination semantics. |
| Persistence | Store provider metadata as discovery provenance. Never create a project candidate or overwrite authored `TypedRelatedResource` values. |

# Implementation plan

Each slice must name exact files and its shared integration owner before implementation.

## 0. Review and pin the SciCrunch contract

**Write set:** new `.wiki/assets/rrid-scicrunch-contract-matrix.md`, reviewed immutable fixtures under `tests/fixtures/rrid/scicrunch/contract/`, and this plan. No Rust production files.

- Pin source revisions, retrieval dates, endpoints/aliases, authentication, field definitions, pagination, status behavior, quotas, terms, provenance, and representative responses.
- Record exact lookup, unknown/unavailable identifiers, authentication failures, rate limits, partial or malformed payloads, unknown enum values, and identifier mismatches.
- Prove the planned credential placement can be redacted by ACORN's existing request, error, tracing, and persistence paths.
- Record a pass/fail decision for lookup and a separate decision for search. Search may remain deferred even if lookup passes.

**Deliverable:** a reviewed contract matrix with explicit gate decisions. A failed gate produces documentation only.

## 1. Add typed exact lookup

**Gate:** Slice 0 approves lookup, stable fields, lifecycle semantics, and credential handling.

**Primary write set:** new `crates/acorn-lib/src/io/api/rrid.rs` and focused tests; integration-owned `crates/acorn-lib/src/io/api/mod.rs`, `crates/acorn-lib/src/io/config/application.rs`, and `crates/acorn-lib/assets/constants/application.json`.

- Implement immutable typed request and response DTOs from the pinned contract.
- Require canonical-identifier agreement and distinguish absent records from malformed or mismatched responses.
- Normalize only approved stable fields and retain reviewed raw provider provenance without credentials.
- Cover 2xx, authentication failures, absence, conflict if documented, 429 with `Retry-After`, 5xx, timeout, redirect policy, malformed bodies, missing fields, unknown values, and identifier mismatch with local fixtures.
- Verify endpoint override, secret redaction, bounded retry, response-size limits, and zero live-network dependency.

**Deliverable:** fixture-backed exact lookup with typed, credential-safe outcomes.

## 2. Integrate resolver lifecycle

**Gate:** exact lookup is stable and green.

**Primary write set:** the RRID resolution branch and focused tests in `crates/acorn-lib/src/analyzer/host/discovery/`; focused CLI resolve tests only where existing behavior needs coverage.

- Route `PID::RRID` through typed lookup when resolution is requested.
- Preserve the canonical local RRID on every result and store provider enrichment only as discovery metadata/provenance.
- Map outcomes into the existing lifecycle contract without treating portal redirects as success.
- Reject offline resolution before request construction and prove no request is sent.
- Keep `PID::RRID` outside project candidates and canonical RAD mutation.

**Deliverable:** opt-in RRID resolution reports trustworthy lifecycle states and never changes project identity.

## 3. Add provider-neutral resource search types

**Gate:** Slice 0 separately approves search fields, pagination, ranking, quotas, and query surface; Slice 2 is green.

**Primary write set:** `RemoteEntity::Resource` in `crates/acorn-schema/src/discovery.rs`, `RemoteProvider::Rrid` and resource-result integration in the RRID discovery provider leaf, plus focused tests.

- Add resource semantics without forcing registry results through project, person, organization, or repository shapes.
- Define a provider-neutral resource result no wider than the stable lookup/search contract.
- Preserve provider ordering and canonical RRID deduplication.

**Deliverable:** stable typed resource results that can be consumed without project persistence.

## 4. Optionally expose bounded CLI/TUI search

**Gate:** Slice 3 is stable, usability is reviewed, and CLI/TUI integration owners approve exact write sets.

**Primary write set:** provider-specific CLI/TUI leaf files and tests; shared arguments, roots, snapshots, and generated help remain integration-owned.

- Add exact-RRID lookup and free-text search through fixed allowlisted queries and hard request/result/page bounds.
- Preserve provider ranking and pagination; do not locally re-rank or expose raw Elasticsearch syntax.
- Show canonical RRID, resource type, proper citation, availability, alerts, loading, empty, and error states where supplied by the approved contract.
- Disable search offline before spawning work and cover empty query, no results, duplicates, unavailable resources, rate limits, malformed records, and partial-page failures.
- Keep results report-only until a user explicitly authors a `TypedRelatedResource`; never create a project candidate.

**Deliverable:** if approved, bounded registry search with consistent CLI/TUI behavior. Otherwise this slice remains deferred without blocking lookup.

## 5. Document and harden provider delivery

**Primary write set:** provider documentation, acceptance fixtures, and generated help only if Slice 4 changes CLI behavior.

- Document credentials, endpoint selection, exact lookup versus search, lifecycle states, offline behavior, bounds, availability/alerts, provenance, and the portable fallback.
- Keep all default tests local and synthetic. Live contract probes, if retained, are explicit opt-in diagnostics with synthetic-safe output.
- Re-run API, resolver, security, optional CLI/TUI, and workspace gates.

**Deliverable:** a documented provider surface that can be disabled or absent without changing portable RRID behavior.

# Delivery order

1. Review and pin the provider contract.
2. Implement typed exact lookup.
3. Integrate resolution lifecycle.
4. Add resource provider types only if the search contract passes.
5. Optionally add bounded CLI/TUI search.
6. Complete provider documentation and repository gates.

# Validation

Focused gates after the corresponding slices exist:

```sh
cargo nextest run -p acorn-lib 'io::api::rrid'
cargo nextest run -p acorn-lib 'analyzer::host::discovery'
cargo nextest run -p acorn-schema 'discovery'
cargo nextest run -p acorn-cli 'commands::gather'
cargo nextest run -p acorn-tui 'screens::gather'
```

The CLI/TUI commands apply only if Slice 4 lands. Repository gates are `cargo fmt --all`, `make lint`, and `make test`; regenerate help only after CLI contract changes and run dependency/audit gates only if dependencies change.

Additionally assert zero default live requests, complete credential redaction, endpoint override behavior, bounded retry and pagination, preservation of normalized RRIDs on every outcome, no project-candidate persistence, and explicit offline short-circuiting.

# Acceptance criteria

- The reviewed matrix identifies supported endpoint behavior, stable fields, pagination, quotas/terms, lifecycle semantics, and credential handling; production work does not begin before the applicable gate passes.
- `acorn-lib::io::api::rrid` uses typed request/response DTOs checked against pinned fixtures, not portal HTML.
- Exact lookup accepts success only for the requested or contract-equivalent canonical RRID and preserves the local identifier on all failures.
- Authentication, absence, rate limiting, server errors, timeouts, malformed payloads, unknown values, and identifier mismatch produce fixture-backed typed outcomes.
- Existing `api::Secret`, endpoint override, redaction, offline, retry, response-bound, and lifecycle contracts are reused and remain green.
- Provider metadata stays in discovery provenance and cannot create project candidates or silently overwrite authored RAD relations.
- `RemoteProvider::Rrid` and `RemoteEntity::Resource` land only with stable semantics and focused compatibility tests.
- Search, if shipped, is bounded, preserves provider ranking/pagination, uses allowlisted typed queries, never exposes raw Elasticsearch DSL, and is disabled offline.
- CLI/TUI search is optional and cannot delay or alter portable RRID support.
- Default tests require no SciCrunch credential or network access.

# Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Public examples are mistaken for a supported stable contract. | High | Require the reviewed matrix and separate lookup/search gate decisions. |
| Query-string API keys leak through URLs, traces, or failures. | Critical | Prefer supported header authentication; otherwise prove full redaction or fail the gate. |
| Elasticsearch details escape into user-facing input. | Critical | Construct fixed typed queries over allowlisted fields and hard bounds. |
| Portal redirects or HTML are treated as record confirmation. | High | Require a typed response with canonical identifier agreement; never scrape HTML. |
| Retries or all-pages search become unbounded. | High | Reuse bounded transport policy and enforce page/request/result ceilings. |
| Resource results become project candidates or mutate RAD. | Critical | Add `Resource` semantics, retain provider data as provenance, and regression-test persistence boundaries. |
| Provider evolution breaks stored or displayed data. | High | Keep DTOs pinned, normalize only reviewed stable fields, preserve provenance, and handle unknown values explicitly. |

# Explicitly out of scope

- RRID parsing, local discovery, generic persistence, RAD validation, and DataCite/JATS mapping, which belong to the portable baseline.
- RRID minting, registration, manuscript rewriting, or publication-mention mining.
- Portal HTML scraping or browser automation as a provider API.
- Raw Elasticsearch DSL supplied by users.
- Unbounded search, background crawling, or replication of the SciCrunch index.
- Automatic mutation of canonical RAD or project-candidate creation from registry results.
- A new database schema solely for provider metadata.

# Related concepts

- [Portable RRID research-resource support](rrid-research-resource-support.md)
- [API roadmap](../refactors/api/roadmap.md)
- [API provider capability matrix](../refactors/api/provider-capability-matrix.md)
- [Analyzer composition and traversal](../refactors/analyzer/analyzer-composition.md)
- [Crate boundary architecture](../refactors/crate-boundary-architecture.md)
