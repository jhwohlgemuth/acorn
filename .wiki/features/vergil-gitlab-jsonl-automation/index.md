---
type: Plan
title: "Automated JSONL management with the Vergil GitLab bot"
description: "Deploy and accept unattended JSONL admission, human-reviewed graduation, cleanup, reporting and callbacks on Vergil after the independent manual MVP."
difficulty: high
impact: high
package: [acorn-cli, acorn-lib, acorn-schema]
tags: [vergil, gitlab, bot, jsonl, graduation, reporting, callbacks]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Automate the GitLab inbox lifecycle on Vergil: durable admission, conflict-safe inbox cleanup, routed review branch/MR, authorized graduation, human merge, scheduled report publication and authenticated callbacks. This plan is a later, independent GitLab track. The working SharePoint list -> downloaded snapshot -> local stage/inspect/apply/report workflow does not wait for it.

Supporting material: [ACRE configuration examples](configuration-examples.md), [callback schema](_assets/callback-event.schema.json), and [notification walkthrough](../powerapps-jsonl-intake/notification-walkthrough.md). Callback/bot sources and acceptance files are in `_assets/`.

# Current evidence

October 26 SharePoint-aware review against `58c22da4` plus the current working tree retained these deployment gates. Focused source checks and retained evidence do not establish new Vergil service or callback acceptance. SharePoint is now the accepted default, and the original GitLab intake/catalog flows are Off. The SharePoint route hands a downloaded local JSONL file to manual CLI ingestion; this plan does not implement an automatic SharePoint-to-GitLab or SharePoint-to-Vergil bridge.

- October 24 owner-only callback routing is saved and tested: new `mr-created`/`report-no-changes` receipts use Jason's Teams self-chat; review requests, completed artifacts and failures use email. Existing receipts keep their frozen destination. Routine replay skipped delivery; a milestone email test passed. [Walkthrough and evidence](../powerapps-jsonl-intake/notification-walkthrough.md). The manual receiver still has no authenticated external callback trigger; bot phases remain open.

- Historical code review on October 24, 2026 used `49b4700d` plus that day's intake changes. Shared `Submission::admit`, `powerautomate/jsonl.rs` durable admission, `gitlab/bot.rs` snapshot cleanup/review processing and paginated cursors are implemented. `gitlab/tests/bot.rs` covers append-cleanup races, ambiguous success, conflicting IDs, restart, storage failure and admitted-route stability. The recorded earlier local workspace validation passed 2,694 tests with 17 skipped; this documentation review did not rerun it. No retained evidence establishes a Vergil service boot or live bot acceptance.
- Durable callback IDs come from persisted keys and remain stable across retry/restart. The owner-only manual Power Automate receiver `dd1d8f67-116c-f55d-7752-7c7d322691c5` has a unique Dataverse receipt key, frozen payload/recipient and Sending/Sent tracking. Three controlled Jason emails, sequential/concurrent deduplication, payload rejection and a simulated uncertain-send hold passed. External authentication, HTTP acknowledgements and real sender lost-response/restart behavior remain unproven.
- Separate intake and Vergil project tokens exist; intake/catalog bindings are verified. The Vergil protected runtime binding and service authority remain unverified. Token references are `acorn-powerautomate-intake` and `acorn-vergil-bot`, with recorded expiry November 23, 2026; recheck expiry before deployment without exposing values.
- Pilot: GitLab `22464`, `main`, `.acorn/intake/logbook.jsonl`; initial folder `acorn`; Power Platform environment **Wohlgemuth, Jason** and recipient Jason Wohlgemuth. PA-02 is closed with intermittent cloud-connector access recorded as a [known pilot limitation](../powerapps-jsonl-intake/intake-access-decisions.md#known-limitation-and-pa-02-closure); this does not establish Vergil runtime access. Future shared deployment is PA-12 in that plan.
- The root reporting include tracking `main` does not satisfy the pilot's immutable-template requirement. Local `report --combine` already renders a combined PDF, and normal report mode implements bounded per-project manifests, package publication and callback intents. Those paths are in `commands/report/mod.rs` and `io/api/gitlab/package.rs`; local renders do not establish scheduled publication. Live boot, ingestion/cleanup, review/merge, scheduled publication and routing acceptance remain. Full current deployment evidence is in the [runbook](../powerapps-jsonl-intake/pilot-runbook.md).

# Boundaries and dependencies

The [form plan](../powerapps-jsonl-intake/index.md) owns producer behavior and Power Platform packaging; the [manual walkthrough](../manual-jsonl-ingestion-graduation/walkthrough.md) documents the implemented local commands and early demo. This plan consumes the same frozen envelope, canonical folder authorization and graduation contract. It does not block local stage/inspect/apply, CLI graduation or dry-run PDF generation.

The SharePoint list is an immutable receipt history, and its collected JSONL is a rebuilt snapshot, not a GitLab inbox to drain. No phase here reads, cleans up or acknowledges SharePoint rows. The future [OAuth download plan](../sharepoint-jsonl-download/index.md) provides interactive delegated capture for an operator; its device sign-in and refresh cache are not an unattended Vergil authentication design. A future bridge would need its own source ownership, durable cursor/admission, replay and acknowledgement contract before deployment. Do not stage manually applied SharePoint rows into the bot inbox or seed its database as an implied handback.

The owner-only callback receiver remains separate from the standard SharePoint app. Its Dataverse/external-auth requirements and this plan's GitLab/MR/package dependencies do not become prerequisites for standard form access or local PDFs. Keep routine save Teams notices, optional manual milestone notifications and future authenticated bot delivery distinct.

BOT-01 resolves bot/review ownership and callback authentication decisions formerly bundled into PA-02. It supplies the protected GitLab runtime binding for PA-03; **service boot can be tested before callbacks** when callback configuration is omitted. PA-04 separately connects the authenticated external receiver. PA-05 waits for service acceptance and explicit source ownership/handoff, not shared-environment migration. Enable only approved callback settings and preserve durable queue semantics while delivery is unavailable. PA-05 may already create the draft MR and queue callbacks; PA-06/07 accept duplicate-safe review/delivery and human graduation/merge; PA-08 then publishes an artifact, BOT-03 accepts schedule/replay, and PA-09 accepts routed end-to-end behavior.

Manual receipts are not bot admission rows. The [handback procedure and offline rehearsal](../manual-jsonl-ingestion-graduation/handback.md) are accepted. For a GitLab inbox only, before touching a manually processed source, PA-05 must approve its actual reconciled source/index baseline using that procedure, or certify a fresh isolated fixture with no manual processing. The rehearsal does not authorize production cleanup or seed the bot database. Never run one-shot intake and the service against the same inbox concurrently. No bot self-merges; human review/merge is explicit. No gateway or public Vergil endpoint is required for GitLab polling.

The current `PowerAutomateCallbackResolver` in `crates/acorn-lib/src/io/api/powerautomate/callback.rs` resolves a protected URL into a destination with no extra headers or signer. It supplies neither an Entra bearer token nor an outbound signature, despite the generic webhook library supporting those mechanisms. Approve a sender-compatible authenticated endpoint or intermediary before PA-04; do not make a trigger public to make the demo easier. Keep the startup inbound secret local to Vergil; it is not an outbound callback signature. Preserve one durable delivery ID, durable acceptance before 2xx, safe uncertain-email reconciliation and per-channel state. An ambiguous non-idempotent email send cannot promise exactly-once delivery.

# Acceptance and validation

One authorized envelope must yield one routed draft MR and reviewed canonical entry, exact-byte inbox cleanup, one immutable report package, and the correct durable callback outcome after replay/restart. Two authorized routes affect only their indexes; an unauthorized record affects none. Preserve rejected/unadmitted and concurrently appended bytes. Record actual flow, service, commit, merge request, package and receipt identities with secret values redacted. Reporting worker isolation remains owned by [runner-pool isolation](../../refactors/ci-runner-pools.md); target-specific ASPECT graduation remains in [sponsor highlights clarity](../sponsor-highlights-clarity.md).

Each phase is a bounded external acceptance outcome. If a code or shared template defect is discovered, add a dedicated repair phase with exact files rather than extending an operational phase. Retire only this plan after its acceptance; preserve the independent form plan and durable manual guides.

# Operator handoff

Use the [Vergil operator runbook](vergil-operator-runbook.md) after the relevant deployment gates below. Commands are procedures, not acceptance evidence.

# Execution phases

`waiting-on-validation` (“Waiting on validation”) means the implementation exists and the phase awaits acceptance evidence. Prerequisites and external blockers still apply; this status does not mean ready to execute or accepted. Phases that still need deployment, integration or policy setup retain their existing status.

## BOT-01 — Establish bot authority, credentials and callback contract

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: Vergil deployment owner, human review authority and callback authentication authority available
- Exact write set: Vergil protected runtime configuration and GitLab principal binding; Power Platform callback policy; `.wiki/features/vergil-gitlab-jsonl-automation/bot-access-decisions.md`
- Primary integration lock: deployment approvals
- Outcome: Name owners, verify branch access with the existing bot credential reference and approve the exact external callback authentication/intermediary contract.
- Focused validation: Dated authority/secret-reference records, no exposed values; current credential validity; approved ingress/egress and licensing; verify configured recipient/store IDs. An OAuth-only receiver cannot be assumed compatible with the current sender.
- Handoff: Release PA-03 boot and PA-04 endpoint integration independently; no shared deployment requirement.

## PA-03 — Prove a native ARM64 Vergil service boot

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BOT-01 GitLab/service approval; pinned ARM64 binary and protected GitLab configuration
- Exact write set: Vergil managed service definition, persistent operation database and protected ACRE/runtime config without JSONL intake; `.wiki/features/vergil-gitlab-jsonl-automation/vergil-boot.md`
- Primary integration lock: Vergil service
- Outcome: Run native ARM64 ACORN polling with supervision, healthy state, a successful GitLab poll and retained cursor after restart.
- Focused validation: Check architecture/version, /health, /ready and /state poll_count/last_error; restart with same database and no --after; redact logs. Omit callback settings when not yet accepted.
- Handoff: Record service revision and GitLab access; healthy review-note polling does not establish JSONL intake acceptance.

## PA-04 — Connect the authenticated external callback receiver

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BOT-01 approved endpoint/authentication; existing manual receiver/ledger accepted
- Exact write set: pilot callback flow HTTP trigger and response contract; protected `POWER_AUTOMATE_CALLBACK_URL` and approved auth references; `.wiki/features/vergil-gitlab-jsonl-automation/callback-integration.md`
- Primary integration lock: Power Automate credentials
- Outcome: Connect the implemented receipt and Teams/email receiver to the approved sender contract using webhook-id and exact event bytes.
- Focused validation: Reject unauthorized requests; validate all five event shapes; durable receipt before 2xx; exact replay after lost response/flow restart; one recorded send or explicit uncertain-send hold; verify run-history redaction and preserve recipients.
- Handoff: Supply only protected endpoint references and delivery acceptance evidence; any sender-auth implementation gap needs a separate bounded phase.

## PA-05 — Accept one-shot JSONL admission on Vergil

- Status: waiting-on-validation
- Validation readiness: Admission and cleanup are implemented. Live acceptance requires PA-03 and approval of the actual reconciled baseline or fresh isolated fixture.
- Budget: 5 active hours maximum
- Prerequisites: PA-03; owner-approved source/index baseline using the [verified handback procedure](../manual-jsonl-ingestion-graduation/handback.md), or certified fresh isolated inbox; approved canonical folder/member registry
- Exact write set: Vergil JSONL source config and persistent database; pilot project `22464` isolated inbox/index baseline and resulting review branch/draft MR; `.wiki/features/vergil-gitlab-jsonl-automation/vergil-intake.md`
- Primary integration lock: intake route
- Outcome: Record the actual source/index/config commits and digests, reconciliation approval or fresh-fixture certificate, and worker/database owner. Stop the service and run one authorized `acorn intake run --config` against the owned GitLab snapshot (never the SharePoint list/export), prove durable admission and exact cleanup, and inspect any resulting draft MR before resuming that service/database. This command also executes bounded review work; it is not an admission-only switch.
- Focused validation: Preserve UUID/digest/routing, validate authorization independently of catalog membership, inspect exact admitted bytes and queued review operation, confirm nonadmitted lines remain. Do not configure unaccepted callback delivery.
- Handoff: Freeze source/config/service revisions and any review branch/MR plus queued callback IDs for recovery and PA-06. The one-shot command does not run the callback worker; do not resume callback delivery until PA-04 is accepted. Producer boundary cases stay in the form plan.

## BOT-02 — Accept append-cleanup races and restart recovery

- Status: waiting-on-validation
- Validation readiness: Recovery behavior is implemented and locally tested; the remaining work is the live isolated recovery matrix after PA-05.
- Budget: 5 active hours maximum
- Prerequisites: PA-05; approved isolated fault fixtures
- Exact write set: pilot project `22464` isolated inbox concurrency fixtures; Vergil service/database test state; `.wiki/features/vergil-gitlab-jsonl-automation/bot-recovery.md`
- Primary integration lock: intake route
- Outcome: Prove snapshot admission and cleanup preserve concurrent append/rejection and recover after lost responses/restarts.
- Focused validation: Test before/after-cleanup response loss, changed bytes under one ID, malformed/unauthorized lines and concurrent append; immutable admission never reroutes on changed config. Inspect exact bytes and operation counts.
- Handoff: Record recovery evidence without touching production receipts or claiming producer concurrency coverage.

## PA-06 — Accept duplicate-safe review and callback delivery

- Status: waiting-on-validation
- Validation readiness: Review operations and durable callback intents are implemented; end-to-end acceptance waits for PA-05 and the external callback integration in PA-04.
- Budget: 5 active hours maximum
- Prerequisites: PA-05, PA-04; authorized isolated submission and review authority
- Exact write set: pilot project `22464` bot review branch and draft merge request; Vergil operation/callback database; `.wiki/features/vergil-gitlab-jsonl-automation/bot-review.md`
- Primary integration lock: isolated pilot repository
- Outcome: Inspect the draft MR and callback intents produced by PA-05, then prove exactly one correctly routed review operation and accepted callback outcome after replay. Create a fresh isolated fixture only if the earlier run did not produce review work.
- Focused validation: Inspect receipt, admission ID, exact-record cleanup, branch/index path, proposed entry, `acorn intake export` diff/receipt and callback; restart/replay preserves one operation.
- Handoff: Hand the exact MR and source evidence to the authorized human reviewer.

## PA-07 — Graduate and merge the reviewed entry

- Status: waiting-on-validation
- Validation readiness: The supported graduation path is implemented; live review-note acceptance and human-approved merge wait for PA-06 and the named reviewer.
- Budget: 5 active hours maximum
- Prerequisites: PA-06; named human reviewer and merge authority available
- Exact write set: pilot project `22464` reviewed merge request and `acorn/index.json`; `.wiki/features/vergil-gitlab-jsonl-automation/bot-graduation.md`
- Primary integration lock: pilot merge request
- Outcome: Accept a supported graduation through the polled review-note path and complete human approval/merge.
- Focused validation: Confirm authorized note through pagination/cursor recovery; compare selected candidate, retained source and canonical diff; record exact human-merged commit. Do not treat an ordinary update as a graduation candidate.
- Handoff: Supply the reviewed merged revision to reporting; preserve manual/automated graduation distinctions.

## PA-08 — Publish the single-project pilot artifact

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-07, PA-04; approved reporting runner isolation (RUN-04/05 evidence or an equivalent approved pilot setup) and existing template usable
- Exact write set: project `22464` reporting include in `.gitlab-ci.yml`; generic package `acorn-automated-artifacts`; `.wiki/features/vergil-gitlab-jsonl-automation/bot-report.md`
- Primary integration lock: scheduled artifact publication
- Outcome: Pin the shared reporting include to an immutable revision and publish one report package plus artifact-ready callback from the exact merged source.
- Focused validation: Compare local dry-run at fixed commit/cutoff with published manifest/PDF/checksum/package and callback receipt; keep TEST ONLY data out of real sponsor output. Unexpected shared-template defects need their own repair.
- Handoff: Record immutable artifact evidence; schedule/replay acceptance is BOT-03.

## BOT-03 — Accept scheduled publication and callback recovery

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-08; approved schedule owner and protected reporting variables
- Exact write set: project `22464` protected reporting schedule; package replay fixtures; Vergil callback queue; `.wiki/features/vergil-gitlab-jsonl-automation/scheduled-replay.md`
- Primary integration lock: scheduled artifact publication
- Outcome: Activate the approved schedule and prove repeated publication and callback delivery have no duplicate accepted effect.
- Focused validation: Inspect package count, unchanged/report-no-changes behavior, persisted webhook-id through sender restart/lost HTTP response and channel receipt state; uncertain email remains held for review.
- Handoff: Record schedule ownership, cadence, replay and recovery evidence before final routing acceptance.

## PA-09 — Validate routed automation and retire the bot plan

- Status: waiting-on-validation
- Validation readiness: Routing behavior is implemented; final live end-to-end acceptance and its operator record wait for BOT-02 and BOT-03.
- Budget: 5 active hours maximum
- Prerequisites: BOT-02, BOT-03; two authorized routes and one unauthorized fixture
- Exact write set: project `22464` isolated ACRE registry and routed indexes; `.wiki/features/vergil-gitlab-jsonl-automation/bot-operations.md`; this plan and portfolio index/network/lane entries
- Primary integration lock: Power Automate operational record
- Outcome: Prove two authorized routes and unauthorized no-write behavior through automated review/reporting, publish the durable operator runbook, and retire only this plan.
- Focused validation: Inspect isolated routed diffs, admission/cleanup, review/merge, artifacts and callbacks; preserve rejected source; verify handback/reset and internal links; run wiki validation.
- Handoff: Remove this plan and its portfolio entries after preserving evidence. Independent form/shared-deployment and manual work retain their own lifecycle.
