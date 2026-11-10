---
type: Plan
title: "Approval-gated collected JSONL export"
description: "Preserve the pre-approval collected baseline and publish only deterministic frozen projections from complete Approved review batches."
difficulty: high
impact: critical
package: [power-platform, sharepoint]
tags: [powerautomate, sharepoint, jsonl, exporter, approval, rollback]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Change only the selection boundary of the accepted SharePoint collected exporter. At cutover, freeze the last verified pre-approval `logbook.jsonl` as an immutable baseline. Thereafter rebuild the collected file as `baseline + complete Approved batches`, never from mutable source receipts or review drafts. Preserve the existing exporter’s exact-byte, bounded-size, serialized-write, previous-read, uncertain-outcome, and readback protections.

# Current evidence

The repaired operational exporter is accepted for source-row aggregation. It captures a cutoff, validates rows, builds deterministic JSONL, requires a readable prior file or explicit 404, disables write retries, and reconciles uncertain writes by exact readback. Failure simulations cover unchanged output, confirmed missing creation, read failures, lost write responses, readback failures/mismatches, and non-overwrite creation. The operational route remains unfiltered until the approval cutover is explicitly accepted.

The existing collected file is retained history. It is not a queue, acknowledgement, or drain. Local application does not remove remote rows. The approval change must not remove any previously collected line, reserialize baseline bytes, or reinterpret earlier corrections.

# Export contract

## Baseline

The cutover baseline record contains the exact prior file bytes, SHA-256 digest, byte count, line count, SharePoint file identity/revision, captured source cutoff, capture instant, exporter revision, and approving operator. Store it read-only in a protected library. The baseline must end with exactly one newline unless it is empty; do not parse and rewrite it during normal publication.

If baseline capture or readback is uncertain, cutover stops. There is no approval-aware exporter activation without a verified baseline.

## Approved batch input

The exporter reads only batch manifests in `Approved` state. For each batch it verifies:

- period, approval instant, finalizer, item/approved/excluded counts, and projection digest;
- one unique batch ID and a deterministic batch order;
- exact frozen file count, names, bytes, per-file digests, and mapping digest;
- each line is one valid JSON object within the existing per-record bound;
- no duplicate approved submission ID across baseline-aware new output or approved batches;
- correction predecessors resolve in baseline or earlier/current approved order.

The exporter does not read `draftProjection`, infer approval from item decisions, or repair a manifest. A missing, corrupt, duplicated, or inconsistent batch is excluded from the candidate and raises a bounded failure; the prior collected file remains.

## Ordering and replay

Append approved batches in stable `(approvedAt, batchId)` order and items in the frozen manifest order derived from report/source ordering. The exact ordering rule is versioned before implementation. Repeated runs over the same baseline and Approved manifests produce byte-identical output. Adding one Approved batch preserves every prior byte and appends only that batch’s frozen lines.

## Bounds and publication

The candidate remains limited to 5,000 total lines and 10 MiB. Count and size include the baseline. Approaching either bound fails closed and requires a separate retention/scale decision; do not delete old rows or split history implicitly.

Publication retains serialized concurrency, read-before-write, compare-before-write, disabled connector retry, conditional creation, exact post-write readback, and a success receipt containing file revision, byte/line count, and digest. Unchanged candidates do not write.

# Cutover and rollback design

PA-38 builds and accepts the exporter only in TEST ONLY storage. PA-41 owns operational baseline capture and activation. Before activation, the current exporter remains unchanged. At activation, disable the old trigger, verify it cannot run concurrently, install the approval-aware flow, and publish only after baseline and Approved-batch checks pass.

Rollback disables the approval-aware trigger and freezes the last verified collected file. It does not enable the old unfiltered flow. Recovery may repair and redeploy the approval-aware flow from its package, but no flow may rebuild from unapproved source rows after the policy boundary has changed.

# Security and operations

The exporter identity reads the baseline, Approved manifests, and frozen projection files, and writes the collected file plus export receipt. It does not need contributor-app access or reviewer draft mutation. Reviewer identities cannot edit the baseline or collected file. Flow run history and failure messages contain IDs/digests/counts, not full sensitive payloads.

Operators monitor scheduled-run freshness, last Approved batch included, collected revision/digest, and failure code. A submission success or batch approval message does not claim that the collected file is fresh until an exporter success receipt includes that batch.

# Acceptance criteria

- The exact pre-approval baseline is preserved byte-for-byte at the start of every candidate.
- Pending, Ready, Finalizing, Failed, unresolved, and excluded records never appear.
- Every included post-cutover line is traceable to one Approved batch, source receipt, review decision, approval UUID/mapping, and frozen digest.
- Repeated runs are byte-identical and do not write unchanged content.
- One new Approved batch appends once in deterministic order without changing prior bytes.
- Count/size overflow, malformed JSON, duplicate IDs, broken correction ancestry, missing files, bad digests, read failure, concurrent writer, uncertain write, and readback mismatch preserve the prior collected file and do not emit success.
- Rollback freezes the last verified file and cannot re-enable the unfiltered exporter.

# Dependencies and boundaries

PA-38 requires accepted PA-37B frozen batches and a synthetic verified baseline. PA-38 does not activate production. PA-40 consumes the isolated exporter in end-to-end acceptance. PA-41 owns operational activation and backup. SharePoint CLI/manual downloads remain independent readers and do not trigger export.

# Execution phases

## PA-38 — Freeze the approved-export projection contract

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted [TEST ONLY finalization evidence](../_assets/sharepoint/intake-approval/finalization/finalization-acceptance.md); isolated export library; verified synthetic pre-approval baseline
- Exact write set: `.wiki/features/powerapps-jsonl-intake/approval-export/index.md`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/exporter/export-contract.json`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/exporter/baseline.json`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/exporter/fixtures.json`
- Primary integration lock: SharePoint collected exporter contract
- Outcome: Freeze baseline metadata, approved-manifest schema, deterministic batch/item ordering, duplicate/correction checks, limits, receipt fields, and rollback rule.
- Focused validation: Review against accepted exporter behavior, PA-37B manifests, JSONL/local replay requirements, existing 5,000-row/10 MiB bounds, correction ancestry, and exact-byte baseline preservation.
- Handoff: Publish versioned candidate/receipt contracts and fixtures before flow generation.

## PA-38A — Build the TEST ONLY approval-aware exporter

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-38
- Exact write set: `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/build-logbook-export.py`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/Combined_logbook_export_LB.clipboard.json`; TEST ONLY flow `Approval Gated Combined Logbook Export TEST ONLY`; TEST ONLY libraries `Sponsor Highlights Approved Projections TEST ONLY` and `Sponsor Highlights Collected Export TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/exporter/generated-definition.json`
- Primary integration lock: SharePoint collected exporter implementation
- Outcome: Generate and install a serialized TEST ONLY flow that constructs `baseline + Approved batches`, skips unchanged candidates, writes conditionally with retries disabled, rereads exactly, and emits a verifiable receipt.
- Focused validation: Static graph checks, formula/action-name checks, least-privilege binding inspection, candidate digest/count verification, unchanged/changed run, and exact readback receipt.
- Handoff: Preserve generated definition, native package, connection-reference map, and test storage IDs. Production remains untouched.

## PA-38B — Accept failure safety, replay, and rollback

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-38A
- Exact write set: `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/export-failure-tests/test_exporter.py`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/exporter/failure-acceptance.md`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/exporter/rollback.md`; TEST ONLY flow `Approval Gated Combined Logbook Export TEST ONLY`
- Primary integration lock: SharePoint exporter acceptance
- Outcome: Prove deterministic replay, append-only approved extension, failure preservation, observable freshness, and a freeze-only rollback in isolated storage.
- Focused validation: Non-Approved states, exclusions, duplicate/corrupt/missing manifests, bad JSON/digest/count/ancestry, 5,001 rows, 10 MiB overflow, prior-read 403/500/timeout, competing writer, lost write response with both commit outcomes, readback failure/mismatch, repeated unchanged run, and disabled-old-exporter rollback rehearsal.
- Handoff: Preserve run IDs, exact before/after bytes and digests, traces, native package, rollback evidence, and PA-40 readiness decision.
