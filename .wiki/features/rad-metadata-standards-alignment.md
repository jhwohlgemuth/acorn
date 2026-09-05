---
type: Plan
title: "Align research activity data with established metadata standards"
description: "Make RAD interoperable with RAiD, DataCite 4.7, DCAT, persistent-identifier registries, and contributor-role vocabularies without replacing its activity-focused authoring model."
difficulty: high
impact: high
package: [acorn-schema]
tags: [feature, research-activity, schema, datacite, datacite-4.7, raid, dcat, crosswalk, metadata, pid, credit, timeline]
status: in-progress
---

# Project code boundaries

- **Update:** `crates/acorn-schema/src/schema/research_activity/{mod.rs,output.rs}`, `crates/acorn-schema/src/schema/pid/raid/mod.rs`, `crates/acorn-schema/src/schema/standard/{datacite.rs,dcat/,crosswalk/}`, `crates/acorn-schema/src/validation/`, `crates/acorn-lib/src/analyzer/` traverse/check consumers, `crates/acorn-cli/src/commands/{check,export}/` crosswalk consumers, `tests/fixtures/schema/{datacite,raid,dcat}/`, and docs.
- **Create:** reusable contributor, relation, funding, lifecycle, license, and loss-reporting types in `crates/acorn-schema/src/schema/research_activity/` or `crates/acorn-schema/src/schema/standard/crosswalk/` plus pinned 4.7 conformance fixtures under `tests/fixtures/schema/datacite/4.7/` and `tests/fixtures/schema/raid/`.
- **Delete:** duplicated or lossy field adapters only after compatibility migrations and round-trip tests; no existing serialized field is removed implicitly. Remove temporary compat shims only after the deposit-readiness validator is the documented gate.

# Dependencies

This plan **provides** the stabilized portable schema/crosswalk contracts consumed by [Standards traversal](../refactors/analyzer/standards-traversal.md) and [CKAN/DCAT](../migrations/ckan-dcat-crosswalk.md); those plans must not start crosswalk integration until the loss-reporting and version contracts here land. The [research activity changelog](research-activity-changelog.md) consumes the mapping and loss-reporting contract and requires the DataCite 4.7 type audit (RAiD/SWHID identifier types, `RelatedItem`) before its Project-export loss table can be finalized. Coordinate `ResearchActivity`/`ResearchOutput` field additions with [OCP infrastructure alignment](ocp-infrastructure-schema-alignment.md) and [lifecycle/credit fields](../refactors/core/research-activity-lifecycle-credit-fields.md) through the schema-features lane lock.

# Prerequisite slice

Slice 0 (loss reporting + 4.7 audit) is the gating prerequisite for all later slices. No `ResearchActivity`/`ResearchOutput` field additions or `ResearchActivity <-> RAiD` / `ResearchOutput <-> DataCite` adapters may start until Slice 0 is green and its pinned fixtures are reviewed.

**Status 2026-09-03:** Slice 0 landed — `crates/acorn-schema/src/schema/standard/datacite.rs:1` now targets 4.7, `RelatedIdentifierType::{Raid,Swhid}` `datacite.rs:172`, `RelationType::Other` `datacite.rs:243`, `ResourceTypeGeneral::{Poster,Presentation}` `datacite.rs:323`, `RelatedItem`/`RelatedItems` + `relationTypeInformation` on `RelatedIdentifier`/`RelatedItem`, `validate_deposit_ready()` on `Record`/`Resource` `datacite.rs:1525`, crosswalk warnings propagated in `crosswalk/mod.rs:232` and `crosswalk/mapping.rs:194`, `analyzer/check.rs:1099` no longer swallows `Err`, `datacite_to_dcat` now uses `as_vec` for `doi->identifier`, and pinned fixtures under `tests/fixtures/schema/datacite/4.7/` with tests in `crates/acorn-schema/src/schema/standard/tests/datacite.rs` and `crates/acorn-schema/src/schema/standard/crosswalk/tests/mod.rs` are green (`cargo nextest -p acorn-schema` 278 passed, `make check-wasm` green, `rg "4\.6" crates/acorn-schema/src/schema/standard/datacite.rs` empty).

**Status 2026-09-05:** Slices 1–3 and most of Slice 4's additive schema surface landed. Shared portable metadata primitives now live in `research_activity/common.rs`; activity metadata includes typed dates, contributors, organizations, funding, related resources, language, subjects, spatial coverage, access, and license; and `ResearchOutput` now includes DataCite publisher, publication year and typed dates, resource types, language, alternate and related identifiers, related items, version, rights, descriptions, subjects, formats, sizes, geographic locations, and distinct creators. Output contributor roles and organizational contributors, complete funder identifiers, output deposit-readiness conversion, and the standards adapters in Slice 5 remain.

# Goal

Make the research activity data (RAD) schema a standards-aligned activity model with explicit, testable interchange profiles. Use RAiD for activity-level semantics, DataCite for research outputs and DOI exchange, DCAT for dataset distributions, established persistent-identifier schemes for identity, and CRediT for output contributor roles.

Preserve RAD's concise authoring fields, including `title`, `subtitle`, and `sections`, rather than reshaping the document into a direct copy of any deposit or registry schema.

# Current evidence

- `crates/acorn-schema/src/schema/research_activity/mod.rs` retains the concise identifier collections and primary `contact` while adding canonical `typedDates`, `contributors`, `organizations`, `funders`, `relatedResources`, `languages`, `subjects`, `spatialCoverage`, `rights`, and `access` fields. Compatibility aliases normalize legacy `sponsors` and `related` input into structured fields; precedence and standards-adapter behavior still need Slice 5 coverage.
- `crates/acorn-schema/src/schema/research_activity/output.rs:110` now provides the additive DataCite-facing fields listed in the 2026-09-05 status above. Its remaining profile gaps are controlled contributor roles, organizational contributors, complete funder identifiers, and a `ResearchOutput` conversion/readiness boundary; those gaps prevent claiming a complete round trip despite the expanded storage model.
- `crates/acorn-schema/src/schema/standard/datacite.rs:1` now models DataCite Metadata Schema 4.7 in JSON (`Record`/`Attributes` `datacite.rs:515`) and XML (`Resource` `datacite.rs:1128`) forms with `RelatedIdentifierType::Raid`/`Swhid` `datacite.rs:168`, `RelationType::Other` `datacite.rs:237`, `ResourceTypeGeneral::Poster`/`Presentation` `datacite.rs:321`, `RelatedItem`/`RelatedItems` and `relationTypeInformation` on `RelatedIdentifier` `datacite.rs:968` + `RelatedItem` `datacite.rs:1080`, and `validate_deposit_ready()` gates `datacite.rs:1525`. The two forms expose symmetric 4.7 coverage; mandatory JSON properties remain optional on `Attributes` with deposit readiness enforced via `validate_deposit_ready()` rather than `Validate`. Reconciled 2026-09-03 against `acorn-schema` at `0.1.76` (Slice 0).
- `crates/acorn-schema/src/schema/standard/crosswalk/` supports DataCite, DCAT, HuWise, and Invenio, but not RAD or `ResearchOutput`. `Standard::crosswalk` dispatch lives in `crates/acorn-lib/src/analyzer/check.rs:1044` (analysis feature) and will migrate with the crosswalk registry.
- `crates/acorn-schema/src/schema/standard/crosswalk/mod.rs:238` `convert` now propagates `FieldMapping::apply` `mapping.rs:194` missing list into `ConversionWarning::omitted` (`from`/`to`/`field` with details) and `crates/acorn-schema/src/schema/standard/crosswalk/mapping.rs:194` `FieldMapping::apply` now propagates `MissingRequiredField`/`TransformationFailed` instead of swallowing. `crates/acorn-lib/src/analyzer/check.rs:1099` `collect_crosswalk_warnings` now propagates `Err` instead of `Err(_) => Vec::new()`, and `crates/acorn-schema/src/schema/standard/crosswalk/mapping.rs:482` `resource_type_to_iri`/`iri_to_resource_type` handle `Poster`/`Presentation`. Reconciled 2026-09-03 (Slice 0).
- `crates/acorn-schema/src/schema/pid/raid/mod.rs` is the portable RAiD schema; RAD-to-RAiD assembly and inference historically lived in the adapter layer (now `crates/acorn-lib/src/io/adapter/`). When present, it has supplied current-date, principal-investigator/lead-organization roles, owner/registration-agency, and CC0 license defaults when RAD does not assert them, and leaves related objects, subjects, and spatial coverage empty. This plan makes inference opt-in and loss-explicit.
- The RAiD metadata model is designed around research activities and standardizes identifiers, dates, titles, contributors, organizations, related objects, subjects, and spatial coverage ([RAiD Metadata Schema](https://metadata.raid.org/en/latest/)). RAiD is governed by ISO 23527:2022 ([ISO overview](https://committee.iso.org/standard/75931.html)).
- DataCite 4.7 retains twenty metadata properties for citable resources and adds `Poster` and `Presentation` resource types, `RAiD` and `SWHID` related-identifier types, an `Other` relation type, and relation-type information ([property overview](https://datacite-metadata-schema.readthedocs.io/en/4.7/properties/), [4.7 update](https://datacite-metadata-schema.readthedocs.io/en/4.7/introduction/version-update/)). Its mandatory citation-oriented fields make it an appropriate output and deposit profile rather than the sole RAD activity model. The kernel-4.7 schema is pinned at `https://schema.datacite.org/meta/kernel-4.7/` and the XML catalog at `https://schema.datacite.org/meta/kernel-4.7/metadata.xsd`.
- DCAT 3 provides qualified relations and attributions for associating resources with typed roles and supports dataset distributions, access URLs, formats, licenses, and provenance ([DCAT 3](https://www.w3.org/TR/vocab-dcat-3/)). DCAT-US v3 profile is tracked separately in [CKAN/DCAT](../migrations/ckan-dcat-crosswalk.md).
- CRediT defines fourteen roles for contributions to research outputs ([CRediT taxonomy](https://credit.niso.org/)). Crossref recommends ROR or Open Funder Registry identifiers and award numbers or grant DOIs in structured funding metadata ([Crossref funding guidance](https://www.crossref.org/documentation/principles-practices/best-practices/funding)).

# Standards profile

| Concern | Primary standard | RAD responsibility |
|---|---|---|
| Research activity identity and lifecycle | RAiD | Dates, contributors, organizations, roles, related objects, subjects, and spatial coverage |
| Publications, datasets, software, and other products | DataCite 4.7 | Output description, validation (`Record`/`Resource`), DOI import, DOI export, `RelatedItem`, `RelatedIdentifier` with `RAiD`/`SWHID` types, `relationType=Other` + `relationTypeInformation` |
| Dataset access and representations | DCAT 3 | Distributions, landing pages, downloads, media types, licenses, and provenance |
| People and organizations | ORCID, ROR, ISNI | Typed, resolvable identifiers with scheme URIs |
| Output contribution roles | CRediT | Controlled output contributor responsibilities distinct from DataCite `ContributorType` |
| Funding | DataCite and Crossref | Funder identity, award number, award URI, and grant DOI |
| Web discovery | Schema.org JSON-LD | Derived discovery representation rather than canonical storage |
| Software outputs | CodeMeta | Optional software-specific interchange profile |

DataCite 4.7 delta watchlist: `ResourceTypeGeneral::Poster`, `Presentation`; `RelatedIdentifierType::RAiD`, `SWHID`; `RelationType::Other`; `RelatedIdentifier.relationTypeInformation`; `RelatedItem` container. All doc links in `crates/acorn-schema/src/schema/standard/datacite.rs` must point to `4.7` after the audit.

# Recommendation

Treat RAD as the canonical activity-focused model and publish explicit profiles at its boundaries:

- Map `ResearchActivity` to and from RAiD metadata where the source contains sufficient assertions.
- Map `ResearchOutput` to and from DataCite records, with a separate deposit-readiness validator for DataCite mandatory properties (distinct from `Validate` on `Attributes`/`Resource`).
- Permit an explicit `ResearchActivity -> DataCite Project` export for users who want to register an activity DOI, but do not require every RAD document to satisfy DataCite deposit requirements.
- Keep the manually authored, discovery-ready `changelog` canonical in RAD. Do not infer DataCite `Updated` from entry occurrence timestamps; Project export may map an explicitly selected aggregate description, but it must report entry details as local-only/lossy rather than flatten the changelog into DataCite.
- Map dataset outputs to DCAT and software outputs to CodeMeta without adding standard-specific fields to unrelated output types.
- Derive Schema.org JSON-LD from canonical RAD values instead of maintaining a parallel discovery record.
- Reject missing target-required values or report them as structured conversion losses. Do not fabricate dates, roles, identifiers, organizations, licenses, or publishers.

Keep the first implementation additive. Do not make schema versioning or a general migration framework a prerequisite for correcting crosswalks, adding optional structured fields, or validating standards conformance.

This plan is a coordination record for the full alignment. Implementation must proceed in gated slices (Slice 0 below is the gate). Broad `crates/acorn-schema/src/schema/` directory ownership is a planning boundary, not an atomic write set — each slice must declare exact files.

# Implementation plan

Slices 0–1 are the portable foundation and must land before activity/output field additions. Each slice declares exact files; directory ownership above is a planning boundary, not a concurrency claim.

## 0. Gate: loss reporting + DataCite 4.7 audit (prerequisite for all later slices)

**Write set:** `crates/acorn-schema/src/schema/standard/datacite.rs`, `crates/acorn-schema/src/schema/standard/crosswalk/{mod.rs,mapping.rs}`, `crates/acorn-schema/src/schema/standard/tests/datacite.rs`, `tests/fixtures/schema/datacite/4.7/` (new), `tests/fixtures/schema/datacite-invalid-*.json`.

- Pin 4.7 conformance fixtures: at least one valid JSON record exercising every 4.7 delta, one valid `Resource` XML exercising `RelatedItem`, plus two invalid records (missing mandatory `titles`/`creators`/`publisher`/`publicationYear`/`resourceType`). Source from `https://schema.datacite.org/meta/kernel-4.7/` and `https://schema.datacite.org/meta/kernel-4.7/example/`; vendor locally, do not fetch at test time.
- Audit `datacite.rs:31` `ContributorType`, `datacite.rs:89` `DateType`, `datacite.rs:172` `RelatedIdentifierType`, `datacite.rs:243` `RelationType`, `datacite.rs:323` `ResourceTypeGeneral` against 4.7. Required deltas:
  - `RelatedIdentifierType::Raid`, `Swhid` (cover RAiD `10.83962/...` and SWHID `swh:1:...`)
  - `RelationType::Other` + optional `relationTypeInformation: Option<String>` on `RelatedIdentifier` `datacite.rs:933` and equivalent on `RelatedItem`
  - `ResourceTypeGeneral::Poster`, `Presentation`
  - New `RelatedItem` and `RelatedItems` types for kernel-4.7 (distinct from `RelatedIdentifier`)
  - Update every doc comment link from `4.6` to `4.7` (grep gate: `rg "4\.6" crates/acorn-schema/src/schema/standard/datacite.rs` must be empty)
  - Make JSON (`Attributes` `datacite.rs:495`) and XML (`Resource` `datacite.rs:969`) property coverage symmetric where the spec defines parity; document intentional asymmetry in the field matrix.
- Repair loss reporting:
  - `crosswalk/mod.rs:232` `convert`: propagate the `Vec<String>` returned by `FieldMapping::apply` `mapping.rs:194` into `Vec<ConversionWarning>` (one `Omitted` warning per missing optional source field, with `from`/`to`/`field` and `details` explaining loss). Do not discard the list; do not initialize `warnings` as empty and return it unchanged.
  - `check.rs:1099` `collect_crosswalk_warnings`: remove the `Err(_) => Vec::new()` swallowing; surface `TransformationFailed` distinctly from `Omitted`.
  - Distinguish warning kinds: `Omitted` (no target field), `Approximated` (semantic shift), `Simplified` (structure collapsed), `BlockNotSelected` remains HuWise-only. Add `source_path`/`target_path` to `ConversionWarning` so CLI can print `attributes.titles[1] -> ...`.
  - Test required-field failures separately from successful-but-lossy conversions; assert `MissingRequiredField` vs. `Ok((_, warnings))` with non-empty warnings.

Deliverable: `cargo nextest run -p acorn-schema standard::datacite` validates pinned 4.7 fixtures; `crosswalk/mod.rs::convert` never returns `Ok((_, []))` when `FieldMapping::apply` reports missing fields.

## 1. Define the conformance and crosswalk contract

- Create a field matrix for `ResearchActivity`, `Changelog`, `ResearchOutput`, RAiD 1.0, DataCite 4.7, DCAT 3, CodeMeta, and Schema.org.
- Classify each mapping as exact, transformed, lossy, local-only, conditionally required, or unsupported.
- Record controlled vocabulary ownership, occurrence constraints, identifier normalization, and language/date formats.
- Define separate validity levels for a valid RAD record, a RAiD-ready activity, and a DataCite-deposit-ready output — the latter is the new `validate_deposit_ready(&Record)` / `validate_deposit_ready(&Resource)` gate, not `Validate` on `Attributes`.
- Pin official RAiD and DataCite 4.7 fixtures used for conformance tests (reuses Slice 0 fixtures; matrix references them by path).

Deliverable: a reviewed mapping table and fixture set that resolve field meaning before Rust types change. Matrix lives in `.wiki/assets/standards-field-matrix.md` and is linked from this plan.

## 2. Add reusable metadata primitives

Add the smallest common types required by the approved matrix (do not add speculative primitives):

- typed identifier with value, scheme, and scheme URI
- agent identifier supporting ORCID, ROR, ISNI, and extensible schemes (reuse existing `ResearchOutput::Affiliation` `output.rs:35` where semantics match)
- dated agent and organization role
- structured activity contributor and organization references
- typed activity date
- typed related resource with identifier, relation, resource type, and activity category
- structured funding reference and award (extend `Funding`/`Award` `output.rs:50,94` rather than duplicating)
- controlled subject with scheme and value URIs
- structured rights and access statements

Reuse primitives across activity and output metadata where semantics are identical. Keep standard-specific containers in `schema::standard` and convert at the boundary. Prefer extending `crates/acorn-schema/src/schema/research_activity/output.rs:113` and `crates/acorn-schema/src/schema/pid/` types over new top-level modules.

Deliverable: portable, validated primitives with adjacent unit tests and no network-dependent validation. No new network, filesystem, or credential fields.

## 3. Complete the activity-level RAD model

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

Retain `title`, `subtitle`, and `sections` as the authoring model. Crosswalks should derive typed titles and descriptions from those fields rather than add a second canonical title hierarchy. Coordinate `organizations`/`contributors`/`funding` additions with `ocp-infrastructure-schema-alignment.md``ComputePlatform`/`ComputeFacility` to avoid dual ownership of org identity.

Deliverable: RAD can represent the RAiD concepts ACORN currently fabricates or drops, without fabricating defaults.

## 4. Complete the output-level DataCite profile

**Partially implemented 2026-09-05:** The additive fields below are present except controlled contributor roles, organizational contributors, and complete funder identifiers. Conversion, round-trip loss reporting, and output-specific deposit readiness remain.

Extend `ResearchOutput` `output.rs:113` with:

- creators distinct from other contributors
- controlled contributor roles, using CRediT where applicable (distinct from `ContributorType` `datacite.rs:35`)
- personal and organizational contributors
- publisher with optional identifier (`publisherIdentifier` parity with `datacite.rs:898`)
- publication year and typed dates (`Date` `datacite.rs:654` with `dateType`)
- controlled general resource type plus a specific type (`ResourceTypes` `datacite.rs:1038`)
- language, alternate identifiers, typed related identifiers (including `RAiD`/`SWHID`), `RelatedItem`s, and version
- structured rights, descriptions, subjects, formats, sizes, and geographic coverage
- complete funder and award identifiers

Audit already performed in Slice 0; this slice implements the `ResearchOutput` side and the deposit-readiness validator. Enforce mandatory DataCite properties only in `validate_deposit_ready` without making them mandatory for partially enriched RAD output records.

Deliverable: a `ResearchOutput` can round-trip every field ACORN claims is lossless and can explain why an incomplete output is not deposit-ready.

## 5. Implement standards adapters and user workflows

- Implement `ResearchActivity <-> RAiD Metadata` with no inferred assertions; remove or gate the legacy inferred date/role/license defaults behind an explicit `allow_infer: bool` that defaults to `false` and emits `ConversionWarning::Approximated` when used.
- Implement `ResearchOutput <-> DataCite Record` with explicit loss reporting (reuses Slice 0 warning pipeline).
- Implement opt-in `ResearchActivity -> DataCite Project` export (general `ResourceTypeGeneral::Project` `datacite.rs:407`).
- Implement dataset-output DCAT and software-output CodeMeta adapters only after their mappings are approved in the matrix.
- Generate Schema.org JSON-LD contexts and types from the canonical activity and output structures.
- Expose validation and conversion through existing `cargo run --bin acorn -- check` / `export` / `schema` workflows instead of adding a new command unless existing command semantics cannot express the operation.
- Honor offline mode and use local fixtures for tests.

Deliverable: users can inspect conformance, conversion failures, and conversion losses before writing output. No network request occurs during `cargo nextest run -p acorn-schema`.

## 6. Address compatibility and versioning only when required

- Preserve existing RAD fields as accepted input while additive structured fields are introduced.
- Define deterministic precedence when a legacy string list field (e.g., `meta.sponsors: Option<Vec<String>>` `mod.rs:278`) and its structured replacement are both present, and report conflicts as `ConversionWarning::Approximated` or validation errors — do not silently prefer one.
- Add focused input normalization at the parse boundary (`eserde` compat) rather than a general migration subsystem.
- Add an explicit RAD schema version only if a later change cannot remain additive or precedence rules become ambiguous.
- Reserve removal of legacy fields for a separately approved breaking-change plan and major schema release.

Deliverable: existing documents continue to load without delaying the standards-alignment work for speculative migrations.

# Validation

Per-slice focused gates, then repository gates:

```sh
cargo nextest run -p acorn-schema --lib standard::datacite
cargo nextest run -p acorn-schema --lib standard::crosswalk
cargo nextest run -p acorn-schema --lib research_activity
cargo nextest run -p acorn-schema --lib pid::raid
cargo check -p acorn-schema --no-default-features
cargo check -p acorn-schema --no-default-features --features alloc
make check-wasm
make lint
make test
make audit # if deps change
```

Additional checks:

- Validate pinned DataCite 4.7 JSON and XML fixtures and two invalid records (`datacite-invalid-*.json`); assert `rg "4\.6" crates/acorn-schema/src/schema/standard/datacite.rs` is empty after Slice 0.
- Validate official RAiD examples and missing-required-field cases.
- Add golden tests for `RAD <-> RAiD` and `ResearchOutput <-> DataCite Record` conversions, including 4.7 `RAiD`/`SWHID`/`Other`/`RelatedItem` cases.
- Add property tests for identifier normalization (DOI, RAiD, SWHID, ROR, ORCID, ISNI), relation mappings, and lossless round trips.
- Assert that conversions never fabricate metadata and that every intentional loss emits a `ConversionWarning` with `from`/`to`/`field`/`loss_type`/`details` and that required-field failures are `CrosswalkError::MissingRequiredField`, not warnings.
- Retain compatibility fixtures for established RAD field names and Markdown front matter (`crates/acorn-schema/src/schema/research_activity/tests/`).
- Snapshot generated RAD, RAiD, DataCite, and JSON-LD schemas or representations where stable output is part of the contract (`insta` snapshots in `crates/acorn-schema/src/schema/standard/tests/snapshots/`).
- Run focused crosswalk/CLI tests after adapter slices: `cargo nextest run -p acorn-cli --lib crosswalk` (or `check`/`export` consumers).
- `cargo fmt --all` before every PR.

# Acceptance criteria

- Every canonical RAD field has a documented standards mapping or an explicit local-only designation in the pinned matrix.
- RAiD export contains no dates, roles, organizations, identifiers, or licenses absent from the RAD source unless `allow_infer: true` is explicitly set, in which case a warning is emitted.
- DataCite validation distinguishes valid internal output metadata (lenient `Validate`) from deposit-ready metadata (`validate_deposit_ready`) and reports every missing mandatory property (`titles`, `creators`, `publisher`, `publicationYear`, `resourceType`).
- `ResearchOutput` supports all DataCite 4.7 properties ACORN documents as supported, or the unsupported property is identified as `unsupported` in the mapping matrix. Specifically, `RelatedIdentifierType` includes `RAiD`/`SWHID`, `RelationType` includes `Other`, `ResourceTypeGeneral` includes `Poster`/`Presentation`, and `RelatedItem` is modeled after the 4.7 schema.
- `crates/acorn-schema/src/schema/standard/datacite.rs` contains no `4.6` doc links; all links point to `4.7`.
- JSON (`Record`/`Attributes`) and XML (`Resource`) DataCite forms have symmetric coverage for spec-defined parity; intentional asymmetry is documented.
- Changelog entries remain losslessly represented in RAD; every value omitted from RAiD or DataCite projection is classified and reported as a warning with source/target paths.
- Every omitted, defaulted, coerced, unsupported, or many-to-one field produces a structured `ConversionWarning`; lossy-but-successful conversions never return an empty warning list when `FieldMapping::apply` reports missing fields.
- Existing RAD fixtures continue to deserialize throughout the additive implementation phases.
- Lossless round-trip tests retain all fields classified as exact or reversible transformations.
- Generated JSON Schema exposes controlled values, nested validation, and conditional requirements where supported; `RelatedItem`/`RAiD`/`SWHID` appear in schema output.
- Default, minimal (`--no-default-features`), `alloc`, and WASM builds remain supported; `make check-wasm` passes per slice.
- No network request occurs during schema validation or fixture-backed tests; `offline` mode respected in CLI consumers.

# Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| A union of standards makes RAD too complex to author. | High | Keep concise RAD authoring fields and isolate standard-specific shapes behind adapters. |
| Similar fields become competing sources of truth. | High | Approve canonical ownership in the mapping matrix and define conflict precedence before adding replacements. Document precedence for `meta.sponsors` vs structured funding. |
| Conversions silently discard metadata. | High | Land Slice 0 (warning propagation) before any RAD crosswalks and test every lossy rule with golden warnings. |
| RAiD or DataCite requirements change. | Medium | Pin profile versions (`kernel-4.7` XSD, RAiD 1.x fixtures), keep controlled vocabularies explicit, and isolate adapters from canonical types. |
| Deposit requirements make partially enriched records invalid. | Medium | Separate RAD validity (`Validate`) from target-specific readiness (`validate_deposit_ready`). |
| Compatibility work expands into an early redesign. | Medium | Prefer additive fields and defer schema versioning until a demonstrated breaking change requires it. |
| 4.7 audit scope creep (full re-derive without fixtures) | Medium | Pin two valid + two invalid 4.7 fixtures first; diff types against them; gate with `rg "4\.6"` empty check. |
| Primitives duplicate existing `ResearchOutput::Affiliation`/`Funding`/`Award` | Medium | Inventory `output.rs:35,50,94` before adding types; extend existing where semantics match. |
| Crosswalk warning fix breaks existing `TryFrom` callers that ignore warnings | Medium | Keep `TryFrom` signature infallible at type level; add warning-aware path separately (`crosswalk_with_warnings`); do not change `TryFrom` to return warnings. |
| Schema-features lane contention on `ResearchActivity` | High | Serialize `ResearchActivity`/`ResearchOutput` edits via the schema-features lane lock `parallel-execution.md`; each slice declares exact files and integration owner. |

# Related concepts

- [SWHID persistent-identifier hardening](swhid-persistent-identifier.md)
- [Research activity changelog](research-activity-changelog.md)
- [OCP infrastructure schema alignment](ocp-infrastructure-schema-alignment.md)
- [CKAN/DCAT crosswalk](../migrations/ckan-dcat-crosswalk.md)
- [Standards traversal](../refactors/analyzer/standards-traversal.md)
- [Lifecycle, credit, license, and award fields](../refactors/core/research-activity-lifecycle-credit-fields.md)
