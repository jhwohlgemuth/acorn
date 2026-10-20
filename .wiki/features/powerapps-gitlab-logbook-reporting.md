---
type: Plan
title: "Route local and GitLab JSONL into project logbooks"
description: "Deliver a manual multi-project JSONL intake MVP, reuse its admission contract in the GitLab bot, and validate Power Automate reporting."
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

Deliver a command-driven local JSONL path that routes one file into a collection of project logbooks. Use it as the MVP and operator fallback for the GitLab bot. Both paths must share submission validation, canonical project selection, authorization, and normalized logbook entries. Then validate the reviewed GitLab path from the Power Apps canvas through Vergil admission, human merge, scheduled artifact publication, and Teams/email callbacks.

# Current evidence

- The ACRE 2.0.0 configuration and strict form envelope are the accepted contract in `ACRE_SPEC.md` and `docs/src/concepts/acre.md`. Project routing uses a configured canonical folder ID or alias; destination and digest are fixed at admission.
- The GitLab bot now snapshots and durably admits JSONL before conflict-safe cleanup, routes mixed-project records to their selected indexes, and persists review-event cursors across pagination and restarts. Local tests cover replays, concurrent inbox edits, rejection isolation, and routed review publication.
- `acorn intake run`, `export`, and `deliver` provide one-shot polling, reviewed diffs, and explicit callback delivery. Standalone `stage/apply` currently handles one local proposal per UUID. Each staged line is projected against its destination's current index, so two lines for the same project share a base digest and the second becomes stale after the first apply. There is no batch command for a mixed-project file. A scratch CLI run verified one local JSONL apply through updated `index.json`, PDF, and checksum manifest; it did not establish batch or live Power Automate/GitLab behavior.
- GitLab admission canonicalizes a folder alias before constructing its entry. Local staging uses the same `Submission::entry` validation but retains the input selector in the entry and rejects every bucket-associated folder. The manual MVP needs a shared canonical admission result and a local collection contract; transport-specific destination and review policy remain separate.
- The published tester canvas sent Teams and email notifications through its attached notification-only flow. The separate GitLab JSONL intake draft is off and unattached. No tested submission was persisted to CSV or JSONL by Power Automate. The pilot flow and service still require PA-02 through PA-09.
- A read-only gate audit on 2026-10-17 found project `22464` suitable for an isolated pilot branch, but did not establish service credentials, cloud-to-GitLab reachability, callback policy, or an approved shared GCC environment. The PA-02 handoff below records the remaining decisions.

# Selected transport and boundaries

- The manual MVP accepts one bounded LF-terminated JSONL snapshot using the existing strict `FormSubmission` envelope. A local ACRE profile maps canonical folder IDs and aliases to existing writable `file:` project indexes, either through `options.folders.root` or explicit paths. It may point at a checked-out project collection. No GitLab credential, bot, Power Automate flow, or network access is required for local intake. The first MVP supports standalone local folders; project creation, bucket copying, and remote destinations are separate workflows.
- One shared, side-effect-free admission step parses and validates each line, resolves the canonical folder, checks submitter access through a caller-supplied variable lookup and the applicable cutoff, and produces the normalized `LogbookEntry` plus the existing `powerautomate:form:v1:<uuid>` operation identity. The GitLab bot and local batch planner consume that result; both retain the exact raw bytes and line digest for replay checks. GitLab still fixes repository revision, branch, and queue identity at durable admission; local intake still requires explicit review and apply.
- Reuse the existing bounded `split_snapshot` JSONL framing and `LogbookProposal`/`ProcessRepositoryResponse` index transformation in both paths. The batch planner folds repeated entries by passing each proposed project content into the next transformation. Keep GitLab API/queue effects and local filesystem/receipt effects in their respective adapters; do not add a second form parser or logbook mutation algorithm.
- Local staging groups accepted entries by canonical destination and folds entries for the same project in physical JSONL order. An identical UUID with identical raw bytes contributes one entry. A UUID already applied through a local batch or existing single-proposal command is skipped when its raw digest agrees; changed bytes under that UUID reject the batch. A stable batch ID derives from the exact source snapshot and resolved configuration digest. Its owner-readable manifest at `<config-directory>/.acorn/intake/batches/<batch-id>.json` records every line and UUID, per-project base/final digests, and proposed content. Report every rejected or conflicting line, but make the whole batch non-applyable; staging never changes a canonical index or drains the source file. CLI summaries and errors omit submission text and secrets.
- Local apply revalidates every route, authorization, source identity, and base digest before its first write. It writes one durable project index at a time and records per-project outcomes. A crash or later write failure may leave a partial batch; `inspect` reports that state and repeating apply resumes only destinations still at their staged base or final digest. An unrelated index change fails closed. Only one writer may own a destination during apply; cross-file crash atomicity is not claimed.
- The app-triggered flow appends one UTF-8 JSONL `FormSubmission` per line to `.acorn/intake/logbook.jsonl` on one approved GitLab branch. It returns a queued receipt with stable submission ID and confirmed commit SHA; ACORN admission and publication have separate receipts.
- Vergil polls an explicitly configured repository file and review-event pages, keeps durable snapshots and operation IDs, removes only admitted bytes, and reports failures without losing rejected or unadmitted records. Existing HTTP webhook support remains independent of this route.
- The bot prepares a project-specific review branch and draft merge request. A person approves and merges; the bot does not merge its own changes. The scheduled job publishes an immutable PDF and manifest package, then an explicit callback command delivers metadata to a protected Power Automate flow.
- Credentials live in approved protected stores and never in source, ordinary run history, app controls, logs, or artifacts. There is no gateway or public Vergil endpoint in the selected GitLab-mediated path. If the tenant rejects the legacy HTTP callback trigger mode, PA-02 records an approved intermediary before PA-04 begins.

# Dependencies and acceptance

PA-10 through PA-18 deliver the local multi-project MVP and shared admission contract without PA-02's external approvals; PA-16 is its final acceptance gate. PA-05 waits for that gate before connecting the live GitLab JSONL flow. Independently, PA-02 resolves owner, environment, licensing, branch, principal, callback, and secret-store decisions. PA-04 then provisions protected bot and callback credentials; PA-03 needs those credentials to prove an authenticated Vergil GitLab poll. PA-06 through PA-09 use the isolated pilot and preserve exact project identity. PA-09 waits for both local and GitLab acceptance before retirement. The [runner-pool plan](../refactors/ci-runner-pools.md) owns reporting worker isolation, and the [database artifact plan](database-backed-artifact-generation.md) owns general database-backed rendering.

Local MVP acceptance requires one JSONL file with multiple lines for one project and lines for at least two other projects to stage as one reviewable batch, then update exactly those local `index.json` logbooks through ACORN commands. Alias and canonical selectors produce the same stored project ID. Invalid, unauthorized, duplicate-conflicting, stale, or remote routes cause no first write; replay and interrupted apply report accurate per-project outcomes without duplicate entries. The GitLab bot and manual path must produce the same canonical entry from the same form line. GitLab acceptance additionally requires one authorized submission to produce one correctly routed review operation and human-merged entry, one immutable report package, one delivered notification event, and no duplicate effects after replay or restart. Two authorized project selections must affect only their configured indexes; an unauthorized selection must affect none. The redacted evidence bundle must identify exact flow, service, commit, merge request, package, callback, and replay outcomes. On completion, move the durable runbook to `.wiki/assets/powerautomate-logbook-reporting.md` or user documentation and retire this plan from every portfolio surface.

# Validation

Use the focused validation in each remaining phase. For the local MVP, verify mixed-project and repeated-project routing, preview/apply separation, exact destination changes, conflicts, and interrupted replay offline. Inspect protected credentials and run-history settings without recording secret values. Verify live GitLab append/cleanup concurrency, restart behavior, human review, artifact checksum, callback delivery, and duplicate-safe replay before widening the pilot. Run the wiki validator after every phase is pruned.

# Manual operator commands (planned PA-14/15 surface)

Use a local ACRE profile with the same canonical folder IDs, aliases, and member rules as the GitLab profile, but with local `file:` destinations. Opt-in `stage --batch` prints a stable batch ID and leaves the input and project indexes untouched; ordinary single-proposal staging retains its current behavior. Review the per-project routes, entries, and proposed diffs from `inspect` before applying that exact batch; the existing `apply --submission-id` command remains available.

For example, a local `options.folders.root` of `file:///srv/acorn/projects/` with folders `alpha` and `beta` routes their records to the existing `/srv/acorn/projects/alpha/index.json` and `/srv/acorn/projects/beta/index.json`. `projectId` may use a configured alias, but the stored entry uses the canonical folder ID.

```sh
acorn intake stage --batch --config "$LOCAL_ACRE" --inbox "$JSONL_FILE"
acorn intake inspect --config "$LOCAL_ACRE" --batch-id "$BATCH_ID"
acorn intake apply --config "$LOCAL_ACRE" --batch-id "$BATCH_ID"
acorn intake inspect --config "$LOCAL_ACRE" --batch-id "$BATCH_ID"
```

For fallback on a GitLab-sourced inbox, stop the bot's intake worker first and use an exact snapshot as `JSONL_FILE`. Keep the local receipt, source bytes, and reviewed index changes together. Before the bot resumes against that inbox, reconcile the local changes into the approved GitLab repository and remove or mark only the exact handled lines through its reviewed process. The manual commands do not silently update GitLab, clear its inbox, or share the bot's operation database; the runbook must require this handoff so one UUID is not handled twice across transports.

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

## PA-10 — Freeze shared form admission

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-lib/src/io/api/powerautomate/form.rs`, `crates/acorn-lib/src/io/api/powerautomate/tests/mod.rs`, `.wiki/features/powerapps-gitlab-logbook-reporting.md`
- Primary integration lock: Power Automate submission contract
- Outcome: Expose one pure admission result with canonical folder ID, authorized normalized entry, existing stable operation key, caller-supplied variable lookup, and optional report-cutoff policy; retain raw JSONL bytes and their digest at the transport boundary.
- Focused validation: Run focused form tests for canonical/alias equivalence, access denial, invalid fields, cutoff, and unchanged operation identity.
- Handoff: Remove PA-10 after preserving the contract; PA-11 and PA-12 consume it without duplicating validation.

## PA-11 — Reuse shared admission in GitLab JSONL

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-10
- Exact write set: `crates/acorn-lib/src/io/api/powerautomate/jsonl.rs`, `crates/acorn-lib/src/io/api/gitlab/tests/jsonl.rs`, `.wiki/features/powerapps-gitlab-logbook-reporting.md`
- Primary integration lock: GitLab JSONL admission
- Outcome: Replace the bot's separate canonicalization and entry construction with the shared admission result while retaining exact-byte replay, fixed GitLab destination/branch, and durable queue behavior.
- Focused validation: Run focused GitLab JSONL tests for alias routing, cutoff, changed bytes under one UUID, rejected lines, admission replay, and unchanged review operations.
- Handoff: Remove PA-11 after preserving parity evidence; PA-16 joins it with the independently built local batch path.

## PA-12 — Stage one routed local JSONL batch

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-10
- Exact write set: `crates/acorn-lib/src/io/api/powerautomate/local.rs`, `crates/acorn-lib/src/io/api/powerautomate/local/batch.rs`, `crates/acorn-lib/src/io/api/powerautomate/tests/local.rs`, `.wiki/features/powerapps-gitlab-logbook-reporting.md`
- Primary integration lock: Local JSONL batch planner
- Outcome: In the opt-in batch path, reuse the bounded JSONL splitter and shared logbook proposal transformation to stage one entry per canonical local destination. Reject an applyable batch if any line fails and write a durable manifest and preview without changing source or indexes; preserve ordinary single-proposal staging.
- Focused validation: Test three distinct projects, aliases, unauthorized/remote/bucket routes, invalid lines, input limits, owner-readable manifest permissions, redacted summaries, and zero canonical writes during stage.
- Handoff: Remove PA-12 after preserving the manifest contract; PA-18 extends it to repeated-project and replay cases.

## PA-18 — Fold repeated-project entries and replay identities

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-12
- Exact write set: `crates/acorn-lib/src/io/api/powerautomate/local/batch.rs`, `crates/acorn-lib/src/io/api/powerautomate/tests/local.rs`, `.wiki/features/powerapps-gitlab-logbook-reporting.md`
- Primary integration lock: Local JSONL batch planner
- Outcome: Fold multiple lines for one destination in physical source order, count an identical UUID/raw line once, skip previously applied local UUIDs with the same digest, and reject changed bytes under a reused UUID.
- Focused validation: Test two entries for one project, interleaved projects, duplicates within/across batches and the single-proposal path, conflicts, exact final content, and no canonical writes during staging.
- Handoff: Remove PA-18 after preserving fold and replay evidence; PA-13 applies exactly the staged per-project results.

## PA-13 — Apply a reviewed local batch

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-18
- Exact write set: `crates/acorn-lib/src/io/api/powerautomate/local.rs`, `crates/acorn-lib/src/io/api/powerautomate/local/batch.rs`, `crates/acorn-lib/src/io/api/powerautomate/tests/local.rs`, `.wiki/features/powerapps-gitlab-logbook-reporting.md`
- Primary integration lock: Local JSONL batch application
- Outcome: Revalidate every route and base before writing, durably replace each project index, and persist per-project applied/partial status. Preserve the existing single-UUID apply path.
- Focused validation: Test no-write preflight failures, stale and changed routes, symlink/destination changes, per-project receipts, and exact final content.
- Handoff: Remove PA-13 after preserving the write contract; PA-17 proves interruption recovery before the CLI exposes batch apply.

## PA-17 — Resume interrupted local batch application

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-13
- Exact write set: `crates/acorn-lib/src/io/api/powerautomate/local/batch.rs`, `crates/acorn-lib/src/io/api/powerautomate/tests/local.rs`, `.wiki/features/powerapps-gitlab-logbook-reporting.md`
- Primary integration lock: Local JSONL batch application
- Outcome: Reopen a partial receipt, recognize destinations already at the exact staged final digest, apply only those still at their base digest, and fail closed on unrelated edits or changed routes.
- Focused validation: Simulate interruption after one project, repeated apply, changed second index, altered receipt, and no duplicate entries or false complete status.
- Handoff: Remove PA-17 after preserving restart evidence; PA-14 may expose batch apply.

## PA-14 — Expose batch stage and apply commands

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-17
- Exact write set: `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/commands/intake/mod.rs`, `crates/acorn-cli/src/commands/intake/tests.rs`, `CommandLineHelp.md`, `.wiki/features/powerapps-gitlab-logbook-reporting.md`
- Primary integration lock: CLI contract
- Outcome: Add `intake stage --batch` to report a stable batch ID and make `intake apply --batch-id` mutually exclusive with existing `--submission-id`, with machine-readable per-project outcomes and a nonzero exit for rejected or partial work. Preserve ordinary `intake stage` behavior.
- Focused validation: Run focused parser and command tests for single-UUID compatibility, batch selection, denied implicit apply, no network requirement, exit behavior, and generated help.
- Handoff: Remove PA-14 after preserving public syntax; PA-15 adds read-only review and guidance.

## PA-15 — Expose batch inspection and local guidance

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-14
- Exact write set: `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/commands/intake/mod.rs`, `crates/acorn-cli/src/commands/intake/tests.rs`, `CommandLineHelp.md`, `ACRE_SPEC.md`, `docs/src/concepts/acre.md`, `docs/src/cli-reference/commands/intake.md`, `.wiki/features/powerapps-gitlab-logbook-reporting.md`
- Primary integration lock: CLI contract
- Outcome: Add read-only `intake inspect --batch-id` showing source identity, each line/UUID, canonical project, destination, proposed diff, and applied/partial state; document the four-command local workflow and its GitLab fallback handoff.
- Focused validation: Run focused inspect/parser tests, generated-help and documentation checks, and verify inspection never mutates source, proposals, or indexes.
- Handoff: Remove PA-15 after preserving the operator contract; PA-16 proves the complete local MVP.

## PA-16 — Prove the manual multi-project MVP

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-11, PA-15
- Exact write set: `tests/fixtures/automation/powerautomate/local-batch.jsonl`, `crates/acorn-lib/src/io/api/powerautomate/tests/local.rs`, `crates/acorn-lib/src/io/api/gitlab/tests/jsonl.rs`, `crates/acorn-cli/src/commands/intake/tests.rs`, `docs/src/cli-reference/commands/intake.md`, `.wiki/features/powerapps-gitlab-logbook-reporting.md`, `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: JSONL local acceptance
- Outcome: Demonstrate one offline file updating three local project logbooks, including two entries for one project, through stage/inspect/apply/inspect; prove the same raw forms normalize identically for GitLab and local paths and document the safe bot handoff.
- Focused validation: Run the command sequence on disposable indexes, compare exact changed paths and entries, replay and interrupted-apply receipts, unauthorized no-write behavior, focused GitLab parity tests, and wiki validation.
- Handoff: Remove PA-16 after preserving the local MVP evidence; satisfy PA-05's local prerequisite while leaving its external deployment gates intact.

## PA-02 — Resolve production owners and access gates

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: existing policy and owner records or available authorized decision makers; the notification-only and local artifact pilots are verified
- Exact write set: Power Platform environment policy, isolated GitLab project `22464` access policy, `.wiki/features/powerapps-gitlab-logbook-reporting.md`
- Primary integration lock: deployment approvals
- Outcome: Name owners and record approved environment, GitLab API route for the future app-triggered flow, callback-trigger, licensing, branch permissions, and secret-storage decisions. No gateway approval is required.
- Focused validation: Verify every stop gate has an accountable owner and dated evidence.
- Handoff: Remove PA-02 and record the approved GitLab target and outbound access requirements for PA-04, then PA-03.

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
- Prerequisites: PA-02
- Exact write set: selected intake flow's protected GitLab credential, Vergil's protected GitLab credential and runtime environment, protected `POWER_AUTOMATE_CALLBACK_URL`, callback flow
- Primary integration lock: Power Automate credentials
- Outcome: Establish separate protected GitLab credentials for the flow and bot, plus one schema-valid callback flow. Keep any startup-required `POWER_AUTOMATE_INBOUND_SECRET` local to Vergil; the JSONL flow does not use it.
- Focused validation: Send a redacted test callback and inspect 2xx, Teams, email, and secure run-history behavior.
- Handoff: Remove PA-04 and provide only secret references to PA-03 and PA-05.

## PA-05 — Connect the GitLab JSONL intake flow

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: PA-03, PA-04, PA-16 (source configuration schema, snapshot reader, durable admission, cleanup, poller, and mixed-project publication code are landed)
- Exact write set: selected Power Apps intake flow, canvas submit action, isolated project `22464` inbox `.acorn/intake/logbook.jsonl`, Vergil JSONL source configuration
- Primary integration lock: intake route
- Outcome: Append the strict version-1 envelope with conflict-safe GitLab retries, configure the implemented repository-file poller, and return a confirmed queued receipt with submission ID and commit SHA.
- Focused validation: With the Vergil service stopped, run one authorized `acorn intake run --config "$ACORN_CONFIG"`; verify stable UUID/content across retries, concurrent append and cleanup, ambiguous responses, serialization round-trips, protected credentials, and app distinction between GitLab persistence and ACORN admission. Restart the service against the same database.
- Handoff: Remove PA-05 and record the tested flow revision, source project/branch/path, and poller revision.
- Draft checkpoint: Flow `6039f419-420f-6922-46ef-04b00d2290f1` is saved and off with typed trigger inputs plus one-time ID and timestamp actions. Resume with the approved branch and secret references from PA-02/04, service/callback readiness from PA-03/04, then implement GitLab read/conditional write/retry, secure run history, confirm commit persistence, return the queued receipt, bind the canvas, and configure the poller. Do not enable or attach the draft before those pieces work together.

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
- Exact write set: project `22464` schedule, generic package `acorn-automated-artifacts`, callback flow
- Primary integration lock: scheduled artifact publication
- Outcome: Publish one immutable report package, deliver `artifact-ready`, and prove replay creates no duplicate effect.
- Focused validation: Run `acorn report --dry-run` at the exact merged commit and cutoff; validate the pinned shared CI include, then inspect manifest, PDF, checksum, callback via `acorn intake deliver`, replay disposition, and package count.
- Handoff: Remove PA-08 and preserve the redacted single-project acceptance bundle.

## PA-09 — Validate routing and retire the JSONL plan

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-08, PA-16
- Exact write set: isolated `.acorn.json`, canvas project choices, `.wiki/assets/powerautomate-logbook-reporting.md`, `.wiki/features/powerapps-gitlab-logbook-reporting.md`, `.wiki/index.md`, `.wiki/features/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: Power Automate operational record
- Outcome: Prove two authorized project routes and one unauthorized rejection, publish the durable runbook, and remove this plan.
- Focused validation: Verify isolated file changes, rejected unauthorized work, internal links, and wiki validation.
- Handoff: Delete this plan and all index, network, and lane entries.
