---
type: Plan
title: "Intake approval pilot acceptance and cutover"
description: "Accept the complete contributor-to-report workflow in isolation, then deploy the consistent apps and approval-gated exporter with explicit operational authorization and rollback."
difficulty: high
impact: critical
package: [power-platform, sharepoint, workspace]
tags: [powerapps, powerautomate, acceptance, deployment, rollback, pdf]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Join the shared experience, approval ledger, and approval-aware exporter into one evidence-backed pilot release. Prove that contributors can submit naturally, reviewers can assemble the report from the WYSIWYG workspace, only finalized projections enter collected JSONL, and ACORN produces the expected PDF. After explicit approval, cut over production with fresh backups, a verified immutable baseline, named audiences, observable freshness, and a freeze-only rollback.

# Release matrix

The isolated acceptance batch includes:

| Case | Required evidence |
|---|---|
| Unchanged project activity | Source and approved bytes/UUID identical; appears once in correct state group. |
| Edited activity | Source remains immutable; new approval UUID; edited text/options appear in preview, JSONL, and PDF. |
| General update | Appears on title page and nowhere else. |
| Status update | Period, health/context, significance, evidence, and risk match preview/PDF. |
| Excluded item | Decision/reason/audit retained; absent from frozen projection, collected file, and PDF. |
| Same-batch correction | Approved predecessor mapping resolves; only intended effective entries appear under local intake rules. |
| Prior-batch correction | Approved predecessor resolves against prior collected history. |
| Competing reviewer | Stale save is rejected and recoverable without silent overwrite. |
| Late source | Finalization blocks; no partial publication. |
| Uncertain export write | Exact reread reconciles or prior file remains without success. |
| Unauthorized contributor/reviewer/finalizer | Denied by connector/flow authorization, not only hidden UI. |

# WYSIWYG acceptance method

For every accepted fixture:

1. Capture the source receipt and materialized review item.
2. Record reviewer edits/decision and freeze the approved projection.
3. Capture each approval-app report page at desktop and 375-pixel widths.
4. Export the same Approved batch through the TEST ONLY collected exporter.
5. Run local `stage`, `inspect`, confirmed `apply`, and combined PDF generation in an isolated workspace.
6. Extract PDF text and render page images.
7. Compare content, order, grouping, labels, visibility, report period, project boundaries, and approximate composition.
8. Record only renderer-engine differences in font metrics, wrapping, or page breaks; any semantic mismatch fails acceptance.

# Human usability acceptance

At least one approved contributor and one approved reviewer who did not build the apps perform representative tasks without implementation coaching. Observe task completion, errors, backtracking, misunderstood labels, and confidence about whether work is saved/published. Required outcomes:

- contributor can submit and recover/retry without believing the report is already published;
- reviewer can find unresolved items, edit from the report view, compare source, exclude with reason, resolve conflicts, understand validation, and identify what finalization will publish;
- both users recognize the same controls, terms, state language, and primary-action placement;
- reviewer can predict the PDF’s content and hierarchy from the approval workspace.

Material task failure or confusion blocks cutover and becomes a bounded repair phase in the owning plan. Accessibility-specific acceptance is deferred at the user's direction; this release gate does not claim those checks passed.

# Cutover runbook

The operational change is one controlled window:

1. Confirm owner authorization, reviewer/finalizer/contributor groups, connections, capacity, DLP/licensing, and support contacts.
2. Export fresh native backups of the contributor app, current exporter, flows, connection references, and relevant list/library schemas.
3. Verify current collected-file freshness; capture exact baseline bytes, revision, counts, digest, cutoff, and timestamp.
4. Provision least-privilege review lists/library and deploy approval flows/app without enabling publication.
5. Publish the shared component revision and update both apps to that exact revision.
6. Disable the old exporter trigger and prove no active run or overlapping trigger remains.
7. Activate the approval-aware exporter with the verified baseline.
8. Submit one operational smoke receipt, confirm it remains absent while unresolved, finalize it, and confirm exact collected/PDF behavior.
9. Update the entry hub, notifications, guides, monitoring, owner matrix, and rollback instructions.

No operational mutation is authorized by this plan text alone. PA-41 requires explicit owner approval immediately before the window.

# Rollback

Rollback disables approval finalization/publication triggers and preserves the last verified collected file. Contributor receipt intake may remain available if it is healthy because receipts are immutable and unpublished until approval resumes. If the contributor UI itself is defective, restore its native backup without changing receipt storage. Never re-enable the old unfiltered exporter. Preserve failed batch state and diagnostics for repair; do not delete or edit audit evidence.

# Acceptance criteria

- PA-40 technical acceptance preserves exact run IDs, revisions, bytes/digests, and PDF evidence. The owner allowed one-account authorization evidence to remain unverified and accepted the controlled source-change guard plus a post-write failure proxy for PA-40 only. Full release still requires denied-role, unchanged-window late-arrival, and connector-level lost-response service checks before operational cutover.
- Both apps use one accepted component revision and pass usability gates. Further accessibility work is deferred by user direction.
- Actual PDF comparison has no semantic mismatch.
- Operational baseline and rollback artifacts are complete before activation.
- One production smoke receipt remains excluded from the collected file while its review decision is unresolved, then appears exactly once after approval and batch finalization.
- Notifications and guides say `received`, `ready for review`, `approved`, and `published to collected export` accurately; no earlier state claims report publication.
- Exporter monitoring names the last included Approved batch and freshness.
- Old unfiltered export cannot run after cutover.

# Current PA-40 evidence and boundaries

The [November 10 end-to-end acceptance](../_assets/sharepoint/intake-approval/acceptance/end-to-end.md) verifies contributor project, General, same-batch correction, prior-batch correction, unchanged approval/exclusion, and live project status through the isolated PA31 source, materializer, connected review, approved freeze/export, and local combined PDF. Corrections name newly approved root IDs; local reports show only effective revised wording. Unchanged project bytes and UUID survived freeze, while an excluded General item remained only in the audit manifest. The TEST ONLY preparation/finalizer definitions trust PA31; the finalizer includes the LF-aware 10 MiB condition. An ETag-stale reviewer save was rejected, then recovered by reload and exact reread. The published TEST ONLY contributor version 17 and saved unpublished reviewer app carried status item 18 through a four-page PDF; durable desktop/375-pixel captures preserve the live reviewer state. A controlled source-interval change was rejected before writes; the owner accepted that service result with the full-window/high-water guard as the PA-40 late-source substitute. A TEST ONLY exporter run passed the post-write failure proxy: exact reread and receipt succeeded, and collected bytes remained unchanged. The original exporter was restored by native package import; a fresh export matches the pre-fault action graph, trigger, and connection reference exactly. The final [accepted JSONL](../_assets/sharepoint/intake-approval/acceptance/accepted.jsonl) has 11 lines and 5,434 bytes, and the [accepted PDF](../_assets/sharepoint/intake-approval/acceptance/accepted.pdf) has four pages. Materializer, preparation, finalizer, and exporter remain Off. A true unchanged-window late arrival, connector-level lost response, and denied-role checks remain PA-41 gates. The first project/General batch retained a stale high-water field after zero-row recovery and must not be finalized.

# Dependencies and boundaries

PA-40 technical acceptance used accepted PA-37B, PA-40S, [PA-38B TEST ONLY export acceptance](../_assets/sharepoint/intake-approval/exporter/failure-acceptance.md), and the [accepted cross-app experience](../_assets/sharepoint/intake-approval/acceptance/cross-app-ui.md). Its one-account and fault-substitution exceptions do not carry into PA-41 operational cutover. PA-41 requires PA-40A plus explicit operational authorization, denied-role evidence, and the true late-arrival and connector-level lost-response checks. This plan does not authorize wider shared-environment migration; PA-12 remains separate. Direct OAuth/CI download and Vergil automation are not release prerequisites.

# Execution phases

## PA-40A — Accept usability and WYSIWYG fidelity

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: [accepted isolated technical chain](../_assets/sharepoint/intake-approval/acceptance/end-to-end.md); two approved participants who did not build the apps (the owner is arranging them); and a read-only path to the accepted Approved batch for exact canvas-to-PDF comparison. The [November 11 acceptance record](../_assets/sharepoint/intake-approval/acceptance/usability.md) preserves the automated checkpoint and published TEST ONLY UX repairs. The updated participant guide maps the published controls and clearly labels older reference screenshots.
- Exact write set: TEST ONLY apps `[PROTOTYPE] Intake Approval` and `Sponsor Highlights Intake UI TEST ONLY`; TEST ONLY component library `Sponsor Highlights UI TEST ONLY` used by the contributor app; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/acceptance/usability.md` and its participant guide and rollback copies; `.wiki/features/powerapps-jsonl-intake/pilot-runbook.md`
- Primary integration lock: Power Platform user acceptance
- Outcome: Accept both primary user journeys, cross-app consistency, and preview-to-PDF expectations with no material coaching or semantic discrepancy. Further accessibility features and checks remain deferred by user direction.
- Focused validation: Two uncoached walkthroughs and task/error observations; post-repair App Checker and desktop/tablet/375-pixel visual checks; live review-save behavior; and page-by-page Approved-batch PDF comparison. Fresh durable player captures remain unavailable through the browser; the guide uses owner-approved labeled references.
- Handoff: Record signed go/no-go, accepted renderer differences, required support wording, and any bounded repair phases. PA-41 remains blocked without go.

## PA-41 — Back up and cut over the operational approval gate

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-40A accepted; denied-role, unchanged-window late-arrival, and connector-level lost-response evidence; explicit owner authorization; named operational reviewer/finalizer/contributor groups; fresh capacity/licensing/DLP check; scheduled change window
- Exact write set: operational apps `Intake Approval` and `Sponsor Highlights Intake`; operational review lists/library and approval flows; operational collected exporter; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/cutover/backup-manifest.json`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/cutover/baseline.json`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/cutover/rollback.md`
- Primary integration lock: Power Platform deployment
- Outcome: Capture recoverable native backups and exact baseline, deploy least-privilege review systems and consistent apps, disable the old exporter, and activate approval-gated publication.
- Focused validation: Backup hashes/import inspectability; connection/destination check; component revisions; group access/denial; no overlapping exporter; exact baseline; trigger state; immediate freeze rollback rehearsal before smoke publication.
- Handoff: Preserve component/app/flow versions, list/library IDs, connection references, audiences, baseline, backups, and rollback authority.

## PA-41A — Verify production smoke, operations, and handoff

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-41
- Exact write set: operational intake/approval apps and approval-aware exporter; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/cutover/production-acceptance.md`; `.wiki/features/powerapps-jsonl-intake/highlights-hub.md`; `.wiki/features/powerapps-jsonl-intake/notification-walkthrough.md`; `.wiki/features/powerapps-jsonl-intake/pilot-runbook.md`; current SharePoint guide publications
- Primary integration lock: Power Platform production acceptance
- Outcome: Prove one operational receipt remains out while unresolved, enters exactly once after Approved finalization, produces expected local PDF content, and leaves operators with accurate monitoring/support/rollback guidance.
- Focused validation: Intended-user access; reviewer/finalizer denial matrix; receipt/frozen/collected exact bytes; exporter freshness; Teams wording; hub/guide links; repeated unchanged run; freeze-only rollback readiness; no old exporter trigger.
- Handoff: Preserve operational acceptance and release PA-12 only when every gate passes. Remove completed phases from their plans after durable evidence is published.
