# PA-39B cross-app UI acceptance

November 10, 2026. The published TEST ONLY component library `Sponsor Highlights UI TEST ONLY` is live at version 18. The published contributor app is live at version 16; the unpublished `[PROTOTYPE] Intake Approval` reviewer app has adopted the same library revision and is saved for Studio Preview. Neither app publishes a report or changes operational approval data.

## Shared behavior and language

| Surface | Contributor | Reviewer |
|---|---|---|
| Segmented choice | Project highlight / General update and three-state status use `cmpSegmentedChoice`. General update retains its canonical report-wide key, has the requested mouseover title, and sits 12 pixels from Project highlight at desktop width. | Three-state status uses the same library component and selected-state check mark. |
| Longer choices | Category and optional event type use the library's native-radio component. History edit restores the saved canonical category/event before correction. | The fixture uses the same radio grammar where present; its existing classic dropdowns retain native selection behavior. |
| Narrative and supporting details | `cmpNarrative` accepts one detail per line, up to 20 nonempty details and 1,000 characters each. | A native multiline editor uses the same line order, with the visible label `Supporting details · optional. One detail per line; move lines to reorder.` The original details remain visible for comparison. |
| Save state | Recovery, pending retry, rejection, and export-ready states use the shared status-message language. | Fixture edits are local and identified as such. The reviewer remains unpublished pending operational binding work. |

The `cmpRepeatableText` library component is a stub. PA-39B accepts line-based editing in both apps and revises the shared UI contract accordingly. The saved reviewer source now checks more than 20 nonempty lines or any line longer than 1,000 characters and blocks Approve through its validation text; the negative runtime branch was not independently confirmed in Preview. The reviewer app's Office Green theme gives its app-owned classic buttons and dropdown accents a green treatment; library segmented controls retain their semantic green token. The reviewer filter now reads `Open` with a title explaining that it shows unresolved highlights, and the dark header retains white title text.

## Checks

| Check | Evidence |
|---|---|
| Contributor placement | [Live placement screenshot](pa39-placement-update.png); published player DOM exposed `General update`, the exact native `title` text, and button bounds with a 12-pixel gap. |
| Reviewer desktop | [Desktop fixture screenshot](pa39b-reviewer-desktop.png) shows the page rail, report canvas, inspector, and green controls. |
| Reviewer tablet | [1024-pixel tablet screenshot](pa39b-reviewer-tablet.png) shows the page picker, report preview, and inspector stacked without a horizontal page canvas. |
| Reviewer phone | [375-pixel phone screenshot](pa39b-reviewer-phone.png) shows the page picker, report preview, inspector, and `Open` filter in a single column. |
| App Checker | Contributor: zero formula, runtime, data-source, and performance issues; 10 deferred accessibility findings. Reviewer: zero formula, runtime, and data-source issues after the shared-component and wording changes; 1 performance advisory and 24 deferred accessibility findings. The later entry-path repair showed zero formula errors in Studio. |
| Contributor observation | The user used the TEST ONLY contributor form, made edits to an earlier entry, and said they reached it by clicking `Submission`. Their initial `...huh?` response to a technical task prompt showed that the observation instructions needed simpler language. No app failure was reported. |
| Reviewer observation and repair | The user twice reported they could not reach the fixture from the reviewer Studio link. Reproduction found that a fresh Preview opened the live ledger, whose `Fixture review` button led to a second screen requiring another `Review fixture` click. The first button now sets a fixture-entry flag and navigates; that screen's `OnVisible` selects its local fixture initializer, so one click opens `Fixture preview · 8 updates · 5 unresolved · Local only`. A synthetic Approve changed the count to 4 unresolved and Unresolved restored 5. On the retry, the user completed the reviewer task. |

The two observations use the same person in both roles at the user's direction. The reviewer reported that some buttons were occluded and that approved, excluded, and unresolved items were hard to tell apart. The user said the listing needs stronger visual hierarchy.

## PA-39C reviewer repair

The saved, unpublished TEST ONLY reviewer app now gives each report card a semibold title and a full-width decision band. The bands combine text and color: amber `NEEDS REVIEW`, green `APPROVED`, and red `EXCLUDED`, followed by the page name and an `EDITED` cue when applicable. The selected decision button uses the same text and color cue; the other actions remain neutral. The inspector stacks all three buttons at narrow panel widths, including the 320-pixel desktop inspector and the 375-pixel phone, and expands when an exclusion reason is required. The phone's report area and mode line have room to render before the inspector; the two sections scroll in order without text crossing their boundary. This keeps the existing fixture decisions, validation, and preview inclusion rules.

The first reviewer `Fixture review` action now reaches the eight-item local fixture in one click. App Checker after the repair reported zero formula, runtime, and data-source issues; 24 accessibility findings and one performance advisory remain deferred as previously recorded. In Preview, the eight-item fixture opened with five unresolved items. Approve, Exclude with a required reason, and Needs review changed the visible selected button and report-card band; the unresolved count responded. The long status card, exclusion reason, and Next problem button remained readable. [Desktop proof](pa39c-reviewer-desktop.jpg) shows all three decision bands and the full exclusion panel; [375-pixel phone proof](pa39c-reviewer-phone.jpg) shows the stacked decision controls and source comparison after scrolling. The fixture state is local to Preview and the reviewer app was not published. The user's original observation has not been repeated as a separate uncoached retest after the repair; PA-40A retains that acceptance gate.

Accessibility and production publication checks remain outside this TEST ONLY acceptance under the user's prior deferral and the PA-39 boundary.
