---
type: Plan
title: "Add first-class RRID research-resource support"
description: "Support RRID parsing, discovery, resource modeling, standards interchange, registry resolution and search across ACORN's portable and host interfaces."
difficulty: high
impact: high
package: [acorn-schema, acorn-lib, acorn-cli, acorn-tui]
tags: [feature, pid, rrid, research-resource, scicrunch, discovery, resolution, datacite, jats, reproducibility]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-schema/src/schema/pid/mod.rs`, `crates/acorn-schema/src/util/constants/{mod,app}.rs`, `crates/acorn-schema/src/validation/mod.rs`, `crates/acorn-schema/src/schema/validate/`, `crates/acorn-schema/src/schema/research_activity/{mod,common}.rs`, `crates/acorn-schema/src/schema/standard/datacite.rs`, `crates/acorn-schema/src/schema/standard/crosswalk/`, adjacent schema tests; `crates/acorn-lib/src/io/api/mod.rs`, `crates/acorn-lib/src/analyzer/host/discovery/`, embedded endpoint configuration, database discovery tests; `crates/acorn-cli/src/{cli,commands/gather}/`, `crates/acorn-tui/src/{app,screens/gather}.rs`; `docs/src/concepts/persistent-identifiers.md`, `docs/src/cli-reference/commands/gather.md`, `docs/src/cli-reference/acceptance-test-plan.md`, and generated CLI help after CLI integration.
- **Create:** `crates/acorn-schema/src/schema/pid/rrid/{mod,tests}.rs`; a dedicated portable research-resource leaf such as `crates/acorn-schema/src/schema/research_activity/resource.rs`; `crates/acorn-lib/src/io/api/rrid.rs` and fixture-backed tests; `tests/fixtures/rrid/` containing provenance-recorded public syntax, resolver, search, DataCite, JATS, and failure fixtures.
- **Delete:** None. Do not remove or repurpose the existing hardware/compute `ResearchActivityMetadata.resources` field.

# Dependencies

- The portable PID slice can start independently in the PID lane, but it must serialize edits to `schema/pid/mod.rs`, shared regex constants, and validation registries with [SWHID persistent-identifier support](swhid-persistent-identifier.md) and PID proof/test work.
- The RAD resource model and DataCite crosswalk slice requires Slice 0 of [RAD metadata standards alignment](rad-metadata-standards-alignment.md), so omitted or approximated RRID relations produce structured conversion warnings rather than silent loss.
- Resolver reporting must consume the current typed discovery lifecycle and coordinate with [link resolution checks](../refactors/analyzer/link-resolution-checks.md). It must not introduce a new generic host trait ahead of a proven alternate consumer.
- The SciCrunch provider leaf must preserve the API contracts and extraction direction in the [API roadmap](../refactors/api/roadmap.md). Endpoint, credential, and DTO work must coordinate with the API shared-contract lane.

# Goal

Make RRIDs first-class evidence for research resources throughout ACORN: parse and normalize them locally, discover them in documents, represent the resources they identify without conflating them with projects or compute requirements, optionally confirm and enrich them from an authoritative service, exchange them through standards that support RRIDs, and expose predictable behavior in the CLI, TUI, database, generated schemas, and documentation.

“Top-tier support” means all of the following are true:

1. Local parsing is portable, deterministic, conservative, and independent of network availability.
2. Discovery recognizes real manuscript forms with low false-positive risk and retains the source location.
3. RRIDs remain resource evidence; they never create or merge research-activity candidates by themselves.
4. RAD has an explicit research-resource model distinct from hardware requirements and research outputs.
5. Resolution and registry search are opt-in, typed, fixture-tested, rate-limit aware, credential-safe, and disabled offline.
6. DataCite 4.7 and JATS RRID structures can be imported or exported without discarding the identifier or silently inventing a relation.
7. Every supported interface reports the same canonical identifier and lifecycle state.

# Current evidence

- `crates/acorn-schema/src/schema/pid/mod.rs` has typed parsing and discovery for ARK, arXiv, DOI, Handle, ISBN, ISNI, ORCID, patent, RAiD, ROR, and SWHID. `PIDINST` and `Unknown` are non-discoverable classifications. There is no `RRID` variant, typed parser, converter method, normalization branch, or identity policy.
- `Identifier::find_all` iterates `PID::is_discoverable`, normalizes results, and removes duplicates. Adding only a regex is insufficient: the PID enum, `From` conversions, `PersistentIdentifierConvert`, serialization, discovery, normalization, and tests must move together.
- `Record::resolve` in `crates/acorn-lib/src/analyzer/host/discovery/mod.rs` has provider-specific branches for arXiv, DOI, Handle, ORCID, RAiD, ROR, and SWHID. Unsupported types are retained with `ResolutionStatus::Unsupported`; this is the correct extension point for RRID resolution.
- Gather persistence stores the normalized identifier, lowercase PID wire name, optional metadata, resolution status, and source in the generic `discoveries` table. RRID discovery history therefore needs no database migration.
- Project-candidate persistence is deliberately limited by `PID::is_project_identifier`. RRIDs identify experimental inputs and related resources, so `PID::RRID` must remain false there.
- `ResearchActivityMetadata.resources` already means hardware/compute requirements, while `outputs` means products of the activity. Neither field accurately models antibodies, organisms, cell lines, plasmids, biosamples, facilities, instruments, or tools used in research.
- `TypedRelatedResource` can preserve a typed identifier and relation but cannot hold RRID resource metadata such as a display name, catalog number, proper citation, availability, or registry alerts. It is an interchange primitive, not the canonical resource model.
- DataCite support already includes `RelatedIdentifierType::Rrid`, but no current RAD mapping produces or consumes first-class RRID resources. DataCite defines RRID as an identifier for key experimental inputs and gives `RRID:SCR_014641` with relation `Cites` as its example ([DataCite 4.7 RRID definition](https://datacite-metadata-schema.readthedocs.io/en/4.7/appendices/appendix-1/relatedIdentifierType/)).
- NISO JATS provides `<resource-id resource-id-type="rrid">` inside a contributed-resource structure and warns consumers to compare the type value case-insensitively ([JATS `resource-id-type`](https://jats.nlm.nih.gov/publishing/tag-library/1.3d2/attribute/resource-id-type.html)). ACORN has no JATS semantic adapter today, so a full JATS document codec is not a prerequisite for portable RRID support.
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
| Identity | Use `rrid:RRID:<payload>` or another single documented namespaced key while preserving payload case until the registry contract proves case-insensitive identity. RRID must not be a project identity. |
| Network semantics | A redirect to a generic RRID portal is not proof that a record exists. Mark resolution successful only when a typed provider response identifies the requested or provider-canonical RRID. |

Do not copy one observed payload pattern into a universal grammar. RRIDs wrap identifiers from heterogeneous authorities, and the Initiative's own examples use underscores, embedded colons, hyphens, and payloads without underscores. Syntax validation proves that a value is a well-formed RRID reference; optional resolution establishes registry presence and current metadata.

# Canonical RAD model

Add `ResearchActivityMetadata.research_resources: Option<Vec<ResearchResource>>`, serialized as `researchResources`. Do not alias it to the existing `resources` key.

The initial `ResearchResource` contract should contain only fields supported by authored RAD or pinned provider/standards fixtures:

- required normalized `rrid`;
- optional human-readable `name` and provider-native `resource_type` string;
- optional `catalog_number`, `description`, and `proper_citation`;
- optional validated websites and alternate typed identifiers;
- optional availability/status plus structured alerts or issues that preserve “unavailable”, deprecated, or known-problem states without invalidating the RRID;
- optional relation information when imported from DataCite or another standard;
- provenance through ACORN's existing enrichment/provenance mechanisms rather than an untyped provider blob in canonical RAD.

Keep the high-level resource type open as a trimmed string in the first release. The portal covers an evolving set of resource classes, and a closed Rust enum would either reject new registry values or collapse them into an uninformative `Other`. A later controlled vocabulary may be added only after fixture evidence establishes stable provider categories and mappings.

Preserve the complete typed resolver response in discovery history/report metadata. Map only explicitly supported fields into `ResearchResource`; unknown response fields remain provenance and do not become speculative RAD fields.

# Implementation plan

Each slice must declare an exact write set at start. Shared `mod.rs`, CLI roots, generated help, endpoint configuration, manifests, and shared fixture indexes are integration locks rather than parallel leaf ownership.

## 0. Freeze the RRID syntax, resolver, and API contracts

**Write set:** this plan, a new `.wiki/assets/rrid-contract-matrix.md`, and new immutable fixtures under `tests/fixtures/rrid/contract/`. No Rust production files.

- Record the source URL, retrieval date, license/provenance, expected canonical identifier, and expected category for representative current RRIDs covering antibodies, organisms, cell lines, plasmids, software/databases, core facilities, instruments, and biosamples.
- Include valid manuscript forms, canonical forms, resolver URLs, mixed-case outer labels, whitespace after `RRID:`, adjacent punctuation, duplicate aliases, Unicode prose boundaries, and identifiers embedded in DataCite and JATS snippets.
- Include invalid or ambiguous cases: bare authority IDs, empty payloads, nested URL query values, control characters, line breaks, malformed percent encoding, doubled labels, unsupported schemes, and ordinary strings containing “rrid”.
- Capture resolver behavior for known, unavailable/deprecated, unknown, rate-limited, unauthorized, and malformed requests. Record redirect chains and content types without treating the web UI as a JSON API.
- Confirm a documented machine API and its authentication, pagination, quotas, terms, and stable response schema with SciCrunch. If no stable API contract is available, split delivery: ship local PID/discovery/standards support and keep metadata enrichment/search behind an experimental feature or deferred follow-on. Never scrape portal HTML.
- Decide whether the provider considers payload case identity-equivalent. Until confirmed, preserve case in normalization and exact identity.

**Deliverable:** a reviewed contract matrix from which every parser and provider test can be generated. No live-network test is required to validate portable parsing.

## 1. Add the portable `RRID` PID type

**Primary write set:** new `crates/acorn-schema/src/schema/pid/rrid/{mod,tests}.rs`; integration-owned changes to `schema/pid/mod.rs`, `util/constants/{mod,app}.rs`, `validation/mod.rs`, `schema/validate/mod.rs`, and their focused tests.

- Implement `RRID` as an immutable parsed value with `Display`, `FromStr`, `JsonSchema`, serde support, `PersistentIdentifier`, and `PersistentIdentifierParse` parity with other PID leaves.
- Keep parsing and canonicalization as pure functions. Store the opaque payload once; derive display and URL forms without mutable state.
- Add an owned parse error with actionable failure categories rather than returning an empty default from the strict `FromStr` path. Preserve the existing `from_string` compatibility behavior expected by the shared trait.
- Add `RE_RRID_TEXT`/`RE_RRID` only for candidate discovery. Perform exact validation and URL decoding in the parser so regex matching cannot define the complete validity contract.
- Add `PID::RRID`, wire name `rrid`, `is_rrid`, discovery, normalization, formatting, conversion, serialization, and `From<RRID> for Identifier` integration.
- Ensure `PID::RRID` is discoverable but not project-identifying. Add exact identity behavior that preserves the opaque payload unless Slice 0 proves a different registry rule.
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

## 3. Add first-class research resources and standards mappings

**Primary write set:** new `schema/research_activity/resource.rs`; integration changes to `schema/research_activity/mod.rs`, `schema/research_activity/common.rs`, DataCite crosswalk leaves, validation tests, and dedicated fixtures. This slice requires the RAD standards loss-reporting gate.

- Add the `ResearchResource` model and optional `researchResources` field defined above, with nested validation and generated JSON Schema coverage.
- Define deterministic merge semantics: normalized RRID is the key; authored non-empty scalar values win; resolver values fill missing fields; lists union deterministically; conflicting provider assertions remain provenance and emit a conflict rather than overwriting authored data.
- Extend typed-identifier validation so `scheme: RRID` validates the value and accepts the scheme name case-insensitively while serializing the canonical scheme consistently.
- Map DataCite 4.7 `RelatedIdentifier` and `RelatedItem` entries with identifier type `RRID` into/from research resources. Preserve `relationType`, `relationTypeInformation`, and `resourceTypeGeneral`; emit a structured warning whenever a richer resource record is simplified to a related identifier.
- Use `Cites` only when the source or explicit export profile asserts that relation. Do not infer a DataCite relation from mere co-occurrence. DataCite requires a relation for each related identifier and recognizes RRID in both RelatedIdentifier and RelatedItem ([DataCite RelatedIdentifier](https://datacite-metadata-schema.readthedocs.io/en/4.7/properties/relatedidentifier/), [DataCite RelatedItem](https://datacite-metadata-schema.readthedocs.io/en/4.7/properties/relateditem/)).
- Add a narrow JATS resource fragment codec or conversion helper only if ACORN already has an XML boundary that can own it by implementation time. Otherwise pin JATS fixtures and document the mapping contract for the future document adapter; do not add a general JATS parser to this feature.
- Update the standards field matrix so RRID/resource mappings are marked exact, transformed, lossy, local-only, or unsupported.

**Deliverable:** RAD can author and validate resources used by an activity, and DataCite RRID relations round-trip every field classified as exact while reporting every intentional loss.

## 4. Add authoritative RRID resolution and enrichment

**Primary write set:** new `crates/acorn-lib/src/io/api/rrid.rs` and its fixture-backed tests; integration-owned changes to `io/api/mod.rs`, embedded endpoint configuration, and the RRID branch in discovery resolution.

- Implement typed request/response DTOs only from the stable contract approved in Slice 0. Reject a successful HTTP response whose returned canonical RRID does not match the request under the approved identity rule.
- Normalize known provider response fields into a provider-neutral record: canonical RRID, name, resource type, proper citation, catalog number, canonical/alternate URLs, alternate identifiers, availability, and alerts. Retain the lossless raw DTO for provenance.
- Map resolution outcomes precisely: known record -> `Resolved`; syntactically valid but absent -> `Failed` with a not-found diagnostic; authenticated endpoint unavailable or contract unsupported -> `Unsupported`; 401/403, 404, 409, 429, 5xx, timeout, redirect loop, malformed body, and identifier mismatch -> typed failures.
- Respect `Retry-After` and bounded existing transport retry policy; never add unbounded retry or per-record busy loops. Do not follow redirects to unrelated hosts without the shared HTTP redirect policy.
- Use a dedicated secret environment variable such as `SCICRUNCH_API_KEY` only if the approved API requires it. Reuse `SecretString`, never serialize/log the credential, and keep public local parsing available without credentials.
- Add endpoint override support through the existing application endpoint model for local mock servers and deployments. Do not hard-code a test-only URL or make live SciCrunch traffic part of the default suite.
- In `--resolve`, retain the normalized RRID on every failure. When the input is a RAD document, map provider fields into the matching `researchResources` entry through the explicit merge rules; never create a project candidate from resolver metadata.
- Reject `--resolve` with `--offline` through the existing CLI guard and add tests proving no request is constructed or sent offline.

**Deliverable:** fixture-backed `--resolve` confirms and enriches known RRIDs with trustworthy lifecycle states while remaining deterministic and credential-safe.

## 5. Add registry search as a provider, conditional on a stable API

**Primary write set:** RRID provider leaf in `analyzer/host/discovery/`, RRID API search DTOs/tests, dedicated CLI gather option tests, and dedicated TUI mode tests. CLI/TUI roots and help are integration-owned.

- Add `RemoteProvider::Rrid` and `RemoteEntity::Resource` without forcing RRID results through project/person/organization semantics.
- Add a CLI mode such as `acorn gather --rrid "HeLa"` only after usability review against the existing `--osti` mode. Support upstream pagination and only those resource filters the API documents; do not expose raw Elasticsearch syntax.
- Normalize each result to `RemoteMatch` plus a resource-specific typed payload. A result may be reported and selected for RAD enrichment, but it must remain report-only for project candidate persistence.
- Add exact-RRID lookup and free-text search paths. Exact lookup should prefer the parser/resolver path; free text should preserve provider ranking and pagination rather than re-rank results locally.
- Add an RRID resource-search mode to the TUI with clear provider labeling, loading/error states, resource type, proper citation, availability/alerts, and canonical RRID. Offline mode must disable it before spawning work.
- Cover empty query, no results, pagination, all-pages bounds, duplicate canonical RRIDs, deprecated/unavailable resources, rate limiting, credentials, malformed records, and partial-page failures.
- If Slice 0 cannot establish a supported stable API, omit this slice from the first release and document portal search as the user workflow. Local PID support, RAD modeling, and standards interchange still ship without it.

**Deliverable:** users can search the authoritative registry from CLI/TUI without HTML scraping or accidentally persisting resources as projects.

## 6. Complete interoperability, documentation, and release hardening

**Primary write set:** PID lifecycle and gather documentation, acceptance-test plan, dedicated fixtures, generated JSON Schema snapshots, and CLI help generated once by the CLI lock owner.

- Document accepted forms, canonical form, local validation versus registry confirmation, supported resource categories, resolver/search credentials, offline behavior, unavailable records, raw/report output, and examples for text, JSON, DataCite, and JATS.
- Add end-to-end acceptance cases for plain text, Markdown, PDF, DOCX, authored RAD, DataCite import/export, local database history, resolved success/failure, offline rejection, and remote search when enabled.
- Add compatibility tests ensuring pre-RRID RAD documents still deserialize and existing `resources` retains its compute/hardware meaning.
- Add generated schema assertions for `PID::RRID`, `researchResources`, nested validation, and DataCite RRID types.
- Run dependency and portability gates. RRID parsing/modeling must remain in `acorn-schema`; network DTOs/protocol behavior remain host/API-owned.
- Record unsupported surfaces explicitly. A complete JATS document importer, automatic RRID registration, manuscript rewriting, and literature-mention mining are follow-on projects unless separately approved.

**Deliverable:** the supported surface is documented, regression-tested, portable, and ready for release without hidden live-service dependencies.

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
- Assert `Identifier::normalized` and `identity_key` are idempotent and that all accepted aliases converge only where the approved contract says they are identical.
- Assert every RRID appears once per normalized discovery/source pair, never as both RRID and generic URL, and never as a project candidate identity.
- Assert resolver fixtures cover 2xx, redirect, 400, 401, 403, 404, 409, 429 with `Retry-After`, 5xx, timeout, malformed payload, unknown enum values, missing required fields, and canonical-identifier mismatch.
- Assert `--offline` and local parsing require no API key and perform zero network calls.
- Golden-test RAD <-> DataCite transformations, including relation preservation and warnings for resource metadata DataCite cannot carry.
- Validate JATS fixture extraction case-insensitively for `resource-id-type` if the narrow adapter lands.

# Acceptance criteria

- `RRID:AB_2140114`, `RRID:IMSR_JAX:000664`, `RRID:CVCL_3588`, `RRID:Addgene_12260`, `RRID:SCR_003512`, and `RRID:SAMN33284296` parse and normalize according to the pinned contract without imposing one authority grammar on the others.
- Mixed-case outer labels, allowed resolver aliases, manuscript spacing, and surrounding punctuation normalize deterministically to `RRID:<payload>`; the opaque payload is not case-folded or rewritten without authoritative evidence.
- Bare source identifiers are not discovered as RRIDs, malformed RRIDs are rejected locally, and valid-but-unregistered or unavailable RRIDs remain distinguishable from syntax errors.
- `PID::RRID` serializes as `rrid`, is discoverable, and is never a project identifier.
- Gather reports and discovery history preserve normalized RRID, source, format, resolution status, and full provider metadata with no database schema migration.
- RAD exposes optional `researchResources` without changing the meaning or wire name of existing hardware `resources` or output `outputs`.
- Resource enrichment is keyed by normalized RRID, preserves authored values, fills only missing fields, unions lists deterministically, and records conflicts/provenance.
- DataCite 4.7 RRID RelatedIdentifier and RelatedItem entries import/export with identifier and relation intact; every simplification or omission emits a structured conversion warning.
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
| Research resources are confused with compute requirements or outputs. | High | Add the distinct `researchResources` field and prohibit aliasing or repurposing `resources`/`outputs`. |
| Provider categories evolve beyond a closed enum. | Medium | Preserve the provider-native trimmed type string initially and add a controlled vocabulary only from demonstrated stable mappings. |
| Unavailable, deprecated, or alerted resources are treated as invalid identifiers. | High | Separate syntax validity, registry presence, availability, and issue states in typed outcomes and RAD fields. |
| DataCite relation semantics are fabricated. | High | Preserve authored/imported relations; use an export default only under an explicit profile and emit an approximation warning. |
| Resolver credentials leak through configuration, debug output, or fixtures. | Critical | Reuse `SecretString`, redact headers/query parameters, use synthetic fixture tokens, and test serialized/debug output. |
| Shared PID, schema, API, CLI, or TUI roots collide with active plans. | Medium | Implement leaf-first, declare exact write sets, and serialize shared integration files through the locks in the parallel execution guide. |

# Explicitly out of scope

- Minting or registering new RRIDs on behalf of users.
- Automatically rewriting manuscripts or inserting citations into source documents.
- Treating a catalog number or native registry identifier as an RRID without the explicit RRID label or authoritative provider assertion.
- Mining publication mention graphs or reproducing SciCrunch curation/robot workflows.
- Building a general JATS document parser solely for RRID support.
- Making RRID validation depend on live network access.

# Related concepts

- [Persistent identifier lifecycle](../../docs/src/concepts/persistent-identifiers.md)
- [RAD metadata standards alignment](rad-metadata-standards-alignment.md)
- [SWHID persistent-identifier support](swhid-persistent-identifier.md)
- [Website and resolution checks](../refactors/analyzer/link-resolution-checks.md)
- [API provider capability matrix](../refactors/api/provider-capability-matrix.md)
- [Crate boundary architecture](../refactors/crate-boundary-architecture.md)
