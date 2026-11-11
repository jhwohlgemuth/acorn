# PA-38 approval exporter rollback

Status: TEST ONLY rehearsal accepted on 2026-11-10; operational cutover remains PA-41.

## Trigger

Use this procedure when an approval-aware export run fails validation, has an uncertain write outcome, cannot verify readback, or emits an incorrect receipt. Do not infer file state from a failed connector action alone.

## Freeze

1. Turn off the approval-aware exporter trigger. Record the flow ID, last run ID, failure code, and time. Prevent a second writer from starting while the outcome is reconciled.
2. Read `logbook.jsonl` directly from the collected library. Record its SharePoint file identity and revision, exact byte count, line count, and SHA-256 digest. Compare the bytes with the last verified success receipt and the candidate from the failed run when available.
3. Preserve the file as found. If readback fails or the bytes match neither known state, label the outcome uncertain and keep the trigger off. Do not overwrite or delete the file to force a rollback.
4. Preserve the exact baseline, Approved manifests, frozen files, flow package, and run trace. Repair the approval-aware flow or its inputs in TEST ONLY storage and validate a fresh candidate before resuming.

## Policy boundary

After operational approval cutover, the old unfiltered exporter must stay disabled. Rollback freezes the last verified collected file; it never re-enables source-row aggregation or rebuilds from mutable receipts. PA-41 owns production cutover and recovery authorization.

## Rehearsal evidence

Flow 6f74ecff-bfcd-4377-afdf-39b03b4beff1 published a verified changed candidate in run 08584072649556317830304898732CU15 and skipped the collected write on unchanged run 08584072648224675823727684520CU20. The file remained 1,886 bytes, four lines, SHA-256 1f64d8de9cb21247a22b7c2256cf9b8d8d7c8cf72dd455356561e8f304102b02. Corrupting the TEST ONLY baseline caused run 08584072643992215426603596726cU14 to fail before publication; an independent collected-file download retained that digest. The baseline was restored and its 940 bytes and SHA-256 4218e52014ed65938a319e2f1f7335d34d449fadb8822eaccc1c1d41763bc9a9 rechecked.

Final replay run 08584072629897307767691674300CU16 saved a readback-verified receipt with the collected file revision. The collected file's SharePoint modified time remained 15:18 ET, its initial publication time. The TEST ONLY trigger was turned Off after that run; the detail page showed Status Off and Turn on. The old operational exporter was not changed or enabled as part of this rehearsal.
