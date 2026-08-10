---
type: Plan
title: "Add research activity lifecycle, credit, license, and award fields"
description: "Give provider metadata dedicated schema targets instead of storing dates, contributors, licenses, and award identifiers in notes."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [schema, research-activity, enrichment, provenance]
status: planned
---

# Goal

Represent activity dates, credited contributors, licenses, and funding award identifiers as typed research activity data so provider enrichment remains lossless and queryable.

# Current evidence

- `crates/acorn-lib/src/schema/research_activity/mod.rs` has no dedicated activity lifecycle date, contributor list, license, or funding award field.
- `crates/acorn-lib/src/io/api/osti.rs` receives `release_date`, developers, contributors, and sponsor `primary_award` values.
- `crates/acorn-lib/src/schema/standard/cff.rs` receives release dates, contacts, authors, versions, and SPDX-compatible licenses.
- `crates/acorn-lib/src/schema/pid/raid.rs` receives typed activity dates, ORCID contributors with positions and CRediT roles, and access metadata.
- `crates/acorn-lib/src/analyzer/discovery/mod.rs` persists provider descriptions as free-form notes but deliberately leaves lifecycle, credit, license, and award values in provenance because no lossless schema target exists.

# Recommendation

Add optional, backward-compatible research activity metadata fields for:

- an activity lifecycle interval using the existing shared `Date` type;
- contributors with stable identifiers, names, optional contact data, administrative positions, and CRediT roles, distinct from the single operational contact point;
- license identifiers and URLs, normalized through the existing SPDX vocabulary where possible;
- funding awards with an award identifier and optional sponsoring organization reference.

Define merge identity and conflict behavior for every new type before connecting providers. Map only explicitly asserted source values: RAiD contributor/contact flags and organization roles, CFF authors/contact/license/date fields, and OSTI release dates, credited people, and primary awards. Keep raw provider values in provenance and do not redirect unsupported values into `notes`.

# Validation

Add adjacent schema round-trip and validation tests, candidate merge tests, and fixture-backed OSTI, CFF, and RAiD mappings. Regenerate JSON schemas and command documentation where their checked-in output changes. Run `make lint`, `make test`, and the portable WASM check because the new domain types belong to `acorn-lib`.

# Acceptance criteria

- Every new field is optional and existing research activity documents deserialize unchanged.
- Lifecycle dates use ISO-compatible validation without conflating metadata-record timestamps with activity dates.
- Contributors retain persistent identifiers and typed roles without being inferred from list position.
- Licenses preserve recognized SPDX identifiers and retain an explicit URL for non-standard terms.
- Funding awards preserve their source identifier and sponsoring-organization relationship.
- Candidate merging deduplicates structured values by documented stable keys and records incompatible populated values as provenance conflicts.
- OSTI, CFF, and RAiD fixtures demonstrate lossless mappings into the new fields.
- Dates, contributors, licenses, and award identifiers are no longer placed in free-form notes.
