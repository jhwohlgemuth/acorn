---
type: Plan
title: "Validate GitLab JSONL intake and deploy Power Automate logbook reporting"
description: "Validate mixed-project JSONL routing and deploy reviewed, duplicate-safe Power Automate logbook reporting."
difficulty: medium
impact: critical
package: [workspace, acorn-lib, acorn-cli, acorn-schema]
tags: [feature, automation, powerapps, powerautomate, gitlab, webhook, logbook, reporting, pdf]
status: in-progress
priority: next
execution: phased
phase_budget_hours: 5
---

# Goal

Deploy the reviewed GitLab JSONL Logbook path from the Power Apps canvas through Vergil admission, human merge, scheduled artifact publication, and Teams/email callbacks. The local implementation is complete enough for a live pilot; this plan contains the remaining owner, access, service, flow, and acceptance gates.

# Current evidence

- The ACRE 2.0.0 configuration and strict form envelope are the accepted contract in `ACRE_SPEC.md` and `docs/src/concepts/acre.md`. Project routing uses a configured canonical folder ID or alias; destination and digest are fixed at admission.
- The GitLab bot now snapshots and durably admits JSONL before conflict-safe cleanup, routes mixed-project records to their selected indexes, and persists review-event cursors across pagination and restarts. Local tests cover replays, concurrent inbox edits, rejection isolation, and routed review publication.
- `acorn intake run`, `export`, and `deliver` provide one-shot polling, reviewed diffs, and explicit callback delivery. Standalone `stage/apply` is a separate local-file proposal path. A scratch CLI run verified local JSONL apply through updated `index.json`, PDF, and checksum manifest; it did not establish live Power Automate or GitLab reachability.
- The published tester canvas sent Teams and email notifications through its attached notification-only flow. The separate GitLab JSONL intake draft is off and unattached. No tested submission was persisted to CSV or JSONL by Power Automate. The pilot flow and service still require PA-02 through PA-09.
- A read-only gate audit on 2026-10-17 found project `22464` suitable for an isolated pilot branch, but did not establish service credentials, cloud-to-GitLab reachability, callback policy, or an approved shared GCC environment. The PA-02 handoff below records the remaining decisions.

# Selected transport and boundaries

- The app-triggered flow appends one UTF-8 JSONL `FormSubmission` per line to `.acorn/intake/logbook.jsonl` on one approved GitLab branch. It returns a queued receipt with stable submission ID and confirmed commit SHA; ACORN admission and publication have separate receipts.
- Vergil polls an explicitly configured repository file and review-event pages, keeps durable snapshots and operation IDs, removes only admitted bytes, and reports failures without losing rejected or unadmitted records. Existing HTTP webhook support remains independent of this route.
- The bot prepares a project-specific review branch and draft merge request. A person approves and merges; the bot does not merge its own changes. The scheduled job publishes an immutable PDF and manifest package, then an explicit callback command delivers metadata to a protected Power Automate flow.
- Credentials live in approved protected stores and never in source, ordinary run history, app controls, logs, or artifacts. There is no gateway or public Vergil endpoint in the selected GitLab-mediated path. If the tenant rejects the legacy HTTP callback trigger mode, PA-02 records an approved intermediary before PA-04 begins.

# Dependencies and acceptance

PA-02 resolves owner, environment, licensing, branch, principal, callback, and secret-store decisions. PA-03 and PA-04 may then proceed independently; both gate PA-05. PA-06 through PA-09 use the isolated pilot and preserve exact project identity. The [runner-pool plan](../refactors/ci-runner-pools.md) owns reporting worker isolation, and the [database artifact plan](database-backed-artifact-generation.md) owns general database-backed rendering.

Acceptance requires one authorized submission to produce one correctly routed review operation and human-merged entry, one immutable report package, one delivered notification event, and no duplicate effects after replay or restart. Two authorized project selections must affect only their configured indexes; an unauthorized selection must affect none. The redacted evidence bundle must identify exact flow, service, commit, merge request, package, callback, and replay outcomes. On completion, move the durable runbook to `.wiki/assets/powerautomate-logbook-reporting.md` or user documentation and retire this plan from every portfolio surface.

# Validation

Use the focused validation in each remaining phase. Inspect protected credentials and run-history settings without recording secret values. Verify live GitLab append/cleanup concurrency, restart behavior, human review, artifact checksum, callback delivery, and duplicate-safe replay before widening the pilot. Run the wiki validator after every phase is pruned.

# Execution phases

## PA-02 — Resolve production owners and access gates

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: existing policy and owner records or available authorized decision makers; the notification-only and local artifact pilots are verified
- Exact write set: Power Platform environment policy, isolated GitLab project `22464` access policy, `.wiki/features/powerapps-gitlab-logbook-reporting.md`
- Primary integration lock: deployment approvals
- Outcome: Name owners and record approved environment, GitLab API route for the future app-triggered flow, callback-trigger, licensing, branch permissions, and secret-storage decisions. No gateway approval is required.
- Focused validation: Verify every stop gate has an accountable owner and dated evidence.
- Handoff: Remove PA-02 and record the approved GitLab target and outbound access requirements for PA-03 and PA-04.

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
- Prerequisites: PA-02
- Exact write set: Vergil managed service definition, protected Vergil runtime environment, `.wiki/features/powerapps-gitlab-logbook-reporting.md`
- Primary integration lock: Vergil service
- Outcome: Run the pinned native ARM64 ACORN service in polling mode with healthy readiness and redacted logs.
- Focused validation: Check `/health`, `/ready`, process supervision, architecture, one authenticated GitLab review-event read, and absence of secret values. Readiness alone does not establish GitLab or callback reachability.
- Handoff: Remove PA-03 and record the verified service revision and outbound GitLab access. Healthy note polling does not establish JSONL intake readiness.

## PA-04 — Provision GitLab and callback secrets

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-02
- Exact write set: selected intake flow's protected GitLab credential, Vergil's protected GitLab credential and runtime environment, protected `POWER_AUTOMATE_CALLBACK_URL`, callback flow
- Primary integration lock: Power Automate credentials
- Outcome: Establish separate protected GitLab credentials for the flow and bot, plus one schema-valid callback flow. Keep any startup-required `POWER_AUTOMATE_INBOUND_SECRET` local to Vergil; the JSONL flow does not use it.
- Focused validation: Send a redacted test callback and inspect 2xx, Teams, email, and secure run-history behavior.
- Handoff: Remove PA-04 and provide only secret references to PA-05.

## PA-05 — Connect the GitLab JSONL intake flow

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: PA-03, PA-04 (source configuration, snapshot reader, durable admission, cleanup, poller, and mixed-project publication are landed)
- Exact write set: selected Power Apps intake flow, canvas submit action, isolated project `22464` inbox `.acorn/intake/logbook.jsonl`, Vergil JSONL source configuration
- Primary integration lock: intake route
- Outcome: Append the strict version-1 envelope with conflict-safe GitLab retries, configure the implemented repository-file poller, and return a confirmed queued receipt with submission ID and commit SHA.
- Focused validation: Verify stable UUID/content across retries, concurrent append and cleanup, ambiguous responses, serialization round-trips, protected credentials, and app distinction between GitLab persistence and ACORN admission.
- Handoff: Remove PA-05 and record the tested flow revision, source project/branch/path, and poller revision.
- Draft checkpoint: Flow `6039f419-420f-6922-46ef-04b00d2290f1` is saved and off with typed trigger inputs plus one-time ID and timestamp actions. Resume with the approved branch and secret references from PA-02/04, service/callback readiness from PA-03/04, then implement GitLab read/conditional write/retry, secure run history, confirm commit persistence, return the queued receipt, bind the canvas, and configure the poller. Do not enable or attach the draft before those pieces work together.

## PA-06 — Create the first duplicate-safe review operation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-05
- Exact write set: isolated GitLab project `22464`, Power Automate intake run, `.wiki/features/powerapps-gitlab-logbook-reporting.md`
- Primary integration lock: isolated pilot repository
- Outcome: Submit one authorized response and create exactly one routed branch and draft merge request.
- Focused validation: Inspect the queued receipt, durable admission and operation ID, exact-record inbox removal, routed branch path, proposed entry, callback, and duplicate-safe persistence across restart.
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
- Focused validation: Validate the pinned shared CI include, then inspect manifest, PDF, checksum, cutoff, callback, replay disposition, and package count.
- Handoff: Remove PA-08 and preserve the redacted single-project acceptance bundle.

## PA-09 — Validate routing and retire the deployment plan

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-08
- Exact write set: isolated `.acorn.json`, canvas project choices, `.wiki/assets/powerautomate-logbook-reporting.md`, `.wiki/index.md`, `.wiki/features/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: Power Automate operational record
- Outcome: Prove two authorized project routes and one unauthorized rejection, publish the durable runbook, and remove this plan.
- Focused validation: Verify isolated file changes, rejected unauthorized work, internal links, and wiki validation.
- Handoff: Delete this plan and all index, network, and lane entries.
