---
type: Plan
title: "Add portable RRID research-resource support"
description: "Add deterministic RRID parsing, local discovery and persistence, RAD validation, and standards interchange without requiring a registry provider."
difficulty: high
impact: high
package: [acorn-schema, acorn-lib, acorn-cli]
tags: [feature, pid, rrid, research-resource, discovery, datacite, jats, reproducibility]
status: in-progress
---

# Project code boundaries

- **Update:** `crates/acorn-schema/src/pid/mod.rs`, `crates/acorn-schema/src/util/constants/{mod,app}.rs`, `crates/acorn-schema/src/validation/mod.rs`, `crates/acorn-schema/src/validate/mod.rs`, focused `ResearchActivity`/`TypedIdentifier` validation and tests, DataCite adapter tests and `.wiki/assets/standards-field-matrix.md`; `crates/acorn-lib/src/analyzer/host/discovery/` and `crates/acorn-lib/src/io/database/tests/discovery.rs`; focused gather tests and documentation in `crates/acorn-cli`; persistent-identifier and gather documentation.
- **Create:** `crates/acorn-schema/src/pid/rrid/{mod,tests}.rs`; immutable parser, discovery, RAD, DataCite 4.7, and JATS 1.4 fixtures under `tests/fixtures/rrid/`.
- **Delete:** None. Do not remove or repurpose `ResearchActivityMetadata.resources`, which remains the hardware/compute-requirements field.

# Dependencies

- The portable RRID leaf can start in the PID/schema lane. Serialize shared edits to `pid/mod.rs`, PID constants, validation registries, `ResearchActivity` types, and standards fixtures with other owners of those integration files.
- [RAD metadata standards alignment](rad-metadata-standards-alignment.md) has landed its DataCite 4.7 type audit, structured loss reporting, and reusable `TypedIdentifier`/`TypedRelatedResource` surface. Its RAD/DataCite adapters remain unfinished, so RRID interchange must coordinate with that plan rather than claim an existing adapter boundary.
- The current discovery lifecycle and generic `discoveries` persistence contract are prerequisites for local gather integration. Provider-backed resolution is a follow-on in [SciCrunch RRID provider](rrid-scicrunch-provider.md), not a prerequisite.
- The landed SWHID implementation at baseline `50e319e1` is the implementation precedent for strict typed parsing, PID enum integration, conservative document discovery, project-identity policy, lifecycle-aware resolution, and fixture-backed resolver tests. There is no active SWHID plan dependency.

# Goal

Make RRIDs first-class, portable evidence for research resources: parse and normalize them locally, discover only explicit citations and approved resolver URLs, persist discovery history, validate authored RAD relations, and exchange supported semantics through DataCite 4.7 and JATS 1.4 fixtures.

This baseline does not contact SciCrunch, accept provider credentials, define remote-search result types, or add CLI/TUI registry search. Those concerns remain independently gated in the provider follow-on.

# Refresh decision (2026-09-07)

- The plan was reconciled against `50e319e1`. SWHID support had landed and schema module flattening was complete at that baseline.
- Keep the canonical form `RRID:<opaque-payload>`. Canonicalize the outer label and manuscript spacing only; preserve payload case and punctuation until an authoritative contract proves a stronger identity rule.
- Require an explicit RRID label or an approved RRID resolver URL for discovery. Bare values such as `SCR_003512` and `CVCL_3588` are not RRID discoveries in arbitrary prose.
- Add `PID::RRID` as discoverable while keeping `PID::is_project_identifier()` false. RRIDs identify experimental inputs and related resources, not research projects.
- Reuse the generic discovery row and lifecycle fields; local RRID persistence needs no migration.
- Represent authored RRIDs through `TypedRelatedResource`. Do not change the meaning of hardware `resources` or add a competing resource collection.
- Preserve explicit DataCite relations and loss reporting. JATS 1.4 remains fixture and mapping coverage unless a real XML adapter boundary exists when the slice begins.

# Implementation status (2026-09-07)

- The portable `RRID` leaf, `PID::RRID` integration, conservative discovery, canonical raw output, generic discovery persistence, typed RAD validation, and user documentation have landed.
- Offline fixtures cover heterogeneous payloads, accepted resolver aliases, invalid syntax, JATS 1.4 markup, and the DataCite 4.7 RRID identifier/relation combination.
- RRID resolution remains unsupported and no provider, credential, registry-search, or TUI surface was added.
- RAD/DataCite import and export remain coordinated with the unfinished standards-alignment adapters. Dedicated PDF and DOCX RRID discovery fixtures also remain open before this plan can be completed.

# Current evidence

- `crates/acorn-schema/src/pid/rrid/` now provides strict local parsing, canonical display, allowlisted resolver aliases, conservative discovery, serde/JSON Schema support, and fixture-backed tests. `PID::RRID` is integrated through centralized discovery, normalization, conversion, serialization, and project-identity policy.
- The SWHID leaf uses strict `FromStr` validation, canonical `Display`, resolver alias handling, conservative `find_all`, serde/JSON Schema support, and PID conversions. Its provider integration preserves `Resolved`, `Failed`, `Unsupported`, and `NotRequested` lifecycle states with fixture-backed tests.
- `crates/acorn-lib/src/analyzer/host/discovery/` retains RRIDs with `NotRequested` by default and `Unsupported` when resolution is requested. Regression tests prove RRIDs do not identify or merge project candidates.
- `crates/acorn-lib/src/io/database/schema.rs` stores normalized identifier, lowercase identifier type, optional metadata, resolution status, source, and source format in generic discovery rows. RRID round-trip coverage uses this existing schema without a migration.
- `ResearchActivityMetadata.related_resources` uses `TypedRelatedResource`, while `resources` remains hardware/compute requirements. `TypedIdentifier` now dispatches `scheme: RRID` values to the canonical parser without changing other schemes.
- DataCite 4.7 models `RelatedIdentifierType::Rrid`, and ACORN's 4.7 fixtures now pin the exact RRID identifier, type, `Cites` relation, and `Software` resource type. ACORN still has no `ResearchActivity`/`ResearchOutput` DataCite adapter to produce or consume that relation. DataCite defines RRID for key experimental inputs and illustrates `RRID:SCR_014641` with an explicit `Cites` relation ([DataCite 4.7 RRID](https://datacite-metadata-schema.readthedocs.io/en/4.7/appendices/appendix-1/relatedIdentifierType/)).
- JATS 1.4 provides RRID markup through `resource-id-type="rrid"` and specifies case-insensitive comparison of that attribute value ([JATS 1.4 RRID markup](https://jats.nlm.nih.gov/publishing/tag-library/1.4/attribute/resource-id-type.html)). ACORN now has an offline JATS 1.4 fragment fixture and mapping documentation, but no general JATS XML adapter.
- The RRID system aggregates identifiers for research resources from multiple source registries and retains unavailable records, so local syntax validity must remain separate from registry presence and availability ([RRID system](https://www.rrids.org/rrid-system)).

# Recommendation

Land one conservative portable RRID leaf, then integrate it through existing discovery, persistence, and typed-related-resource contracts. Treat syntax validity, registry presence, and resource availability as separate facts. Coordinate standards mappings with the existing adapter owner, and release this baseline independently of SciCrunch access.

# Identifier contract

| Concern | Required behavior |
|---|---|
| Canonical display | Emit `RRID:<payload>` with uppercase `RRID` and no whitespace after the delimiter. |
| Payload | Treat the non-empty, bounded payload as opaque. Preserve case and valid punctuation; do not impose one authority grammar or invent a checksum. |
| Accepted manuscript form | Match the outer label case-insensitively and tolerate horizontal whitespace around the label delimiter only where fixtures approve it. Reject controls, line breaks, malformed percent escapes, URL query/fragment residue, and adjacent prose. |
| Discovery | Discover explicit RRID labels and allowlisted resolver URL forms only. Never classify a bare source-registry identifier as an RRID from prose. |
| Resolver URL | Emit one reviewed persistent HTTPS form and accept only fixture-approved RRID resolver aliases. Strip tracking query data rather than making it identity-bearing. |
| Deduplication | Use canonical RRID plus the existing source policy. An explicit RRID inside a resolver URL must not also appear as a generic URL discovery. |
| Identity | `PID::RRID` is discoverable and serializes as `rrid`, but is never project-identifying. |
| Network | Parsing, validation, discovery, serialization, and persistence perform no network access and require no credentials. |

Representative fixtures must cover `RRID:AB_2140114`, `RRID:IMSR_JAX:000664`, `RRID:CVCL_3588`, `RRID:Addgene_12260`, `RRID:SCR_003512`, and `RRID:SAMN33284296`. These examples deliberately exercise heterogeneous payload punctuation.

# RAD and standards contract

Represent an authored RRID through the existing relation type:

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

The example relation is explicit source data, not a default. Plain-text co-occurrence must not infer `Cites`, a resource type, or a scheme URI. DataCite mapping preserves `relationType`, `relationTypeInformation`, and `resourceTypeGeneral`; missing target-required relations fail readiness or produce the existing structured conversion outcome.

Coordinate DataCite work with the unfinished standards-alignment adapters. Map `RelatedItem` only where that adapter can retain its structure, and record every simplification or omission in the standards field matrix. For JATS, add case-insensitive RRID fragment fixtures and a documented mapping; add executable XML conversion only if an existing or newly approved XML adapter owns the behavior.

# Implementation plan

Each slice must declare an exact write set before implementation. Shared module roots, schema types, fixtures, and generated artifacts are integration locks.

## 0. Freeze portable syntax and fixture contracts

**Status:** Landed for the portable parser and current standards claims.

**Write set:** this plan and new immutable fixtures under `tests/fixtures/rrid/`; no Rust production files.

- Record source URL, retrieval date, provenance, input form, expected canonical form, and expected validity for representative payload families.
- Cover canonical and manuscript forms, approved resolver aliases, mixed-case outer labels, spacing, Unicode prose boundaries, punctuation, duplicate aliases, DataCite snippets, and JATS 1.4 fragments.
- Cover bare authority identifiers, empty payloads, doubled labels, control characters, line breaks, malformed percent encoding, URL query values, unsupported schemes, and ordinary colon-delimited prose.
- Decide the emitted resolver URL from documented persistent behavior. Do not use a portal redirect as proof of registry existence.

**Deliverable:** a reviewed offline fixture set from which parser, discovery, and mapping tests are generated.

## 1. Add the portable `RRID` PID leaf

**Status:** Landed.

**Primary write set:** new `crates/acorn-schema/src/pid/rrid/{mod,tests}.rs`; focused integration changes in `pid/mod.rs`, `util/constants/{mod,app}.rs`, `validation/mod.rs`, and `validate/mod.rs`.

- Implement an immutable parsed value with strict `FromStr`, actionable owned errors, canonical `Display`, serde, JSON Schema, `PersistentIdentifier`, and `PersistentIdentifierParse` parity with landed PID leaves.
- Keep exact parsing separate from discovery regexes. Regexes locate candidates; the parser decides validity and URL decoding.
- Add `PID::RRID`, wire name `rrid`, enum helpers, normalization, formatting, conversions, serialization, and `From<RRID> for Identifier`.
- Make RRID discoverable and explicitly non-project-identifying.
- Add scalar and typed-identifier validation for `scheme: RRID`, using the same scheme case policy as other typed identifiers.
- Test parser totality, parse/display round trips, idempotent normalization, deterministic resolver aliases, adjacent-prose rejection, and collisions with URL, Handle, RAiD, DOI, and ordinary colon-delimited text.

**Deliverable:** `acorn-schema` parses, validates, discovers, serializes, and renders RRIDs with no `std`, filesystem, network, or provider dependency.

## 2. Integrate local discovery and persistence

**Status:** Partially landed. Text discovery, alias deduplication, canonical CLI output, lifecycle behavior, generic persistence, and non-project identity are covered; dedicated PDF and DOCX fixtures remain.

**Primary write set:** focused discovery tests and implementation in `crates/acorn-lib/src/analyzer/host/discovery/`, `crates/acorn-lib/src/io/database/tests/discovery.rs`, gather tests in `crates/acorn-cli`, and dedicated document fixtures.

- Add RRID to plain-text, PDF, and DOCX discovery coverage where those fixture harnesses already exist.
- Verify alias deduplication within a source, preservation across sources, stable ordering, lowercase `identifier_type: "rrid"`, and generic database round trips without a migration.
- Preserve `NotRequested` by default and `Unsupported` when resolution is requested before the provider follow-on lands.
- Prove RRIDs never enter project candidates, create research-activity rows, or merge unrelated candidates.
- Keep raw/report output canonical and machine-readable; do not replace RRID lines with citation prose.

**Deliverable:** local gather reports and persists one normalized RRID with source and lifecycle state and no project-candidate side effects.

## 3. Complete RAD validation and standards interchange

**Status:** Partially landed. Typed RRID validation and DataCite/JATS fixture semantics are covered; RAD/DataCite adapter conversion remains with the standards-alignment plan.

**Primary write set:** focused `TypedIdentifier`/`TypedRelatedResource` validation, standards-alignment adapter files when that owner approves the shared boundary, DataCite tests, dedicated fixtures, and `.wiki/assets/standards-field-matrix.md`.

- Validate authored RRIDs through the portable parser while preserving the explicit related-resource relation.
- Add DataCite 4.7 RRID import/export only through the standards-alignment adapter, preserving identifier and relation semantics and emitting structured losses.
- Classify `RelatedItem` and JATS fields as exact, transformed, simplified, local-only, or unsupported in the standards matrix.
- Keep JATS at fixture/mapping coverage unless a real XML adapter boundary exists.
- Prove legacy RAD documents still deserialize and hardware `resources` retains its meaning.

**Deliverable:** RAD can author and validate RRID relations, and every claimed standards transformation has fixture-backed semantics and loss reporting.

## 4. Document and harden the portable release

**Status:** Partially landed. User documentation and default, minimal, alloc, lint, test, and WASM gates are green; dedicated PDF and DOCX acceptance fixtures remain.

**Primary write set:** persistent-identifier and gather documentation, generated schema assertions, dedicated acceptance fixtures, and this plan.

- Document canonical and accepted forms, local validation versus registry confirmation, non-project identity, persistence behavior, standards coverage, and the independently gated provider follow-on.
- Add end-to-end local cases for text, PDF, DOCX, authored RAD, DataCite, JATS fixtures, and discovery history.
- Run default, minimal, alloc, and WASM portability gates. No portable test contacts N2T, SciCrunch, an RRID portal, or a source registry.

**Deliverable:** the portable RRID surface is documented, compatible, and releasable without a live-service dependency.

# Delivery order

1. Freeze offline syntax and mapping fixtures.
2. Land the portable PID leaf.
3. Integrate local discovery and generic persistence.
4. Coordinate RAD/DataCite work with the unfinished standards adapters.
5. Complete documentation and portability gates.

The [SciCrunch provider](rrid-scicrunch-provider.md) may begin contract review independently, but no provider implementation is part of this delivery.

# Validation

Focused implementation gates use the flattened `acorn-schema` module paths:

```sh
cargo nextest run -p acorn-schema 'pid::rrid'
cargo nextest run -p acorn-schema 'research_activity'
cargo nextest run -p acorn-schema 'standard::tests::datacite'
cargo nextest run -p acorn-schema 'standard::crosswalk'
cargo nextest run -p acorn-lib 'analyzer::host::discovery'
cargo nextest run -p acorn-lib 'io::database::tests::discovery'
cargo nextest run -p acorn-cli 'commands::gather'
```

Repository gates:

```sh
cargo fmt --all
cargo check -p acorn-schema --no-default-features
cargo check -p acorn-schema --no-default-features --features alloc
make check-wasm
make lint
make test
```

Also verify fixture tests are offline, normalization is idempotent, aliases deduplicate correctly, explicit RRIDs do not duplicate as generic URLs, RRIDs never become project identities, and DataCite relation losses are explicit.

Results on 2026-09-07: `cargo fmt --all`, both minimal schema checks, `make check-wasm`, `make lint`, and `make test` passed. The full test run completed 1,564 tests with 1,564 passing and 14 skipped.

# Acceptance criteria

- Representative heterogeneous RRID payloads parse and normalize to `RRID:<opaque-payload>` without case-folding or imposing one authority grammar.
- Mixed-case outer labels, approved resolver aliases, manuscript spacing, and surrounding punctuation normalize exactly as pinned fixtures specify.
- Bare source identifiers are not discovered as RRIDs; malformed RRIDs fail locally; registry presence or availability is not inferred from syntax.
- `PID::RRID` serializes as `rrid`, is discoverable, and is never a project identifier.
- Gather and generic discovery persistence retain canonical identifier, source, format, and lifecycle status without a database migration or project-candidate side effect.
- Authored RRIDs use `relatedResources`/`TypedRelatedResource`; hardware `resources` is unchanged.
- DataCite 4.7 mappings preserve explicit identifier and relation semantics through the coordinated adapter and report every simplification or omission.
- JATS claims and fixtures target 1.4 without implying a general JATS parser.
- Existing RAD fixtures and PID behavior remain compatible; default, minimal, alloc, and WASM checks stay green.
- Portable RRID behavior requires no credentials and performs no network access.

# Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Heterogeneous payloads are reduced to one guessed grammar. | High | Keep payloads opaque and derive behavior from multiple cited fixture families. |
| RRIDs become project identities and merge unrelated activities. | Critical | Keep `is_project_identifier` false and add candidate/database regression tests. |
| A second RAD resource model overlaps hardware requirements or related resources. | High | Reuse `TypedRelatedResource`; leave hardware `resources` unchanged. |
| DataCite relations are fabricated from co-occurrence. | High | Preserve explicit relations and require readiness failures or structured losses for missing target semantics. |
| JATS fixture coverage is described as a complete adapter. | Medium | Claim executable conversion only when a real XML adapter owns it. |
| Provider uncertainty delays portable support. | High | Keep all network, credential, DTO, and search work in the separately gated follow-on. |

# Explicitly out of scope

- SciCrunch credentials, endpoint configuration, lookup DTOs, registry resolution, remote-search provider types, CLI registry search, and TUI work.
- Minting or registering RRIDs.
- Rewriting manuscripts or mining publication mention graphs.
- Treating bare catalog or registry identifiers as RRIDs.
- Adding rich provider metadata to authored RAD or a second canonical resource collection.
- Building a general JATS document parser solely for RRID support.
- Making validation depend on network access.

# Related concepts

- [SciCrunch RRID provider](rrid-scicrunch-provider.md)
- [Persistent identifier lifecycle](../../docs/src/concepts/persistent-identifiers.md)
- [RAD metadata standards alignment](rad-metadata-standards-alignment.md)
- [Analyzer composition and traversal](../refactors/analyzer/analyzer-composition.md)
- [Crate boundary architecture](../refactors/crate-boundary-architecture.md)
