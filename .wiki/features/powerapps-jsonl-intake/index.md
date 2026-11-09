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

Coordinate the remaining Sponsor Highlights intake work without duplicating implementation phases. The accepted SharePoint route remains operational while six atomic plans add a report-authoring approval experience, gate the collected JSONL on approved batches, align the contributor form, retain a restorable GitLab fallback, and prepare a later approved shared deployment.

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
| 1 | [Shared intake and approval experience](approval-experience/index.md) | PA-39 and bounded suffix phases | PA-35B is accepted with remaining accessibility work deferred by user direction; refit contributor intake to the accepted shared UI. |
| 2 | [Approval ledger and finalization](approval-ledger/index.md) | PA-37S, PA-37A, and PA-37B | Bind batches to their source list, freeze auditable approval projections, and commit complete batches after exact readback. |
| 3 | [Approval-gated collected export](approval-export/index.md) | PA-38 and bounded suffix phases | Preserve the pre-approval baseline and extend the collected JSONL only with complete approved batches. |
| 4 | [Approval pilot acceptance and cutover](approval-release/index.md) | PA-40, PA-41 and bounded suffix phases | Accept the end-to-end report-authoring workflow, then deploy both consistent apps and the approval gate with rollback. |
| 5 | [Retained GitLab fallback hardening](gitlab-fallback-hardening/index.md) | PA-15, PA-16, PA-17 | Finish bounded-write, concurrency, and native-restore acceptance only in recreated isolated systems. |
| 6 | [Shared Power Platform deployment](shared-deployment/index.md) | PA-12 and PA-12A | Move the accepted SharePoint solution to a named shared environment and audience after the pilot release. |

# Dependency order

PA-34 froze the UI, accessibility, field-to-report, and preview contract; PA-35 delivered the fixture-only reviewer shell. PA-35A accepted the activity report canvas against an ACORN PDF; the PA-35C repair published shared library revision 1.0.1 and the reviewer adopted it. PA-35B accepted the status/correction experience, with all further accessibility features and checks deferred by user direction. PA-36 storage, PA-36A/36B TEST ONLY batch materialization and delegated reads, and PA-36C reviewer-triggered AI summaries are accepted. PA-37's compact binding, changed/unchanged/conflict/retry states, and non-Ready app-state check are accepted. PA-37S is binding each batch to its materialization source before PA-37A prepares projection identities and correction mappings. PA-38 consumes frozen approvals; PA-39 may refit the contributor app after the shared components and approval behavior are accepted. PA-40 joins approval, export, and contributor work for isolated acceptance. PA-41 performs the pilot cutover only after explicit operational approval. PA-12 remains blocked until PA-41A is accepted and a shared target, security groups, licensing, capacity, ownership, and notification destination are named.

PA-15 through PA-17 are independent of the SharePoint approval path and do not block it. They remain useful only as fallback restoration evidence. The operational GitLab flows stay Off unless a separately authorized restoration decision follows accepted isolated evidence.

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
- [Vergil automation](../vergil-gitlab-jsonl-automation/index.md) consumes only its approved GitLab inbox. There is no implicit SharePoint-to-GitLab bridge, acknowledgement, or cleanup path.
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
