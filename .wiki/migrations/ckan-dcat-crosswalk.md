---
type: Plan
title: "CKAN package to DCAT crosswalk"
description: "Turn the paired CKAN and DCAT fixtures into a tested CKAN-to-DCAT v3 crosswalk while separating CKAN publishing from metadata normalization."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [ckan, dcat, dcat-us, crosswalk, metadata, fixtures]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/schema/standard/{dcat,crosswalk}/`, `src/analyzer/check.rs`, `crates/acorn-cli/src/{cli/parse.rs,commands/export/mod.rs}`, and schema/crosswalk tests.
- **Create:** CKAN source types and meaningful fixtures under `tests/fixtures/schema/{ckan,dcat}/`; target `acorn-schema` after extraction.
- **Delete:** lossy placeholder fixture assertions and any duplicate CKAN conversion route after the direct crosswalk is authoritative.

# Dependencies

Consumes the stabilized DCAT/loss contracts in the [standards field matrix](../assets/standards-field-matrix.md) and must preserve the schema/adapter boundary in the [crate boundary architecture](../refactors/crate-boundary-architecture.md).

# Goal

Make the three CKAN `package_show` fixtures executable evidence for a first-class CKAN → DCAT conversion. The first delivery should read CKAN package JSON, preserve or explicitly warn about source metadata, produce ACORN's DCAT model, and validate the result. It should not imply that a CKAN API record is already DCAT or that arbitrary DCAT can be sent back to CKAN safely.

# Direction decision

Implement **CKAN → DCAT first**.

The source files in `tests/fixtures/schema/ckan/` are CKAN package records. The same stems in `tests/fixtures/schema/dcat/` are hand-normalized DCAT expectations. That fixture layout describes a source-to-expected-output test even though it is not wired into the crosswalk engine.

DCAT → CKAN should be planned separately after a publishing use case identifies the target CKAN version, extensions, and API action. A CKAN `package_show` result contains read-only and server-maintained properties; a payload for `package_create` or `package_update` is a different contract. A reverse serializer that reproduces `package_show` JSON would therefore not, by itself, provide a safe CKAN publishing capability.

# Current evidence

- `Standard` parsing and `Standard::crosswalk()` support DataCite, DCAT, InvenioRDM, and HuWise, but have no CKAN variant or dispatch path (`crates/acorn-cli/src/cli/parse.rs`, `crates/acorn-lib/src/analyzer/check.rs`).
- The three CKAN fixtures are not referenced by tests or production code. Each has a manually derived file with the same stem under `tests/fixtures/schema/dcat/`.
- The CKAN fixtures resemble unwrapped `package_show` result objects and include core fields plus extension-specific `contact`, `creator`, `publisher`, and `qualified_relation` fields.
- All three fixture `resources` arrays are empty, so they cannot verify the important CKAN resource → DCAT distribution mapping.
- The expected DCAT files omit `@type`, a catalog or catalog-record wrapper, distributions, profile declaration, and several CKAN administrative fields. They are valid inputs for ACORN's permissive `dcat::Dataset`, but they are not evidence of full DCAT-US v3 conformance.
- The expected files currently map `metadata_created` and `metadata_modified` to dataset `issued` and `modified`. CKAN describes those values as metadata-record administration, while DCAT distinguishes a dataset from its catalog record. The custom CKAN fields `issued` and `modified` are better candidates for dataset lifecycle dates.
- Crosswalk warning collection covers direct DataCite mappings and DCAT → DataCite, not conversions routed through DataCite. In the CLI, `--strict` and `--dry-run` currently call the warning-free conversion path, so strict mode cannot enforce its documented loss check and dry-run always reports zero warnings (`crates/acorn-cli/src/commands/export/mod.rs`).

# CKAN compared with DCAT v3 and DCAT-US v3

CKAN's Action API returns an application record used to operate a catalog. DCAT 3 is an RDF vocabulary for interoperable descriptions of catalogs, datasets, data services, and distributions. DCAT-US v3 is a U.S. application profile of DCAT 3 with a concrete JSON Schema, requirement levels, and federal profile structures. These are related models, not interchangeable JSON shapes.

| Concern | CKAN package record | W3C DCAT 3 / DCAT-US v3 consequence |
|---|---|---|
| Primary purpose | Catalog application state returned by actions such as `package_show` | Portable semantic description of catalog resources; DCAT-US adds a federal validation profile |
| Identity | Server UUID `id`, unique slug `name`, and optional page `url` | RDF node IRI (`@id`) and one or more domain identifiers; a local UUID is not automatically a globally resolvable IRI |
| Container | One package result, sometimes wrapped by the Action API response envelope | A `Catalog` contains datasets/services/series; a `CatalogRecord` can describe registration metadata |
| Dataset text | Scalar `title` and `notes`; `notes` may contain HTML | DCAT properties have RDF cardinality; ACORN permits one-or-many title/description values, while profile JSON shapes may be narrower |
| Files and APIs | `resources` are CKAN-managed child records with storage, cache, datastore, hash, and tracking fields | `dcat:Distribution` describes an accessible representation; `dcat:DataService` describes service-based access |
| Classification | `tags` and `groups` are CKAN objects with IDs and lifecycle state | `dcat:keyword` is literal text; `dcat:theme` generally identifies a concept; grouping has no automatic one-field equivalent |
| Organization and people | `owner_org`, an expanded `organization`, legacy author/maintainer scalars, and plugin fields | `dcterms:publisher`, `dcterms:creator`, and `dcat:contactPoint` have distinct semantics and structured ranges |
| Record timestamps | `metadata_created` and `metadata_modified` describe CKAN metadata administration | Prefer `dcat:CatalogRecord` dates; do not silently substitute them for dataset release/update dates |
| State and visibility | `state`, `private`, `isopen`, ownership, counts, and user IDs drive CKAN behavior | Access rights, restrictions, status, and provenance require semantic values; many operational fields have no DCAT equivalent |
| Extensibility | Plugins add arbitrary top-level fields and `extras` key/value entries | DCAT is extensible through RDF vocabularies; DCAT-US accepts defined profile structures and validates a specified JSON shape |
| Relationships | CKAN subject/object relationship records and plugin-specific qualified relations | DCAT uses resource IRIs and qualified relationships with roles; CKAN labels need controlled, reversible mapping |
| Validation | Depends on CKAN version, dataset schema, and installed plugins | DCAT 3 defines vocabulary semantics; DCAT-US v3 uses JSON Schema 2020-12 and Mandatory/Recommended/Optional requirements |

The model distinctions are documented by the [CKAN Action API](https://docs.ckan.org/en/latest/api/), the [W3C DCAT 3 Recommendation](https://www.w3.org/TR/vocab-dcat-3/), and the [DCAT-US v3 overview and schema reference](https://resources.data.gov/resources/dcat-us3/).

# Recommended mapping policy

Use a direct CKAN → DCAT mapping rather than routing through DataCite. Preserve the distinction between semantic dataset metadata and CKAN administration.

| CKAN source | DCAT target | Policy |
|---|---|---|
| `url` | `@id`, `landingPage` | Use an absolute public dataset URL as the node IRI and landing page; never emit fixture-local `localhost` as a production IRI without a warning |
| `id` | `identifier` | Preserve the CKAN UUID as an identifier, not an IRI |
| `name` | warning or alternate identifier | Preserve the slug only when the target representation has an explicit identifier policy |
| `title` | `title` | Preserve text and adapt cardinality to the selected target profile |
| `notes` | `description` | Define deterministic HTML handling; retain text boundaries and warn if markup is discarded |
| custom `issued`, `modified` | dataset `issued`, `modified` | Prefer the domain fields after ISO 8601 validation |
| `metadata_created`, `metadata_modified` | catalog-record `issued`, `modified` | Preserve in a `CatalogRecord` when the output includes record provenance; otherwise issue loss warnings |
| custom `publisher`, `creator`, `contact` | `publisher`, `creator`, `contactPoint` | Map structured extension values explicitly; define fallback precedence for core `organization`, author, and maintainer fields |
| `tags[].name` | `keyword` | Preserve tag labels; do not infer DCAT themes without a vocabulary identifier |
| `resources[]` | `distribution[]` | Map public access URL, download URL, title, description, media type/format, size, dates, license, and checksum only where source semantics support them |
| `qualified_relation` and CKAN relationships | `qualifiedRelation` | Normalize resource identifiers and map relationship labels to documented role IRIs; warn on unknown roles |
| `license_id`, `license_url`, `license_title` | `license` and/or rights label | Prefer a stable license URI; do not place a display label in an IRI-only field |
| `private`, `isopen`, `state` | access/status fields or warning | Map only through an explicit controlled policy; never equate CKAN workflow state with a DCAT status IRI by spelling alone |
| `groups`, ownership, user IDs, counts, tracking, cache/datastore state | no dataset equivalent | Omit with categorized conversion warnings unless a separate provenance extension is selected |
| unknown plugin fields and `extras` | extension policy or warning | Capture unknown values during parsing so schema evolution does not silently discard them |

# Implementation plan

## 1. Define the target contract

1. Decide whether `--to dcat` means the flexible ACORN/W3C DCAT model or strict DCAT-US v3 JSON. Keep those concepts distinct in names, validation, and help text.
2. Treat the existing paired files as ACORN DCAT golden outputs initially. Add profile-specific goldens only after DCAT-US requirements and cardinalities are represented by the schema.
3. Document whether output is a single `Dataset`, a `Catalog` containing datasets, or a dataset plus catalog-record provenance. Prefer a catalog form when preserving CKAN metadata timestamps.

## 2. Add a CKAN source model

1. Add a `schema::standard::ckan` module with a `Package`, `Resource`, `Tag`, `Organization`, relationship, and known extension types needed by the fixtures.
2. Model a `package_show` result separately from the optional Action API `{success, result, error}` envelope.
3. Accept CKAN extension fields without silently discarding them. Use an explicit flattened value map where CKAN's plugin extensibility makes `deny_unknown_fields` incorrect, and surface unmapped entries as warnings.
4. Keep server-read models separate from any future create/update payload models.

## 3. Implement direct CKAN → DCAT conversion

1. Add `Standard::Ckan`, CLI parsing/help, content inference based on CKAN-specific structural evidence, schema dispatch, and `--from ckan --to dcat` routing.
2. Implement `SchemaExtractor` or a dedicated pure converter using the mapping policy above. Do not force CKAN fields through DataCite where that would erase catalog administration, resources, or access semantics.
3. Generate `ConversionWarning` values for unsupported operational fields, unknown plugin fields, HTML normalization, ambiguous agents, non-public URLs, and unrecognized relationship roles.
4. Validate the produced ACORN DCAT value before serialization and retain record-index context in failures.

## 4. Make fixtures meaningful tests

1. Add one golden test per CKAN/DCAT stem. Parse the CKAN input, convert it, compare semantic output with the expected DCAT fixture, and validate the output.
2. Revise the expected fixtures to encode the agreed date, identity, agent-precedence, and relationship policies instead of preserving the earlier hand-conversion assumptions.
3. Add a fixture with at least one downloadable resource and one service/API resource; the current empty arrays do not exercise distributions.
4. Add cases for an Action API envelope, core-only CKAN, custom plugin fields, `extras`, HTML notes, missing public URL, private data, unknown relation type, malformed resource URL, and a batch of packages.
5. Assert warnings by field and loss type so intentional omissions remain visible.

## 5. Repair crosswalk loss enforcement

1. Make normal, dry-run, and strict paths use the same conversion-with-warnings result.
2. Ensure dry-run reports warnings without writing and strict mode fails after collecting warnings.
3. Add CLI tests for successful CKAN conversion, invalid CKAN input, dry-run, strict loss failure, output validation, source inference, and explicit `--from ckan`.
4. Remove or resolve the platform-ambiguous ignored crosswalk tests before using them as regression evidence.

## 6. Gate any DCAT → CKAN follow-up

Create a separate plan only when a consumer requires CKAN publishing. That plan must select a CKAN version and plugin profile, target `package_create` or `package_update`, define slug/organization/resource ownership, exclude server-managed response fields, and test against that request schema. If only a portable export file is required, do not add reverse conversion.

# Validation

Run focused checks during implementation, then the repository gates:

```sh
cargo nextest run --package acorn-lib ckan
cargo nextest run --package acorn-cli crosswalk
cargo run --bin acorn -- export tests/fixtures/schema/ckan/0sbdv-gh024.json --format json --from ckan --to dcat --output target/test_artifacts/ckan-crosswalk
cargo run --bin acorn -- check --standard dcat target/test_artifacts/ckan-crosswalk/0sbdv-gh024.json
make lint
make test
make book
```

# Acceptance criteria

1. `ckan` is a recognized explicit source standard and is inferred only from sufficiently distinctive CKAN package fields.
2. Each existing CKAN fixture drives a passing golden CKAN → DCAT conversion test; no fixture remains an unreferenced sample.
3. At least one resource-bearing fixture verifies Distribution and DataService policy.
4. Dataset dates are not populated from CKAN metadata-record timestamps unless the policy explicitly documents an approximation and emits a warning.
5. Unknown plugin fields and CKAN-only operational fields are either preserved through an approved extension or reported as conversion warnings.
6. Converted output passes the declared target contract's validation; documentation does not call flexible ACORN DCAT output DCAT-US-conformant unless the DCAT-US validator agrees.
7. Normal, dry-run, and strict modes observe the same warning set; dry-run writes nothing and strict fails on any loss warning.
8. No DCAT → CKAN support is advertised without a separately approved CKAN write-payload contract and tests.
