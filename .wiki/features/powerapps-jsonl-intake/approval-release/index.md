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

- PA-40 technical acceptance preserves exact run IDs, revisions, bytes/digests, and PDF evidence. The owner allowed one-account authorization evidence to remain unverified and accepted the controlled source-change guard plus a post-write failure proxy for PA-40 only. Full release requires PA-40D denied-role, PA-40E unchanged-window late-arrival, and PA-40F connector-level lost-response service acceptance before operational cutover.
- Both apps use one accepted component revision and pass usability gates. Further accessibility work is deferred by user direction.
- Actual PDF comparison has no semantic mismatch.
- Operational baseline and rollback artifacts are complete before activation.
- One production smoke receipt remains excluded from the collected file while its review decision is unresolved, then appears exactly once after approval and batch finalization.
- Notifications and guides say `received`, `ready for review`, `approved`, and `published to collected export` accurately; no earlier state claims report publication.
- Exporter monitoring names the last included Approved batch and freshness.
- Old unfiltered export cannot run after cutover.

# Current evidence and boundaries

The [November 10 end-to-end acceptance](../_assets/sharepoint/intake-approval/acceptance/end-to-end.md) records the isolated contributor -> review -> frozen approval -> collected JSONL -> local PDF chain for project, General, status, corrections, unchanged approval, and exclusion. ETag conflict/recovery and exact native exporter restoration passed. Its final [JSONL](../_assets/sharepoint/intake-approval/acceptance/accepted.jsonl) has 11 lines and 5,434 bytes; the [PDF](../_assets/sharepoint/intake-approval/acceptance/accepted.pdf) has four pages. The one-account authorization exception, changed-source-window check, and post-write failure proxy completed PA-40 technical acceptance only. PA-40D/E/F must supply the missing release evidence. The recovered first project/General batch has a stale high-water field and must not be finalized. Materialization, preparation, finalization, and collected export remain Off outside controlled acceptance runs.

The [November 12 beta evidence](../_assets/sharepoint/intake-approval/beta-repair/verification.md) records published contributor correction/UI repairs, Standard reviewer designation, repaired app sharing, and SharePoint-only app-facing summary requests with AI in a separate owner-run queue processor. Owner-context save and summary checks passed; participant launch is owner-confirmed. Participant correction, batch read/save, automatic summary, and contextual regeneration remain unverified. The four participant originals and Ready batch are preserved. PA-40A still needs uncoached observations and exact Approved-batch canvas/PDF comparison; successful launch or Standard designation alone does not satisfy it.

November 24 review: retained summary flow definitions confirm the app-facing SharePoint connection and separate background SharePoint/Dataverse connections; the saved final JSONL still has 11 valid JSON records and 5,434 bytes. This is repository evidence, not a fresh tenant, licensing, or participant acceptance run. Native recovery artifacts must be resolved and revalidated under the [sanitized-copy limits](../../../privacy.md) before reuse.

# Dependencies and boundaries

PA-40 technical acceptance used accepted PA-37B, PA-40S, [PA-38B TEST ONLY export acceptance](../_assets/sharepoint/intake-approval/exporter/failure-acceptance.md), and the [accepted cross-app experience](../_assets/sharepoint/intake-approval/acceptance/cross-app-ui.md). Its one-account and fault-substitution exceptions do not carry into PA-41 operational cutover. PA-41 requires PA-40A and PA-40D/E/F acceptance plus explicit operational authorization. PA-40D/E/F may proceed when their own external prerequisites are met, independently of participant walkthrough scheduling; all PA-40 acceptance work serializes on the Power Platform user acceptance lock. Material defects become separate bounded repair phases; unavailable service-injection methods remain blockers rather than substituted passes. This plan does not authorize wider shared-environment migration; PA-12 remains separate. Direct OAuth/CI download and Vergil automation are not release prerequisites.

# Execution phases

## PA-40B — Validate participant intake and correction repairs

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: c5c available to retry a correction in the published beta; participant retesting is deferred until the owner is ready. Owner-context BDI creation/correction and desktop/375-pixel UI checks passed; see [beta evidence](../_assets/sharepoint/intake-approval/beta-repair/verification.md).
- Exact write set: contributor app `427852d5-c8a6-4218-ab2d-7db34e1106cc` and bound intake/catalog flows only if the participant retest exposes a defect; acceptance/usability.md and beta-repair/verification.md under `../_assets/sharepoint/intake-approval/`; this plan and affected indexes.
- Primary integration lock: Power Platform user acceptance
- Outcome: Verify the participant can edit her submission, receive a new correction receipt, and recover from a validation/conflict response without changing the original record.
- Focused validation: Participant-account edit/save/reopen and receipt ancestry; preserve exact failure text if any. Existing owner-context and structural guard evidence does not stand in for this retest.
- Handoff: Record participant result and remove this phase after acceptance, or isolate any new defect into a bounded repair.

## PA-40C — Validate standard-license reviewer access

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: Niki available to test her four-item Ready batch; the owner confirmed that the reviewer form opens on November 12. Published maker designation is Standard; owner-context read/save and automatic/contextual generation passed. App sharing now includes Niki as User after the owner clarified the latest launch failure may be permission-related. Power Apps confirmed the permission save and the owner confirmed successful launch. Batch read/save and automatic/contextual summary results remain pending.
- Exact write set: reviewer app `ef761a81-bd35-4854-8de0-f91a452ba73f` and its five SharePoint-facing flows only for a demonstrated access defect; current participant evidence and AI contract under `../_assets/sharepoint/intake-approval/`; this plan and affected indexes.
- Primary integration lock: Power Platform user acceptance
- Outcome: Establish participant launch/read/edit/save and retained AI use without a premium user license or trial. Resolve any remaining tenant-specific licensing or policy requirement explicitly.
- Focused validation: Intended-user four-item batch read, decision save, automatic summary and contextual regeneration; successful launch is already owner-confirmed. Preserve any exact licensing/policy error. Historical maker connection listings are not the current execution-path audit.
- Handoff: Record intended-user acceptance; no inference from the owner’s successful runs or the Standard designation alone.

## PA-40A — Accept usability and WYSIWYG fidelity

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: PA-40B and PA-40C participant validation; [accepted isolated technical chain](../_assets/sharepoint/intake-approval/acceptance/end-to-end.md); approved non-builder contributor/reviewer walkthroughs; and a read-only path to the accepted Approved batch for exact canvas-to-PDF comparison. The [acceptance record](../_assets/sharepoint/intake-approval/acceptance/usability.md) preserves the four participant receipts and Ready batch. The published reviewer is now Standard; intended-user access remains to be retested. The refreshed participant guide uses current labeled redrawings.
- Exact write set: beta apps `[BETA] Sponsor Highlights Review` and `[BETA] Sponsor Highlights Intake`; shared component library `[BETA] Sponsor Highlights UI`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/acceptance/usability.md` and its participant guide and rollback copies; `.wiki/features/powerapps-jsonl-intake/pilot-runbook.md`
- Primary integration lock: Power Platform user acceptance
- Outcome: Accept both primary user journeys, cross-app consistency, and preview-to-PDF expectations with no material coaching or semantic discrepancy. Further accessibility features and checks remain deferred by user direction.
- Focused validation: Two uncoached walkthroughs and task/error observations; post-repair App Checker and desktop/tablet/375-pixel visual checks; live review-save behavior; and page-by-page Approved-batch PDF comparison. Fresh durable player captures remain unavailable through the browser; the guide uses owner-approved labeled references.
- Handoff: Record signed go/no-go, accepted renderer differences, required support wording, and any bounded repair phases. PA-41 remains blocked without go.

## PA-40D — Accept contributor, reviewer, and finalizer authorization

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: approved distinct test identities and role assignments for the isolated beta route; access to connector/flow invocation and audit evidence; exact app/flow bindings and fixture destinations rechecked. The historical owner-only exception does not satisfy this gate.
- Exact write set: named isolated external systems — `[BETA] Sponsor Highlights Intake`, `[BETA] Sponsor Highlights Review`, their bound intake/read/save flows, and the isolated preparation/finalizer flows against PA31 TEST ONLY receipts and review storage; isolated test-user app/run-only/group assignments; `../_assets/sharepoint/intake-approval/acceptance/authorization.md`; this plan and affected indexes.
- Primary integration lock: Power Platform user acceptance
- Outcome: Establish service-enforced contributor ownership and distinct reviewer/finalizer authority without broadening operational access.
- Focused validation: Approved positive controls and denied unauthenticated/unauthorized or wrong-role calls; another contributor cannot correct a receipt, a contributor cannot review, and a reviewer without finalizer authority cannot prepare/finalize. Attempt service calls independently of hidden UI; compare receipt/review/frozen/export state before and after denials and verify no unauthorized effect.
- Handoff: Preserve role matrix, exact flow/app revisions, redacted identities, run/error evidence, unchanged protected state, and restored test assignments. Keep this phase open for any untested denial.

## PA-40E — Accept late source detection within an unchanged closed period

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: authorized isolated PA31 source/review fixture; an approved reproducible method for delayed visibility or backdated TEST ONLY Created timestamps; fresh native backup and restoration path. No demonstrated unchanged-window injection method is currently recorded.
- Exact write set: named isolated external systems — PA31 TEST ONLY receipts list `31a7f178-6729-4252-a2bc-e06a07838f5c`, its isolated materializer/preparation/finalizer and disposable review batch/frozen destination; `../_assets/sharepoint/intake-approval/acceptance/late-arrival.md`; this plan and affected indexes.
- Primary integration lock: Power Platform user acceptance
- Outcome: Prove a newly visible in-period receipt above the captured source watermark blocks finalization without changing the closed period or publishing a partial batch.
- Focused validation: Capture period, source IDs/watermark, draft/frozen state and collected bytes; introduce the approved late-visible receipt while preserving the exact period; invoke finalization and verify rejection before commit/publication, no Approved state, and unchanged collected bytes. An administrative period change does not pass this gate.
- Handoff: Record the injection method, source visibility/timestamps, exact before/after state and run IDs; restore fixtures and leave controlled flows Off. If the service cannot reproduce the condition, retain the blocker without replacing it with the prior proxy.

## PA-40F — Accept connector-level lost-response reconciliation

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: approved reproducible connector-level lost-response/timeout method in the isolated exporter; verified Approved fixture and exact prior collected file; fresh native backup with inspectable restore. The prior failing action after a successful write is not this evidence.
- Exact write set: named isolated external systems — TEST ONLY collected exporter `6f74ecff-bfcd-4377-afdf-39b03b4beff1`, its isolated collected file and freshness receipt; `../_assets/sharepoint/intake-approval/acceptance/lost-response.md` and native backup/restore artifacts under `../_assets/sharepoint/intake-approval/exporter/`; this plan and affected indexes.
- Primary integration lock: Power Platform user acceptance
- Outcome: Prove an uncertain SharePoint write is reconciled from exact reread without blind duplicate publication or a false success receipt.
- Focused validation: Exercise a write whose connector response is genuinely lost/uncertain; independently inspect destination bytes/revision and flow actions. Accept success only after exact expected bytes are read back; otherwise retain a failure/uncertain outcome without claiming publication. Replay must not duplicate content. Restore and compare the full native action graph, trigger, and connection references, leaving the exporter Off.
- Handoff: Preserve connector fault evidence, run IDs, exact bytes/revisions, receipt outcome, replay result, and verified restore. If connector-level injection is unavailable, keep this gate blocked; a post-write Compose failure remains insufficient.

## PA-41 — Back up and cut over the operational approval gate

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-40A, PA-40D, PA-40E, and PA-40F accepted; explicit owner authorization; named operational reviewer/finalizer/contributor groups; fresh capacity/licensing/DLP check; scheduled change window
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
