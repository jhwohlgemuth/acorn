---
type: Plan
title: "Canonical research activity database"
description: "Evolve ACORN's candidate store into a durable, revisioned source of truth for research activity data, linked logbook entries, enrichment evidence, and reconciliation decisions."
difficulty: high
impact: critical
package: [acorn-lib, acorn-cli, acorn-schema]
tags: [feature, database, research-activity, logbook, enrichment, provenance, revision, reconciliation]
status: in-progress
priority: next
---

# Goal

Make ACORN's database the durable working truth for research activity data: ingest authored or discovered activities, retain evidence and history, apply reviewed enrichments, link logbook developments to their activity, and recover safely from mutations. Files and remote records remain import/export surfaces rather than the only canonical state.

# Current evidence

- `crates/acorn-lib/src/io/database/candidate.rs` already creates and enriches canonical partial RAD JSON by normalized identity keys, retains provenance, and reports ambiguous conflicts without merging them.
- `crates/acorn-lib/src/io/database/logbook.rs` now persists complete typed `ResearchActivity` values, synchronizes linked logbook entries, retrieves typed canonical data, and performs logbook graduation in one transaction.
- `crates/acorn-lib/assets/migrations/` now contains numbered SQL definitions for every database table, including `research_activities`, `logbook_entries`, and immutable `research_activity_revisions`. Graduation stores the complete pre-change RAD snapshot before updating canonical data.
- `crates/acorn-lib/src/io/mcp/tools.rs` routes the mutating graduation tool through that durable transaction and returns both the canonical activity ID and backup revision ID.
- Storage still lacks an explicit migration ledger, database-level relations/indexes, a general revision policy for mutations beyond graduation, reconciliation commands, and a query API suitable for all artifact consumers.

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/database/{candidate.rs,logbook.rs,mod.rs,schema.rs}`, dedicated database tests, `crates/acorn-lib/src/io/config/transfer.rs`, relevant MCP service code, and focused CLI command/argument files after the CLI integration lock is available.
- **Create:** the migration ledger and compatibility metadata that execute the numbered SQL assets under `crates/acorn-lib/assets/migrations/`; typed repository/query/reconciliation leaves under `crates/acorn-lib/src/io/database/`; dedicated fixtures under `tests/fixtures/database/`; an `acorn database` or narrowly named RAD-store CLI command leaf.
- **Delete:** ad hoc schema creation and duplicate candidate/full-RAD matching paths only after migrated databases, recovery tests, and all callers use one repository contract.

# Dependencies

This plan consumes portable RAD/logbook types and preserves the serialized shapes in the [standards field matrix](../assets/standards-field-matrix.md). It provides the canonical query and snapshot contract required by [database-backed artifact generation](database-backed-artifact-generation.md). A future `acorn-store` extraction follows the crate-boundary network and is not a prerequisite for completing the in-tree contract.

# Recommendation

1. Establish a numbered migration ledger and migrate existing databases without destructive recreation. Add unique indexes for portable IDs and activity/entry identity, explicit relation integrity where SQLite and DuckDB semantics agree, and startup compatibility checks.
2. Consolidate candidate and complete-RAD persistence behind one repository API. Identity matching must remain exact and normalized; zero matches create, one match enriches, and multiple matches produce a reviewable conflict.
3. Record every canonical mutation as an immutable revision with operation, actor/interface, timestamp, source evidence, and optional parent revision. Add restore as a new revision-producing transaction rather than deleting history.
4. Model provenance observations and conflicts as queryable records instead of opaque JSON arrays while preserving lossless compatibility during migration. Never overwrite a conflicting asserted value automatically.
5. Define logbook retention explicitly: synchronized entries are append-preserving, graduation archives rather than deletes, and removed source entries remain recoverable through revisions. Add indexes and query filters for timestamp, state, category, origin, and archive status.
6. Add transactional ingest, inspect, history, diff, reconcile, and restore service methods, then expose thin CLI/MCP surfaces with dry-run and machine-readable results. Respect the configured database path and offline policy.
7. Add concurrency and crash tests covering rollback, retry, duplicate ingest, conflicting identities, revision restoration, and both SQLite and DuckDB feature paths.

# Validation

- Run focused database tests under the default SQLite backend and the DuckDB feature path.
- Run `make lint`, `make test`, and `make check-wasm`; database additions must stay behind the native `std` capability and not expand the portable schema dependency graph.
- Migrate a database created by the previous released schema, verify row counts and JSON byte preservation, then exercise ingest, enrichment, graduation, history, and restore.
- Inject a failure after revision insertion and prove the whole mutation rolls back; retry the same request and prove IDs and history remain consistent.

# Acceptance criteria

- Existing databases upgrade through ordered, tested migrations without data loss.
- Every canonical mutation has a queryable immutable revision and restore creates an auditable successor state.
- Canonical RAD, linked logbook entries, provenance, and unresolved conflicts can be queried through typed APIs without parsing command output.
- Exact repeated ingest is idempotent; enrichment is additive; ambiguous or conflicting assertions are retained for review and never silently overwrite truth.
- Graduation atomically backs up the prior RAD, updates canonical data, and archives linked entries under one stable research activity ID.
- SQLite and DuckDB tests, full lint/tests, and portable feature gates pass.
