# PA-37A/B TEST ONLY acceptance evidence

November 9, 2026. Scope: the Wohlgemuth, Jason Power Automate environment and the restricted Research Enablement TEST ONLY review lists and approved-projection library. The operational intake and collected exporter were not changed.

## Installed flows

| Flow | ID | Observed state after validation |
| --- | --- | --- |
| Prepare Approval Finalization TEST ONLY | `c581cd8e-94cc-4c05-984c-072d3ec1424d` | Off |
| Finalize Approval Batch TEST ONLY | `4380fbd0-3c5f-45cf-8aa0-5a06424082db` | Off |
| Read Approved Batch TEST ONLY | `1a6d07d9-fee5-4736-9578-a58e1dd73f9a` | Off |

All three imported flows passed Flow checker with zero errors and zero warnings while On. They are owner-only and unshared. The finalizer and reader were regenerated from `build-finalization.py`; their action trees and flow IDs matched the imported ZIPs. The preparation flow was saved after correcting its `Invalid_items_PA37A` filter; the builder contains the same repair.

## Projection preparation

| Case | Direct evidence |
| --- | --- |
| Draft batch rejected before review and source reads | Run `08584073302905215962086362256CU17` stopped at `Ready_batch_PA37A`. |
| Edited root and same-batch correction | Run `08584073300412516775783513349CU23` produced two Pending JSONL files and an exact-readback manifest. The root received approval ID `54a58a1b-cc15-44b6-bd66-e088545c879e`; the correction received `20996ee1-6590-4382-8b65-0f3c5d472103` and pointed to the approved root. |
| Stable retry | Run `08584073297659778842406640995CU10` wrote a new Pending candidate with the same approved IDs, mapping, and payload Base64 values. |
| Exclusion | Run `08584073295945368424957222072CU01` produced one included file and recorded the excluded correction, reason, and draft snapshot. |
| Excluded root with included correction | Run `08584073292851639290879934089CU15` failed in `Revision_PA37A` before manifest creation. Both review decisions were then restored to Approve. |
| Candidate handed to finalization | Run `08584073292170054724081502341CU25` prepared `Pending/f8a27882-b876-4546-b46a-23db188a7912/92065035-c4fd-44b5-adc3-93a948b13783/` with two files and a manifest. |

The pure `projection-oracle.py --self-test` passed on November 9 for reordered rows, changed and unchanged content, same-batch and prior-Approved ancestry, stable retry, exclusion, invalid graph, and activity/status cases. The prior-Approved ancestry path was checked in the builder and oracle but was not exercised in a cloud run.

## Finalization and Approved read

The finalizer committed fixture batch `f8a27882-b876-4546-b46a-23db188a7912` from candidate `92065035-c4fd-44b5-adc3-93a948b13783` in run `08584073273369684541807522165cU14` (Succeeded, 14 seconds). Run history showed `Exact_source`, `Each_review_row`, `Acquire_lease`, `Each_frozen_row`, `Exact_frozen_manifest`, `Still_exact`, `Commit_approved`, `Committed`, and `Respond_approved` succeeding. The SharePoint batch showed State `Approved`, ApprovedCount `2`, ExcludedCount `0`, UnresolvedCount `0`, a projection fingerprint, and the finalizer identity/time. `Approved/<batchId>/` contained `1.jsonl`, `2.jsonl`, and `manifest.json`; the Approved manifest matched the selected Pending manifest exactly.

The reader run `08584073272678036574530806396CU05` succeeded through its Approved-only gate, manifest check, two-row frozen-file loop, complete-read check, and response. Its secure response content was hidden in run history. An earlier Ready-batch reader run `08584073274900234689270906624CU07` failed at `Approved_only_PA37R` with the manifest read skipped. Duplicate finalization run `08584073271298107727767732024CU18` returned through `Read_existing_manifest` and `Respond_existing`; lease acquisition was skipped. A stale exclusion candidate run `08584073274189738057591186392CU07` failed at `ReviewChangedPA37B` before lease acquisition. Empty Ready batch `4aaece96-96dd-4df0-9f48-71350f514971` failed at the Ready gate in run `08584073269966335878045470038CU31`; pending-manifest read and lease acquisition were skipped.

## Bounded validation

The installed definition checks current finalizer/exporter group membership, complete source interval membership and exact source receipts, review ETags/versions/decisions/draft bytes, candidate bytes and mappings, conditional Ready-to-Finalizing and Approved writes, exact frozen file and manifest readback, post-freeze source/review snapshots, and Approved-only reader access. These checks are definition evidence; they do not establish the behavior of fault scenarios that were not injected.

After the cloud acceptance runs, a local review found that the finalizer's 10 MiB check omitted the LF separators added by the reader between included rows. The builder adds one byte per boundary before lease acquisition. Local action-shape, dependency-order, and one-, two-, and 5,000-row byte-boundary checks pass. On November 10, the existing TEST ONLY finalizer was edited directly to install this expression and trust the PA31 receipt list; it saved with zero checker errors and one long synchronous-response warning. Two-row PA31 project/General and same-batch correction candidates finalized successfully in runs `08584072534030371219662616198CU24` and `08584072522550346746579252106CU19`; the flow returned to Off. The exact 10 MiB cloud boundary has not been exercised. See the [PA-40 end-to-end checkpoint](../acceptance/end-to-end.md) for the prepared/frozen manifest and file identities.

No cloud run injected a late in-period source, source or review mutation during freeze, two simultaneous finalizers, a timeout/lost response, missing or corrupted frozen files, a revoked finalizer, a prior-Approved correction, or a 5,000-row finalization. Those remain bounded TEST ONLY acceptance limits. The reviewed flow is not bound to the operational intake or exporter, and all three PA-37 flows were returned to Off after the runs above.

## Local verification

`/opt/homebrew/bin/python3 -B finalization/projection-oracle.py --self-test` passed. `build-finalization.py` regenerated prepare, finalize, and reader ZIPs from the native template without shape-validation errors. Action-tree and flow-ID comparison of the regenerated finalizer and reader against the imported packages returned `True` for both.
