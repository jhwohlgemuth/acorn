---
type: Plan
title: "Complete structured research-activity enrichment"
description: "Map provider and logbook evidence into the existing activity date, contributor, rights, and funding fields with deterministic merge and conflict behavior."
difficulty: medium
impact: high
package: [acorn-schema, acorn-lib]
tags: [schema, research-activity, enrichment, provenance, logbook]
status: planned
---

# Goal

Use the structured activity metadata already owned by `acorn-schema` as the destination for provider enrichment and accepted logbook evidence. Preserve source provenance, deterministic merge identity, and explicit conflicts instead of copying supported dates, contributors, rights, or awards into free-form notes.

# Current evidence

- `ResearchActivityMetadata` already exposes `typed_dates`, `contributors`, `funders`, `rights`, and `access` in `crates/acorn-schema/src/research_activity/mod.rs`.
- The corresponding portable types and validation live in `crates/acorn-schema/src/research_activity/common.rs` and reuse output funding types where appropriate.
- RAiD and CFF adapters expose activity dates, credited people, roles, and rights assertions; `crates/acorn-lib/src/io/api/osti.rs` exposes release dates, developers, contributors, and sponsor award identifiers.
- Discovery and candidate persistence retain provider evidence, but they do not yet project every supported assertion into the structured activity fields with one documented merge policy.
- `GraduationCandidates` currently proposes only administration classification and personnel contributors. It has no source-category rules for dates, rights, or funding.

# Project code boundaries

- **Update:** research-activity logbook rules and tests in `crates/acorn-schema/src/research_activity/`; provider/discovery and candidate merge adapters in `crates/acorn-lib/src/io/`; focused OSTI, CFF, and RAiD fixtures and tests.
- **Create:** dedicated merge-policy helpers only where at least two adapters need the same stable-key or conflict behavior.
- **Delete:** note-only enrichment fallbacks after the same assertion and its provenance are retained structurally.

# Dependencies

Preserve the implemented [standards field matrix](../../assets/standards-field-matrix.md), crosswalk loss reporting, and Power Automate reviewer-controlled graduation flow. This plan consumes the existing schema; it does not add another parallel metadata model.

# Recommendation

1. Define stable merge identities for typed dates, contributors, rights, and funding awards. Replaying identical evidence must be idempotent; incompatible populated values must retain the canonical value and record a provenance conflict.
2. Map only explicitly asserted OSTI, CFF, and RAiD values. Do not infer roles, dates, licenses, or sponsors from list position or unrelated provider fields.
3. Extend `GraduationCandidates` with explicit logbook category/event rules for supported structured targets, and archive an entry only after its selected candidates merge successfully.
4. Keep raw provider values in provenance and remove note fallbacks only after fixture-backed parity proves no information is lost.

# Validation

- Add schema round-trip and validation cases for every populated destination.
- Add idempotent replay, stable-key deduplication, populated-conflict, and provenance-retention tests for candidate merging.
- Add fixture-backed OSTI, CFF, and RAiD mappings plus accepted/rejected logbook graduation cases.
- Run focused `acorn-schema` and `acorn-lib` tests, `make check-wasm`, `make lint`, and `make test`.

# Acceptance criteria

- Supported provider dates, contributors, rights, and award identifiers populate the existing structured activity fields rather than notes.
- Merge identity and conflict behavior are documented and deterministic for every populated type.
- Replaying identical provider or logbook evidence does not duplicate values or provenance.
- Each logbook graduation target has an explicit source rule, stable candidate identity, accepted-merge test, and conflict test.
- Unsupported or ambiguous assertions remain in provenance without manufacturing structured values.
- Existing research-activity documents remain compatible and the portable feature matrix passes.
