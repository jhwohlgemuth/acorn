---
type: Plan
title: "Align research activity data with established metadata standards"
description: "Make RAD interoperable with RAiD, DataCite, DCAT, persistent-identifier registries, and contributor-role vocabularies without replacing its activity-focused authoring model."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [feature, research-activity, schema, datacite, raid, dcat, crosswalk, metadata, pid, credit, timeline]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/schema/research_activity/{mod,output}.rs`, `src/schema/pid/raid/mod.rs`, `src/schema/standard/{datacite,dcat,crosswalk}/`, analyzers/crosswalk CLI consumers, fixtures, and docs; target `acorn-schema` after extraction.
- **Create:** reusable contributor, relation, funding, lifecycle, license, and loss-reporting types plus conformance fixtures under the schema boundary.
- **Delete:** duplicated or lossy field adapters only after compatibility migrations and round-trip tests; no existing serialized field is removed implicitly.

# Dependencies

[Standards traversal](../refactors/analyzer/standards-traversal.md) and [CKAN/DCAT](../migrations/ckan-dcat-crosswalk.md) consume the stabilized schema/crosswalk contracts. The [research activity changelog](research-activity-changelog.md) consumes the mapping and loss-reporting contract while remaining canonical RAD content.

# Goal

Make the research activity data (RAD) schema a standards-aligned activity model with explicit, testable interchange profiles. Use RAiD for activity-level semantics, DataCite for research outputs and DOI exchange, DCAT for dataset distributions, established persistent-identifier schemes for identity, and CRediT for output contributor roles.

Preserve RAD's concise authoring fields, including `title`, `subtitle`, and `sections`, rather than reshaping the document into a direct copy of any deposit or registry schema.

# Current evidence

- `crates/acorn-lib/src/schema/research_activity/mod.rs` stores activity identifiers in separate `doi`, `handle`, `raid`, `swhid`, and `ror` collections. It represents one primary `contact`, while sponsors, partners, and related activities remain string lists.
- `crates/acorn-lib/src/schema/research_activity/output.rs` provides structured research outputs with contributors, affiliations, funding, awards, websites, keywords, and access metadata. It does not yet represent the complete DataCite property set, including creator/contributor distinction, publisher, language, version, typed relations, formats, sizes, geographic coverage, and multiple typed descriptions.
- `crates/acorn-lib/src/schema/standard/datacite.rs` models a substantial portion of DataCite 4.6 in JSON and XML forms, but DataCite 4.7 is now current, the two forms do not expose the same properties, and several mandatory JSON properties are optional Rust fields. The implementation therefore needs a version and conformance audit before it becomes the RAD output contract.
- `crates/acorn-lib/src/schema/standard/crosswalk/` supports DataCite, DCAT, HuWise, and Invenio, but not RAD or `ResearchOutput`.
- `crosswalk::convert` initializes an empty warning collection and discards the missing-field list returned by `FieldMapping::apply`, so a lossy conversion can currently return no `ConversionWarning` values.
- `crates/acorn-lib/src/schema/pid/raid/mod.rs` converts RAD to RAiD metadata by supplying the current date, principal-investigator and lead-organization roles, owner and registration-agency values, and a CC0 license when the RAD record does not assert them. It leaves related objects, subjects, and spatial coverage empty.
- The RAiD metadata model is designed around research activities and standardizes identifiers, dates, titles, contributors, organizations, related objects, subjects, and spatial coverage ([RAiD Metadata Schema](https://metadata.raid.org/en/latest/)). RAiD is governed by ISO 23527:2022 ([ISO overview](https://committee.iso.org/standard/75931.html)).
- DataCite 4.7 retains twenty metadata properties for citable resources and adds `Poster` and `Presentation` resource types, `RAiD` and `SWHID` related-identifier types, an `Other` relation type, and relation-type information ([property overview](https://datacite-metadata-schema.readthedocs.io/en/4.7/properties/), [4.7 update](https://datacite-metadata-schema.readthedocs.io/en/4.7/introduction/version-update/)). Its mandatory citation-oriented fields make it an appropriate output and deposit profile rather than the sole RAD activity model.
- DCAT 3 provides qualified relations and attributions for associating resources with typed roles and supports dataset distributions, access URLs, formats, licenses, and provenance ([DCAT 3](https://www.w3.org/TR/vocab-dcat-3/)).
- CRediT defines fourteen roles for contributions to research outputs ([CRediT taxonomy](https://credit.niso.org/)). Crossref recommends ROR or Open Funder Registry identifiers and award numbers or grant DOIs in structured funding metadata ([Crossref funding guidance](https://www.crossref.org/documentation/principles-practices/best-practices/funding)).

# Standards profile

| Concern | Primary standard | RAD responsibility |
|---|---|---|
| Research activity identity and lifecycle | RAiD | Dates, contributors, organizations, roles, related objects, subjects, and spatial coverage |
| Publications, datasets, software, and other products | DataCite 4.7 | Output description, validation, DOI import, and DOI export |
| Dataset access and representations | DCAT 3 | Distributions, landing pages, downloads, media types, licenses, and provenance |
| People and organizations | ORCID, ROR, ISNI | Typed, resolvable identifiers with scheme URIs |
| Output contribution roles | CRediT | Controlled output contributor responsibilities |
| Funding | DataCite and Crossref | Funder identity, award number, award URI, and grant DOI |
| Web discovery | Schema.org JSON-LD | Derived discovery representation rather than canonical storage |
| Software outputs | CodeMeta | Optional software-specific interchange profile |

# Recommendation

Treat RAD as the canonical activity-focused model and publish explicit profiles at its boundaries:

- Map `ResearchActivity` to and from RAiD metadata where the source contains sufficient assertions.
- Map `ResearchOutput` to and from DataCite records, with a separate deposit-readiness validator for DataCite mandatory properties.
- Permit an explicit `ResearchActivity -> DataCite Project` export for users who want to register an activity DOI, but do not require every RAD document to satisfy DataCite deposit requirements.
- Keep the manually authored, discovery-ready `changelog` canonical in RAD. Do not infer DataCite `Updated` from entry occurrence timestamps; Project export may map an explicitly selected aggregate description, but it must report entry details as local-only/lossy rather than flatten the changelog into DataCite.
- Map dataset outputs to DCAT and software outputs to CodeMeta without adding standard-specific fields to unrelated output types.
- Derive Schema.org JSON-LD from canonical RAD values instead of maintaining a parallel discovery record.
- Reject missing target-required values or report them as structured conversion losses. Do not fabricate dates, roles, identifiers, organizations, licenses, or publishers.

Keep the first implementation additive. Do not make schema versioning or a general migration framework a prerequisite for correcting crosswalks, adding optional structured fields, or validating standards conformance.

# Implementation plan

## 1. Define the conformance and crosswalk contract

- Create a field matrix for `ResearchActivity`, `Changelog`, `ResearchOutput`, RAiD 1.0, DataCite 4.7, DCAT 3, CodeMeta, and Schema.org.
- Classify each mapping as exact, transformed, lossy, local-only, conditionally required, or unsupported.
- Record controlled vocabulary ownership, occurrence constraints, identifier normalization, and language/date formats.
- Define separate validity levels for a valid RAD record, a RAiD-ready activity, and a DataCite-deposit-ready output.
- Pin official RAiD and DataCite 4.7 fixtures used for conformance tests.

Deliverable: a reviewed mapping table and fixture set that resolve field meaning before Rust types change.

## 2. Make crosswalk loss reporting trustworthy

- Propagate missing optional mappings from `FieldMapping::apply` into `ConversionWarning` values.
- Distinguish omitted, defaulted, coerced, many-to-one, and unsupported fields.
- Add source and target paths to warnings so CLI output can identify the affected value.
- Test required-field failures separately from successful but lossy conversions.
- Remove inferred values from RAD-to-RAiD conversion unless the source explicitly asserts them.

Deliverable: conversions never appear lossless when fields were omitted or invented.

## 3. Add reusable metadata primitives

Add the smallest common types required by the approved mappings:

- typed identifier with value, scheme, and scheme URI
- agent identifier supporting ORCID, ROR, ISNI, and extensible schemes
- dated agent and organization role
- structured activity contributor and organization references
- typed activity date
- typed related resource with identifier, relation, resource type, and activity category
- structured funding reference and award
- controlled subject with scheme and value URIs
- structured rights and access statements

Reuse these primitives across activity and output metadata where their semantics are identical. Keep standard-specific containers in `schema::standard` and convert at the boundary.

Deliverable: portable, validated primitives with adjacent unit tests and no network-dependent validation.

## 4. Complete the activity-level RAD model

Add optional structured fields for:

- actual activity start and end dates plus metadata creation and update dates
- contributors with identifiers, affiliations, roles, contact/leader assertions, and active periods
- organizations with identifiers, roles, and active periods
- funders and associated awards
- typed related activities and related objects
- language and controlled subjects
- spatial coverage
- metadata access and license
- typed identifiers for new identifier schemes

Retain `title`, `subtitle`, and `sections` as the authoring model. Crosswalks should derive typed titles and descriptions from those fields rather than add a second canonical title hierarchy.

Deliverable: RAD can represent the RAiD concepts ACORN currently fabricates or drops.

## 5. Complete the output-level DataCite profile

Extend `ResearchOutput` with:

- creators distinct from other contributors
- controlled contributor roles, using CRediT where applicable
- personal and organizational contributors
- publisher with optional identifier
- publication year and typed dates
- controlled general resource type plus a specific type
- language, alternate identifiers, typed related identifiers, and version
- structured rights, descriptions, subjects, formats, sizes, and geographic coverage
- complete funder and award identifiers

Audit the existing DataCite JSON and XML types against all DataCite 4.7 properties, including `RelatedItem`, RAiD/SWHID related-identifier types, `relationType=Other`, and `relationTypeInformation`. Enforce mandatory properties in the deposit-readiness validator without making them mandatory for partially enriched RAD output records.

Deliverable: a `ResearchOutput` can round-trip every field ACORN claims is lossless and can explain why an incomplete output is not deposit-ready.

## 6. Implement standards adapters and user workflows

- Implement `ResearchActivity <-> RAiD Metadata` with no inferred assertions.
- Implement `ResearchOutput <-> DataCite Record` with explicit loss reporting.
- Implement opt-in `ResearchActivity -> DataCite Project` export.
- Implement dataset-output DCAT and software-output CodeMeta adapters only after their mappings are approved.
- Generate Schema.org JSON-LD contexts and types from the canonical activity and output structures.
- Expose validation and conversion through existing schema, check, import, export, format, or gather workflows instead of adding a new command unless existing command semantics cannot express the operation.
- Honor offline mode and use local fixtures for tests.

Deliverable: users can inspect conformance, conversion failures, and conversion losses before writing output.

## 7. Address compatibility and versioning only when required

- Preserve existing RAD fields as accepted input while additive structured fields are introduced.
- Define deterministic precedence when a legacy field and its structured replacement are both present, and report conflicts.
- Add focused input normalization at the parse boundary rather than a general migration subsystem.
- Add an explicit RAD schema version only if a later change cannot remain additive or precedence rules become ambiguous.
- Reserve removal of legacy fields for a separately approved breaking-change plan and major schema release.

Deliverable: existing documents continue to load without delaying the standards-alignment work for speculative migrations.

# Validation

- Validate official DataCite 4.7 JSON and XML examples and representative invalid records.
- Validate official RAiD examples and missing-required-field cases.
- Add golden tests for RAD-to-RAiD and output-to-DataCite conversions.
- Add property tests for identifier normalization, relation mappings, and lossless round trips.
- Assert that conversions never fabricate metadata and that every intentional loss emits a warning.
- Retain compatibility fixtures for established RAD field names and Markdown front matter.
- Snapshot generated RAD, RAiD, DataCite, and JSON-LD schemas or representations where stable output is part of the contract.
- Run focused schema, PID, crosswalk, enrichment, Markdown, and CLI tests.
- Run `cargo fmt --all`, `make lint`, `make test`, and `make wasm`.
- Run `make audit` if implementation adds or changes dependencies.

# Acceptance criteria

- Every canonical RAD field has a documented standards mapping or an explicit local-only designation.
- RAiD export contains no dates, roles, organizations, identifiers, or licenses absent from the RAD source.
- DataCite validation distinguishes valid internal output metadata from deposit-ready metadata and reports every missing mandatory property.
- `ResearchOutput` supports all DataCite 4.7 properties ACORN documents as supported, or the unsupported property is identified in the mapping matrix.
- Changelog entries remain losslessly represented in RAD; every value omitted from RAiD or DataCite projection is classified and reported.
- Every omitted, defaulted, coerced, unsupported, or many-to-one field produces a structured conversion warning.
- Existing RAD fixtures continue to deserialize throughout the additive implementation phases.
- Lossless round-trip tests retain all fields classified as exact or reversible transformations.
- Generated JSON Schema exposes controlled values, nested validation, and conditional requirements where supported.
- Default, minimal, alloc, and WASM builds remain supported.
- No network request occurs during schema validation or fixture-backed tests.

# Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| A union of standards makes RAD too complex to author. | High | Keep concise RAD authoring fields and isolate standard-specific shapes behind adapters. |
| Similar fields become competing sources of truth. | High | Approve canonical ownership in the mapping matrix and define conflict precedence before adding replacements. |
| Conversions silently discard metadata. | High | Repair warning propagation before adding RAD crosswalks and test every lossy rule. |
| RAiD or DataCite requirements change. | Medium | Pin profile versions, keep controlled vocabularies explicit, and isolate adapters from canonical types. |
| Deposit requirements make partially enriched records invalid. | Medium | Separate RAD validity from target-specific readiness validation. |
| Compatibility work expands into an early redesign. | Medium | Prefer additive fields and defer schema versioning until a demonstrated breaking change requires it. |

# Related concepts

- [SWHID persistent-identifier support](swhid-persistent-identifier.md)
- [Standards traversal](../refactors/analyzer/standards-traversal.md)
