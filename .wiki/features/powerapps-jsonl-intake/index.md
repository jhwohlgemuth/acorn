---
type: Plan
title: "PowerApp and Power Automate JSONL intake"
description: "Finish SharePoint catalog error feedback and approved deployment using the accepted exporter and native restore; retain separate GitLab and deployment gates."
difficulty: medium
impact: critical
package: [workspace]
tags: [powerapps, powerautomate, gitlab, sharepoint, jsonl, intake]
status: in-progress
priority: next
execution: phased
phase_budget_hours: 5
---

# Goal

Finish the deployed SharePoint route for intended contributors and operators: form -> immutable list receipt -> collected JSONL snapshot -> local `stage` -> `inspect` -> confirmed `apply` -> combined PDF. The owner-context three-row demonstration is accepted evidence for this manual workflow. Graduate metadata only when needed; Vergil, direct OAuth download and external callbacks are not prerequisites for local reporting.

Preserve the separate enabled GitLab pilot and its unfinished route-specific acceptance. SharePoint is the focus of the standard-connector path, not an already accepted replacement or an authorization to disable GitLab. Wider access and optional cutover/shared deployment remain explicit gates; isolated collected-export and native-restoration acceptance are recorded below.

Supporting material: [pilot runbook](pilot-runbook.md), [notification walkthrough](notification-walkthrough.md), and [source bundle](source-bundle-readme.md). Form sources, PDFs and acceptance files are in `_assets/`.

# Current evidence

Reviewed October 27, 2026 against `5c9acff4` plus the current working tree, retained October 26–27 cloud evidence, saved flow definitions and focused current-code checks. The later October 26 live pass accepted PA-19 storage/catalog and PA-20 intake/export using isolated SharePoint copies, and Jason's PA-15 unsafe-tail rejection. A subsequent isolated pass accepted Jason's mixed-line append and PA-21 app-driven export recovery/replay and conflict handling. The owner then confirmed the isolated full-restart and denied-storage procedure complete, accepting PA-21. PA15 and PA20 test intake flows and the PA19 test catalog are Off. PA-26 isolated exporter repair then passed local and native fault acceptance, with its temporary harness removed and test flow Off. October 27 PA-23 aggregate acceptance and PA-27 empty-response repair also passed, including actual 5,000/5,001-row pagination, exact/over 10 MiB, independent downloads and local replay/conflict checks. The clean repaired test exporter passed its final smoke run and remains Off; operational revisions remain unchanged. PA-24 native backup and isolated restore also passed October 27: six receipts and 13 projects recovered, followed by an app-driven seventh receipt and exact collected-file readback. All three PA24 flows are Off. Both operational pilots remain enabled. No workspace suite was needed for these cloud checks; recheck actual revisions before deployment or further acceptance.

| Track | Verified state | Remaining acceptance |
| --- | --- | --- |
| SharePoint pilot | Separate published app, 13-project catalog, immutable receipt and per-submission JSONL export; owner/app persistence, reload and new-receipt Teams tests passed. Isolated PA-19 storage/catalog and PA-20 size/concurrency/export-recovery acceptance passed October 26. Five-minute combined `logbook.jsonl` exporter passed initial/repeat runs and exact flow readback. Three later live demo submissions and Teams actions passed; a six-row download preserved the earlier prefix exactly, and the three selected rows produced a local ACORN PDF. | PA-22 intended standard-user access passed October 27 after owner-backed connection settings: c5c receipt, Teams send and exact collected inclusion verified; PA-28 isolated catalog-error feedback is accepted; PA-25 retains installation. Native SharePoint restoration passed October 27. Collected export acceptance passed October 27; Owner-context manual CLI handoff passed. Owner premium access does not establish standard-user access. |
| GitLab pilot | Original app and intake/catalog remain enabled. Dated append, replay and Teams evidence is retained. PA-02 closed by owner decision with intermittent 403/504 connectivity accepted as a limitation. | PA-15 remaining byte boundaries, PA-16 concurrent writes, PA-17 app-driven native restore and ORNL callback/table restoration. No successful single run closes those gates. |
| Recovery and packaging | PA-14 browser recovery and PA-18 native reference repair are accepted. GitLab native v6 app/catalog restore passed; native backups remain intact. | ORNL callback restoration lacks `prvCreateEntity`; fresh app-driven GitLab write acceptance remains unproven. The separate SharePoint native backup/restore passed October 27; see the retained recovery package and instructions. |
| Deployment | Both routes remain separate and enabled in Wohlgemuth, Jason. Jason is the operational owner and notification recipient. | PA-25 owns any later route cutover; PA-12 owns shared-environment deployment. Neither is implied by pilot publication. Vergil remains a separate plan. |

The original DH access investigation is resolved by the owner's October 26 confirmation: he lacked a premium entitlement required by the GitLab custom connector. No DH-specific check remains. See the [licensing resolution](intake-access-decisions.md#original-participant-licensing-issue-resolved). PA-22 acceptance covers the separate SharePoint route with the approved standard-license participant; it does not establish access to the original premium app.

Durable evidence: [SharePoint native backup and restore](_assets/sharepoint/backup/README.md), [SharePoint pilot](sharepoint-pilot.md), [GitLab boundaries](boundary-acceptance.md), [access decision](intake-access-decisions.md#known-limitation-and-pa-02-closure), [native restore handoff](_assets/solution-export/PA17-RESTORE-HANDOFF.md), [browser recovery](browser-recovery-acceptance.md), [write failure](write-failure-acceptance.md), [notification routing](notification-walkthrough.md), and [manual demo](../manual-jsonl-ingestion-graduation/walkthrough.md). These records preserve dated runs and limitations without keeping completed phases here.

The October 26 [cleanup record](test-artifact-cleanup.md#october-26-portfolio-and-inventory-review) records removed prototypes/drafts and retained dependencies. PA-15/17 test copies remain needed; their GitLab write-capable test flows are Off. Current PDFs cover both branches and remain available in `_assets/`. The combined SharePoint file is a collected logbook; individual receipt files remain separate evidence. Direct SharePoint URL ingestion is not implemented: download the file before local CLI ingestion. The separate [SharePoint CLI download plan](../sharepoint-jsonl-download/index.md) owns reusable OAuth 2.0/Entra authentication with macOS/Windows/Linux credential caching and Graph file capture, with SharePoint URI intake and verified local snapshots as the first consumer. It adds ACRE `auth.profiles`/`auth.sources`, first-declared-profile defaults, `--device` and protected OS caching. It does not replace the recorded participant access/licensing evidence or the accepted producer completeness checks, and requires no operational app/flow changes.

# Boundaries and decisions

October 27 PA-25 authorization: the owner explicitly requested installation of the accepted SharePoint revisions and disabling the GitLab connector route with low-effort restoration. Use the existing published SharePoint app and the already approved o9w/c5c audience. Disable GitLab intake/catalog flows after replacement verification; preserve app, custom connectors, credentials, connections, data and native backups. This authorizes operational deployment for PA-25 and supersedes earlier no-cutover assumptions for this phase only. Do not reroute pending GitLab receipts.

The October 26 authorization produced a separate standard-connector SharePoint pilot alongside GitLab. Preserve its replaced intake/catalog bindings, separate browser recovery key and SharePoint receipt semantics. An inactive premium branch within an app is not sufficient evidence of standard licensing. Preserve the GitLab version for rollback/future premium use; do not disable the operational route before replacement acceptance. Never automatically send an unresolved GitLab receipt to the SharePoint route. The private site's current membership does not imply access for every intended contributor.

October 24 acceptance constraint: perform PA-14/17 and PA-02/15/16 testing only in explicitly named test copies alongside the operational components in Wohlgemuth, Jason and ORNL Personal Productivity. Do not edit, publish, disable, rebind or import over operational app/flow/connector components, alter their credentials, or write their inbox. Use isolated test source paths. Browser restart and storage-denial tests must use an isolated browser session without closing operational tabs or changing their site storage settings. Retain any unmet acceptance instead of weakening this boundary.

The two storage contracts differ:

- GitLab conditionally creates/appends an inbox while preserving previous bytes and proving the immutable commit. Retry uses the same UUID/payload; unavailable proof remains unresolved.
- SharePoint stores one immutable list receipt per UUID and one individual JSONL receipt file. The five-minute exporter rebuilds a collected snapshot through a captured item-ID cutoff. It does not append to or drain the list. The app's JSONL-ready panel confirms the individual receipt file, not inclusion in the latest combined snapshot. New arrivals after the cutoff wait for a later run.

The collected file is a retained history, not a work queue. Applying a downloaded batch does not delete list rows, remove exported lines, mark remote receipts processed or acknowledge anything to the producer. Repeated downloads must preserve UUIDs and exact bytes so existing local replay/conflict checks can work; use the same local intake state/destinations. A fresh local workspace is not evidence that old submissions were already applied elsewhere. Do not delete older list rows to evade the export cap. The current 5,000-row/10 MiB bounds remain operational limits until a separately approved retention/scale design exists. Operators check combined-file freshness and exporter run history before downloading; no automated combined-export failure alert is currently evidenced, and a fresh app confirmation alone cannot establish a fresh collected file.

The October 26 read-only deployment inspection confirmed the prior-read failure path in the operational exporter. [PA-26 isolated repair is accepted](sharepoint-pilot.md#october-26-isolated-exporter-repair-acceptance-pa-26): only a readable prior file or explicit failed 404 admits publishing; uncertain writes require exact readback before a success receipt. The repaired source and Off test copy are separate from the unchanged operational revision. [PA-23/27 aggregate and empty-file acceptance passed October 27](sharepoint-pilot.md#october-27-collected-export-acceptance). The [PA-24 native backup and isolated restore](_assets/sharepoint/backup/README.md) is accepted; PA-25 owns operational installation. A write attempt cannot establish that the prior file survived, and an unresolved outcome must never trigger blind rollback.

SharePoint faults, maximum-size fixtures, competing writers, browser-storage tests and restore tests use explicitly named TEST ONLY copies and storage. Future isolated tests may read the published pilot for baseline evidence but must not inject malformed rows, change bindings, delete receipts, replace its collected file or disrupt normal runs. Before any test, verify every connector destination and disable unrelated notifications. Stop on a destination mismatch. PA19 TEST ONLY receipt/project lists, JSONL library and catalog flow, plus the PA20 TEST ONLY intake flow, were created and verified October 26. PA20 test intake and PA19 test catalog are Off after acceptance. The catalog remains filtered to one synthetic project with its seeding condition disabled. The native PA21 test app now exists and is published; see the dated SharePoint evidence for its exact ID. PA26 TEST ONLY SharePoint Logbook Export (`28cba548-c862-484c-b838-ea21eaf52630`) is retained Off with the accepted repair and no fault harness. PA24 app/catalog/intake/exporter copies are retained as restore evidence; all three PA24 flows are Off, and their one-time data-restoration helper was removed. [Inventory and restore proof](_assets/sharepoint/backup/README.md) record exact IDs. Other planned test names describe copies still to create.

Normal intake sends best-effort Teams save notices under each route's recorded gate. The October 27 SharePoint recipient repair adds a submitter chat confirmation while retaining Jason's operator notice; its notification-only send test passed. All current milestone routes already include Jason; [routing evidence](notification-walkthrough.md#october-27-participant-delivery-repair) distinguishes this from the earlier owner-only PA-22 notice. Local CLI commands send no messages; later Teams/email milestones use the separate owner-triggered receiver. Its Dataverse/external-callback requirements are not dependencies of the standard SharePoint app. The [Vergil plan](../vergil-gitlab-jsonl-automation/index.md) owns future authenticated unattended callbacks; keep that integration outside app bindings and the accepted standard-user proof.

Local browser recovery remains approved for non-sensitive data only. Neither received/exported SharePoint feedback nor a GitLab queued receipt means admitted, graduated, merged or published. The [manual walkthrough](../manual-jsonl-ingestion-graduation/walkthrough.md#sharepoint-download-to-a-combined-local-pdf) now documents the defaults-based four-step flow and retains the separate GitLab graduation rehearsal as dated evidence.

# Dependencies and acceptance

**Current manual MVP:** The SharePoint three-row owner demonstration already proves downloaded exact rows -> stage/inspect/apply -> combined local PDF; the earlier GitLab personnel rehearsal additionally proves supported graduation. Neither proves intended standard-user access or full reliability. For a new live demo, recheck approved access and combined-export freshness. If unavailable, use the retained exact local snapshot and label the capture as recorded evidence. Do not repeat a completed demo phase or reopen the retired manual-ingestion plan.

**Next work:** PA-28 isolated catalog-error feedback is accepted with [native canvas backup, fault-injection evidence and installation instructions](_assets/sharepoint/catalog-recovery/README.md). PA-25 owns installation of that revision and the accepted exporter after the owner selects the route and audience. PA-22 participant acceptance is complete: the c5c receipt, successful Teams action and exact seven-row collected download are verified in the [acceptance record](sharepoint-pilot.md#october-27-standard-participant-acceptance). Owner-context CLI/PDF handoff and PA-21/23/24/26/27 acceptance remain valid. PA-25 retains the explicit owner route decision and installation/rollback work. Group selection and environment capacity belong to the deferred larger rollout. Direct CLI download (SPD) is optional; browser download remains supported.

**Wider release gate:** SharePoint participant acceptance is recorded; collected-export acceptance is complete using PA-26/27 repairs. Native SharePoint restore is accepted, with [packages and recovery instructions](_assets/sharepoint/backup/README.md). PA-25 owns any explicit default-route cutover and installation of the accepted revisions, with rollback. GitLab-only PA-15–17 are independent and do not block the standard SharePoint route. PA-12 requires the acceptance/licensing of whichever route is chosen and an approved shared target. Callback integration, bot deployment and sponsor publication have separate owners. Preserve unfinished phase IDs and never infer cutover permission from a planning update.

# Validation

Inspect route-specific evidence: SharePoint list UUID/payload, individual receipt file, combined cutoff/count/digest and local batch; GitLab line deltas and immutable commit proof. Verify replay and conflict behavior. Record exact flow revisions and run IDs in the runbook. No Rust changes or full workspace tests are needed for these operational phases. Keep TEST ONLY evidence out of sponsor reports. On each completion, prune its phase and reconcile this plan, indexes, network and lane.

# Execution phases

`waiting-on-validation` (“Waiting on validation”) means the implementation exists and the phase awaits acceptance evidence. Prerequisites and external blockers still apply; this status does not mean ready to execute or accepted. Phases that still need deployment, integration or policy setup retain their existing status.

## PA-25 — Accept a SharePoint pilot cutover and rollback

- Status: in-progress
- Budget: 5 active hours maximum
- Prerequisites: recorded [participant acceptance](sharepoint-pilot.md#october-27-standard-participant-acceptance); use the accepted repaired exporter, [accepted catalog-feedback overlay](_assets/sharepoint/catalog-recovery/README.md) and [native recovery handoff](_assets/sharepoint/backup/README.md); owner selection of the replacement link and intended audience
- Exact write set: approved accepted SharePoint app/flow revision installation and pilot entry links, selected GitLab flow enablement only if explicitly authorized for cutover, SharePoint/GitLab runbooks and user guides
- Primary integration lock: Power Platform deployment
- Outcome: The chosen default route and rollback procedure are explicit, with no dual submission or loss of unresolved receipts.
- Focused validation: Prove the published entry link, audience access and receipt routing; reconcile pending receipts by their original route. Preserve GitLab credentials/components for rollback and future premium use. Do not infer permission to disable it from this plan.
- Handoff: Record the actual route decision and rollback evidence; shared-environment migration remains PA-12.

## PA-15 — Accept bounded inbox and field limits

- Status: waiting-on-validation
- Validation readiness: Field limits and unsafe-tail rejection passed in both environments; ORNL oversized-file rejection and Jason mixed-line preservation also passed. Remaining: ORNL mixed-line preservation, exact-cap append in both environments, and oversized-file rejection in Jason. The latest Jason exact-cap and oversized-file cases each exhausted five attempts without acceptance. Exact-cap attempts 2 and 4 reached the append but received network-range 403; other failures stopped at catalog/file reads. Independent snapshots retained fixture bytes and file revisions. Remaining cases require runnable connector calls and independent byte/revision verification. See [boundary acceptance](boundary-acceptance.md) for dated attempts and proofs. The append cap is 10,485,759 UTF-8 bytes. PA-02 remains closed with the connectivity limitation accepted.
- Budget: 5 active hours maximum
- Prerequisites: safe isolated fixtures and restored baseline; authorized connector access sufficient to execute each case
- Exact write set: pilot project `22464` isolated intake test files; validation actions in isolated intake flow copies in both test environments; `.wiki/features/powerapps-jsonl-intake/boundary-acceptance.md`; `.wiki/features/powerapps-jsonl-intake/_assets/boundary-tests/`
- Primary integration lock: intake route
- Outcome: Accept ORNL mixed-line preservation and exact-cap append in both environments, plus one-byte-over-cap rejection in Jason. Preserve accepted Jason mixed-line/unsafe-tail, field and ORNL rejection evidence.
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

## PA-17 — Accept the native GitLab Power Platform restore

- Status: waiting-on-validation
- Validation readiness: The complete native backup set exists. App-driven intake and full callback/table restoration still need acceptance; GitLab write denials and missing ORNL `prvCreateEntity` remain blockers.
- Budget: 5 active hours maximum
- Prerequisites: permitted GitLab writes for app-driven intake acceptance; ORNL capability to create the isolated callback table (`prvCreateEntity`), or an explicitly revised restore target. Native v6 app/connector/reference import and catalog acceptance are already verified.
- Exact write set: isolated ACORN PA17 test solutions and cloned components in Wohlgemuth, Jason and ORNL Personal Productivity; read-only exports of operational components; `.wiki/features/powerapps-jsonl-intake/_assets/solution-export/`; `.wiki/features/powerapps-jsonl-intake/restore-target-readiness.md`
- Primary integration lock: Power Platform packaging
- Outcome: Accept the remaining app-driven GitLab write and callback/table restore using the existing complete native backup; retain accepted import/catalog evidence rather than repeating backup creation.
- Focused validation: Inspect exported app, flows, connector and connection references; verify no secret values; perform import validation in an authorized disposable target when available, otherwise record the export as partial evidence and keep PA-17 open until restore verification is accepted.
- Handoff: Record export version, hashes and restore result. A source ZIP alone does not satisfy this phase.

## PA-12 — Move the accepted pilot to an approved shared environment

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: selected route accepted (SharePoint participant acceptance already recorded, plus approved installation of the accepted exporter revision, or GitLab PA-15–17); named approved GCC environment, maker/sharing rights and capacity for the intended audience, route-appropriate licensing, contributor security group, owner and recipient group/channel
- Exact write set: target Power Platform solution import, protected connections, app/flow sharing and notification destination; `.wiki/features/powerapps-jsonl-intake/shared-deployment.md`
- Primary integration lock: Power Platform deployment
- Outcome: Deploy the accepted app/flows and switch future notifications from Jason to the approved shared destination.
- Focused validation: Verify catalog and route-specific persistence/replay under intended user permissions: SharePoint receipt plus collected export, or GitLab append/commit; verify notification deduplication; preserve old receipt recipients and rollback; a new Teams channel has its own delivery state.
- Handoff: Preserve destination IDs and protected reference names; prune only after target acceptance. This is outside the current pilot MVP.
