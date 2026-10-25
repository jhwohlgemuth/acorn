# Sponsor Highlights intake handoff

Plan assets now live in local `_assets/` folders. Historical PDFs, screenshots and execution records retain their original paths; the links and commands in this walkthrough use the current layout.

Updated 2026-10-24 after published Teams routing, milestone email acceptance, and the illustrated workflow update.

## Current notification walkthrough

[Open the current PowerApp/Power Automate walkthrough](notification-walkthrough.md). New confirmed intake saves, review-workspace creation, and no-change updates use Jason's Teams self-chat. Review requests, report delivery, and actionable failures keep email. Callback events remain owner-triggered until Vergil integration is accepted. The updated user/reviewer guide has 26 pages and developer guide has 27; their opening updates supersede older notification descriptions below.

Controlled LandScan submission `82a8dd58-8c60-4ec5-abf5-cb251ab9e033` added exactly one line at commit `03ff5836289447034f40f2a4898c155441e6cb66`, preserving the previous six lines. The app receipt and actual Teams notice matched. Routine callback exact replay skipped delivery; a labeled milestone email test passed. The inbox now has seven TEST ONLY entries. See the walkthrough for run IDs, recovery rules, and screenshot evidence.

## Next-week demo

The [demo packet](../manual-jsonl-ingestion-graduation/walkthrough.md) follows one real, synthetic personnel submission through the form receipt, immutable JSONL commit, local stage/inspect/apply, reviewed contributor graduation and a local PDF. [Open the illustrated walkthrough](../manual-jsonl-ingestion-graduation/_assets/demo-walkthrough.pdf) or [the resulting report](../manual-jsonl-ingestion-graduation/_assets/demo-report.pdf). Both guides retain that demo addendum; the current notification updates precede it.

Submission `4313fcfc-0201-441e-8bda-c77718a31dcf` appended exactly one line at `3090aa5714b4c6ff59c5dd0b6ecba999f918caa2`. At that rehearsal, the remote inbox contained six TEST ONLY lines. Exact replay reached Queued replay. The frozen sixth line and the full immutable snapshot are preserved byte-for-byte with SHA-256 hashes. The [isolated failed-commit test](write-failure-acceptance.md) also verified a single unresolved response with success confirmation skipped. All eleven local rehearsal steps passed; `make lint` passed and `make test` passed 2,670 tests (17 skipped). These results do not authorize production handback or establish reliable cloud egress.

The [rehearsal script](../manual-jsonl-ingestion-graduation/_assets/rehearse.py) creates a new disposable directory and uses the native Yes / No buttons for separate intake and graduation approval. No is the default. Unattended intake apply requires `--yes`; graduation uses `run --index` with reviewed IDs and their digest, or `--force` to accept all eligible promotions. `--dry-run` previews and `--interactive` selects candidates before confirming the final write. It refuses to overwrite an existing run. The recorded-source fallback runs without Vergil or cloud access. Earlier screenshots and test counts below are dated evidence from before this sixth submission.

## Published visual polish

The title is **Sponsor Highlights**, with a 64-pixel text box so descenders remain visible. The header is 208 pixels tall and the subtitle starts at 144 pixels. Subsections use `01 // HIGHLIGHT DETAILS` and `02 // THE DEVELOPMENT`; character counters retain a single slash. The layout has 56 pixels of bottom padding. The submit footer includes additional spacing and the attribution “ACORN · Oak Ridge National Laboratory” and “App owner: Jason Wohlgemuth.”

Desktop and 360 × 800 phone previews were visually checked. Studio formula and accessibility checks reported **No errors found**. The saved app was published in the existing pilot environment; after the player's new-version Refresh prompt, the live title, subsection labels, and footer were verified. No highlight was submitted during this visual check. A structural comparison of the before/after layout found only these presentation changes and the attribution label; submission formulas are unchanged.

Current layout source: `_assets/final-intake.pa.yaml`. Pre-change source: `_assets/before-visual-polish.pa.yaml`. Screenshots: `_assets/visual-polish-published-header.png`, `_assets/visual-polish-published-footer.png`, `_assets/visual-polish-phone-top.png`, and `_assets/visual-polish-phone-bottom.png`. The PDF guides retain their earlier screenshots as dated evidence.

## Deployed pilot

- App: `15ae605b-585b-49f5-a413-2b091f919a35`, environment `221f44a6-1468-ea4e-b93d-cd52f1723079` (the user's developer environment).
- [Open Sponsor Highlights](https://apps.gov.powerapps.us/play/e/221f44a6-1468-ea4e-b93d-cd52f1723079/a/15ae605b-585b-49f5-a413-2b091f919a35).
- GitLab project `22464`, `main`, `.acorn/intake/logbook.jsonl`, as selected by the user.
- Intake flow `ff4a134b-5fd3-4465-ab19-0325c2f2bf1b`, **ACORN Logbook GitLab JSONL Intake (Draft)**, is On. The existing name is retained to preserve the canvas binding; it now contains the saved `Persist submission v3` scope.
- Catalog flow `cd27888f-dd74-4675-9dd1-abc246ba56d5`, **ACORN Project Catalog**, is On.
- Both use the user's existing protected **ACORN GitLab Logbook** API-key connection. No credential value was read, copied, or embedded in source or the app.

The ORNL logo, green palette, contour background, responsive form, required project selection, schema-aligned fields, submit feedback, and browser recovery are published. The unused notification-only flow was removed from the app without deleting that flow. This publication does not configure Vergil, admission authorization, review, reporting, or callbacks.

## Submission behavior

Submit freezes one compact envelope, UUID, selected project ID, submission timestamp, and submitter. It stores that exact envelope with `SaveData` before invoking the flow. Storage failure prevents sending. The user explicitly chose local browser storage for non-sensitive highlights; the form states that limit.

While submitting, fields and the submit button are disabled. A matching `queued` receipt with the original UUID and a 40-character commit SHA opens one persistent result panel with receipt details. The panel replaces stacked native notifications; an old failure cannot remain beside a newer success. Back to highlight returns to the preserved fields and inline status. The message distinguishes saving to the logbook from ACORN review. Rejected input may be edited; conflicting identity requires owner review. Unknown, mismatched, or failed responses retain the same envelope for retry. A new highlight is available only after confirmed persistence or explicit rejection.

Startup uses `LoadData`, verifies the stored owner and UUID, restores form values, and retains the original payload. Confirmed receipts survive reload. New highlight clears the browser record before resetting the form. Storage uses one pending slot per app/browser; use one app tab at a time. Browser-data clearing or another browser/device does not preserve that local slot. The GitLab commit remains the durable persistence receipt.

Microsoft documents [SaveData/LoadData](https://learn.microsoft.com/en-us/power-platform/power-fx/reference/function-savedata-loaddata) web-storage limits and the lack of encryption. Formula-level error management and web storage were already enabled. Live storage testing used the published player, not Studio preview.

## Project catalog and intake

The catalog reads one immutable repository revision. A present `.acorn.jsonc` `folders` property is authoritative, including an empty array. It supports ACRE canonical string/object IDs, JSONC comments and trailing commas. Without that property, it discovers parent folders containing `index.json` or `index.yaml`, deduplicating IDs. Root indexes and symlinks are excluded. Invalid configuration and denied reads stop without fallback. Bounds are 10,000 tree entries and a 1 MiB configuration file. Aliases do not create duplicate choices. Catalog membership is separate from ACORN admission authorization.

The original pilot configuration contained `.acorn.jsonc` with `schemaVersion: "2.0.0"` and `folders: ["acorn"]`, in commit `57661ee2dc680b7d18fedefd294c42c20ba78eba`. The actual `acorn` choice loaded automatically in the published app and was selected for the successful submission. No members, destinations, or service permissions were added.

On October 24, commit `1ecd6f78` imported 12 NGA-sponsored project folders from NSSD and expanded the authoritative folder list to 13 IDs, including the existing `acorn` option. The published app listed all 13; the notification walkthrough used `landscan`. The pilot repository `NGA-PROJECTS.md` records the source revision and selection.

Intake validates the strict envelope and authenticated caller, then reconciles the submission marker against up to 100 file-history commits. Replay uses immutable commit/parent evidence and bypasses current catalog membership. New submissions require exactly one current catalog match. The saved implementation folds this condition into existing validation to stay within Power Automate's eight-level nesting limit.

A fresh append uses one conditional GitLab create/update action with automatic write retries disabled, preserves existing bytes, and verifies the immutable resulting file before returning `queued`. Malformed or oversized content, unsafe file tails, conflicts, and uncertain results fail without claiming persistence. Failure/timeout/skipped fallback is explicitly configured after `Persist submission v3`. Response properties consumed by the app are lowercase `commitsha`, `disposition`, `message`, and `submissionid`.

## Verification on 2026-10-24

- Published app automatically loaded the configured `acorn` project; required fields gated submission.
- Synthetic canvas submission `ed6303e5-f09e-4953-bae8-39af1263b4b8` succeeded in flow run `08584087350097179517237018651CU22`.
- [Commit 2618af3f](https://code.ornl.gov/research-enablement/experiments/power-automate/-/commit/2618af3fcfa6d39d5b76598fb5e5dd3854e5be58) adds exactly one JSONL line and removes none, preserving the two earlier test lines. The rendered line matches the selected project, entered fields, flat details array, UUID, and caller. Frozen bytes are in `_assets/test-published-canvas-submission.json`.
- The app showed a persistent green confirmation, the same UUID, and commit prefix `2618af3fcfa6`. Reload restored all entered fields and the same confirmed receipt. New highlight cleared the browser slot and reset fields.
- Exact replay of that frozen canvas envelope in run `08584087347859041073787136133CU02` reached **Queued replay** after immutable comparison; the new-append branch was skipped.
- Final formula and accessibility checks had zero issues. Static labels/images have empty OnSelect formulas and stay outside keyboard tab order; interactive buttons have visible focus. Desktop and 360-pixel phone previews show wrapped helper text, reachable controls, and no oversized blank form area.
- The final app was published again after accessibility fixes and removal of the unused notification flow. The published player was reloaded and visually verified. A historical Studio MissingConnectionReference runtime notice remains from before the successful live invocation; the live canvas call is the binding evidence.

## Interrupted-response retry on 2026-10-24

The requested retry successfully loaded the `acorn` catalog through the unchanged connection. A new TEST ONLY canvas submission was interrupted while the app displayed Saving, before its response. Reload and closing/reopening the app tab both restored every entered field and original receipt ID `d899a9c1-b512-498f-a3bf-4a4e11383050`; the app offered Retry submission and kept fields locked.

Original run `08584087341420123404672511978CU21` appended exactly one line in [commit 904ff8d6](https://code.ornl.gov/research-enablement/experiments/power-automate/-/commit/904ff8d65180a14cec511aba832d94c7fe25e470), preserving all three earlier records. Its Queued new response failed because the client connection had closed, as intended by the interruption test. The frozen envelope retained submission timestamp `2026-10-24T20:12:23.285Z`, project `acorn`, caller, and two details.

Recovery run `08584087341254266835972382123CU20` reached List submission history, which returned HTTP 403 with the explicit body: “Access from your IP address is currently restricted due to security controls applied to your network range.” The fallback returned unresolved, and the app correctly kept the pending payload instead of showing success or creating another identity. GitLab history showed only one commit for this ID. A subsequent user-requested recovery succeeded in run `08584087336848872984322544930CU21` at approximately `2026-10-24T20:20Z`. It returned the original UUID and commit `904ff8d65180a14cec511aba832d94c7fe25e470`; GitLab history had no additional commit. The browser now holds the confirmed receipt. Interrupted-response reconciliation has passed. Closing/reopening the app tab is not a full Chrome-process restart test.

Evidence: `_assets/interrupted-pending-recovered.png`, `_assets/interrupted-append-commit.png`, `_assets/recovery-history-network-denial.png`, `_assets/interrupted-recovery-pending.png`, and `_assets/interrupted-recovery-confirmed.png`. The last image exposed overlapping native failure/success banners; `_assets/published-result-panel.png` verifies the subsequent repair. Formula and accessibility checks reported no errors. The published player restored the confirmed receipt in the new result panel; Back to highlight restored the original locked fields and matching receipt without another flow invocation.

## Additional acceptance evidence

| Case | Run ID | Outcome |
| --- | --- | --- |
| Empty details array | `08584087336482307064368817185CU00` | Parse rejected `items: []` before write. The contract requires `null` or at least one item. Original payload preserved as `_assets/test-empty-items-rejection.json`. |
| Unavailable project | `08584087335723542965847541865CU31` | Structurally valid `test-unavailable-project` with `items: null` reached catalog membership rejection. Read failed response succeeded; append path skipped. No new commit. Payload: `_assets/test-unavailable-project.json`. |
| Over-limit fields | `08584087335388595989820899070CU30` | Initial main read returned explicit network-range HTTP 403. Persistence scope skipped. This does **not** validate the field boundaries. Payload: `_assets/test-field-boundaries.json`. |
| Concurrent writer A | `08584087334930010569600460127CU07` | Main read returned explicit network-range HTTP 403; persistence skipped. UUID `146b840c-52b2-4d3c-a257-df6e507f808a`. |
| Concurrent writer B | `08584087334929462355064561859CU25` | Main, history, catalog and inbox reads succeeded; Commit append returned the same explicit network-range HTTP 403. Unresolved write response succeeded, immutable verification skipped. UUID `7a899477-da02-47bb-a02b-798a9c97fe7d`. |

The two writer invocations were launched at `2026-10-24T20:23:12Z`. Frozen payloads are in `_assets/test-concurrent-writers.json`. No concurrent-writer commit appeared: history still ended at `904ff8d6`. The pair is blocked acceptance, not a successful concurrency test. Screenshots: `_assets/unavailable-project-rejection.png`, `_assets/field-boundaries-network-denial.png`, `_assets/concurrent-writer-network-denial.png`, `_assets/concurrent-append-network-denial.png`.

## Network access handoff

The explicit upstream error is: “Access from your IP address is currently restricted due to security controls applied to your network range.” This occurred on both reads and the commit endpoint. Successful intervening calls used the same connection and route; they do not establish reliable access. Earlier catalog-only failures hid their upstream body, so their precise cause remains unproven.

The ORNL GitLab/network administrator needs to correlate successful and denied requests in server/access logs and establish an approved connector egress route. The agent has not sent this handoff to anyone or changed credentials, routing, permissions, or network controls.

- Source: Power Platform GCC developer environment `221f44a6-1468-ea4e-b93d-cd52f1723079`; intake flow `ff4a134b-5fd3-4465-ab19-0325c2f2bf1b`; custom connector **ACORN GitLab Logbook**.
- Destination: `code.ornl.gov` HTTPS 443; GitLab project `22464`, approved branch `main`, file `.acorn/intake/logbook.jsonl`. Required calls include branch, file/history/catalog reads and repository commits.
- Compare successful recovery at about `20:20Z` and unavailable-project reads at `20:21:53Z` with the failed `20:12:40Z` history read, `20:22:26Z` main read, and `20:23:12Z` concurrent requests above, all on 2026-10-24. Record actual source IPs and which control denied each request.
- Review the currently published GCC connector service-tag coverage against the actual environment's egress. Microsoft lists `AzureConnectors.USGovArizona`, `AzureConnectors.USGovIowa`, `AzureConnectors.USGovTexas`, and `AzureConnectors.USGovVirginia` for GCC. Use current official cloud ranges and the administrator's approved scope; do not broadly allow all Azure or assume one successful IP covers connector traffic. See [Microsoft connector outbound IP guidance](https://learn.microsoft.com/en-us/connectors/common/outbound-ip-addresses) and [Power Automate IP configuration](https://learn.microsoft.com/en-us/power-automate/ip-address-configuration).
- After the approved change, verify repeated catalog/history reads and a controlled append/replay, then rerun the frozen boundary/concurrent-writer cases. Preserve the no-automatic-write-retry rule and conditional append behavior. Do not add repeated 403 retries to mask the denial.

## Deployment prerequisites

The user named **Jason Wohlgemuth** as operational owner on 2026-10-24 and subsequently created the separate intake and Vergil project access tokens. The pilot Dataverse callback receipt table, active unique delivery-ID key, and owner-only manual notification flow are deployed; independent claim, send, duplicate, conflict, and uncertain-send tests pass. External endpoint authentication remains outstanding. The user confirmed on 2026-10-24 that the pilot stays in Wohlgemuth, Jason; future shared-environment migration is tracked separately in PA-12. Jason Wohlgemuth is the initial notification recipient, with a configurable destination for a later group channel. ORNL Personal Productivity is the tenant's default environment; shared environment access does not automatically share the app or establish approval for this workload.

Read-only administration checks found:

- The flow is personally owned by Jason. The environment picker permits building in **Wohlgemuth, Jason** and **ORNL Personal Productivity**. **Enterprise Test** and **Enterprise Prod** are listed under other environments.
- The Power Platform admin center lists one administered environment: **Wohlgemuth, Jason**, ID above, Developer, Ready, Dataverse Yes, Managed Yes, GCC, Citizen Developers group. This does not prove an approved shared deployment or rights to Enterprise Test/Prod. Evidence: `_assets/admin-environment-scope.png`.
- GitLab project `22464` now has two verified active project access tokens: `acorn-vergil-bot` (token ID `9143`) and `acorn-powerautomate-intake` (replacement token ID `9145`; original `9144` revoked). Both show Maintainer role and only `api` scope, expiring `2026-11-23T00:00:00Z` (November 22 at 7 PM Eastern). The intake replacement now shows last use at 2026-10-24 5:07:10 PM Eastern, matching the verification run; Vergil still shows Never. No secret values were opened. The earlier separate service-account inventory was empty; project tokens create their own bot identities. Evidence: `../vergil-gitlab-jsonl-automation/_assets/bot-tokens-created.png`.

The user rotated the lost intake token and reported “saved and both updated” on 2026-10-24. The replacement `9145` is verified active and original `9144` revoked; Vergil token `9143` is unaffected. No secret values were inspected. Evidence: `_assets/intake-rotation-verified.png`.

Both protected Power Automate updates are now verified in the same prepared Chrome connections. `acorn-powerautomate-intake` (`shared-acorn-20gitla-aef5b2c7-15ae-44da-afa0-00f8aecefdf0`) shows Connected and modified `2026-10-24T21:06:13Z`; `acorn-powerautomate-intake-catalog` (`5a113b098bb44a8fa7136bea1ec39e14`) shows Connected and modified `2026-10-24T21:06:10Z`. The original connection IDs and flow bindings were preserved. The intake flow references the first connection; catalog references the second. GitLab replacement token `9145` records use at `2026-10-24T21:07:10Z`, matching the intake verification start. No token values were inspected. Evidence: `_assets/updated-intake-connections-verified.png`, `_assets/replacement-intake-token-used.png`. Vergil's protected runtime binding remains unverified.

Exact replay run `08584087308550066067194637333CU26` at `2026-10-24T21:07:10Z` succeeded with the frozen `_assets/test-published-canvas-submission.json` payload. Main read, immutable prior/parent reads, exact prior-byte comparison, and **Queued replay** succeeded; the new-submission branch and outer fallback were skipped. Protected response bodies remained hidden. Catalog run `08584087308293753494808200795CU08` at `2026-10-24T21:07:36Z`, triggered from the published app, returned status 200 with project `acorn`, source `config`, and unchanged revision `904ff8d65180a14cec511aba832d94c7fe25e470`. These checks verify the updated protected connections and duplicate-free replay behavior; they do not establish reliable write egress or resolve prior network denials. Evidence: `_assets/replacement-token-exact-replay.png`, `_assets/replacement-token-catalog-success.png`.

One read-only catalog verification, run `08584087318069468450593089204CU28` at `2026-10-24T20:51:19Z`, succeeded and returned status 200 with `projects: [{"id":"acorn","label":"acorn"}]`, source `config`, revision `904ff8d65180a14cec511aba832d94c7fe25e470`. This confirms that invocation's catalog behavior, not which token was used or reliable cloud egress. This earlier read preceded the now-verified connection changes; later verification is recorded above. Evidence: `_assets/catalog-after-reported-credential-update.png`.

Remaining setup requires licensing/sharing decisions, the protected Vergil runtime binding, callback authentication and end-to-end sender recovery acceptance, and Vergil deployment authority. The current pilot environment is confirmed; shared migration is PA-12. New secret entry must be completed by the user in the protected connection/store UI, never pasted into chat or app variables.

The current ACORN callback sender has no Entra bearer token. An OAuth-only Power Automate HTTP trigger therefore requires an approved intermediary or a separately scoped sender-authentication change. The legacy Anyone/SAS option needs explicit tenant approval. The GCC audience documented by Microsoft is `https://gov.service.flow.microsoft.us/`. See [HTTP trigger authentication](https://learn.microsoft.com/en-us/power-automate/oauth-authentication). A durable record alone cannot guarantee exactly one email/Teams effect after an uncertain connector send; the owner must provide reconciliation for that outcome. Protect payload-bearing actions using [secure inputs and outputs](https://learn.microsoft.com/en-us/power-automate/guidance/coding-guidelines/use-secure-inputs-outputs-triggers).

## Independent form acceptance — 2026-10-24

Eight published-player checks passed without clicking Submit: 1,201 summary characters disabled submission while 1,200 enabled it; 21 nonempty detail lines disabled submission while 20 enabled it; a 1,001-character detail disabled submission while 1,000 enabled it; blank detail lines were ignored; and a whitespace-only summary disabled submission. Pasting can exceed the text input's configured maximum, so the separate submit eligibility check is necessary and was verified. These are client-side checks, not acceptance of the flow's server-side boundaries. Evidence: `_assets/published-form-boundary-evidence.json`.

The recovered-project help text was corrected to identify the fixed project instead of asking the user to load projects. The change was saved and published in the existing pilot environment. Both app flow bindings were refreshed successfully, then saved and published. The local `_assets/final-intake.pa.yaml` reflects the help-text correction; Ruby/Psych parsing passed and the standalone submit/restore formulas still match the YAML. Studio reported no formula errors before publication. A stale `MissingConnectionReference` warning for ACORN GitLab Logbook (correlation `03902013-cc4e-4fdb-8fa8-cb5a7ef8fc0f`) disappeared after refreshing both bindings and opening a clean editor session: Runtime reported **No errors found**. Formula and accessibility checks also reported **No errors found**. Studio briefly reopened to a blank panel; Home → Edit loaded a replacement session, and Override reclaimed editing control from the stalled, already-saved tab. The working replacement editor is preserved. Evidence: `_assets/refreshed-bindings-no-runtime-errors.png`. This clears the observed Studio warning, but does not verify which protected token each connection uses. The published app loaded its catalog successfully during the test and after reload.

Studio preview exercised the actual recovery-unavailable branch: one “Your highlight needs attention” panel appeared, Back to highlight returned to the form, the inline explanation remained, and Submit was disabled. No app-storage settings were changed and no submission was attempted. This is evidence for error-panel rendering and the preview recovery guard; published-browser storage denial, including SaveData failure after form preparation, remains untested. Evidence: `_assets/storage-unavailable-preview-feedback.png`, `_assets/storage-unavailable-submit-disabled.txt`.

New highlight cleared the prior confirmed synthetic receipt from browser storage before testing. Reload discarded the unsent synthetic field values and left a fresh form; no new JSONL record was submitted. The four prior TEST ONLY records and immutable receipt evidence remain in GitLab. Evidence: `_assets/published-form-after-validation.png`. The two credential handoffs and token pages remain open; no full Chrome-process restart was attempted while those pages were needed.

## Updated-connection boundary check — 2026-10-24

Run `08584087307572974429473214032CU20` at `2026-10-24T21:08:48Z` used the unchanged frozen `_assets/test-field-boundaries.json`: summary length 1,201, 21 details, and one detail of length 1,001. Main access succeeded; Parse submission failed and every dependent envelope/history/write branch was skipped. The outer unresolved fallback succeeded. The combined-invalid record was therefore stopped before any append. This is a no-write validation pass, not a `rejected` receipt assertion, and not proof of each individual server boundary or exact-limit acceptance. Individual and exact-limit server cases remain. Evidence: `_assets/replacement-token-boundary-no-write.png`.

## Validation-error history limitation

The updated-connection boundary run explicitly reported all three violations: `entry.summary` maxLength 1200, `entry.items` maxItems 20, and `entry.items[0]` maxLength 1000. Despite Parse submission having `runtimeConfiguration.secureData.properties: ["inputs"]` and the inputs being hidden, its validation-error output included the synthetic summary/detail values. No credential value was exposed or read. Do not claim blanket payload redaction for failed validation. Keep the user-approved non-sensitive-only scope and include this behavior in the operator handoff; handling sensitive highlights would require a separate verified change.

Microsoft's [workflow secure-history documentation](https://learn.microsoft.com/en-us/azure/logic-apps/set-up-security-permissions#considerations-for-securing-inputs-and-outputs) documents that Parse JSON supports Secure Inputs rather than a separate Secure Outputs toggle and describes hidden normal outputs. That does not override the observed failure output in this GCC flow. No unsupported protection flags were inserted, no strict schema checks were removed, and no history was deleted.

## Remaining acceptance

Interrupted-response reconciliation and unavailable-project rejection now pass. Full Chrome-process restart, storage denial, malformed-tail/size boundaries, concurrent creators/writers, and append-versus-cleanup acceptance remain. Reload and app-tab closure are already proven; they are not a full browser restart. PA-03/04 and Vergil admission/cleanup remain separate gates for full PA-05 acceptance. All five current inbox records are **TEST ONLY** and must be excluded from sponsor reporting.

## Operator recovery

The primary flow owner is Jason Wohlgemuth. If the app reports an identity conflict or repeated unresolved outcome, retain the pending browser record and receipt ID. Inspect flow history and GitLab file history for `acorn-submission:<UUID>`, including beyond the 100-commit automatic bound. Compare the immutable commit's file with its parent and the exact pending envelope. Current inbox absence is not proof of failure because cleanup may already have occurred. Do not generate a fresh UUID or clear pending data merely to get past an uncertain result. Resume the same payload only after resolving access or proving the prior outcome; archive the immutable evidence before operator-led reconciliation.

## Reproducible deployment files

- `_assets/final-intake.pa.yaml`: final published form controls, including ORNL assets and all behavior formulas.
- `_assets/submission-result-panel.pa.yaml`: standalone result panel, included in the final form source.
- `_assets/final-screen-onvisible.fx`: `Select(RestoreDraftButton)` on `SubmitEntryScreen.OnVisible`.
- `_assets/submit-intake-onselect.fx`, `_assets/restore-draft-onselect.fx`, `_assets/edit-intake-onselect.fx`: standalone behavior sources.
- `_assets/final-intake-scope.clipboard.json`: saved v3 scope; import via classic flow designer My clipboard.
- `_assets/final-intake-fallback.clipboard.json`: fallback template bound to v3. After import, explicitly bind its run-after to the current persistence scope for **Failed, TimedOut, Skipped**, excluding Succeeded; clipboard import can reset this configuration.
- `_assets/before-final-intake-scope.clipboard.json`, `_assets/submission-feedback.pa.yaml`, `_assets/original-screen.pa.yaml`: backups, not current deployment sources.
- `_assets/gitlab-logbook-connector.swagger.json`, `_assets/project-catalog.swagger.yaml`, `_assets/project-catalog.cs`: connector definition and server-side code. Custom code is enabled only for GetProjectCatalog; Swagger import alone does not install it.
- `_assets/catalog-response.clipboard.json`: catalog flow response template.
- `_assets/catalog-tests/`: thirteen focused C# checks previously passed; command `dotnet run --project .wiki/features/powerapps-jsonl-intake/_assets/catalog-tests/Test.csproj` with .NET 10.
- `_assets/published-submit-success.png`, `_assets/published-append-commit.png`, `_assets/published-replay-run.png`, `_assets/final-phone-top.png`, `_assets/final-phone-bottom.png`, `_assets/catalog-forbidden.png`: current evidence.

## Earlier controlled flow tests

| Test | Run ID | Verified outcome |
| --- | --- | --- |
| Initial missing input | `08584087425497328260210420375CU16` | GitLab branch read succeeded; Parse submission failed on null input. No write. Exposed missing trigger input and incorrect fallback condition. |
| Empty object after repair | `08584087424105151131659569850CU27` | Parse submission rejected `{}`; downstream validation/write path skipped; Unresolved failure response succeeded. |
| Synthetic first append | `08584087423395796594926927709CU06` | Envelope and caller accepted; missing inbox handled; commit and immutable readback succeeded; Queued new response ran. |
| Exact envelope replay | `08584087422402084189649029248CU31` | Prior commit and snapshot reconciled against the parent; Queued replay response ran; append branch skipped. |
| Same ID, changed summary | `08584087421518637877863055922CU16` | List submission history returned HTTP 403 with an explicit network-range restriction. No write or conflict evaluation; Unresolved failure response ran. The later fresh invocation below resolved this test. |
| Same conflict payload, fresh invocation | `08584087413874801527655543259CU27` | History and prior snapshot reads succeeded; Identity conflict response ran; replay and append branches skipped. No connector, endpoint, credential, or access-control changes. |
| Mismatched submitter | `08584087413087966035112792490CU15` | Valid structure with `mismatched-submitter@example.invalid` reached Rejected envelope; history and append skipped. Main-branch preflight read still ran. GitLab main remained at d9c0ac3a after both rejection tests. |
| Existing-file append | `08584087412476317262159789449CU04` | Current inbox read, conditional update, immutable readback, and Queued new succeeded. Commit 4acdee09 adds exactly one line and preserves the original line. |


Run URLs use `https://make.gov.powerautomate.us/environments/221f44a6-1468-ea4e-b93d-cd52f1723079/flows/ff4a134b-5fd3-4465-ab19-0325c2f2bf1b/runs/<RUN-ID>?v3=false`.

## Independent completion checkpoint — 2026-10-24

Fresh published-player submission `3b2cf21e-73b3-45fa-90aa-5309176aab3b` succeeded at `2026-10-24T21:33:16Z` in run `08584087292885127177206211553CU30`. The replacement intake bot authored commit `7dd33b6d6094ab4d0b2ce13d8c58296a210064d5`, adding exactly one line and removing none, preserving all four prior test records. The player showed one Highlight saved panel and the matching commit receipt. Frozen persisted bytes: `_assets/test-replacement-fresh-append.json`. Exact manual replay run `08584087292123422329137363598CU01` at `21:34:33Z` passed immutable prior/parent comparison and Queued replay, skipping the new-submission and fallback paths. The run-history Resubmit button did not produce an observable new run; verification used the manual Run form with frozen bytes instead. Evidence: `_assets/replacement-fresh-append-confirmed.png`, `_assets/replacement-fresh-replay.png`, and `_assets/replacement-fresh-replay-evidence.txt`. Five current inbox records are TEST ONLY. This proves replacement-credential writes and replay, but does not establish the cause or permanent resolution of earlier network denials.

At `2026-10-24T21:35:59Z`, both frozen concurrent-writer payloads were launched together. Runs `08584087291259499856793792419CU11` and `08584087291259577391911943458CU24` passed main/history reads, schema, caller and envelope checks, then Load project catalog returned 403 Forbidden with “Could not read the project repository.” Inbox read/write were skipped; Unresolved failure succeeded. The catalog wrapper does not expose the upstream denial body, so these runs cannot individually establish a network-range cause. No concurrent append occurred and repository head remained `7dd33b6d6094ab4d0b2ce13d8c58296a210064d5`. Evidence: `_assets/concurrency-updated-1329059072.txt` and `_assets/concurrency-updated-1329059075.txt`. Further denied-route retries stopped; no access controls were changed.

### Single write-failure response repair

The existing intake flow was saved after removing only the nested `Unresolved_write` response. A failed/timed-out commit now reaches the existing outer unresolved fallback; success-only immutable readback remains attached directly to `Commit_append`. This removes the previously observed double-response path without adding write retries. Flow checker reported zero errors and zero warnings. The saved source is `_assets/final-intake-scope.clipboard.json`; the prior source is `_assets/before-single-fallback-scope.clipboard.json`. Evidence: `_assets/single-fallback-flow-checker.png` and `_assets/single-fallback-readback-dependency.png`. A live failed-commit test of the repaired version remains outstanding because catalog access is currently denied.

### Solution packaging checkpoint

Created `ACORN Sponsor Highlights Intake` (`ACORNSponsorHighlightsIntake`, version `1.0.0.0`), solution ID `b2f22320-f3cf-f111-a05c-001dd8062052`, in the existing pilot environment. Existing Sponsor Highlights, JSONL Intake and Project Catalog components were added with their identities preserved. Adding the existing custom connector from Outside Dataverse stalled with a portal TypeError at `2026-10-24T21:38:35Z`; the connector has not been added. The solution subsequently gained both connection references and the callback receipt table, columns, and unique key. Diagnostic evidence: `_assets/solution-connector-add-error.json`. This is not yet a complete export. Microsoft documents a connector-first separate-solution import order and requires custom connectors to be solution-aware before references can be exported: [custom connectors in solutions](https://learn.microsoft.com/en-us/connectors/custom-connectors/customconnectorssolutions).


### Isolated live server boundaries

After the response repair, all three independently invalid cases stopped at Parse submission with exactly the intended schema error; timestamp/envelope/history/write work was skipped and the outer unresolved response succeeded. These are no-write rejection passes, not positive exact-limit acceptance.

| Case | Run | Error |
| --- | --- | --- |
| Summary 1,201 | `08584087288514099919085040243CU12` | `entry.summary`, maxLength 1200 |
| Details 21 | `08584087286734770819559602868CU00` | `entry.items`, maxItems 20 |
| One detail 1,001 | `08584087286071784128898628726CU12` | `entry.items[0]`, maxLength 1000 |

Frozen inputs: `_assets/test-isolated-boundaries.json`; rendered action evidence: `_assets/summary-over-rejection.txt`, `_assets/details-over-rejection.txt`, `_assets/detail-over-rejection.txt`. Exact-limit input is staged but has not been submitted while catalog access is denied.

### Packaging export result

Both existing connection references were added to the solution. Export version 1.0.0.1 explicitly failed because the referenced custom connector must first be added to a Dataverse solution. The alternate classic editor eventually loaded after sign-in, but a connector-add workaround has not been verified; no security setting was changed. The live app and two flows retain their original identities and remain enabled. This is an incomplete migration package, not an import-ready backup.

### Callback receiver preparation

The user confirmed Jason Wohlgemuth as the initial recipient and the existing pilot environment. Future shared migration and a group channel are PA-12. Intended initial email destination is `o9w@ornl.gov`; freeze the destination with each claimed receipt so a future configuration change cannot reroute a retry unexpectedly.

`../vergil-gitlab-jsonl-automation/_assets/callback-event.schema.json` and `../vergil-gitlab-jsonl-automation/_assets/callback-event-fixtures.json` follow the actual current `ArtifactEvent` representation: fields are `project_id`, `merge_request_url`, and `package_url`. The prior plan incorrectly used camelCase field names. A cached-serde compilation using the source enum's exact serialization attributes produced `{"event":"review-needed","merge_request_url":"https://code.ornl.gov/example/-/merge_requests/1","project_id":"acorn"}`. The enum-level `rename_all` does not rename struct-variant fields. No Vergil or Rust source was changed. Fixture links are synthetic and do not assert that a merge request/package exists.

Authentication remains a release gate: the current outbound sender has no Entra bearer token. Do not activate an anonymous/SAS receiver or weaken authentication as a workaround without the explicit access decision. The owner-only manual receiver has now sent controlled TEST ONLY notifications to Jason and recorded their delivery states. No externally callable callback endpoint exists yet.

### Callback ledger provisioned

Table **ACORN Callback Receipts** (`cr1d0_acorncallbackreceipts`, table ID `5e3c082e-f4cf-f111-a05c-001dd8062052`) is user/team-owned in the pilot solution. Required primary `cr1d0_webhookid` has maximum length 400 and alternate key `cr1d0_uniquewebhookid` is **Active**. Columns `cr1d0_payloadjson` (multiline 65,536 characters), `cr1d0_emailrecipient` (400), `cr1d0_emailstate`, and `cr1d0_emailattemptrun` preserve exact input, frozen destination, delivery state, and attempt identity. No sharing permissions were widened. Evidence: `../vergil-gitlab-jsonl-automation/_assets/callback-unique-key-active.png`. The manual validation flow is separate from an externally reachable receiver. The manual notification path is enabled; no public callback endpoint has been activated.

The owner-only instant flow **ACORN Callback Receipt Validation** (`dd1d8f67-116c-f55d-7752-7c7d322691c5`) is saved and On in the same solution, with Dataverse reference `cr1d0_sharedcommondataserviceforapps_5298a`. It validates delivery-ID length and the strict event schema, then attempts one unique receipt creation with automatic retries disabled. It is an owner-only manual receipt/notification flow; conversion to the authenticated external receiver remains outstanding. First artifact-ready test run `08584087280680953294196390662CU07` passed and created row `185eac5d-f5cf-f111-a05c-001dd8062052` at `2026-10-24T21:53:38Z`, retaining the exact payload, `o9w@ornl.gov`, and Pending state. Evidence: `../vergil-gitlab-jsonl-automation/_assets/callback-first-claim.txt` and `../vergil-gitlab-jsonl-automation/_assets/callback-first-claim.png`. Power Automate rejected JSON Schema `pattern`; the compatible saved delivery check uses a one-element JSON array with a required nonempty string of at most 400 characters. The subsequent notification and guard tests below cover sequential duplicate and uncertain-send handling. Endpoint authentication remains outstanding.

All five event fixtures passed in the manual receipt validator: artifact-ready `08584087280680953294196390662CU07`, failed `08584087280233556631342477788CU05`, mr-created `08584087280173134595172518075CU22`, report-no-changes `08584087280025048509062858047CU25`, and review-needed `08584087279934435300149713384CU01`. Exact duplicate claim `08584087279494021723442845390CU05` failed with `Entity Key Unique Webhook ID violated`, explicitly preventing a second record. This proves atomic duplicate protection at creation, not complete duplicate notification handling. Evidence: `../vergil-gitlab-jsonl-automation/_assets/callback-event-runs.txt` and `../vergil-gitlab-jsonl-automation/_assets/callback-duplicate-claim.txt`.

`../vergil-gitlab-jsonl-automation/_assets/callback-delivery-scope.clipboard.json` implements the saved serialized receipt lookup, exact-payload comparison, Pending → Sending (with flow run ID) → Sent transitions, and one selected Teams or Outlook send with automatic retries disabled. New routine events freeze the Teams route, and milestones freeze email; see the current walkthrough. Sent duplicates skip delivery; Sending/unknown states stop for owner reconciliation. The scope must run after successful, failed, or timed-out claim scope, and the manual trigger must have concurrency set to one. Claims are re-read from Dataverse before any send. The saved trigger uses concurrency one and secure inputs/outputs. The live acceptance results below verify delivery and sequential replay guards. Connector operation definitions follow [Microsoft Dataverse](https://learn.microsoft.com/en-us/connectors/commondataserviceforapps/); source imports still require existing protected connection bindings.

Offline contract validation passed 36/36 checks using `../vergil-gitlab-jsonl-automation/_assets/check-callback-schema.py` with `jsonschema==4.26.0` in a temporary environment. The checks accept all five actual event shapes and reject missing fields, extra fields, wrong project types, unknown events, camelCase representations, null, and arrays. Results: `../vergil-gitlab-jsonl-automation/_assets/callback-schema-checks.json`. This complements the five live event claims; it does not replace live negative receiver or email tests.

For the uncertain-email test, only synthetic receipt `acorn-test-failed-20261024` (row `bf66e572-f5cf-f111-a05c-001dd8062052`) was changed from Pending to Sending in the table. Its attempt field explicitly reads `TEST ONLY simulated uncertain send; no email attempted`. This is fault injection for the reconciliation guard, not evidence that a send occurred. Run `08584087270820424131919890428CU22` reached Reconcile email with `DeliveryNeedsReview`, skipped Send email once, and left the simulated Sending record unchanged. This verifies the hold-for-review branch without attempting an uncertain resend.

The guarded notification save returned `Request to XRM API failed` / `Timeout occurred when calling the XRM api` after several minutes. One controlled retry succeeded. The editor reload had removed both manual input fields; Webhook ID (`text`) and Event JSON (`text_1`) were restored, saved, and verified in the run form. The live notification tests below confirm the saved version executes. The two clipboard scopes and `../vergil-gitlab-jsonl-automation/_assets/callback-manual-trigger.json` preserve the source. Evidence: `../vergil-gitlab-jsonl-automation/_assets/callback-save-timeout.txt` and `../vergil-gitlab-jsonl-automation/_assets/callback-save-timeout.png`.


### Earlier email-only manual notification acceptance (2026-10-24)

The current [Teams/milestone routing](notification-walkthrough.md) supersedes the channel selection in this historical test section. Existing receipts retain their frozen recipients.

Existing Outlook connection reference `new_sharedoffice365_29833` (Office 365 Outlook Default-29833, `o9w@ornl.gov`) is bound alongside Dataverse. Email recipient is frozen at receipt creation. To switch future receipts to another approved email destination, update Claim receipt's recipient value. A Teams group channel needs a separate channel action and independent channel state; PA-12 owns that migration. Existing receipts retain their original destination.

| Case | Run ID | Verified outcome |
| --- | --- | --- |
| Resume Pending artifact receipt | `08584087272391353126142508200CU12` | Mark sending, Send email once, and Mark sent succeeded; stored Sent with this attempt ID. |
| Exact replay after Sent | `08584087272014395411131905854CU21` | Duplicate no send succeeded; all three sending actions skipped; original attempt retained. |
| Same ID, changed cutoff | `08584087270954123269969813787CU10` | Payload conflict terminated the run; email and state updates skipped. |
| Simulated uncertain send | `08584087270820424131919890428CU22` | DeliveryNeedsReview; email skipped; Sending retained for owner review. |
| Missing required message | `08584087270736137078963332238CU15` | Validate event failed; Claim receipt and Find receipt skipped; Reject invalid terminated before delivery. |
| New report-no-changes receipt | `08584087270599311451259340633CU06` | Run succeeded; new row `087158b6-f7cf-f111-a05c-001dd8062052` is Sent with this attempt ID. |

Three controlled TEST ONLY emails were accepted by Outlook. Inbox receipt was not independently inspected. Evidence: `../vergil-gitlab-jsonl-automation/_assets/callback-email-sent.txt`, `../vergil-gitlab-jsonl-automation/_assets/callback-email-replay.txt`, `../vergil-gitlab-jsonl-automation/_assets/callback-payload-conflict.txt`, `../vergil-gitlab-jsonl-automation/_assets/callback-uncertain-held.txt`, `../vergil-gitlab-jsonl-automation/_assets/callback-invalid-event.txt`, and `../vergil-gitlab-jsonl-automation/_assets/callback-sent-ledger.txt`. The uncertainty run page also displayed a generic mixed-authentication banner; its actual action evidence shows successful receipt lookup and the intentional DeliveryNeedsReview termination. No authentication setting was changed in response.

At this earlier acceptance point, the receipt table contained seven synthetic rows: artifact-ready, fresh-notification, and concurrent-notification are Sent, failed is the clearly marked Sending simulation, and the other three original event fixtures remain Pending. Do not treat these as real sponsor events. Concurrent duplicate notification passed as documented below. Process/restart recovery with the real sender, external HTTP acknowledgement, and authentication acceptance remain unproven.

### Notification recovery procedure

1. Find the receipt by exact Webhook ID and compare the exact Event JSON. A changed payload is a conflict; preserve the stored row and investigate the sender.
2. A Sent row is final for that destination; exact replay skips delivery. Pending can be retried with the identical ID and original JSON through the owner-only flow.
3. For Sending or another unresolved state, inspect the recorded flow attempt and the selected Teams chat or the sending account's Outlook Sent Items. Never reset Pending solely because a connector timed out. If delivery is proven, reconcile the row to Sent and retain its attempt ID. If non-delivery is proven, record the investigation outside the receipt and then authorize a retry of the unchanged payload. Ambiguous delivery stays held.
4. Keep this single serialized receiver as the only automated writer of delivery state. Another concurrently running receiver would need an atomic state transition claim in addition to the unique receipt key.

### Source backup

`_assets/sponsor-highlights-source-bundle.zip` contains source files and selected redacted acceptance evidence with SHA-256 hashes in `_assets/source-bundle-manifest.json`. It is a source backup, not an importable Power Platform solution. The portal custom-connector inclusion failure still prevents a complete solution export.


### Concurrent callback acceptance

Two identical requests for `acorn-test-concurrent-notification-20261024` were launched together at `2026-10-24T22:12:13.383Z`, using the review-needed fixture. Run `08584087269518872627330047333CU28` owns the only receipt `56d0b9f1-f7cf-f111-a05c-001dd8062052`, now Sent with that attempt ID. Run `08584087269515367688075901384CU25` reached Duplicate no send; Mark sending, Send email once, and Mark sent were all skipped. Both runs succeeded. Evidence: `../vergil-gitlab-jsonl-automation/_assets/callback-concurrent-runs.txt`, `../vergil-gitlab-jsonl-automation/_assets/callback-concurrent-no-send.txt`, and `../vergil-gitlab-jsonl-automation/_assets/callback-concurrent-ledger.txt`. The manual trigger remains serialized; this test does not authorize a second independent receiver.

Power Automate reports trigger-concurrency throttling after the simultaneous test. Concurrency one is intentional: retain it to serialize the non-idempotent notification action. The two requests completed successfully. Do not apply the portal suggestion to disable concurrency control without implementing an atomic per-channel delivery claim.


### Remaining pilot release gates

- Establish reliable approved Power Automate-to-GitLab egress; the frozen concurrent writer tests still receive 403 before writing. Then finish exact-limit, malformed-tail/size and concurrent creator/writer acceptance. The isolated failed-commit response test now passes; see [failure acceptance](write-failure-acceptance.md). Do not change network controls or rotate working credentials to evade that denial.
- Complete the importable solution backup after the existing custom connector can be added to Dataverse. Current source backup remains usable for review/reconstruction but is not an import package.
- Validate published-browser storage denial and a full Chrome process restart when the user's token-display pages and other unsaved work are secured. Reload and app-tab recovery already pass. Keep one app tab per browser pending slot.
- Choose and approve the sender-compatible callback authentication, then test external acknowledgement and lost-response recovery. The manual notification receiver is available independently of Vergil.
- PA-12 separately covers the later approved shared environment and group/channel destination. Keep the current pilot environment and Jason destination until that phase is accepted.

The manual receiver was disabled, its Off state verified, then re-enabled and its On state verified. Exact replay `08584087268597403318991083635CU22` at `2026-10-24T22:13:45Z` reached Duplicate no send and skipped every sending action. Receipt `56d0b9f1-f7cf-f111-a05c-001dd8062052` retained its original Sent attempt `08584087269518872627330047333CU28`. Evidence: `../vergil-gitlab-jsonl-automation/_assets/callback-reactivation-replay.txt`. This verifies flow reactivation, not a sender-process restart or lost HTTP response.

### Project-list button wording (2026-10-24)

The published button now uses the existing catalog state: Load Project List before a successful load, Loading projects… during a request, and Refresh Project List while the catalog is loaded. The Text formula has no reported formula errors. No submission or catalog-loading behavior changed.

The user/reviewer PDF includes a Pikchr workflow and notification map on page 2. See the [diagram source and routing notes](notification-walkthrough.md).
