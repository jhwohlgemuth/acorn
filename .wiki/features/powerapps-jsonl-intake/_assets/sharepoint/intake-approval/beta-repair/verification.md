# November 12 beta repairs

Work remains in progress. This record distinguishes published changes from remaining release acceptance.

## Contributor changes verified in the published player

App `427852d5-c8a6-4218-ab2d-7db34e1106cc` is named `[BETA] Sponsor Highlights Intake`. The published player shows both toggle states, Title Case choice buttons, Submit/Edit/Cancel actions, a red validation message inside Report Preview, and compact footer sizing based on visible controls. Shared narrative inputs now have an eight-pixel bottom clearance. The shared radio control resets its stored selection when a draft changes. Desktop and 375 × 812 Studio previews confirmed complete narrative borders in Activity, General, and Project Status, including evidence and conditional risk explanation. The narrow Classified Access choice now fits without changing its canonical key. Independent participant acceptance remains open.

The failed c5c correction run `08584071014607499125458904966CU17` exposed schema errors for root `range` and status entry fields `significance`, `health`, `evidence`, and `risk`. The correction parser had accepted only Activity records. The saved intake flow now accepts the same Activity/Project Status alternatives as new submissions, plus required revision metadata; ownership, predecessor, identity, and conflict guards remain. It additionally preserves entry kind across corrections.

New submissions use the actual project catalog, bounded at 1,000 entries, instead of an acorn-only fixture condition. General entries must have General placement; status records must identify a project. The summary prefix restriction is removed. The response runs after success, failure, timeout, or skip and distinguishes invalid fields, invalid dates, and other validation failures.

Live synthetic verification under the owner's account:

| Case | Receipt | Observed result |
|---|---|---|
| BDI Project Status, without TEST ONLY prefix | `0b1342a8-6397-41c8-a119-0d957264470a` | SharePoint receipt and export confirmed |
| Edit that status record, changing Evidence | `4027017c-9deb-477d-92c9-fb132923d066` | All status fields restored; correction receipt and export confirmed |

These checks do not replace c5c's participant-context retest. Her four original submissions and Ready batch `ed4821fc-8097-416e-aee1-dd7f795d2e17` were not edited.

## Project catalog titles

The SharePoint catalog now uses verified index titles for all twelve projects with an index source. Eleven came from `index.json` title attributes and HIFLD from the `index.md` heading in pilot repository revision `ca480f129ece7f8ef6bb27572fc33c8988020225`. The acorn directory has no index title in that snapshot, so its existing ID label remains the fallback. General placement is unchanged. All fourteen ProjectId values were preserved; the catalog flow already maps Title to label. SharePoint row readback confirmed every updated title and revision.

## Published reviewer and background AI

The published reviewer is named `[BETA] Sponsor Highlights Review` and starts on ReviewLedgerLiveScreen. The unused contributor screen, empty review screen, fixture shell and project fixture buttons were removed. Four unused flow bindings and the direct AI model data source were removed. Both apps and the shared library are published with [BETA] names. Maker Details displays Standard for the reviewer app. The current Studio has no direct data sources and retains only the five live review flows. Maker connection/flow listings still show historical dependencies; those listings alone are not proof of current execution paths.

Background flow `e731badd-095c-49bf-806a-c0b6b5132bd5`, `[BETA] Generate Reviewer Summary`, was imported and enabled. It reads the restricted Summary Queue list `442cdec0-52b9-4f24-97b6-092961312c41` and runs on the owner's plan, with no associated app. An initial synthetic run skipped generation because the owner's SharePoint Email differed from the login identity; the guard now matches the canonical Author Claims value. A replay generated and saved a ready response for synthetic request `260b2246-5327-4ee6-9bfc-7c320d96fd49` in 3.2 seconds. The request contains only invented documentation-review facts.

The SharePoint-only replacement was imported into existing request flow `0ddfbf82-88c1-499f-93af-c3d33718ff22`, enabled, and refreshed in the app. It authorizes the caller against reviewer group 42, queues bounded content using the owner’s SharePoint connection, polls for a result, and returns the existing summary response. Its Connections panel contains SharePoint only. The background flow owns the AI Builder/Dataverse execution and has no associated apps. No premium licenses were purchased or assigned.

Synthetic item 19 was already approved. Saving its unchanged review in the live ledger generated a suggestion automatically; contextual regeneration also succeeded. Queue request `09c7381b-9453-44a4-a620-caeeb3cc243b` contains the added November 12 documentation-review context and a ready response that preserves it. No generated suggestion was applied to the saved review.

The published Report View was also repaired: Save uses the same approved-save trigger, the obsolete Generate Summary action is hidden before the first attempt, Use Suggestion and Regenerate Summary use Title Case, and a 2,000-character context field is available. A blank validation state no longer displays Action Required. Decision and comparison section heights sum visible children, and the AI panel has no inner scroll. Context is read directly from its field and reset on screen entry, item selection, or project-page change. An early guarded context formula omitted the typed text; the final direct-field formula was verified by queue request `9d6430e3-8b3c-4827-b704-30999d61bfcd`, whose request includes the added documentation date and whose ready response includes November 12 and identifies the result as synthetic.

The existing prompt remains available; its exported metadata labels the action Standard, while Microsoft’s AI Builder licensing pages differ in their wording about standalone flows. The observed Standard app designation and successful owner-context service runs do not establish the participant’s entitlement or tenant-specific approval. A fresh participant launch/read/save remains required; no premium trial is a prerequisite in the user guide.


## Shared toggle regression and final UI checks

The pointer hit area previously changed the native switch through Reset without reliably updating its selected key. It now explicitly sets the next key and invokes OnChange; the two-choice output derives its key directly from the native switch state. Both apps imported the final library revision published November 12 at 15:01:41 Eastern. Studio temporarily retained stale imported-property metadata; saving and reloading cleared the transient Choices errors. Fresh sessions reported no formula issues before publication. Accessibility advisories remain outside the owner-deferred acceptance scope.

Fresh Studio and published-player checks verified that Project Status reveals health, significance, and evidence, disables General placement, and returns correctly to Activity. General placement hides the project field and changes the preview to the report title page. The final narrow risk label is Classified Access with its key icon; its payload key remains classified-environment-access. Narrative border and bottom-preview screenshots were inspected in the live browser; durable guide images are explicitly labeled redrawings, not exported captures.

## Documentation, cleanup, and remaining acceptance

The five-page participant guide uses only current labeled redrawings, [BETA] names, both toggle states, project titles, Submit/Edit/Save labels, preview validation, and automatic/contextual summary instructions. It no longer asks participants to add TEST ONLY or use fixture controls.

The active intake/catalog/read/save/summary and retained batch/finalization/export display names use [BETA]. Flow identities, trigger states, receipt storage, and operational publication boundaries are preserved. Two unused reviewer screens, the fixture shell/navigation, four unused app flow bindings, and the direct AI model binding were removed. The obsolete JSONL reader `261bef08-d788-45f6-9127-7ffadafca0b0` has no associated apps, was disabled, and has a ZIP-integrity-checked native backup `reviewer-updates-before-cleanup.zip`; permanent deletion awaits the requested action-time confirmation.

The two custom solutions remain nonempty: ACORN SharePoint contains two flows and one connection reference; ACORN Sponsor Highlights Intake contains the retained premium fallback app, three flows, and three connection references. Their current contents are recovery/operational dependencies, so they were retained. Explicitly protected standalone summary flows and the owner’s unrelated summ flow were not changed. No receipt, review audit, or participant batch was removed.

Remaining: c5c must reopen the current reviewer and verify launch/read/save without the former licensing error, and retry editing a submission. Her four originals and Ready batch remain untouched. Independent PA-40A walkthrough/fidelity acceptance and PA-41 denied-role, late-arrival, lost-response, and authorized operational cutover gates remain open. The obsolete reader’s deletion is pending confirmation; the native recovery copy remains local.
