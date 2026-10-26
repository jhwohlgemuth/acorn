---
type: Note
title: "SharePoint standard-connector pilot"
description: "Deployment evidence and handoff for the parallel Sponsor Highlights SharePoint route."
---

# Current deployment

October 26, 2026: the owner requested a shared list through the ORNL SharePoint directory. The existing private [Research Enablement site](https://ornl.sharepoint.com/sites/ResearchEnablement) exposes creation rights and currently displays two group members. Its membership and sharing settings were not changed. This is shared site storage, not a personal Microsoft Lists location; it does not establish access for every intended contributor.

Created in that site:

| Component | Location | State |
|---|---|---|
| Intake list | [Sponsor Highlights Intake Pilot](https://ornl.sharepoint.com/sites/ResearchEnablement/Lists/Sponsor%20Highlights%20Intake%20Pilot/AllItems.aspx) | TEST ONLY receipts from manual and published-app acceptance; list ID `a904a6c6-5dd0-427b-9575-8167c4c0c412` |
| Project catalog | [Sponsor Highlights Projects Pilot](https://ornl.sharepoint.com/sites/ResearchEnablement/Lists/Sponsor%20Highlights%20Projects%20Pilot/AllItems.aspx) | 13 seeded rows; list ID `88dfa78d-a62f-4e0f-aae5-0d5ea52758ed` |
| JSONL library | [Sponsor Highlights JSONL Pilot](https://ornl.sharepoint.com/sites/ResearchEnablement/Sponsor%20Highlights%20JSONL%20Pilot/Forms/AllItems.aspx) | Receipt-named JSONL files; exact readback passed in manual and app-driven runs |

The original Sponsor Highlights app, GitLab intake/catalog bindings and operational logbook are unchanged. Native Save As created and published [Sponsor Highlights - SharePoint Pilot](https://apps.gov.powerapps.us/play/e/221f44a6-1468-ea4e-b93d-cd52f1723079/a/163f110f-7f8a-401c-894e-0786b2c018a4), app ID `163f110f-7f8a-401c-894e-0786b2c018a4`, in Wohlgemuth, Jason. Its GitLab intake/catalog bindings were removed after switching formulas to the two SharePoint pilot flows. Studio reports no formula or data-source errors. Owner-context app submissions passed; intended-user acceptance and cutover remain open.

Solution **ACORN SharePoint Pilot** (`0fafd397-64d1-f111-a05c-001dd8062052`) was created with the two pilot flows and SharePoint connection reference `cr1d0_sharedsharepointonline_0c871`. This registers the flows for selection in Studio; it does not add Dataverse data actions to the intake. The subsequent Teams action uses existing **Microsoft Teams Default-52fd4**, under `o9w@ornl.gov`; the connection credentials were not changed. The app uses `SponsorHighlightSharePointPending` for browser recovery, separate from the original GitLab key. It distinguishes a received SharePoint receipt from a confirmed JSONL export; a pending export retains the UUID and offers a safe retry. [Form formula changes](_assets/sharepoint/form-patches.json) were compared with the live [layout](_assets/sharepoint/layout.pa.yaml) and [result panel](_assets/sharepoint/result-panel.pa.yaml). No original GitLab bindings, commit fields, queued states or original storage key remain in those captured controls. The app still needs inclusion in an exported native solution backup.

# Intake storage contract

| Column | Saved type/settings | Purpose |
|---|---|---|
| Title | Existing single-line text | Short receipt display label; not the full summary |
| SubmissionId | Single-line text, required, unique, maximum 36 characters | Stable receipt UUID, enforced unique by SharePoint |
| PayloadBase64A | Plain multiline text, required, append changes off | First 45,000 characters of base64 of the original UTF-8 envelope |
| PayloadBase64B | Plain multiline text | Remaining base64 characters, empty for smaller envelopes |
| ProjectId | Plain multiline text | Supports the existing 512-character project ID contract |
| Summary | Plain multiline text | Readable summary; preserves the existing 1,200-character limit in the flow |
| SubmitterEmail | Single-line text | Caller email after server-side verification |
| ExportStatus | Single-line text, default `Pending` | `Exported` only after exact JSONL readback |
| ExportFileUrl | Plain multiline text | Confirmed export location |

The two payload fields avoid the per-field multiline text ceiling for the existing maximum 65,535-byte envelope. Base64 is encoding, not encryption. The flow must still enforce the envelope schema, raw size and newline rules, caller identity, timestamps, project membership and exact replay comparison. List column creation alone supplies none of those validations.

Export must decode A followed by B and preserve the exact original JSON bytes, then add one LF. The intake preserves one receipt-named `.jsonl` file per accepted submission. The separate serialized exporter collects list rows into the shared `logbook.jsonl` described below. Do not reconstruct JSON from display columns or treat the list's CSV/Excel export as JSONL. A received list receipt and an exported file are distinct states; neither means ACORN admission, graduation, merge or publication.

# Project catalog acceptance

The catalog is seeded from the retained [October 25 response](_assets/solution-export/pa18-ornl-catalog-response.json), revision `03ff5836289447034f40f2a4898c155441e6cb66`. It is a recorded configuration snapshot, not a fresh repository scan. All 13 rows have `Title`, `ProjectId` and `SourceRevision`; display labels currently equal IDs. Future catalog maintenance is explicit until a separate synchronization path is accepted.

`ACORN SharePoint Pilot Catalog` is saved in Wohlgemuth, Jason (`221f44a6-1468-ea4e-b93d-cd52f1723079`), flow ID `6f8a4ab5-dd66-4904-9f5d-f307519ab4d5`. Its only external connector is standard SharePoint, using connection reference `shared-sharepointonl-4aa12f84-9554-41c4-a184-b21799bd4104` under the existing ORNL account. No credential values are stored here.

The native flow uses Power Apps V2, Get items, Select and Respond to a Power App or flow. Get items orders by Title, requests 1,001 entries and enables pagination to 1,001. The response text output `catalogjson` contains `{source: "sharepoint", projects: [{id, label}]}`. More than 1,000 results produces an empty catalog with `error: "catalog_limit"`; the app must treat that as failure. Overflow behavior is authored but not yet exercised with a large fixture.

Manual run `08584085728203907521416207681CU22` succeeded at 17:01:05 UTC on October 26. All four actions succeeded; the response contained all 13 expected IDs and labels. This proves owner-context connector execution, not an app-driven run or standard-user licensing acceptance.

![Shared project catalog](_assets/sharepoint/projects-list.png)

![Successful standard catalog flow](_assets/sharepoint/catalog-success.png)

# Intake implementation and acceptance

`ACORN SharePoint Pilot Intake` is saved in the same environment, flow ID `64679e9c-ae76-4263-b8fa-8832835516ca`. It uses the existing SharePoint and Teams connections and no GitLab connector. After Teams was added, the designer checker reported zero errors and eight connection-refresh advisories. The app binding was subsequently refreshed, republished and successfully exercised. The response is intentionally retained for confirmed receipt feedback.

October 26 owner-run acceptance used non-sensitive receipt `5e505b15-5f25-4ae7-8e13-7b4df6f6715e` with Unicode and escaped newlines:

| Case | Run ID | Observed result |
|---|---|---|
| New submission | `08584085716278869310025255830CU17` | One receipt and file, exact export readback, Exported status. Response metadata was subsequently repaired. |
| Exact replay | `08584085712650569052440212072CU08` | Receipt creation skipped; non-overwrite file creation failed as expected, existing file readback passed, response succeeded. |
| Same UUID, changed payload | `08584085711980621045990695644CU06` | Conflict branch succeeded; receipt/file creation and export update skipped. |
| Malformed envelope | `08584085711189995757802689639CU25` | Parse failed, persistence skipped, outer response succeeded. |

The [independent receipt readback](_assets/sharepoint/receipt-readback.json) confirmed exactly one matching row and byte-identical original payload after replay and conflict. Flow readback compared the JSONL bytes with the original UTF-8 envelope plus one LF; an independently downloaded file has not been verified. Secure response values were hidden in run history. The subsequent published-app tests below verified the successful receipt/export response; app-driven conflict and pending-export responses remain unaccepted. Preserved action-status evidence: [replay](_assets/sharepoint/replay-run-evidence.txt), [conflict](_assets/sharepoint/conflict-run-evidence.txt), [malformed input](_assets/sharepoint/invalid-run-evidence.txt).

Remaining acceptance includes catalog overflow, payload size boundaries, concurrent attempts, export-failure recovery, app-driven conflict/replay and standard-license participant access.

# Published-app acceptance and notifications

The published pilot loaded all 13 projects through the SharePoint catalog. App-driven run `08584085698401694139777770661CU04` created receipt `02b7231b-47e0-4034-b49b-0b6e594548e0`, confirmed its export, and displayed **Highlight saved · JSONL ready**. A full page reload restored the same UUID, confirmation and frozen form fields. Starting a new highlight cleared only the pilot recovery slot and reloaded the catalog. The [saved list envelope](_assets/sharepoint/app-submission.json) independently retains the submitted Unicode details. This is page-reload evidence; it does not repeat full browser-restart or denied-storage acceptance for the new app.

After adding Teams, the intake binding was refreshed and the pilot app republished. App-driven run `08584085694369117892624466851CU16` created receipt `64b836e0-016f-4493-9894-a6fe79acc8de`. Receipt creation, JSONL creation/readback, export status, response, and **Post SharePoint update to Teams** all succeeded. See [action outcomes](_assets/sharepoint/teams-app-run-evidence.txt) and [native Teams action](_assets/sharepoint/teams-action-native.json). This proves connector execution; it does not establish that a recipient read the message.

Manual exact replay of the original app envelope (`02b7231b-47e0-4034-b49b-0b6e594548e0`) passed in run `08584085692740963250031363046CU21`. Receipt creation was skipped; non-overwrite file creation failed as expected, existing-file readback and exact-export verification succeeded, and the response succeeded. **Post SharePoint update to Teams** was skipped. See [replay action outcomes](_assets/sharepoint/teams-replay-run-evidence.txt). This verifies the deployed notification gate without creating another receipt or sending another notice; it is not an app-driven retry test.

The formatted Teams notice goes to Jason through the existing connection's self-notification action. It includes the project, verified export link and receipt ID, with a TEST ONLY marker for test summaries. It runs after response handling and requires both a newly created receipt and a confirmed export, so receipt feedback is independent of notification delivery and ordinary replays skip delivery. Notification is best effort with automatic retry disabled. If export is first confirmed on a later retry, this initial-only notification gate skips the notice; durable delivery/recovery tracking is not implemented. No routine intake email was added. Existing milestone-email handling in the manual workflow is unchanged.

![Published SharePoint pilot confirmation](_assets/sharepoint/app-teams-success.png)

The reproducible [clipboard builder](_assets/sharepoint/build-intake.py) retains the existing envelope and caller validation, creates a unique list receipt, compares exact stored payload bytes, creates a receipt-named file without overwriting, and reads the file back before marking it exported. Its two generated clipboard actions are deployment source, not an importable Power Platform backup. Compose and Parse JSON retain the platform-supported secure-input settings. The native classic designer required `string(null)` for empty string expression arguments. Response fields require dynamic-field schema metadata; configure the response to run after all four scope outcomes in the designer because clipboard insertion resets that outer dependency.

The [active plan](index.md) assigns PA-19 storage/catalog acceptance, PA-20 validated intake and exact export, PA-21 isolated app binding/recovery, PA-22 intended-user access and manual handoff, PA-24 native backup/restore, and PA-25 any later cutover. Preserve GitLab pending receipts in the original app and never automatically reroute an unresolved submission. The SharePoint app uses its own browser storage key and reports received/exported independently of GitLab commit receipts.


# Combined logbook export

The SharePoint list is the collection source of truth. [Download the combined logbook](https://ornl.sharepoint.com/sites/ResearchEnablement/Sponsor%20Highlights%20JSONL%20Pilot/logbook.jsonl), rather than assembling individual receipt files. `ACORN SharePoint Pilot Logbook Export` (`51efa5f9-6e69-4d75-a53b-2c2965fd51bd`) runs every five minutes in Wohlgemuth, Jason with trigger concurrency one, using the existing standard SharePoint connection. It does not change either intake flow, app bindings, permissions or notifications.

Each run captures the highest list item ID, retrieves rows through that cutoff in ascending ID order, decodes the original payload, validates receipt/project identity and rejects literal newlines. It joins exact original JSON strings with LF and adds the final LF. It rebuilds the snapshot rather than appending concurrently. New arrivals after the cutoff are collected on the next run. Normal intake does not modify prior payloads; this is not a transactional snapshot against manual list edits or deletion.

Pagination requests 5,001 rows so the 5,000-row guard rejects overflow. A separate UTF-8 byte calculation caps the snapshot at 10 MiB. Read, decoding, validation or size failure stops publication and retains the prior file. Exact matching content skips the write. Changed content replaces logbook.jsonl and is read back for byte equality. Individual UUID files remain receipt evidence. The app's JSONL-ready message refers to its individual receipt file; allow the next scheduled refresh before downloading the collected logbook. Exporter failures are visible in run history; no new notification outbox was added.

Saved source: [exporter builder](_assets/sharepoint/build-logbook-export.py) and [clipboard action](_assets/sharepoint/Combined_logbook_export_LB.clipboard.json). These are not a native backup.

October 26 evidence: initial run `08584085656857537345648662769CU31` succeeded; repeat `08584085656693382083258906261CU24` succeeded with Write logbook skipped and exact readback/export receipt succeeded. [Repeat action evidence](_assets/sharepoint/combined-export-repeat-evidence.txt) is retained. The library visibly contained logbook.jsonl alongside the three test receipt files. Secure-data propagation hides the count receipt, so an independent downloaded row count is not claimed. The browser download attempt stalled and returned no verified local file. At that checkpoint, download-to-CLI and the broader matrix remained unaccepted. The later live demonstration below accepts the owner-context downloaded-file-to-CLI path; standard-user access and concurrent arrivals, empty/overflow/size/failure cases remain open in PA-23/PA-22. Test-only receipts must be excluded from real reports.

# October 26 live demonstration and local report

At the owner's request, three new explicitly synthetic highlights were submitted through the published SharePoint app. Each displayed **Highlight saved · JSONL ready**, and each intake run's **Post SharePoint update to Teams** action succeeded. No routine email was expected or sent by intake. The examples cover Data/Completed, Software/In progress and Collaboration/Upcoming, all routed to `acorn`.

| Example | Receipt | Intake run |
| --- | --- | --- |
| Geospatial data quality review | `dff4315e-8106-4eb8-83ad-44f6df767476` | `08584085557069877718061222716CU28` |
| Map validation dashboard | `67c0e548-ac0c-4a01-829c-9e4708b3cf1f` | `08584085556666191837902385414CU30` |
| Analyst walkthrough | `32baf0fa-96a7-43c0-8e80-cd71c26b4ca3` | `08584085556294438987943610816CU22` |

The combined exporter was manually triggered without changing its schedule or definition. Run `08584085556021667768976833698CU24` succeeded. The single-file browser download completed: [logbook.jsonl](_assets/sharepoint/demo-20261026/logbook.jsonl) contains six LF-terminated UTF-8 records, 3,413 bytes, SHA-256 `84eef36810338f5ebabb074d4e9993a09b9a910e38da82dc458f4344a18a09c7`. Its first 1,395 bytes exactly match the earlier three-row download. The three new rows were selected by their receipt IDs without reserializing their bytes; prior acceptance records were excluded from the report.

ACORN 0.5.1 staged and inspected the selected batch (`7f481b73c72c1a294ee0bfa64585ec604f19bbd84d237c79e7006cdc30652da8`): three accepted, zero rejected or replayed. The reviewed synthetic batch was applied with `--yes` only to a separate local demo index. Local snapshot `af48c2fc35150e2aa7766410941cc946d77a1b10` then generated the [three-page ACORN report](_assets/sharepoint/demo-20261026/sharepoint-demo-report.pdf) using `report --offline --dry-run --combine`, with the global `--offline` flag placed before `report`. The reporting window is October 26 inclusive through October 27 exclusive, UTC. All three PDF pages were rendered and visually checked; all three summaries/details are present. No GitLab publication, canonical index update, metadata graduation, Vergil processing or milestone email occurred.

[Machine-readable evidence and hashes](_assets/sharepoint/demo-20261026/evidence.json), [selected exact rows](_assets/sharepoint/demo-20261026/selected-demo.jsonl), [review diff](_assets/sharepoint/demo-20261026/inspect.txt), and [ingested demo index](_assets/sharepoint/demo-20261026/ingested.json) preserve the handoff. The local report proves owner-context download-to-CLI reporting. It does not establish standard-license user access or close PA-22/23. Independent comparison with every stored list payload, concurrent arrivals, empty/overflow/size/failure cases remain open. A bulk receipt ZIP download did not complete; the successful single-file download is the source used here.

![Collected JSONL and receipt files after three live submissions](_assets/sharepoint/demo-20261026/jsonl-library.png)
