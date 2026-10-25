---
type: Plan
title: "Manual JSONL ingestion and graduation handback"
description: "Reconcile manually reviewed JSONL and canonical indexes before handing intake back to the GitLab bot."
difficulty: medium
impact: high
package: [acorn-cli, acorn-lib, acorn-schema]
tags: [jsonl, intake, graduation, cli, demo, reporting]
status: planned
priority: later
execution: phased
phase_budget_hours: 5
---

# Goal

Reconcile manually reviewed canonical indexes and exact completed JSONL lines before automated ingestion resumes. The next-week demo is implemented and packaged; the [demo packet](walkthrough.md) is the durable walkthrough and acceptance evidence. This remaining production handback work does not block the isolated demonstration.

Supporting material: [walkthrough](walkthrough.md) and [demo bundle](demo-bundle-readme.md). Rehearsal scripts, captured records and PDFs are in `_assets/`.

# Current evidence

- Reviewed October 24, 2026 against `49b4700d` plus the current uncommitted intake implementation. The command reference and tests, rather than the older PDF screenshots, define the current API.
- [URI staging](../../../docs/src/cli-reference/commands/intake.md#jsonl-sources-for-local-review) accepts local paths, `file:` URIs and GitLab HTTPS file links through `intake stage --source`. `crates/acorn-cli/src/commands/intake/source.rs` captures remote bytes at an immutable commit with provenance for offline stage/inspect/apply. It does not mutate the remote inbox. Local destination routing and member authorization still require an explicit ACRE registry.
- [Local graduation](../../../docs/src/cli-reference/commands/intake.md#local-graduation) is `intake run --index`: `--dry-run` previews, `--interactive` selects entries and confirms the final write, `--force` accepts all eligible promotions, and `--accept` with `--expected-digest` applies reviewed IDs. A bare run with candidates requires an approval mode. Dry-run always prevents writes; force preserves validation and backups. Hidden `candidates`/`graduate` compatibility commands remain. `graduation.rs` and its tests cover JSON/JSONC/YAML/YML/ZON, cancellation, stale input, unsupported entries, replay and byte-exact backups.
- Graduation currently supports contributor and classification candidates only. Target-specific ASPECT changes remain in [sponsor highlights clarity](../sponsor-highlights-clarity.md); the current CLI does not complete that work. `intake run --config` is the separate online GitLab worker mode, not local graduation, and rejects local-only flags.
- The [demo packet](walkthrough.md) preserves the personnel envelope `4313fcfc-0201-441e-8bda-c77718a31dcf` from immutable commit `3090aa5714b4c6ff59c5dd0b6ecba999f918caa2`. Both current-command eleven-step rehearsals passed, including two local PDF renders, replay and rejection checks. Ten native terminal checks passed. The latest implementation validation passed `make lint`, `make test` (2,694 passed; 17 skipped), the book build and wiki validator; these were not rerun as part of this documentation review.
- Canonical production data and the remote inbox were unchanged by rehearsal. Local receipts are not bot admission rows. Local stage/apply/graduation and combined/dry-run reporting do not send Teams or email automatically; milestone notifications use the owner-triggered receiver until external integration is accepted. MAN-06 remains the production handback gate.

# Scope and decisions

Local ingestion and graduation remain separately controlled operations. Graduation approval can use interactive review, explicit reviewed IDs with a digest, or `--force` for all eligible promotions. The demo uses a fictitious personnel contributor in a disposable local index, preserving the exact captured envelope. It leaves the remote inbox intact. Do not restart a bot with an empty admission database against manually processed production lines and assume local receipts will prevent re-admission.

# Remaining dependency and acceptance

The demo packet is usable independently of Vergil, callback authentication and shared deployment. If approved cloud access fails on presentation day, show the recorded form receipt/commit and run the exact captured source locally; label the split accurately. Remaining live-producer acceptance belongs to the [intake plan](../powerapps-jsonl-intake/index.md).

Before any real manual-to-bot transition, MAN-06 must reconcile the exact source and canonical state while the bot is stopped. A separate fresh bot fixture is an alternative only when explicitly certified. Rejected, unknown and newly appended lines must survive handback. A real canonical merge remains human-authorized.

# Execution phases

## MAN-06 — Define and verify safe manual-to-bot handback

- Status: planned
- Budget: 3 active hours maximum
- Prerequisites: reviewed manual receipts; designated repository/review authority; isolated repository copy for rehearsal
- Exact write set: `.wiki/features/manual-jsonl-ingestion-graduation/handback.md`; disposable handback fixture checkout; pilot project `22464` manual reconciliation branch only when specifically authorized
- Primary integration lock: isolated pilot repository
- Outcome: Specify and rehearse how reviewed local index changes and exact completed lines are reconciled before bot activation. Real canonical merge remains human-authorized.
- Focused validation: Keep bot stopped; compare original/current source revisions; handle only exact accepted line bytes; preserve new/rejected lines and provenance. Confirm already-applied local data is not re-admitted by merely restarting with an empty bot database.
- Handoff: Give PA-05 an explicit reconciled source/index baseline or certify a separate fresh bot fixture. No production inbox deletion is implied by demo completion.
