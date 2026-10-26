---
type: Plan
title: "PowerApp and Power Automate JSONL intake"
description: "Add a separate standard-connector SharePoint intake and JSONL export while preserving the GitLab pilot and its remaining acceptance."
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

Finish reliability, browser/boundary/concurrency acceptance, restoration and later shared deployment for the published Sponsor Highlights form. The recorded single-user demo already covers project selection, non-sensitive submission and exactly one confirmed GitLab JSONL record. A fresh live demonstration in the week of October 26, 2026 still depends on approved access. Neither Vergil nor an external callback endpoint is required for form submission.

Supporting material: [pilot runbook](pilot-runbook.md), [notification walkthrough](notification-walkthrough.md), and [source bundle](source-bundle-readme.md). Form sources, PDFs and acceptance files are in `_assets/`.

# Current evidence

Reviewed October 26, 2026 against `ca774b01` plus the current working tree, retained acceptance records, and live flow/solution inventories in both environments. This review did not rerun the full workspace or external acceptance matrices.

| Track | Verified state | Remaining acceptance |
| --- | --- | --- |
| SharePoint pilot | Separate published app, 13-project catalog, immutable receipt and per-submission JSONL export; owner/app persistence, reload and new-receipt Teams tests passed. Five-minute combined `logbook.jsonl` exporter passed initial/repeat runs and exact flow readback. Three later live demo submissions and Teams actions passed; a six-row download preserved the earlier prefix exactly, and the three selected rows produced a local ACORN PDF. | PA-19–21 storage, intake and browser edge cases; PA-23 full stored-payload comparison and aggregate boundaries; PA-22 intended standard-user access. Owner-context manual CLI handoff passed. Owner premium access does not establish standard-user access. |
| GitLab pilot | Original app and intake/catalog remain enabled. Dated append, replay and Teams evidence is retained. PA-02 closed by owner decision with intermittent 403/504 connectivity accepted as a limitation. | PA-15 remaining byte boundaries, PA-16 concurrent writes, PA-17 app-driven native restore and ORNL callback/table restoration. No successful single run closes those gates. |
| Recovery and packaging | PA-14 browser recovery and PA-18 native reference repair are accepted. GitLab native v6 app/catalog restore passed; native backups remain intact. | ORNL callback restoration lacks `prvCreateEntity`; fresh app-driven GitLab write acceptance remains unproven. The separate SharePoint route needs its own native backup/restore, PA-24. |
| Deployment | Both routes remain separate and enabled in Wohlgemuth, Jason. Jason is the operational owner and notification recipient. | PA-25 owns any later route cutover; PA-12 owns shared-environment deployment. Neither is implied by pilot publication. Vergil remains a separate plan. |

Durable evidence: [SharePoint pilot](sharepoint-pilot.md), [GitLab boundaries](boundary-acceptance.md), [access decision](intake-access-decisions.md#known-limitation-and-pa-02-closure), [native restore handoff](_assets/solution-export/PA17-RESTORE-HANDOFF.md), [browser recovery](browser-recovery-acceptance.md), [write failure](write-failure-acceptance.md), [notification routing](notification-walkthrough.md), and [manual demo](../manual-jsonl-ingestion-graduation/walkthrough.md). These records preserve dated runs and limitations without keeping completed phases here.

The October 26 [cleanup record](test-artifact-cleanup.md#october-26-portfolio-and-inventory-review) records removed prototypes/drafts and retained dependencies. PA-15/17 test copies remain needed; their write-capable flows are Off. Current PDFs cover both branches and remain available in `_assets/`. The combined SharePoint file is a collected logbook; individual receipt files remain separate evidence. Direct SharePoint URL ingestion is not implemented: download the file before local CLI ingestion. The separate [SharePoint CLI download plan](../sharepoint-jsonl-download/index.md) owns reusable OAuth 2.0/Entra authentication with macOS/Windows/Linux credential caching and Graph file capture, with SharePoint URI intake and verified local snapshots as the first consumer. It does not replace PA-22 access/licensing or PA-23 exporter acceptance, and requires no operational app/flow changes.

# Boundaries and decisions

October 26 SharePoint authorization: build a separate standard-connector pilot alongside GitLab. An inactive premium branch is not sufficient evidence of standard licensing. Both intake and catalog bindings must be replaced in a separate app copy, with its own browser recovery key and SharePoint receipt semantics. Preserve the GitLab version for rollback/future premium use; do not disable the operational route before replacement acceptance. Never automatically send an unresolved GitLab receipt to the SharePoint route. The private site's current membership does not imply access for every intended contributor.

October 24 acceptance constraint: perform PA-14/17 and PA-02/15/16 testing only in explicitly named test copies alongside the operational components in Wohlgemuth, Jason and ORNL Personal Productivity. Do not edit, publish, disable, rebind or import over operational app/flow/connector components, alter their credentials, or write their inbox. Use isolated test source paths. Browser restart and storage-denial tests must use an isolated browser session without closing operational tabs or changing their site storage settings. Retain any unmet acceptance instead of weakening this boundary.

Creation/update means creating a missing inbox or conditionally appending to an existing file while preserving existing bytes. It does not mean editing a previously accepted record under the same submission ID. Keep frozen content and UUID across retries, disabled automatic write retries, immutable commit proof, and an unresolved outcome when proof is unavailable. Do not bypass access denials.

The demo uses one browser tab, one contributor and sequential submissions in the existing pilot. Local browser recovery is approved for non-sensitive data only. A queued receipt means persisted in GitLab, not admitted, graduated, merged, or reported. The completed [manual demo](../manual-jsonl-ingestion-graduation/walkthrough.md) covers local ingestion, graduation and PDF rendering; it does not merge or publish canonical production changes. Local CLI commands do not automatically invoke the manual notification flow. The [Vergil plan](../vergil-gitlab-jsonl-automation/index.md) owns authenticated external callbacks and unattended processing.

# Dependencies and acceptance

**Demo gate:** The fresh personnel capture, exact replay and immutable snapshot handoff are preserved in the [demo packet](../manual-jsonl-ingestion-graduation/walkthrough.md). The repaired single unresolved response passed isolated cloud fault injection; [evidence](write-failure-acceptance.md) records the runtime and limits. A live demo also needs successful approved access at rehearsal. If denied, use the exact previously captured record in the manual demonstration and label the intake segment as recorded evidence; do not claim a fresh live save. PA-02 is closed under the owner-approved known-limitation decision; this does not guarantee that a live save will be available during the demo.

**Wider release gate:** SharePoint acceptance is PA-19–23, native restore is PA-24, and any pilot cutover is PA-25. GitLab acceptance remains PA-15–17. PA-12 requires the acceptance and licensing of the selected route plus an approved shared target; GitLab-only gates do not block the separate standard route. None is completed by a single-user demo. Preserve original unfinished phase IDs.

# Validation

Inspect actual action outcomes, line deltas, receipt UUID/commit, immutable snapshots and replay results. Record exact flow revisions and run IDs in the runbook. No Rust changes or full workspace tests are needed for these operational phases. Keep TEST ONLY evidence out of sponsor reports. On each completion, prune its phase and reconcile this plan, indexes, network and lane.

# Execution phases

`waiting-on-validation` (“Waiting on validation”) means the implementation exists and the phase awaits acceptance evidence. Prerequisites and external blockers still apply; this status does not mean ready to execute or accepted. Phases that still need deployment, integration or policy setup retain their existing status.

## PA-19 — Accept standard SharePoint storage and project catalog

- Status: waiting-on-validation
- Validation readiness: Shared storage and the 13-project catalog are deployed and owner/app calls passed. Catalog overflow, permission inheritance review and maximum payload capacity remain to be accepted.
- Budget: 5 active hours maximum
- Prerequisites: existing Research Enablement site and authorized SharePoint connection, now observed
- Exact write set: the three named Sponsor Highlights Pilot lists/library at `https://ornl.sharepoint.com/sites/ResearchEnablement`; `ACORN SharePoint Pilot Catalog` in Wohlgemuth, Jason; `.wiki/features/powerapps-jsonl-intake/sharepoint-pilot.md`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/`
- Primary integration lock: SharePoint pilot storage
- Outcome: Shared receipt storage with unique IDs, exact-payload capacity, export library and a bounded standard catalog returning the recorded 13 projects.
- Focused validation: Inspect list membership inheritance and maximum payload capacity; fail closed on catalog overflow; retain the recorded 13-ID acceptance without claiming live GitLab synchronization.
- Handoff: Record URLs, IDs, connection reference and schema; no operational cutover.

## PA-20 — Accept standard intake and verified JSONL export

- Status: waiting-on-validation
- Validation readiness: New/replay/conflict/malformed persistence branches passed, and published-app exports plus a Teams action succeeded. Size limits, concurrent attempts and export-failure recovery remain unaccepted.
- Budget: 5 active hours maximum
- Prerequisites: PA-19; retain current envelope validation and authenticated caller checks
- Exact write set: `ACORN SharePoint Pilot Intake` (including JSONL export and Teams notice) in Wohlgemuth, Jason; Sponsor Highlights Intake Pilot and JSONL Pilot only; `.wiki/features/powerapps-jsonl-intake/sharepoint-pilot.md`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/`
- Primary integration lock: SharePoint pilot intake
- Outcome: One immutable receipt per UUID, exact replay, changed-payload conflict, and one verified UTF-8 JSONL file per accepted receipt. Notification follows confirmed storage; no dual submission to GitLab.
- Focused validation: Simultaneous same-ID attempts, size boundary, export failure and exact readback after recovery. Preserve the new/replay/conflict/malformed and Unicode evidence; no success on unresolved writes or truncated exports.
- Handoff: Preserve source and acceptance IDs; list receipt is received, export is exported, neither means graduated.

## PA-21 — Accept the separate standard-only canvas app

- Status: waiting-on-validation
- Validation readiness: Separate app published with only the SharePoint flow bindings, distinct recovery key and received/exported feedback. Project loading, fresh submit, confirmed receipt reload and new-highlight reset passed. App-driven replay/conflict and pending-export recovery remain.
- Budget: 5 active hours maximum
- Prerequisites: PA-19 and PA-20; verified native copy with its own app ID
- Exact write set: new `Sponsor Highlights - SharePoint Pilot` app and `ACORN SharePoint Pilot` solution containing only pilot components in Wohlgemuth, Jason; `.wiki/features/powerapps-jsonl-intake/sharepoint-pilot.md`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/`
- Primary integration lock: SharePoint pilot canvas
- Outcome: Catalog and submission use only standard flows; recovery uses a separate local-storage namespace and honest SharePoint receipt panels.
- Focused validation: Exercise app-driven replay/conflict and pending-export recovery, browser restart and denied storage against this copy; preserve original pending GitLab receipts and the published fresh-submit/reload evidence.
- Handoff: Record published app ID and app-driven acceptance; do not replace the operational link yet.

## PA-23 — Accept the collected SharePoint logbook export

- Status: waiting-on-validation
- Budget: 5 active hours maximum
- Prerequisites: deployed SharePoint receipt list and existing authorized connection
- Exact write set: isolated exporter acceptance fixtures; `ACORN SharePoint Pilot Logbook Export`; `_assets/sharepoint/`; SharePoint runbook and guide evidence
- Primary integration lock: SharePoint pilot logbook export
- Outcome: Accept independent downloaded bytes, concurrent arrivals, empty list, malformed rows, pagination overflow and 10 MiB boundary behavior without disturbing operational intake.
- Evidence: Serialized initial/repeat runs and exact flow readback passed. A later six-row download preserved the earlier three-row prefix byte-for-byte; three selected rows passed local ACORN ingestion and reporting. See [live demonstration](sharepoint-pilot.md#october-26-live-demonstration-and-local-report). Full stored-payload comparison and aggregate edge cases remain.
- Focused validation: Download and independently compare every source line and Unicode byte; add an isolated arrival across the cutoff; verify empty output and failure retention at row/byte boundaries. Do not inject malformed rows into the live list.
- Handoff: Preserve evidence and prune accepted cases; do not treat successful small runs as overflow or concurrency acceptance.

## PA-22 — Accept standard-user access and manual CLI handoff

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: published separate SharePoint app; intended standard-license participant and approved pilot access. Owner-context downloaded-file-to-CLI handoff is recorded; participant-context access remains.
- Exact write set: sharing/run-only settings of the named SharePoint pilot app, flows and storage; `.wiki/features/powerapps-jsonl-intake/sharepoint-pilot.md`; pilot runbook and user/developer guides under this plan
- Primary integration lock: Power Platform deployment
- Outcome: An intended participant launches, loads projects and submits without a premium prompt; a downloaded collected logbook passes local ACORN stage/inspect with receipt identity preserved.
- Focused validation: Use the participant's actual entitlement and permissions; verify Teams feedback and their exported receipt in the local batch. Preserve the accepted owner-context three-row CLI/PDF proof rather than repeating it as a new claim of standard-user access. Owner premium access or a Standard badge alone is insufficient. Access testing can proceed before other edge-case matrices finish.
- Handoff: Record participant-context acceptance and local batch proof; do not switch the operational link or disable GitLab.

## PA-24 — Produce and restore the native SharePoint backup

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: current SharePoint app/flow revisions frozen for export; authorized isolated import target and test storage
- Exact write set: read-only exports of `Sponsor Highlights - SharePoint Pilot`, its catalog/intake/combined-export flows and references; isolated SharePoint restore copies; `_assets/sharepoint/backup/`; `sharepoint-pilot.md`
- Primary integration lock: Power Platform packaging
- Outcome: A native importable app/flow backup and documented list/library schemas restore as isolated components with separate destinations and recovery storage.
- Focused validation: Inspect dependencies, archive hashes and absence of secrets; import without overwriting operational components, bind test storage, and prove catalog, receipt export and collected-logbook readback. State explicitly that solution exports do not back up SharePoint rows or credentials.
- Handoff: Retain native files, schema/data recovery instructions and restore proof; source snippets and GitLab PA-17 archives do not satisfy this phase.

## PA-25 — Accept a SharePoint pilot cutover and rollback

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-19–24 accepted; owner selection of the replacement link and intended audience
- Exact write set: pilot entry links, selected GitLab flow enablement only if explicitly authorized for cutover, SharePoint/GitLab runbooks and user guides
- Primary integration lock: Power Platform deployment
- Outcome: The chosen default route and rollback procedure are explicit, with no dual submission or loss of unresolved receipts.
- Focused validation: Prove the published entry link, audience access and receipt routing; reconcile pending receipts by their original route. Preserve GitLab credentials/components for rollback and future premium use. Do not infer permission to disable it from this plan.
- Handoff: Record the actual route decision and rollback evidence; shared-environment migration remains PA-12.

## PA-15 — Accept bounded inbox and field limits

- Status: waiting-on-validation
- Validation readiness: All listed field-limit and field-rejection cases passed in both environments. ORNL unsafe-tail and one-byte-over-cap rejection also passed with unchanged bytes/revision. The requested five-attempt series completed 34 runs with three new accepted cases. Each still-open case received five attempts; GitLab 403/504 responses prevented acceptance. Remaining cases need runnable connector calls and independent byte/revision verification. See boundary-acceptance.md. The actual append cap is 10,485,759 UTF-8 bytes. PA-02 remains closed with the connectivity limitation accepted.
- Budget: 5 active hours maximum
- Prerequisites: safe isolated fixtures and restored baseline; authorized connector access sufficient to execute each case
- Exact write set: pilot project `22464` isolated intake test files; validation actions in isolated intake flow copies in both test environments; `.wiki/features/powerapps-jsonl-intake/boundary-acceptance.md`; `.wiki/features/powerapps-jsonl-intake/_assets/boundary-tests/`
- Primary integration lock: intake route
- Outcome: Accept mixed-line preservation and exact-cap append in both environments, plus unsafe-tail and one-byte-over-cap rejection in Jason. Preserve accepted field and ORNL rejection evidence.
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
- Outcome: Produce a complete native importable backup set with dependency order and restore instructions after the isolated connector/reference export repair.
- Focused validation: Inspect exported app, flows, connector and connection references; verify no secret values; perform import validation in an authorized disposable target when available, otherwise record the export as partial evidence and keep PA-17 open until restore verification is accepted.
- Handoff: Record export version, hashes and restore result. A source ZIP alone does not satisfy this phase.

## PA-12 — Move the accepted pilot to an approved shared environment

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: selected route accepted (SharePoint PA-19–24, or GitLab PA-15–17); named approved GCC environment, maker/sharing rights, route-appropriate licensing, owner and recipient group/channel
- Exact write set: target Power Platform solution import, protected connections, app/flow sharing and notification destination; `.wiki/features/powerapps-jsonl-intake/shared-deployment.md`
- Primary integration lock: Power Platform deployment
- Outcome: Deploy the accepted app/flows and switch future notifications from Jason to the approved shared destination.
- Focused validation: Verify catalog, append/replay and notification deduplication under intended user permissions; preserve old receipt recipients and rollback; a new Teams channel has its own delivery state.
- Handoff: Preserve destination IDs and protected reference names; prune only after target acceptance. This is outside the current pilot MVP.
