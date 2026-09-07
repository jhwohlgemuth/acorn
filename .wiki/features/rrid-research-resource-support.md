---
type: Plan
title: "Add first-class RRID research-resource support"
description: "Support portable RRID parsing and discovery, standards-aligned RAD relations, and contract-gated SciCrunch resolution and search."
difficulty: high
impact: high
package: [acorn-schema, acorn-lib, acorn-cli, acorn-tui]
tags: [feature, pid, rrid, research-resource, scicrunch, discovery, resolution, datacite, jats, reproducibility]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-schema/src/pid/mod.rs`, `crates/acorn-schema/src/util/constants/{mod,app}.rs`, `crates/acorn-schema/src/validation/mod.rs`, `crates/acorn-schema/src/validate/mod.rs`, `crates/acorn-schema/src/research_activity/common.rs`, `crates/acorn-schema/src/standard/datacite.rs`, and adjacent tests; `crates/acorn-lib/src/io/api/mod.rs`, `crates/acorn-lib/src/analyzer/host/discovery/`, endpoint configuration, and database discovery tests; focused gather code/tests in `crates/acorn-cli` and `crates/acorn-tui`; PID, gather, and acceptance documentation; generated CLI help only if arguments change.
- **Create:** `crates/acorn-schema/src/pid/rrid/{mod,tests}.rs`; `.wiki/assets/rrid-contract-matrix.md`; immutable fixtures under `tests/fixtures/rrid/`; `crates/acorn-lib/src/io/api/rrid.rs` only after the provider contract gate passes.
- **Delete:** None. Do not remove or repurpose the existing hardware/compute `ResearchActivityMetadata.resources` field.

# Dependencies

- The portable PID slice can start independently in the PID lane, but it must serialize edits to `pid/mod.rs`, shared regex constants, and validation registries with [SWHID persistent-identifier hardening](swhid-persistent-identifier.md) and PID proof/test work.
- [RAD metadata standards alignment](rad-metadata-standards-alignment.md) Slice 0 and the reusable `TypedIdentifier`/`TypedRelatedResource` surface have landed. RRID can use that surface; adapter changes must still coordinate with the remaining alignment work and shared fixtures.
- Resolver reporting must consume the current typed discovery lifecycle and coordinate with [link resolution checks](../refactors/analyzer/analyzer-composition.md). It must not introduce a new generic host trait ahead of a proven alternate consumer.
- The SciCrunch provider leaf must preserve the API contracts and extraction direction in the [API roadmap](../refactors/api/roadmap.md). Endpoint, credential, and DTO work must coordinate with the API shared-contract lane.

# Goal

Make RRIDs first-class evidence for research resources throughout ACORN: parse and normalize them locally, discover them in documents, represent the resources they identify without conflating them with projects or compute requirements, optionally confirm and enrich them from an authoritative service, exchange them through standards that support RRIDs, and expose predictable behavior in the CLI, TUI, database, generated schemas, and documentation.

“Top-tier support” means all of the following are true:

1. Local parsing is portable, deterministic, conservative, and independent of network availability.
2. Discovery recognizes real manuscript forms with low false-positive risk and retains the source location.
3. RRIDs remain resource evidence; they never create or merge research-activity candidates by themselves.
4. RAD stores RRIDs through the existing typed related-resource contract, with an explicit relation and no second resource collection.
5. Resolution and registry search are opt-in, typed, fixture-tested, rate-limit aware, credential-safe, and disabled offline.
6. DataCite 4.7 and JATS 1.4 mappings preserve the identifier and report every unsupported semantic without inventing a relation.
7. Every supported interface reports the same canonical identifier and lifecycle state.

# Refresh decision (2026-09-06)

- No RRID parser, `PID::RRID` variant, discovery branch, resolver, or registry provider has landed.
- Do not add a new `researchResources` field in this plan. The now-landed `ResearchActivityMetadata.related_resources: Option<Vec<TypedRelatedResource>>` already carries a typed identifier, relation, relation detail, general resource type, and category.
- Keep proper citation, vendor, catalog number, availability, issues, raw categories, and other provider-rich values in discovery resolution metadata and provenance. A later plan may add authored fields if concrete round-trip requirements justify them.
- The RRID site documents portal search and points API users to SciCrunch developer tools. The SciCrunch API Handbook documents an API-key-authenticated Elasticsearch pass-through, production aliases, `size`/`from` pagination, exact `rrid.curie` lookup, and free-text queries. It references Elasticsearch 6.8 and places the key in a query parameter. This supports a contract probe, not an assumption of stable fields, quotas, terms, or credential handling ([RRID system](https://www.rrids.org/rrid-system), [RIN API overview](https://github.com/SciCrunch/API-Handbook/blob/master/resources-information-network/overview.md), [RIN search examples](https://github.com/SciCrunch/API-Handbook/blob/master/resources-information-network/basic-rin-search-examples.md)).
- JATS 1.4, approved on 2024-10-31, is the current NISO standard. RRID fixtures and claims must target it ([JATS 1.4](https://jats.nlm.nih.gov/publishing/1.4/)).

# Recommendation

Ship RRID in two tiers. The baseline tier adds the portable PID leaf, local discovery and persistence, `TypedRelatedResource` validation, and DataCite/JATS mapping fixtures. It has no credential or live-service dependency. The enhanced tier adds SciCrunch lookup and search only after the contract matrix confirms a supported endpoint, stable response fields, bounded pagination, and credential handling that passes ACORN's redaction requirements.

Keep the schema change small. Reuse `relatedResources`, preserve explicit relation semantics, and keep registry-only metadata in discovery provenance. This avoids an overlapping resource collection while leaving a clear follow-on path if authored rich resource metadata becomes a demonstrated requirement.

# Current evidence

- `crates/acorn-schema/src/pid/mod.rs` has typed parsing and discovery for ARK, arXiv, DOI, Handle, ISBN, ISNI, ORCID, patent, RAiD, ROR, and SWHID. `PIDINST` and `Unknown` are non-discoverable classifications. There is no `RRID` variant, typed parser, converter method, normalization branch, or identity policy.
- `Identifier::find_all` iterates `PID::is_discoverable`, normalizes results, and removes duplicates. Adding only a regex is insufficient: the PID enum, `From` conversions, `PersistentIdentifierConvert`, serialization, discovery, normalization, and tests must move together.
- `Record::resolve` in `crates/acorn-lib/src/analyzer/host/discovery/mod.rs` has provider-specific branches for arXiv, DOI, Handle, ORCID, RAiD, ROR, and SWHID. Unsupported types are retained with `ResolutionStatus::Unsupported`; this is the correct extension point for RRID resolution.
- Gather persistence stores the normalized identifier, lowercase PID wire name, optional metadata, resolution status, and source in the generic `discoveries` table. RRID discovery history therefore needs no database migration.
- Project-candidate persistence is deliberately limited by `PID::is_project_identifier`. RRIDs identify experimental inputs and related resources, so `PID::RRID` must remain false there.
- `ResearchActivityMetadata.resources` means hardware/compute requirements. The landed `related_resources` field is the correct canonical location for RRID relations.
- `TypedRelatedResource` stores a `TypedIdentifier`, relation, relation detail, general resource type, and category. Its scheme-aware validation does not yet recognize RRID. Rich registry metadata remains discovery provenance in this plan.
- DataCite support already includes `RelatedIdentifierType::Rrid`, but no current RAD mapping produces or consumes first-class RRID resources. DataCite defines RRID as an identifier for key experimental inputs and gives `RRID:SCR_014641` with relation `Cites` as its example ([DataCite 4.7 RRID definition](https://datacite-metadata-schema.readthedocs.io/en/4.7/appendices/appendix-1/relatedIdentifierType/)).
- NISO JATS 1.4 provides `<resource-id resource-id-type="rrid">` inside contributed-resource markup and warns consumers to compare the type value case-insensitively ([JATS 1.4 `resource-id-type`](https://jats.nlm.nih.gov/publishing/tag-library/1.4/attribute/resource-id-type.html)). ACORN has no JATS semantic adapter today, so a full JATS document codec is not a prerequisite.
- The RRID Initiative describes RRIDs as identifiers for resources including facilities, instruments, software, antibodies, organisms, and cell lines, and directs users to the RRID portal for citation text ([RRID home](https://www.rrids.org/)). Its author guidance shows multiple payload families, including `AB_2140114`, `IMSR_JAX:000664`, `CVCL_3588`, `Addgene_12260`, `SCR_003512`, and `SAMN33284296` ([RRID author guidance](https://www.rrids.org/draft-texts)).
- The RRID index aggregates authoritative source registries, keeps unavailable records rather than deleting them, and exposes API access through SciCrunch developer tools ([RRID system](https://www.rrids.org/rrid-system)). Availability warnings and withdrawn records must therefore remain valid RRIDs rather than being treated as parse failures.

# Identifier contract

The portable contract must be agreed and fixture-tested before host integration.

| Concern | Required behavior |
|---|---|
| Canonical display | `RRID:<payload>` with an uppercase `RRID` label and no whitespace after the delimiter. |
| Accepted labels | Match the outer `RRID:` label case-insensitively and tolerate horizontal whitespace used in prose, then canonicalize only the outer label and spacing. |
| Payload handling | Treat the registry-assigned payload as opaque and preserve its case and punctuation. Do not lowercase, split, or rewrite it without an authoritative returned canonical identifier. |
| Local validity | Require an explicit RRID label, a non-empty bounded payload, and visible non-whitespace characters; reject controls, line breaks, surrounding URL query/fragment text, malformed percent escapes, and trailing prose punctuation. Do not invent a check digit or require one payload family. |
| Authority hints | Optional accessors may expose a conservative leading authority hint, but validity must not require an underscore: current official examples include both delimiter-bearing and delimiter-free payloads. |
| Discovery | Discover explicit `RRID:` citations and allowlisted RRID resolver URLs only. Never discover bare values such as `SCR_003512` or `CVCL_0030` in arbitrary prose. |
| Resolver URL | Emit one documented persistent HTTPS form, initially `https://n2t.net/RRID:<payload>` if the contract fixtures confirm the redirect behavior. Accept current SciCrunch/RRID portal resolver URLs as input aliases without retaining tracking query parameters. |
| Deduplication | Use the normalized RRID within the existing identifier/source policy. Preserve payload case until the registry contract proves another identity rule. RRID must not be a project identity. |
| Network semantics | A redirect to a generic RRID portal is not proof that a record exists. Mark resolution successful only when a typed provider response identifies the requested or provider-canonical RRID. |

Do not copy one observed payload pattern into a universal grammar. RRIDs wrap identifiers from heterogeneous authorities, and the Initiative's own examples use underscores, embedded colons, hyphens, and payloads without underscores. Syntax validation proves that a value is a well-formed RRID reference; optional resolution establishes registry presence and current metadata.

# RAD and standards contract

Represent an authored RRID through the existing `TypedRelatedResource`:

```json
{
  "identifier": {
    "value": "RRID:SCR_003512",
    "scheme": "RRID"
  },
  "relationType": "Cites",
  "resourceTypeGeneral": "Software"
}
```

The current RAD type and DataCite both require a relation. Preserve authored and imported relations. Plain-text discovery must not infer `Cites`, a resource type, or a scheme URI. A conversion profile may supply a default only when it reports an approximation.

DataCite import/export maps RRID `RelatedIdentifier` values to this structure. Map `RelatedItem` only where the existing adapter supports its additional structure without loss, and report simplification through the crosswalk warning contract. JATS fixtures map `resource-id-type="rrid"` case-insensitively to the same identifier. Resource name and type remain adapter work unless their destination is approved in the standards field matrix.

Preserve typed provider responses in discovery history/report metadata. Do not add provider fields to canonical RAD until authored and interchange requirements establish a stable model.

# Implementation plan

Each slice must declare an exact write set at start. Shared `mod.rs`, CLI roots, generated help, endpoint configuration, manifests, and shared fixture indexes are integration locks rather than parallel leaf ownership.

## 0. Freeze the RRID syntax, resolver, and API contracts

**Write set:** this plan, a new `.wiki/assets/rrid-contract-matrix.md`, and new immutable fixtures under `tests/fixtures/rrid/contract/`. No Rust production files.

- Record the source URL, retrieval date, license/provenance, expected canonical identifier, and expected category for representative current RRIDs covering antibodies, organisms, cell lines, plasmids, software/databases, core facilities, instruments, and biosamples.
- Include valid manuscript forms, canonical forms, resolver URLs, mixed-case outer labels, whitespace after `RRID:`, adjacent punctuation, duplicate aliases, Unicode prose boundaries, and identifiers embedded in DataCite and JATS snippets.
- Include invalid or ambiguous cases: bare authority IDs, empty payloads, nested URL query values, control characters, line breaks, malformed percent encoding, doubled labels, unsupported schemes, and ordinary strings containing “rrid”.
- Capture resolver behavior for known, unavailable/deprecated, unknown, rate-limited, unauthorized, and malformed requests. Record redirect chains and content types without treating the web UI as a JSON API.
- Confirm a documented machine API and its authentication, pagination, quotas, terms, and stable response schema with SciCrunch. If no stable API contract is available, split delivery: ship local PID/discovery/standards support and keep metadata enrichment/search behind an experimental feature or deferred follow-on. Never scrape portal HTML.
- Pin the SciCrunch API Handbook revision, supported production aliases, allowed fields, Elasticsearch compatibility, and response examples. Review whether its documented query-parameter key can satisfy ACORN's URL/error/trace redaction guarantees; prefer a supported header form and defer provider work if credentials cannot be protected.
- Decide whether the provider considers payload case identity-equivalent. Until confirmed, preserve case in normalization and exact identity.

**Deliverable:** a reviewed contract matrix from which every parser and provider test can be generated. No live-network test is required to validate portable parsing.

## 1. Add the portable `RRID` PID type

**Primary write set:** new `crates/acorn-schema/src/pid/rrid/{mod,tests}.rs`; integration-owned changes to `pid/mod.rs`, `util/constants/{mod,app}.rs`, `validation/mod.rs`, `validate/mod.rs`, and their focused tests.

- Implement `RRID` as an immutable parsed value with `Display`, `FromStr`, `JsonSchema`, serde support, `PersistentIdentifier`, and `PersistentIdentifierParse` parity with other PID leaves.
- Keep parsing and canonicalization as pure functions. Store the opaque payload once; derive display and URL forms without mutable state.
- Add an owned parse error with actionable failure categories rather than returning an empty default from the strict `FromStr` path. Preserve the existing `from_string` compatibility behavior expected by the shared trait.
- Add `RE_RRID_TEXT`/`RE_RRID` only for candidate discovery. Perform exact validation and URL decoding in the parser so regex matching cannot define the complete validity contract.
- Add `PID::RRID`, wire name `rrid`, `is_rrid`, discovery, normalization, formatting, conversion, serialization, and `From<RRID> for Identifier` integration.
- Ensure `PID::RRID` is discoverable but not project-identifying. Reuse generic normalized discovery storage and preserve the opaque payload unless Slice 0 proves a different registry rule.
- Add `rules::rrid` and schema-level `is_rrid` validation for authored fields and typed identifiers whose declared scheme is `RRID`.
- Cover every contract fixture plus fuzz/property invariants: parsing never panics, `parse(display(x)) == x`, normalization is idempotent, resolver alias normalization is deterministic, surrounding prose is not retained, and invalid input never becomes an arbitrary URL or Handle.

**Deliverable:** `acorn-schema` users can parse, validate, normalize, discover, serialize, and render RRIDs with no `std`, filesystem, network, or provider dependency.

## 2. Integrate RRIDs with discovery, reports, and persistence

**Primary write set:** `crates/acorn-lib/src/analyzer/host/discovery/{mod.rs,tests/mod.rs}`, `crates/acorn-cli/src/commands/gather/tests/`, `crates/acorn-lib/src/io/database/tests/discovery.rs`, and dedicated gather fixtures. Shared roots are integration-owned.

- Add RRID to the all-PID discovery matrix and PDF/DOCX/plain-text fixture coverage.
- Verify alias deduplication within a source, preservation across distinct sources, stable report ordering, lowercase `identifier_type: "rrid"`, and generic database round trips.
- Preserve `ResolutionStatus::NotRequested` without network access by default and `Unsupported` until the provider slice lands.
- Add regression tests proving an RRID never enters `Records::candidates`, never creates a research-activity database row, and does not merge unrelated candidates merely because it appears beside a DOI.
- Add collision tests against URL, Handle, RAiD, DOI, and ordinary colon-delimited text. Explicit RRID matches must suppress the resolver URL from being reported again as a generic URL.
- Make raw output the canonical `RRID:<payload>` before and after resolution unless a resolved proper citation is explicitly requested by a future dedicated output option; do not silently replace machine-readable RRID lines with prose.

**Deliverable:** `acorn gather --text "... RRID:CVCL_0030 ..."` reports and persists one normalized RRID locally, with no candidate side effects.

## 3. Complete RAD and standards interchange

**Primary write set:** focused typed-identifier validation, DataCite adapter tests, dedicated fixtures, and `.wiki/assets/standards-field-matrix.md`. Coordinate shared adapter edits with RAD alignment.

- Extend typed-identifier validation so `scheme: RRID` validates with the portable parser and accepts the scheme name case-insensitively. Canonicalize the scheme name only if that matches the shared typed-identifier policy.
- Map DataCite 4.7 RRID `RelatedIdentifier` values through `TypedRelatedResource`, preserving `relationType`, `relationTypeInformation`, and `resourceTypeGeneral`.
- Use `Cites` only when the source or an explicit conversion profile asserts it. Missing target-required relations fail readiness or use the existing structured conversion outcome. Co-occurrence alone never creates a relation ([DataCite RelatedIdentifier](https://datacite-metadata-schema.readthedocs.io/en/4.7/properties/relatedidentifier/)).
- Classify `RelatedItem` and JATS 1.4 fields as exact, transformed, simplified, local-only, or unsupported in the standards field matrix. Add a narrow JATS fragment adapter only if an existing XML boundary can own it; otherwise retain mapping fixtures for a future document adapter.
- Add compatibility tests proving existing RAD documents still deserialize and `resources` retains its hardware/compute meaning.

**Deliverable:** RAD can author and validate an RRID relation, and each claimed DataCite round trip is fixture-tested with explicit loss reporting.

## 4. Add authoritative RRID resolution and enrichment

**Gate:** Slice 0 confirms a supported API contract and credential policy.

**Primary write set:** new `crates/acorn-lib/src/io/api/rrid.rs` and its fixture-backed tests; integration-owned changes to `io/api/mod.rs`, embedded endpoint configuration, and the RRID branch in discovery resolution.

- Implement typed request/response DTOs only from the contract approved in Slice 0. Exact lookup must query the documented `rrid.curie` field and reject a response whose canonical RRID does not match the request.
- Normalize known provider response fields into a provider-neutral record: canonical RRID, name, resource type, proper citation, catalog number, canonical/alternate URLs, alternate identifiers, availability, and alerts. Retain the lossless raw DTO for provenance.
- Map resolution outcomes precisely: known record -> `Resolved`; syntactically valid but absent -> `Failed` with a not-found diagnostic; authenticated endpoint unavailable or contract unsupported -> `Unsupported`; 401/403, 404, 409, 429, 5xx, timeout, redirect loop, malformed body, and identifier mismatch -> typed failures.
- Respect `Retry-After` and bounded existing transport retry policy; never add unbounded retry or per-record busy loops. Do not follow redirects to unrelated hosts without the shared HTTP redirect policy.
- Use the environment variable approved by the contract matrix. Reuse `SecretString`; redact the key from URLs, errors, traces, database metadata, and debug output. Prefer a supported header form because the handbook's query-parameter example creates a leakage risk.
- Add endpoint override support through the existing application endpoint model for local mock servers and deployments. Do not hard-code a test-only URL or make live SciCrunch traffic part of the default suite.
- In `--resolve`, retain the normalized RRID on every failure. Keep provider enrichment in typed resolution metadata/provenance; never create a project candidate or mutate canonical RAD from resolver metadata in this plan.
- Reject `--resolve` with `--offline` through the existing CLI guard and add tests proving no request is constructed or sent offline.

**Deliverable:** fixture-backed `--resolve` confirms and enriches known RRIDs with trustworthy lifecycle states while remaining deterministic and credential-safe.

## 5. Add registry search as a provider, conditional on a stable API

**Gate:** Slice 4 is green and Slice 0 approves pagination, ranking, index aliases, and the allowed query surface.

**Primary write set:** RRID provider leaf in `analyzer/host/discovery/`, RRID API search DTOs/tests, dedicated CLI gather option tests, and dedicated TUI mode tests. CLI/TUI roots and help are integration-owned.

- Add `RemoteProvider::Rrid` and `RemoteEntity::Resource` without forcing RRID results through project/person/organization semantics.
- Add a CLI mode such as `acorn gather --rrid "HeLa"` only after usability review against the existing `--osti` mode. Build fixed typed queries over allowlisted fields, bound `size`/`from` pagination, and never expose raw Elasticsearch syntax.
- Normalize each result to `RemoteMatch` plus a resource-specific typed payload. A result may be reported and selected for RAD enrichment, but it must remain report-only for project candidate persistence.
- Add exact-RRID lookup and free-text search paths. Exact lookup should prefer the parser/resolver path; free text should preserve provider ranking and pagination rather than re-rank results locally.
- Add an RRID resource-search mode to the TUI with clear provider labeling, loading/error states, resource type, proper citation, availability/alerts, and canonical RRID. Offline mode must disable it before spawning work.
- Cover empty query, no results, pagination, all-pages bounds, duplicate canonical RRIDs, deprecated/unavailable resources, rate limiting, credentials, malformed records, and partial-page failures.
- If Slice 0 cannot establish a supported stable API, omit this slice from the first release and document portal search as the user workflow. Local PID support, RAD modeling, and standards interchange still ship without it.

**Deliverable:** users can search the authoritative registry from CLI/TUI without HTML scraping or accidentally persisting resources as projects.

## 6. Complete interoperability, documentation, and release hardening

**Primary write set:** PID lifecycle and gather documentation, acceptance-test plan, dedicated fixtures, generated JSON Schema snapshots, and CLI help generated once by the CLI lock owner.

- Document accepted forms, canonical form, local validation versus registry confirmation, supported resource categories, resolver/search credentials, offline behavior, unavailable records, raw/report output, and examples for text, JSON, DataCite, and JATS 1.4.
- Add end-to-end acceptance cases for plain text, Markdown, PDF, DOCX, authored RAD, DataCite import/export, local database history, resolved success/failure, offline rejection, and remote search when enabled.
- Add compatibility tests ensuring pre-RRID RAD documents still deserialize and existing `resources` retains its compute/hardware meaning.
- Add generated schema assertions for `PID::RRID`, RRID-aware typed identifiers, and DataCite RRID types.
- Run dependency and portability gates. RRID parsing/modeling must remain in `acorn-schema`; network DTOs/protocol behavior remain host/API-owned.
- Record two delivery tiers: portable/local/standards support, and optional provider-backed support. A complete JATS document importer, automatic RRID registration, manuscript rewriting, rich authored registry metadata, and literature-mention mining are follow-on projects unless separately approved.

**Deliverable:** the supported surface is documented, regression-tested, portable, and ready for release without hidden live-service dependencies.

# Delivery order

1. Freeze the contract matrix.
2. Land the portable PID leaf.
3. Integrate local discovery and persistence.
4. Add RAD/DataCite interchange.
5. Complete baseline documentation and repository gates.
6. Add provider lookup, then search, only if their gates pass.

Slices 1 through 3 plus the applicable part of Slice 6 define the baseline release. Provider uncertainty must not delay it.

# Validation

Focused gates by slice:

```sh
cargo nextest run -p acorn-schema --lib pid::rrid
cargo nextest run -p acorn-schema --lib schema::pid
cargo nextest run -p acorn-schema --lib research_activity
cargo nextest run -p acorn-schema --lib standard::datacite
cargo nextest run -p acorn-schema --lib standard::crosswalk
cargo nextest run -p acorn-lib --lib analyzer::host::discovery
cargo nextest run -p acorn-lib --lib io::api
cargo nextest run -p acorn-cli --lib commands::gather
cargo nextest run -p acorn-tui --lib screens::gather
```

Repository gates after each integrated slice:

```sh
cargo fmt --all
cargo check -p acorn-schema --no-default-features
cargo check -p acorn-schema --no-default-features --features alloc
make check-wasm
make lint
make test
make help # only after CLI help changes
make audit # only if dependencies change
```

Additional verification:

- Run contract fixtures entirely offline; no test contacts N2T, SciCrunch, rrid.site, or a source registry by default.
- Run property/fuzz cases over arbitrary UTF-8 and percent-encoded input; parsing must not panic or consume adjacent prose.
- Assert normalization is idempotent and accepted aliases converge only where the approved contract says they are identical.
- Assert every RRID appears once per normalized discovery/source pair, never as both RRID and generic URL, and never as a project candidate identity.
- Assert resolver fixtures cover 2xx, redirect, 400, 401, 403, 404, 409, 429 with `Retry-After`, 5xx, timeout, malformed payload, unknown enum values, missing required fields, and canonical-identifier mismatch.
- Assert `--offline` and local parsing require no API key and perform zero network calls.
- Golden-test RAD <-> DataCite transformations, including relation preservation and structured warnings for every simplification or omission.
- Validate JATS fixture extraction case-insensitively for `resource-id-type` if the narrow adapter lands.

# Acceptance criteria

- `RRID:AB_2140114`, `RRID:IMSR_JAX:000664`, `RRID:CVCL_3588`, `RRID:Addgene_12260`, `RRID:SCR_003512`, and `RRID:SAMN33284296` parse and normalize according to the pinned contract without imposing one authority grammar on the others.
- Mixed-case outer labels, allowed resolver aliases, manuscript spacing, and surrounding punctuation normalize deterministically to `RRID:<payload>`; the opaque payload is not case-folded or rewritten without authoritative evidence.
- Bare source identifiers are not discovered as RRIDs, malformed RRIDs are rejected locally, and valid-but-unregistered or unavailable RRIDs remain distinguishable from syntax errors.
- `PID::RRID` serializes as `rrid`, is discoverable, and is never a project identifier.
- Gather reports and discovery history preserve normalized RRID, source, format, resolution status, and full provider metadata with no database schema migration.
- RAD represents RRIDs through existing `relatedResources` without changing the meaning or wire name of hardware `resources` or output `outputs`.
- Provider enrichment remains typed discovery metadata/provenance and does not silently overwrite authored RAD.
- DataCite 4.7 RRID `RelatedIdentifier` values import/export with identifier and relation intact; supported `RelatedItem` mappings and every simplification or omission are explicit in the standards field matrix.
- JATS claims and fixtures target the current 1.4 standard without implying a general JATS parser.
- Resolution is opt-in, status-specific, fixture-backed, bounded under rate limiting, disabled offline, and never treats a generic redirect as proof of record existence.
- Search, if shipped, uses a documented supported machine API, never scrapes HTML, preserves provider pagination/ranking, and cannot create research-activity candidates.
- CLI, TUI, JSON/YAML/Markdown/raw reports, generated schemas, persistent-identifier documentation, and acceptance tests agree on the canonical identifier and lifecycle semantics.
- Existing RAD fixtures and PID behavior remain compatible; default, minimal, alloc, and WASM builds stay green.

# Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| RRID payload families are mistaken for one uniform grammar. | High | Keep payloads opaque, require the explicit outer label for discovery, and derive the acceptance matrix from multiple current authority families. |
| A portal redirect is mistaken for registry confirmation. | High | Require a typed record whose canonical identifier matches the request; model redirect-only or UI-only behavior separately. |
| SciCrunch API details, quotas, or access terms are unstable or undocumented. | High | Gate provider work on a pinned supported contract; ship portable/local/standards support independently; never scrape HTML. |
| RRIDs are persisted as project identities and merge unrelated activities. | Critical | Keep `is_project_identifier` false and add candidate/database regression tests before provider integration. |
| RAD gains an overlapping speculative resource model. | High | Reuse `TypedRelatedResource`; plan richer authored fields only from demonstrated round-trip needs. |
| Provider categories and response fields evolve. | Medium | Keep a small provider-neutral report view and preserve the pinned typed response as provenance. |
| Unavailable, deprecated, or alerted resources are treated as invalid identifiers. | High | Separate syntax validity, registry presence, availability, and issue states in typed outcomes and RAD fields. |
| DataCite relation semantics are fabricated. | High | Preserve authored/imported relations; use an export default only under an explicit profile and emit an approximation warning. |
| Resolver credentials leak through configuration, debug output, or fixtures. | Critical | Reuse `SecretString`, redact headers/query parameters, use synthetic fixture tokens, and test serialized/debug output. |
| User input reaches Elasticsearch as raw DSL. | Critical | Build fixed typed queries over allowlisted fields and bound pagination. |
| Shared PID, schema, API, CLI, or TUI roots collide with active plans. | Medium | Implement leaf-first, declare exact write sets, and serialize shared integration files through the locks in the parallel execution guide. |

# Explicitly out of scope

- Minting or registering new RRIDs on behalf of users.
- Automatically rewriting manuscripts or inserting citations into source documents.
- Treating a catalog number or native registry identifier as an RRID without the explicit RRID label or authoritative provider assertion.
- Mining publication mention graphs or reproducing SciCrunch curation/robot workflows.
- Building a general JATS document parser solely for RRID support.
- Adding a second canonical RAD resource collection or rich authored registry model in this plan.
- Making RRID validation depend on live network access.

# Related concepts

- [Persistent identifier lifecycle](../../docs/src/concepts/persistent-identifiers.md)
- [RAD metadata standards alignment](rad-metadata-standards-alignment.md)
- [SWHID persistent-identifier hardening](swhid-persistent-identifier.md)
- [Analyzer composition and traversal](../refactors/analyzer/analyzer-composition.md)
- [API provider capability matrix](../refactors/api/provider-capability-matrix.md)
- [Crate boundary architecture](../refactors/crate-boundary-architecture.md)
