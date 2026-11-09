# PA-35A activity preview acceptance

November 7, 2026. The saved, unpublished `[PROTOTYPE] Intake Approval` app contains a local eight-update review fixture. Six activity projections are in [activities.jsonl](../fixtures/activities.jsonl); the two status projections are in [status-and-corrections.jsonl](../fixtures/status-and-corrections.jsonl). The reviewer can edit the selected summary in the report canvas, edit structured activity fields and ordered supporting-item lines in the inspector, compare source text and fields, and mark an item unresolved, approved, or excluded. The `Only` preview omits the one excluded Borealis activity while the audit card remains in the full view. Edits reset a prior approval to unresolved. Approve is disabled while the selected item has validation errors.

## Renderer comparison

The same seven included projections were staged in an isolated copy of the ACORN report fixtures: seven accepted, zero rejected, then applied to two project indexes. A correction to the Atlas software activity staged separately: one accepted, zero rejected. The [combined October PDF](pa35-fixture-report.pdf) was regenerated after the correction (SHA-256 `d6b8d1b45623d68df4ea28fca8c7b45dfbccc5dd8f0e93154a319e10b083d450`). The report has the October 2026 title and `01OCT–01NOV` interval; General updates show October 17 Collaboration then October 8 Data; Atlas shows its status, October 13 Completed Method, and October 21 In progress Software; Borealis shows its status and October 27 Upcoming Personnel. The excluded October 12 Borealis activity is absent. Project subtitles, keywords, investigators, sponsors, summaries, bullet order, significance, evidence, and risk text were compared with the preview.

The preview keeps these records in report order when General is selected. Its page controls put the chosen project first and retain the other pages below; the note above the list identifies this navigation mode. General restores PDF order. The classic Canvas gallery does not provide a scroll-to-row action, so this is a bounded navigation difference. The app uses compact text cards rather than the PDF's icons, colored group bands, group counts, provenance labels, and page breaks. These visual differences do not alter the source projection.

## UI checks

- A typed summary edit changed the visible report card and added `Edited` to its state. Reopening the fixture restored the original local data.
- The seven included cards and eight full-view cards were observed. General and both projects were reachable on the wide rail and 375-pixel picker.
- The [375-pixel phone](pa35-phone-preview.jpg), [1080-pixel tablet](pa35-tablet-preview.jpg), and [wide landscape](pa35-wide-preview.jpg) captures show the report and stacked or adjacent inspector.
- Power Apps App Checker settled at zero formula, runtime, and accessibility issues. Its one inherited performance advisory concerns the copied contributor control `Button6` and predates this workspace.

The fixture decisions and edits are local to the preview session. They do not write a ledger or finalize a batch.
