# PA-17 isolated native restore instructions

Checkpoint: October 25, 2026. Native app, flow, connector and callback/table archives are present; complete recovery remains unaccepted. Use the adjacent `pa17-native-backup-manifest.json` to verify archive hashes before upload. Never edit a native archive to change identifiers or bindings.

## Target and identity checks

Use the explicitly named PA17 test solutions in Wohlgemuth, Jason (`221f44a6-1468-ea4e-b93d-cd52f1723079`) or ORNL Personal Productivity (`Default-db3dbd43-4c4b-4544-9f8a-0553f9f5f25e`). Inspect the target environment and solution unique name before each import. Existing test component identities may be updated; operational identities must never be overwritten for testing. Do not use Publish all customizations.

`ACORNPA17CallbackBackup_1_0_0_1.zip` is a read-only backup of original callback/table identities. It is not the isolated test package. Do not import it into Jason's operational environment during these tests. Edited legacy derivative packages and their builder were removed; use the unchanged native test solution.

`SponsorHighlightsNativeBackup20261025_20261025185256.zip` is the fresh unchanged operational canvas/flow backup. It retains operational routes and delivery behavior; do not use it for test imports. Use v6 below for isolated restoration.

## Dependency order

1. Register the test connector first. This is already done in both environments; connector-only native version `1.0.0.1` was the historical bootstrap and has been removed. Do not downgrade the existing test solution.
2. Native `ACORNPA17RestoreReadinessTest_1_0_0_6.zip` has imported successfully into the existing ORNL **ACORN PA17 Restore Readiness Test** solution (October 25, 3:18–3:19 PM; activity `f8525c54-972a-46a7-a164-d751d8994483`). Do not repeat this accepted import. For future recovery, use the same isolated solution identity. Version `1.0.0.4` is already accepted in ORNL for the read-only catalog. Version `1.0.0.5` imported successfully October 25, 2:32–2:33 PM local display time; activity `e3ca7d29-526f-45e4-953b-16ec8117ebcb`. Do not repeat v5; v6 adds the native isolated canvas. Leave automatic activation unchecked. Restored manual replay and identity-conflict rejection passed. A fresh commit returned an explicit network-range 403; the intake was returned Off. Calling-app and fresh-write acceptance remain pending. Select the existing protected **ACORN PA18 ORNL TEST ONLY** connection for `new_PA18TESTONLYGitLabReference` in ORNL, or **ACORN PA17 GitLab TEST ONLY** in Jason. Map `new_PA17TESTONLYTeamsReference` to the authorized owner connection. Credentials are not in the package; the owner completes any credential prompt.
3. Import native `ACORNPA17CallbackRestoreTest_1_0_0_1.zip` as **ACORN PA17 Callback Restore Test**. It contains the new table and native callback plus dedicated Dataverse, Teams and Outlook references. Map only the named test references to existing authorized owner connections. The Teams test reference is shared by the two test packages, so retain the same mapping. ORNL import was attempted and failed because this account lacks `prvCreateEntity`; the table and flow were Unprocessed. Do not retry under unchanged privileges or substitute an operational table.
4. Verify `new_uniquewebhookid` is Active on `new_webhookid` in `new_pa17callbackreceipt`. Table rows are not included in the ZIP. Receipt data recovery requires a separate approved data backup; never claim solution import restores prior receipts.
5. Version 1.0.0.6 now includes the native PA17 canvas copy, bound to the isolated native intake and catalog. After import, verify `PA17SponsorHighlightPending` and aliases `PA17TESTONLYNativeIntake`/`PA17TESTONLYNativeCatalog`, with workflow entity IDs `9be4b2b3-98d0-f111-a05c-001dd8062052`/`3cb20a10-95d0-f111-a05c-001dd8062052`. Source and restored ORNL app catalogs both list 13 projects. ORNL app ID is `abd4b93f-3abc-43f1-82ae-1cc33e1f3527`; catalog run `08584086507609818994513589261CU31` passed. App submission acceptance remains. Do not replace operational associations or storage.

## Preflight before running test copies

Keep intake disabled until all checks pass. Inspect actual read/history inputs and commit body as well as connector routes: every file reference must use `.acorn/intake/acceptance/pa17-logbook.jsonl` in project `22464`, branch `main`. The schema enum alone is not an access boundary. Preserve disabled automatic commit retries and conditional-write checks. Confirm catalog custom code is enabled only for GetProjectCatalog, 6,901 bytes, SHA-256 `38d6b9ca805cb6e862521e343fb67afce4b7859fd99c431b754ee0a9f99725c0`.

Intake's notification condition and callback's Pending-email condition contain `equals(false, true)`. Inspect both after import. All test delivery actions must remain skipped. Callback Dataverse actions must use only `new_pa17callbackreceipts`; original `cr1d0_` table identifiers must not occur in that test flow. Check connection references and run-only bindings in the destination. Source `runtimeSource: invoker` is not proof that the restored app caller has an effective connection or license.

## Acceptance and evidence

In each target, capture exact component IDs, package hash, protected connection names, run IDs and timestamps. Run catalog, one fresh TEST ONLY intake submission, exact immutable replay and changed-payload conflict. Confirm the isolated GitLab file's exact line delta and immutable commit proof. Confirm no operational inbox changes and no notifications. Use the separate test app for calling-app acceptance; a manual flow run is not a substitute.

For callback storage, create one synthetic receipt, replay the same ID/payload, then change the payload under that ID. The first creates one row, replay succeeds without another row, and conflict terminates without altering the existing row. All Teams/email actions must be skipped. Jason passed these three storage tests; ORNL remains untested because table import was denied. Return idle test flows to Off when acceptance ends, without changing operational flows.

Native v6 target import and calling-app catalog acceptance are complete. A completed recovery still requires app-driven intake acceptance and callback/table restoration. Retain PA-17 until those results exist. This test package deliberately suppresses delivery and does not replace the original operational notification definition or authorize shared production deployment.
