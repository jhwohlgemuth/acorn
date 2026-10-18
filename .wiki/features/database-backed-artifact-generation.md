---
type: Plan
title: "Database-backed artifact generation"
description: "Generate reproducible RAD, update, standards, document, and presentation artifacts from a selected canonical database snapshot."
difficulty: medium
impact: high
package: [acorn-lib, acorn-cli, acorn-schema]
tags: [feature, database, export, artifact, research-activity, provenance, reproducibility]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Let users create the same supported artifacts from ACORN's maintained research activity database that they can create from files, with an explicit activity/revision selection and enough provenance to reproduce the result.

# Current evidence

- `crates/acorn-lib/src/io/database/logbook.rs` provides typed retrieval of current `ResearchActivity` data, linked logbook entries, and immutable revisions.
- Existing `InputOutput`, crosswalk, logbook-update, PDF, PowerPoint, and template paths already transform supplied typed data; most renderers do not need database awareness.
- Export entry points currently begin with a source file or in-memory activity. There is no shared selector for canonical activity ID versus revision ID, no artifact manifest, and no end-to-end database export test.

# Project code boundaries

- **Update:** focused export service and command files in `crates/acorn-lib/src/io/` and `crates/acorn-cli/src/commands/export/`, CLI arguments/help during a single integration window, inbound ACORN MCP operation inputs, and dedicated export tests.
- **Create:** a database artifact-source selector and reproducibility manifest leaf in `crates/acorn-lib/src/io/`; database-export fixtures under `tests/fixtures/database/export/`.
- **Delete:** duplicate file/database renderer dispatch only after both sources feed one typed rendering pipeline.

# Dependencies

The implemented canonical database contract supplies typed lookup, revision identity, restore semantics, and migration compatibility. This plan reuses the bounded reporting-window and artifact-manifest contracts from the implemented [Power Automate reporting workflow](powerapps-gitlab-logbook-reporting.md). Standards-specific artifacts also follow the implemented [standards field matrix](../assets/standards-field-matrix.md) and crosswalk loss contract. Renderer hardening remains owned by the existing export/OOXML plans.

# Recommendation

1. Add an artifact source enum that selects a canonical activity by portable database ID or an immutable revision ID and resolves to a typed `ResearchActivity` before renderer dispatch.
2. Reuse existing serializers, crosswalks, update reports, PDF, and PowerPoint transforms unchanged after source resolution. Database code must not absorb rendering policy.
3. Reuse the reporting workflow's machine-readable manifest identity, source, time-bound, ACORN-version, option, and digest fields; extend it with database activity and revision IDs rather than introducing a parallel artifact manifest.
4. Add CLI and inbound ACORN MCP operation selection with mutually exclusive file/activity/revision sources, explicit overwrite policy, and clear errors for missing or ambiguous identities. The outbound MCP client does not proxy or automatically expose this operation.
5. Support current RAD JSON/YAML/Markdown and logbook-update artifacts first; add standards profiles and binary documents only after deterministic source-to-render tests are green.

# Validation

- Seed a temporary database, export current and historical revisions, and compare canonical text outputs with equivalent file-backed exports.
- Prove a later enrichment changes current-state output but does not change an artifact regenerated from the earlier revision.
- Run focused SQLite and DuckDB export tests, `make lint`, `make test`, and renderer-specific visual checks for PDF/PPTX changes.

# Acceptance criteria

- Every existing supported renderer can consume a typed activity resolved from the database without a parallel implementation.
- Users can select current state or one immutable revision by portable ID through CLI and inbound MCP operation contracts.
- Generated artifacts include a reproducibility manifest and stable input digest.
- Database-backed logbook reports use the same bounded-window and manifest semantics as automated bucket-file artifacts.
- Historical revision exports remain byte-stable where the target format is deterministic and semantically stable otherwise.
- File-backed behavior remains compatible and all full-workspace and renderer validation passes.

# Execution phases

## DBA-01 — Define the database artifact selector

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-lib/src/io/database/artifact.rs`, `crates/acorn-lib/src/io/database/mod.rs`, `crates/acorn-lib/src/io/database/tests/mod.rs`
- Primary integration lock: database artifact API
- Outcome: Represent file, current activity, and immutable revision sources with unambiguous validation.
- Focused validation: Run focused selector construction and invalid-identity tests.
- Handoff: Remove DBA-01 and expose the selector to backend resolution.

## DBA-02 — Resolve SQLite current and historical sources

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DBA-01
- Exact write set: `crates/acorn-lib/src/io/database/artifact.rs`
- Primary integration lock: database artifact API
- Outcome: Resolve a portable activity ID or revision ID to typed activity data and exact provenance.
- Focused validation: Seed SQLite and compare current and historical resolutions.
- Handoff: Remove DBA-02 and preserve backend-neutral fixtures for DBA-03.

## DBA-03 — Prove DuckDB selector parity

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DBA-02
- Exact write set: `crates/acorn-lib/src/io/database/artifact.rs`, `tests/fixtures/database/export/manifest.json`
- Primary integration lock: database fixtures
- Outcome: Produce the same typed selection and provenance through DuckDB.
- Focused validation: Run equivalent SQLite and DuckDB selector cases.
- Handoff: Remove DBA-03 and freeze the backend-neutral source contract.

## DBA-04 — Extend the reproducibility manifest

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DBA-02
- Exact write set: `crates/acorn-schema/src/report.rs`
- Primary integration lock: artifact manifest schema
- Outcome: Add database activity/revision identity, options, version, window, and stable input digest to the existing manifest contract.
- Focused validation: Run serde, schema, digest, and backward-compatibility tests.
- Handoff: Remove DBA-04 and provide the manifest type to renderer phases.

## DBA-05 — Unify deterministic text rendering

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DBA-03, DBA-04
- Exact write set: `crates/acorn-cli/src/commands/export/mod.rs`
- Primary integration lock: export command
- Outcome: Feed file and database sources through one RAD JSON/YAML/Markdown rendering pipeline.
- Focused validation: Compare byte-stable file/current/revision outputs.
- Handoff: Remove DBA-05 and record the shared renderer entry point.

## DBA-06 — Render bounded logbook updates

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DBA-04
- Exact write set: `crates/acorn-lib/src/io/database/artifact.rs`
- Primary integration lock: database artifact API
- Outcome: Generate bounded-window update artifacts using the shared reporting manifest semantics.
- Focused validation: Test empty, current, historical, and later-enrichment windows.
- Handoff: Remove DBA-06 and expose the bounded artifact to CLI composition.

## DBA-07 — Add CLI database source selection

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DBA-05, DBA-06
- Exact write set: `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/commands/export/mod.rs`, `CommandLineHelp.md`, `crates/acorn-cli/src/commands/export/tests/mod.rs`
- Primary integration lock: CLI syntax and generated help
- Outcome: Add mutually exclusive file/activity/revision inputs and explicit overwrite behavior.
- Focused validation: Run parser, command, collision, missing-ID, and generated-help tests.
- Handoff: Remove DBA-07 and freeze the user-facing source syntax.

## DBA-08 — Add the inbound MCP operation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DBA-05, DBA-06
- Exact write set: `crates/acorn-lib/src/io/mcp/server.rs`, `crates/acorn-lib/src/io/mcp/tests/mod.rs`
- Primary integration lock: inbound MCP catalog
- Outcome: Expose typed database source selection without proxying through the outbound client.
- Focused validation: Test effects, limits, missing/ambiguous identities, overwrite policy, and deterministic results.
- Handoff: Remove DBA-08 and record the stable operation schema.

## DBA-09 — Complete renderer parity and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: DBA-07, DBA-08, EXP-10
- Exact write set: `tests/fixtures/database/export/manifest.json`, `docs/src/cli-reference/commands/export.md`, `.wiki/index.md`, `.wiki/features/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/features/database-backed-artifact-generation.md`
- Primary integration lock: renderer validation
- Outcome: Cover standards and binary renderers, run full gates, and remove the plan.
- Focused validation: Run both backends, renderer visual checks, `make lint`, `make test`, and wiki validation.
- Handoff: Delete this plan and all index, network, and lane entries.
