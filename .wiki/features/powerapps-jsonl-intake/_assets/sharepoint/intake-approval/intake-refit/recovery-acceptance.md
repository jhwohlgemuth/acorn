# PA-39A correction and recovery acceptance

November 10, 2026. The published `Sponsor Highlights Intake UI TEST ONLY` app (`427852d5-c8a6-4218-ab2d-7db34e1106cc`, live version 16) calls only `Sponsor Highlights Intake Fixture TEST ONLY` (`80dd782a-13dd-f111-84ea-001dd8062052`) for submission and history. The fixture flow is On and writes to the PA-19 TEST ONLY receipt list and `PA19 TEST ONLY JSONL` library. The app still calls the existing catalog flow to read project choices. No operational submission or correction flow binding remains in the exported package. The exported app package is `correction-recovery.msapp` (saved draft version 17, SHA-256 `bba4f02c8f00007bcb11ec52634793dfc50b4e923d2ecd9704e7413f339cad91`); `correction-recovery.pa.yaml` is its exact `SubmitEntryScreen` source.

## Observed runs

| Scenario | Observed result |
|---|---|
| Original synthetic ACORN highlight | Receipt `984f6269-7ad9-44e5-9513-c2272789e14a`; SharePoint receipt and export confirmed. Reload restored the exported state. |
| Edit and correct that highlight | Project `acorn` and report placement stayed locked. Synthetic correction produced receipt `b92043aa-ba54-497b-97ff-3498ea46906b`; flow run `08584072758225585084017189572CU16` succeeded. Its request retained `projectId: acorn`, `entry.global: false`, and revision root/supersedes both pointing to the original receipt. |
| Pending browser restart and retry | The fixture flow was turned Off before a synthetic ACORN submission. Pending receipt `bfe7185a-3cda-4b4c-8f82-5a0093e5e06d` survived reload. After the fixture flow was turned On, Retry submission used the same receipt ID and reached `Highlight saved · Export ready`. The flow was verified On afterward. |
| Rejected edit | A synthetic report-wide original was rejected by the fixture's ACORN-only original-submission rule. The app showed `Highlight needs a correction` and offered Edit highlight. This is a fixture restriction, not a general report-wide acceptance result. |
| Shared radio reset repair | Published TEST ONLY library version 18 retains `ResetSelection` and maps the radio default to its canonical key. Contributor app preloads the chosen history category/event keys before opening an edit. A saved `software` correction reopened with `software` checked and Save correction enabled; Cancel editing cleared the category and disabled Submit highlight. |
| Placement label and spacing | Live version 16 shows `General update`, with the native button title `Highlight with no associated project`. The first and second button bounds are 12 pixels apart at desktop width. The exported `Choices` formula retains `Report-wide update (no associated project)` as the second canonical key. |

The app's request payload and guard formulas preserve the existing receipt ID on retry, reject a different account's saved browser record, and require an exact export URL before showing Export ready. App Checker settled at zero formula, runtime, and data-source issues; ten accessibility findings remain under the user's November 8 deferral.

## Limits carried into cross-app acceptance

Other-account recovery, the received-but-not-exported retry branch, stale-revision conflict, and catalog failure were inspected in the exported formulas but were not independently fault-injected in this session. The fixture rejects report-wide *original* submissions, so that route cannot establish report-wide correction behavior. The app is owner-only TEST ONLY; its synthetic receipts do not publish a report. The downloaded package includes the read-only catalog flow as a related resource, but the saved app's submission/history data source points only at the isolated fixture flow.
