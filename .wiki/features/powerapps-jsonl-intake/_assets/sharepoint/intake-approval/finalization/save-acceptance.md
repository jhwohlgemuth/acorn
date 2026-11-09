# PA-37 optimistic-concurrency save acceptance

Status: accepted on the TEST ONLY track.

## Fixed boundaries

- Flow: `Save Approval Item TEST ONLY`.
- App: `[PROTOTYPE] Intake Approval`, saved and unpublished.
- Reviewers group: SharePoint group ID `42`, checked on every call.
- Mutable fields: canonical two-part draft projection, draft fingerprint/version/audit fields, decision, reason, validation state, and validation errors only.
- Source receipts, captured source fields, batch state, approved fields, approved projection files, exporter inputs, and contributor rows remain outside the write path.
- `DraftDigest` is a deterministic, non-cryptographic edge fingerprint. Exact equality always compares the complete concatenated Base64 draft, and optimistic concurrency always uses item ETag plus draft version.

## Validation matrix

| Case | Required evidence | Result |
|---|---|---|
| Changed save | Old/new ETag, old/new draft version, canonical readback | Passed in run `08584073727962498035610290463CU21`: ETag `"2"` / version `2` advanced atomically to ETag `"3"` / version `3`; the exact reread matched the full canonical draft, fingerprint, audit fields, decision, reason, and validation state |
| Unchanged save | Same ETag/version before and after repeated identical request | Passed in run `08584073723644956291424417183CU11`; the repeated pre-save ETag/version request returned unchanged and skipped the write |
| Stale ETag | Conflict response and unchanged current item | Passed in live flow run `08584074143341635889672618714CU00`; stale branch succeeded and no write ran |
| Simultaneous reviewers | Two requests from one original ETag; one write and one conflict | Passed: writer A run `08584073721360878165002298695CU30` wrote ETag `"3"` / version `3` to ETag `"4"` / version `4`; writer B run `08584073721251947088503286858CU30` used the same original pair, returned conflict, and skipped the write |
| Duplicate retry | Identical retry returns unchanged without a new version | Passed in run `08584073721149700938255380590CU13`; the writer-A retry returned unchanged and skipped the write |
| Lost response | Retry with pre-save ETag reconciles the exact saved state as unchanged | Passed by the same unchanged runs: each reused the pre-save concurrency pair and reconciled the complete saved state without a new version |
| Invalid decision | Invalid response and no write | Passed in run `08584073721049402100752267140CU28` |
| Exclude without reason | Invalid response and no write | Passed in run `08584073720935307878462089115CU25` |
| Non-Ready batch edit | Batch-not-ready response and no item disclosure/write | Passed against Draft batch `0fe6c775-617c-4912-9b53-6e936cee214c` in run `08584073720830080756135976650CU09` |
| Revoked reviewer | Forbidden response before batch/item read | Bounded identity limit: the TEST ONLY reviewer group contains only the owner, so membership was not revoked during the session |
| App save states | Saving, Saved, Conflict, and Failed retain recoverable local edits | Passed. The compact changed request advanced the app from ETag `"8"` to exact reread `"9"`; unchanged save stayed at `"9"`; a second app writer advanced to `"10"`; the stale client displayed `Conflict`, kept its local summary, and retained ETag `"9"`; after the writer saved that retained summary at `"11"`, retry returned unchanged `Saved` and adopted exact ETag `"11"` without another increment. Exclude-without-reason and service failures displayed `Failed` and retained the draft. On November 9, the fixture batch was temporarily set to `Failed`; the live app showed `Failed · This batch is locked. Your local draft is retained.` and kept the local summary `PA-37 TEST ONLY local conflict retained`. The batch was then visibly restored to `Ready` with its counts unchanged |

## Current implementation evidence

- Import completed as `Save Approval Item TEST ONLY`, flow ID `8b17527b-c901-416c-aebb-baa353bbca7a`.
- The imported definition has a Power Apps V2 request schema, per-call reviewer membership query, exact Ready/item guards, bounded validation, deterministic source-derived canonical draft serialization, unchanged-state reconciliation, and exact reread checks.
- The request URI and body were rebuilt from clean expression values. The repaired flow saved with Flow checker reporting zero errors and zero warnings. It remains owner-only and is not bound to a published app.
- A request with current item ETag `"2"` and stale draft version `1` completed successfully through `Respond stale PA37`; `Write item PA37`, `Respond saved PA37`, and `Respond unchanged PA37` were skipped. The selected fixture remained at item ETag `"2"`, draft version `2`, and its prior draft bytes.
- Run `08584073727962498035610290463CU21` proved the clean atomic changed save and exact reread. Direct SharePoint REST evidence showed ETag `"3"`, draft version `3`, decision `Approve`, validation `Valid`, and the complete Base64 draft decoding to summary `PA-37 TEST ONLY optimistic save` while retaining the remaining captured source structure.
- Writer A run `08584073721360878165002298695CU30` then advanced the same item to ETag `"4"`, draft version `4`, and summary `PA-37 TEST ONLY writer A`. The exact final REST reread still showed those values after the competing writer, retry, invalid-decision, missing-reason, and Draft-batch cases, proving that none of those later cases mutated the item.
- `Save Approval Item TEST ONLY` now accepts one compact `RequestJson` string and parses the seven request fields in the flow. `Respond saved PA37`, `Respond unchanged PA37`, and `Respond stale PA37` return native `string(...)` expressions rather than literal `@{...}` text. The app calls that single compact input and parses the returned item JSON.
- `[PROTOTYPE] Intake Approval` contains `SaveReviewButton` and `SaveStatusLabel`, is saved and unpublished, and has no formula errors. Live preview proved changed and unchanged `Saved`, stale-writer `Conflict`, validation and service `Failed`, local-draft retention, retry reconciliation, and successful replacement of the scalar ETag/version with exact reread values. The final restored preview showed summary `PA-37 TEST ONLY local conflict retained`, decision `Approve`, `Saved. No server change was needed; exact reread applied.`, and ETag `"11"`.

## Bounded identity limit

The server contract, compact app binding, exact reread, and live changed/unchanged/conflict/retry/validation/service states are accepted. The November 9 non-Ready app check used batch `f8a27882-b876-4546-b46a-23db188a7912`; the server returned the locked response and the app retained the local draft. The revoked-reviewer case remains a bounded identity limit unless a second TEST ONLY identity becomes available. Finalization remains disabled until PA-37B.
