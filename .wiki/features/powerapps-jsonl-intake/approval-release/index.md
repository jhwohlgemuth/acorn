---
type: Plan
title: "Intake approval pilot acceptance and cutover"
description: "Accept the complete contributor-to-report workflow in isolation, then deploy the consistent apps and approval-gated exporter with explicit operational authorization and rollback."
difficulty: high
impact: critical
package: [power-platform, sharepoint, workspace]
tags: [powerapps, powerautomate, acceptance, deployment, rollback, pdf]
status: in-progress
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

- Every release-matrix case passes with exact run IDs, revisions, bytes/digests, screenshots, PDF, and authorization evidence.
- Both apps use one accepted component revision and pass usability gates. Further accessibility work is deferred by user direction.
- Actual PDF comparison has no semantic mismatch.
- Operational baseline and rollback artifacts are complete before activation.
- One production smoke receipt remains excluded from the collected file while its review decision is unresolved, then appears exactly once after approval and batch finalization.
- Notifications and guides say `received`, `ready for review`, `approved`, and `published to collected export` accurately; no earlier state claims report publication.
- Exporter monitoring names the last included Approved batch and freshness.
- Old unfiltered export cannot run after cutover.

# Current PA-40 evidence and boundaries

The [November 10 end-to-end checkpoint](../_assets/sharepoint/intake-approval/acceptance/end-to-end.md) verifies contributor project, General, and same-batch correction receipts through the isolated PA31 source and materializer. The saved, unpublished reviewer WYSIWYG canvas reads live Ready-batch rows and saves decisions through the optimistic-concurrency flow. Clean project/General and same-batch correction batches were each prepared, finalized, exported with exact frozen bytes, staged and applied locally, and compared against three-page PDFs. The correction revision names its newly approved root ID; the local report shows only the effective revised wording. The correction root was approved without an intentional edit, but its reviewer-saved draft differed at the byte level, so the finalizer issued a new UUID under the accepted identity contract; unchanged activity remains unverified. The TEST ONLY preparation/finalizer definitions now trust PA31; the finalizer includes the LF-aware 10 MiB condition. An ETag-stale reviewer save was rejected; after reloading and comparing the server row, the retry saved with exact reread. The first project/General batch retained a stale high-water field after zero-row recovery and must not be finalized. Live status, the remaining release matrix, and durable live-report captures remain open. The user directed that the unauthorized-role denial check use no second account and be recorded as unverified. PA-40 therefore remains blocked under its current criterion.

# Dependencies and boundaries

PA-40 requires accepted PA-37B and PA-40S, [PA-38B TEST ONLY export acceptance](../_assets/sharepoint/intake-approval/exporter/failure-acceptance.md), the [accepted cross-app experience](../_assets/sharepoint/intake-approval/acceptance/cross-app-ui.md), and the PA-40V integration repair below. The unauthorized-role row additionally requires a distinct denied identity or an explicit revision of the release criterion; it cannot be marked passed with the owner account. PA-41 requires PA-40A plus explicit operational authorization. This plan does not authorize wider shared-environment migration; PA-12 remains separate. Direct OAuth/CI download and Vergil automation are not release prerequisites.

# Execution phases

## PA-40V — Bind the report canvas to live review decisions

- Status: in-progress
- Budget: 5 active hours maximum
- Prerequisites: accepted [source-binding evidence](../_assets/sharepoint/intake-approval/acceptance/end-to-end.md) and [optimistic-concurrency save evidence](../_assets/sharepoint/intake-approval/finalization/save-acceptance.md)
- Exact write set: saved, unpublished `[PROTOTYPE] Intake Approval` TEST ONLY app, its TEST ONLY review read/save flow bindings, and `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/acceptance/end-to-end.md`
- Primary integration lock: Power Platform reviewer authoring
- Outcome: The report canvas previews and edits live Ready-batch rows, and its decisions use the accepted optimistic-concurrency save path.
- Focused validation: Live source comparison, edited activity, status and General placement, excluded reason/visibility, saved ETag/version, stale-writer recovery, and desktop/375-pixel report captures; retain the local UI fixture separately.
- Handoff: Preserve app/flow revisions and exact decision evidence before PA-40 resumes.

## PA-40 — Accept functional end-to-end report authoring

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: accepted [source-binding evidence](../_assets/sharepoint/intake-approval/acceptance/end-to-end.md) and PA-40V; accepted [TEST ONLY finalization evidence](../_assets/sharepoint/intake-approval/finalization/finalization-acceptance.md); accepted [TEST ONLY export evidence](../_assets/sharepoint/intake-approval/exporter/failure-acceptance.md); [accepted cross-app experience](../_assets/sharepoint/intake-approval/acceptance/cross-app-ui.md); named test contributor and reviewer; isolated ACORN workspace; distinct denied-role identity or explicit criterion revision
- Exact write set: TEST ONLY approval/intake apps, lists, flows, component library, and export library; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/acceptance/end-to-end.md`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/acceptance/accepted.jsonl`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/acceptance/accepted.pdf`
- Primary integration lock: Power Platform intake acceptance
- Outcome: Execute the release matrix through submission, materialization, review, finalization, approved export, local intake, and combined PDF without production changes.
- Focused validation: Exact IDs/bytes/digests/counts; unauthorized roles; replay; stale ETag; late source; correction mapping; exclusion; uncertain write; local stage/inspect/apply; PDF text/order/visibility comparison.
- Handoff: Preserve complete technical evidence and defects. Semantic mismatch or unsafe recovery blocks PA-40A.

## PA-40A — Accept usability and WYSIWYG fidelity

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-40; approved uncoached contributor and reviewer participants
- Exact write set: TEST ONLY apps `[PROTOTYPE] Intake Approval` and `Sponsor Highlights Intake UI TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/acceptance/usability.md`; `.wiki/features/powerapps-jsonl-intake/pilot-runbook.md`
- Primary integration lock: Power Platform user acceptance
- Outcome: Accept both primary user journeys, cross-app consistency, and preview-to-PDF expectations with no material coaching or semantic discrepancy. Further accessibility features and checks remain deferred by user direction.
- Focused validation: Two uncoached walkthroughs, task/error observations, formula/runtime App Checker, desktop/tablet/375-pixel captures, and page-by-page PDF comparison.
- Handoff: Record signed go/no-go, accepted renderer differences, required support wording, and any bounded repair phases. PA-41 remains blocked without go.

## PA-41 — Back up and cut over the operational approval gate

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-40A accepted; explicit owner authorization; named operational reviewer/finalizer/contributor groups; fresh capacity/licensing/DLP check; scheduled change window
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
