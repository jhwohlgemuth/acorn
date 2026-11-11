# Sponsor Highlights shared UI contract

- Contract version: `1.0.7` for the TEST ONLY app and shared-control behavior; the original PA-34 baseline remains `1.0.0`
- Power Apps component library: `Sponsor Highlights UI TEST ONLY`
- Status: TEST ONLY revision; actual published properties are recorded below
- Source baseline: ACORN `10b17f22`, November 7, 2026

This contract defines the shared controls, tokens, responsive behavior, state language, and field order used by the TEST ONLY contributor and reviewer apps. The component library contains presentation and interaction components only. It does not connect to SharePoint, call flows, make approval decisions, or mutate operational data.

The user deferred all further accessibility features and checks on November 8, 2026. Existing labels, focus cues, and live validation remain implemented; the unperformed screen-reader, zoom, contrast, and cross-app accessibility checks are not acceptance gates for the current pilot.

PA-39C changed the reviewer app's own decision controls and report-card styling without changing the published component library. The fixture decision bar reads `Needs review`, `Approve`, and `Exclude`; its selected option adds a check mark and the matching amber, green, or red state color. Every report card repeats the decision in text and a full-width color band, followed by the page name. At narrow inspector widths, the decision buttons stack at full width. An excluded item shows its required reason field and the inclusion state before Next problem. The [cross-app acceptance record](acceptance/cross-app-ui.md) contains desktop and 375-pixel proof.

## Authoritative sources

- `crates/acorn-lib/src/io/api/powerautomate/form.rs` defines the admitted form envelope and the activity and status-update shapes.
- `crates/acorn-schema/src/research_activity/logbook.rs` defines canonical values, limits, nested validation, and report context.
- `crates/acorn-schema/src/types.rs` defines classification values.
- `crates/acorn-lib/src/io/export/updates/html.rs` defines visible report content and ordering.
- `../layout.pa.yaml` records the current contributor payload, recovery behavior, and fixed vocabularies.
- `report-preview-map.json` is the machine-readable field-to-control and field-to-report map for this contract.

Unknown fields fail visibly. The apps show `Unsupported field: <path>` as a publication blocker and retain the source value for diagnostics. They never silently discard or reinterpret an unknown field.

## Component properties

All component property names use PascalCase. The published segmented-choice control reads `Key` and `Label` from each choice record; callers preserve row order. Other choice controls retain their documented table shapes pending the PA-39 component audit.

Interactive component instances need a visible label, `AccessibleLabel`, visible focus, a minimum 44-pixel target, and text in addition to color for state. The reviewer supplies a separate visible label for `cmpSegmentedChoice`. Disabled components remain readable. Components expose an `OnChange` behavior input and never write app collections or global variables directly.

### `cmpBooleanToggle`

Use only for genuine Boolean values.

| Property | Direction | Type | Meaning |
|---|---|---|---|
| `AccessibleLabel` | Input | Text | Complete announced purpose, including both states. |
| `DefaultValue` | Input | Boolean | Source value on reset. |
| `Disabled` | Input | Boolean | Prevent changes while retaining state text. |
| `ErrorText` | Input | Text | Validation message; blank when valid. |
| `FalseLabel` | Input | Text | Meaning of the off state. |
| `HelpText` | Input | Text | Optional adjacent guidance. |
| `Label` | Input | Text | Visible field label. |
| `OnChange` | Input | Event | App-owned response to a deliberate change. |
| `Required` | Input | Boolean | Adds visible and announced required language. |
| `TrueLabel` | Input | Text | Meaning of the on state. |
| `Value` | Output | Boolean | Current value. |

### `cmpSegmentedChoice`

The published implementation supports two or three short, mutually exclusive choices. Every option is a real button. The selected option adds a check mark to its text; the component does not claim radio semantics. The buttons sit in a row above the 500-pixel app breakpoint and in a full-width stack below it. Two-choice rows have a 12-pixel horizontal gap; three-choice rows have a 4-pixel gap. Button activation updates a component-scoped selected key before invoking `OnChange`; `Reset` clears that key so `DefaultKey` can initialize the next record. The contributor placement control displays `General update` while retaining the canonical key `Report-wide update (no associated project)`.

| Property | Direction | Type | Meaning |
|---|---|---|---|
| `AccessibleLabel` | Input | Text | Announced group purpose. |
| `Choices` | Input | Table | Two or three ordered records with `Key` and `Label`. |
| `DefaultKey` | Input | Text | Canonical source key on reset. |
| `Disabled` | Input | Boolean | Prevent changes. |
| `ErrorText` | Input | Text | Validation message; blank when valid. |
| `HelpText` | Input | Text | Optional adjacent guidance. |
| `Label` | Input | Text | Visible field label. |
| `OnChange` | Input | Event | App-owned response to a deliberate change. |
| `Required` | Input | Boolean | Adds visible and announced required language. |
| `SecondChoiceTooltip` | Input | Text | Mouseover title for the second button; blank when unused. The contributor placement instance uses `Highlight with no associated project`. |
| `SelectedKey` | Output | Text | Canonical key of the selected button. |

### `cmpRadioChoice`

Use for more than five or long mutually exclusive choices. It wraps the native Radio control and preserves native keyboard behavior. Options are vertical at compact and medium widths.

| Property | Direction | Type | Meaning |
|---|---|---|---|
| `AccessibleLabel` | Input | Text | Announced group purpose. |
| `Choices` | Input | Table | Ordered choice records. |
| `DefaultKey` | Input | Text | Canonical source key on reset. |
| `Disabled` | Input | Boolean | Prevent changes. |
| `ErrorText` | Input | Text | Validation message; blank when valid. |
| `HelpText` | Input | Text | Optional adjacent guidance. |
| `Label` | Input | Text | Visible field label. |
| `OnChange` | Input | Event | App-owned response to a deliberate change. |
| `Required` | Input | Boolean | Adds visible and announced required language. |
| `SelectedKey` | Output | Text | Canonical key of the selected option. |

### `cmpCheckboxChoice`

Use for fixed multi-select values. No current admitted field uses this component. It is included so both apps share one future-safe menu grammar without misrepresenting repeatable narrative items as choices.

| Property | Direction | Type | Meaning |
|---|---|---|---|
| `AccessibleLabel` | Input | Text | Announced group purpose. |
| `Choices` | Input | Table | Ordered choice records. |
| `DefaultKeys` | Input | Table | Source keys on reset. |
| `Disabled` | Input | Boolean | Prevent changes. |
| `ErrorText` | Input | Text | Validation message; blank when valid. |
| `HelpText` | Input | Text | Optional adjacent guidance. |
| `Label` | Input | Text | Visible field label. |
| `MaxSelected` | Input | Number | Maximum selected values; zero means no extra limit. |
| `OnChange` | Input | Event | App-owned response to a deliberate change. |
| `Required` | Input | Boolean | Adds visible and announced required language. |
| `SelectedCount` | Output | Number | Current selected count. |
| `SelectedKeys` | Output | Table | Selected canonical keys in choice order. |

The component includes a visible selected count and a `Clear all` button. Each option is a native Checkbox control and is never announced as a radio option.

### `cmpSearchChoice`

Use for long or dynamic collections such as projects. It wraps a native Combo box and does not claim completeness when its source is paged or server-filtered.

| Property | Direction | Type | Meaning |
|---|---|---|---|
| `AccessibleLabel` | Input | Text | Announced selector purpose. |
| `DefaultKey` | Input | Text | Canonical source key on reset. |
| `Disabled` | Input | Boolean | Prevent changes. |
| `ErrorText` | Input | Text | Validation or loading failure. |
| `HelpText` | Input | Text | Optional adjacent guidance. |
| `Items` | Input | Table | Records with `Key`, `Label`, and optional `Description`. |
| `Label` | Input | Text | Visible field label. |
| `Loading` | Input | Boolean | Shows and announces loading state. |
| `OnChange` | Input | Event | App-owned response to a deliberate change. |
| `Required` | Input | Boolean | Adds visible and announced required language. |
| `SelectedKey` | Output | Text | Canonical key of the selected record. |

### `cmpNarrative`

Use for one text value. It wraps a multiline native Text Input and keeps count, requirement, purpose, and error text adjacent.

| Property | Direction | Type | Meaning |
|---|---|---|---|
| `AccessibleLabel` | Input | Text | Announced field purpose. |
| `DefaultValue` | Input | Text | Source value on reset. |
| `Disabled` | Input | Boolean | Prevent changes. |
| `ErrorText` | Input | Text | Validation message; blank when valid. |
| `HelpText` | Input | Text | Optional adjacent guidance. |
| `Label` | Input | Text | Visible field label. |
| `MaxLength` | Input | Number | Accepted character limit. |
| `OnChange` | Input | Event | App-owned response to a deliberate change. |
| `Required` | Input | Boolean | Adds visible and announced required language. |
| `Value` | Output | Text | Current text without implicit trimming. |

### `cmpRepeatableText`

This component remains a library stub and is not used by either accepted app. Supporting items use `cmpNarrative` in the contributor app and a native multiline text input in the reviewer app. Each nonempty line is one item; moving a line reorders items. Both apps retain the 20-item and 1,000-character-per-item limits. This line-based behavior is the accepted PA-39B contract; the table below describes a future row-based component and is not a current acceptance requirement.

| Property | Direction | Type | Meaning |
|---|---|---|---|
| `AccessibleLabel` | Input | Text | Announced collection purpose. |
| `DefaultItems` | Input | Table | Ordered source text rows. |
| `Disabled` | Input | Boolean | Prevent add, remove, edit, and reorder. |
| `ErrorText` | Input | Text | Collection validation message. |
| `HelpText` | Input | Text | Optional adjacent guidance. |
| `Label` | Input | Text | Visible collection label. |
| `MaxItems` | Input | Number | Maximum row count. |
| `MaxLength` | Input | Number | Maximum characters per row. |
| `OnChange` | Input | Event | App-owned response to a deliberate change. |
| `Required` | Input | Boolean | Adds visible and announced required language. |
| `Items` | Output | Table | Nonempty text rows in displayed order. |

Each row has an explicit label, Remove button, and Up/Down buttons. Reordering preserves focus on the moved row and is disabled at the applicable boundary.

### `cmpStatusMessage`

Use for empty, loading, save, validation, conflict, recovery, exclusion, approval, and publication states.

| Property | Direction | Type | Meaning |
|---|---|---|---|
| `Message` | Input | Text | Exact state detail supplied by the app. |
| `State` | Input | Text | `Loading`, `Success`, `Error`, or `Info`; controls text color. |

Revision 1.0.1 renders the complete `Message` in an auto-height label with `Live.Polite`. The reviewer app supplies descriptive message text and sizes the instance for the message. This published component has no action button, icon, or separate heading; recovery actions remain app-owned.

## Tokens

### Color

| Token | Value | Use |
|---|---|---|
| `ColorBackground` | `#F5F8F6` | App background. |
| `ColorBorder` | `#B7C9C0` | Neutral borders and dividers. |
| `ColorDanger` | `#A4262C` | Errors and destructive warnings. |
| `ColorFocus` | `#123B31` | Three-pixel keyboard focus border. |
| `ColorMutedText` | `#52685F` | Secondary text on white or background surfaces. |
| `ColorPrimary` | `#007833` | Primary actions and selected accents. |
| `ColorPrimarySoft` | `#D9EDDF` | Selected and success-adjacent background. |
| `ColorSurface` | `#FFFFFF` | Cards, fields, and report pages. |
| `ColorSurfaceSoft` | `#EDF4EF` | Secondary controls and grouped regions. |
| `ColorText` | `#123B31` | Primary text. |
| `ColorWarning` | `#8A4B08` | Warning text and icons. |

Text and state colors must meet 4.5:1 contrast against their assigned background. Large decorative marks do not carry required meaning.

### Geometry and type

| Token | Value |
|---|---|
| `ControlHeight` | `44` |
| `CornerCard` | `12` |
| `CornerControl` | `6` |
| `ElevationCard` | `0 2 8 rgba(18,59,49,0.12)` |
| `FocusWidth` | `3` |
| `FontBody` | `Segoe UI` |
| `FontSizeBody` | `12` |
| `FontSizeHelp` | `10` |
| `FontSizeHeading` | `18` |
| `PageMaxWidth` | `816` |
| `Space1` | `4` |
| `Space2` | `8` |
| `Space3` | `12` |
| `Space4` | `16` |
| `Space5` | `24` |
| `Space6` | `32` |

## Breakpoints and motion

| Key | Width | Behavior |
|---|---|---|
| `Compact` | `0-599` | One column; page rail becomes a page picker; inspector follows the selected card; segmented controls may wrap without reordering; sticky bar shows status and one primary action. |
| `Medium` | `600-1023` | Page picker or narrow rail; report and inspector stack when either would fall below 320 pixels. |
| `Wide` | `1024+` | Page rail, report canvas, and inspector display as three regions; report canvas retains the visual page boundary. |

Scale to fit is disabled. The 375-pixel acceptance width must complete every task without horizontal scrolling. Motion is optional, never conveys state, and is disabled when reduced motion is requested. No required transition exceeds 150 milliseconds.

## State language

| Key | Title | Default message |
|---|---|---|
| `approved` | Approved | This item is approved for the batch. |
| `conflict` | Newer changes need review | Reload the latest item before saving your decision. |
| `empty` | Nothing to review | No items match the current page and filters. |
| `error` | This item needs attention | Correct the highlighted fields before continuing. |
| `excluded` | Excluded | This item will stay in the audit history and will not appear in the report. |
| `loading` | Loading review | The batch and report context are loading. |
| `offline` | Connection interrupted | Your visible draft is retained. Reconnect before saving or finalizing. |
| `publication-blocked` | Final review blocked | Resolve every blocker before final review. |
| `saved` | Saved | Your latest review changes are saved. |
| `saving` | Saving | Saving your review changes. |
| `warning` | Review this value | Confirm the value before approving this item. |

## Field order

### Contributor activity

1. Entry type, when status intake is enabled
2. Project
3. Report placement
4. Date
5. Category
6. State
7. Event type
8. Summary
9. Supporting items
10. Validation summary and submit action

Classification is not added to contributor intake by this contract.

### Reviewer activity inspector

1. Read-only entry type, project, source identity, and correction ancestry
2. Review decision and explicit include-in-report state
3. Report placement
4. Date
5. Category
6. State
7. Event type
8. Classification
9. Summary
10. Supporting items
11. Read-only origin, discovery, archive, audit, and receipt metadata
12. Validation summary and previous/next problem actions

### Reviewer status-update inspector

1. Read-only entry type, project, source identity, timestamp, and correction ancestry
2. Review decision and explicit include-in-report state
3. Reporting window
4. Health and conditional health context
5. Summary
6. Significance
7. Evidence toggle and evidence
8. Risk toggle, risk kind, and explanation
9. Read-only audit and receipt metadata
10. Validation summary and previous/next problem actions

## Source preservation and system-controlled fields

- `projectId` is routing identity and is always read-only. Page navigation changes focus, not routing.
- `identifier`, `origin`, `discovery`, `archived`, `submitter`, `submissionId`, `submittedAt`, `revision`, receipt IDs, approval IDs, and audit timestamps are read-only.
- A missing classification is presented as Unclassified but remains `null` until the reviewer deliberately changes the classification control. Explicit `unclassified` remains explicit.
- Manual entries omit discovery. Discovered entries retain provider, source, and discovery timestamp.
- Corrections retain the root and immediate predecessor receipt identities.
- Exclusion retains source data and audit history. It changes preview visibility only through the approval decision contract.
- Trimming and normalization occur only at the existing admission boundary. Components do not trim values implicitly.

## Validation rules

- Activity summary: required, 1-1,200 characters.
- Supporting items: optional, 1-20 nonempty rows when present, at most 1,000 characters each.
- Status summary: required, 1-1,200 characters.
- Status significance: required, 1-1,000 characters.
- Evidence: optional and nonempty when present, at most 1,000 characters.
- Health context: optional for at-risk, delayed, and paused; nonempty when present, at most 1,000 characters.
- Risk explanation: required when risk is present, 1-1,000 characters.
- Reporting windows are half-open. The inclusive left timestamp must precede the exclusive right timestamp. Status updates require a bounded effective range.
- Classification requires category `administration` and event type `other`.
- Exclusion requires an exclusion reason under the approval-ledger contract.

## Report-preview rules

- The title page shows the report month, report interval, and General updates.
- Project pages show title, optional subtitle, keywords, principal investigators, and sponsors as read-only context.
- Status updates appear first in source order.
- Activity sections follow in `completed`, `in-progress`, `upcoming` order. Empty activity sections are omitted.
- Activity cards show category and icon, date, event, origin or discovery provider, non-Unclassified classification, archive state, summary paragraphs, and ordered supporting items.
- Status cards show health, timestamp, summary, significance, optional health context, optional evidence, and optional risk kind and explanation.
- The preview preserves project boundaries, general/project placement, item order, batch order, and inclusion/exclusion.
- Pixel-identical line wrapping, font metrics, and automatic PDF page breaks are outside this contract.

## Versioning and adoption

The PA-34 library baseline was published with semantic version `1.0.0`. The TEST ONLY library's live version 18, published November 10, 2026, carries the radio reset repair, `SecondChoiceTooltip`, and two-choice spacing. Both isolated apps explicitly adopted that revision. The contributor app's live version 16 uses the shorter placement label and tooltip. The reviewer prototype remains unpublished and uses the Office Green theme for its app-owned classic buttons and dropdown accents; shared segmented and narrative components retain the semantic tokens above.
