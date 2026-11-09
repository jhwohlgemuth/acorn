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

On November 8, 2026, the TEST ONLY reviewer app draft generated a summary from a synthetic local fixture through `Summarize Reviewer Item TEST ONLY`; the flow's `Run a prompt` action succeeded. The app remains unpublished and fixture review does not write review-ledger rows. The materializer is Off. Its earlier rejection was caused by a `State` string comparison: SharePoint returns `State` as a choice object with `Value: "Draft"`. The saved guard now reads `State.Value`, retains the exact-one-row and closed-period checks, and passes Flow checker with zero errors and warnings. Corrected run `08584074345624719645844541435CU01` took the guard's yes branch and succeeded through the bounded source reads; no review rows were written. See [summary contract](../_assets/sharepoint/intake-approval/app/ai-summary-contract.md).

The review-item scope is now saved in the TEST ONLY materializer. Empty-period run `08584074331870042769622156638CU28` reconciled zero source and review rows and marked batch `4aaece96-96dd-4df0-9f48-71350f514971` Ready with zero counts. The flow was returned to Off. This does not validate populated, paged, correction, retry, integrity-failure, or authorization cases. See [materialization acceptance](../_assets/sharepoint/intake-approval/storage/materialization-acceptance.md).

The approved October 26 TEST ONLY run created six restricted review items under batch `aae6eba5-ea56-477d-abbe-93161e743866`; its first reconciliation rejected the populated batch. Guarded retries added no items. A diagnostic run compared all eleven fingerprint fields for all six source/review pairs successfully, while whole-array string equality failed. With the user's action-time approval, the saved scope was replaced with a generated version that normalizes numeric source item IDs to text and compares arrays of fingerprint strings. Run `08584074303517479023363906361CU00` succeeded. SharePoint now shows the batch Ready with source/item/unresolved counts of six and exactly six review-item rows. The materializer remains Off. See [materialization acceptance](../_assets/sharepoint/intake-approval/storage/materialization-acceptance.md).

The batch creator reused the October 26 batch on an exact-period retry, rejected an overlapping period, and rejected a future period before lookup. A malformed start timestamp failed before batch creation. The user chose isolated TEST ONLY fixtures for correction, malformed payload, larger pagination, partial retry, and mutation cases. The copied source fixture list contains a synthetic root/correction pair. Fixture materializer run `08584074290834244800397373633CU25` reached Ready with source/item/unresolved counts of two; the correction review item retained its root and predecessor. A third synthetic row with a mismatched payload submission ID was rejected in run `08584074285640202835322304200CU26`; its batch stayed Draft and no review item was created. A Ready-batch retry initially failed at the batch-state guard. With the user's action-time approval, both materializers were repaired and saved with zero checker errors and warnings. Fixture retry run `08584074271372033577423346817CU21` succeeded through the Ready branch and skipped the source read. Both materializers are Off, and no operational source rows were altered. See [materialization acceptance](../_assets/sharepoint/intake-approval/storage/materialization-acceptance.md).

The separate TEST ONLY seed flow was imported as `b83ff8ad-2ecf-44f1-9e4b-2d612bc76f78` and its 100-row run `08584074246986131369023945305CU02` succeeded through the explicit 100-response guard. The closed fixture batch `288f52db-78ae-43e5-8a40-c2e413e3b7b3` reached Ready in materializer run `08584074244989093229734115024CU20` with high-water ID 103 and source/item/unresolved counts of 100. The seed range was advanced to distinct UUIDs after user approval, and run `08584074241108200013464837542CU16` created and verified 5,000 reserved-example source rows. Materializer run `08584074235619154980944169987CU28` succeeded; the batch is Ready with high-water ID 5103 and source/item/unresolved counts of 5,000. The seed and fixture materializer flows are Off. A missing-parent correction fixture failed at `InvalidCorrectionGraphPA36A` before creating an item. See [materialization acceptance](../_assets/sharepoint/intake-approval/storage/materialization-acceptance.md).

PA-36A and PA-36B are accepted on the TEST ONLY track. Materialization reconciled empty, six-row, correction, 100-row, and 5,000-row batches; rejected malformed payload and correction fixtures; reused partial rows without duplication; and returned an existing Ready result after an uncertain caller response. The materializers and seed flow are Off. Three imported Power Apps V2 read flows match the [read contract](../_assets/sharepoint/intake-approval/storage/read-contract.json), passed Flow checker, and remain On, owner-only, and unshared. The unpublished app listed five Ready batches, traversed all 5,000 IDs without a cursor gap or duplicate, stopped correctly on 100-row and empty batches, excluded an Approved/Valid row from Needs review while retaining its Unresolved correction, and displayed distinct decoded source and nonempty draft JSON with item ETag `"2"`. [Materialization acceptance](../_assets/sharepoint/intake-approval/storage/materialization-acceptance.md) and [read acceptance](../_assets/sharepoint/intake-approval/storage/read-acceptance.md) record run IDs and bounded identity/fault-injection limits. Mutation and late-arrival enforcement remain PA-37B finalization gates; revoked-reviewer write denial remains a PA-37 gate.

PA-36C is accepted on the TEST ONLY track. The saved, unpublished reviewer now generates a suggestion from the selected original entry through `Summarize Reviewer Item TEST ONLY`, sends only the entry summary/items/category/state/event type/timestamp, and never sends submitter, source identifiers, ETags, saved draft, validation state, or decision. A live reserved-example review item and separate activity/status fixtures produced bounded factual suggestions. Discard cleared the result; Use suggestion changed only local app state and displayed `Applied locally (not saved)` while the saved draft, Approve decision, Valid state, and item ETag `"2"` remained unchanged. Switching to the correction item cleared the prior result before its read completed. Empty, over-12,000-character, empty/over-1,200-character response, stale-selection, and service-error states are present in the checker-clean formulas. App checker reported zero formula issues. The successful flow route is available, but external capacity failure was not injected and the future credit source/quota remain unknown. See [summary contract](../_assets/sharepoint/intake-approval/app/ai-summary-contract.md).

PA-37 is accepted on the TEST ONLY track. The rebuilt atomic REST request advanced the fixture from ETag/version `"2"`/`2` to `"3"`/`3` with exact reread; an identical pre-save retry returned unchanged. Of two different requests using ETag/version `"3"`/`3`, writer A advanced the item to `"4"`/`4` and writer B returned conflict without writing. A lost-response retry reconciled unchanged, while invalid decision, exclude-without-reason, and Draft-batch requests all rejected without mutation. The owner-only reviewer group keeps revoked-reviewer denial as a bounded identity limit. The saved, unpublished app calls one compact JSON trigger input and live preview proved changed `Saved` (`"8"` to `"9"`), unchanged `Saved` at `"9"`, a second writer at `"10"`, stale-client `Conflict` with its local draft retained at `"9"`, and lost-response retry reconciliation at exact ETag `"11"` without another increment. Exclude-without-reason and transient service failures displayed `Failed` and retained local edits. On November 9 the selected TEST ONLY fixture batch was temporarily set to `Failed`; the app showed `Failed · This batch is locked. Your local draft is retained.` and kept the local summary, after which the batch was visibly restored to `Ready` with counts unchanged. See [save acceptance](../_assets/sharepoint/intake-approval/finalization/save-acceptance.md).

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
| `sourcePayloadBase64A/B` / `sourceETag` | Exact captured UTF-8 JSON encoding (split at 45,000 characters) and source SharePoint ETag; never edited. |
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

The existing intake caps encoded receipts at 87,380 characters. Store the captured payload in two plain multiline list columns, as the operational receipt list already does, with a 45,000-character first part. The app receives only fields required for the current page and never broad list access. The user approved exact encoded-byte comparison and source ETag for PA-36A because the standard cloud-flow expression set has no SHA-256 function; do not label an encoding or ETag as a cryptographic digest. Later draft/finalization digest fields remain reserved until their phases select a supported implementation.

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
5. Validate each exact source payload and capture its ETag; create review items by unique membership key.
6. Requery/count the same bounded source set and compare IDs, count, ETags, and complete encoded payloads with materialized items.
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
- Source mutation: any ETag or exact encoded-byte change from the captured source is a terminal integrity failure; do not adopt new bytes silently.
- Late in-period source: block finalization and require a documented rematerialization decision.
- Invalid correction graph: block the affected item and batch; never publish a dangling `supersedesSubmissionId`.
- Reviewer loses group membership: the next flow call denies access; cached app data does not grant authority.
- Approved payload/library unavailable: batch remains non-visible or Failed; prior collected export is unaffected.

# Boundaries

This plan owns TEST ONLY review lists/libraries, review flows, app data bindings, concurrency, finalization, and frozen projections. The user authorized one narrow operational source-list metadata change for PA-36A: add an index on `Created`, which was absent on November 8, 2026. It does not modify source rows, the collected exporter, contributor app, ACORN schema, or local report renderer. The exporter plan consumes only Approved manifests and frozen bytes.

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

## PA-37S — Bind batches to their materialization source

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: accepted TEST ONLY materialization evidence recorded in the current evidence
- Exact write set: TEST ONLY list `Sponsor Highlights Review Batches TEST ONLY`; TEST ONLY flows `Materialize Approval Batch TEST ONLY` and `Materialize Approval Batch Fixture TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/storage/schema.json`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/storage/build-materialization.py`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/storage/build-source-binding.py`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/storage/source-binding-acceptance.md`
- Primary integration lock: approval batch source binding
- Outcome: Record one immutable trusted source-list ID on each batch when materialization starts, and backfill accepted TEST ONLY batches from their verified materialization flow histories.
- Focused validation: Both materializers set the expected list ID, exact retries preserve it, all seven existing TEST ONLY batches have the correct binding, a mismatched source is rejected, and no operational source rows or exporter settings change.
- Handoff: Preserve the schema and flow revisions, all seven batch bindings, run IDs, and mismatch rejection evidence for PA-37A/B.

## PA-37A — Freeze approval UUIDs and correction-safe projections

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted TEST ONLY save evidence recorded in the current evidence; PA-37S
- Exact write set: TEST ONLY flow `Prepare Approval Finalization TEST ONLY`; TEST ONLY library `Sponsor Highlights Approved Projections TEST ONLY` pending area; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/finalization/projection-contract.json`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/finalization/correction-fixtures.jsonl`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/finalization/projection-oracle.py`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/finalization/build-finalization.py`
- Primary integration lock: approval projection identity
- Outcome: Deterministically validate included drafts, assign stable approval UUIDs, build source-to-approved mappings, rewrite correction ancestry, and produce canonical candidate JSONL without committing batch visibility.
- Focused validation: Byte-identical versus edited content, same-batch/prior-batch corrections, excluded root/descendant combinations, reordered source rows, duplicate retry, UUID collision, dangling/cyclic ancestry, activity/status validation, and deterministic bytes.
- Handoff: Preserve candidate bytes, mappings, digests, and rejection reasons for final commit.

## PA-37B — Commit complete batches after exact frozen readback

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-37A
- Exact write set: TEST ONLY flows `Finalize Approval Batch TEST ONLY` and `Read Approved Batch TEST ONLY`; TEST ONLY library `Sponsor Highlights Approved Projections TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/finalization/build-finalization.py`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/finalization/finalization-contract.json`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/finalization/finalization-acceptance.md`
- Primary integration lock: approval finalization
- Outcome: Recheck source completeness, acquire one finalization lease, freeze every included payload/mapping, reread exact bytes, reconcile counts/digest, and set Approved as the single visibility commit.
- Focused validation: Complete/incomplete batch, late source, concurrent finalizers, timeout before/after writes, missing/corrupt frozen file, readback mismatch, duplicate finalization, empty batch policy, and unauthorized finalizer.
- Handoff: Preserve Approved manifests, exact files/digests, run IDs, and non-visible failure evidence for PA-38.
