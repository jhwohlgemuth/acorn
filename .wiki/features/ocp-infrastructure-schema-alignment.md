---
type: Plan
title: "Align infrastructure metadata with Open Compute Project records"
description: "Add portable infrastructure and OCP schema models while converting existing standard modules to folder format and using the consolidated portable validation contract."
difficulty: high
impact: high
package: [acorn-lib, acorn-schema]
tags: [feature, schema, infrastructure, hardware, open-compute-project, ocp, redfish, standards, validation]
status: planned
priority: next
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/schema/hardware/`, `src/schema/research_activity/mod.rs`, portable validation and adjacent schema tests/fixtures; target `acorn-schema` after extraction.
- **Create:** OCP/standard claim modules and fixtures under the schema boundary described by this plan.
- **Delete:** superseded single-file standard modules only when folder conversions preserve public re-exports; no schema field is deleted.

# Dependencies

Land the portable `acorn-schema` boundary before broad module moves. The consolidated `schema::validate` folder module and portable rule primitives are already implemented; this plan consumes them without repeating the consolidation. Coordinate shared RAD fields with [metadata standards alignment](rad-metadata-standards-alignment.md).

# Goal

Represent how a research activity requires and uses compute infrastructure, and connect that infrastructure to authoritative Open Compute Project (OCP) contributions, recognition records, and management profiles without turning ACORN into a hardware controller or certification authority.

The implementation must introduce exactly two new logical Rust modules:

- `acorn::schema::infrastructure`
- `acorn::schema::standard::ocp`

Existing standard modules may move to folder-format entry points without changing their Rust paths. Generic standard-reference types remain in the existing `acorn::schema::standard` root. Schema validation is already consolidated in `acorn::schema::validate`. Sustainability data models are outside this plan.

# Current evidence

- `ResearchActivityMetadata::resources` describes minimum hardware or compute requirements and stores `Option<Vec<Resource>>` in `crates/acorn-lib/src/schema/research_activity/mod.rs`.
- `Resource` already models ASIC, CPU, DPU, DSP, FPGA, GPU, NPU, neuromorphic, quantum, sensor, and TPU requirements in `crates/acorn-lib/src/schema/hardware/mod.rs`.
- `Resource` currently derives only `Clone, Debug, Serialize, JsonSchema` and does not implement `Validate`; no `#[validate(...)]` attribute appears within `schema::hardware` (including `memory.rs`, `vendor.rs`, and the `quantum` submodule), and `ResearchActivityMetadata::resources` carries only `#[eserde(compat)]` without `#[validate(nested)]`. Resource requirements are therefore not validated through a complete RAD value today; this plan must add the `Validate` derive to `Resource` and its nested hardware types and request nested validation on the metadata field.
- `schema::standard` exposes CFF, crosswalk, DataCite, DCAT, Dublin Core, HuWise, InvenioRDM, and text models. `dcat` and `crosswalk` are already folder-format modules; `cff`, `datacite`, `dublin_core`, `huwise`, `invenio`, and `text` remain root `.rs` entry files.
- `schema::validate` is already a folder-format module (`crates/acorn-lib/src/schema/validate/mod.rs`, exported as `pub mod validate` from `crates/acorn-lib/src/schema/mod.rs`) that co-locates general and standard-specific validators, importing `standard::datacite::GeoLocationPoint` and other DataCite types plus `standard::invenio::PersonOrOrgIdentifier` and geonames helpers. No `standard::validate` module exists; the only remaining internal helper is `standard::dcat::validate` (`pub(crate)` in `crates/acorn-lib/src/schema/standard/dcat/mod.rs`), which is scoped to DCAT views and separate from the consolidated contract.
- No `schema::infrastructure` or `schema::standard::ocp` module currently exists.
- The planned crate boundary assigns standards, hardware data models, validation, and crosswalks to portable `acorn-schema`; network resolution and provider DTOs remain adapter concerns ([crate boundary architecture](../refactors/crate-boundary-architecture.md)).
- OCP applies open collaboration to scalable compute infrastructure and organizes contributions around efficiency, impact, openness, scalability, and sustainability, but OCP describes itself as a community that develops technology norms rather than a conventional standards body ([OCP history, mission, and tenets](https://www.opencompute.org/about), [OCP home](https://www.opencompute.org/)).
- The OCP contribution process distinguishes specifications, design files, embedded software, reference architectures, tested configurations, requirements documents, and other contribution forms; accepted records are published in the OCP Contributions Database ([OCP contribution process](https://www.opencompute.org/contributions/welcome), [OCP Contributions Database](https://www.opencompute.org/contributions)).
- OCP hardware-management profiles build on DMTF Redfish and publish profile definitions separately from validator execution, so ACORN must preserve the distinction between a referenced profile and evidence that a product passed a conformance test ([OCP Profiles](https://github.com/opencomputeproject/OCP-Profiles), [DMTF Redfish Interop Validator](https://github.com/DMTF/Redfish-Interop-Validator)).

# Direction decisions

## Preserve requirement semantics

Keep `ResearchActivityMetadata::resources` as the minimum compute requirement. Do not add platform, facility, OCP recognition, or management-profile fields to individual `Resource` variants.

Represent an actual configured system separately as infrastructure. A GPU remains a hardware resource; a server containing GPUs, carrying standards claims, and associated with an OCP record is a compute platform.

## Separate claims from recognition

A standards claim describes a relationship such as `conformsTo`, `implements`, or `testedAgainst`. OCP recognition is an externally granted fact tied to an authoritative record. Manufacturer, architecture, model name, use of an OCP specification, or an image containing an OCP badge must never cause ACORN to infer recognition.

## Keep the portable schema data-only

Schema types parse, serialize, validate, and crosswalk immutable data. OCP catalog access, Redfish requests, caching, authentication, and offline behavior belong to future API or host adapters. Management endpoints, BMC credentials, tokens, and control operations are not represented in public RAD.

## Enforce the module boundary literally

All initial infrastructure types live directly in `schema/infrastructure/mod.rs`. All initial OCP types live directly in `schema/standard/ocp/mod.rs`. Do not create `standard::reference`, `infrastructure::platform`, `infrastructure::facility`, `infrastructure::environment`, `ocp::contribution`, `ocp::recognition`, `ocp::profile`, or `ocp::crosswalk` modules in this plan.

# Target layout

```text
schema/
├── infrastructure/
│   └── mod.rs
├── validate/
│   └── mod.rs
└── standard/
    ├── mod.rs
    ├── cff/
    │   └── mod.rs
    ├── crosswalk/
    │   └── mod.rs
    ├── datacite/
    │   └── mod.rs
    ├── dcat/
    │   └── mod.rs
    ├── dublin_core/
    │   └── mod.rs
    ├── huwise/
    │   └── mod.rs
    ├── invenio/
    │   └── mod.rs
    ├── ocp/
    │   └── mod.rs
    ├── tests/
    │   └── mod.rs
    └── text/
        └── mod.rs
```

Only `infrastructure` and `standard::ocp` are new logical modules. `validate` and the converted standard paths already exist as Rust modules.

# Schema design

## Generic standard claims

Add the following public types directly to the existing `schema::standard` root:

- `StandardReference`: authority, identifier, optional title, optional version, and canonical URI.
- `StandardsClaim`: standard reference, relationship, assessment, and optional evidence URI.
- `StandardsRelationship`: `ConformsTo`, `Implements`, `TestedAgainst`, `BasedOn`, and `References`.
- `ClaimAssessment`: `Declared`, `Inferred`, and `Verified`.

Require an absolute HTTP or HTTPS URI, non-empty authority and identifier, and evidence for a verified claim. Reject duplicate claims with the same authority, identifier, version, and relationship.

## OCP records

Define the following directly in `schema::standard::ocp`:

- `OcpContributionId`: validated `S` followed by one or more digits without assuming a fixed width.
- `OcpProject`: an open newtype rather than an enum because OCP projects and workstreams evolve.
- `OcpContributionKind`: specification, design files, embedded software, reference architecture, tested configuration, requirements document, white paper, and an open fallback.
- `OcpContribution`: identifier, title, project, contribution kind, optional version, optional status, canonical URI, and optional license.
- `OcpRecognitionProgram`: Accepted, Inspired, Ready, S.A.F.E., Integrated Solution, and an open fallback.
- `OcpRecognition`: program, subject identifier, authoritative source URI, and optional award and expiration dates.
- `ManagementProfileClaim`: OCP profile reference, underlying standard reference, optional validator identity and version, optional test date, conformance result, and optional report URI.
- `ConformanceResult`: pass, fail, partial, and not tested.

An OCP Ready recognition is valid only for a facility. Platform recognition may use the product-oriented programs supported by the model. A passing management-profile claim requires evidence from a report or authoritative OCP product record.

## Infrastructure

Define the following directly in `schema::infrastructure`:

- `ComputePlatform`: identifier, name, optional manufacturer and model, contained `Resource` values, standards claims, OCP recognitions, and management-profile claims.
- `ComputeFacility`: identifier, name, optional operator and appropriately coarse location, standards claims, and OCP recognitions.
- `ComputeEnvironment`: platform, optional facility, resources actually present, and optional observation timestamp.

Validate nested resources and claims. Restrict OCP Ready recognition to `ComputeFacility`, reject OCP Ready on `ComputePlatform`, and do not infer claims from vendor or architecture fields.

# Implementation plan

## 1. Establish a behavioral baseline

1. Run focused schema and standard tests before moving files.
2. Record representative serialization snapshots and public import paths for CFF, DataCite, DCAT, Dublin Core, HuWise, InvenioRDM, text, crosswalk, and validation.
3. Confirm default, no-default-features, alloc, and WASM schema configurations before structural changes.

## 2. Consume the consolidated validation module

The folder-format `schema::validate` consolidation has landed. This plan relies on it without repeating the move:

1. The consolidated `schema::validate` folder module is already present with general and standard validators together (see Current evidence); re-confirm it before moving files.
2. Reference the portable rule primitives (`nonempty`, URI checks) where new claim fields need them.
3. Do not re-perform the move or re-create `schema/standard/validate.rs`.
4. Preserve existing validation behavior and error codes from the consolidated module.

## 3. Convert existing standard modules to folder format

Move entry files without changing logical module paths:

| Existing entry | Folder-format entry | Rust path |
|---|---|---|
| `standard/cff.rs` | `standard/cff/mod.rs` | `schema::standard::cff` |
| `standard/datacite.rs` | `standard/datacite/mod.rs` | `schema::standard::datacite` |
| `standard/dublin_core.rs` | `standard/dublin_core/mod.rs` | `schema::standard::dublin_core` |
| `standard/huwise.rs` | `standard/huwise/mod.rs` | `schema::standard::huwise` |
| `standard/invenio.rs` | `standard/invenio/mod.rs` | `schema::standard::invenio` |
| `standard/text.rs` | `standard/text/mod.rs` | `schema::standard::text` |

Leave `standard/dcat`, `standard/crosswalk`, and `standard/tests` in their existing folder layouts. Move one entry at a time and run focused checks between moves so path or relative-import regressions remain attributable.

## 4. Add generic standard claims and OCP types

1. Add generic reference and claim types to `schema::standard::mod.rs` with portable validation, serde, and JSON Schema support.
2. Create `schema/standard/ocp/mod.rs` and add contribution, recognition, and management-profile types.
3. Add pure conversions from `OcpContribution` to generic standard references or claims in `ocp/mod.rs` or the existing `standard::crosswalk::mapping` module; do not create another OCP submodule.
4. Add focused tests for identifiers, open fallback values, evidence rules, recognition source URIs, and management-profile results.

## 5. Add infrastructure and integrate RAD

1. Create `schema/infrastructure/mod.rs` with platform, facility, and compute-environment types.
2. Export the module and its primary types from the existing schema facade as appropriate.
3. Derive `Validate` on `Resource` and its nested hardware types (none currently derives it), keep `ResearchActivityMetadata::resources` unchanged in representation and meaning, and add `#[validate(nested)]` so resource validation executes through a complete RAD value.
4. Add an optional `infrastructure: Option<Vec<ComputeEnvironment>>` field to research-activity metadata.
5. Add the corresponding optional linked-data context term using established predicates such as `dcterms:conformsTo`, `dcterms:identifier`, `dcterms:source`, `dcterms:hasVersion`, `schema:manufacturer`, and `schema:location` where their semantics fit.
6. Do not invent an OCP RDF namespace; use canonical OCP record URLs as identifiers.

## 6. Preserve compatibility and document the boundary

1. Keep every migrated standard module's Rust path and existing public items source-compatible.
2. Keep existing RAD documents valid because infrastructure is optional and the resource representation is unchanged.
3. Document the distinction between required resources, an actual environment, a standards claim, an OCP recognition, and a conformance-test result.
4. Document that schema models do not resolve OCP records, contact Redfish services, operate hardware, or grant OCP recognition.

# Validation

Run focused checks after each implementation stage, followed by repository gates:

```sh
cargo nextest run --package acorn-lib schema::standard
cargo nextest run --package acorn-lib schema::hardware
cargo nextest run --package acorn-lib schema::research_activity
cargo check -p acorn-lib --no-default-features
cargo check -p acorn-lib --no-default-features --features alloc
make check-wasm
make lint
make test
make book
```

Also verify:

- Existing public imports compile without changes after each folder migration.
- No paired `standard/foo.rs` and `standard/foo/mod.rs` entry files remain.
- `standard/validate.rs` and the `standard::validate` declaration are absent.
- `schema/validate/mod.rs` preserves existing validator behavior, feature gates, and error codes.
- Existing standard and RAD serialization snapshots remain unchanged except for explicitly approved optional-field output.
- JSON and YAML round trips cover all new types.
- Unknown fields are rejected where the external record is modeled as a closed contract; open OCP categories retain explicit fallback variants.
- Complete RAD validation rejects an invalid nested `Resource`.
- A verified claim without evidence fails validation.
- OCP Ready attached to a platform fails validation, while an evidence-backed facility recognition succeeds.
- A passing management-profile claim without a report or authoritative record fails validation.
- Linked-data output resolves standards relationships to canonical predicates and official OCP URLs.
- `rg` finds no sustainability schema fields, types, crosswalks, or implementation steps introduced by this plan.

# Acceptance criteria

1. The only new logical Rust modules are `acorn::schema::infrastructure` and `acorn::schema::standard::ocp`.
2. `schema::validate` is a folder-format module containing both general and standard-specific schema validators; `schema::standard::validate` no longer exists.
3. CFF, DataCite, Dublin Core, HuWise, InvenioRDM, and text use folder-format entry points while retaining their existing Rust paths and public behavior.
4. Generic standard claims preserve authority, identifier, version, relationship, assessment, canonical source, and verification evidence.
5. OCP contributions, recognition programs, and management-profile results are distinct typed concepts with authoritative evidence.
6. A compute requirement remains distinct from the platform and facility actually used by a research activity.
7. Existing RAD documents and standard imports remain compatible; new infrastructure metadata is optional.
8. Nested resource validation runs through complete RAD validation.
9. No ACORN behavior infers OCP recognition from a vendor, architecture, product name, standards claim, or badge image.
10. Portable schema code performs no network, filesystem, credential, hardware-control, or live Redfish operations.
11. Default, minimal, alloc, and WASM checks plus full lint, test, and book gates pass.
12. No sustainability data model or crosswalk is added by this plan.

# Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Folder moves unintentionally change public paths or relative imports. | High | Move one entry at a time and compile existing import sites after each move. |
| Consolidated validation broadens internal visibility. | Medium | Use `pub(crate)` for relocated standard-specific helpers. |
| Generic claims are mistaken for official recognition. | High | Keep claim assessment and OCP recognition as separate types with distinct validation. |
| OCP projects or contribution categories evolve. | Medium | Use open newtypes and explicit fallback variants instead of closed project enums. |
| Detailed infrastructure exposes operationally sensitive information. | High | Exclude credentials, management endpoints, hostnames, serial numbers, and fine-grained facility locations from the portable public model. |
| OCP catalog or Redfish access leaks into portable schemas. | High | Restrict this plan to immutable data models and pure conversions; place future transport in adapters. |

# Related concepts

- [Crate boundary architecture](../refactors/crate-boundary-architecture.md)
- [CKAN package to DCAT crosswalk](../migrations/ckan-dcat-crosswalk.md)
- [SWHID persistent-identifier hardening](swhid-persistent-identifier.md)
