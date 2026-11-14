# November 12 resource cleanup

Scope: the Wohlgemuth, Jason environment `221f44a6-1468-ea4e-b93d-cd52f1723079` and the task's Chrome tabs. This is a cleanup evidence record, not release or cutover acceptance.

## Deleted with final owner confirmation

| Resource | ID | Evidence |
|---|---|---|
| [PREMIUM] Highlights Intake app | `15ae605b-585b-49f5-a413-2b091f919a35` | Retired GitLab route; current native canvas downloaded from Studio. Its package lists the two premium flows below. |
| [PREMIUM] Highlights Intake - Project Catalog | `cd27888f-dd74-4675-9dd1-abc246ba56d5` | Off in the 30-flow inventory; no run-only sharing or associated apps shown on its details page. Used by the retired app's package. |
| [PREMIUM] Highlights Intake - GitLab Intake | `6039f419-420f-6922-46ef-04b00d2290f1` | Off; no run-only sharing or associated apps shown. Most recent visible run October 26; no run data for the last seven days. Used by the retired app's package. |
| [BETA] Seed Review Source Fixtures | `b83ff8ad-2ecf-44f1-9e4b-2d612bc76f78` | Off, no associated apps or run-only sharing. Two successful November 8 fixture runs. Fresh native package exported. |
| [BETA] Materialize Approval Batch Fixture | `ec17c0f4-f486-489b-b9b2-908e16e9b882` | Off, no associated apps or run-only sharing. Latest visible run November 9. Separate from participant intake materialization. Fresh native package exported. |

The owner confirmed “Delete all five listed resources” on November 12. All five were then deleted through their native confirmation dialogs. Power Apps reported Successfully deleted app and displayed only the two BETA apps and the operational Highlights Intake app. Power Automate's inventory fell from 30 to 26 flows, with all four approved flow names absent. The retained live services and batch/export utilities remained listed with their prior enabled/disabled states.

## Recovery evidence

The [artifact manifest](artifact-manifest.json) records hashes and ZIP-integrity checks for the fresh canvas and two fixture packages. Chrome blocked the fresh premium app-package download with `ERR_BLOCKED_BY_CLIENT`; no browser protection was bypassed. Studio's Download a copy succeeded for the current app. The [October 25 native package](../../../solution-export/SponsorHighlightsNativeBackup20261025_20261025185256.zip) remains available for the older premium app and both GitLab flows; its ZIP integrity was checked during this pass. That older flow archive is not claimed to be a fresh exact backup of the installed flows. Restore requires connection mapping and validation.

## Retained resources

### November 12 app recovery

At the owner's request, the fresh `premium-intake-before.msapp` was imported into the same environment and saved as `[PREMIUM] Highlights Intake`, replacement app ID `90dabac0-caf3-4f80-9ba6-6fae22e5408c`. Its source SHA-256 remains `6c186784e3486925622e2847b3972ae2ae1703eecb70dee39867165f8795ad24`. Studio's formula checker reported no errors. The replacement appears in the app inventory with Premium licensing. This is a backup import, not an original-ID undelete; original version history and sharing were not restored.

Version 6 is published and marked **Live** in Power Apps (November 12, 6:50 PM ET). [Replacement app](https://apps.gov.powerapps.us/play/e/221f44a6-1468-ea4e-b93d-cd52f1723079/a/90dabac0-caf3-4f80-9ba6-6fae22e5408c).

After the owner explicitly approved “Restore Both Supporting Flows,” the Power Automate Management connection was created and Microsoft's Restore Deleted Flow as Admin action recovered both original resources. Recovery helper `[BETA] Recover Premium Intake Dependencies` (`2319a31d-c322-4dbb-af0e-929eb22068cb`) succeeded at 6:34 PM ET, run `08584070804111222848045019116CU05`. The helper is now Off. The earlier automatic approval-review block was resolved by that explicit approval.

GitLab Intake retains runtime resource `6039f419-420f-6922-46ef-04b00d2290f1`, with workflow ID `b1109888-5aca-f111-a05c-001dd8062052`; its details showed On. Catalog recovery and activation also exposed runtime resource `814fdf7b-5260-08d1-cab6-cd1c3550e13b`, with Dataverse workflow ID `e66abd69-d9cf-f111-a05c-001dd8062052`. That resource showed On and recorded a successful catalog run at 6:48 PM ET (`08584070795924371911815682093CU09`). Retained connection references showed healthy status. The flows appeared in the default solution, with no custom solution membership reported.

Both app bindings were removed and re-added from the current environment. Catalog's single formula reference now uses `'[PREMIUM]HighlightsIntake-ProjectCatalog'.Run()`; GitLab Intake retained its existing `ACORNLogbookGitLabJSONLIntake(Draft)` alias and submission formula. App checker reported no formula errors before the final publish. The player briefly loaded all 13 project choices at 6:48 PM, but this did not remain working after the final version 6 publish.

**Recovery remains incomplete: the published app cannot currently load projects or accept submissions.** Live Monitor at 6:53 PM and 6:54 PM recorded HTTP 400, `WorkflowTriggerIsNotEnabled`, naming original catalog runtime `cd27888f-dd74-4675-9dd1-abc246ba56d5` and its `manual` trigger with state `Disabled`. The app still invoked that original runtime despite re-adding its binding. Direct Turn On reported On in the details UI but did not resolve the execution error. The additional catalog runtime's successful run does not establish that the final published app is operational.

Attempts to repair activation through the approved management connection stalled repeatedly in both Power Automate designers. Those helper edits were not saved or run and were discarded by closing the editor; the helper's last saved definition contains the two restore actions and remains Off. The remaining repair is to reconcile the catalog runtime and app binding, then verify project loading before submission validation. No new highlight was submitted and no GitLab data was written during recovery. Post-recovery writes, replay, and participant access remain unverified.

The original deletion inventory below is the checkpoint before this replacement app was created.

The final inventory contains three canvas apps and 26 flows. The two published BETA apps and their shared UI library are in use. The standard `Highlights Intake` app (`163f110f-7f8a-401c-894e-0786b2c018a4`) is the operational SharePoint form pending PA-41 cutover, not an unused duplicate.

The live intake/catalog/editing, review read/save, queued and background summary, invitation, coverage, and operational exporter services remain. Batch creation/materialization, approved-batch read, finalization, gated export, storage provisioning and access configuration remain for participant acceptance, deployment, and cutover. Off alone does not make those services obsolete. The separately protected summary experiments remain outside this cleanup.

Both custom unmanaged solutions remain nonempty and useful:

- ACORN SharePoint (`0fafd397-64d1-f111-a05c-001dd8062052`) contains the live BETA project catalog, operational SharePoint intake, and their SharePoint connection reference.
- ACORN Sponsor Highlights Intake (`b2f22320-f3cf-f111-a05c-001dd8062052`) contained the deleted premium set alongside the separate Callback Receipt Validation flow, Callback Receipt table, four AI Skill Config objects, and three connection references. The solution itself, callback data, and connection configuration were not deleted by this cleanup.

The Common Data Services Default Solution and Default Solution are system containers and remain untouched. No data table, connection, receipt, audit record, or participant batch was changed.

## Tabs

Closed stale intake player `1329061120` and stale reviewer player `1329061126`. The stale intake had an empty summary; the stale review showed the selected source item with its unchanged unresolved decision. Closed the temporary solution-inspection tab, incidental Microsoft 365 app-list tab, and both deletion-dialog tabs after completion. The final browser inventory contains only the two current BETA form tabs and the owner's unrelated Supercomputer Tour Request designer. Existing form drafts were not refreshed or edited.
