---
type: Plan
title: "Align research activity data with established metadata standards"
description: "Make RAD interoperable with RAiD, DataCite 4.7, DCAT, persistent-identifier registries, and contributor-role vocabularies without replacing its activity-focused authoring model."
difficulty: high
impact: high
package: [acorn-schema, acorn-lib, acorn-cli]
tags: [feature, research-activity, schema, datacite, datacite-4.7, raid, dcat, crosswalk, metadata, pid, credit, timeline]
status: in-progress
---

# Project code boundaries

- **Update:** `crates/acorn-schema/src/research_activity/{mod.rs,output.rs}`, `crates/acorn-schema/src/pid/raid/mod.rs`, `crates/acorn-schema/src/standard/{datacite.rs,dcat/,crosswalk/}`, `crates/acorn-schema/src/validation/`, `crates/acorn-lib/src/{analyzer/,io/enrichment.rs,io/workflow/}` traverse/check/enrichment consumers, `crates/acorn-cli/src/commands/{check,export,format}/` crosswalk and enrichment consumers, `tests/fixtures/schema/{datacite,raid,dcat}/`, and docs.
- **Create:** reusable contributor, relation, funding, lifecycle, license, and loss-reporting types in `crates/acorn-schema/src/research_activity/` or `crates/acorn-schema/src/standard/crosswalk/` plus pinned 4.7 conformance fixtures under `tests/fixtures/schema/datacite/4.7/` and `tests/fixtures/schema/raid/`.
- **Delete:** duplicated or lossy field adapters only after compatibility migrations and round-trip tests; no existing serialized field is removed implicitly. Remove temporary compat shims only after the deposit-readiness validator is the documented gate.

# Dependencies

This plan **provides** the stabilized portable schema/crosswalk contracts consumed by [Standards traversal](../refactors/analyzer/analyzer-composition.md) and [CKAN/DCAT](../migrations/ckan-dcat-crosswalk.md); those plans must not start crosswalk integration until the loss-reporting and version contracts here land. The implemented research activity logbook consumes the mapping and loss-reporting contract and requires the DataCite 4.7 type audit (RAiD/RRID/SWHID identifier types, `RelatedItem`) before its Project-export loss table can be finalized. The landed portable RRID baseline also delegates `TypedRelatedResource` RRID relation conversion and loss reporting to this plan's `ResearchActivity -> DataCite Project` and `ResearchOutput <-> DataCite Record` adapters. Coordinate `ResearchActivity`/`ResearchOutput` field additions with [OCP infrastructure alignment](ocp-infrastructure-schema-alignment.md) and [lifecycle/credit fields](../refactors/core/research-activity-lifecycle-credit-fields.md) through the schema-features lane lock.

# Goal

Make the research activity data (RAD) schema a standards-aligned activity model with explicit, testable interchange profiles. Use RAiD for activity-level semantics, DataCite for research outputs and DOI exchange, DCAT for dataset distributions, established persistent-identifier schemes for identity, and CRediT for output contributor roles.

Preserve RAD's concise authoring fields, including `title`, `subtitle`, and `sections`, rather than reshaping the document into a direct copy of any deposit or registry schema.

# Current evidence

- `crates/acorn-schema/src/research_activity/mod.rs` retains the concise identifier collections and primary `contact` while adding canonical `typedDates`, `contributors`, `organizations`, `funders`, `relatedResources`, `languages`, `subjects`, `spatialCoverage`, `rights`, and `access` fields. Compatibility aliases normalize legacy `sponsors` and `related` input into structured fields; precedence and standards-adapter behavior still need Slice 5 coverage.
- `crates/acorn-schema/src/research_activity/output.rs` now provides controlled DataCite contributor types, CRediT roles, personal and organizational contributors, general funder registry identifiers, and an output-specific `validate_deposit_ready()` gate. `crates/acorn-schema/src/standard/crosswalk/research_output.rs` implements warning-aware `ResearchOutput <-> DataCite Record` conversion across the supported 4.7 properties, including RAiD/RRID/SWHID relations, `Other` relation information, organizational creators, grant DOI normalization, and structured warnings for every modeled local-only or simplified value. Reconciled 2026-09-16; the focused research-output and 30-test crosswalk suites, full 361-test schema suite, and full 1,729-test repository suite pass (Slice 1).
- `crates/acorn-lib/src/io/enrichment.rs` reuses `Enrich`, `EnrichmentMetadata`, and `Pipeline` for best-effort ORCID enrichment of RAD contacts, activity contributors, and output people plus CFF, DataCite JSON, and Invenio person records. `acorn format --standard ... --enrich --provider orcid` dispatches through the existing generic workflow, requires a unique normalized structured-name, exact given-name component plus family name, alternate display-name, or exact-email match, preserves populated values, and reports lookup failures plus ambiguous or absent compatible matches per person. Contact lookup first tries exact email with the person's name, falls back to name-only discovery, and retries every unresolved name result with an `email:*@domain` constraint. If that does not resolve the person, affiliations are matched by name or alias against the embedded `organization.json` hierarchy; the root parent's name is used as an institution discriminator and its ROR is combined with the person's name for a final lookup. Empty or ambiguous refinements retain the stronger prior response, and explicit identifiers resolve directly. Normalized full emails keep otherwise identical people in separate requests. Website URLs are not followed; site-specific resolution remains outside the provider-neutral core for a future extension or configurable allowlist. Reconciled 2026-09-16; the focused 15-test enrichment suite and full 1,726-test repository suite pass.
- `crates/acorn-lib/src/io/workflow/mod.rs` plans enriched source files concurrently up to the existing `CommonRuntime::threads` limit while retaining source order; zero is normalized to one. Provider stages, per-document ORCID requests, hook evaluation, and transactional application remain sequential. Reconciled 2026-09-16; the focused 15-test workflow suite and 10-test format-command suite pass.
- `crates/acorn-schema/src/standard/datacite.rs:1` now models DataCite Metadata Schema 4.7 in JSON (`Record`/`Attributes` `datacite.rs:515`) and XML (`Resource` `datacite.rs:1128`) forms with `RelatedIdentifierType::Raid`/`Swhid` `datacite.rs:168`, `RelationType::Other` `datacite.rs:237`, `ResourceTypeGeneral::Poster`/`Presentation` `datacite.rs:321`, `RelatedItem`/`RelatedItems` and `relationTypeInformation` on `RelatedIdentifier` `datacite.rs:968` + `RelatedItem` `datacite.rs:1080`, and `validate_deposit_ready()` gates `datacite.rs:1525`. The two forms expose symmetric 4.7 coverage; mandatory JSON properties remain optional on `Attributes` with deposit readiness enforced via `validate_deposit_ready()` rather than `Validate`. Reconciled 2026-09-03 against `acorn-schema` at `0.1.76` (Slice 0).
- `crates/acorn-schema/src/standard/crosswalk/` supports warning-aware `ResearchOutput <-> DataCite Record` conversion in addition to DataCite, DCAT, HuWise, and Invenio. Whole-activity RAD/RAiD and DataCite Project adapters remain in Slice 2. `Standard::crosswalk` dispatch lives in `crates/acorn-lib/src/analyzer/check.rs:1044` (analysis feature) and will migrate with the crosswalk registry.
- `crates/acorn-schema/src/standard/crosswalk/mod.rs:238` `convert` now propagates `FieldMapping::apply` `mapping.rs:194` missing list into `ConversionWarning::omitted` (`from`/`to`/`field` with details) and `crates/acorn-schema/src/standard/crosswalk/mapping.rs:194` `FieldMapping::apply` now propagates `MissingRequiredField`/`TransformationFailed` instead of swallowing. `crates/acorn-lib/src/analyzer/check.rs:1099` `collect_crosswalk_warnings` now propagates `Err` instead of `Err(_) => Vec::new()`, and `crates/acorn-schema/src/standard/crosswalk/mapping.rs:482` `resource_type_to_iri`/`iri_to_resource_type` handle `Poster`/`Presentation`. Reconciled 2026-09-03 (Slice 0).
- `crates/acorn-schema/src/pid/raid/mod.rs` is the portable RAiD schema; RAD-to-RAiD assembly and inference historically lived in the adapter layer (now `crates/acorn-lib/src/io/adapter/`). When present, it has supplied current-date, principal-investigator/lead-organization roles, owner/registration-agency, and CC0 license defaults when RAD does not assert them, and leaves related objects, subjects, and spatial coverage empty. This plan makes inference opt-in and loss-explicit.
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

DataCite 4.7 delta watchlist: `ResourceTypeGeneral::Poster`, `Presentation`; `RelatedIdentifierType::RAiD`, `SWHID`; `RelationType::Other`; `RelatedIdentifier.relationTypeInformation`; `RelatedItem` container. All doc links in `crates/acorn-schema/src/standard/datacite.rs` must point to `4.7` after the audit.

# Recommendation

Treat RAD as the canonical activity-focused model and publish explicit profiles at its boundaries:

- Map `ResearchActivity` to and from RAiD metadata where the source contains sufficient assertions.
- Map `ResearchOutput` to and from DataCite records, with a separate deposit-readiness validator for DataCite mandatory properties (distinct from `Validate` on `Attributes`/`Resource`).
- Permit an explicit `ResearchActivity -> DataCite Project` export for users who want to register an activity DOI, but do not require every RAD document to satisfy DataCite deposit requirements.
- Keep the manually authored, discovery-ready `logbook` canonical in RAD. Do not infer DataCite `Updated` from entry occurrence timestamps; Project export may map an explicitly selected aggregate description, but it must report entry details as local-only/lossy rather than flatten the logbook into DataCite.
- Map dataset outputs to DCAT and software outputs to CodeMeta without adding standard-specific fields to unrelated output types.
- Derive Schema.org JSON-LD from canonical RAD values instead of maintaining a parallel discovery record.
- Reject missing target-required values or report them as structured conversion losses. Do not fabricate dates, roles, identifiers, organizations, licenses, or publishers.

Keep the first implementation additive. Do not make schema versioning or a general migration framework a prerequisite for correcting crosswalks, adding optional structured fields, or validating standards conformance.

This plan is a coordination record for the full alignment. Remaining implementation proceeds in gated slices. Broad `crates/acorn-schema/src/` directory ownership is a planning boundary, not an atomic write set — each slice must declare exact files.

# Implementation plan

The remaining slices cover activity-level standards adapters and compatibility. Each slice declares exact files; directory ownership above is a planning boundary, not a concurrency claim.

## 1. Complete the output-level DataCite profile

**Implemented 2026-09-16:** Exact write set: `research_activity/{common.rs,output.rs,tests/mod.rs}` and `standard/crosswalk/{mod.rs,research_output.rs,tests/mod.rs}`, plus this plan and its field-matrix/index/network records. The additive model, warning-aware Record adapter, and output-specific deposit gate are covered by focused round-trip and missing-mandatory-property tests.

Extend `ResearchOutput` `output.rs:113` with:

- creators distinct from other contributors
- controlled contributor roles, using CRediT where applicable (distinct from `ContributorType` `datacite.rs:35`)
- personal and organizational contributors
- publisher with optional identifier (`publisherIdentifier` parity with `datacite.rs:898`)
- publication year and typed dates (`Date` `datacite.rs:654` with `dateType`)
- controlled general resource type plus a specific type (`ResourceTypes` `datacite.rs:1038`)
- language, alternate identifiers, typed related identifiers (including `RAiD`/`RRID`/`SWHID`), `RelatedItem`s, and version
- structured rights, descriptions, subjects, formats, sizes, and geographic coverage
- complete funder and award identifiers

The DataCite audit is complete; this slice implements the `ResearchOutput` side and the deposit-readiness validator. Enforce mandatory DataCite properties only in `validate_deposit_ready` without making them mandatory for partially enriched RAD output records.

Delivered: a `ResearchOutput` round-trips every field ACORN classifies as exact or reversible, reports local-only and simplified fields, and explains every missing DataCite mandatory property without making partial RAD output metadata invalid.

## 2. Implement standards adapters and user workflows

- Implement `ResearchActivity <-> RAiD Metadata` with no inferred assertions; remove or gate the legacy inferred date/role/license defaults behind an explicit `allow_infer: bool` that defaults to `false` and emits `ConversionWarning::Approximated` when used.
- Implement `ResearchOutput <-> DataCite Record` with explicit loss reporting (reuses the established warning pipeline), preserving authored RRID identifiers, explicit relations, relation information, and general resource types without inferred semantics.
- Implement opt-in `ResearchActivity -> DataCite Project` export (general `ResourceTypeGeneral::Project` `datacite.rs:407`), preserving authored RRID relations without inferring relations from discovery or co-occurrence.
- Implement dataset-output DCAT and software-output CodeMeta adapters only after their mappings are approved in the matrix.
- Generate Schema.org JSON-LD contexts and types from the canonical activity and output structures.
- Expose validation and conversion through existing `cargo run --bin acorn -- check` / `export` / `schema` workflows instead of adding a new command unless existing command semantics cannot express the operation.
- Honor offline mode and use local fixtures for tests.

Deliverable: users can inspect conformance, conversion failures, and conversion losses before writing output. No network request occurs during `cargo nextest run -p acorn-schema`.

## 3. Address compatibility and versioning only when required

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
cargo test -p acorn-lib io::tests::enrichment --lib
cargo test -p acorn-lib io::workflow --lib
cargo test -p acorn-cli commands::format
make check-wasm
make lint
make test
make audit # if deps change
```

Additional checks:

- Validate pinned DataCite 4.7 JSON and XML fixtures and two invalid records (`datacite-invalid-*.json`); retain the existing `rg "4\.6" crates/acorn-schema/src/standard/datacite.rs` gate.
- Validate official RAiD examples and missing-required-field cases.
- Add golden tests for `RAD <-> RAiD`, `ResearchActivity -> DataCite Project`, and `ResearchOutput <-> DataCite Record` conversions, including activity- and output-level RRID relations plus 4.7 `RAiD`/`SWHID`/`Other`/`RelatedItem` cases.
- Add property tests for identifier normalization (DOI, RAiD, SWHID, ROR, ORCID, ISNI), relation mappings, and lossless round trips.
- Assert that conversions never fabricate metadata and that every intentional loss emits a `ConversionWarning` with `from`/`to`/`field`/`loss_type`/`details` and that required-field failures are `CrosswalkError::MissingRequiredField`, not warnings.
- Retain compatibility fixtures for established RAD field names and Markdown front matter (`crates/acorn-schema/src/research_activity/tests/`).
- Snapshot generated RAD, RAiD, DataCite, and JSON-LD schemas or representations where stable output is part of the contract (`insta` snapshots in `crates/acorn-schema/src/standard/tests/snapshots/`).
- Run focused crosswalk/CLI tests after adapter slices: `cargo nextest run -p acorn-cli --lib crosswalk` (or `check`/`export` consumers).
- `cargo fmt --all` before every PR.

# Acceptance criteria

- Every canonical RAD field has a documented standards mapping or an explicit local-only designation in the pinned matrix.
- RAiD export contains no dates, roles, organizations, identifiers, or licenses absent from the RAD source unless `allow_infer: true` is explicitly set, in which case a warning is emitted.
- DataCite validation distinguishes valid internal output metadata (lenient `Validate`) from deposit-ready metadata (`validate_deposit_ready`) and reports every missing mandatory property (`titles`, `creators`, `publisher`, `publicationYear`, `resourceType`).
- DataCite adapters preserve authored RRID identifiers and explicit relation semantics from activity `relatedResources` and output `relatedIdentifiers`, or emit a structured warning for every simplification or omission.
- `ResearchOutput` supports all DataCite 4.7 properties ACORN documents as supported, or the unsupported property is identified as `unsupported` in the mapping matrix. Specifically, `RelatedIdentifierType` includes `RAiD`/`RRID`/`SWHID`, authored RRID relations retain their explicit relation semantics, `RelationType` includes `Other`, `ResourceTypeGeneral` includes `Poster`/`Presentation`, and `RelatedItem` is modeled after the 4.7 schema.
- `crates/acorn-schema/src/standard/datacite.rs` contains no `4.6` doc links; all links point to `4.7`.
- JSON (`Record`/`Attributes`) and XML (`Resource`) DataCite forms have symmetric coverage for spec-defined parity; intentional asymmetry is documented.
- Logbook entries remain losslessly represented in RAD; every value omitted from RAiD or DataCite projection is classified and reported as a warning with source/target paths.
- Every omitted, defaulted, coerced, unsupported, or many-to-one field produces a structured `ConversionWarning`; lossy-but-successful conversions never return an empty warning list when `FieldMapping::apply` reports missing fields.
- Existing RAD fixtures continue to deserialize throughout the additive implementation phases.
- ORCID contact enrichment tries exact email with the person's name, falls back to name-only discovery, retries every unresolved search with a normalized contact email-domain filter, and then retries with the root-parent ROR resolved from an embedded organization-hierarchy affiliation name or alias. Root organization names participate in institution disambiguation, exact alternate display names and exact given-name components are accepted, absent compatible matches are reported, empty refinements retain the stronger prior response, and contacts with different full email addresses are not deduplicated. Website URLs are not used as identity evidence.
- Enriched workflow planning honors the global thread limit across files, preserves source order regardless of completion order, treats zero as a concurrency limit of one, and leaves provider ordering and transactional writes unchanged.
- Lossless round-trip tests retain all fields classified as exact or reversible transformations.
- Generated JSON Schema exposes controlled values, nested validation, and conditional requirements where supported; `RelatedItem`/`RAiD`/`RRID`/`SWHID` appear in schema output.
- Default, minimal (`--no-default-features`), `alloc`, and WASM builds remain supported; `make check-wasm` passes per slice.
- No network request occurs during schema validation or fixture-backed tests; `offline` mode respected in CLI consumers.

# Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| A union of standards makes RAD too complex to author. | High | Keep concise RAD authoring fields and isolate standard-specific shapes behind adapters. |
| Similar fields become competing sources of truth. | High | Approve canonical ownership in the mapping matrix and define conflict precedence before adding replacements. Document precedence for `meta.sponsors` vs structured funding. |
| Conversions silently discard metadata. | High | Preserve the landed warning-propagation gate and test every lossy rule with golden warnings. |
| RAiD or DataCite requirements change. | Medium | Pin profile versions (`kernel-4.7` XSD, RAiD 1.x fixtures), keep controlled vocabularies explicit, and isolate adapters from canonical types. |
| Deposit requirements make partially enriched records invalid. | Medium | Separate RAD validity (`Validate`) from target-specific readiness (`validate_deposit_ready`). |
| Compatibility work expands into an early redesign. | Medium | Prefer additive fields and defer schema versioning until a demonstrated breaking change requires it. |
| 4.7 audit scope creep (full re-derive without fixtures) | Medium | Pin two valid + two invalid 4.7 fixtures first; diff types against them; gate with `rg "4\.6"` empty check. |
| Primitives duplicate existing `ResearchOutput::Affiliation`/`Funding`/`Award` | Medium | Inventory `output.rs:35,50,94` before adding types; extend existing where semantics match. |
| Crosswalk warning fix breaks existing `TryFrom` callers that ignore warnings | Medium | Keep `TryFrom` signature infallible at type level; add warning-aware path separately (`crosswalk_with_warnings`); do not change `TryFrom` to return warnings. |
| Schema-features lane contention on `ResearchActivity` | High | Serialize `ResearchActivity`/`ResearchOutput` edits via the schema-features lane lock `parallel-execution.md`; each slice declares exact files and integration owner. |

# Related concepts

- SWHID persistent-identifier hardening
- [OCP infrastructure schema alignment](ocp-infrastructure-schema-alignment.md)
- [CKAN/DCAT crosswalk](../migrations/ckan-dcat-crosswalk.md)
- [Analyzer composition and traversal](../refactors/analyzer/analyzer-composition.md)
- [Lifecycle, credit, license, and award fields](../refactors/core/research-activity-lifecycle-credit-fields.md)
