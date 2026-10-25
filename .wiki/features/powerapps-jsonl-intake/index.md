---
type: Plan
title: "PowerApp and Power Automate JSONL intake"
description: "Accept non-sensitive highlights into GitLab JSONL with confirmed receipts, then complete independent intake hardening and shared deployment."
difficulty: medium
impact: critical
package: [workspace]
tags: [powerapps, powerautomate, gitlab, jsonl, intake]
status: in-progress
priority: next
execution: phased
phase_budget_hours: 5
---

# Goal

Finish reliability, browser/boundary/concurrency acceptance, restoration and later shared deployment for the published Sponsor Highlights form. The recorded single-user demo already covers project selection, non-sensitive submission and exactly one confirmed GitLab JSONL record. A fresh live demonstration in the week of October 26, 2026 still depends on approved access. Neither Vergil nor an external callback endpoint is required for form submission.

Supporting material: [pilot runbook](pilot-runbook.md), [notification walkthrough](notification-walkthrough.md), and [source bundle](source-bundle-readme.md). Form sources, PDFs and acceptance files are in `_assets/`.

# Current evidence

- October 24 notification walkthrough passed: a fresh published LandScan submission added one JSONL line at `03ff5836289447034f40f2a4898c155441e6cb66` and sent the confirmed-save Teams notice to Jason. The app flow binding was refreshed and republished. Routine callback replay skipped delivery; a milestone email test passed. [Current routing, screenshots, run IDs, and limits](notification-walkthrough.md) are durable evidence. Intake notices are best effort after confirmation; this does not close PA-02 or wider release gates.

- The isolated saved-flow copy passed simulated failed-commit acceptance in run `08584087221220248940052484856CU15`: commit Failed, readback/confirmation/success response Skipped, one outer unresolved response Succeeded. The live flow was unchanged; timeout/skipped dependencies were checked against preserved source. See [failure acceptance](write-failure-acceptance.md).

- October 24 demo acceptance captured UUID `4313fcfc-0201-441e-8bda-c77718a31dcf` at commit `3090aa5714b4c6ff59c5dd0b6ecba999f918caa2` (+1/-0), preserved all six exact JSONL lines, and replayed through Queued replay in run `08584087227067729212084836766CU19`. The same record completed local ingestion, reviewed graduation and PDF rehearsal; see the [demo packet](../manual-jsonl-ingestion-graduation/walkthrough.md). This dated success does not close PA-02.

- Code and saved Power Platform sources reviewed on October 24, 2026 against `49b4700d` plus the current uncommitted intake changes. Cloud statements below describe retained acceptance evidence, not a new live inspection. The [pilot runbook](pilot-runbook.md) and [notification walkthrough](notification-walkthrough.md) preserve the deployed-revision evidence.
- The ORNL-branded app, project catalog, result panel, and browser-local pending slot are published in **Wohlgemuth, Jason**, environment `221f44a6-1468-ea4e-b93d-cd52f1723079`. Jason Wohlgemuth is the operational owner. The button displays Load Project List before loading and Refresh Project List after a successful load.
- October 24 visual polish is published and verified: unclipped Title Case heading, `//` subsection separators, and padded submit footer with ACORN/ORNL and owner attribution. Desktop and 360-pixel phone previews passed visual inspection; formula and accessibility checks found no errors. Updated source and screenshots are in the runbook assets. This presentation-only change does not complete the remaining intake acceptance phases.
- Intake `ff4a134b-5fd3-4465-ab19-0325c2f2bf1b` and catalog `cd27888f-dd74-4675-9dd1-abc246ba56d5` are enabled with protected connections. Target: project `22464`, `main`, `.acorn/intake/logbook.jsonl`. ACRE `folders` is authoritative when present; otherwise discover `index.json`/`index.yaml` folders. Catalog membership does not establish ingestion authorization.
- Fresh append, immutable replay, caller/project rejection, interrupted-response reconciliation, reload and app-tab recovery passed. The latest retained append proof is `03ff5836289447034f40f2a4898c155441e6cb66`: seven TEST ONLY lines, all previous bytes preserved, and exact replay without another append. Earlier five- and six-line snapshots remain historical fixtures. Isolated server over-limit cases passed.
- Intermittent explicit network-range 403s still affect reads and writes through the unchanged connector. A successful read or isolated append does not prove reliable egress. Credential rotation and binding are verified; the [runbook](pilot-runbook.md) contains administrator correlation evidence.
- The duplicate failure response is removed. Its isolated persistence check had zero errors/warnings; the later Teams-enabled flow checker recorded zero errors and nine connection-change warnings, followed by app binding refresh, publication and successful submission; isolated cloud failed-commit acceptance now passes; see [failure evidence](write-failure-acceptance.md). Complete solution export failed because the custom connector could not be included. Full browser-process restart, published storage denial, exact-limit/malformed-tail/size and concurrent-write cases remain. Failed schema output may echo values, so the pilot remains non-sensitive only.
- [User/reviewer guide](_assets/sponsor-highlights-user-reviewer-guide.pdf) and [developer guide](_assets/sponsor-highlights-developer-guide.pdf) document the verified pilot, with screenshots. They are evidence snapshots, not proof that later integration is complete.

# Boundaries and decisions

Creation/update means creating a missing inbox or conditionally appending to an existing file while preserving existing bytes. It does not mean editing a previously accepted record under the same submission ID. Keep frozen content and UUID across retries, disabled automatic write retries, immutable commit proof, and an unresolved outcome when proof is unavailable. Do not bypass access denials.

The demo uses one browser tab, one contributor and sequential submissions in the existing pilot. Local browser recovery is approved for non-sensitive data only. A queued receipt means persisted in GitLab, not admitted, graduated, merged, or reported. The completed [manual demo](../manual-jsonl-ingestion-graduation/index.md) covers local ingestion, graduation and PDF rendering; it does not merge or publish canonical production changes. Local CLI commands do not automatically invoke the manual notification flow. The [Vergil plan](../vergil-gitlab-jsonl-automation/index.md) owns authenticated external callbacks and unattended processing.

# Dependencies and acceptance

**Demo gate:** The fresh personnel capture, exact replay and immutable snapshot handoff are preserved in the [demo packet](../manual-jsonl-ingestion-graduation/walkthrough.md). The repaired single unresolved response passed isolated cloud fault injection; [evidence](write-failure-acceptance.md) records the runtime and limits. A live demo also needs successful approved access at rehearsal. If denied, use the exact previously captured record in the manual demonstration and label the intake segment as recorded evidence; do not claim a fresh live save. PA-02 remains open until reliable access and final-flow licensing have dated owner evidence.

**Wider release gate:** PA-02, PA-14 through PA-17, and PA-12 cover remaining acceptance, backup, and shared deployment. None is silently declared complete by a successful single-user demo. PA-12 is a future shared release and does not gate the current pilot. Preserve original PA phase IDs; the completed demo capture is recorded in durable evidence; independent remaining matrices retain their bounded phase IDs.

# Validation

Inspect actual action outcomes, line deltas, receipt UUID/commit, immutable snapshots and replay results. Record exact flow revisions and run IDs in the runbook. No Rust changes or full workspace tests are needed for these operational phases. Keep TEST ONLY evidence out of sponsor reports. On each completion, prune its phase and reconcile this plan, indexes, network and lane.

# Execution phases

## PA-02 — Resolve intake access and pilot licensing

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: ORNL administrator/network owner and Power Platform licensing authority available
- Exact write set: Power Platform pilot licensing records; project `22464` access-policy records; `.wiki/features/powerapps-jsonl-intake/intake-access-decisions.md`
- Primary integration lock: deployment approvals
- Outcome: Identify the denying control and approved connector egress route; record accountable owners, dates and licensing evidence. Callback/Vergil decisions belong to BOT-01.
- Focused validation: Correlate successful and denied run timestamps/IPs; verify approved read, conditional append and exact replay through existing protected connections. Never expose credentials.
- Handoff: Release access-dependent intake acceptance. Future shared-environment approval stays in PA-12.

## PA-14 — Accept browser recovery and result panels

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: published app; non-sensitive controlled browser tests; preserve user credential-display pages and unsaved work
- Exact write set: Sponsor Highlights recovery/result formulas; `.wiki/features/powerapps-jsonl-intake/browser-recovery-acceptance.md`; `.wiki/features/powerapps-jsonl-intake/_assets/browser-recovery/`
- Primary integration lock: intake route
- Outcome: Finish full browser-process restart, published storage-denial and conflict/error panel acceptance within the single-tab contract.
- Focused validation: Retain identical frozen envelope/ID after restart; storage failure blocks send; current error/conflict panel offers only the correct action. Do not claim cross-device or concurrent-tab support.
- Handoff: Record screenshots and run IDs; any newly found defect gets its own bounded repair phase.

## PA-15 — Accept bounded inbox and field limits

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: PA-02; safe isolated fixtures and restored baseline
- Exact write set: pilot project `22464` isolated intake test files; existing intake validation actions; `.wiki/features/powerapps-jsonl-intake/boundary-acceptance.md`
- Primary integration lock: intake route
- Outcome: Complete exact field limits, blank/rejected-line preservation, malformed tail and file-size boundary acceptance.
- Focused validation: Use one dimension per case; accepted records append once, rejected records do not alter any bytes, unsafe tails fail closed; preserve original pilot inbox.
- Handoff: Record per-case outcomes and restore isolated fixtures; no implied concurrency acceptance.

## PA-16 — Accept concurrent inbox creation and append

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: PA-02, PA-15; approved isolated source files
- Exact write set: pilot project `22464` isolated concurrency test files; existing intake conditional-write actions; `.wiki/features/powerapps-jsonl-intake/concurrency-acceptance.md`
- Primary integration lock: intake route
- Outcome: Prove simultaneous creators/writers do not overwrite lines or duplicate the same submission.
- Focused validation: Inspect competing revisions, conditional-write failure/reconciliation and immutable receipts; preserve one-write limit and unresolved results when history proof is bounded. No Vergil cleanup runs in this phase.
- Handoff: Preserve concurrency evidence; append-versus-bot-cleanup belongs to BOT-02.

## PA-17 — Produce a restorable Power Platform backup

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: existing source bundle; access to pilot solution and connector authoring
- Exact write set: pilot solution `ACORNSponsorHighlightsIntake` and associated custom connector; `.wiki/features/powerapps-jsonl-intake/_assets/solution-export/`
- Primary integration lock: Power Platform packaging
- Outcome: Resolve the connector inclusion/export issue and produce a complete importable package with dependency and restore instructions.
- Focused validation: Inspect exported app, flows, connector and connection references; verify no secret values; perform import validation in an authorized disposable target when available, otherwise record the export as partial evidence and keep PA-17 open until restore verification is accepted.
- Handoff: Record export version, hashes and restore result. A source ZIP alone does not satisfy this phase.

## PA-12 — Move the accepted pilot to an approved shared environment

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: PA-02, PA-14, PA-15, PA-16 and PA-17 accepted; named approved GCC environment, maker/sharing rights, licensing, owner and recipient group/channel
- Exact write set: target Power Platform solution import, protected connections, app/flow sharing and notification destination; `.wiki/features/powerapps-jsonl-intake/shared-deployment.md`
- Primary integration lock: Power Platform deployment
- Outcome: Deploy the accepted app/flows and switch future notifications from Jason to the approved shared destination.
- Focused validation: Verify catalog, append/replay and notification deduplication under intended user permissions; preserve old receipt recipients and rollback; a new Teams channel has its own delivery state.
- Handoff: Preserve destination IDs and protected reference names; prune only after target acceptance. This is outside next week's pilot MVP.
