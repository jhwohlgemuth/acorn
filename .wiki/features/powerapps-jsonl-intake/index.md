---
type: Plan
title: "PowerApp and Power Automate highlight intake portfolio"
description: "Coordinate the accepted SharePoint intake, report-first approval gate, consistent form experience, retained GitLab fallback, and later shared deployment."
difficulty: mixed
impact: critical
package: [workspace]
tags: [powerapps, powerautomate, sharepoint, jsonl, intake, approval, reporting]
status: in-progress
execution: coordination
priority: next
---

> **Sanitized copy:** Account labels and related assets have been redacted. Historical checksums and import results describe the originals; see [privacy and reuse limits](../../privacy.md).

# Purpose

Coordinate the remaining Sponsor Highlights intake work without duplicating implementation phases. The accepted SharePoint route remains operational while four atomic plans finish participant and release acceptance, deploy the implemented approval gate, retain a restorable GitLab fallback, and prepare a later approved shared deployment.

This epic owns cross-plan product boundaries, dependency order, and the definition of the released workflow. It contains no executable phases. Each child plan has a five-hour phase budget, one primary lock per phase, exact files or named external systems, focused validation, and a mergeable or externally verifiable handoff.

# Accepted baseline

The current default is the SharePoint workflow in the Wohlgemuth, Jason pilot:

1. The contributor canvas app writes one immutable SharePoint list receipt and one individual JSONL receipt file per submission UUID.
2. Browser recovery, exact retry, correction ancestry, standard-license participant access, native backup/restore, catalog feedback, and the repaired collected exporter are accepted.
3. The five-minute exporter currently rebuilds the combined `logbook.jsonl` through a captured source item-ID cutoff. It is bounded to 5,000 rows and 10 MiB, requires a readable prior file or an explicit 404, and accepts a write only after exact readback.
4. An operator downloads the collected file, then uses local `stage -> inspect -> apply -> report` commands. The accepted three-row demonstration proves that manual workflow.
5. The retained GitLab intake and catalog flows are Off. Their app, connector, credentials, backups, and evidence remain available for an explicitly approved restoration attempt.

Durable evidence remains in the [SharePoint pilot](sharepoint-pilot.md), [native backup](_assets/sharepoint/backup/README.md), [operational cutover](_assets/sharepoint/cutover/README.md), [submission editing](_assets/sharepoint/submission-editing/README.md), [boundary acceptance](boundary-acceptance.md), [browser recovery](browser-recovery-acceptance.md), [write-failure acceptance](write-failure-acceptance.md), [notification walkthrough](notification-walkthrough.md), [cleanup record](test-artifact-cleanup.md), and [manual reporting walkthrough](../manual-jsonl-ingestion-graduation/walkthrough.md).

# Remaining plans

| Order | Plan | Stable phases | Remaining outcome |
|---:|---|---|---|
| 1 | [Approval pilot acceptance and cutover](approval-release/index.md) | PA-40B, PA-40C, PA-40A, PA-40D–F, PA-41, PA-41A | Accept the end-to-end report-authoring workflow, then deploy both consistent apps and the approval gate with rollback. |
| 2 | [Retained GitLab fallback hardening](gitlab-fallback-hardening/index.md) | PA-15, PA-16, PA-17 | Finish bounded-write, concurrency, and native-restore acceptance only in recreated isolated systems. |
| 3 | [Shared Power Platform deployment](shared-deployment/index.md) | PA-12 and PA-12A | Move the accepted SharePoint solution to a named shared environment and audience after the pilot release. |
| 4 | [Scheduled GitLab and SharePoint sync release](scheduled-sync/index.md) | SGS-01, SGS-02, SGS-03 | Bind and accept the hardened catalog and approved-snapshot schedulers, then deploy with monitoring and disable-only rollback. |

# Dependency order

The shared contributor/reviewer experience, immutable corrections, review ledger, optimistic-concurrency saves, summaries, finalization, and approval-aware exporter are implemented. [End-to-end acceptance](_assets/sharepoint/intake-approval/acceptance/end-to-end.md) records the isolated technical chain through collected JSONL and PDF, including the limited PA-40 fault substitutes. [Beta repair evidence](_assets/sharepoint/intake-approval/beta-repair/verification.md) records the current published forms, Standard reviewer designation, SharePoint-only app-facing summary request, background AI, and repaired app sharing. Participant launch is owner-confirmed; correction, batch read/save, and summary use remain unverified in participant context.

The remaining release path is PA-40B/40C participant retests -> PA-40A uncoached usability and exact Approved-batch canvas/PDF comparison. PA-40D denied-role acceptance, PA-40E unchanged-window late-arrival acceptance, and PA-40F connector-level lost-response acceptance are separate pre-cutover gates. They do not depend on the walkthrough, but all PA-40 acceptance phases serialize on the same user-acceptance lock to preserve shared fixtures and app state. PA-41 requires PA-40A and PA-40D/E/F, then explicit operational approval; PA-41A verifies production and hands off operations.

PA-12/12A wait for PA-41A and approved shared-target readiness, including groups, licensing/capacity, ownership, protected connections, notification destination, and baseline/history policy. PA-15 through PA-17 remain independent fallback validation and do not block SharePoint release; operational GitLab flows remain Off. Further accessibility features and checks remain deferred by user direction.

The [scheduled-sync plan](scheduled-sync/index.md) owns the explicit approved-export mirror and catalog scheduler. Its target-free source hardening and offline tests are complete; target binding and hosted acceptance remain blocked. SGS-03 operational mirroring waits for PA-41A, while SGS-01/02 isolated validation can proceed independently once their own targets are approved. It never feeds the Vergil inbox and does not gate the SharePoint-only release.

The November 24 repository review inspected these plans, retained flow definitions, acceptance artifacts, and November 12 beta evidence. It did not rerun tenant acceptance or establish external prerequisites. Historical native artifacts remain subject to [sanitization and reuse limits](../../privacy.md).

The [November 25 live failure audit](_assets/sharepoint/flow-failure-audit-2026-11-25/README.md) distinguishes repaired intake/export faults from expected finalizer/exporter guard tests. Fresh owner-context summary requests succeeded from both Power Automate and the published app; historical caller timeouts remain unexplained. The subsequent [batch-summary startup repair](_assets/sharepoint/batch-summary-repair/README.md) published an explicit timer-start sequence in app version 130 and verified automatic generation on published startup. PA-40C retains participant-context acceptance. The published app has evolved beyond the November 12 UI description; no participant or cutover phase is complete from these checks.

# Cross-plan decisions

## Ease of use and consistent controls

Ease of use is the leading product criterion. Both canvas apps consume one versioned component library and token contract. Genuine yes/no values use labeled toggles. Three-to-five short mutually exclusive choices use segmented button bars. Longer single-choice sets use native radio menus. Multi-choice sets use the same visual menu family but native checkbox semantics and an explicit selected count; they are never announced as radio groups. Long or dynamic collections use searchable selectors. Free text is reserved for narrative content with adjacent requirements, limits, and errors.

The apps share typography, spacing, color, focus, validation, empty/loading/error states, responsive breakpoints, status language, field order, and primary-action placement. Component-library updates are published, version-noted, explicitly adopted by each app, and regression-tested; a local copied component is not an accepted substitute because it stops receiving library updates.

## Report-authoring approval surface

The approval app is the pre-PDF report assembly surface, not a SharePoint list editor. Its page rail, report canvas, inline inspector, compare view, validation summary, and final batch summary mirror the actual ACORN title page, project pages, status-update cards, completed/in-progress/upcoming groups, report period, entry order, and inclusion rules.

WYSIWYG means semantic and layout fidelity: the same content, ordering, grouping, labels, visibility, period, project boundaries, and approximate page composition. Canvas Apps and the Chromium-backed PDF renderer use different layout engines, so exact font metrics and line/page breaks are not promised. Every acceptance fixture renders a real ACORN combined PDF from the same frozen projection and records any bounded difference.

## Immutable source and approval projection

Contributor receipts remain immutable. Reviewer changes are drafts and frozen projections in a separate review ledger. They never overwrite a source list row or individual receipt file. A modified projection receives a stable approval UUID; unchanged content may retain its source UUID only when the frozen bytes are exactly identical. Correction ancestry is rewritten through the source-to-approved mapping and must resolve to an included predecessor in the same or an earlier approved batch.

A reporting period is a closed half-open UTC interval over SharePoint receipt `Created`. Materialization captures a source high-water item ID. Finalization rechecks source completeness and requires every item to be approved or explicitly excluded with a reason. The batch becomes exporter-visible only after all included payloads are frozen and read back exactly.

## Collected export and rollback

At pilot cutover, the last verified collected file becomes an immutable pre-approval baseline. The approval-aware exporter deterministically rebuilds `baseline + complete approved batches`, preserving current count, size, serialization, prior-read, conditional-write, and exact-readback protections. Pending, draft, finalizing, failed, unresolved, and excluded items never enter the collected file.

Rollback freezes the last verified collected file and disables further publication. It must not re-enable the former unfiltered exporter, reconstruct content from mutable drafts, or delete source receipts.

# Related-plan boundaries

- [SharePoint JSONL download](../sharepoint-jsonl-download/index.md) and [headless CI download](../sharepoint-headless-ci-download/index.md) read the collected file. They do not force exporter freshness, prove review completeness, or mutate Power Platform systems. Before PA-41 they capture the accepted current exporter; after PA-41 they capture the approval-gated file.
- [Power Automate Teams workflow bot](../powerautomate-teams-bot/index.md) may add a future bounded intake or approval capability only through an accepted target-system contract and effect ledger. It does not own review state, finalization, or publication.
- [Vergil automation](../vergil-gitlab-jsonl-automation/index.md) consumes only its approved GitLab inbox. The scheduled approved-export mirror is separate from that inbox; there is no implicit mirror-to-intake admission, acknowledgement, or SharePoint cleanup path.
- [Sponsor highlights clarity](../sponsor-highlights-clarity.md) owns future editorial synthesis, ASPECT evidence, renderer changes, accessibility of generated PDFs, and sponsor publication. This portfolio owns the pre-export review of fields already admitted by the intake schema; it does not invent the future editorial schema.
- [Structured enrichment](../../refactors/core/research-activity-enrichment-integration.md) owns canonical enrichment and conflict policy. Reviewer edits here do not silently graduate or enrich canonical research-activity metadata.
- [Runner-pool isolation](../../refactors/ci-runner-pools.md) owns reporting-worker credentials and concurrency. It is unrelated to the interactive SharePoint approval gate.

# Portfolio acceptance

The coordinated work is accepted only when:

- both apps use the same published component revision and pass desktop and 375-pixel usability checks; further accessibility features and checks are deferred by user direction;
- a reviewer can assemble a closed period from the report canvas without opening raw JSON or SharePoint list administration;
- original, edited, excluded, correction, general-update, activity, and status-update fixtures produce the expected frozen projection;
- app preview and actual ACORN PDF agree on content, order, grouping, visibility, report period, and project boundaries;
- the collected JSONL contains the immutable baseline plus complete approved batches only, with deterministic replay and safe uncertain-write behavior;
- source receipts, contributor identity, review history, decisions, mappings, frozen bytes, and publication evidence remain auditable;
- the operational cutover has fresh backups, named owners/audiences, exact before/after bytes, a tested freeze rollback, and explicit approval;
- the former unfiltered exporter cannot resume accidentally after cutover.
