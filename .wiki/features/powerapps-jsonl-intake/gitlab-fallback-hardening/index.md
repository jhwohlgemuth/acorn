---
type: Plan
title: "Retained GitLab intake fallback hardening"
description: "Finish isolated boundary, concurrency, and native-restore acceptance for the retained Off GitLab Power Platform route without disturbing the SharePoint default."
difficulty: high
impact: medium
package: [power-platform, gitlab]
tags: [powerapps, powerautomate, gitlab, fallback, concurrency, restore]
status: waiting-on-validation
execution: phased
phase_budget_hours: 5
---

# Goal

Establish whether the retained GitLab intake route can be safely restored as a fallback. Recreate only disposable TEST ONLY copies from accepted backups, complete the remaining byte-boundary and concurrent-write matrix, and prove the native solution can be imported with its required callback/table capability. Keep operational GitLab flows Off throughout this plan.

# Current evidence

Field limits, unsafe-tail rejection, Jason mixed-line preservation, and ORNL oversized-file rejection have partial accepted evidence. Remaining PA-15 cases are ORNL mixed-line preservation, exact-cap append in both environments, and oversized-file rejection in Jason. Earlier Jason exact-cap/oversize attempts were blocked by intermittent connector/file-read failures. The old PA-15 test flow was removed November 5 and must be recreated from retained evidence.

Append/creation concurrency logic is implemented and locally exercised, but live isolated simultaneous-writer acceptance remains. Native v6 app/connector/reference import and catalog behavior are accepted. Remaining PA-17 work is an app-driven GitLab write plus callback/table restoration in an authorized disposable target; the ORNL target previously lacked `prvCreateEntity`.

# Boundaries

- Do not edit, publish, enable, rebind, import over, or write through operational GitLab or SharePoint components.
- Use explicitly named TEST ONLY apps, flows, repositories/paths, connections, lists/tables, and browser sessions.
- Inspect every destination before a write and stop on mismatch.
- Preserve original pilot inbox bytes and revisions. Faults use isolated files only.
- Do not treat intermittent connector denial as acceptance; record the blocker and retain exact evidence.
- Passing this plan does not authorize restoration. A later explicit route decision must compare the accepted SharePoint release with the fallback’s licensing, ownership, and support cost.

# Acceptance criteria

- All remaining PA-15 cases prove accepted append or rejection with independent before/after bytes and revisions.
- PA-16 proves competing creators/writers preserve all distinct lines, deduplicate exact retry, and reconcile conditional-write failure without overwrite.
- PA-17 proves import, connection-reference rebinding, app-driven write, catalog behavior, callback/table restoration, and secret-free package inspection in an authorized disposable target.
- Test artifacts are inventoried and cleaned without deleting retained backups/evidence.
- Operational GitLab flows remain Off and SharePoint remains unchanged.

# Execution phases

## PA-15 — Accept bounded inbox and field limits

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: safe isolated fixtures; fresh isolated Jason and ORNL flow copies recreated from retained evidence; authorized connector access
- Exact write set: pilot project `22464` isolated PA-15 test files; isolated intake flow copies `ACORN PA15 Jason TEST ONLY` and `ACORN PA15 ORNL TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/boundary-acceptance.md`; `.wiki/features/powerapps-jsonl-intake/_assets/boundary-tests/README.md`
- Primary integration lock: GitLab intake route
- Outcome: Complete ORNL mixed-line preservation, exact-cap append in both environments, and one-byte-over-cap rejection in Jason while retaining already accepted cases.
- Focused validation: One dimension per case; independent GitLab before/after bytes, file revision, flow run, and app response; accepted records append once, rejected records change no bytes, and unsafe tails fail closed.
- Handoff: Record per-case outcomes and restore isolated fixtures. No implied concurrency acceptance.

## PA-16 — Accept concurrent inbox creation and append

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: PA-15; approved isolated files; fresh isolated flow copies; usable authorized connectivity
- Exact write set: pilot project `22464` isolated PA-16 concurrency files; isolated flows `ACORN PA16 Jason TEST ONLY` and `ACORN PA16 ORNL TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/concurrency-acceptance.md`
- Primary integration lock: GitLab intake route
- Outcome: Prove simultaneous creators/writers cannot overwrite distinct lines or duplicate the same submission.
- Focused validation: Competing create, competing append, exact retry, conditional-write failure, lost response, reconciliation, final bytes/revision, and immutable receipt identities. No Vergil cleanup runs concurrently.
- Handoff: Preserve concurrency evidence and return all test flows/files to the recorded disposition.

## PA-17 — Accept native GitLab Power Platform restore

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: fresh retained-backup import into an authorized disposable target; permitted GitLab test writes; capability to create the callback table or an approved revised restore target
- Exact write set: isolated solution `ACORN PA17 Restore TEST ONLY`; isolated cloned components in Wohlgemuth, Jason and the approved disposable target; `.wiki/features/powerapps-jsonl-intake/_assets/solution-export/PA17-RESTORE-HANDOFF.md`; `.wiki/features/powerapps-jsonl-intake/restore-target-readiness.md`
- Primary integration lock: Power Platform packaging
- Outcome: Prove secret-free native restore, connection rebinding, app-driven GitLab write, catalog response, and callback/table restoration from the retained package.
- Focused validation: Package/component inventory and hashes; import logs; environment-variable/connection references; absence of secrets; isolated app submission; exact GitLab bytes; callback/table create/read; no operational component changes.
- Handoff: Record restore result, target capabilities, hashes, cleanup, and remaining blockers. A source ZIP or partial import alone does not satisfy the phase.
