---
type: Plan
title: "Shared Power Platform intake deployment"
description: "Move the accepted approval-gated Sponsor Highlights solution from the owner pilot to a named shared environment, groups, connections, and notification destination."
difficulty: high
impact: critical
package: [power-platform, sharepoint]
tags: [powerapps, powerautomate, deployment, governance, sharepoint]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Deploy the accepted SharePoint contributor and approval apps, review ledger, approval-aware exporter, and notifications to an approved shared environment for the intended contributor/reviewer/operator audience. Preserve immutable receipts, the approval boundary, component consistency, least privilege, native recovery, and freeze-only rollback.

# Current evidence

The owner-context pilot is accepted for contributor intake, recovery, corrections, standard-license access, collected export, native backup, and local reporting. The new approval release is not yet implemented or accepted. No shared target environment, capacity allocation, maker/owner group, contributor group, reviewer/finalizer group, SharePoint site/library, connection owner, DLP decision, or Teams destination is recorded as approved.

Two-person pilot access does not establish organization-wide licensing, connector policy, capacity, or performance. A private site’s membership does not authorize raw receipt/review/export access for every app user.

# Target readiness contract

Before deployment, record:

- environment ID, region/cloud, Dataverse availability if required, capacity owner, DLP policy, and solution publisher;
- non-personal primary/secondary solution owners and connection-reference rotation procedure;
- contributor, reviewer, finalizer, operator, and support security groups with least-privilege matrix;
- SharePoint site/list/library IDs, internal column names, indexing, retention, versioning, audit, backup, and recovery owners;
- component-library sharing/update owner and adopted revision;
- Teams/email notification destinations and recipient ownership;
- expected contributor count, monthly receipt/batch volume, peak concurrency, 5,000-row/10 MiB collected-history limit, and monitoring thresholds;
- approved data classification, retention, incident, access-review, and offboarding rules.

If any prerequisite is unavailable, keep the owner pilot operational and the phase blocked. Do not silently choose a personal environment or broaden permissions.

# Migration contract

Migration uses a managed/native solution package and explicit environment variables/connection references. Import into the shared target without pointing at operational pilot storage. Provision target storage and permissions, import/update the component library, apps, and flows, then run synthetic acceptance. Only after target acceptance may future notifications and contributor entry links switch. Existing pilot receipts/history remain available under their recorded owner and are not copied or deleted unless a separately approved data-migration phase defines identity, duplicates, corrections, baseline, and retention.

The target approval exporter starts from an explicitly chosen baseline. If the target is a clean future-only deployment, baseline may be the verified accepted pilot collected history copied once with provenance. If history must remain separate, the target starts empty and guides state the reporting boundary. This decision is required before PA-12; do not infer it during import.

# Acceptance criteria

- Named target, groups, owners, licensing/capacity, DLP, storage, connections, notification destination, baseline policy, and rollback are approved.
- Intended contributors can use only contributor surfaces; reviewers/finalizers/operators have distinct tested access; unauthorized identities are denied.
- Both apps use the accepted component revision and retain usability/accessibility behavior.
- Submission, immutable receipt, correction, review, finalization, approved export, browser/Graph download, local stage/inspect/apply, and PDF report pass in the target.
- Retry/replay, competing review, exporter failure, monitoring, backup import, owner rotation, and freeze-only rollback pass.
- Cutover changes only future entry/notification links after acceptance; old receipts and destinations remain recoverable.

# Dependencies and boundaries

PA-12 waits for PA-41A. It also waits for every target-readiness decision above. GitLab fallback acceptance is not required for the SharePoint route. OAuth/CI downloader plans may continue independently; browser download remains supported. Vergil, Teams bot, sponsor publication, and editorial synthesis are separate deployments.

# Execution phases

## PA-12 — Provision and accept the shared target in isolation

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: PA-41A accepted; approved target environment/site/storage; capacity/licensing/DLP; solution owners and protected connections; contributor/reviewer/finalizer/operator groups; notification destination; baseline/history decision
- Exact write set: target Power Platform solution `Sponsor Highlights Shared`; target component library, contributor/approval apps, flows, lists, libraries, environment variables, connection references, and security assignments; `.wiki/features/powerapps-jsonl-intake/shared-deployment/index.md`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/shared-deployment/readiness.json`
- Primary integration lock: Power Platform shared deployment
- Outcome: Provision/import the accepted solution into the shared target with isolated storage and least privilege, then accept the complete synthetic workflow without changing pilot entry links or notifications.
- Focused validation: Import/package hashes; component revision; group allow/deny matrix; immutable receipt; correction; closed batch; competing reviewer; finalization; approved export; download; local PDF; notification test; monitoring; backup/restore; freeze rollback.
- Handoff: Preserve target IDs, owners, groups, connection references, baseline decision, runs, exact artifacts, and go/no-go for future-link cutover.

## PA-12A — Cut future use to the accepted shared deployment

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-12 accepted; explicit owner approval for future-link/notification cutover; fresh pilot and target backups; support window
- Exact write set: Sponsor Highlights entry hub; approved contributor/reviewer links; future notification destinations; target monitoring/operations guide; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/shared-deployment/cutover.md`
- Primary integration lock: Power Platform shared deployment
- Outcome: Switch future users and notifications to the accepted shared target while preserving pilot evidence, receipts, recovery, and a documented return path.
- Focused validation: Intended-user link/access checks, one live target receipt through Approved export/PDF, no new pilot receipt, correct notification audience, freshness dashboard, owner/support handoff, and rollback rehearsal.
- Handoff: Preserve final destination IDs, effective time, owner matrix, backups, monitoring, rollback, and durable user/operator documentation; then retire this plan when acceptance evidence is durable.
