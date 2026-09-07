---
type: Plan
title: "Add a timestamped research activity logbook with since-filtered updates export"
description: "Represent manually authored time-coded personnel, code, technical, publication, and general developments in a logbook that augments declarative meta/contact data, graduates via archive (not delete) into meta/contact, and supports since-timestamp-filtered updates artifacts (PDF/MD/DOCX) plus MCP tools; reserve nanopublication references for a future slice."
difficulty: high
impact: high
package: [acorn-lib, acorn-macros, acorn-cli]
tags: [feature, research-activity, schema, logbook, updates, provenance, discovery, code, software, datacite, raid, mcp, nanopublication]
status: planned
priority: next
---

# Project code boundaries

- **Update:** `crates/acorn-schema/src/schema/research_activity/{mod,markdown}.rs`, `crates/acorn-schema/src/schema/{mod,validate/{mod,tests}.rs}`, `src/analyzer/error.rs`, `crates/acorn-macros/src/lib.rs` and validation macro tests, linked-data contexts, prose/export renderers (`crates/acorn-cli/src/template/mod.rs`, `crates/acorn-lib/src/io/powerpoint/mod.rs`), standards mapping fixtures, `crates/acorn-lib/src/io/mcp/{mod,tools}.rs`, `crates/acorn-cli/src/cli/arguments.rs`, `crates/acorn-cli/src/commands/export/{mod,pdf}.rs`, generated schemas and documentation; target `acorn-schema` after extraction.
- **Create:** `crates/acorn-schema/src/schema/research_activity/logbook.rs`, `crates/acorn-cli/src/commands/export/updates.rs` (or `logbook_updates.rs`), dedicated JSON/YAML/Markdown compatibility fixtures, `tests/fixtures/schema/logbook/` and whiteboard-derived fixture, MCP goldens for `export_research_activity_updates` / `propose|apply_logbook_graduation`.
- **Delete:** None. Existing RAD documents and prose fields remain valid.

# Dependencies

Coordinate the standards mappings with [RAD metadata standards alignment](rad-metadata-standards-alignment.md) and the distinction between activity dates and record timestamps with [research activity lifecycle, credit, license, and award fields](../refactors/core/research-activity-lifecycle-credit-fields.md). The built-in `timestamp` and `nonempty` rules, derive expansion, and analyzer mappings are implemented; this plan consumes them without re-adding them. The `Provider` reuse for discovery provenance (`crates/acorn-schema/src/enrichment.rs`) and the `Target::Updates` addition serialize behind the schema-features lane lock (`ResearchActivity`/contexts/renderers/fixtures) and CLI contract lock.

# Goal

Add an optional `logbook` attribute to `ResearchActivity`, alongside `sections` and `notes`, containing **time-coded** developments in the activity. The primary theme of `logbook` is temporal: every entry carries a required RFC 3339 instant with explicit offset (`timestamp`) whose meaning is interpreted via `state` (`completed`/`in-progress`/`upcoming`). Non-temporal declarative truth remains in `meta`/`contact`/`sections` (contributors, organizations, funders, typed dates, outputs, contact point, etc.); `logbook` augments what cannot yet be integrated into static data. Track document-level edits like `title` changes via git, not logbook.

Preserve the whiteboard model's collection summary, ordered entries, update categories (`code`, `technical`, `personnel`, `publication`, `general`), and progress states while giving each concept one unambiguous schema role. The example workflow — `personnel` logbook entry added for a new contributor → optionally enriched → graduate to `meta.contributors` via archive (entry retained, not deleted) → generate a separate `updates` artifact (PDF/MD/DOCX) filtered `since <timestamp>` for month-to-month reporting — must be supported end-to-end.

Manual authoring is the default. The model retains explicit `origin` and optional `discovery` provenance so future provider workflows can propose dynamically discovered logbook entries without changing the canonical shape. Discovery produces candidates for review; it does not silently overwrite manual entries.

The logbook records domain developments. It is not a replayable stream of JSON patches, a complete audit trail of edits to the RAD document, or a replacement for canonical activity fields. Future nanopublication ingest will reference Trusty-URI nanopubs from `logbook` entries via typed related resources; this slice reserves the extension point without implementing it.

# Current evidence

- `ResearchActivity` contains authored `sections`, one contact, and `notes` but no `logbook` or ordered event collection (`crates/acorn-schema/src/schema/research_activity/mod.rs`).
- `ResearchActivityMetadata` has one current `Status` (`active`/`paused`/`completed`/`canceled`) and structured outputs, but cannot say when a milestone occurred or distinguish completed/current/upcoming entries.
- The shared `Date` type represents a start/end interval at day granularity, not an instant; `timestamp` built-in handles instants.
- `EnrichmentProvenance` records provider field paths under `ResearchActivityMetadata::enrichment`; it does not record research developments or revision history.
- Portable `Provider` enum identifies OSTI, OpenAlex, CiteAs, ORCID, ROR enrichment sources (`crates/acorn-schema/src/enrichment.rs`). A logbook discovery block can reuse that identity without network coupling; do not conflate with `Provider` in `crates/acorn-schema/src/schema/agent/mod.rs`.
- Export plumbing: `FileFormat`/`Target` (`crates/acorn-cli/src/cli/arguments.rs:201/294`), `ExportHandler` `PdfExportHandler`/`PowerPointExportHandler`/`StructuredExportHandler` (`crates/acorn-cli/src/commands/export/mod.rs`), `ResearchActivity::serialize_as`/`to_html` (`crates/acorn-cli/src/commands/export/pdf.rs:51`).
- MCP registry `ToolRegistry::acorn()` (`crates/acorn-lib/src/io/mcp/tools.rs:207`) currently exposes `acorn.export_research_activity_data` (`ExportInput:72` -> `ExportOutput:77`) for whole-RAD serialization, `acorn.format_and_enrich_research_activity_data`, and `validate`/`check`/`crosswalk`; no since-filtered updates or graduation tools exist yet.
- DataCite 4.7 / RAiD have no general narrative event stream; `Updated` and activity interval must not be inferred from logbook timestamps.

# Recommendation

Use **logbook entry** for an individual development and **logbook** for the collection. `logbook` replaces `changelog` to avoid software-diff connotation and `Journal`/`Invenio::Journal` collision; `highlights` remains the existing export `Target::Highlight` view and is not reused as a field name.

```text
ResearchActivity
└── logbook (optional; beside sections and notes)
    ├── summary (optional)
    └── entries (one or more)
        ├── identifier (required, unique within the logbook)
        ├── timestamp (required RFC 3339 temporal anchor, explicit offset)
        ├── category (required: code|technical|personnel|publication|general)
        ├── state (required: completed|in-progress|upcoming)
        ├── origin (manual default; discovered for future workflows)
        ├── discovery (required when origin=discovered)
        ├── summary (required short prose)
        └── items (optional supporting statements)
```

Model the Rust types as `Logbook`, `LogbookEntry`, `LogbookCategory`, `LogbookState`, `LogbookEntryOrigin`, `LogbookDiscovery`. Place `logbook: Option<Logbook>` directly on `ResearchActivity` in the authored-content area beside `sections` and `notes`, not under `meta`.

Default `origin` to `manual`. A `discovered` entry must include `LogbookDiscovery` with non-empty provider-native `source` and `discovered_at` timestamp; manual entries omit it. Deterministic candidate `identifier` from `provider+source`; collisions are conflicts, never silent merges. Category and `state` remain orthogonal to `meta.status` (activity-wide lifecycle) and to `typed_dates`/dated roles.

Require explicit UTC offset in `timestamp`, retain supplied offset, sort only for derived displays, preserve source order for round-tripping. `code`=repo/notebook/workflow/release, `technical`=methods/infra not code. `code`/`publication` entries are narrative pointers; canonical software/publication identity stays in `meta.outputs` (typed related resource only after that primitive is approved).

## Archive graduation (not delete)

Pure, offline functions `propose_logbook_graduation(&ResearchActivity)->Vec<GraduationCandidate>` and `apply_logbook_graduation(activity, accepted_ids)->activity`:
- Scope: only `meta` + `contact` fields (`contributors`/`organizations`/`funders`/`related_resources`/`languages`/`subjects`/`spatial_coverage`/`rights`/`access`/`typed_dates`/`websites`/`media`/`technology`/`partners`/`resources` + `ContactPoint` normalization). `title`/`subtitle`/`sections` changes remain git-tracked, not logbook-graduable.
- `apply` copies accepted entries into declarative fields and **archives** the logbook entry (retains it, optionally normalizes `state` to `completed`). Future `since` windows naturally exclude it when `timestamp < since`; no deletion, full audit history preserved.
- Honor `offline` and keep network/matching outside portable schema.

## Since-filtered updates artifact

New separate export type `Target::Updates` (distinct from `FactSheet`/`Highlight`/`Poster`): `acorn export --target updates --since <timestamp> --format pdf|markdown|docx --output ./export` generates a standalone file `/{identifier}-updates-{since}.pdf|md|docx` containing only `logbook.entries` where `timestamp >= since` (offset-aware) plus declarative header. Month-to-month reporting is consumer-side comparison of two artifacts with different `since` values; no git diff required this slice. Handler siblings `PdfExportHandler`/`StructuredExportHandler`; DOCX via new `DocxExportHandler` (HTML->DOCX or `anydoc`), reuse `ChartOptions`/`to_html` where applicable.

## MCP tools output

Extend `crates/acorn-lib/src/io/mcp/tools.rs:207` deterministically (name-ordered):
- `acorn.export_research_activity_updates` `UpdatesExportInput {activity, since: String (timestamp), format: UpdatesExportFormat(Pdf|Markdown|Docx)} -> UpdatesExportOutput {content, since, format, entry_count, archived_count}` — read-only, `mcp:true`.
- `acorn.propose_logbook_graduation` (read-only, pure) and `acorn.apply_logbook_graduation` (requires `allow_mutation=true`; MCP can expose propose-only while Needle exposes apply). Publish Needle catalog via `commands/export/mod.rs:36` `tools()` and update goldens. `validate`/`check`/`crosswalk` require no schema change beyond new `logbook` validation errors.

## Nanopublication future ingest

Reserved, not implemented this slice: `LogbookEntry` will gain `references: Option<Vec<TypedRelatedResource>>` (`crates/acorn-schema/src/schema/research_activity/common.rs:264`) to point at nanopub Trusty URIs (`http://purl.org/np/…`) with `relation_type=References|Other` and `relation_type_information` for `npx:retracts` semantics. Keep `#[serde(deny_unknown_fields, rename_all="camelCase")]` with future `#[eserde(compat)]` on the field; do not add `Provider::Nanopub` variant or ingest handler now. Document placeholder row in `standards-field-matrix.md` as `local-only` until DataCite `relatedIdentifierType` supports nanopubs.

# Proposed Rust data model

```rust
use crate::enrichment::Provider;
use crate::validation::{Validate, ValidationError};
use acorn_core::prelude::alloc::{String, Vec};
use bon::Builder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_trim::{option_string_trim, string_trim};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "validate_logbook_identifiers", skip_on_field_errors = false))]
pub struct Logbook {
    /// Editorial overview of the logbook.
    #[validate(nonempty, length(max = 1000))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub summary: Option<String>,
    /// Ordered time-coded developments. Source order preserved; displays sort newest-first.
    #[validate(nested, length(min = 1, max = 1000))]
    pub entries: Vec<LogbookEntry>,
}

#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "validate_logbook_entry_origin", skip_on_field_errors = false))]
pub struct LogbookEntry {
    /// Stable local identifier, unique within this logbook.
    #[validate(nonempty, kebabcase, length(max = 100))]
    #[serde(deserialize_with = "string_trim")]
    pub identifier: String,
    /// Temporal anchor interpreted per `state`, with explicit UTC offset.
    #[validate(timestamp)]
    pub timestamp: String,
    /// Subject of the development.
    pub category: LogbookCategory,
    /// Horizon: completed (when finished), in-progress (true-as-of), upcoming (expected).
    pub state: LogbookState,
    /// How the entry entered the logbook.
    #[builder(default)]
    #[serde(default)]
    pub origin: LogbookEntryOrigin,
    /// Provenance for discovered entries.
    #[validate(nested)]
    pub discovery: Option<LogbookDiscovery>,
    /// Concise description.
    #[validate(nonempty, length(max = 500))]
    #[serde(deserialize_with = "string_trim")]
    pub summary: String,
    /// Supporting statements.
    #[validate(length(min = 1, max = 20), each(nonempty), each(length(max = 1000)))]
    pub items: Option<Vec<String>>,
    // Future: nanopublication references (reserved)
    // #[validate(nested)]
    // pub references: Option<Vec<TypedRelatedResource>>,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LogbookCategory { Code, Technical, Personnel, Publication, General, }

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LogbookState { Completed, InProgress, Upcoming, }

#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LogbookEntryOrigin { #[default] Manual, Discovered, }

#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LogbookDiscovery {
    pub provider: Provider,
    #[validate(nonempty, length(max = 2048))]
    #[serde(deserialize_with = "string_trim")]
    pub source: String,
    #[validate(timestamp)]
    pub discovered_at: String,
}

fn validate_logbook_entry_origin(entry: &LogbookEntry) -> core::result::Result<(), ValidationError> {
    match (entry.origin, entry.discovery.as_ref()) {
        (LogbookEntryOrigin::Manual, None) | (LogbookEntryOrigin::Discovered, Some(_)) => Ok(()),
        _ => Err(ValidationError::new("logbook_origin").with_message("manual entries omit discovery; discovered entries require it".into())),
    }
}
fn validate_logbook_identifiers(logbook: &Logbook) -> core::result::Result<(), ValidationError> {
    let duplicate = logbook.entries.iter().enumerate().find_map(|(index, entry)| {
        let identifier = entry.identifier.trim();
        (!identifier.is_empty() && logbook.entries[..index].iter().any(|p| p.identifier.trim() == identifier)).then_some(identifier)
    });
    match duplicate {
        Some(id) => { let mut e = ValidationError::new("logbook_identifier").with_message("logbook entry identifiers must be unique".into()); e.add_param("identifier".into(), &id); Err(e) },
        None => Ok(()),
    }
}
```

Addition to `ResearchActivity`:
```rust
/// Time-coded developments augmenting declarative meta/contact/sections. Archived entries retained.
/// Distinct from typed_dates/dated roles/status/outputs; see field-matrix Validity Levels.
#[validate(nested)]
#[eserde(compat)]
pub logbook: Option<Logbook>,
```

# Validation contract

| Rule | Attribute | Contract | Code |
|---|---|---|---|
| RFC 3339 instant | `#[validate(timestamp)]` | Complete date+time with `Z`/offset; reject date-only/interval/zone-name/missing-offset | `timestamp` |
| Nonempty text | `#[validate(nonempty)]` | ≥1 non-whitespace scalar; trimming is serde concern | `nonempty` |

Constraints:
| Path | Validation |
|---|---|
| `logbook.summary` | nonempty ≤1000 |
| `logbook.entries` | 1..1000, nested, identifiers unique |
| `entries[].identifier` | nonempty kebab ≤100 |
| `entries[].timestamp` | `timestamp` with offset |
| `entries[].category` | closed enum `code|technical|personnel|publication|general` |
| `entries[].state` | `completed|in-progress|upcoming` |
| `entries[].origin/discovery` | `manual`↔None, `discovered`↔Some(valid) |
| `entries[].summary` | nonempty ≤500 |
| `entries[].items` | 1..20 each nonempty ≤1000 |
| `entries[].discovery.source` | nonempty ≤2048 |
| `entries[].discovery.discoveredAt` | `timestamp` |

Do not clock-validate `timestamp`/`discoveredAt`; state plausibility is an analyzer check, not schema.

# DataCite and RAiD alignment impact

| RAD value | DataCite | RAiD |
|---|---|---|
| `timestamp` | No auto `dateType=Updated`; only from separate canonical update field | Do not map to activity interval/registry metadata |
| `logbook.summary` | Optionally `Description Other` | Only if RAiD description profile compatible |
| `category/state/summary/items/origin/discovery` | No direct mapping — emit loss warning `local-only` | Keep RAD-local |
| `code|publication` with output | Map canonical `meta.outputs` via `RelatedIdentifier/RelatedItem` (RAiD `relatedObject`) — keep prose local | — |
| Nanopub reference (future `references`) | Map via `RelatedIdentifier` with `relationType=References`, `relatedIdentifierType` pending nanopub | `relatedObject` with trusty URI |
| Snapshot version | `Version` + `HasVersion` semantics | Corresponding relation |

# Implementation plan

## 1. Confirm semantic contract
- Confirm `logbook`/`LogbookEntry` naming, time-coded vs declarative split (`meta`/`contact` canonical, logbook augments), archive-not-delete, `since` filtering, scope `meta`+`contact` only.
- Approve categories/states and that future `references` will hold nanopub URIs via `TypedRelatedResource`.
- Field matrix + non-goals reviewed.

## 2. Add portable schema types
- Create `logbook.rs`, wire `logbook: Option<Logbook>` + `ResearchActivityContext.logbook` in ACORN JSON-LD namespace, builders, `Doc("Time-coded developments … distinct from typed_dates …")`.

## 3. Archive graduation
- Implement `propose_logbook_graduation` / `apply_logbook_graduation` pure fns, deterministic discovered identifiers, conflict reporting, offline.

## 4. Since-filtered updates export
- Add `Target::Updates` + `FileFormat::Docx` (where applicable), `UpdatesExportHandler` + CLI `--since` (timestamp), standalone PDF/MD/DOCX outputs, `since` goldens.
- Extend `ToolRegistry::acorn()` with `acorn.export_research_activity_updates` and graduation tools (propose read-only, apply gated), update Needle catalog `tools.json`.

## 5. Authoring & rendering
- JSON/YAML direct editing first; Markdown read-only rendering initially (or compact unambiguous representation). Render newest-first, accessible labels, keep summary separate from subtitle.

## 6. Discovery extension point + nanopub reserve
- Pure provider->candidate, `Provider` reuse, explicit acceptance, deterministic id. Add doc placeholder for nanopub `references` and field-matrix row without adding the field.

## 7. Standards adapters
- Classify every logbook field `local-only` except optional aggregate `summary`; never infer lifecycle dates from entries; emit path-aware loss warnings.

## 8. Verify compatibility and behavior
- Tombstone timestamp/blank/length/cardinality/identifier/duplicate/empty-list/category/state/origin/discovery tests; WASM/alloc; multi-error ordered; JSON/YAML round-trip; whiteboard fixture; compatibility (no `logbook` unchanged); graduation archive/dedup; since filter (offset/comparison/empty); MCP `since` and output-size (1MiB) tests; DataCite/RAiD loss goldens.
- `cargo fmt --all && make lint && make test && make check-wasm`; regenerate schemas/help/docs.

# Acceptance criteria

- `ResearchActivity.logbook` time-coded, `timestamp` offset required, `state`-dependent interpretation.
- `meta`/`contact` remain canonical; `logbook` augments; title/section edits tracked by git, not logbook.
- Personnel example graduates to `meta.contributors` via archive (entry retained); declarative delta + logbook slice report via `since`.
- `Target::Updates` generates separate PDF/MD/DOCX `since` file; empty and offset cases handled.
- MCP `export_research_activity_updates` + `propose|apply_logbook_graduation` exposed deterministically; existing `validate`/`check` report `logbook_*` errors.
- Discovery attributable (`provider`+`source`+`discovered_at`), deterministic id, conflict not overwrite, offline honored.
- Category `code` distinct from `technical`; no DataCite `Updated` inference; JSON-LD ACORN-owned.
- Nanopub reference reserved as `TypedRelatedResource` URI, not implemented, documented as future.

# Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Update = JSON patch | High | Define as domain development; archive graduation separate from audit |
| Discovery weakens control | High | Manual default, pure candidates, explicit acceptance |
| Untraceable claims | High | Require `LogbookDiscovery` for `discovered` |
| Mutable id | High | Stable `identifier`, collision conflict |
| State vs status conflation | High | Separate enums/doc |
| Two publication truths | High | Keep `meta.outputs` canonical; logbook narrative only |
| Flatten to DataCite | High | Local-only + loss warnings |
| JSON-LD false equivalence | Medium | ACORN namespace |
| Nanopub duplication | Medium | Reference via `TypedRelatedResource` URI, not embedded assertion |
| Fixed categories narrow | Medium | `general` + extension policy |
| Timestamp ordering unstable | Medium | Preserve source order, derived newest-first key |

# Related concepts

- [RAD metadata standards alignment](rad-metadata-standards-alignment.md)
- [Research activity lifecycle, credit, license, and award fields](../refactors/core/research-activity-lifecycle-credit-fields.md)
