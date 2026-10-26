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

- PA-15 October 25 boundary acceptance: exact 1,200-character summaries and 1,000-character details passed in both environments; Jason also passed the 20-detail count. All listed field rejections passed. The latest series added ORNL detail acceptance, then three attempts each for ORNL count, mixed-line and tail cases stopped at GitLab 403. Independent file-read timeout prevented the next size-fixture setup. Both test flows returned Off and two readbacks verified the original 457-byte baseline. Remaining acceptance covers ORNL 20 details plus mixed-line, unsafe-tail and size boundaries in both environments; [case matrix and byte proofs](boundary-acceptance.md). Completed simulation artifacts were removed separately; [cleanup inventory](test-artifact-cleanup.md).

- PA-02 is closed by owner decision on October 25, 2026: the Jason-only pilot licensing assessment and access characterization are accepted, with intermittent GitLab connectivity recorded as a [known limitation](intake-access-decisions.md#known-limitation-and-pa-02-closure). Reliable connectivity has not been proven or repaired. PA-15/16/17 retain their own unpassed acceptance tests.

- PA-02 October 25 evening acceptance: Jason's assigned Power Apps Per User Plan for Government and both test flows' invoking-user context were rechecked. App-driven catalog calls passed in both environments. ORNL restored-app intake reached the correct isolated flow, passed branch/history/catalog checks, then received a network-range HTTP 403 at `Read current PA17v2` (`2026-10-25T22:21:18Z`). The app retained an unresolved receipt; the write branch was skipped and test intake returned Off. The scoped pilot licensing assessment is complete; repeatable read/write/replay access is not accepted. [Dated test evidence](intake-access-decisions.md#october-25-evening-pa-02-acceptance).

- PA-17 handoff: native archive hashes and ZIP integrity reverified October 25. [Consolidated restore handoff](_assets/solution-export/PA17-RESTORE-HANDOFF.md) separates operational backups from isolated import inputs and lists only the remaining acceptance. PA-17 is waiting on validation, with execution blocked by the observed GitLab network-range write denial and missing ORNL `prvCreateEntity`; it has not been marked complete or reduced in scope.

- October 25 native canvas recovery progressed: a fresh unchanged operational app package was downloaded read-only. Native Save As created **PA17 TEST ONLY Sponsor Highlights** (`b4336414-edfb-4aae-b4e3-d8b994c19b65`), bound only to test flows with separate `PA17SponsorHighlightPending` storage. The saved/published copy lists all 13 projects. Native solution `1.0.0.6` includes this app, both flows, connector and two references; archive checks found zero missing dependencies and the solution checker reported zero issues. ORNL v6 import succeeded at 3:18–3:19 PM; the restored app also lists all 13 projects through a successful app-driven catalog run. Fresh app-driven writes and callback/table restoration remain unaccepted. Test intake remains Off. See [native canvas checkpoint](restore-target-readiness.md#native-canvas-and-solution-1006-checkpoint).

- October 24–25 isolated work restored a connector-only solution with byte-identical catalog code, exported a native callback/table backup, and exported isolated connector routes. Jason’s protected test connection passed branch/catalog reads; the missing isolated file returned HTTP 404. Both edited app-package revisions failed import and are retired from deployment attempts. A natively authored catalog flow passed, then its connector call passed again after solution conversion. PA-18 repaired the reference through native solution authoring: all three catalog steps passed, and native export `1.0.0.4` succeeded with verified dependencies, isolated routes and unchanged custom code. Native 1.0.0.4 then imported into ORNL, bound its protected test connection and returned HTTP 200 with all 13 projects; all three steps passed. PA-18 is accepted and pruned; PA-17 remains blocked on app-driven intake and callback/table restore acceptance. ORNL callback/table import subsequently confirmed the privilege limit: `prvCreateEntity` is missing; package schema validation passed but the components and activation remained Unprocessed. See [target readiness, exact run outcomes and archive hashes](restore-target-readiness.md). Operational components were unchanged.
- October 25 PA-17 native intake/catalog export `1.0.0.5` and a separate native callback/table test export `1.0.0.1` succeeded with zero missing dependencies and isolated destinations. Jason callback first receipt, exact replay and payload-conflict tests passed with one unchanged receipt row and all Teams/email actions skipped. Jason native manual intake also created one isolated JSONL line and replayed without a second commit or Teams notice. A subsequent intake conflict attempt hit an explicit network-range 403 before persistence; the unresolved response passed, but conflict acceptance remains open. Both tested write-capable flows were returned to Off. The later v6 checkpoint supplies the app copy/export and source catalog acceptance; app-driven submission and ORNL full restoration remain pending; ORNL callback/table import is blocked by missing `prvCreateEntity`, and the table-free intake/catalog version 1.0.0.5 import succeeded in ORNL (2:32–2:33 PM local display). Restored ORNL manual replay and identity-conflict rejection passed; a fresh commit was denied by the same explicit network-range restriction after successful reads. The test intake returned Off with the 457-byte logbook unchanged. Fresh ORNL write and calling-app acceptance remain pending. PA-17 stays open. See [native package inventory and acceptance](restore-target-readiness.md#native-package-inventory-and-restore-order).
- Jason's Power Apps Per User Plan for Government is directly observed. The owner has no administrator ticket and requests using existing access. [License/access evidence](intake-access-decisions.md) supports the Jason-only app-triggered pilot; repeatable egress remains unverified. No administrator ticket was required for isolated testing; connectivity is now an accepted pilot limitation.
- PA-14's simulated-response package imported as a new app and two new flows in both requested environments. Studio exposed stale intake references despite remapped outer dependencies; both test-app bindings and their formula aliases were repaired before submission. Both published test copies passed queued, rejected, unresolved and conflict panels, plus tab-reload recovery and same-ID retry. The original simulation rules were restored after conflict testing. On October 25 the owner confirmed browser-restart recovery in both environments, with fields locked and retry retaining each original receipt ID. The owner subsequently accepted denied-storage testing in both environments; PA-14 is closed on the combined agent-observed and owner-reported evidence. See [browser recovery checkpoint](browser-recovery-acceptance.md). Operational components were unchanged.

- October 24 notification walkthrough passed: a fresh published LandScan submission added one JSONL line at `03ff5836289447034f40f2a4898c155441e6cb66` and sent the confirmed-save Teams notice to Jason. The app flow binding was refreshed and republished. Routine callback replay skipped delivery; a milestone email test passed. [Current routing, screenshots, run IDs, and limits](notification-walkthrough.md) are durable evidence. Intake notices are best effort after confirmation; this does not establish reliable connectivity or close wider release gates.

- The isolated saved-flow copy passed simulated failed-commit acceptance in run `08584087221220248940052484856CU15`: commit Failed, readback/confirmation/success response Skipped, one outer unresolved response Succeeded. The live flow was unchanged; timeout/skipped dependencies were checked against preserved source. See [failure acceptance](write-failure-acceptance.md).

- October 24 demo acceptance captured UUID `4313fcfc-0201-441e-8bda-c77718a31dcf` at commit `3090aa5714b4c6ff59c5dd0b6ecba999f918caa2` (+1/-0), preserved all six exact JSONL lines, and replayed through Queued replay in run `08584087227067729212084836766CU19`. The same record completed local ingestion, reviewed graduation and PDF rehearsal; see the [demo packet](../manual-jsonl-ingestion-graduation/walkthrough.md). This dated success does not establish reliable connectivity.

- Code and saved Power Platform sources reviewed on October 24, 2026 against `49b4700d` plus the current uncommitted intake changes. Cloud statements below describe retained acceptance evidence, not a new live inspection. The [pilot runbook](pilot-runbook.md) and [notification walkthrough](notification-walkthrough.md) preserve the deployed-revision evidence.
- The ORNL-branded app, project catalog, result panel, and browser-local pending slot are published in **Wohlgemuth, Jason**, environment `221f44a6-1468-ea4e-b93d-cd52f1723079`. Jason Wohlgemuth is the operational owner. The button displays Load Project List before loading and Refresh Project List after a successful load.
- October 24 visual polish is published and verified: unclipped Title Case heading, `//` subsection separators, and padded submit footer with ACORN/ORNL and owner attribution. Desktop and 360-pixel phone previews passed visual inspection; formula and accessibility checks found no errors. Updated source and screenshots are in the runbook assets. This presentation-only change does not complete the remaining intake acceptance phases.
- Intake `ff4a134b-5fd3-4465-ab19-0325c2f2bf1b` and catalog `cd27888f-dd74-4675-9dd1-abc246ba56d5` are enabled with protected connections. Target: project `22464`, `main`, `.acorn/intake/logbook.jsonl`. ACRE `folders` is authoritative when present; otherwise discover `index.json`/`index.yaml` folders. Catalog membership does not establish ingestion authorization.
- Fresh append, immutable replay, caller/project rejection, interrupted-response reconciliation, reload and app-tab recovery passed. The latest retained append proof is `03ff5836289447034f40f2a4898c155441e6cb66`: seven TEST ONLY lines, all previous bytes preserved, and exact replay without another append. Earlier five- and six-line snapshots remain historical fixtures. Isolated server over-limit cases passed.
- Intermittent explicit network-range 403s still affect reads and writes through the unchanged connector. A successful read or isolated append does not prove reliable egress. Credential rotation and binding are verified; the [runbook](pilot-runbook.md) contains administrator correlation evidence.
- The duplicate failure response is removed. Its isolated persistence check had zero errors/warnings; the later Teams-enabled flow checker recorded zero errors and nine connection-change warnings, followed by app binding refresh, publication and successful submission; isolated cloud failed-commit acceptance now passes; see [failure evidence](write-failure-acceptance.md). The isolated native v6 app/flow/connector solution now exports and imports; app-driven intake and callback/table restore acceptance remain blocked. Exact-limit/malformed-tail/size and concurrent-write cases remain. Failed schema output may echo values, so the pilot remains non-sensitive only.
- [User/reviewer guide](_assets/sponsor-highlights-user-reviewer-guide.pdf) and [developer guide](_assets/sponsor-highlights-developer-guide.pdf) document the verified pilot, with screenshots. They are evidence snapshots, not proof that later integration is complete.

# Boundaries and decisions

October 24 acceptance constraint: perform PA-14/17 and PA-02/15/16 testing only in explicitly named test copies alongside the operational components in Wohlgemuth, Jason and ORNL Personal Productivity. Do not edit, publish, disable, rebind or import over operational app/flow/connector components, alter their credentials, or write their inbox. Use isolated test source paths. Browser restart and storage-denial tests must use an isolated browser session without closing operational tabs or changing their site storage settings. Retain any unmet acceptance instead of weakening this boundary.

Creation/update means creating a missing inbox or conditionally appending to an existing file while preserving existing bytes. It does not mean editing a previously accepted record under the same submission ID. Keep frozen content and UUID across retries, disabled automatic write retries, immutable commit proof, and an unresolved outcome when proof is unavailable. Do not bypass access denials.

The demo uses one browser tab, one contributor and sequential submissions in the existing pilot. Local browser recovery is approved for non-sensitive data only. A queued receipt means persisted in GitLab, not admitted, graduated, merged, or reported. The completed [manual demo](../manual-jsonl-ingestion-graduation/walkthrough.md) covers local ingestion, graduation and PDF rendering; it does not merge or publish canonical production changes. Local CLI commands do not automatically invoke the manual notification flow. The [Vergil plan](../vergil-gitlab-jsonl-automation/index.md) owns authenticated external callbacks and unattended processing.

# Dependencies and acceptance

**Demo gate:** The fresh personnel capture, exact replay and immutable snapshot handoff are preserved in the [demo packet](../manual-jsonl-ingestion-graduation/walkthrough.md). The repaired single unresolved response passed isolated cloud fault injection; [evidence](write-failure-acceptance.md) records the runtime and limits. A live demo also needs successful approved access at rehearsal. If denied, use the exact previously captured record in the manual demonstration and label the intake segment as recorded evidence; do not claim a fresh live save. PA-02 is closed under the owner-approved known-limitation decision; this does not guarantee that a live save will be available during the demo.

**Wider release gate:** PA-15 through PA-17 and PA-12 cover remaining acceptance, complete backup restoration, and shared deployment. None is silently declared complete by a successful single-user demo. PA-12 is a future shared release and does not gate the current pilot. Preserve original PA phase IDs; the completed demo capture is recorded in durable evidence; independent remaining matrices retain their bounded phase IDs.

# Validation

Inspect actual action outcomes, line deltas, receipt UUID/commit, immutable snapshots and replay results. Record exact flow revisions and run IDs in the runbook. No Rust changes or full workspace tests are needed for these operational phases. Keep TEST ONLY evidence out of sponsor reports. On each completion, prune its phase and reconcile this plan, indexes, network and lane.

# Execution phases

`waiting-on-validation` (“Waiting on validation”) means the implementation exists and the phase awaits acceptance evidence. Prerequisites and external blockers still apply; this status does not mean ready to execute or accepted. Phases that still need deployment, integration or policy setup retain their existing status.

## PA-15 — Accept bounded inbox and field limits

- Status: in-progress
- Validation readiness: Exact-summary, exact-detail and 20-detail appends passed in both environments. All listed field rejections passed. ORNL malformed-tail rejection also passed with unchanged bytes/revision. The requested five-attempt series is running; ORNL preservation and Jason tail exhausted five attempts with upstream 403/504 failures. File-size and Jason preservation tests remain in progress. See boundary-acceptance.md for retained evidence and the final case matrix. The actual append cap is 10,485,759 UTF-8 bytes. PA-02 remains closed with the connectivity limitation accepted.
- Budget: 5 active hours maximum
- Prerequisites: safe isolated fixtures and restored baseline; authorized connector access sufficient to execute each case
- Exact write set: pilot project `22464` isolated intake test files; validation actions in isolated intake flow copies in both test environments; `.wiki/features/powerapps-jsonl-intake/boundary-acceptance.md`; `.wiki/features/powerapps-jsonl-intake/_assets/boundary-tests/`
- Primary integration lock: intake route
- Outcome: Accept blank/rejected-line preservation, malformed tail and file-size boundaries in both environments. Preserve already accepted summary/detail/count and field-rejection evidence.
- Focused validation: Use one dimension per case; accepted records append once, rejected records do not alter any bytes, unsafe tails fail closed; preserve original pilot inbox.
- Handoff: Record per-case outcomes and restore isolated fixtures; no implied concurrency acceptance.

## PA-16 — Accept concurrent inbox creation and append

- Status: waiting-on-validation
- Validation readiness: Conditional writes and reconciliation are implemented; live concurrent-creation/append acceptance waits for PA-15 and usable authorized connectivity during the test. PA-02 closure does not accept concurrency behavior.
- Budget: 5 active hours maximum
- Prerequisites: PA-15; approved isolated source files and usable authorized connectivity during the test
- Exact write set: pilot project `22464` isolated concurrency test files; conditional-write actions in isolated intake flow copies in both test environments; `.wiki/features/powerapps-jsonl-intake/concurrency-acceptance.md`
- Primary integration lock: intake route
- Outcome: Prove simultaneous creators/writers do not overwrite lines or duplicate the same submission.
- Focused validation: Inspect competing revisions, conditional-write failure/reconciliation and immutable receipts; preserve one-write limit and unresolved results when history proof is bounded. No Vergil cleanup runs in this phase.
- Handoff: Preserve concurrency evidence; append-versus-bot-cleanup belongs to BOT-02.

## PA-17 — Produce a restorable Power Platform backup

- Status: waiting-on-validation
- Validation readiness: The complete native backup set exists. App-driven intake and full callback/table restoration still need acceptance; GitLab write denials and missing ORNL `prvCreateEntity` remain blockers.
- Budget: 5 active hours maximum
- Prerequisites: permitted GitLab writes for app-driven intake acceptance; ORNL capability to create the isolated callback table (`prvCreateEntity`), or an explicitly revised restore target. Native v6 app/connector/reference import and catalog acceptance are already verified.
- Exact write set: isolated ACORN PA17 test solutions and cloned components in Wohlgemuth, Jason and ORNL Personal Productivity; read-only exports of operational components; `.wiki/features/powerapps-jsonl-intake/_assets/solution-export/`; `.wiki/features/powerapps-jsonl-intake/restore-target-readiness.md`
- Primary integration lock: Power Platform packaging
- Outcome: Produce a complete native importable backup set with dependency order and restore instructions after the isolated connector/reference export repair.
- Focused validation: Inspect exported app, flows, connector and connection references; verify no secret values; perform import validation in an authorized disposable target when available, otherwise record the export as partial evidence and keep PA-17 open until restore verification is accepted.
- Handoff: Record export version, hashes and restore result. A source ZIP alone does not satisfy this phase.

## PA-12 — Move the accepted pilot to an approved shared environment

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: PA-15, PA-16 and PA-17 accepted; named approved GCC environment, maker/sharing rights, licensing, owner and recipient group/channel
- Exact write set: target Power Platform solution import, protected connections, app/flow sharing and notification destination; `.wiki/features/powerapps-jsonl-intake/shared-deployment.md`
- Primary integration lock: Power Platform deployment
- Outcome: Deploy the accepted app/flows and switch future notifications from Jason to the approved shared destination.
- Focused validation: Verify catalog, append/replay and notification deduplication under intended user permissions; preserve old receipt recipients and rollback; a new Teams channel has its own delivery state.
- Handoff: Preserve destination IDs and protected reference names; prune only after target acceptance. This is outside next week's pilot MVP.
