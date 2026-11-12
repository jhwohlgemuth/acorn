# Sponsor Highlights shared UI contract

- Contract version: `1.0.12` for the TEST ONLY app and shared-control behavior; the original PA-34 baseline remains `1.0.0`
- Power Apps component library: `Sponsor Highlights UI TEST ONLY`
- Status: TEST ONLY revision; actual published properties are recorded below
- Source baseline: ACORN `10b17f22`, November 7, 2026

This contract defines the shared controls, tokens, responsive behavior, state language, and field order used by the TEST ONLY contributor and reviewer apps. The component library contains presentation and interaction components only. It does not connect to SharePoint, call flows, make approval decisions, or mutate operational data.

The user deferred all further accessibility features and checks on November 8, 2026. Existing labels, focus cues, and live validation remain implemented; the unperformed screen-reader, zoom, contrast, and cross-app accessibility checks are not acceptance gates for the current pilot.

The reviewer app's live ledger and fixture workspace use three single-choice decision buttons: `◷ Unresolved` (amber `#8A4B08`), `✓ Approve` (ORNL green `#00662C`), and `× Exclude` (red `#A4262C`). The selected choice has a solid fill and white text; alternatives use a neutral fill with colored text and border. Hover previews the choice color, and pressing darkens it. Buttons retain zero focus-border thickness. The live ledger uses a horizontal row above 499 pixels and a full-width stack below that breakpoint; each button is 40 pixels high with 10-pixel gaps. Fixture buttons stack at narrow inspector widths.

Live decision buttons remain disabled until an item is selected and while saving. Choosing a decision changes the local draft only; Save persists it. Exclude changes the notes placeholder to a required-reason prompt, and the existing save validation remains authoritative. The hidden native decision control preserves the existing save payload contract. Fixture decisions remain local-only, with the required exclusion reason and inclusion state shown before Next problem. Report cards continue to repeat the decision in text and a color band. The [usability record](acceptance/usability.md) records the current desktop and 375-pixel checks.

After a confirmed live-ledger save (`saved` or `unchanged`) with `approve` selected, the app invokes the existing AI summary generation action automatically. The trigger runs only when the save state is `Saved` and generation is not already in progress. Conflicts, validation failures, service failures, and non-approved decisions do not trigger it. Generation remains a separate suggestion: Use suggestion changes the local summary and a later Save persists it. Generation errors leave the confirmed review intact, and After an initial generation attempt, the panel shows Additional context (optional), limited to 2,000 characters, followed by `↻ Regenerate summary`. The action is labeled `↻ Retry summary` for error, invalid, stale, or over-limit states and is hidden while generation runs. Context is disabled during generation. The field is scoped by batch/item key and is not persisted with the review. Generation appends the typed context to the original submission under an explicit reviewer-provided-context heading; the combined 12,000-character input limit and 1,200-character output limit remain enforced. Regeneration never writes the review summary or decision. Use suggestion updates the local summary, resets its input, and marks the review dirty; Save is still required. The panel sizes to its visible children without nested scrolling. Fixture decisions remain local-only.

The November 12 compact typography revision uses Power Apps size 11 for shared input values, choices, narrative text, and status messages; app-owned reviewer body text and intake receipt/preview text follow the same scale. Supporting metadata and helper labels use size 10. Shared field labels remain size 12, contributor section titles remain size 14, and primary actions and page titles retain their existing emphasis. Control dimensions and spacing are preserved so smaller text has more breathing room.

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

The TEST ONLY implementation uses a native toggle for two choices and buttons for three choices. The intake's two toggles occupy separate left-aligned rows. Their visible values begin ten pixels after the 66-pixel switch tracks, with a shared left edge. A separate left-aligned text label sits behind the native toggle's transparent built-in text. A transparent button above the enabled switch and value supplies the pointer cursor; it is excluded from the Tab sequence and hidden when the native control is disabled. Its click changes a separate component-local toggle-default variable and resets the native toggle, whose existing check/uncheck events preserve the selected key and app callback. Component reset clears both variables. The native switch retains its announced value and keyboard target. The on state uses ORNL Energy green. The selected button remains ORNL green while its alternatives become neutral gray and remain clickable. Hovering an alternative uses ORNL green with white text; choosing another replaces the selected canonical key. Button activation updates the component-scoped selected key before invoking `OnChange`; `Reset` clears that key so `DefaultKey` can initialize the next record. The contributor placement control displays `General update` while retaining the canonical key `Report-wide update (no associated project)`.

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

The TEST ONLY implementation presents mutually exclusive choices in a two-column gallery of icon buttons; the native Radio control is hidden. Selected buttons use ORNL green. Once a selection exists, alternative buttons use neutral gray fill and text but remain clickable. On hover they use ORNL green (`#00662C`) with white text, returning to gray when the pointer leaves. Activating another button replaces the selected canonical key. The intake sizes its health and risk galleries to show every option without inner scrolling.

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
| `ColorChoiceMuted` | `#F1F2F3` | Unselected alternatives after a single choice is selected; alternatives remain clickable. |
| `ColorChoiceMutedText` | `#6B7074` | Text on unselected gray choice buttons. |
| `ColorDanger` | `#A4262C` | Errors and destructive warnings. |
| `ColorFocus` | `#006BA6` | ORNL Infinity: three-pixel focus border on text fields and dropdowns; matching hover border where supported. Buttons and toggles have no focus outline. |
| `ColorMutedText` | `#52685F` | Secondary text on white or background surfaces. |
| `ColorPrimary` | `#007833` | Primary actions and selected accents. |
| `ColorPrimarySoft` | `#D9EDDF` | Selected and success-adjacent background. |
| `ColorSurface` | `#FFFFFF` | Cards, fields, and report pages. |
| `ColorSurfaceSoft` | `#EDF4EF` | Secondary controls and grouped regions. |
| `ColorText` | `#123B31` | Primary text. |
| `ColorToggleOn` | `#7DBA00` | ORNL Energy: intake toggle on state and on-state hover. |
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
| `FocusWidthButtonToggle` | `0` |
| `FontBody` | `Segoe UI` |
| `FontSizeBody` | `11` |
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

The current [BETA] library and both isolated apps were published November 12, 2026. Both apps adopted the library revision published at 15:01:41 Eastern. Shared narrative fields have complete bottom clearance; choice buttons use Title Case, green selected/hover states, and gray unselected states. Two-choice toggles show both states and derive their canonical output from the native switch. Buttons and toggles have no focus outline at the owner’s request; fields retain blue focus borders. Accessibility acceptance remains deferred.

The current contributor order is Submission Type, Report Placement, Project when applicable, Date, mode-specific options, narrative fields, Report Preview with validation, then Submit. Project labels use catalog titles and retain stable IDs. Current live review editing supports summary, decision, and decision reason; the broader inspector field matrix above remains the target contract, not a claim that every source field is editable in the beta.

Both reviewer views generate an AI suggestion after an approved Save and support explicit application and contextual regeneration through the standard SharePoint request path. See [beta verification](beta-repair/verification.md) for publication, visual checks, runtime evidence, and participant acceptance still required.
