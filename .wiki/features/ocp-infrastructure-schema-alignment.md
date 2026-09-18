---
type: Plan
title: "Align infrastructure metadata with Open Compute Project records"
description: "Add portable infrastructure and OCP record models on the current one-or-many RAD, validation, and loss-aware standards contracts."
difficulty: high
impact: high
package: [acorn-schema]
tags: [feature, schema, infrastructure, hardware, open-compute-project, ocp, redfish, standards, validation]
status: planned
priority: next
---

# Project code boundaries

- **Update:** `crates/acorn-schema/src/lib.rs`, `crates/acorn-schema/src/research_activity/{mod.rs,tests/mod.rs}`, `crates/acorn-schema/src/standard/{mod.rs,tests/mod.rs}`, `crates/acorn-schema/src/standard/crosswalk/research_activity/` for explicit local-only loss reporting, `crates/acorn-schema/src/tests/mod.rs`, and `.wiki/assets/standards-field-matrix.md`.
- **Create:** `crates/acorn-schema/src/infrastructure.rs`, `crates/acorn-schema/src/standard/ocp.rs`, `crates/acorn-schema/src/tests/infrastructure.rs`, `crates/acorn-schema/src/standard/tests/ocp.rs`, and OCP fixtures under `tests/fixtures/schema/ocp/`.
- **Delete:** None. Existing standard and hardware modules remain in place.

# Dependencies

The portable `acorn-schema` boundary, consolidated `acorn_schema::validation` module, generic `OneOrMany<T>: Validate` traversal, and the loss-aware RAD crosswalk contract recorded in the [standards field matrix](../assets/standards-field-matrix.md) are implemented. This plan consumes them without reopening validation-macro or crosswalk architecture.

# Goal

Represent how a research activity requires and uses compute infrastructure, and connect that infrastructure to authoritative Open Compute Project (OCP) contributions, recognition records, and management profiles without turning ACORN into a hardware controller or certification authority.

The implementation must introduce exactly two new logical Rust modules:

- `acorn_schema::infrastructure`
- `acorn_schema::standard::ocp`

Generic standard-reference types remain in the existing `acorn_schema::standard` root. Schema validation is already consolidated in `acorn_schema::validation`. Existing standard and hardware modules do not move. Sustainability data models are outside this plan.

# Current evidence

- `ResearchActivityMetadata::resources` describes minimum hardware or compute requirements and stores `Option<OneOrMany<Resource>>` in `crates/acorn-schema/src/research_activity/mod.rs`. It uses the shared one-or-many deserializer and `#[builder(into)]`, but it does not request nested validation.
- The landed metadata-alignment fields (`outputs`, typed dates, contributors, organizations, funders, related resources, languages, subjects, spatial coverage, and rights) establish the current structured-field pattern: `Option<OneOrMany<T>>`, `#[builder(into)]`, nested validation where `T: Validate`, one-or-many deserialization, and compatibility defaults for new linked-data context terms. Infrastructure must follow that pattern rather than introduce `Option<Vec<T>>`.
- `Resource` already models ASIC, CPU, DPU, DSP, FPGA, GPU, NPU, neuromorphic, quantum, sensor, and TPU requirements in `crates/acorn-schema/src/hardware/mod.rs`.
- `Resource` derives `Clone, Debug, Serialize, JsonSchema` and has a manual `Validate` implementation that delegates to `hardware::vendor::validate_vendor`. Direct resource validation therefore rejects known architecture/vendor mismatches. `OneOrMany<T>` also implements `Validate` and prefixes item failures with `[index]`; adding `#[validate(nested)]` to the metadata field is sufficient to carry the existing resource contract through full RAD validation without replacing it with a derive.
- Hardware serialization and vendor tests exist in `crates/acorn-schema/src/hardware/tests/mod.rs`, but `hardware/mod.rs` currently declares that module behind `#[cfg(any())]`, so those tests are dormant. Active research-activity tests cover resource deserialization but do not prove that a nested resource failure is reported from `ResearchActivity::validate`.
- `acorn_schema::standard` exposes CFF, crosswalk, DataCite, DCAT, Dublin Core, HuWise, InvenioRDM, and text models. Leaf modules such as `cff`, `datacite`, `dublin_core`, `huwise`, `invenio`, and `text` use root `.rs` entry files; only modules with child leaves use directories. Initial OCP types need no child module, so `standard/ocp.rs` matches the current layout without moving unrelated files.
- `acorn_schema::validation` is a folder-format module (`crates/acorn-schema/src/validation/mod.rs`, exported as `pub mod validation` from `crates/acorn-schema/src/lib.rs`) with `nested`, `each`, `not`, `optional`, and `or` combinators plus portable `nonempty`, `timestamp`, and `url` rules. The built-in `url` rule accepts any absolute URI scheme, so the OCP HTTP(S)-only requirement needs a small schema-local rule rather than a change to the generic validator. No `standard::validation` module exists; `standard::dcat::validate` remains a DCAT-scoped internal helper.
- No `acorn_schema::infrastructure` or `acorn_schema::standard::ocp` module currently exists.
- The implemented `ResearchActivity <-> RAiD Metadata`, `ResearchActivity -> DataCite Project`, and structured conversion-loss contracts make new RAD fields explicit crosswalk concerns. Infrastructure has no approved RAiD or DataCite projection, so the field matrix and export warnings must classify it as local-only instead of silently dropping it.
- The planned crate boundary assigns standards, hardware data models, validation, and crosswalks to portable `acorn-schema`; network resolution and provider DTOs remain adapter concerns ([crate boundary architecture](../refactors/crate-boundary-architecture.md)).
- OCP applies open collaboration to scalable compute infrastructure and organizes contributions around efficiency, impact, openness, scalability, and sustainability, but OCP describes itself as a community that develops technology norms rather than a conventional standards body ([OCP history, mission, and tenets](https://www.opencompute.org/about), [OCP home](https://www.opencompute.org/)).
- The OCP contribution process distinguishes specifications, design files, software, reference architectures, tested configurations, requirements documents, and other contribution forms; accepted records are published in the OCP Contributions Database ([OCP contribution taxonomy](https://www.opencompute.org/contributions/welcome), [current contribution process](https://www.opencompute.org/contributions/how-to-contribute), [OCP Contributions Database](https://www.opencompute.org/contributions)).
- OCP distinguishes product/device recognition (`Accepted`, `Inspired`, and `S.A.F.E.`), facility recognition (`Ready`), and Foundation-approved integrated solutions. These scopes must determine which infrastructure type may carry each recognition ([OCP products and recognition](https://www.opencompute.org/products), [OCP S.A.F.E. program](https://www.opencompute.org/community/ocp-safe-program)).
- OCP hardware-management profiles build on DMTF Redfish and publish profile definitions separately from validator execution, so ACORN must preserve the distinction between a referenced profile and evidence that a product passed a conformance test ([OCP hardware-management profiles](https://github.com/opencomputeproject/HWMgmt-OCP-Profiles), [DMTF Redfish Interop Validator](https://github.com/DMTF/Redfish-Interop-Validator)).

# Direction decisions

## Preserve requirement semantics

Keep `ResearchActivityMetadata::resources: Option<OneOrMany<Resource>>` as the minimum compute requirement. Do not add platform, facility, OCP recognition, or management-profile fields to individual `Resource` variants.

Represent an actual configured system separately as infrastructure. A GPU remains a hardware resource; a server containing GPUs, carrying standards claims, and associated with an OCP record is a compute platform.

## Separate claims from recognition

A standards claim describes a relationship such as `conformsTo`, `implements`, or `testedAgainst`. OCP recognition is an externally granted fact tied to an authoritative record. Manufacturer, architecture, model name, use of an OCP specification, or an image containing an OCP badge must never cause ACORN to infer recognition.

## Keep the portable schema data-only

Schema types parse, serialize, validate, and crosswalk immutable data. OCP catalog access, Redfish requests, caching, authentication, and offline behavior belong to future API or host adapters. Management endpoints, BMC credentials, tokens, and control operations are not represented in public RAD.

## Keep OCP records outside hardware

`acorn_schema::hardware` owns intrinsic resource, architecture, memory, and vendor semantics. OCP contribution, recognition, and profile records are external authority data, not hardware primitives, so they belong to `acorn_schema::standard::ocp` even when their subject is a device.

Enforce this one-way dependency graph:

```text
hardware          standard::ocp
       \          /
        infrastructure
              |
              v
      research_activity
```

- `hardware` depends on none of the new modules.
- `standard::ocp` may depend on generic types in `standard`, but not on `hardware`, `infrastructure`, or `research_activity`.
- `infrastructure` composes `hardware::Resource`, generic standards claims, and `standard::ocp` records.
- `research_activity` owns only the optional metadata field that contains infrastructure values.

## Enforce the file boundary literally

All initial infrastructure types live directly in `crates/acorn-schema/src/infrastructure.rs`. All initial OCP types live directly in `crates/acorn-schema/src/standard/ocp.rs`. Do not create `standard::reference`, `infrastructure::platform`, `infrastructure::facility`, `infrastructure::environment`, `ocp::contribution`, `ocp::recognition`, `ocp::profile`, or `ocp::crosswalk` modules in this plan.

# Target layout

This tree shows only the new files and their existing registration points. All other schema files remain in place.

```text
crates/acorn-schema/src/
├── infrastructure.rs
├── lib.rs
├── tests/
│   ├── infrastructure.rs
│   └── mod.rs
└── standard/
    ├── mod.rs
    ├── ocp.rs
    └── tests/
        ├── mod.rs
        └── ocp.rs
```

Only `infrastructure` and `standard::ocp` are new public logical modules. Their single-file entry points match the current layout and avoid speculative submodule structure.

# Schema design

## Generic standard claims

Add the following public types directly to the existing `acorn_schema::standard` root:

- `StandardReference`: authority, identifier, optional title, optional version, and canonical URI.
- `StandardsClaim`: standard reference, relationship, assessment, and optional evidence URI.
- `StandardsRelationship`: `ConformsTo`, `Implements`, `TestedAgainst`, `BasedOn`, and `References`.
- `ClaimAssessment`: `Declared`, `Inferred`, and `Verified`.

Require an absolute HTTP or HTTPS URI with a schema-local validator, non-empty authority and identifier, and evidence for a verified claim. Reject duplicate claims with the same authority, identifier, version, and relationship through an owning-value schema rule.

## OCP records

Define the following directly in `acorn_schema::standard::ocp`:

- `OcpContributionId`: validated `S` followed by one or more digits without assuming a fixed width.
- `OcpProject`: an open newtype rather than an enum because OCP projects and workstreams evolve.
- `OcpContributionKind`: specification, design files, software, reference architecture, tested configuration, requirements document, white paper, and an open fallback.
- `OcpContribution`: identifier, title, project, contribution kind, optional version, optional status, canonical URI, and optional license.
- `OcpRecognitionProgram`: Accepted, Inspired, Ready, S.A.F.E., Integrated Solution, and an open fallback.
- `OcpRecognition`: program, authority-facing subject identifier, authoritative source URI, and optional RFC 3339 `awarded_at` and `expires_at` timestamps.
- `ManagementProfileClaim`: OCP profile reference, underlying standard reference, optional validator identity and version, optional RFC 3339 `tested_at`, conformance result, and optional evidence URI for a validator report or authoritative OCP record.
- `ConformanceResult`: pass, fail, partial, and not tested.

Every tested management-profile result (`Pass`, `Fail`, or `Partial`) requires both `tested_at` and `evidence_uri`; `NotTested` makes no conformance assertion. Recognition placement is validated by the owning infrastructure type: `Accepted`, `Inspired`, and `S.A.F.E.` apply to platforms or devices; `Ready` applies only to facilities; and `Integrated Solution` applies to composed environments. An open fallback remains available for future programs but still requires an authoritative source URI.

Trim string inputs and reject empty contribution titles, project names, recognition subjects, and open-fallback payloads. Reuse nested validation for every embedded `StandardReference` instead of duplicating its URI and identity checks.

Connect an infrastructure standards claim to an OCP contribution through a borrowed, pure projection that sets the `StandardReference` authority to Open Compute Project and copies the contribution's `S` identifier, title, version, and canonical URI. The claim author must still choose the relationship and assessment; conversion must not infer conformance. Keep the OCP-specific contribution record available as its own type rather than embedding a mutable catalog snapshot in every platform. The recognition `subject_identifier` preserves the authority's identity and is not required to equal an ACORN-local owner identifier; nesting supplies the association.

## Infrastructure

Define the following directly in `acorn_schema::infrastructure`:

- `ComputePlatform`: `identifier`, `name`, optional `manufacturer` and `model`, non-empty installed `resources: OneOrMany<Resource>`, and optional `standards_claims`, product/device `ocp_recognitions`, and `management_profile_claims` collections.
- `ComputeFacility`: `identifier`, `name`, optional `operator` and appropriately coarse `location`, and optional `standards_claims` and facility `ocp_recognitions` collections.
- `ComputeEnvironment`: `identifier`, `name`, non-empty `platforms: OneOrMany<ComputePlatform>`, optional `facility`, optional environment-level `ocp_recognitions`, and optional RFC 3339 `observed_at` timestamp.

Use `Option<OneOrMany<T>>` for every optional collection above, consistent with current RAD fields. Validate nested resources and claims with the existing `Validate`/`OneOrMany` contracts. Reject empty installed-resource and platform collections and recognition programs attached to the wrong infrastructure kind. Require `expires_at` to follow `awarded_at` when both are present, validate all timestamps with the existing `timestamp` rule, and do not infer claims from vendor or architecture fields.

# Implementation plan

## 1. Establish a behavioral baseline

1. Run focused research-activity, standard, and crosswalk tests before adding fields.
2. Record representative RAD serialization snapshots and current public paths for `hardware::Resource`, `standard`, and `validation`.
3. Add one active regression case to `research_activity/tests/mod.rs`: first prove the current direct `Resource::validate` vendor-mismatch behavior, then make the same mismatch fail through complete RAD validation after `#[validate(nested)]` is added. The existing `hardware/tests` module is disabled by `#[cfg(any())]`; do not enable or rewrite that unrelated suite in this plan.
4. Confirm default, no-default-features, alloc, and WASM schema configurations before structural changes.

## 2. Consume the consolidated validation module

The folder-format `acorn_schema::validation` consolidation and collection traversal have landed. This plan relies on them without repeating or expanding the validation framework:

1. Reuse `nonempty`, `timestamp`, `nested`, `each`, and the existing whole-value schema-rule contract for new fields and cross-field invariants.
2. Keep the generic `url` rule unchanged. Add one private HTTP(S)-only helper beside the standard claim types and reuse it from OCP types that require authoritative web records.
3. Reuse `OneOrMany<T>: Validate` for indexed collection paths; do not add OCP-specific collection traversal.
4. Do not create `standard/validation.rs` or change `acorn-macros` unless a failing focused test proves the implemented combinators cannot express a required rule.
5. Preserve existing validation behavior and error codes from the consolidated module.

## 3. Add generic standard claims and OCP types

1. Add generic reference and claim types to `crates/acorn-schema/src/standard/mod.rs` with portable validation, serde, and JSON Schema support.
2. Create `crates/acorn-schema/src/standard/ocp.rs`, register it without an `std` feature gate, and add contribution, recognition, and management-profile types without importing hardware or infrastructure types or using host-only dependencies.
3. Add a borrowed pure conversion from `&OcpContribution` to `StandardReference` in `standard/ocp.rs`. Do not infer a `StandardsClaim` relationship or assessment, couple the generic crosswalk mapping module to OCP, or create another OCP submodule.
4. Add focused tests in the existing `standard/tests` layout for identifiers, open fallback values, HTTP(S)-only authoritative URIs, evidence rules, recognition source URIs, tested-result timestamps, and management-profile results.

## 4. Add infrastructure and integrate RAD

1. Create `crates/acorn-schema/src/infrastructure.rs` with platform, facility, and compute-environment types that compose hardware resources with generic and OCP claims.
2. Export `pub mod infrastructure` from the crate root without an `std` feature gate. Keep its types module-qualified instead of adding speculative crate-root re-exports.
3. Preserve the current manual `Validate for Resource` implementation and its `validate_vendor` behavior. Add `#[validate(nested)]` to `ResearchActivityMetadata::resources` so `OneOrMany<Resource>` failures propagate through complete RAD validation with indexed paths.
4. Add an optional `infrastructure: Option<OneOrMany<ComputeEnvironment>>` field using the same `#[builder(into)]`, one-or-many deserialization, nested validation, and camel-case serialization conventions as adjacent structured metadata.
5. Add the corresponding optional field to `ResearchActivityMetadataContext`, its default builder, and a compatibility default function so older explicit `@context` objects remain readable. Use established predicates such as `dcterms:conformsTo`, `dcterms:identifier`, `dcterms:source`, `dcterms:hasVersion`, `schema:manufacturer`, and `schema:location` where their semantics fit.
6. Classify `infrastructure` as local-only in the standards field matrix and emit structured conversion warnings from RAiD and DataCite project export when populated; do not silently discard it.
7. Do not invent an OCP RDF namespace; use canonical OCP record URLs as identifiers.

## 5. Preserve compatibility and document the boundary

1. Keep all existing standard and hardware module paths and public items unchanged.
2. Keep existing RAD documents valid because infrastructure is optional, accepts scalar-or-array input through `OneOrMany`, and the existing resource representation is unchanged.
3. Update the standards field matrix's stale `resources: Option<Vec<Resource>>` notation to the implemented `Option<OneOrMany<Resource>>` type while preserving its `0..n` serialized cardinality.
4. Document the distinction between required resources, an actual environment, a standards claim, an OCP recognition, and a conformance-test result.
5. Document that schema models do not resolve OCP records, contact Redfish services, operate hardware, or grant OCP recognition.

# Validation

Run focused checks after each implementation stage, followed by repository gates:

```sh
cargo nextest run --package acorn-schema standard
cargo nextest run --package acorn-schema research_activity
cargo nextest run --package acorn-schema infrastructure
cargo nextest run --package acorn-schema crosswalk
cargo check -p acorn-schema --no-default-features
cargo check -p acorn-schema --no-default-features --features alloc
cargo check -p acorn-schema --no-default-features --features wasm
make check-wasm
make lint
make test
make book
```

Also verify:

- Existing standard and hardware public imports compile without changes.
- `hardware` has no dependency on OCP or infrastructure, `standard::ocp` has no dependency on hardware or RAD, and only `infrastructure` composes those domains.
- `standard/validation.rs` and the `standard::validation` declaration are absent.
- `validation/mod.rs` preserves existing validator behavior, feature gates, and error codes.
- Existing standard and RAD serialization snapshots remain unchanged except for explicitly approved optional-field output.
- JSON and YAML round trips cover all new types.
- Unknown fields are rejected where the external record is modeled as a closed contract; open OCP categories retain explicit fallback variants.
- Direct `Resource::validate` retains the current architecture/vendor mismatch behavior, and complete RAD validation reports that failure at an indexed path under `meta.resources`.
- A verified claim without evidence fails validation.
- `Accepted`, `Inspired`, or `S.A.F.E.` on a facility fails; `Ready` on a platform fails; and `Integrated Solution` outside an environment fails.
- `Pass`, `Fail`, and `Partial` management-profile claims without `tested_at` or `evidence_uri` fail validation; `NotTested` remains explicitly non-assertive.
- Empty installed-resource or environment-platform collections fail validation.
- An OCP contribution projected into a generic reference sets the Open Compute Project authority and retains its `S` identifier, title, version, and canonical URI without inferring a claim relationship.
- Recognition and observation timestamps reject missing offsets, and expiration cannot precede award.
- Linked-data context uses established predicates where their semantics fit and official OCP URLs for OCP identities; no OCP namespace or relationship predicate is invented.
- RAiD and DataCite project conversion report populated infrastructure as an explicit local-only loss, and the standards field matrix records the same classification.
- `rg` finds no sustainability schema fields, types, crosswalks, or implementation steps introduced by this plan.

# Acceptance criteria

1. The only new logical Rust modules are the single-file `acorn_schema::infrastructure` and `acorn_schema::standard::ocp`; no existing standard or hardware module moves.
2. `hardware` remains independent, `standard::ocp` depends only on generic schema/standard contracts, `infrastructure` composes both domains, and `research_activity` only contains the optional infrastructure field.
3. Generic standard claims preserve authority, identifier, version, relationship, assessment, canonical source, and verification evidence.
4. OCP contributions, recognition programs, and management-profile results are distinct typed concepts with authoritative evidence; contribution-backed claims retain the authoritative contribution identity.
5. Product/device, facility, and integrated-solution recognitions cannot be attached to the wrong infrastructure kind.
6. A compute requirement remains distinct from the installed platforms and optional facility actually used by a research activity; each environment has its own identity and does not duplicate platform resources.
7. Existing RAD documents and standard imports remain compatible; new infrastructure metadata is `Option<OneOrMany<ComputeEnvironment>>` and optional context terms have compatibility defaults.
8. Nested resource validation runs through complete RAD validation without replacing the current manual `Resource` validator.
9. Infrastructure is classified and reported as local-only by the implemented RAiD/DataCite loss contract until an approved target mapping exists.
10. No ACORN behavior infers OCP recognition from a vendor, architecture, product name, standards claim, or badge image.
11. Portable schema code performs no network, filesystem, credential, hardware-control, or live Redfish operations.
12. Default, minimal, alloc, and WASM checks plus full lint, test, and book gates pass.
13. No sustainability data model or crosswalk is added by this plan.

# Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| OCP types leak into hardware or create a dependency cycle. | High | Keep OCP authority records independent and compose them with resources only in `infrastructure`. |
| Resource work replaces or bypasses the implemented manual vendor validator. | High | Preserve `Validate for Resource`, add only metadata nesting, and assert the same failure directly and through a complete RAD value. |
| Dormant hardware tests expand the feature into unrelated repair. | Medium | Add focused active regression coverage under research-activity tests and leave the disabled suite unchanged. |
| Generic claims are mistaken for official recognition. | High | Keep claim assessment and OCP recognition as separate types with distinct validation. |
| OCP projects or contribution categories evolve. | Medium | Use open newtypes and explicit fallback variants instead of closed project enums. |
| Recognition scope is flattened across products, facilities, and integrated solutions. | High | Validate each program at the owning infrastructure type and require an authoritative source URI. |
| Detailed infrastructure exposes operationally sensitive information. | High | Exclude credentials, management endpoints, hostnames, serial numbers, and fine-grained facility locations from the portable public model. |
| OCP catalog or Redfish access leaks into portable schemas. | High | Restrict this plan to immutable data models and pure conversions; place future transport in adapters. |
| New infrastructure metadata is silently omitted by existing standards exports. | High | Add field-matrix coverage and explicit local-only conversion warnings before accepting the RAD field. |

# Related concepts

- [Crate boundary architecture](../refactors/crate-boundary-architecture.md)
- [CKAN package to DCAT crosswalk](../migrations/ckan-dcat-crosswalk.md)
