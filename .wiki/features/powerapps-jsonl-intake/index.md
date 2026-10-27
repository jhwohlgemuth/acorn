---
type: Plan
title: "PowerApp and Power Automate JSONL intake"
description: "Accept the deployed SharePoint list-to-JSONL manual workflow, repair exporter failure handling, and retain separate GitLab, restoration and deployment gates."
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

Preserve the separate enabled GitLab pilot and its unfinished route-specific acceptance. SharePoint is the focus of the standard-connector path, not an already accepted replacement or an authorization to disable GitLab. Wider access, reliability, native restoration and optional cutover/shared deployment remain explicit gates.

Supporting material: [pilot runbook](pilot-runbook.md), [notification walkthrough](notification-walkthrough.md), and [source bundle](source-bundle-readme.md). Form sources, PDFs and acceptance files are in `_assets/`.

# Current evidence

Reviewed October 26, 2026 against `58c22da4` plus the current working tree, retained October 26 cloud evidence, saved flow definitions and focused current-code checks. The later October 26 live pass accepted PA-19 storage/catalog and PA-20 intake/export using isolated SharePoint copies, and Jason's PA-15 unsafe-tail rejection. Test flows were returned to Off; both operational pilots remain enabled. No workspace suite was needed for these cloud checks; recheck actual revisions before deployment or further acceptance.

| Track | Verified state | Remaining acceptance |
| --- | --- | --- |
| SharePoint pilot | Separate published app, 13-project catalog, immutable receipt and per-submission JSONL export; owner/app persistence, reload and new-receipt Teams tests passed. Isolated PA-19 storage/catalog and PA-20 size/concurrency/export-recovery acceptance passed October 26. Five-minute combined `logbook.jsonl` exporter passed initial/repeat runs and exact flow readback. Three later live demo submissions and Teams actions passed; a six-row download preserved the earlier prefix exactly, and the three selected rows produced a local ACORN PDF. | PA-21 browser edge cases; PA-26 exporter failure-path repair, then PA-23 full stored-payload comparison and aggregate boundaries; PA-22 intended standard-user access. Owner-context manual CLI handoff passed. Owner premium access does not establish standard-user access. |
| GitLab pilot | Original app and intake/catalog remain enabled. Dated append, replay and Teams evidence is retained. PA-02 closed by owner decision with intermittent 403/504 connectivity accepted as a limitation. | PA-15 remaining byte boundaries, PA-16 concurrent writes, PA-17 app-driven native restore and ORNL callback/table restoration. No successful single run closes those gates. |
| Recovery and packaging | PA-14 browser recovery and PA-18 native reference repair are accepted. GitLab native v6 app/catalog restore passed; native backups remain intact. | ORNL callback restoration lacks `prvCreateEntity`; fresh app-driven GitLab write acceptance remains unproven. The separate SharePoint route needs its own native backup/restore, PA-24. |
| Deployment | Both routes remain separate and enabled in Wohlgemuth, Jason. Jason is the operational owner and notification recipient. | PA-25 owns any later route cutover; PA-12 owns shared-environment deployment. Neither is implied by pilot publication. Vergil remains a separate plan. |

The original DH access investigation is resolved by the owner's October 26 confirmation: he lacked a premium entitlement required by the GitLab custom connector. No DH-specific check remains. See the [licensing resolution](intake-access-decisions.md#original-participant-licensing-issue-resolved). PA-22 concerns the separate SharePoint route and can use any approved standard-license participant; it does not require repairing or retesting access to the original premium app.

Durable evidence: [SharePoint pilot](sharepoint-pilot.md), [GitLab boundaries](boundary-acceptance.md), [access decision](intake-access-decisions.md#known-limitation-and-pa-02-closure), [native restore handoff](_assets/solution-export/PA17-RESTORE-HANDOFF.md), [browser recovery](browser-recovery-acceptance.md), [write failure](write-failure-acceptance.md), [notification routing](notification-walkthrough.md), and [manual demo](../manual-jsonl-ingestion-graduation/walkthrough.md). These records preserve dated runs and limitations without keeping completed phases here.

The October 26 [cleanup record](test-artifact-cleanup.md#october-26-portfolio-and-inventory-review) records removed prototypes/drafts and retained dependencies. PA-15/17 test copies remain needed; their write-capable flows are Off. Current PDFs cover both branches and remain available in `_assets/`. The combined SharePoint file is a collected logbook; individual receipt files remain separate evidence. Direct SharePoint URL ingestion is not implemented: download the file before local CLI ingestion. The separate [SharePoint CLI download plan](../sharepoint-jsonl-download/index.md) owns reusable OAuth 2.0/Entra authentication with macOS/Windows/Linux credential caching and Graph file capture, with SharePoint URI intake and verified local snapshots as the first consumer. It adds ACRE `auth.profiles`/`auth.sources`, first-declared-profile defaults, `--device` and protected OS caching. It does not replace PA-22 access/licensing or PA-23/26 exporter acceptance, and requires no operational app/flow changes.

# Boundaries and decisions

The October 26 authorization produced a separate standard-connector SharePoint pilot alongside GitLab. Preserve its replaced intake/catalog bindings, separate browser recovery key and SharePoint receipt semantics. An inactive premium branch within an app is not sufficient evidence of standard licensing. Preserve the GitLab version for rollback/future premium use; do not disable the operational route before replacement acceptance. Never automatically send an unresolved GitLab receipt to the SharePoint route. The private site's current membership does not imply access for every intended contributor.

October 24 acceptance constraint: perform PA-14/17 and PA-02/15/16 testing only in explicitly named test copies alongside the operational components in Wohlgemuth, Jason and ORNL Personal Productivity. Do not edit, publish, disable, rebind or import over operational app/flow/connector components, alter their credentials, or write their inbox. Use isolated test source paths. Browser restart and storage-denial tests must use an isolated browser session without closing operational tabs or changing their site storage settings. Retain any unmet acceptance instead of weakening this boundary.

The two storage contracts differ:

- GitLab conditionally creates/appends an inbox while preserving previous bytes and proving the immutable commit. Retry uses the same UUID/payload; unavailable proof remains unresolved.
- SharePoint stores one immutable list receipt per UUID and one individual JSONL receipt file. The five-minute exporter rebuilds a collected snapshot through a captured item-ID cutoff. It does not append to or drain the list. The app's JSONL-ready panel confirms the individual receipt file, not inclusion in the latest combined snapshot. New arrivals after the cutoff wait for a later run.

The collected file is a retained history, not a work queue. Applying a downloaded batch does not delete list rows, remove exported lines, mark remote receipts processed or acknowledge anything to the producer. Repeated downloads must preserve UUIDs and exact bytes so existing local replay/conflict checks can work; use the same local intake state/destinations. A fresh local workspace is not evidence that old submissions were already applied elsewhere. Do not delete older list rows to evade the export cap. The current 5,000-row/10 MiB bounds remain operational limits until a separately approved retention/scale design exists. Operators check combined-file freshness and exporter run history before downloading; no automated combined-export failure alert is currently evidenced, and a fresh app confirmation alone cannot establish a fresh collected file.

Saved exporter source allows `Read_previous_logbook_LB` failure, timeout or skip to reach an overwrite; only a confirmed missing file should authorize first creation. Post-write readback failure also cannot prove the prior file survived. PA-26 owns the isolated repair. PA-23 must distinguish validation/read failures before publication (prior file unchanged) from ambiguous publication/readback outcomes (inspect and reconcile; no verified-success claim). Current source/clipboard evidence is not a fresh deployment inspection.

SharePoint faults, maximum-size fixtures, competing writers, browser-storage tests and restore tests use explicitly named TEST ONLY copies and storage. Remaining PA-21/23/26 tests may read the published pilot for baseline evidence but must not inject malformed rows, change bindings, delete receipts, replace its collected file or disrupt normal runs. Before any test, verify every connector destination and disable unrelated notifications. Stop on a destination mismatch. PA19 TEST ONLY receipt/project lists, JSONL library and catalog flow, plus the PA20 TEST ONLY intake flow, were created and verified October 26. Both test flows are Off. Other planned test names describe copies still to create; see the dated SharePoint evidence for exact IDs.

Normal intake sends best-effort Teams save notices under each route's recorded gate. Local CLI commands send no messages; later Teams/email milestones use the separate owner-triggered receiver. Its Dataverse/external-callback requirements are not dependencies of the standard SharePoint app. The [Vergil plan](../vergil-gitlab-jsonl-automation/index.md) owns future authenticated unattended callbacks; keep that integration outside app bindings and PA-22's standard-user proof.

Local browser recovery remains approved for non-sensitive data only. Neither received/exported SharePoint feedback nor a GitLab queued receipt means admitted, graduated, merged or published. The [manual walkthrough](../manual-jsonl-ingestion-graduation/walkthrough.md#sharepoint-download-to-a-combined-local-pdf) now documents the defaults-based four-step flow and retains the separate GitLab graduation rehearsal as dated evidence.

# Dependencies and acceptance

**Current manual MVP:** The SharePoint three-row owner demonstration already proves downloaded exact rows -> stage/inspect/apply -> combined local PDF; the earlier GitLab personnel rehearsal additionally proves supported graduation. Neither proves intended standard-user access or full reliability. For a new live demo, recheck approved access and combined-export freshness. If unavailable, use the retained exact local snapshot and label the capture as recorded evidence. Do not repeat a completed demo phase or reopen the retired manual-ingestion plan.

**Next work:** PA-21 can use the accepted isolated storage/intake after creating and verifying a separate native app copy; its browser restart and storage-denial actions need the owner's isolated browser participation. PA-22 intended-participant access remains the highest-value external check and can proceed independently. PA-26 prepares the isolated exporter repair, then PA-23 closes aggregate-export acceptance. PA-24 captures/restores the accepted SharePoint revision. Direct CLI download (SPD) is an optional operator convenience; the working browser download remains supported.

**Wider release gate:** SharePoint acceptance requires remaining PA-21–23 plus PA-26's repaired exporter revision; native restore is PA-24. PA-25 owns any explicit default-route cutover and installation of the accepted revisions, with rollback. GitLab-only PA-15–17 are independent and do not block the standard SharePoint route. PA-12 requires the acceptance/licensing of whichever route is chosen and an approved shared target. Callback integration, bot deployment and sponsor publication have separate owners. Preserve unfinished phase IDs and never infer cutover permission from a planning update.

# Validation

Inspect route-specific evidence: SharePoint list UUID/payload, individual receipt file, combined cutoff/count/digest and local batch; GitLab line deltas and immutable commit proof. Verify replay and conflict behavior. Record exact flow revisions and run IDs in the runbook. No Rust changes or full workspace tests are needed for these operational phases. Keep TEST ONLY evidence out of sponsor reports. On each completion, prune its phase and reconcile this plan, indexes, network and lane.

# Execution phases

`waiting-on-validation` (“Waiting on validation”) means the implementation exists and the phase awaits acceptance evidence. Prerequisites and external blockers still apply; this status does not mean ready to execute or accepted. Phases that still need deployment, integration or policy setup retain their existing status.

## PA-21 — Accept the separate standard-only canvas app

- Status: waiting-on-validation
- Validation readiness: Separate app published with only the SharePoint flow bindings, distinct recovery key and received/exported feedback. Project loading, fresh submit, confirmed receipt reload and new-highlight reset passed. App-driven replay/conflict and pending-export recovery remain.
- Budget: 5 active hours maximum
- Prerequisites: isolated storage/intake accepted October 26; verified native copy with its own app ID
- Exact write set: `PA21 TEST ONLY Sponsor Highlights SharePoint` app/solution copy in Wohlgemuth, Jason, using PA19/20 test endpoints and a test-only browser recovery key; published app is a read-only baseline; `.wiki/features/powerapps-jsonl-intake/sharepoint-pilot.md`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/`
- Primary integration lock: SharePoint pilot canvas
- Outcome: Catalog and submission use only standard flows; recovery uses a separate local-storage namespace and honest SharePoint receipt panels.
- Focused validation: Exercise app-driven replay/conflict and pending-export recovery, browser restart and denied storage against this copy; preserve original pending GitLab receipts and the published fresh-submit/reload evidence.
- Handoff: Record source/published pilot revision, test app ID and app-driven acceptance against equivalent logic; carry any required release revision to PA-25 without replacing the operational link.

## PA-26 — Repair collected-export failure handling in isolation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: saved exporter definition and known-good snapshot fixture; authorized isolated SharePoint test storage/connection; compare deployed source read-only before attributing the saved-source defect to a live revision
- Exact write set: `_assets/sharepoint/build-logbook-export.py`, `_assets/sharepoint/Combined_logbook_export_LB.clipboard.json`, `_assets/sharepoint/export-failure-tests/` (new); `PA26 TEST ONLY SharePoint Logbook Export` and isolated test list/library; `sharepoint-pilot.md`
- Primary integration lock: SharePoint pilot logbook export
- Outcome: Only a confirmed missing-file response permits first creation; prior-file read denial, timeout and other errors stop before overwrite. A write timeout or failed/mismatched readback is an unresolved outcome, never proof that the old file was retained. Reconcile by reading and comparing exact expected bytes before retry; a later serialized rebuild uses the immutable list receipts without deleting them or duplicating rows.
- Focused validation: Isolated confirmed-404 first creation, 403/5xx/read-timeout no-write, lost write response with both committed/uncommitted outcomes, and readback failure/mismatch. Compare byte hashes and write counts. Do not blindly roll back over a later valid snapshot or claim verified success from action status alone. Fixtures and run evidence contain no credentials.
- Handoff: Reviewed repair artifact, source/revision hashes and isolated proof for PA-23; PA-24 backs up the accepted revision and PA-25 installs it only as part of an approved release/cutover. No operational mutation during this phase.

## PA-23 — Accept the collected SharePoint logbook export

- Status: waiting-on-validation
- Budget: 5 active hours maximum
- Prerequisites: PA-26 repaired isolated exporter; accepted isolated test storage and existing authorized connection; live pilot revision recorded read-only
- Exact write set: `PA26 TEST ONLY SharePoint Logbook Export` and its isolated lists/library/acceptance fixtures; `_assets/sharepoint/`; `sharepoint-pilot.md`; published logbook is read-only evidence
- Primary integration lock: SharePoint pilot logbook export
- Outcome: Accept independent downloaded bytes, concurrent arrivals, empty list, malformed rows, pagination overflow and 10 MiB boundary behavior without disturbing operational intake.
- Evidence: Serialized initial/repeat runs and exact flow readback passed. A later six-row download preserved the earlier three-row prefix byte-for-byte; three selected rows passed local ACORN ingestion and reporting. See [live demonstration](sharepoint-pilot.md#october-26-live-demonstration-and-local-report). Full stored-payload comparison and aggregate edge cases remain.
- Focused validation: Independently compare every source line and Unicode byte through the cutoff; inject an isolated later arrival and confirm next-run inclusion. Test zero rows, 5,000/5,001 rows and exact/over 10 MiB with permitted record sizes. Before-publish failures preserve the prior digest; write timeout/readback mismatch remain unresolved until reconciliation. Repeat full-snapshot local staging against the same intake state to verify replays and conflicts without changing remote rows. Manual list edits/deletion are outside transactional-snapshot guarantees.
- Handoff: Preserve evidence and prune accepted cases; do not treat successful small runs as overflow or concurrency acceptance.

## PA-22 — Accept standard-user access and manual CLI handoff

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: published separate SharePoint app; intended standard-license participant and approved pilot access. Owner-context downloaded-file-to-CLI handoff is recorded; SharePoint participant-context access remains. Any approved standard-license participant may validate it; DH’s original GitLab premium-license denial is resolved and is not a prerequisite.
- Exact write set: sharing/run-only settings of the named SharePoint pilot app, flows and storage; `.wiki/features/powerapps-jsonl-intake/sharepoint-pilot.md`; pilot runbook and user/developer guides under this plan
- Primary integration lock: Power Platform deployment
- Outcome: An intended standard-license participant launches the separate SharePoint app, loads projects and submits without a premium prompt; an authorized operator downloads the next combined snapshot and completes stage -> inspect -> confirmed apply -> combined local PDF in an isolated destination. Contributors need only their approved role; do not assume every contributor is also a CLI operator or list administrator.
- Focused validation: Inspect both catalog/intake app bindings and invoked flows for premium dependencies, then use the participant's actual entitlement and approved permissions. Verify save/Teams action evidence, individual receipt, later combined inclusion and the operator's local batch/PDF; app JSONL-ready alone is insufficient. Preserve the accepted owner-context three-row CLI/PDF proof rather than repeating it as a new claim of standard-user access. Owner premium access or a Standard badge alone is insufficient. Access testing can proceed before other edge-case matrices finish.
- Handoff: Record participant-context acceptance and local batch proof; do not switch the operational link or disable GitLab.

## PA-24 — Produce and restore the native SharePoint backup

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: intended SharePoint release revisions frozen, including PA-26 exporter repair and PA-23 acceptance; authorized isolated import target and test storage
- Exact write set: read-only exports of `Sponsor Highlights - SharePoint Pilot` and its catalog/intake flows/references, plus the accepted PA-26/23 isolated exporter release revision; isolated SharePoint restore copies; distinguish the current deployed export from the repaired release candidate; `_assets/sharepoint/backup/`; `sharepoint-pilot.md`
- Primary integration lock: Power Platform packaging
- Outcome: A native importable app/flow backup and documented list/library schemas restore as isolated components with separate destinations and recovery storage.
- Focused validation: Inspect dependencies, archive hashes and absence of secrets; import without overwriting operational components, bind test storage, and prove catalog, receipt export and collected-logbook readback. Restore representative synthetic receipt data, unique-ID rules and both individual/combined exports; validate list/library schemas, permissions, versioning and connection bindings. State explicitly that solution exports do not back up SharePoint rows or credentials; preserve separate data-recovery instructions and recorded cutoff/digests.
- Handoff: Retain native files, schema/data recovery instructions and restore proof; source snippets and GitLab PA-17 archives do not satisfy this phase.

## PA-25 — Accept a SharePoint pilot cutover and rollback

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: remaining PA-21–24 and PA-26 accepted; owner selection of the replacement link and intended audience
- Exact write set: approved accepted SharePoint app/flow revision installation and pilot entry links, selected GitLab flow enablement only if explicitly authorized for cutover, SharePoint/GitLab runbooks and user guides
- Primary integration lock: Power Platform deployment
- Outcome: The chosen default route and rollback procedure are explicit, with no dual submission or loss of unresolved receipts.
- Focused validation: Prove the published entry link, audience access and receipt routing; reconcile pending receipts by their original route. Preserve GitLab credentials/components for rollback and future premium use. Do not infer permission to disable it from this plan.
- Handoff: Record the actual route decision and rollback evidence; shared-environment migration remains PA-12.

## PA-15 — Accept bounded inbox and field limits

- Status: waiting-on-validation
- Validation readiness: Field limits and unsafe-tail rejection passed in both environments; ORNL oversized-file rejection also passed. Mixed-line preservation and exact-cap append remain in both environments, plus oversized-file rejection in Jason. Five October 26 Jason mixed-line attempts did not append; the final attempt and the separate oversized-file attempt stopped at Forbidden branch access. Remaining cases require runnable connector calls and independent byte/revision verification. See [boundary acceptance](boundary-acceptance.md) for dated attempts and proofs. The append cap is 10,485,759 UTF-8 bytes. PA-02 remains closed with the connectivity limitation accepted.
- Budget: 5 active hours maximum
- Prerequisites: safe isolated fixtures and restored baseline; authorized connector access sufficient to execute each case
- Exact write set: pilot project `22464` isolated intake test files; validation actions in isolated intake flow copies in both test environments; `.wiki/features/powerapps-jsonl-intake/boundary-acceptance.md`; `.wiki/features/powerapps-jsonl-intake/_assets/boundary-tests/`
- Primary integration lock: intake route
- Outcome: Accept mixed-line preservation and exact-cap append in both environments, plus one-byte-over-cap rejection in Jason. Jason unsafe-tail rejection passed on October 26. Preserve accepted field and ORNL rejection evidence.
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
- Prerequisites: selected route accepted (SharePoint remaining PA-21–24 plus PA-26, or GitLab PA-15–17); named approved GCC environment, maker/sharing rights, route-appropriate licensing, owner and recipient group/channel
- Exact write set: target Power Platform solution import, protected connections, app/flow sharing and notification destination; `.wiki/features/powerapps-jsonl-intake/shared-deployment.md`
- Primary integration lock: Power Platform deployment
- Outcome: Deploy the accepted app/flows and switch future notifications from Jason to the approved shared destination.
- Focused validation: Verify catalog and route-specific persistence/replay under intended user permissions: SharePoint receipt plus collected export, or GitLab append/commit; verify notification deduplication; preserve old receipt recipients and rollback; a new Teams channel has its own delivery state.
- Handoff: Preserve destination IDs and protected reference names; prune only after target acceptance. This is outside the current pilot MVP.
