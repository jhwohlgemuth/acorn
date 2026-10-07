---
type: Plan
title: "Database-backed artifact generation"
description: "Generate reproducible RAD, update, standards, document, and presentation artifacts from a selected canonical database snapshot."
difficulty: medium
impact: high
package: [acorn-lib, acorn-cli, acorn-schema]
tags: [feature, database, export, artifact, research-activity, provenance, reproducibility]
status: planned
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
