---
type: Plan
title: "Scheduled GitLab and SharePoint sync release"
description: "Bind and accept the hardened catalog and approved-snapshot flows in isolation, then activate an owned and monitored deployment."
difficulty: high
impact: high
package: [power-platform, sharepoint, gitlab]
tags: [powerautomate, schedule, sync, approval, deployment]
status: waiting-on-validation
execution: phased
phase_budget_hours: 5
---

# Outcome and current evidence

Release the two owner-run scheduled flows: GitLab project choices -> SharePoint
catalog, and approval-receipted SharePoint JSONL -> dedicated GitLab approved
snapshot. The second flow never feeds the Vergil inbox, modifies review state,
or finalizes a batch. This is an explicit approved-export mirror, not automatic
intake/graduation. It does not gate the SharePoint-only pilot release.

The November 24 review at `c6b70b1e` plus local hardening inspected the retained
builders, connector contract, and C# fallback labels. The [sources and runbook](../_assets/sharepoint/gitlab-sync/README.md)
now provide strict config validation, complete catalog preflight, uniqueness and
ETag safeguards, exact readback after uncertain writes, stable/fresh approved
source capture, prefix-preserving GitLab replacement, and serialized schedule
templates. Static checks and 18 synthetic graph-execution tests passed. These
are offline results; no hosted flow, connector, target, or schedule was changed.

The existing configuration is historical and all readiness flags remain false.
Generated connection references are unbound. The last recorded live beta flows
were Off; that state has not been freshly verified. Their native deployments
have not received the hardened sources. No new SharePoint/GitLab target is
selected or authorized by this plan. The [sanitized-copy limits](../../../privacy.md)
apply to retained packages before reuse.

# Readiness and boundaries

Before an external phase starts, an owner must identify its exact isolated
source/destination IDs, connections, permissions, and disposable fixtures.
External waiting is a prerequisite, not phase time. Both schedulers and all
readiness flags stay disabled outside an approved test/change window.

The catalog requires a unique indexed ProjectId field and a single active writer;
no phase silently provisions a site/repository or broadens access. The snapshot
requires an existing dedicated `.acorn/approved/*.jsonl` file with an approved
empty or exact-prefix baseline, plus an accepted exporter receipt contract.
Initial file creation is a separately approved target setup action. The 48-hour
receipt-age default and 03:00/03:30 Eastern template times need deployment-owner
acceptance. No notifications or new alert destinations are inferred.

SGS-01 and SGS-02 have independent data paths but serialize on the deployment
lock and the exact shared connector resources. Production Sponsor Highlights
publication additionally waits for PA-41A. GitLab fallback PA-15/16/17 and Vergil
acceptance remain independent; success here grants neither inbox admission nor
report publication authority.

# Acceptance criteria

- Hosted import/binding and intended-owner execution match the reviewed generated definitions; readiness flags, connections, permissions, and single-writer state are independently inspected.
- Catalog negatives fail before writes; normal and uncertain outcomes converge without duplicate IDs, wildcard overwrites, General-row changes, or loss of existing display titles.
- Approved snapshot writes require a fresh receipt and stable source identity/revision, preserve prior complete-line history, and reconcile exact bytes after real connector uncertainty.
- Full source/destination permissions, continuation links, maximum-size responses, path encoding, ETag fields, timeouts, and lost responses are tested in the actual connector environment.
- A production smoke, unchanged replay, scheduled run, failure/missed-run detection, native restore inspection, and owner handoff have durable evidence. Disable-only rollback preserves verified data.

# Execution phases

## SGS-01 — Accept the catalog sync in an isolated target

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: owner-approved isolated GitLab project/ref and SharePoint catalog list; protected connector bindings; unique indexed ProjectId field; synthetic test identities/data; fresh native backup; no other writer; target selection is intentionally unresolved.
- Exact write set: an approved isolated copy of `[BETA] GitLab Project Catalog Sync` and its catalog list/connection bindings named in the acceptance manifest; `../_assets/sharepoint/gitlab-sync/catalog-acceptance.md`; this plan and affected indexes. The historical flow ID is `031af465-9b60-4e7b-7960-028505194a42`, not implicit authorization to modify it.
- Primary integration lock: Power Platform deployment
- Outcome: Import/rebind the hardened catalog scope and serialized trigger, then demonstrate safe catalog reconciliation under the intended identity.
- Focused validation: Complete/empty/over-limit/continued source; case-insensitive duplicates; reserved General; removal; human title preservation; field uniqueness; two-writer create and ETag conflicts; committed timeout/readback; unreadable readback; partial-run replay; denied role; saved native source comparison and disabled final state.
- Handoff: Record exact target/flow/connector versions, schema, permissions, run IDs, rows/ETags, fault methods, native backup, and disposition. Any new defect gets a bounded repair; local simulations do not close this phase.

## SGS-02 — Accept approved snapshot mirroring in isolation

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: owner-approved isolated approval exporter/library and existing dedicated GitLab file/ref; verified empty or exact-prefix baseline; protected connections; reproducible source-change and connector-fault methods; fresh native backups; target selection is intentionally unresolved.
- Exact write set: an approved isolated copy of `[BETA] Approved JSONL GitLab Sync`, its approved-export test library/receipt fixtures, and dedicated GitLab test file named in the acceptance manifest; `../_assets/sharepoint/gitlab-sync/approved-acceptance.md`; this plan and affected indexes. The historical flow ID is `044451dc-7cdf-f111-84ea-001dd8062052`, not implicit authorization to modify it.
- Primary integration lock: Power Platform deployment
- Outcome: Establish hosted receipt/snapshot checks, conditional update, exact readback, and safe recovery without writing an intake inbox.
- Focused validation: Connector path encoding and metadata shapes; unchanged replay; 404/403/429; stale/future/malformed receipt; truncated listing; 10 MiB boundary; content/metadata race; missing revision; newer/divergent GitLab content; concurrent update; real write timeout/lost response with and without commit; readback failure; exact native restore and disabled final state.
- Handoff: Preserve exact bytes/revisions, receipt/run identities, permission evidence, fault methods, and before/after native definitions. Proxy faults or changed reporting windows cannot replace the specific service cases.

## SGS-03 — Activate and hand off owned scheduled synchronization

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: SGS-01 and SGS-02 accepted; PA-41A for Sponsor Highlights operational mirroring; named production targets and owners; accepted cadence/freshness policy; monitoring/support/rotation destination; explicit activation approval; fresh backups and a window with no in-flight old runs. Waiting for a scheduled observation is external time.
- Exact write set: approved catalog and approved-snapshot production flow copies, their bound connection references and triggers, and owner-approved monitoring configuration; `../_assets/sharepoint/gitlab-sync/production-acceptance.md`; `../_assets/sharepoint/gitlab-sync/README.md`; this plan and affected indexes.
- Primary integration lock: Power Platform deployment
- Outcome: Activate one writer per target with serialized runs, verified source authority, observable freshness/failures, and a disable-only rollback.
- Focused validation: Fresh native export and isolated import inspection; exact target and role allow/deny checks; no overlapping old writer; smoke and unchanged replay; one scheduled observation; failed/missed-run detection; connection-owner continuity; stale upstream export; run-history access/retention; disable and reconcile rollback rehearsal.
- Handoff: Publish target/version/owner matrix, baseline, run evidence, cadence, alert/rotation procedure, and recovery guide. Retire this plan only after all enabled paths have accepted external evidence.
