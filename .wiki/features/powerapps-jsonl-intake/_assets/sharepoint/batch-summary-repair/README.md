# Automatic batch summary startup repair

November 25, 2026. Scope: `[BETA] Sponsor Highlights Review`, app `ef761a81-bd35-4854-8de0-f91a452ba73f`, `ApprovalReviewScreen`. The request and background Power Automate flows are unchanged.

## Reproduction and diagnosis

Published version 129 loaded the four-item Ready batch and left two summaries queued. A fresh player launch and the app's Reload action both reproduced the stall; the refreshed request-flow history showed no new summary request. Explicit Regenerate had already succeeded in the [earlier audit](../flow-failure-audit-2026-11-25/README.md). This localizes the observed defect to automatic dispatch in the published app rather than summary generation in Power Automate.

The fresh [native app backup](reviewer-before.msapp) exposes a one-second repeating `ReviewAiTimer`. Its `Start` expression depends on `varSummaryIndex`; the batch loader set that index to 1 and then reset the timer, while `AutoStart` also started it at screen entry. The startup/reset combination did not reliably dispatch in the published player. The original formulas did process the queue in Studio Preview, so the precise platform-level reason for the player/Preview difference is not established by this test.

## Change

Only two control properties changed, as verified across all native control/component formulas in the before/after downloads:

1. `ReviewBack.OnSelect` now sets `varSummaryIndex` to 0 before loading. Once it has built the summary queue, it resets `ReviewAiTimer` and then sets the index to 1. Failed loads leave the timer stopped.
2. `ReviewAiTimer.AutoStart` is false. The existing `Start` expression now controls startup after queue initialization rather than competing with screen-entry startup.

The [exact formula delta](formula-delta.json), [readable loader formula](ReviewBack-OnSelect.after.fx), [before source](before-ApprovalReviewScreen.pa.yaml), [after source](after-ApprovalReviewScreen.pa.yaml), and [repaired native copy](reviewer-after.msapp) are retained. Data-source and resource references compare equal. All other control/component formulas compare equal, including the flow calls, authorization bindings, input/output limits, existing-summary preservation, suggestion application, review save, and timer callback.

## Validation and publication

- Both native archives pass ZIP integrity checks. The exact expected two-property delta passes; the loader equals the prepared formula and AutoStart is false.
- App Checker reports no formula errors. Both exported checker reports contain the same 65 pre-existing accessibility findings and two performance findings; this repair does not claim to resolve them.
- Corrected Preview Reload generated both queued suggestions sequentially. Runs `08584059841419174266250541875CU24` and `08584059841030849618122945985CU21` succeeded in 39 and 35 seconds. The pending count reached zero without selecting Regenerate.
- Version 130 was published, and the maker Versions page marked it Live. [Publication capture](published-version.jpg).
- After the player offered Refresh for the new version, a fresh published load dispatched both queued summaries without Regenerate. Runs `08584059838748643341489211509CU13` and `08584059838355297389170927157CU02` succeeded in 39 and 34 seconds; the pending count reached zero. The selected source item 24 displayed Use Suggestion while remaining Draft v1, ETag `"1"`, Needs review.
- Published Reload also dispatched and completed both suggestions automatically: `08584059837523421192833789670CU02` succeeded in 38 seconds and `08584059837136466301213114325CU00` in 34 seconds. The pending count again reached zero, with Use Suggestion available and the same source item draft version, ETag, and decision. [Run evidence](verification.json) and [successful run capture](published-runs.jpg).

Both fresh startup and repeated Reload therefore pass in the published owner-context app. The version moved from Power Apps runtime 3.26102.13 to 3.26103.35 during normal republishing; these checks establish the repaired published behavior, without separating runtime-version effects from the explicit-start formula change.

Opening the older player immediately after publishing initially displayed a new-version-preparation notice. The test waits for the product's Refresh notice before evaluating the repaired published app. Microsoft documents this [publication propagation behavior](https://learn.microsoft.com/en-us/power-apps/maker/canvas-apps/save-publish-app#in-app-notifications-for-new-versions).

No suggestion was applied or saved to a review item, and no source receipt, decision, finalization, or export was changed. Owner-context checks do not close the participant-context PA-40C release gate. The earlier four historical response timeouts also remain unexplained; this repair addresses the separately reproduced automatic-dispatch stall.

Scoped diff whitespace checks passed. Full wiki validation reported four unrelated stale prerequisite references (ABW-04/05 in ABW-07/08/09) in the concurrently edited browser-workspace plan; it reported no Power Platform plan errors. Those unrelated files were left untouched.

## Recovery

Version 129 remains in app version history. The native before/after copies and original formulas support rollback of these two properties. Restoring the prior app would restore its automatic-dispatch defect; it would not undo generated queue requests or alter review records. Avoid importing the broader package or restoring unrelated flow definitions for this repair.
