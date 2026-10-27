# PA-23 aggregate acceptance evidence

All native runs target `PA26 TEST ONLY SharePoint Logbook Export` (`28cba548-c862-484c-b838-ea21eaf52630`), PA19 TEST ONLY Receipts and PA19 TEST ONLY JSONL in the Jason environment. The operational exporter remains On at its October 26, 14:57 revision; `operational-readonly-state.txt` records the read-only check.

The builders create Power Automate clipboard definitions locally. They do not call SharePoint. Native test runs are manually started once, and the test flow is turned Off immediately so recurrence cannot seed duplicate fixtures. Seeding definitions are not release components and must not be blindly resubmitted: SharePoint batches are nontransactional and UUIDs are unique.

- `build_acceptance.py`: cutoff/late-arrival and empty/malformed fixtures. `small-snapshots-first.clipboard.json` preserves the initial native run that exposed the zero-byte response defect. `recovery-snapshots.clipboard.json` uses the repaired response handling.
- `build_boundaries.py`: native guard and real file write/readback tests with generated receipt arrays. These do not prove connector pagination. Large expected files are regenerated under ignored `output/pa23-20261027/boundary-fixtures`.
- `build_pagination.py`: real list rows, 100 writes per sequential SharePoint batch, verification of every 201 response, then actual GetItems retrieval at 5,000 and 5,001 rows. The two-row preflight verified the batch response shape before bulk seeding.
- `accepted-isolated-exporter.clipboard.json`: clean candidate using the normal secure-data settings, test list/library, and the same single valid synthetic fixture filter as the pre-test baseline.
- `downloaded-cutoff.jsonl` and `downloaded-next.jsonl`: independent browser downloads, compared byte-for-byte with the separately generated originals.
- `cli-download-acceptance.json`: offline stage/inspect/apply and replay/conflict results from the downloaded snapshots against one isolated local state/index. It records the existing local executable hash; it is not a claim of a new build.
- `cli-boundary-acceptance.json`: supplementary exact-size staging and one-project capacity/reused-UUID rejection; no large batch was applied.
- `independent-downloads.json`: independent downloaded byte counts and hashes for size/guard/empty cases. Reproducible large downloads are not duplicated in the wiki.

Expected guard-failure branches in test harnesses use Compose markers so independent prior-file assertions run afterward. Release definitions retain their terminating failure guards. Synthetic contents must remain outside sponsor reports.
