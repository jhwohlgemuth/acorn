---
type: Plan
title: "Complete structured research-activity enrichment"
description: "Map provider and logbook evidence into the existing activity date, contributor, rights, and funding fields with deterministic merge and conflict behavior."
difficulty: medium
impact: high
package: [acorn-schema, acorn-lib]
tags: [schema, research-activity, enrichment, provenance, logbook]
status: planned
execution: phased
phase_budget_hours: 5
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

# Execution phases

## ENR-01 — Freeze merge identities and conflict policy

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-lib/src/analyzer/host/discovery/mod.rs`, focused enrichment policy tests
- Primary integration lock: discovery candidate merge
- Outcome: Define deterministic identities, authored-value precedence, replay behavior, and conflict provenance.
- Focused validation: Run table tests for identical, complementary, and conflicting evidence.
- Handoff: Remove ENR-01 and reuse the policy in each field slice.

## ENR-02 — Map typed activity dates

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ENR-01
- Exact write set: `crates/acorn-lib/src/analyzer/host/discovery/mod.rs`, date enrichment tests
- Primary integration lock: discovery candidate merge
- Outcome: Map only explicit OSTI, CFF, RAiD, and supported logbook date assertions.
- Focused validation: Test identities, conflicts, replay, and absence of inferred dates.
- Handoff: Remove ENR-02 and preserve date fixtures for integration.

## ENR-03 — Map contributors and organizations

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ENR-01
- Exact write set: `crates/acorn-lib/src/analyzer/host/discovery/mod.rs`, contributor enrichment tests
- Primary integration lock: discovery candidate merge
- Outcome: Project explicitly credited people, roles, and organizations without list-position inference.
- Focused validation: Test stable identity, role conflicts, provider replay, and authored precedence.
- Handoff: Remove ENR-03 and preserve contributor fixtures.

## ENR-04 — Map rights and access

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ENR-01
- Exact write set: `crates/acorn-lib/src/analyzer/host/discovery/mod.rs`, rights enrichment tests
- Primary integration lock: discovery candidate merge
- Outcome: Project explicit licenses, rights statements, and access assertions with loss reporting.
- Focused validation: Test URI/label identities, incompatible values, ambiguity, and replay.
- Handoff: Remove ENR-04 and preserve rights fixtures.

## ENR-05 — Map funding awards

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ENR-01
- Exact write set: `crates/acorn-lib/src/analyzer/host/discovery/mod.rs`, funding enrichment tests
- Primary integration lock: discovery candidate merge
- Outcome: Project explicit sponsor and award evidence without manufacturing missing funders.
- Focused validation: Test award identities, sponsor conflicts, missing fields, and replay.
- Handoff: Remove ENR-05 and preserve funding fixtures.

## ENR-06 — Extend graduation rules

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ENR-02, ENR-03, ENR-04, ENR-05
- Exact write set: `crates/acorn-lib/src/io/database/logbook.rs`, graduation tests
- Primary integration lock: logbook graduation
- Outcome: Add explicit category/event targets and archive only after selected candidates merge successfully.
- Focused validation: Test accepted merges, conflicts, retry, and retained unarchived entries.
- Handoff: Remove ENR-06 and record supported graduation mappings.

## ENR-07 — Prove provenance and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ENR-06
- Exact write set: provider/discovery fixtures, enrichment documentation, wiki indexes, this plan
- Primary integration lock: enrichment integration
- Outcome: Prove end-to-end replay, conflicts, compatibility, and durable provenance before removing the plan.
- Focused validation: Run focused provider/database tests, portable checks, `make lint`, `make test`, and wiki validation.
- Handoff: Delete this plan and every index, network, and lane entry.
