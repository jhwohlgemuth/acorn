---
type: Plan
title: "Shared intake and approval experience"
description: "Create one accessible Power Apps interaction system, a report-shaped approval workspace, and a contributor form that uses the same components and language."
difficulty: high
impact: critical
package: [power-platform]
tags: [powerapps, usability, accessibility, responsive, wysiwyg, intake, approval]
status: planned
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

# Accessibility and usability gates

- Use responsive auto-layout containers with Scale to fit disabled; do not position the main workflow with fixed coordinates.
- Prefer modern native Toggle, Radio, Checkbox, Button, Text Input, and Date Picker controls. A button bar remains buttons semantically.
- Every interactive control has a visible label, meaningful AccessibleLabel, visible focus, logical source/tab order, and keyboard-only path. Avoid positive custom TabIndex values.
- State is conveyed with text and icon in addition to color. Custom colors meet at least 4.5:1 text contrast.
- Dynamic save, validation, and conflict messages use an accessible live-status pattern supported by the platform.
- The accessibility checker has no unresolved errors. Warnings require a written disposition.
- Usability acceptance includes a contributor and reviewer completing their primary task without implementation coaching; observed confusion becomes a bounded repair phase.

# Dependencies and boundaries

PA-34 is first. PA-35 follows it and uses fixtures only. PA-35A and PA-35B complete the reviewer surface before PA-39 refits the contributor app. The approval ledger may proceed in parallel after PA-34, but production bindings wait for its accepted contracts. This plan never changes production lists, flows, exporter selection, or permissions.

Reviewer edits may change only fields admitted by the frozen form/report contract. Project routing, submitter, source receipt, correction ancestry, source/approval UUIDs, and audit timestamps are read-only. Editorial synthesis proposed by [Sponsor highlights clarity](../../sponsor-highlights-clarity.md) is not added here until its schema and renderer are separately accepted.

# Acceptance criteria

- One published component-library revision supplies every shared control used by both isolated apps.
- The field mapping covers every admitted activity and status-update field and every visible report element; unknown fields fail visibly.
- The reviewer can complete title-page and project-page review from the report workspace, with original-versus-edited inspection and no raw-JSON editing.
- Contributor activity, correction, browser recovery, retry, validation, and catalog behavior remain byte-compatible for unchanged inputs.
- Both apps pass App Checker, accessibility checker, keyboard-only, screen-reader smoke, desktop, tablet, and 375-pixel checks.
- Fixture previews and actual ACORN PDFs match the WYSIWYG contract, with bounded differences recorded.

# Execution phases

## PA-34 — Freeze the shared form and report-preview contract

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: approved interaction direction; current form schema, report renderer, and canvas sources available
- Exact write set: `.wiki/features/powerapps-jsonl-intake/approval-experience/index.md`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/ui-contract.md`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/report-preview-map.json`; TEST ONLY component library `Sponsor Highlights UI TEST ONLY`
- Primary integration lock: Power Apps shared UI contract
- Outcome: Freeze component names, inputs/outputs, tokens, breakpoints, state language, control-selection rules, field order, system-controlled fields, and field-to-report mapping.
- Focused validation: Review every current schema field and fixed vocabulary; verify radio/checkbox/button semantics; compare the mapping with `form.rs`, `logbook.rs`, `updates/html.rs`, and current canvas sources; run component harness checks at desktop and 375-pixel widths.
- Handoff: Publish a versioned contract and harness revision. PA-35 and PA-36 may start only after unresolved field or accessibility decisions are zero.

## PA-35 — Build the fixture-backed approval shell

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-34
- Exact write set: TEST ONLY canvas app `Intake Approval TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/app/approval-shell.pa.yaml`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/fixtures/batch-index.json`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/fixtures/report-context.json`
- Primary integration lock: approval app canvas
- Outcome: Deliver the responsive batch dashboard, page rail/page picker, canvas, inspector shell, sticky status bar, shared loading/empty/error states, and fixture loader with no live data mutations.
- Focused validation: Run App Checker and accessibility checker; verify keyboard traversal, persisted selection, desktop and 375-pixel layout, and exact fixture page/count display.
- Handoff: Preserve the exported TEST ONLY app and screenshots. No report card editing or live SharePoint connection is claimed.

## PA-35A — Render and edit activity report pages

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-35
- Exact write set: TEST ONLY canvas app `Intake Approval TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/app/activity-preview.pa.yaml`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/fixtures/activities.jsonl`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/acceptance/activity-preview.md`
- Primary integration lock: approval app canvas
- Outcome: Render title-page and project activity cards in report order, with inline summary editing, structured inspector controls, supporting-item editing/reordering, source comparison, validation, and include/exclude preview behavior.
- Focused validation: Compare general/project placement, three state groups, category/event/classification details, paragraphs, item order, empty groups, and errors with actual ACORN HTML/PDF from the same fixtures.
- Handoff: Preserve fixture projections, preview captures, PDF captures, and remaining renderer differences. Status updates remain for PA-35B.

## PA-35B — Complete status, correction, and responsive review behavior

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-35A
- Exact write set: TEST ONLY canvas app `Intake Approval TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/app/status-preview.pa.yaml`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/fixtures/status-and-corrections.jsonl`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/acceptance/experience.md`
- Primary integration lock: approval app canvas
- Outcome: Add status-update cards, conditional health/risk controls, reporting-window behavior, correction ancestry display, validation summary, next-problem navigation, narrow inspector stacking, and final accessibility behavior.
- Focused validation: Exercise all health states, risk/no-risk, evidence/no-evidence, source/edited comparison, correction chains, keyboard-only completion, screen-reader smoke, 200% zoom, desktop, tablet, and 375-pixel layouts.
- Handoff: Record the accepted fixture-only reviewer experience and component revision. Live save/finalize behavior belongs to the ledger plan.

## PA-39 — Refit contributor activity entry to the shared components

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-35B; isolated copy of the published contributor app and bindings
- Exact write set: isolated contributor app `Sponsor Highlights Intake UI TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/layout.pa.yaml`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/intake-refit/activity.pa.yaml`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/intake-refit/payload-parity.json`
- Primary integration lock: contributor app canvas
- Outcome: Replace fixed dropdowns with the accepted placement bar, state bar, category radio menu, event radio menu, shared narrative components, shared validation summary, and report-card preview while preserving the project selector and payload contract.
- Focused validation: Compare old/new payload bytes for unchanged activity fixtures; verify all fixed values, validation limits, project loading/retry, general placement, desktop and 375-pixel behavior.
- Handoff: Preserve the isolated app export, component version, property diff, payload parity, screenshots, and rollback package.

## PA-39A — Preserve correction, recovery, and save-status behavior

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-39
- Exact write set: isolated contributor app `Sponsor Highlights Intake UI TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/submission-editing/build-app.py`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/intake-refit/correction-recovery.pa.yaml`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/intake-refit/recovery-acceptance.md`
- Primary integration lock: contributor app canvas
- Outcome: Rebind history/correction, immutable routing, browser recovery, exact retry, conflict, rejection, export-status, and success panels to the shared components and language without weakening existing guards.
- Focused validation: Original and correction submission, pending browser restart, other-user recovery block, received/export retry, conflict, rejected edit, catalog failure, and unchanged notification inputs.
- Handoff: Preserve exact request/response evidence, isolated app package, and any bounded wording differences requiring guide updates.

## PA-39B — Accept cross-app visual and interaction consistency

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-35B; PA-39A
- Exact write set: TEST ONLY apps `Intake Approval TEST ONLY` and `Sponsor Highlights Intake UI TEST ONLY`; TEST ONLY component library `Sponsor Highlights UI TEST ONLY`; `.wiki/features/powerapps-jsonl-intake/_assets/sharepoint/intake-approval/acceptance/cross-app-ui.md`
- Primary integration lock: Power Apps shared UI contract
- Outcome: Accept one component revision, token set, terminology, focus model, error language, responsive behavior, and report-preview vocabulary across both isolated apps.
- Focused validation: Side-by-side component inventory, scripted field-order walkthrough, keyboard-only primary tasks, screen-reader smoke, accessibility/App Checker, 200% zoom, desktop/tablet/375-pixel screenshots, and two uncoached task observations.
- Handoff: Freeze the cross-app UI revision consumed by PA-40. Any material usability defect becomes a new bounded repair phase.
