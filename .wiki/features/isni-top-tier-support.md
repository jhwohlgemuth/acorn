---
type: Plan
title: "Promote ISNI to top-tier persistent-identifier support"
description: "Complete ACORN's existing ISNI support with unambiguous discovery, strict agent-identifier validation, public registry resolution, typed person/organization metadata, provenance, and end-to-end gather coverage."
difficulty: medium
impact: high
package: [acorn-schema, acorn-lib]
tags: [feature, pid, isni, identity, metadata, discovery, resolver, sru, xml]
status: in-progress
---

# Project code boundaries

- **Update:** `crates/acorn-schema/src/pid/isni/{mod.rs,tests.rs}`, `crates/acorn-schema/src/util/constants/mod.rs`, `crates/acorn-schema/src/research_activity/common.rs`, `crates/acorn-schema/src/validate/{mod.rs,tests.rs}`, `crates/acorn-lib/src/io/api/{mod.rs,tests/mod.rs}`, `crates/acorn-lib/assets/constants/application.json`, `crates/acorn-lib/src/analyzer/host/discovery/{mod.rs,tests/mod.rs}`, `docs/src/concepts/persistent-identifiers.md`, and `docs/src/cli-reference/commands/gather.md`.
- **Create:** `crates/acorn-lib/src/io/api/isni.rs`, `crates/acorn-lib/src/io/api/tests/isni.rs`, and dedicated response fixtures under `tests/fixtures/isni/`.
- **Delete:** None.

# Dependencies

The portable parser slice uses the existing `PersistentIdentifier`, `PersistentIdentifierParse`, `Identifier`, and validation contracts in `acorn-schema`. Its `AgentIdentifier` validation change must coordinate with [RAD metadata standards alignment](rad-metadata-standards-alignment.md), which owns the broader contributor and organization model.

The resolver slice uses the existing bounded HTTP transport, endpoint configuration, XML parsing, `ResolutionOutcome`, and gather provenance contracts. It must coordinate `io/api/mod.rs` and `application.json` with the API shared-contract lane, and `analyzer/host/discovery/mod.rs` with [analyzer composition and traversal](../refactors/analyzer/analyzer-composition.md). It does not depend on a new CLI argument, database migration, authentication mechanism, or crate dependency.

# Goal

Promote ISNI from strong local parsing plus standards-field validation to top-tier PID support across ACORN's complete applicable lifecycle:

1. recognize an explicitly identified ISNI without confusing it with the syntactically compatible ORCID number space;
2. parse, validate, normalize, and expose its canonical resolver URL locally and portably;
3. validate ISNI values wherever RAD and supported standards declare an ISNI scheme;
4. discover and retain ISNI person or organization evidence without treating it as research-activity identity;
5. optionally resolve a valid ISNI through the public registry during `acorn gather --resolve`;
6. preserve registry metadata and provenance while rendering a useful public name in raw output; and
7. fail closed, quickly, and observably when the registry is unavailable, malformed, ambiguous, or returns no assigned record.

“Top tier” is capability-based, not a claim that every PID has the same semantics. ISNI identifies a public identity, so it must not create or merge a project candidate merely because it appears in input.

# Current evidence

- `crates/acorn-schema/src/pid/isni/mod.rs` already implements typed parsing, ISO 7064 MOD 11-2 validation, canonical `https://isni.org/isni/...` display, and ORCID-aware extraction. `PID::ISNI`, `Identifier::normalized`, conversion traits, generic discovery, and validator dispatch are already wired.
- `crates/acorn-schema/src/pid/isni/tests.rs` covers valid and invalid checksums, Metadata Tools/idutils vectors, spaced and hyphenated inputs, canonical URLs, generic PID conversion, and ISNI URL precedence over ORCID.
- The current local contract is incomplete: `ISNI::url()` inherits the trait's empty default even though `Display` emits a resolver URL. Explicit text such as `ISNI 1422 4586 3573 0476` is not an anchored parse form, and bare 16-character values are intrinsically ambiguous with ORCID because both use the same format and checksum algorithm.
- `crates/acorn-schema/src/validate/mod.rs` strictly validates ISNI in DataCite and InvenioRDM typed fields, and RAiD contributor validation accepts ISNI. `research_activity/common.rs` documents `AgentIdentifier` as supporting ISNI but currently validates only non-emptiness and URI syntax, not agreement between `scheme`, `scheme_uri`, and `value`.
- `crates/acorn-lib/src/analyzer/host/discovery/mod.rs` discovers and retains ISNI, but its resolver dispatch reports ISNI as unsupported. The lifecycle documentation therefore describes ISNI as local-only.
- ISO 27729:2024 defines ISNI for public identities of persons and organizations involved across content-industry creation, production, management, and distribution, and explicitly positions it as a disambiguation and linking identifier rather than a comprehensive identity profile ([ISO 27729:2024](https://www.iso.org/standard/87177.html)).
- The ISNI International Agency's current technical-documentation page publishes SRU search guidelines and an enquiry-response XSD, and states that the database supports public search/API interaction ([technical documentation](https://isni.bowker.com/technical-documentation), [data inputs and outputs](https://isni.bowker.com/data-inputs-and-outputs)).
- A live anonymous probe on 2026-09-08 confirmed that `https://isni.oclc.org/sru/` accepts an exact `pica.isn` query over HTTPS and returns one `isni-b` namespaced XML record for `0000000492299539`, including its canonical URI, confidence, organization name, type, sources, and location. A second probe returned person-name variants and merged identifiers for `0000000121491740`. The nominal `https://isni.org/sru/` path returned HTTP 403, so the implementation must use the documented OCLC-hosted service rather than derive an API endpoint from the public resolver URI.

# Support contract

| Lifecycle capability | Required ISNI behavior |
|---|---|
| Find | Discover resolver URLs and explicitly labeled `ISNI ...` forms; do not guess whether an unlabeled checksum-compatible number is ISNI or ORCID. |
| Parse | Accept canonical URL, HTTP legacy URL, explicit `ISNI` label, compact value, spaces, and hyphens when the caller explicitly requests ISNI. |
| Validate | Require exactly 15 decimal digits plus a decimal or `X` MOD 11-2 check character; require the entire explicit input to match. |
| Normalize | Emit `https://isni.org/isni/<16-character-value>` with no embedded spaces or hyphens. |
| Components | Return the compact identifier as `identifier`/`suffix`, the canonical resolver base as `schema_uri`, the check character from `check_digit`, and the same canonical URL from `url` and `Display`. |
| Typed schemas | Enforce value/scheme agreement for ISNI in RAD, DataCite, InvenioRDM, and RAiD boundaries while leaving unknown extensible schemes unchanged. |
| Classify | Retain as person-or-organization evidence; never promote to project identity or infer a relationship from co-occurrence. |
| Resolve | With `--resolve`, make one exact registry lookup per unique normalized ISNI and accept one matching assigned record. |
| Report | Preserve the normalized ISNI and render a deterministic public person or organization name when available. |
| Persist | Retain the complete typed resolver outcome as discovery provenance; do not flatten identity metadata into unrelated project fields. |
| Offline/failure | Make no request without `--resolve`; reject `--resolve --offline`; keep locally valid evidence when resolution fails. |

# Recommendation

Build the smallest read-only provider around exact ISNI lookup. Use the public SRU `searchRetrieve` operation with `recordSchema=isni-b`, an exact `pica.isn` query, and a small `maximumRecords` bound. Do not implement name search: name results are ambiguous and would require a user-facing selection and reconciliation workflow. Do not implement registration, correction, AtomPub submission, or bulk downloads: those belong to registration agencies and authenticated data-contribution workflows.

Represent the response with a narrow typed envelope that distinguishes `personOrFiction` from `organisation`, retains canonical and merged ISNIs, data confidence, names/name use, source identifiers, organization type, and supplied location. Keep optional and repeated XML fields optional/repeated in Rust, and preserve the unmodified response body in resolver provenance if the typed subset cannot represent every returned public field. Never manufacture a legal name, preferred identity, affiliation, or person/organization relationship.

Choose the raw display name deterministically:

1. a non-empty public-use person name containing both forename and surname;
2. another complete person name;
3. a non-empty organization main name;
4. the canonical ISNI URL when no displayable name is supplied.

Deduplicate equivalent names before selection and preserve source order only as a final tie-breaker. The discovered identifier remains evidence even if the response reports it as a merged alias; retain the registry's canonical identifier separately and do not silently rewrite persisted historical evidence.

# Implementation plan

## 1. Close the portable ISNI contract

**Atomic write set:** `crates/acorn-schema/src/pid/isni/{mod.rs,tests.rs}`, `crates/acorn-schema/src/util/constants/mod.rs`.

- Separate explicit ISNI discovery syntax from explicitly typed parsing. Add an `ISNI` label-aware discovery pattern while retaining compact, spaced, and hyphenated values for callers that already specify `PID::ISNI`.
- Make `ISNI::find_all` accept resolver URLs and labeled values without excluding them merely because their payload also satisfies ORCID's checksum algorithm.
- Preserve the existing conservative generic policy for unlabeled 16-character values; document the ORCID/ISNI ambiguity instead of assigning both identifiers.
- Implement `PersistentIdentifier::url()` for ISNI and make it identical to the canonical `Display` output for populated identifiers.
- Keep normalization pure and allocation-only so `acorn-schema` remains available without `std`.
- Add focused tests for labeled compact/spaced/hyphenated forms, surrounding prose punctuation, lowercase `x` rejection or normalization according to the approved ISO contract, ORCID URLs, unlabeled ambiguity, malformed labels/domains, `url()` parity, empty values, and duplicate extraction.

**Gate:** `cargo nextest run -p acorn-schema --lib pid::isni` and the no-default/alloc checks pass.

## 2. Enforce ISNI in agent-identifier schema boundaries

**Atomic write set:** `crates/acorn-schema/src/research_activity/common.rs`, `crates/acorn-schema/src/validate/{mod.rs,tests.rs}`.

- Reuse one pure scheme-aware validator for `AgentIdentifier` and existing DataCite/Invenio name-identifier boundaries.
- When `scheme` is `ISNI`, require `value` to pass `rules::isni`; when a scheme URI is supplied, require an accepted canonical ISNI scheme/resolver URI after trailing-slash normalization.
- Apply the same coherence checks to ORCID and ROR while touching this shared validator so `AgentIdentifier` does not give ISNI weaker or inconsistent semantics. Preserve pass-through behavior for explicitly named extensible schemes.
- Attach the identifier value, scheme, and field path to structured validation errors. Do not perform network validation in `acorn-schema`.
- Test valid canonical and human-readable ISNIs, invalid checksums, mismatched schemes, mismatched scheme URIs, missing schemes, and unknown extensible schemes.

**Gate:** `cargo nextest run -p acorn-schema --lib validate` and `cargo nextest run -p acorn-schema --lib research_activity` pass.

## 3. Add an exact, read-only ISNI SRU provider

**Atomic write set:** `crates/acorn-lib/src/io/api/isni.rs`, `crates/acorn-lib/src/io/api/tests/isni.rs`, `tests/fixtures/isni/{person.xml,organization.xml,not-found.xml,diagnostic.xml,malformed.xml}`.

- Add documented request and response types for the SRU 1.1 `searchRetrieve` envelope and the `isni-b` enquiry record needed by exact lookup.
- Add `resolve(&ISNI)` plus a fixture-injectable `resolve_with(endpoint, &ISNI)` boundary. Validate and compact the identifier before network access.
- Build parameters as data, percent-encode them once, use HTTPS, set `maximumRecords` to a small bound, and request only `recordSchema=isni-b`. Do not interpolate unchecked input into CQL.
- Parse namespace-qualified XML without depending on element prefix spelling. Reject HTTP failures, SRU diagnostics, malformed XML, zero records, multiple assigned matches, missing canonical ISNI, and a response whose canonical/merged identifiers do not account for the requested value.
- Return a typed record containing the registry identity kind, names, canonical identifier, merged identifiers, confidence, sources, and other directly represented public fields. Preserve raw XML in the outcome only for fields the typed model deliberately does not project.
- Make timeouts and body-size limits use ACORN's existing bounded HTTP transport. Add no credential or mutation support.
- Use local fixtures and the in-process test server for all automated tests; keep the live registry probe manual and ignored so CI never depends on ISNI availability.

**Gate:** focused provider tests cover a person, an organization, not found, SRU diagnostic, malformed XML, HTTP failure, invalid input before network access, alias/canonical matching, and deterministic preferred-name selection.

## 4. Integrate resolution into gather

**Atomic write set:** `crates/acorn-lib/src/io/api/mod.rs`, `crates/acorn-lib/assets/constants/application.json`, `crates/acorn-lib/src/io/api/tests/mod.rs`, `crates/acorn-lib/src/analyzer/host/discovery/{mod.rs,tests/mod.rs}`.

- Register the ISNI provider and its OCLC-hosted SRU endpoint through the existing endpoint configuration rather than conflating it with `https://isni.org/isni`, which remains the identifier resolver/display URI.
- Add `PID::ISNI` to `Record::resolve` and serialize the typed response through the existing `ResolutionOutcome` path.
- Extend resolved raw rendering to emit `Name (canonical ISNI URL)` when a usable public name exists and the canonical URL otherwise.
- Keep ISNI out of `PID::is_project_identifier`, `ArtifactCandidate::try_from`, and partial RAD project-field mapping. Resolution must enrich the discovery report and provenance only unless a future source explicitly relates the identity to a contributor or organization field.
- Preserve current failure semantics: a resolver failure produces a failed lifecycle check while retaining the normalized local identifier. Ensure duplicate occurrences cause at most one logical lookup in the deduplicated records pipeline.
- Add tests for supported/resolved/failed status transitions, person and organization raw output, canonical merged aliases, metadata serialization, absence of project candidates, no resolution when not requested, and offline rejection at the CLI boundary.

**Gate:** `cargo nextest run -p acorn-lib --lib io::api::tests::isni`, focused discovery tests, and gather CLI tests pass without external network access.

## 5. Document and verify the promoted lifecycle

**Atomic write set:** `docs/src/concepts/persistent-identifiers.md`, `docs/src/cli-reference/commands/gather.md`.

- Replace the local-only ISNI statement with exact public-registry resolution behavior, endpoint/provenance semantics, and the person-or-organization evidence limitation.
- Document explicit-label discovery and why ACORN does not guess unlabeled ORCID/ISNI-compatible values.
- State that registry resolution confirms an assigned public record but does not prove legal identity, copyright ownership, or a project relationship.
- Include one canonical lookup example and the expected failure behavior without promising registry uptime or comprehensive biography data.

**Gate:** `make doc` succeeds and documentation contains no claim that ISNI is a project identifier or that network resolution is validation.

# Validation

Run focused gates after each slice, followed by:

```sh
cargo fmt --all --check
cargo nextest run -p acorn-schema --lib pid::isni
cargo nextest run -p acorn-schema --lib validate
cargo nextest run -p acorn-schema --lib research_activity
cargo nextest run -p acorn-lib --lib io::api::tests::isni
cargo nextest run -p acorn-lib --lib analyzer::host::discovery
cargo nextest run -p acorn-cli commands::gather
cargo check -p acorn-schema --no-default-features
cargo check -p acorn-schema --no-default-features --features alloc
make check-wasm
make lint
make test
make doc
```

No live network call is part of the required test suite. If implementation adds a dependency despite the current XML/HTTP stack being sufficient, run `make audit` and justify the dependency in the plan before changing manifests.

# Acceptance criteria

1. Every explicitly labeled ISNI and canonical ISNI resolver URL in supported input is discovered once, normalized to one HTTPS URL, and never misclassified as ORCID.
2. Explicitly typed compact, spaced, and hyphenated ISNIs validate only when their length, character set, and MOD 11-2 check character are correct.
3. `ISNI::url()`, `ISNI::to_string()`, `Identifier::normalized().value`, and documented canonical output agree.
4. RAD, DataCite, InvenioRDM, and RAiD ISNI fields reject invalid checksums and scheme mismatches without contacting the network.
5. `acorn gather --resolve` performs an exact, bounded public-registry lookup for each unique valid ISNI and records a typed resolved or failed outcome.
6. Person and organization fixtures both render deterministic public names while preserving the canonical ISNI, aliases, sources, and confidence as provenance.
7. Zero-result, diagnostic, ambiguous, malformed, timeout, and HTTP-error responses retain local ISNI evidence and produce actionable failures without panics.
8. ISNI never creates or merges a research-activity candidate and never implies a contributor, affiliation, ownership, or rights relationship from co-occurrence.
9. `--offline` and non-`--resolve` workflows make no ISNI network request.
10. Portable schema checks, WASM checks, focused tests, the full suite, lint, and documentation all pass with no external network dependency.
