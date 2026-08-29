---
type: Plan
title: "Add a timestamped research activity changelog"
description: "Represent manually authored code, technical, personnel, publication, and general updates with progress states while preserving provenance for future dynamically discovered entries."
difficulty: high
impact: high
package: [acorn-lib, acorn-macros, acorn-cli]
tags: [feature, research-activity, schema, changelog, timeline, provenance, discovery, code, software, datacite, raid]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/schema/research_activity/{mod,markdown}.rs`, `src/validation/{mod,tests}.rs`, `src/analyzer/error.rs`, `crates/acorn-macros/src/lib.rs` and validation macro tests, linked-data contexts, prose/export renderers, standards mapping fixtures, generated schemas, and documentation; target `acorn-schema` after extraction.
- **Create:** `crates/acorn-lib/src/schema/research_activity/changelog.rs` and dedicated JSON, YAML, and Markdown compatibility fixtures.
- **Delete:** None. Existing RAD documents and prose fields remain valid.

# Dependencies

Coordinate the standards mappings with [RAD metadata standards alignment](rad-metadata-standards-alignment.md) and the distinction between activity dates and record timestamps with [research activity lifecycle, credit, license, and award fields](../refactors/core/research-activity-lifecycle-credit-fields.md). The [schema I/O validation handoff](../refactors/core/schema-io-validation-handoff.md) precedes module moves into `acorn-schema`.

# Goal

Add an optional `changelog` attribute to `ResearchActivity`, alongside `sections` and `notes`, containing timestamped developments in the activity. Preserve the whiteboard model's collection summary, ordered entries, update categories (`code`, `technical`, `personnel`, `publication`, and `general`), and progress states (`completed`, `in-progress`, and `upcoming`) while giving each concept one unambiguous schema role.

Manual authoring is the default and first implementation mechanism. The model also retains an explicit entry origin and optional discovery provenance so future provider workflows can propose dynamically discovered changelog entries without changing the canonical shape. Discovery produces candidates for review by default; it does not silently overwrite or reinterpret manual entries.

The changelog records domain developments. It is not a replayable stream of JSON patches, a complete audit trail of edits to the RAD document, or a replacement for canonical activity fields such as lifecycle dates, contributors, outputs, and status.

# Current evidence

- `ResearchActivity` contains authored `sections`, one contact, and `notes` but no `changelog` or ordered event collection (`crates/acorn-lib/src/schema/research_activity/mod.rs`).
- `ResearchActivityMetadata` has one current `Status` value (`active`, `paused`, `completed`, or `canceled`) and structured outputs, but it cannot say when a milestone occurred or distinguish completed, current, and upcoming update entries.
- The shared `Date` type represents a start/end interval at day granularity. It does not represent an instant and should not be reused for a timestamped entry.
- `EnrichmentProvenance` records which provider populated field paths; it does not record research developments or document revision history.
- The portable `Provider` enum already identifies OSTI, OpenAlex, CiteAs, ORCID, and ROR enrichment sources. A changelog discovery block can reuse that identity without coupling the schema to network access (`crates/acorn-lib/src/enrichment.rs`).
- DataCite 4.7 is designed for citation and discovery metadata rather than discipline-specific complete descriptions. Its repeatable Date property includes `Updated`, and its relation properties can connect separately identified versions or related outputs ([DataCite 4.7 overview](https://datacite-metadata-schema.readthedocs.io/en/4.7/), [Date](https://datacite-metadata-schema.readthedocs.io/en/4.7/properties/date/), [RelatedIdentifier](https://datacite-metadata-schema.readthedocs.io/en/4.7/properties/relatedidentifier/)).
- RAiD represents the activity interval, record-created and record-updated metadata, dated contributor and organization roles, and related objects. It does not define a general narrative event stream ([RAiD schema](https://metadata.raid.org/en/latest/), [activity dates](https://metadata.raid.org/en/latest/core/dates.html), [related objects](https://metadata.raid.org/en/latest/core/relatedObjects.html)).

# Recommendation

Use **changelog entry** for an individual development and **changelog** for the collection. This follows the requested serialized attribute name and avoids an additional alias:

```text
ResearchActivity
└── changelog (optional; beside sections and notes)
    ├── summary (optional)
    └── entries (one or more)
        ├── identifier (required, unique within the changelog)
        ├── timestamp (required RFC 3339 temporal anchor)
        ├── category (required controlled value)
        ├── state (required controlled value)
        ├── origin (manual by default; discovered for future workflows)
        ├── discovery (required provenance when origin is discovered)
        ├── summary (required short prose)
        └── items (optional list of supporting statements)
```

Model the Rust types as `Changelog`, `ChangelogEntry`, `ChangelogCategory`, `ChangelogState`, `ChangelogEntryOrigin`, and `ChangelogDiscovery`. Place `changelog: Option<Changelog>` directly on `ResearchActivity`, in the authored-content area beside `sections` and `notes`, rather than under `meta`, because it describes the activity rather than metadata about the RAD record.

Default `origin` to `manual` in builders and deserialization. A `discovered` entry must include `ChangelogDiscovery` with the provider, a non-empty provider-native source or lookup value, and discovery timestamp. A manual entry must not require discovery fields. This conditional rule keeps ordinary hand-authored records concise while making future automation attributable and reviewable.

Give every entry a required, stable `identifier` that is unique within its changelog. Use this identifier for edits, links, merges, conflict reports, and accepted discovery candidates. Do not derive identity from mutable prose, state, category, or timestamp. Manual authors choose a local kebab-case identifier; discovery adapters derive a deterministic identifier from the provider and provider-native source, then surface collisions as conflicts rather than silently merging entries.

Keep category and state orthogonal:

- category answers what changed: `code`, `technical`, `personnel`, `publication`, or `general`;
- state answers its horizon: `completed`, `in-progress`, or `upcoming`;
- the activity-wide `meta.status` continues to answer whether the research activity as a whole is active, paused, completed, or canceled.

Require an explicit UTC offset in `timestamp` and retain the supplied offset when serializing. Interpret the field with the entry state: completion time for `completed`, the time at which the statement is true for `in-progress`, and expected time for `upcoming`. This neutral name avoids claiming that a future development has already occurred. Sort only for derived displays; preserve serialized entry order so equal timestamps and editorial ordering round-trip deterministically. Add a separate `recordedAt` only when a concrete ingestion or audit workflow needs it; do not add two timestamps speculatively.

Use `code` for source-code changes, software releases, repository changes, computational notebooks, workflows, and related implementation developments. Use `technical` for broader scientific, engineering, infrastructure, data-processing, or methodological developments that are not specifically code changes. This explicit distinction prevents code updates from being hidden inside a catch-all technical category.

Treat `code` and `publication` entries as narrative announcements. Associated software, workflow, notebook, dataset, or publication metadata remains canonical in `meta.outputs`. Add typed related-resource references only after the standards-alignment relation primitive is approved, rather than embedding a second output model inside a changelog entry.

# Proposed Rust data model

The following is planning code, not an implementation patch. The proposed built-in `timestamp` and `nonempty` rules are described below; schema-level validation handles identifier uniqueness and contradictory manual/discovered provenance:

```rust
use crate::enrichment::Provider;
use crate::prelude::alloc::{String, Vec};
use crate::validation::{Validate, ValidationError};
use bon::Builder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_trim::{option_string_trim, string_trim};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "validate_changelog_identifiers", skip_on_field_errors = false))]
pub struct Changelog {
    /// Editorial overview of the changelog.
    #[validate(nonempty, length(max = 1000))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub summary: Option<String>,
    /// Ordered research activity developments.
    #[validate(nested, length(min = 1, max = 1000))]
    pub entries: Vec<ChangelogEntry>,
}

#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "validate_changelog_entry_origin", skip_on_field_errors = false))]
pub struct ChangelogEntry {
    /// Stable local identifier, unique within this changelog.
    #[validate(nonempty, kebabcase, length(max = 100))]
    #[serde(deserialize_with = "string_trim")]
    pub identifier: String,
    /// Temporal anchor interpreted according to `state`, including an explicit UTC offset.
    #[validate(timestamp)]
    pub timestamp: String,
    /// Subject of the development.
    pub category: ChangelogCategory,
    /// Whether the development is completed, underway, or anticipated.
    pub state: ChangelogState,
    /// How the entry entered the changelog.
    #[builder(default)]
    #[serde(default)]
    pub origin: ChangelogEntryOrigin,
    /// Provider provenance for a dynamically discovered entry.
    #[validate(nested)]
    pub discovery: Option<ChangelogDiscovery>,
    /// Concise description of the development.
    #[validate(nonempty, length(max = 500))]
    #[serde(deserialize_with = "string_trim")]
    pub summary: String,
    /// Supporting statements displayed beneath the summary.
    #[validate(length(min = 1, max = 20), each(nonempty), each(length(max = 1000)))]
    pub items: Option<Vec<String>>,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChangelogCategory {
    /// Source-code, software-release, repository, notebook, or workflow development.
    Code,
    /// Scientific, engineering, infrastructure, data-processing, or methodological development that is not specifically a code change.
    Technical,
    /// Team or contributor development.
    Personnel,
    /// Publication or other research-output development.
    Publication,
    /// Development not covered by another category.
    General,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChangelogState {
    /// Work represented by the entry is finished.
    Completed,
    /// Work represented by the entry is underway.
    InProgress,
    /// Work represented by the entry is anticipated.
    Upcoming,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChangelogEntryOrigin {
    /// Entry was authored directly by a person.
    #[default]
    Manual,
    /// Entry was proposed by a discovery provider.
    Discovered,
}

#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ChangelogDiscovery {
    /// Provider that proposed the entry.
    pub provider: Provider,
    /// Provider-native identifier, query, or URL used to discover it.
    #[validate(nonempty, length(max = 2048))]
    #[serde(deserialize_with = "string_trim")]
    pub source: String,
    /// When ACORN discovered the candidate, including an explicit UTC offset.
    #[validate(timestamp)]
    pub discovered_at: String,
}

fn validate_changelog_entry_origin(entry: &ChangelogEntry) -> core::result::Result<(), ValidationError> {
    match (entry.origin, entry.discovery.as_ref()) {
        (ChangelogEntryOrigin::Manual, None) | (ChangelogEntryOrigin::Discovered, Some(_)) => Ok(()),
        _ => Err(ValidationError::new("changelog_origin")
            .with_message("manual entries omit discovery; discovered entries require it".into())),
    }
}

fn validate_changelog_identifiers(changelog: &Changelog) -> core::result::Result<(), ValidationError> {
    let duplicate = changelog.entries.iter().enumerate().find_map(|(index, entry)| {
        let identifier = entry.identifier.trim();
        (!identifier.is_empty()
            && changelog.entries[..index]
            .iter()
            .any(|previous| previous.identifier.trim() == identifier))
            .then_some(identifier)
    });
    match duplicate {
        Some(identifier) => {
            let mut error = ValidationError::new("changelog_identifier")
                .with_message("changelog entry identifiers must be unique".into());
            error.add_param("identifier".into(), identifier);
            Err(error)
        }
        None => Ok(()),
    }
}
```

# Validation contract and built-in rules

Add two portable built-in rules to the validation derive rather than leaving reusable timestamp and text checks in the changelog module:

| Rule | Attribute | Contract | Stable error code |
|---|---|---|---|
| RFC 3339 instant | `#[validate(timestamp)]` | Accept a complete date and time with `Z` or a numeric UTC offset. Reject date-only values, intervals, time-zone annotations, missing offsets, and invalid calendar/time components. Parsing is deterministic and does not consult the clock or network. | `timestamp` |
| Nonempty text | `#[validate(nonempty)]` | Accept a string containing at least one non-whitespace Unicode scalar value. Do not mutate the value; serde trimming remains a normalization concern. Support use within `each(nonempty)`. | `nonempty` |

Implement both in `validation::rules`, add corresponding `ValidationRule` variants and derive expansion in `acorn-macros`, and map their codes to dedicated analyzer error kinds. Test direct fields, `Option<String>`, `each(...)`, custom message/code overrides, invalid macro syntax, default/alloc/WASM behavior, and deterministic validation paths.

Do not add generic `unique_by` or conditional-presence macro rules in this slice. Changelog identifier uniqueness and the `origin`/`discovery` relationship are cross-field domain invariants with no second demonstrated consumer, so the schema-level functions remain smaller and clearer.

The initial field constraints are:

| Path | Validation |
|---|---|
| `changelog.summary` | Optional; when present, nonempty and at most 1,000 bytes under the existing `length` rule. |
| `changelog.entries` | Between 1 and 1,000 entries; nested validation; identifiers unique within the collection. |
| `entries[].identifier` | Nonempty kebab-case; at most 100 bytes; immutable identity within the changelog. |
| `entries[].timestamp` | Built-in `timestamp`; explicit offset required. |
| `entries[].category` | Required closed enum: `code`, `technical`, `personnel`, `publication`, or `general`. |
| `entries[].state` | Required closed enum: `completed`, `in-progress`, or `upcoming`. |
| `entries[].origin` and `discovery` | `manual` requires no discovery block; `discovered` requires exactly one valid discovery block. |
| `entries[].summary` | Nonempty and at most 500 bytes. |
| `entries[].items` | Optional; when present, 1–20 items; every item nonempty and at most 1,000 bytes. |
| `entries[].discovery.provider` | Required `Provider` enum value. |
| `entries[].discovery.source` | Required, nonempty, and at most 2,048 bytes. It may be a provider identifier, query, or URL, so URL validation is not universally applicable. |
| `entries[].discovery.discoveredAt` | Built-in `timestamp`; explicit offset required. |

These are recommended initial resource limits. Verify them against representative authored and discovered fixtures before implementation freezes the contract. Increasing a limit is backward-compatible; reducing one later may invalidate existing records and requires an explicit migration decision.

Do not validate `timestamp` or `discoveredAt` against the current clock. In particular, do not require `upcoming` timestamps to be in the future, `completed` timestamps to be in the past, or `discoveredAt` to follow the entry timestamp: those rules would reject backfilled records, clock-skewed providers, and legitimately discovered future events. State/time plausibility may be reported later as a non-blocking analyzer check, not schema validity.

Schema validation must collect all independent field failures in declaration order. Cross-field validation runs with `skip_on_field_errors = false`, but it must avoid duplicating a field-level error. Duplicate-identifier diagnostics include the duplicated identifier as a structured parameter; origin/discovery errors resolve to the affected entry path through nested validation.

The corresponding addition to `ResearchActivity` is deliberately small:

```rust
/// Timestamped developments associated with the research activity.
#[validate(nested)]
#[eserde(compat)]
pub changelog: Option<Changelog>,
```

# DataCite and RAiD alignment impact

The changelog extends the canonical RAD activity profile but does not extend the DataCite record type. Manual and discovered entries use the same semantic mapping; `origin` and discovery provenance remain local to RAD:

| RAD value | DataCite Project export | RAiD export |
|---|---|---|
| Entry `timestamp` | No automatic mapping. It is an entry-specific temporal anchor whose meaning depends on state, not the update time of the registered DataCite resource or its metadata. Derive `dateType=Updated` only from a separately asserted canonical resource-update field. | Do not map to the RAiD activity start/end interval or registry metadata timestamp. |
| Changelog-level `summary` | Optionally derive one `Description` with `descriptionType=Other` under an explicit export policy. | Map only if the approved RAiD description profile has a compatible, non-duplicative description type. |
| Entry category, state, summary, and items | No direct mapping. Emit structured loss information instead of flattening every entry into DataCite descriptions. | No direct core mapping. Keep as RAD-local authored content. |
| Entry origin and discovery provenance | No direct mapping. Keep local and report as intentionally local rather than descriptive loss. | No direct core mapping. Keep as RAD provenance. |
| `code` or `publication` entry associated with an identified output | After the shared related-resource primitive lands, map the canonical output through `RelatedIdentifier` or `RelatedItem`, using its DOI, SWHID, or other supported identifier and appropriate resource type or relation. Keep changelog prose local. | Map a PID-bearing output through `relatedObject`; do not convert the narrative entry itself into a related object. |
| Separately identified activity snapshot | Use `Version` and `HasVersion`/`IsVersionOf` or `IsNewVersionOf`/`IsPreviousVersionOf` as appropriate. | Relate separately identified activities or objects through the corresponding RAiD relation. |

DataCite 4.7 adds RAiD and SWHID as controlled `relatedIdentifierType` values plus `relationTypeInformation`, so the broader alignment plan must move from 4.6 to 4.7 before implementing this crosswalk ([4.7 update](https://datacite-metadata-schema.readthedocs.io/en/4.7/introduction/version-update/)). A new DOI is not required for every update. If ACORN later supports citable snapshots, minting and retention policy must be a separate repository/depositor concern; DataCite's dynamic-data guidance distinguishes continuously updated resources from identified snapshots and subsets ([dynamic dataset guidance](https://datacite-metadata-schema.readthedocs.io/en/4.7/guidance/dynamic-datasets/)).

# Implementation plan

## 1. Confirm the semantic contract

- Confirm `changelog`/`ChangelogEntry` as the serialized field and Rust naming pair.
- Confirm that entries are domain developments rather than machine-generated field mutations.
- Confirm manual entry creation as the default workflow and discovery as an attributable candidate-producing extension.
- Confirm stable, changelog-local entry identifiers and deterministic collision behavior.
- Approve the required category and state vocabularies, their definitions, aliases, and extension policy.
- Decide whether timestamps must always be instants or whether date-only historical entries are required. Prefer one required RFC 3339 representation for the first slice.

Deliverable: a reviewed field matrix with examples and explicit non-goals.

## 2. Add the portable schema types

- Add built-in `timestamp` and `nonempty` validation rules, derive-macro parsing and expansion, analyzer error-code mappings, unit tests, and compile-fail UI tests.
- Create the changelog types in a dedicated leaf module.
- Require a non-empty entry summary, category, state, a unique kebab-case identifier for every entry, and at least one entry when `changelog` is present.
- Apply the built-in `timestamp` rule so changelog timestamps require an explicit offset and reject the date-only and interval forms supported by the broader legacy helper.
- Use `serde(deny_unknown_fields)`, camel-case serialization, nested validation, builders, JSON Schema, and complete public documentation consistent with adjacent RAD types.
- Add `changelog: Option<Changelog>` to `ResearchActivity` beside the authored content fields and add its JSON-LD context without changing defaults for existing records.
- Define `changelog` and its child terms in the ACORN JSON-LD namespace unless the approved field matrix identifies an exact external vocabulary term; do not imply DataCite ownership in canonical linked data.
- Default `ChangelogEntryOrigin` to `Manual` and validate the `Discovered`/`ChangelogDiscovery` conditional relationship.

Deliverable: old fixtures deserialize unchanged and representative logs round-trip without reordering.

## 3. Define authoring and rendering behavior

- Make direct JSON, YAML, and supported Markdown editing the initial changelog-entry mechanism.
- Extend Markdown parsing only if a compact, unambiguous representation can preserve timestamps, categories, states, summaries, and items. Otherwise keep the first slice JSON/YAML-only and render the log read-only in Markdown/prose.
- Render entries newest-first for public timelines unless a target explicitly requires chronological order.
- Use accessible headings, visible dates, and text labels; do not encode category or state by color alone.
- Keep log summary prose separate from the activity subtitle and section summaries.

Deliverable: stable prose and artifact snapshots with accessible ordering and labels.

## 4. Preserve a dynamic-discovery extension point

- Define a pure provider-to-candidate conversion that returns proposed `ChangelogEntry` values without mutating `ResearchActivity`.
- Reuse the portable `Provider` identity and require a non-empty provider-native source or lookup value plus `discoveredAt` timestamp.
- Require explicit acceptance before a discovered candidate is appended in the default workflow.
- Derive a deterministic candidate identifier from provider plus source. Treat an identifier collision with different populated values as a conflict, and never replace a manual entry automatically.
- Keep network access and provider-specific matching outside the portable schema module and honor offline mode.

Deliverable: a future discovery adapter can create attributable candidates without changing the serialized changelog contract or weakening manual ownership.

## 5. Integrate standards adapters

- Add every changelog field to the RAD/RAiD/DataCite field matrix as exact, transformed, lossy, or local-only.
- For opt-in `ResearchActivity -> DataCite Project`, map only the compatible aggregate values in the table above.
- Preserve software, workflow, notebook, and publication metadata in `ResearchOutput <-> DataCite`; a changelog entry may point to that output but does not become the output.
- Ensure every omitted entry produces one structured conversion warning at the narrowest useful source path.
- Do not infer RAiD lifecycle dates, contributor periods, or organization periods from narrative entries.

Deliverable: adapters explain all projection losses and never manufacture registry assertions.

## 6. Verify compatibility and behavior

- Add valid and invalid timestamp, blank text, length boundary, list cardinality, identifier, duplicate-identifier, empty-entry-list, every category including `code`, state, manual-default, and discovered-provenance tests adjacent to the new module.
- Test `timestamp` with `Z`, colon and compact numeric offsets, fractional seconds, date-only values, intervals, missing offsets, named-zone annotations, invalid calendar values, and alloc/WASM builds; test `nonempty` with ASCII and Unicode whitespace plus non-ASCII visible text.
- Assert multi-error reports contain every independent failure with stable codes, paths, parameters, and declaration order.
- Add JSON/YAML round-trip tests and one representative whiteboard-derived fixture.
- Add compatibility tests proving records without `changelog` retain their serialized and rendered behavior.
- Add candidate acceptance, rejection, deduplication, and manual-entry-preservation tests before enabling a discovery provider.
- Add ordering tests for equal timestamps and offset-bearing timestamps.
- Add DataCite Project and RAiD loss-reporting golden tests.
- Regenerate schemas and documentation where checked-in output changes.
- Run focused schema tests, then `cargo fmt --all`, `make lint`, `make test`, and `make wasm`.

# Acceptance criteria

- An optional `changelog` attribute appears on `ResearchActivity` beside `sections` and `notes` and contains a summary plus one or more timestamped entries without affecting existing RAD input.
- Manual authoring is the default entry mechanism and serializes with `origin=manual` when the field is emitted.
- A discovered entry is attributable to a provider and a non-empty provider-native source, and discovery produces a reviewable candidate without silently altering manual entries.
- Every entry has a stable kebab-case identifier unique within its changelog; merge and discovery conflicts are reported against that identifier.
- Changelog validation reports all independent failures with stable field/index paths rather than stopping at the first invalid entry.
- Category, update state, activity-wide status, lifecycle dates, and record metadata timestamps have distinct documented meanings.
- Every entry has an explicit state, so its temporal anchor is interpreted consistently as completed, current-as-of, or expected.
- Entry timestamps round-trip with an explicit offset and deterministic order.
- Entry temporal anchors are not exported as DataCite `Updated` dates without a separate canonical resource-update assertion.
- Code and publication entries do not duplicate or override canonical `ResearchOutput` metadata.
- The controlled category vocabulary includes `code`, and its documented meaning is distinct from `technical`.
- JSON Schema exposes controlled category and state values plus nested validation.
- Built-in `timestamp` and `nonempty` validators are documented, portable, usable through `each(...)` where applicable, and covered by derive UI tests.
- Markdown/prose and export rendering is accessible and deterministic wherever the log is supported.
- Every DataCite or RAiD field has an explicit mapping classification, and every omitted entry is visible through structured loss reporting.
- DataCite Project export does not convert every entry into a new DOI, version, or free-text description.
- Default, minimal, alloc, and WASM builds remain supported with no network-dependent validation.

# Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| “Update” is interpreted as a JSON mutation. | High | State that entries are authored domain developments and reserve patch/audit semantics for a separate design. |
| Discovery weakens manual editorial control. | High | Keep manual as the default, generate candidates purely, and require acceptance before append. |
| Discovered claims cannot be traced to their source. | High | Require provider and discovery provenance for every `discovered` entry. |
| Mutable prose or timestamps are used as entry identity. | High | Require a stable changelog-local identifier and report conflicting reuse instead of guessing. |
| Entry state conflicts with activity-wide status. | High | Use separate enums and document the question each answers. |
| Publication facts acquire two sources of truth. | High | Keep output metadata in `ResearchOutput`; log entries contain narrative and later typed references only. |
| Code updates duplicate software release or repository metadata. | High | Keep software identity, version, identifiers, and relations in `ResearchOutput`; changelog entries provide narrative context and typed references only. |
| DataCite export silently flattens or loses the timeline. | High | Keep the timeline RAD-local, map only compatible aggregate values, and emit path-aware loss warnings. |
| JSON-LD implies a false standards equivalence. | Medium | Use ACORN-owned changelog terms until the reviewed field matrix approves an exact external term. |
| A fixed category list becomes too narrow. | Medium | Begin with the evidenced categories, define `general` clearly, and add values only with examples and mappings. |
| Timestamps create unstable display order. | Medium | Preserve source order for storage and define timestamp plus source position as the derived-display key. |

# Related concepts

- [RAD metadata standards alignment](rad-metadata-standards-alignment.md)
- [Research activity lifecycle, credit, license, and award fields](../refactors/core/research-activity-lifecycle-credit-fields.md)
- [Schema I/O and validation handoff](../refactors/core/schema-io-validation-handoff.md)
