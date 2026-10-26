# Power Platform test artifact cleanup

Owner-requested cleanup performed October 25, 2026. Only completed, explicitly test-only resources were removed. Operational apps, flows, connectors, connections, credentials, inbox, callback table and receipts were not edited.

## Removed cloud resources

| Environment | Resource | ID |
| --- | --- | --- |
| Wohlgemuth, Jason | PA14 TEST ONLY Sponsor Highlights | `62ef8e08-4253-4fd8-91e1-a7c0a13d7fd3` |
| Wohlgemuth, Jason | PA14 TEST ONLY intake simulation | `7364dc72-5b4b-45aa-9dea-c95dc91614be` |
| Wohlgemuth, Jason | PA14 TEST ONLY catalog simulation | `8fdddd79-52e5-4576-bd95-19c3ffb40be7` |
| ORNL Personal Productivity | PA14 TEST ONLY Sponsor Highlights | `8fcc3d17-88d3-4dd5-a990-b4ef9843013a` |
| ORNL Personal Productivity | PA14 TEST ONLY intake simulation | `9af88959-801d-4d6f-892c-e34f42c3b38a` |
| ORNL Personal Productivity | PA14 TEST ONLY catalog simulation | `9f830720-ea02-46aa-bdac-e493052ed32f` |
| Wohlgemuth, Jason | ACORN Intake Failure Acceptance TEST ONLY 2026-10-24 | workflow `74365385-02d0-f111-a05c-001dd8062052`; listing identity `1e5461ab-a4b6-3c27-66ba-cefca88e553a` |

The app IDs were checked on their Details pages before removal. The failure mock was Off, had no associated apps and no custom solution references. Refreshed inventories verified the removed resources were absent. [Browser-recovery evidence](browser-recovery-acceptance.md) and [failure-response evidence](write-failure-acceptance.md) remain; their cloud links are now historical.

![Jason app inventory after cleanup](_assets/boundary-tests/jason-app-cleanup.png)

![ORNL app inventory after cleanup](_assets/boundary-tests/ornl-app-cleanup.png)

## Retained resources

- Operational Sponsor Highlights, its intake/catalog flows, and manual callback receipt/notification workflow.
- Both PA-17 native test apps, test solutions, intake/catalog flows, callback test flow, connectors and protected references. Full restore acceptance is unfinished.
- The two PA-15 boundary copies, with notifications disabled, for the remaining acceptance matrix. Their IDs and final states are recorded in [boundary acceptance](boundary-acceptance.md).
- Older `ACORN Logbook Canvas Notifications`, ORNL `ACORN Logbook Intake`, and ORNL `Test` flows: their ownership/dependencies were not established as disposable, so they were not removed.
- Native restore archives, manifests, current PDFs, demo packet and durable acceptance evidence.

## Wiki asset cleanup

Two personal Teams screenshots were removed from plan assets. Current guides use the fictional `teams-example.png`; the personal screenshots were already excluded from distributed archives. Four byte-identical duplicate files were removed, preserving their canonical copies and updating the current runbook link. Exact paths and replacements are recorded in [asset-cleanup.json](_assets/boundary-tests/asset-cleanup.json).

Source/demo archives and manifests were rebuilt from their selected current files. Historical failure screenshots and native restore packages are retained because they explain unresolved acceptance or support recovery. This cleanup does not certify PA-15, PA-16 or PA-17.
