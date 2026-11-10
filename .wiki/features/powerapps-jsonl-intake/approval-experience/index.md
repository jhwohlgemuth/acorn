---
type: Plan
title: "Shared intake and approval experience"
description: "Create one shared Power Apps interaction system, a report-shaped approval workspace, and a contributor form that uses the same components and language."
difficulty: high
impact: critical
package: [power-platform]
tags: [powerapps, usability, responsive, wysiwyg, intake, approval]
status: in-progress
execution: phased
phase_budget_hours: 5
priority: next
---

# Goal

Make the contributor and reviewer experiences feel like two roles in one product. Freeze a shared component and design-token contract, build the reviewer app around the structure of the generated report, and replace the contributor app's fixed-choice dropdowns with the same accessible control grammar without changing accepted receipt, correction, recovery, or notification behavior.

# Current evidence

The current contributor source in `_assets/sharepoint/layout.pa.yaml` uses classic dropdowns for category, state, event type, and report placement. Category has 12 values, state has three values, event type has eight values plus no value, and placement has two non-Boolean choices. Project selection is a dynamic catalog and should remain searchable. Summary and supporting items are narrative inputs with existing 1,200-character, 20-item, and 1,000-character-per-item limits.

The admitted schema supports two entry shapes. Activities include project, placement, date, category, state, optional event type, optional classification, summary, and supporting items. Status updates include project, reporting window, summary, significance, six health states with conditional context, optional evidence, and an optional risk with 11 kinds and an explanation. Identity, receipt IDs, revision ancestry, origin, routing, and approval IDs are system-controlled.

The actual combined report has a General updates title page followed by project pages. Project pages render status updates first, then activity groups in completed, in-progress, and upcoming order. Activity cards show category, date, event/provenance/classification/archive details, summary paragraphs, and supporting items. Status cards show reporting window, health/context, summary, significance, evidence, and risk. This renderer—not the current form layout—is the preview contract.

Current Microsoft guidance supports reusable component libraries, responsive auto-layout containers, modern controls, horizontal radio groups, accessibility checking, and labeled toggles. It also documents material constraints: component updates must be published and explicitly adopted by consuming apps; nondelegable Canvas queries can return incomplete results; and custom composite radio/checkbox roles or custom arrow-key behavior are not reliably available. The design therefore prefers native controls and honest button semantics over visually clever but inaccessible composites. See [component libraries](https://learn.microsoft.com/power-apps/maker/canvas-apps/component-library), [responsive layouts](https://learn.microsoft.com/power-apps/maker/canvas-apps/create-responsive-layout), [delegation](https://learn.microsoft.com/power-apps/maker/canvas-apps/delegation-overview), and [accessibility limitations](https://learn.microsoft.com/power-apps/maker/canvas-apps/accessible-apps-limitations).

The November 7, 2026 PA-34 preparation check at `10b17f22` verified the contract against `crates/acorn-lib/src/io/api/powerautomate/form.rs`, `crates/acorn-schema/src/research_activity/logbook.rs`, `crates/acorn-schema/src/types.rs`, `crates/acorn-lib/src/io/export/updates/html.rs`, and `_assets/sharepoint/layout.pa.yaml`. The fixed vocabularies, field limits, status-first report order, and three activity-state groups still match this plan. The current Canvas source still uses classic dropdowns for project, category, state, event type, and report placement. The selected Power Apps environment is accessible and currently contains no component libraries, so PA-34 creates `Sponsor Highlights UI TEST ONLY` as a new TEST ONLY library rather than revising an existing library. No operational app, flow, connection, or data mutation is part of that preparation.

PA-34 completed on November 7, 2026. Contract revision 1.0.0 is frozen in `ui-contract.md` and `report-preview-map.json`. The published TEST ONLY component library `Sponsor Highlights UI TEST ONLY` contains `cmpBooleanToggle`, `cmpSegmentedChoice`, `cmpRadioChoice`, `cmpCheckboxChoice`, `cmpSearchChoice`, `cmpNarrative`, `cmpRepeatableText`, and `cmpStatusMessage`. Its native-control harness passed 375-pixel and 816-pixel width checks, and Power Apps App Checker reported no formula, runtime, accessibility, or performance issues after recheck. The library remains isolated from operational apps, flows, connections, and data.

PA-35 completed on November 7, 2026 in the saved, unpublished `[PROTOTYPE] Intake Approval` app. The fixture shell opens on `ApprovalReviewScreen`, shows the October batch's three pages and exact 3/8/4/1/2/1 counts, offers the empty September period, and keeps its status bar and page selection coherent at desktop and 375-pixel widths. Keyboard page navigation, persisted selection within the editor session, and App Checker formula/runtime/accessibility categories passed. The original local download, sanitized wiki app/source, local screenshots, two performance-advisory dispositions, and scope limits are recorded in [PA-35 validation](../_assets/sharepoint/intake-approval/acceptance/pa35-validation.md). The copied contributor screen remains hidden and retains inherited connection references; no live binding or approval edit was added.

A subsequent scoped TEST ONLY change replaced the fixture page rail with the intake catalog's dynamic project list and a continuous, read-only view of the current collected logbook. Desktop project selection and the 375-pixel dropdown scroll to section headers; the fresh-load preview reconciled 29 parsed updates against the read flow's source count. The app is saved and unpublished. [Live report validation](../_assets/sharepoint/intake-approval/acceptance/live-report.md) records the flow, source, evidence, and limits. This collected-file preview remained separate from PA-35A/PA-35B editing and the now-accepted PA-36B bounded review-ledger paging.

PA-35A's activity report canvas and isolated ACORN comparison are recorded in [activity preview acceptance](../_assets/sharepoint/intake-approval/acceptance/activity-preview.md). The seven included projections and one valid Atlas correction rendered in the [October fixture PDF](../_assets/sharepoint/intake-approval/acceptance/pa35-fixture-report.pdf). The app can edit a selected summary in the report canvas, edit activity fields and ordered items in the inspector, compare source values, and preview exclusion. PA-35B completed the status, validation, correction, and responsive fixture behavior in the saved unpublished reviewer app. The repaired TEST ONLY shared library revision 1.0.1 is published and adopted in the reviewer. `cmpNarrative` edits the report-canvas summary, `cmpSegmentedChoice` edits activity state, and `cmpStatusMessage` displays validation. Guarded change handlers preserve decisions on record navigation. [Experience evidence](../_assets/sharepoint/intake-approval/acceptance/experience.md) records the actual component APIs, status and validation behavior, phone/tablet/wide captures, keyboard completion, App Checker result, final PDF comparison, and the user's deferral of remaining accessibility work.

PA-39 completed on November 10, 2026 in the saved, unpublished `Sponsor Highlights Intake UI TEST ONLY` app. The TEST ONLY library revision 1.0.2 repaired two-choice segmented bars, selected-state visuals, and blank narrative defaults; the contributor app adopted it while the reviewer remains on 1.0.1 pending cross-app acceptance. The form now uses shared placement and state bars, category and event radio menus, summary and detail narratives, and a validation message, plus an app-owned report-card preview. `cmpRepeatableText` remained a nonfunctional stub, so supporting details use `cmpNarrative` with the prior one-line-per-item payload semantics; PA-39B must reconcile this contract difference. The project catalog loaded, refreshed, and allowed selection in desktop and 375-pixel preview; the retry/error branch was inspected without fault injection. Summary and detail limits reported errors. The copied submit control is `DisplayMode.Disabled`, and no flow was invoked. The [refit artifacts](../_assets/sharepoint/intake-approval/intake-refit/payload-parity.json) include the exported app package, exact form source, six synthetic fixture byte comparisons, and phone/desktop captures. `layout.pa.yaml` remains the pre-editing generator baseline; the exported package is the full current test app source and rollback point for later refits. The archived production and exported refit JSON record expressions match exactly after six control-reference substitutions; the fixture comparison is a local model, not a Power Fx runtime payload capture. Studio showed no formula errors; its ten accessibility findings are deferred by user direction. The isolated package retains operational flow references and must not be published or used for live submission in this phase.

# Interaction contract

## Control decision table

| Value shape | Required control | Rules |
|---|---|---|
| Genuine Boolean | Labeled modern toggle | Label includes the meaning of both states; color and switch position are never the only cues. |
| Two non-Boolean choices | Two-button segmented bar | Each option is a real button with selected state repeated in text/AccessibleLabel; no false radio role. |
| Three to five short exclusive choices | Segmented button bar | Stable left-to-right order; one selected value; visible focus; selected check/icon plus text; minimum touch target. |
| More than five or long exclusive choices | Native radio menu | Vertical by default; search only when scanning becomes slow; option descriptions may appear below labels. |
| Multiple choices | Native checkbox menu | Same card/menu rhythm as radio menus, explicit selected count and clear-all action; never called or announced as radio. |
| Long or dynamic collection | Searchable combobox | Server-filtered or flow-paged results; no client-only completeness claim. |
| Narrative | Text input or repeatable text rows | Adjacent label, purpose, count, requirement, validation, and preserved draft on recoverable failure. |

The current schema has no fixed multi-select vocabulary. Supporting items are repeatable authored statements, not checkbox choices. The checkbox-menu component is still part of the shared library for future fields, but it is not forced onto narrative data.

## Field mapping

| Field | Contributor control | Reviewer control | Report effect |
|---|---|---|---|
| Entry type | Two-button `Activity` / `Status update` selector when status intake is enabled | Read-only type badge | Chooses activity or status card shape. |
| Project | Searchable project selector | Read-only routing identity; page navigation may move focus, not reroute | Selects the project page. |
| Report placement | Two-button `Project page` / `General updates` bar | Read-only for corrections; otherwise editable only under approved routing rules | Moves an activity between project page and title page. |
| Date | Date picker | Date picker with source comparison | Card date and ordering. |
| Category | Native 12-option radio menu, grouped for scanability without changing canonical value | Same menu | Activity icon and heading. |
| State | Three-button `Completed` / `In progress` / `Upcoming` bar | Same bar | Status group and order. |
| Event type | Optional native radio menu with `Not an event` plus eight values | Same menu | Event detail line. |
| Classification | Not added to contributor scope without a separate authorization decision | Four-option reviewer button bar with warning and policy validation | Detail line; non-Unclassified values require administration category. |
| Summary | Multiline text with character count | Inline card edit plus inspector | Main activity/status narrative. |
| Supporting items | Repeatable text rows, add/remove/reorder | Same rows in inspector | Bulleted list. |
| Reporting window | `Use batch period` toggle plus conditional date range | Same, with source comparison | Status card period. |
| Health | Six-option native radio menu | Same menu; conditional reason for at-risk, delayed, paused | Status label and health context. |
| Significance | Multiline text | Inline/inspector edit | Leadership significance. |
| Evidence | `Add evidence` toggle plus conditional text | Same | Evidence paragraph. |
| Risk | `Include risk or dependency` toggle; 11-option radio menu plus explanation | Same | Risk kind and explanation. |
| Review decision | Not shown | Three-button `Unresolved` / `Approve` / `Exclude` bar; exclusion requires reason | Governs batch completeness and publication. |
| Include in report | Not shown | Labeled toggle derived from decision and shown explicitly | Controls preview visibility; never bypasses audit history. |

## Shared design tokens and states

The component library owns semantic colors, typography, spacing, corner radii, elevation, page/card widths, touch targets, focus borders, icons, control heights, breakpoints, and reduced-motion behavior. It also owns the exact content patterns for required, optional, help, empty, loading, saved, saving, warning, validation error, conflict, offline/recovery, excluded, approved, and publication-blocked states.

Do not fork or locally copy library components in either app. Each app records the adopted library revision. A component change includes version notes and a cross-app visual/behavior regression before adoption.

# Approval workspace design

## Desktop composition

The reviewer opens a batch dashboard, selects a closed period, and enters a three-region workspace:

1. A page rail lists the title page and project pages, with counts for unresolved, edited, approved, and excluded items.
2. The report canvas uses the real report hierarchy and labels. Cards are edited in place for common narrative fields; selecting a card opens the adjacent inspector for structured choices, source comparison, audit metadata, and validation.
3. A sticky batch bar shows period, save state, blockers, preview/PDF comparison status, and the final-review action.

The workspace supports filter-to-problem without changing report order, keyboard next/previous item, collapse/expand of long cards, and a persistent return point. Raw JSON is available only as a diagnostic download to authorized operators, never as the normal editing surface.

## Narrow composition

At 375 CSS pixels, the page rail becomes a page picker, the inspector follows the selected card, and the sticky batch bar reduces to status plus one primary action. Field order and labels remain identical. No horizontal page canvas is required to complete review; the visual page boundary remains visible as a labeled divider.

## WYSIWYG contract

The preview must match the ACORN projection for:

- title-page month, reporting interval, and General updates;
- project title, subtitle, keywords, investigators, and sponsors as read-only context;
- status-update-first ordering and completed/in-progress/upcoming grouping;
- entry dates, category/health labels, details, paragraphs, lists, evidence, and risk;
- inclusion/exclusion, project boundaries, empty-section behavior, and batch order;
- validation blockers that would prevent the same projection from being accepted locally.

Pixel-identical line breaks, font metrics, and automatic page breaks are explicitly out of scope. The app must not claim final PDF fidelity until the same frozen JSONL has been rendered by ACORN and compared in acceptance.

# Usability and deferred accessibility

- Use responsive auto-layout containers with Scale to fit disabled; do not position the main workflow with fixed coordinates.
- Keep the existing native controls, labels, focus styles, text state cues, and live validation behavior in the accepted reviewer prototype.
- Usability acceptance includes a contributor and reviewer completing their primary task without implementation coaching; observed confusion becomes a bounded repair phase.

The user deferred all further accessibility features and gates on November 8, 2026. Screen-reader, 200% zoom, keyboard-only, contrast, and accessibility-checker acceptance are not prerequisites for PA-39 through PA-41. Existing behavior is preserved; this deferral does not claim those checks passed.

# Dependencies and boundaries

PA-34, PA-35, PA-35A, PA-35B, the PA-35C repair, and PA-39 are complete. PA-39A next preserves correction and recovery behavior in the refit. The approval ledger may proceed in parallel, but production bindings wait for its accepted contracts. This plan never changes production lists, flows, exporter selection, or permissions.

Reviewer edits may change only fields admitted by the frozen form/report contract. Project routing, submitter, source receipt, correction ancestry, source/approval UUIDs, and audit timestamps are read-only. Editorial synthesis proposed by [Sponsor highlights clarity](../../sponsor-highlights-clarity.md) is not added here until its schema and renderer are separately accepted.

# Acceptance criteria

- One published component-library revision supplies every shared control used by both isolated apps.
- The field mapping covers every admitted activity and status-update field and every visible report element; unknown fields fail visibly.
- The reviewer can complete title-page and project-page review from the report workspace, with original-versus-edited inspection and no raw-JSON editing.
- Contributor activity, correction, browser recovery, retry, validation, and catalog behavior remain byte-compatible for unchanged inputs.
- Both apps pass formula/runtime App Checker and desktop, tablet, and 375-pixel checks.
- Fixture previews and actual ACORN PDFs match the WYSIWYG contract, with bounded differences recorded.

# Execution phases

## PA-39A — Preserve correction, recovery, and save-status behavior

- Status: in-progress
- Budget: 5 active hours maximum
- Prerequisites: accepted contributor activity refit recorded in the current evidence
- Exact write set: isolated contributor app `Sponsor Highlights Intake UI TEST ONLY`; `Sponsor Highlights Intake Fixture TEST ONLY` flow; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/submission-editing/build-app.py`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/intake-refit/correction-recovery.pa.yaml`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/intake-refit/correction-recovery.msapp`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/intake-refit/recovery-acceptance.md`
- Primary integration lock: contributor app canvas
- Outcome: Rebind history/correction, immutable routing, browser recovery, exact retry, conflict, rejection, export-status, and success panels to the shared components and language without weakening existing guards.
- Focused validation: Original and correction submission, pending browser restart, other-user recovery block, received/export retry, conflict, rejected edit, catalog failure, and unchanged notification inputs.
- Handoff: Preserve exact request/response evidence, isolated app package, and any bounded wording differences requiring guide updates.

## PA-39B — Accept cross-app visual and interaction consistency

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted reviewer fixture experience recorded in the current evidence; PA-39A
- Exact write set: TEST ONLY apps `[PROTOTYPE] Intake Approval` and `Sponsor Highlights Intake UI TEST ONLY`; TEST ONLY component library `Sponsor Highlights UI TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/acceptance/cross-app-ui.md`
- Primary integration lock: Power Apps shared UI contract
- Outcome: Accept one component revision, token set, terminology, error language, responsive behavior, supporting-detail control behavior, and report-preview vocabulary across both isolated apps. Repair or explicitly revise the `cmpRepeatableText` contract before freezing the shared revision.
- Focused validation: Side-by-side component inventory, supporting-detail add/remove/reorder or accepted line-based behavior, scripted field-order walkthrough, formula/runtime App Checker, desktop/tablet/375-pixel screenshots, and two uncoached task observations. Accessibility checks remain deferred at the user's direction.
- Handoff: Freeze the cross-app UI revision consumed by PA-40. Any material usability defect becomes a new bounded repair phase.
