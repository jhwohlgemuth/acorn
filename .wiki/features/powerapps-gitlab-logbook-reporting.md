---
type: Plan
title: "Complete GitLab JSONL logbook reporting"
description: "Expose durable callback identity, deploy protected Power Automate flows, and validate live GitLab logbook reporting on Vergil."
difficulty: high
impact: critical
package: [workspace, acorn-lib, acorn-cli, acorn-schema]
tags: [feature, automation, powerapps, powerautomate, gitlab, jsonl, local, logbook, reporting, pdf]
status: in-progress
priority: next
execution: phased
phase_budget_hours: 5
---

# Goal

Complete the protected Power Automate intake and callback flows, then validate the Power Apps to GitLab to human review to report and callback path on Vergil.

# Current evidence

- The ACRE 2.0.0 configuration and strict form envelope are the accepted contract in `ACRE_SPEC.md` and `docs/src/concepts/acre.md`. Project routing uses a configured canonical folder ID or alias; destination and digest are fixed at admission.
- The GitLab bot now snapshots and durably admits JSONL before conflict-safe cleanup, routes mixed-project records to their selected indexes, and persists review-event cursors across pagination and restarts. Local tests cover replays, concurrent inbox edits, rejection isolation, and routed review publication.
- The offline manual MVP now stages, inspects, applies, and resumes a mixed-project JSONL batch. `make lint`, `make test` (2,525 passed), focused batch tests, and a disposable CLI run verified four lines across three local indexes, including two entries for one project, exact routes, replay, no-write failures, and receipt recovery. The local operator contract is documented in `ACRE_SPEC.md` and the intake CLI guide.
- The local batch and GitLab JSONL adapter now consume `Submission::admit` directly for canonical folder identity, authorization, normalized entry, and stable operation identity. The 39 focused admission, GitLab review, and bot cleanup tests pass, including preserved form fields and notifications, canonical alias cutoff checks, rejection before external reads, immutable replay after access/routing changes, and cleanup concurrency. Focused package Clippy passes with warnings denied. The GitLab destination and review branch remain fixed at admission.
- A source review at `1ff52b2a` found a remaining callback identity gap: `PowerAutomateCallbackResolver` returns a URL-authenticated destination without a signer or additional headers. `CallbackWorker::run_once` sends `callback_key` as `webhook-id` only through `dispatch_signed`; the unsigned path sends neither that ID nor an equivalent body field. Existing queue deduplication tests do not prove receiver deduplication when delivery succeeds but its response is lost. PA-12 owns this bounded repair; PA-04 owns receiver persistence and notification recovery.
- Remaining external implementation is the callback flow and its delivery ledger (PA-04), the JSONL append/reconciliation flow and canvas action (PA-05), the Vergil service/configuration (PA-03), and the pilot's pinned reporting include and schedule (PA-08). No exported Power Platform solution or canvas source was found in this checkout. The root reporting include currently tracks `main`; that does not establish the pilot's required pinned include. External flow, template, and host state were not inspected during this review.
- The published tester canvas sent Teams and email notifications through its attached notification-only flow. The separate GitLab JSONL intake draft is off and unattached. No tested submission was persisted to CSV or JSONL by Power Automate. The pilot flow and service still require PA-02 through PA-09.
- A read-only gate audit on 2026-10-17 found project `22464` suitable for an isolated pilot branch, but did not establish service credentials, cloud-to-GitLab reachability, callback policy, or an approved shared GCC environment. The PA-02 handoff below records the remaining decisions.

# Selected transport and boundaries

- The offline manual batch accepts bounded LF-terminated JSONL and standalone `file:` project indexes. It retains exact source bytes, per-project previews, and durable receipts; the GitLab flow remains a separate transport and review adapter.
- The app-triggered flow appends one UTF-8 JSONL `FormSubmission` per line to `.acorn/intake/logbook.jsonl` on one approved GitLab branch. It returns a queued receipt with stable submission ID and confirmed commit SHA; ACORN admission and publication have separate receipts.
- Vergil polls an explicitly configured repository file and review-event pages, keeps durable snapshots and operation IDs, removes only admitted bytes, and reports failures without losing rejected or unadmitted records. Existing HTTP webhook support remains independent of this route.
- The bot prepares a project-specific review branch and draft merge request. A person approves and merges; the bot does not merge its own changes. The scheduled job publishes an immutable PDF and manifest package, then an explicit callback command delivers metadata to a protected Power Automate flow.
- Credentials live in approved protected stores and never in source, ordinary run history, app controls, logs, or artifacts. There is no gateway or public Vergil endpoint in the selected GitLab-mediated path. If the tenant rejects the legacy HTTP callback trigger mode, PA-02 records an approved intermediary before PA-04 begins.

# Dependencies and acceptance

PA-12 can expose durable callback identity without external approval. PA-02 resolves owners, access, callback authentication, and the receiver ledger; PA-04 consumes PA-12 and those decisions to provision protected credentials and duplicate-safe notifications; PA-03 proves authenticated Vergil polling. PA-05 waits for those external gates before connecting the live flow to the implemented shared admission path. PA-06 through PA-09 prove review, publication, callbacks, and routing. The completed offline manual MVP is available as an operator fallback, with an explicit repository and inbox handoff before a GitLab bot resumes. The [runner-pool plan](../refactors/ci-runner-pools.md) owns reporting worker isolation, and the [database artifact plan](database-backed-artifact-generation.md) owns general database-backed rendering.

GitLab acceptance requires one authorized submission to produce one correctly routed review operation and human-merged entry, one immutable report package, one delivered notification event, and no duplicate effects after replay or restart. Two authorized project selections must affect only their configured indexes; an unauthorized selection must affect none. The redacted evidence bundle must identify exact flow, service, commit, merge request, package, callback, and replay outcomes. On completion, move the durable runbook to `.wiki/assets/powerautomate-logbook-reporting.md` or user documentation and retire this plan from every portfolio surface.

# Validation

Use the focused validation in each remaining phase. Inspect protected credentials and run-history settings without recording secret values. Verify live GitLab append/cleanup concurrency, restart behavior, human review, artifact checksum, callback delivery, and duplicate-safe replay before widening the pilot. Run the wiki validator after every phase is pruned.

# Vergil operator commands

Use the approved native ARM64 `acorn` binary, protected ACRE file, and service environment on Vergil. The protected environment supplies `CI_SERVER_HOST=code.ornl.gov`, a bot-scoped `GITLAB_TOKEN`, a persistent `ACORN_DATABASE_PATH`, and `POWER_AUTOMATE_INBOUND_SECRET` referenced by `variables.custom`; it supplies `POWER_AUTOMATE_CALLBACK_URL` when callbacks are enabled. Do not put their values in command arguments, source, or ordinary logs. Confirm `uname -m` is `aarch64` or `arm64` and record `acorn --version` with the pinned binary revision. Use one durable database for the bot and one-shot commands; stop the service before manually processing the same pilot inbox.

PA-03 starts the managed service with a pilot-project ID and a first-boot ISO 8601 UTC cursor chosen before the first review event. Use the [ACRE example](../assets/acre-logbook-configuration-examples.md) only after replacing its placeholder project, folders, and variable references with approved values; omit `options.powerAutomate.intake` for this first boot. PA-05 adds that source after the flow and branch are approved. On later restarts, retain the same database and omit `--after` so the durable event cursor controls the resume point. Keep the HTTP listener on Vergil loopback; the selected path uses GitLab polling and needs no public endpoint.

Run this foreground command in the service supervisor or a dedicated shell:

```sh
acorn serve bot 22464 --config "$ACORN_CONFIG" --event-source poll \
  --bind 127.0.0.1:3000 --poll-interval 10 --after "$INITIAL_AFTER"
```

From a second shell on Vergil, check the running service:

```sh
curl -fsS http://127.0.0.1:3000/health
curl -fsS http://127.0.0.1:3000/ready
curl -fsS http://127.0.0.1:3000/state
```

After at least one polling interval, `/state` must show `poll_count` above zero and `last_error` absent. `/health` and `/ready` establish local liveness and operation-runtime readiness; the successful state poll establishes a GitLab review-event read. Check supervised restart, the retained cursor, and redacted logs. When PA-05 has an approved pilot inbox, stop the bot before a one-shot intake check; `intake run` can admit records and process review work, so use only the authorized isolated branch and inspect its JSON summary before restarting the service.

After stopping the first service instance, use this command for the restart check and the managed service definition; leave out the first-boot cursor:

```sh
acorn serve bot 22464 --config "$ACORN_CONFIG" --event-source poll \
  --bind 127.0.0.1:3000 --poll-interval 10
```

Run the second-shell `/state` check again and compare its cursor with the durable pre-restart value.

```sh
acorn intake run --config "$ACORN_CONFIG"
acorn intake export --config "$ACORN_CONFIG" \
  --submission-id "$SUBMISSION_ID" --output change.diff
acorn intake deliver --config "$ACORN_CONFIG"
acorn report --dry-run --config "$ACORN_CONFIG" --root "$REVIEWED_CHECKOUT" \
  --commit "$MERGED_SHA" --through "$REPORT_CUTOFF" \
  --project "$PROJECT_ID" --output "$REPORT_OUTPUT"
```

Run `intake export` after PA-06 creates the draft merge request, `intake deliver` when PA-04 has a callback flow and a queued intent exists, and the report dry-run after PA-07 records the exact merged commit. PA-08 compares the local PDF and manifest with the immutable published package. PA-09 moves this verified command sequence into the durable operational runbook.

# Execution phases

## PA-12 — Expose durable callback identity on unsigned delivery

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none; source review baseline is the clean worktree at `1ff52b2a`
- Exact write set: `crates/acorn-lib/src/io/api/webhooks/store.rs`, `crates/acorn-lib/src/io/api/webhooks/tests.rs`, `docs/src/concepts/acre.md`, `.wiki/features/powerapps-gitlab-logbook-reporting.md`, `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: durable callback dispatch
- Outcome: Every durable unsigned callback carries one authoritative `webhook-id` derived from its stored callback key. The ID and payload remain identical across retries and process restarts. Signed delivery retains its existing Standard Webhooks contract. This header supplies identity, not authentication.
- Focused validation: Extend `RecordingService` or a dedicated fake receiver to accept a notification and lose the HTTP response, then restart the worker against the same database and make the retry due without sleeping. Assert identical ID/payload on both attempts and one receiver effect when keyed by that ID. Assert distinct callback keys with identical payloads remain distinct; a resolver cannot substitute a different ID; malformed header values fail without dispatch; existing signed verification, attempt accounting, URL/header redaction, and one-shot HTTP safety still pass. Use `cargo nextest run -p acorn-lib --lib --features agentic -E 'test(io::api::webhooks::tests::) | test(io::api::powerautomate::tests::test_callback_)'` and focused package Clippy with warnings denied.
- Handoff: Document `webhook-id`, its durability, and receiver responsibilities in the ACRE guide, then remove PA-12 and its PA-04 prerequisite and reconcile the owned wiki surfaces. No database migration, event-body change, CLI syntax, or OAuth acquisition is required. Real notification delivery still requires PA-04.

## PA-02 — Resolve production owners and access gates

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: existing policy and owner records or available authorized decision makers; the notification-only and local artifact pilots are verified
- Exact write set: Power Platform environment policy, isolated GitLab project `22464` access policy, `.wiki/features/powerapps-gitlab-logbook-reporting.md`
- Primary integration lock: deployment approvals
- Outcome: Name owners and record approved environment, GitLab API route for the future app-triggered flow, callback-trigger, licensing, branch permissions, and secret-storage decisions. No gateway approval is required.
- Focused validation: Verify every stop gate has an accountable owner and dated evidence.
- Handoff: Remove PA-02 and record the approved GitLab target and outbound access requirements for PA-04, then PA-03.

### PA-02 implementation preparation, 2026-10-20

PA-02 is the next external phase and remains blocked: the repository review found no decision record superseding the 2026-10-17 audit. PA-12 can proceed independently. The audit is historical evidence, not a current approval or live connectivity check. Waiting for decision makers is a prerequisite outside the five-hour active budget.

For each row below, record the accountable owner, approving authority, decision, UTC date, and a reference to the approval or policy record. Record identifiers and protected-store references only; secret values belong in PA-04's approved stores.

| Required decision | Evidence needed to release the gate |
| --- | --- |
| Shared Power Platform deployment | Approved GCC environment ID, app/flow owners, maker and sharing rights, dependent-resource access, and applicable final-flow licensing decision. |
| GitLab integration | Approved service/API route, isolated project `22464` branch and inbox path, separate flow and bot principals, authorized scopes/roles, branch permissions, and outbound access requirements. Current user push rights do not prove service access. |
| Callback | Approved callback trigger authentication mode, or a named approved intermediary if the legacy mode is disallowed; callback owner, recipients, secure run-history requirements, and a durable receiver ledger with atomic unique callback IDs and per-channel recovery records. |
| Protected stores and Vergil operations | Named credential stores and custodians, Vergil service owner and deployment authority, and ownership of callback URL and startup inbound-secret references. Provisioning and service boot belong to PA-04 and PA-03. |
| Review and notifications | Named human review/merge authority and accountable Teams/email destination owners. |

Release PA-02 only when every row has dated approval evidence and no unresolved access decision. Policy questions require authorized decision makers; do not infer approval from the developer environment, a notification-only run, or the signed-in user's project role. The current callback resolver sends no Entra bearer token: an OAuth-only trigger needs the approved intermediary unless a separate bounded authentication phase is scoped. Microsoft documents the tenant-restricted trigger modes and cloud-specific token audiences in its [HTTP trigger authentication guide](https://learn.microsoft.com/en-us/power-automate/oauth-authentication). Preserve the approved target and secret references as the PA-04 handoff; PA-04 also needs PA-12, PA-03 follows PA-04, and PA-05 follows PA-03/04. No external system is changed during preparation.

### PA-02 gate audit, 2026-10-17

| Gate | Verified evidence | Decision still needed |
| --- | --- | --- |
| Power Platform | Published tester pilot works in the `221f…` developer environment; **Sponsor Highlights** is owned by Jason Wohlgemuth and currently designated Standard. Enterprise Test and Enterprise Prod appear as viewable environments. | ORNL-approved shared GCC environment, maker and sharing rights, dependent-resource access, and final-flow licensing under [Microsoft's current instant-flow rules](https://learn.microsoft.com/en-us/power-platform/admin/power-automate-licensing/faqs). |
| GitLab | Internal project `22464`, `main` protected for Maintainers, pilot branch unprotected at `447fbd45084496006a70aa9c0bdd2ef5db4c1b75`; signed-in owner can push. Pre-existing `responses.csv` on `main` is unrelated to the Power Apps test; no project CI variables are configured. | Approved branch and separate flow/bot principals with the required scopes; approved API route for the future flow. Live reachability and write proof belong to PA-05. |
| App-triggered intake flow | The tested **Sponsor Highlights** app attaches only the notification flow. A separate JSONL draft in the same environment is off and has no associated app; **Highlight** has a Dataverse source and no attached flow. The default environment has **ACORN Logbook Intake** and **Test**. The user confirmed no CSV or JSONL save occurred in the tested path. | After the environment and access decisions, finish the draft's JSONL append, secure its actions, and attach it to the canvas only when the queued receipt is valid. |
| Callback | No callback flow or trigger policy was established by the notification pilot. [Microsoft describes `Anyone` as a legacy open-access trigger mode](https://learn.microsoft.com/en-us/power-automate/oauth-authentication). | Tenant approval for that mode or an approved intermediary, plus approved recipients. |
| Secrets and operations | Vergil is the selected ARM64 host; no deployment credential was placed in the repository during this audit. | Protected stores and owners for flow/bot GitLab credentials, callback URL, and Vergil service; named app/flow, review/merge, Teams, and email owners. |

### Handoff

- Verified baseline: published tester notification delivery and the read-only Power Platform and isolated GitLab observations above.
- Changed surfaces: this plan and its portfolio references. Opening the separate **Highlight** app for inspection caused Power Apps to create unpublished version 2 at 14:09:58 on 2026-10-17; version 1 remains live. No Save or Publish action was used, and no flow, GitLab branch, credential, or service was changed.
- Validation run: Power Automate run details, tester-shared Teams evidence, Power Apps editor inventories, and read-only GitLab project, branch, protection, member-role, and file-list queries.
- Remaining defect: written deployment decisions are not yet identified. Current branch access for the signed-in user does not establish service-credential access or cloud reachability; no CSV or JSONL write was part of the tested flow.
- Next action: obtain the existing policy/owner records, verify the approved integration route, and record dated decisions before removing PA-02.

## PA-03 — Prove a native ARM64 Vergil service boot

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-04
- Exact write set: Vergil managed service definition, protected Vergil runtime environment, protected Vergil ACRE configuration without `options.powerAutomate.intake`, `.wiki/features/powerapps-gitlab-logbook-reporting.md`
- Primary integration lock: Vergil service
- Outcome: Run the pinned native ARM64 ACORN service in polling mode with healthy readiness and redacted logs.
- Focused validation: Run the Vergil service and second-shell checks above; check `/health`, `/ready`, `/state` with a successful poll count and no error, process supervision, architecture, durable cursor after restart without `--after`, and absence of secret values. Readiness alone does not establish GitLab or callback reachability.
- Handoff: Remove PA-03 and record the verified service revision and outbound GitLab access. Healthy note polling does not establish JSONL intake readiness.

## PA-04 — Provision GitLab and callback secrets

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-02, PA-12; approved receiver-ledger system and actual resource identifiers are recorded before this phase starts
- Exact write set: selected intake flow's protected GitLab credential, Vergil's protected GitLab credential and runtime environment, protected `POWER_AUTOMATE_CALLBACK_URL`, approved callback flow and its callback delivery ledger
- Primary integration lock: Power Automate credentials
- Outcome: Establish separate protected GitLab credentials for the flow and bot, plus one schema-valid callback flow that persists incoming IDs and suppresses repeated notification effects. Keep any startup-required `POWER_AUTOMATE_INBOUND_SECRET` local to Vergil; the JSONL flow does not use it.
- Focused validation: Send every supported event shape, then replay an event after a lost response and a flow restart. Inspect one Teams/email effect per configured channel, durable duplicate disposition, safe handling of partial channel completion, 2xx only after durable acceptance, and secure run-history behavior.
- Handoff: Remove PA-04 and provide only secret references to PA-03 and PA-05.

### PA-04 implementation preparation

- Build the trigger schema from `ArtifactEvent` in `crates/acorn-lib/src/io/api/webhooks/event.rs`: `mr-created` and `review-needed` carry `projectId`/`mergeRequestUrl`; `artifact-ready` carries `projectId`/`packageUrl`/`checksum`/`through`; `report-no-changes` carries `projectId`/`through`; `failed` carries `projectId`/`message`. Do not require a submission UUID or operation ID in these bodies; they are not supplied. Read the delivery identity from PA-12's `webhook-id` header.
- Atomically claim each callback ID in the approved durable ledger before channel work. Store enough payload evidence to reject a reused ID with changed content. A duplicate returns the existing disposition and resumes only channel work whose outcome is known to be pending. Concurrent duplicates must not both send.
- Record Teams and email outcomes separately. A crash during send, or a connector timeout after possible delivery, requires reconciliation; do not blindly resend an uncertain channel. A ledger write alone cannot make a non-idempotent notification connector exactly once. Return success after durable acceptance only when the flow has an owned retry/recovery path; otherwise keep the sender's retry meaningful.
- Resolve protected credentials through approved connections/stores and apply [Microsoft's secure input/output settings](https://learn.microsoft.com/en-us/power-automate/guidance/coding-guidelines/use-secure-inputs-outputs-triggers) to the relevant trigger/actions. `POWER_AUTOMATE_INBOUND_SECRET` authenticates ACORN's inbound HTTP route; it is not an outbound callback signature or bearer token.

## PA-05 — Connect the GitLab JSONL intake flow

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: PA-03, PA-04 (shared form admission, source configuration schema, snapshot reader, durable admission, cleanup, poller, and mixed-project publication code are landed)
- Exact write set: selected Power Apps intake flow, canvas submit action, isolated project `22464` inbox `.acorn/intake/logbook.jsonl`, Vergil JSONL source configuration
- Primary integration lock: intake route
- Outcome: Append the strict version-1 envelope with conflict-safe GitLab retries, configure the implemented repository-file poller, and return a confirmed queued receipt with submission ID and commit SHA.
- Focused validation: With the Vergil service stopped, run one authorized `acorn intake run --config "$ACORN_CONFIG"`; verify stable UUID/content across retries, concurrent append and cleanup, ambiguous responses, serialization round-trips, protected credentials, and app distinction between GitLab persistence and ACORN admission. Restart the service against the same database.
- Handoff: Remove PA-05 and record the tested flow revision, source project/branch/path, and poller revision.
- Draft checkpoint: Flow `6039f419-420f-6922-46ef-04b00d2290f1` is saved and off with typed trigger inputs plus one-time ID and timestamp actions. Resume with the approved branch and secret references from PA-02/04, service/callback readiness from PA-03/04, then implement GitLab read/conditional write/retry, secure run history, confirm commit persistence, return the queued receipt, bind the canvas, and configure the poller. Do not enable or attach the draft before those pieces work together.

### PA-05 implementation preparation

1. Define the pending submission before the first append attempt: preserve one UUID, submission timestamp, canonical project selection, submitter identity, and exact serialized envelope throughout retries, including a repeated canvas invocation after an uncertain response. Serialize as one UTF-8 JSON object plus LF; never regenerate these values inside a retry action. Reusing an ID with changed serialized bytes is a conflict, not an edit.
2. Read the inbox and its `last_commit_id`, preserve existing bytes including rejected/blank lines, and append only the pending line. Use an atomic create/update action through the [GitLab Commits API](https://docs.gitlab.com/api/commits/) so a successful write returns its actual commit ID. On update, supply the observed file `last_commit_id`; on create, reconcile a concurrent creator. Preserve the configured byte limits and reject an existing nonterminated tail.
3. Use a bounded read/recompute/write loop. GitLab documents that [repository-file commit failures can return a nonspecific 400](https://docs.gitlab.com/api/repository_files/); do not handle only 409. Re-read on possible conflicts or uncertain responses, but stop on authentication/authorization failures or invalid input. Correlate attempts with a stable submission marker in the commit message. Before retrying an uncertain append, inspect immutable commit evidence for that marker and exact line, including when the bot has already removed the line. Current inbox absence does not prove the write failed. If persistence cannot be established within the bound, return an explicit unresolved result and retain the pending identity.
4. Return a queued receipt only with the stable submission ID and a verified commit SHA. The canvas retains unresolved submissions, suppresses concurrent duplicate submit actions, and distinguishes GitLab persistence from ACORN admission, human review, and publication. Bind the flow only after these states work together against the approved branch.
5. Stop Vergil while proving one-shot admission on the approved inbox, inspect the durable admission, exact-byte cleanup, and routed review operation, then resume the same service/database. Prove concurrent appends, append versus cleanup, malformed/unauthorized lines, changed bytes under one ID, lost write responses before and after cleanup, and whole-flow/canvas retries. Preserve the flow revision and redacted evidence before pruning this phase.

## PA-06 — Create the first duplicate-safe review operation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-05
- Exact write set: isolated GitLab project `22464`, Power Automate intake run, `.wiki/features/powerapps-gitlab-logbook-reporting.md`
- Primary integration lock: isolated pilot repository
- Outcome: Submit one authorized response and create exactly one routed branch and draft merge request.
- Focused validation: Inspect the queued receipt, durable admission and operation ID, exact-record inbox removal, routed branch path, proposed entry, `acorn intake export` diff and receipt, callback, and duplicate-safe persistence across restart.
- Handoff: Remove PA-06 and pass the merge request to human review.

## PA-07 — Graduate and merge the reviewed entry

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-06
- Exact write set: isolated GitLab merge request, `acorn/index.json`
- Primary integration lock: pilot merge request
- Outcome: Complete human review, authorized graduation, approval, and human merge while preserving project identity.
- Focused validation: Confirm the authorized graduation note is found through the paginated polling path, then compare the merged entry with the accepted submission and reviewed diff.
- Handoff: Remove PA-07 and record the exact merged commit for reporting.

## PA-08 — Publish and replay the single-project pilot

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-07
- Exact write set: project `22464` reporting include in `.gitlab-ci.yml` and protected schedule, generic package `acorn-automated-artifacts`, callback flow
- Primary integration lock: scheduled artifact publication
- Outcome: Pin the pilot's existing shared reporting include to a verified immutable template revision, configure its protected schedule, publish one immutable report package, deliver `artifact-ready`, and prove replay creates no duplicate effect. The template itself is an external read dependency; an unexpected template defect needs a separate bounded repair phase.
- Focused validation: Run `acorn report --dry-run` at the exact merged commit and cutoff; validate the pinned shared CI include, then inspect manifest, PDF, checksum, callback via `acorn intake deliver`, replay disposition, and package count.
- Handoff: Remove PA-08 and preserve the redacted single-project acceptance bundle.

## PA-09 — Validate routing and retire the JSONL plan

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-08
- Exact write set: isolated `.acorn.json`, canvas project choices, `.wiki/assets/powerautomate-logbook-reporting.md`, `.wiki/features/powerapps-gitlab-logbook-reporting.md`, `.wiki/index.md`, `.wiki/features/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: Power Automate operational record
- Outcome: Prove two authorized project routes and one unauthorized rejection, publish the durable runbook, and remove this plan.
- Focused validation: Verify isolated file changes, rejected unauthorized work, internal links, and wiki validation.
- Handoff: Delete this plan and all index, network, and lane entries.
