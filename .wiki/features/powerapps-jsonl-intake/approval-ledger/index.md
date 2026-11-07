---
type: Plan
title: "Intake approval ledger and finalization"
description: "Materialize complete closed-period review batches, save concurrent reviewer drafts safely, and freeze correction-safe approved projections for publication."
difficulty: high
impact: critical
package: [power-platform, sharepoint]
tags: [powerautomate, sharepoint, approval, concurrency, audit, jsonl]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Add a separate, auditable review ledger between immutable contributor receipts and the collected exporter. Reviewers work on a stable closed-period snapshot, save structured edits with optimistic concurrency, resolve every item, and finalize an immutable set of approved JSONL projections. No reviewer action may rewrite source receipts or make a partial batch exporter-visible.

# Current evidence

The operational SharePoint source list and individual receipt files are immutable by UUID. The existing exporter reads source rows through a captured item-ID cutoff and preserves the prior collected file on read/write uncertainty. Those properties are accepted and remain the safety baseline.

Canvas Apps delegation is not sufficient for completeness-sensitive batch construction. Microsoft documents that nondelegable expressions process only a bounded local prefix. SharePoint `Get items` also has list-view and pagination constraints. Batch materialization and final completeness checks therefore belong in explicit Power Automate flows using indexed server-side filters, stable ordering, pagination, count reconciliation, and a captured high-water ID—not client-side `ClearCollect`, `GroupBy`, or nondelegable filters. See [Power Apps delegation](https://learn.microsoft.com/power-apps/maker/canvas-apps/delegation-overview) and [SharePoint Get items guidance](https://learn.microsoft.com/sharepoint/dev/business-apps/power-automate/guidance/working-with-get-items-and-get-files).

# Data model

## Review batch

One `Review Batch` row stores:

| Field | Contract |
|---|---|
| `batchId` | Stable GUID, unique and immutable. |
| `periodStartUtc` / `periodEndUtc` | Half-open `[start, end)` interval; end must be in the past before materialization. |
| `sourceHighWaterId` | Maximum source list item ID captured before enumeration. |
| `sourceCount` | Count of source rows in the interval through the high-water ID. |
| `state` | `Draft`, `Materializing`, `Ready`, `Finalizing`, `Approved`, or `Failed`; transitions are monotonic. |
| `createdBy`, `createdAt`, `materializedAt` | Audit identity and instants. |
| `itemCount`, `approvedCount`, `excludedCount`, `unresolvedCount` | Reconciled summary counts. |
| `projectionDigest` | Digest over deterministic frozen included bytes and mapping after finalization. |
| `approvedBy`, `approvedAt` | Final human authority and instant. |
| `failureCode` | Bounded diagnostic; no raw payload or secret. |

Create a unique key over the normalized period plus review-policy revision so exact retries return the same batch. Overlapping periods require an explicit operator decision and are rejected by default.

## Review item

One `Review Item` row stores:

| Field | Contract |
|---|---|
| `batchId` + `sourceItemId` | Unique source membership key. |
| `sourceSubmissionId` | Immutable contributor UUID. |
| `sourcePayload` / `sourceDigest` | Exact captured JSON and digest; never edited. |
| `sourceCreatedUtc` | Receipt `Created` instant used for period membership. |
| `sourceRevisionRoot` / `sourceSupersedes` | Immutable correction ancestry. |
| `draftProjection` | Structured reviewer draft; mutable only while batch is `Ready`. |
| `draftDigest`, `draftVersion`, `modifiedBy`, `modifiedAt` | Optimistic concurrency/audit fields. |
| `decision` | `Unresolved`, `Approve`, or `Exclude`. |
| `decisionReason` | Required for exclusion; optional bounded reviewer note for approval. |
| `approvedSubmissionId` | Source UUID for byte-identical content or stable synthetic UUID for changed content. |
| `approvedPayload` / `approvedDigest` | Frozen exact JSONL object and digest. |
| `approvedSupersedes` | Rewritten approved predecessor, when any. |
| `validationState` / `validationErrors` | Bounded machine-readable result used by the app. |

Store large/raw payloads in a private review library if list-column limits or run-history exposure make list storage unsafe; the schema contract must choose one representation before implementation. The app receives only fields required for the current page and never broad library access.

# State and authorization contract

Only a named reviewer group may list batches, read review items, or save drafts. Only a smaller finalizer group may request finalization. The flow verifies caller identity independently of hidden buttons. Contributors retain no review-list access. Exporter identity receives read access only to Approved batch manifests and frozen projection files; it cannot read mutable drafts unless required for reconciliation in TEST ONLY validation.

Allowed transitions are:

`Draft -> Materializing -> Ready -> Finalizing -> Approved`

Any uncertain materialization/finalization result becomes `Failed` or remains in its prior non-visible state with a diagnostic. There is no transition from `Approved` back to editable. A superseding batch is a new immutable batch, not a reopened one.

# Materialization algorithm

1. Validate authorization, UTC timestamps, `start < end`, and `end <= now`.
2. Create or resolve the idempotent batch key.
3. Capture the current maximum source item ID.
4. Enumerate source rows where `Created >= start`, `Created < end`, and `ID <= highWater`, ordered by ID, using indexed filters and explicit pagination.
5. Validate each exact source payload and digest; create review items by unique membership key.
6. Requery/count the same bounded source set and compare IDs, count, and digests with materialized items.
7. Mark `Ready` only when the complete set matches. Empty periods may become Ready with zero items, but finalization policy must explicitly permit or reject empty publication.

New source rows after the high-water ID are not silently added. If a row with a `Created` value inside the closed interval arrives later, finalization's complete-source recheck blocks the batch and requires a new materialization/superseding batch decision.

# Save and finalization contract

Draft saves send `batchId`, `sourceItemId`, expected SharePoint ETag/version, structured field values, decision, and reason. The save flow rereads batch/item state, validates authorization and schema, rejects stale ETags, computes the canonical draft/digest, writes once, then rereads the exact saved version. Autosave is debounced; the app shows `Saving`, `Saved`, `Conflict`, or `Failed` and never discards the local edit on a recoverable failure.

Finalization acquires a batch lease, rechecks the source interval/high-water set, validates every draft, and requires zero unresolved items. For each included item it deterministically assigns the approval UUID, rewrites correction ancestry through the source-to-approved map, serializes canonical JSON, stores frozen bytes, and rereads them. Changed content receives a UUID derived from stable batch/source/draft identity or a stored once-only generated UUID; retry returns the same ID. Excluding a root requires explicit treatment of descendants; an included correction cannot point to an excluded or unresolved predecessor.

Only after every included payload, mapping, count, and digest rereads exactly does the flow set the batch to `Approved`. That single state is the publication commit. The app cannot set it directly.

# Failure and concurrency cases

- Two reviewers saving the same item: one succeeds; the stale ETag receives a conflict with current version and no silent last-writer-wins overwrite.
- Duplicate materialization/finalization requests: return the existing state/result without duplicate rows or new UUIDs.
- Flow timeout or lost response: reconcile by rereading batch/item state and digest before retry.
- Source mutation: any digest change from the captured source is a terminal integrity failure; do not adopt new bytes silently.
- Late in-period source: block finalization and require a documented rematerialization decision.
- Invalid correction graph: block the affected item and batch; never publish a dangling `supersedesSubmissionId`.
- Reviewer loses group membership: the next flow call denies access; cached app data does not grant authority.
- Approved payload/library unavailable: batch remains non-visible or Failed; prior collected export is unaffected.

# Boundaries

This plan owns TEST ONLY review lists/libraries, review flows, app data bindings, concurrency, finalization, and frozen projections. It does not modify the collected exporter, production source list, contributor app, ACORN schema, or local report renderer. The exporter plan consumes only Approved manifests and frozen bytes.

# Acceptance criteria

- Materialization proves exact membership for empty, normal, paged, and maximum accepted batches without nondelegable client filtering.
- Unauthorized users cannot enumerate batch metadata, source payloads, drafts, or frozen projections.
- Source bytes and digests remain unchanged throughout review.
- Stale ETags, duplicate calls, timeouts, and competing reviewers cannot lose edits or produce silent overwrites.
- Every Approved batch has zero unresolved items, reconciled counts, complete mappings, exact frozen readback, and one finalizer identity.
- Changed content has stable approval UUIDs; unchanged exact content does not receive an unnecessary identity change.
- Correction ancestry is valid across same-batch and earlier-approved-batch predecessors; excluded ancestry cannot dangle.
- No state other than Approved is exporter-visible.

# Execution phases

## PA-36 — Freeze review storage, state, and permission contracts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-34; approved TEST ONLY SharePoint site/list/library destinations
- Exact write set: `.wiki/features/powerapps-jsonl-intake/approval-ledger/index.md`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/storage/schema.json`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/storage/permissions.md`; TEST ONLY lists `Sponsor Highlights Review Batches TEST ONLY` and `Sponsor Highlights Review Items TEST ONLY`; TEST ONLY library `Sponsor Highlights Approved Projections TEST ONLY`
- Primary integration lock: approval ledger schema
- Outcome: Freeze columns, internal names, indexes, unique keys, state transitions, retention, payload location, reviewer/finalizer/exporter permissions, and audit fields.
- Focused validation: Schema review against source payload sizes, SharePoint limits, least privilege, correction fields, app paging needs, and exporter consumption; prove contributors and unrelated site members cannot read review data.
- Handoff: Preserve exported schemas, permissions, IDs, and a versioned state diagram. PA-36A and PA-35 may proceed independently.

## PA-36A — Materialize complete closed-period batches

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-36
- Exact write set: TEST ONLY flows `Create Approval Batch TEST ONLY` and `Materialize Approval Batch TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/storage/materialization.json`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/storage/materialization-acceptance.md`
- Primary integration lock: approval batch materialization
- Outcome: Create idempotent closed-period batches, capture high-water ID, enumerate indexed/paged source rows, validate exact payloads, reconcile membership, and expose Ready only after complete materialization.
- Focused validation: Invalid/open/future/overlapping periods, empty batch, 1/100/5,000 rows, multi-page reads, duplicate request, late arrival, malformed payload, changed source digest, timeout/retry, and unauthorized caller.
- Handoff: Preserve flow definitions, query/index evidence, run IDs, counts, digests, and failure codes. No app read or editing is claimed.

## PA-36B — Expose delegated review paging to the app

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-36A; PA-35
- Exact write set: TEST ONLY flows `List Approval Batches TEST ONLY`, `Read Approval Page TEST ONLY`, and `Read Approval Item TEST ONLY`; TEST ONLY app `Intake Approval TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/storage/read-contract.json`
- Primary integration lock: approval app data binding
- Outcome: Return bounded batch summaries, stable report-order pages, and one full selected item without client-side completeness assumptions or broad list permissions.
- Focused validation: First/middle/last page, stable cursor, filter-to-problem, empty result, maximum batch, unauthorized caller, revoked membership, source/edited comparison, and no duplicate/missing item IDs.
- Handoff: Freeze request/response schemas and delegation evidence for draft-save integration.

## PA-37 — Save validated drafts with optimistic concurrency

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-35B; PA-36B
- Exact write set: TEST ONLY flow `Save Approval Item TEST ONLY`; TEST ONLY app `Intake Approval TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/finalization/save-contract.json`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/finalization/save-acceptance.md`
- Primary integration lock: approval item concurrency
- Outcome: Validate and save structured drafts/decisions with expected ETag, canonical digest, exact reread, debounced app status, and conflict recovery.
- Focused validation: Changed/unchanged save, stale ETag, simultaneous reviewers, duplicate retry, lost response, invalid field combinations, excluded-without-reason, approved batch edit, and revoked reviewer.
- Handoff: Preserve flow/app revisions, ETag traces, canonical draft examples, and conflict UX evidence. Finalization remains disabled.

## PA-37A — Freeze approval UUIDs and correction-safe projections

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-37
- Exact write set: TEST ONLY flow `Prepare Approval Finalization TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/finalization/projection-contract.json`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/finalization/correction-fixtures.jsonl`
- Primary integration lock: approval projection identity
- Outcome: Deterministically validate included drafts, assign stable approval UUIDs, build source-to-approved mappings, rewrite correction ancestry, and produce canonical candidate JSONL without committing batch visibility.
- Focused validation: Byte-identical versus edited content, same-batch/prior-batch corrections, excluded root/descendant combinations, reordered source rows, duplicate retry, UUID collision, dangling/cyclic ancestry, activity/status validation, and deterministic bytes.
- Handoff: Preserve candidate bytes, mappings, digests, and rejection reasons for final commit.

## PA-37B — Commit complete batches after exact frozen readback

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-37A
- Exact write set: TEST ONLY flows `Finalize Approval Batch TEST ONLY` and `Read Approved Batch TEST ONLY`; TEST ONLY library `Sponsor Highlights Approved Projections TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/finalization/finalization-acceptance.md`
- Primary integration lock: approval finalization
- Outcome: Recheck source completeness, acquire one finalization lease, freeze every included payload/mapping, reread exact bytes, reconcile counts/digest, and set Approved as the single visibility commit.
- Focused validation: Complete/incomplete batch, late source, concurrent finalizers, timeout before/after writes, missing/corrupt frozen file, readback mismatch, duplicate finalization, empty batch policy, and unauthorized finalizer.
- Handoff: Preserve Approved manifests, exact files/digests, run IDs, and non-visible failure evidence for PA-38.
