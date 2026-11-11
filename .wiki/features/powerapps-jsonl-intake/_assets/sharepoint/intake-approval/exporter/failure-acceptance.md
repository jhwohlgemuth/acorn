# PA-38B TEST ONLY acceptance

Accepted for isolated PA-40 use on 2026-11-10. This evidence does not authorize operational cutover. The tested [flow](https://make.gov.powerautomate.us/environments/221f44a6-1468-ea4e-b93d-cd52f1723079/flows/6f74ecff-bfcd-4377-afdf-39b03b4beff1/details) has ID 6f74ecff-bfcd-4377-afdf-39b03b4beff1 and uses the existing o9w@ornl.gov SharePoint connection. Its trigger is **Off** after acceptance. The operational unfiltered exporter was not changed.

## Exact isolated inputs and output

- The separate Sponsor Highlights Collected Export TEST ONLY library contains a synthetic baseline. An independent SharePoint download matched the local baseline.jsonl: 940 bytes, two JSONL lines, SHA-256 4218e52014ed65938a319e2f1f7335d34d449fadb8822eaccc1c1d41763bc9a9. The original was restored and downloaded again after the fault test with the same digest.
- The accepted PA-37B batch f8a27882-b876-4546-b46a-23db188a7912 has two included frozen files and no excluded files. The manifest's encoded payloads matched downloaded frozen bytes. The collected logbook.jsonl is exactly baseline + 1.jsonl + LF + 2.jsonl + LF: 1,886 bytes, four lines, SHA-256 1f64d8de9cb21247a22b7c2256cf9b8d8d7c8cf72dd455356561e8f304102b02.
- The final replay [run 08584072629897307767691674300CU16](https://make.gov.powerautomate.us/environments/221f44a6-1468-ea4e-b93d-cd52f1723079/flows/6f74ecff-bfcd-4377-afdf-39b03b4beff1/runs/08584072629897307767691674300CU16?v3=false) succeeded at 15:51 ET. Its saved receipt in the collected library records one Approved batch, four lines, 1,886 bytes, the baseline digest, last batch ID, file identity, and a nonempty SharePoint ETag: "{83949E5C-C42A-4189-A4FD-7C946CA3666F},1". The flow reread the receipt exactly before succeeding. The collected file's SharePoint modification time remained 15:18 ET after replay, confirming no unchanged-output write.

## Service runs and rollback

| Run | Observation |
|---|---|
| 08584072649556317830304898732CU15, 15:18 ET | First changed candidate succeeded: explicit prior-file 404, one collected write, and exact output readback. |
| 08584072648224675823727684520CU20, 15:21 ET | Unchanged replay succeeded; Write logbook LB was skipped. |
| 08584072643992215426603596726cU14, 15:28 ET | A corrupted TEST ONLY baseline caused Exact baseline PA38 to fail. The collected file retained the 1,886 bytes and SHA-256 above. The baseline was restored and rechecked. |
| 08584072635852628983331279617cU14, 15:41 ET | Updated batch-count and included-order guards succeeded on the accepted batch. |
| 08584072633196249360978186632CU12, 15:46 ET | First persisted receipt succeeded, but its file revision was null. This is defect evidence, not a valid freshness receipt. |
| 08584072629897307767691674300CU16, 15:51 ET | Corrected metadata read, nonempty revision guard, receipt write, and exact receipt readback succeeded. |

The freeze-only rehearsal turned the TEST ONLY trigger Off, read the collected file with the same exact digest, and left the operational exporter untouched. See [rollback](rollback.md).

## Local failure matrix and limits

The focused test command in test_exporter.py passed 19 tests after final package generation. Candidate tests cover baseline prefix, deterministic replay, append-only Approved extension, non-Approved states, exclusions, duplicate IDs, invalid manifests/counts/order/fingerprint, changed or missing frozen bytes, invalid JSON, correction ancestry, 5,001 rows, and 10 MiB overflow. Graph simulations cover changed/unchanged publication, confirmed-missing creation, prior-read 403/500/timeout, a competing writer, both lost-write-response outcomes, readback failure/mismatch, disabled retries, and non-overwrite creation. Static checks cover top-level initializers, included-order enforcement, and receipt dependency order.

Only changed, unchanged, baseline-corruption, and freeze paths were exercised in Power Automate. Connector faults, competing writes, lost responses, and upper bounds were simulated against the generated graph; their service behavior remains a PA-40/PA-41 acceptance risk. The PA-37B b64-edge-v1 projection fingerprint and export receipt fingerprint are diagnostic, not collision-resistant digests. Exact frozen-byte and logbook readback comparisons enforce equality during a run; the SHA-256 values above document accepted files. The TEST ONLY baseline is editable by the current flow identity; PA-41 must capture and protect the operational baseline and bind a suitably scoped identity before activation.

PA-40 readiness: **Yes for isolated end-to-end acceptance**. Production cutover remains gated by PA-40A and PA-41, including access, baseline protection, monitoring, and operational backup.
