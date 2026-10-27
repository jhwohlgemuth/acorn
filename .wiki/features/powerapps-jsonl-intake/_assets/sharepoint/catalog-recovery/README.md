# Catalog failure feedback acceptance — PA-28

Accepted October 27, 2026 in **Wohlgemuth, Jason**. This is an isolated, saved implementation. Operational installation belongs to PA-25; neither operational route was edited, published, disabled or given new permissions.

## Retained components

| Component | Identity and final state |
|---|---|
| Canvas app | `PA28 TEST ONLY Sponsor Highlights SharePoint`, `de4cfeaf-7944-4342-bd84-efc6ea99b3ef`; saved, unpublished |
| Read-only catalog | `PA28 TEST ONLY SharePoint Catalog`, workflow entity `a1e621ad-1ed2-f111-a05c-001dd8062052`; **Off** after testing |
| Bound intake | `PA20 TEST ONLY SharePoint Intake`, workflow entity `cad28673-a0d1-f111-a05c-001dd8062052`; remained **Off**, never invoked |
| Catalog data | Research Enablement site, PA19 TEST ONLY Projects list `40f2847e-3698-4ae6-b84b-7e42002c84eb`; filter `Title eq 'PA19 TEST ONLY 1'`, ordered by Title, top/pagination 1,001 |

The copied catalog's old seeding action was removed. The final catalog has only Power Apps V2, Get items, Select and Respond. All injected fixtures were removed or disabled before the final successful real read. No receipt, JSONL file, email or Teams message was created by these tests. Synthetic draft text was entered only in Studio preview.

## Implementation

[RefreshProjectsButton-onselect.fx](RefreshProjectsButton-onselect.fx) and [control-properties.json](control-properties.json) are the installation overlay, using the operational catalog alias. The test app substitutes `PA28TESTONLYSharePointCatalog` and binds its unchanged submission formula to the disabled PA20 intake. Do not copy that test intake binding to production.

The button clears old choices before every lookup, rejects malformed, empty, duplicate, non-string or oversized project records, and enables the selector only after a valid catalog loads. Failures leave an amber, persistent, screen-reader-announced message and an enabled **Retry Project List** button. Access denial has permission guidance; service failures have connection/retry guidance. Success clears the error and preserves **Load Project List** / **Refresh Project List** behavior. Draft fields and pending-submission guards are preserved.

The connector call is directly inside `IfError`, before assignment to the response variable. Wrapping an assignment followed by `true` hid the injected transport error and incorrectly reached the invalid-catalog path; the final formula fixes that behavior.

[catalog-response.clipboard.json](catalog-response.clipboard.json) contains the response action. It returns safe error codes without connector diagnostics. HTTP 401/403 becomes `access_denied`; other failed reads become `unavailable`; failed projection becomes `invalid_catalog`; overflow remains `catalog_limit`. Configure Respond to run after Select succeeds, fails, is skipped or times out. Clipboard insertion resets this dependency, so verify it explicitly. [Saved response readback](saved-response.json) and [run-after checkboxes](response-run-after.txt) preserve the deployed test configuration.

## Acceptance evidence

| Case | Method and result | Evidence |
|---|---|---|
| Real successful catalog read | Native manual run `08584084899589753669675586460CU31`; synthetic project returned | [Run](catalog-success-run.txt) |
| Connector action failure | Native Static Result on Get items: Failed / BadRequest; run `08584084896685240095212937575CU09`. Get items Failed, Select Skipped, Respond Succeeded. App showed transient guidance, enabled Retry and cleared the previously selected project | [Flow](connector-failure-run.txt), [app](app-connector-failure.txt), [screenshot](app-connector-failure.png) |
| Denied access | Injected `{source:"sharepoint",error:"access_denied",projects:[]}` response in the test button. Permission guidance persisted, Retry enabled, old selection cleared; unsent summary retained | [App](app-access-denied.txt), [screenshot](app-access-denied.png) |
| Transport failure / timeout handling | Injected a typed `ErrorKind.Network` result at the connector call. Connection guidance persisted, Retry enabled and selector disabled | [App](app-transport-timeout.txt), [screenshot](app-transport-timeout.png) |
| Actual unavailable trigger | After cleanup turned the PA28 catalog Off, the final real connector call failed and displayed connection guidance with Retry enabled, selector disabled and draft preserved | [App](app-disabled-flow.txt), [screenshot](app-disabled-flow.png) |
| Invalid response | Malformed JSON, projects object instead of array, numeric ID, duplicate ID and empty list all rejected; no stale choices enabled | [Malformed JSON](app-malformed-json.txt), [shape](app-wrong-shape.txt), [numeric ID](app-numeric-id.txt), [duplicate](app-duplicate-id.txt), [empty](app-empty-catalog.txt) |
| Successful retry on final formula | Real SharePoint run `08584084892369290985114926623CU16`, 213 ms. Error cleared, selector enabled, button returned to Refresh Project List, unsent summary retained | [App](app-final-retry.txt), [screenshot](app-final-retry.png), [run history](final-flow-runs.txt) |
| Cleanup | Catalog Off, intake Off, real connector formula restored | [Catalog](catalog-off.txt), [intake](intake-off.txt), [native controls](native-control-readback.json) |

Denial and transport timeout are controlled fault injections, not a new live permission revocation or a measured platform timeout. No account, connection, storage or network security setting was changed. The native failed-action test verifies the flow's error response; the injected app tests verify its display and recovery behavior. Existing participant-access acceptance remains separate.

To reproduce the transport fixture, temporarily replace only the test button's catalog call with `If(true, Error({Kind: ErrorKind.Network, Message: "PA28 TEST ONLY transport timeout"}), {catalogjson: ""})`. The record fallback preserves the response type. Restore the real call afterward. For native failed-action testing, use `{"status":"Failed","error":{"code":"BadRequest","message":"PA28 TEST ONLY connector failure"}}`, then disable Static Result and save before the successful retry.

## Native backup and later installation

[pa28-accepted.msapp](pa28-accepted.msapp) was downloaded through Studio after saving the final real-connection formula. SHA-256: `b938a3c4b027bbe7148a31be3cfd139c86cc840d6f68099e2f09f0cc66b24c1d`. Its seven changed control properties were compared with the overlay, allowing only the catalog alias substitution. [Native App Checker](native-app-checker.json) has zero results and successful execution. This native canvas backup does not include a separately importable catalog-flow package; the response source, native readback and exact test destinations above are the catalog handoff.

Under PA-25's separately authorized installation:

1. Back up the current published SharePoint app and catalog revision, including owner-backed run-only connection settings. Preserve the app ID, audience, storage key and operational intake binding.
2. Install only the safe response expression and its four run-after outcomes in the operational catalog. Retain its operational list, query, pagination and connection; never install the PA19 test filter or destination. Confirm no static results are enabled.
3. Apply the seven properties in `control-properties.json` to the operational form, using its actual catalog alias. Refresh the catalog binding if its response metadata changed. Preserve ProjectHelp's existing error-aware Text/AutoHeight properties and all submission/recovery formulas.
4. Save, check formulas, verify the real project list and a representative standard participant's retry, then publish the explicitly selected app revision. Reconfirm notification routing through the existing intake acceptance procedure; catalog acceptance alone does not prove notification delivery.
5. If verification fails, restore the previous app/catalog revisions without altering receipts or route selection. Keep both pilots enabled unless the owner separately authorizes a route cutover.

To reopen this isolated test, enable only the named PA28 read-only catalog and preview the saved PA28 app; keep PA20 intake Off. Turn the catalog Off again afterward.
