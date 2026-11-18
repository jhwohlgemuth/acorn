---
type: Plan
title: "Adopt shadcn-style components in acorn-web without changing its appearance"
description: "Replace eligible acorn-web controls with shadcn-style Dioxus components and use official shadcn/ui only in the existing React map island, preserving the current visual and functional contract."
difficulty: high
impact: medium
package: [acorn-web]
tags: [migration, shadcn, dioxus, react, visual-parity, accessibility]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Adopt reusable shadcn-style components wherever they improve the existing browser UI, while retaining the current appearance: layout, typography, colors, borders, spacing, shadows, icons, responsive behavior, and dark/light themes. Preserve the existing routes, operations, loaded workspace, repository credentials, map, accessibility behavior, and single-WASM release contract.

The main application remains Dioxus. Its components use the Dioxus Components project's copied Rust components and unstyled primitives, adapted to ACORN's CSS. The existing React geospatial island is the only place for **official React shadcn/ui** components. This distinction is part of the outcome: a Dioxus analogue must never be reported as the official React package. Converting the whole application to React is a different architecture migration and is excluded here.

# Current evidence — November 17, 2026

- **Verified:** `crates/acorn-web/Cargo.toml:19` pins Dioxus `=0.7.3` with web/router features; `crates/acorn-web/src/main.rs:10-21,72-87` composes the Rust routes and browser assets. `crates/acorn-web/geospatial/package.json:13-21` and `geospatial/src/map.tsx:80-109` show a separate React/Resium island containing the layer widget. There is no shadcn or Tailwind dependency in those manifests.
- **Verified:** `crates/acorn-web/assets/app.css:1-20,59-60,498-499,618-650,806-829` defines the current theme, cards, Portfolio list/card geometry, button depth, drawer, and tabs. `src/workspace.rs:198-327` uses native search/select controls, project cards, an activity table, and a local carousel. `src/details.rs:184-316` couples a modal `<dialog>`, resize handle, tabs, and source/checks panels to the details drawer. `src/ui.rs:413-479` owns sidebar navigation, theme, notifications, and route focus.
- **Verified:** `crates/acorn-web/scripts/parity-smoke.mjs:73,151-228` already captures browser screenshots and checks catalog/drawer behavior. `Makefile:485-558` provides the web build, checks, tests, and Pages smoke. The worktree contains uncommitted browser/catalog and wiki changes; each phase must recheck its baseline and coordinate exact write sets before editing.
- **Verified external sources (opened November 17):** [shadcn/ui manual installation](https://ui.shadcn.com/docs/installation/manual) requires React/TSX imports and Tailwind styling. [Dioxus Components README](https://github.com/DioxusLabs/dioxus-components#readme) describes copied shadcn-style Dioxus components built on unstyled primitives. Its [component catalog](https://dioxuslabs.github.io/dioxus-components/) includes button, card, input, select, tabs, sheet, switch, and other controls, but no carousel in the inspected catalog. The current upstream [workspace manifest](https://github.com/DioxusLabs/dioxus-components/blob/main/Cargo.toml) uses Dioxus `0.7.8`; compatibility with ACORN's exact `0.7.3` pin is **unresolved** until SCU-02.
- **Inference:** applying the vendor's default theme or Tailwind preflight would change ACORN's appearance. A visually matched component can still change DOM, focus, keyboard, or screen-reader behavior, so visual and interaction checks are both required. No component migration or visual comparison has yet been run.

# Component disposition

| Existing surface | Planned component path | Appearance and behavior constraint |
|---|---|---|
| Shared buttons, cards, fields, text inputs, theme choices | Dioxus copied components or thin wrappers over unstyled primitives | Keep existing class hooks and CSS values; preserve button types, disabled/busy states, names, and focus rings. |
| Portfolio filter selects, list/cards toggle, project cards and activity table | Native-select-compatible Dioxus wrapper, toggle group, card; keep semantic table | Preserve compact list density, equal card gaps, green link colors, sort announcements, and filter behavior. A custom select is accepted only if it matches both the closed control and open-menu behavior across target browsers. |
| Details tabs and modal drawer | Dioxus tabs; consider sheet/dialog primitive only if it preserves the native modal and resize contract | Keep panel width, animation, Escape/focus return, responsive full-width mode, keyboard resizing, and exact content/scroll behavior. |
| Loaded file tree and disclosure groups | Dioxus collapsible only where it reproduces current native `<details>` behavior | Preserve recursive expansion and selected-file state. |
| Project/activity image carousel, CodeMirror, Driver.js, Cesium controls | Retain specialized implementation | The inspected Dioxus catalog has no equivalent carousel; these integrations have behavior beyond a generic UI primitive. |
| React map layer widget | Official shadcn/ui radio/checkbox/disclosure primitives where a visually matched replacement works | Scope generated utilities to the map bundle; do not install global Tailwind reset or alter Cesium controls. |

After each migration, remove the specific obsolete markup/style/handler only when its replacement passes parity. Remaining native elements are an intentional result when replacing them would change the appearance or lose behavior. Do not add a default shadcn theme, a global Tailwind reset, a second application runtime, or a duplicate state store.

# Dependencies and coordination

- Lane: [Rust browser and desktop workspace](../parallel-execution.md). SCU phases serialize with ABW, [Research Activity Index](../features/research-activity-index/index.md), [browser performance](../refactors/browser-performance.md), [frontend build quality](../testing/frontend-build-quality.md), and assisted RAD work whenever their exact Rust, CSS, map, package, or test files overlap. No phase silently edits a file held by another active implementation.
- SCU-01 freezes the visual baseline from a stable browser/catalog revision. It can begin as read-only inventory now, but screenshots used for comparison must be recaptured after the current uncommitted catalog work settles. SCU-02 must pass before component migration. SCU-03 through SCU-08 follow in order, except SCU-07 may run after SCU-01 with an isolated map write set and no overlapping map owner. SCU-09 follows all component phases.
- A Dioxus version change, if required, owns `Cargo.lock`, `crates/acorn-web/Cargo.toml`, and the pinned CLI/build contract under the workspace graph and browser build locks. Coordinate with ABW-11 desktop embedding before changing the shared UI version. Do not advance the whole workspace's dependency graph solely to make this migration possible.
- ABW-19 hosted catalog acceptance and ABW-44/18 map work retain their own acceptance gates. The migration neither counts as their completion nor changes their data contract. The React layer-widget phase waits for any concurrent ABW map edit to land.
- No hosted publication, private token, or live NSSD bucket is needed for component implementation; final hosted acceptance, if requested later, remains separate from local parity.

# Validation and acceptance

1. Capture a fixed baseline at the same revision, browser version, viewport, zoom, font environment, data fixture, theme, and component state as the candidate. Cover Home, Portfolio list/cards (including images and missing-image placeholder), details overview/source/checks, loaded tree, each tool's input/result shell, Settings, About, and the map layer widget. Include desktop, 390px mobile, 200% zoom, dark/light, hover/focus/open/disabled/error states, and reduced motion where relevant.
2. Require **zero unexplained visual differences** in the reviewed screenshot pairs. Automated diffs flag changed pixels; human review resolves antialiasing or platform noise. No planned restyling is accepted as migration progress. Also compare computed geometry, colors, typography, and visible text for controls that browser screenshots cannot capture reliably, such as native popup menus.
3. Verify keyboard order, Tab/Shift+Tab, Enter/Space, arrows for tabs/radio/select, Escape, focus return, labels, roles, selected/expanded/pressed state, disabled/busy state, screen-reader announcements where available, and pointer/touch controls. Preserve existing mouse/touch slideshow and drawer resizing behavior.
4. Run focused `cargo test -p acorn-web`, WASM/Clippy checks from `make web-check`, existing JS/map tests, the relevant `ACORN_PARITY_SCOPE` browser checks, `make web-build`, and `make web-test-pages` on a disposable available port. The release must retain one WASM payload, nine operation exports, correct `/acorn/` routing and no FOUC regression. Run `make web-test` and any final required workspace gate only after the implementation phases stabilize.
5. Measure compressed/uncompressed Rust WASM, CSS, and map JavaScript before and after. Any increase must be explained by an adopted component and accepted against browser performance budgets; no duplicate React runtime or unnecessary Tailwind reset is allowed.

Acceptance means the reviewed component inventory records each eligible control as migrated or as a specific, evidence-backed native/specialized exception; the application looks the same at the covered states; interactions and accessibility remain at least as capable; and the build/test contracts above pass. A visual or behavioral regression keeps its phase open or creates a bounded repair phase.

# Execution phases

## SCU-01 — Freeze component inventory and visual baseline

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: stable browser/catalog revision and an available local browser build
- Exact write set: `crates/acorn-web/scripts/component-parity.mjs` (new), `crates/acorn-web/README.md`, `.wiki/migrations/acorn-web-shadcn-components.md`
- Primary integration lock: browser component parity baseline
- Outcome: Record each candidate control, its current classes/behavior, fixed synthetic fixture and review states; add repeatable before/after screenshot and interaction capture without changing product UI.
- Focused validation: Run the new capture against the baseline and repeat it unchanged to establish deterministic geometry/pixels; verify dark/light, desktop/mobile, and loaded Portfolio/drawer states.
- Handoff: Record the frozen revision, browser/viewport settings, artifact location and any unstable states; remove SCU-01 and make SCU-02 ready.

## SCU-02 — Prove the compatible Dioxus component foundation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SCU-01; exclusive workspace graph and browser build lock
- Exact write set: `crates/acorn-web/Cargo.toml`, `Cargo.lock`, `Makefile`, `crates/acorn-web/src/main.rs`, `crates/acorn-web/src/components.rs` (new), `crates/acorn-web/src/components/button.rs` (new), `crates/acorn-web/assets/app.css`, `.wiki/migrations/acorn-web-shadcn-components.md`
- Primary integration lock: Dioxus component and version contract
- Outcome: Pin a Dioxus Components source/release compatible with the application; if necessary, update the exact Dioxus and CLI pins together. Integrate one copied, ACORN-styled button as a proof without loading the vendor theme. If compatible integration cannot preserve the one-WASM build, record a no-go and revise this plan before later phases start.
- Focused validation: `make web-check`, `cargo test -p acorn-web`, `make web-build`, license/dependency review, and the SCU-01 button screenshots and keyboard states. Check that no second Dioxus major/minor runtime or global CSS reset enters the bundle.
- Handoff: Record exact component/Dioxus/CLI versions, license, binary/CSS deltas and parity evidence; remove SCU-02 and make SCU-03 ready.

## SCU-03 — Migrate shell and Home controls

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SCU-02
- Exact write set: `crates/acorn-web/src/components.rs`, `crates/acorn-web/src/components/button.rs`, `crates/acorn-web/src/components/card.rs` (new), `crates/acorn-web/src/ui.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/component-parity.mjs`, `.wiki/migrations/acorn-web-shadcn-components.md`
- Primary integration lock: browser shell and Home UI
- Outcome: Adopt the styled Dioxus button/card boundary for Home actions, navigation controls, theme control, and Home cards while retaining current markup hooks, route focus, shadows, icons, and responsive behavior.
- Focused validation: Home/shell parity captures in both themes and viewport sizes; keyboard navigation, theme persistence and tour/focus behavior; focused Rust and browser shell checks.
- Handoff: Record migrated instances and any native exceptions, remove obsolete selectors only when unused, prune SCU-03 and release the shell lock.

## SCU-04 — Migrate loader, settings, and tool form controls

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SCU-03
- Exact write set: `crates/acorn-web/src/components.rs`, `crates/acorn-web/src/components/field.rs` (new), `crates/acorn-web/src/components/button.rs`, `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/component-parity.mjs`, `.wiki/migrations/acorn-web-shadcn-components.md`
- Primary integration lock: browser input and settings UI
- Outcome: Apply ACORN-styled field/input/button components to local/remote loaders, Settings and the shared tool input shell. Keep password and repository token handling, file picker, validation timing, form labels, status messages, and delete controls unchanged.
- Focused validation: Component parity captures for empty/filled/disabled/error/busy forms; local file and remote mock flows; storage and deletion smoke; focused Rust/JS tests.
- Handoff: Record migrated forms and exceptions, prune SCU-04 and release the form lock.

## SCU-05 — Migrate Portfolio controls and cards

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SCU-04; no active ABW-19 or ABW-64 Portfolio write overlap
- Exact write set: `crates/acorn-web/src/components.rs`, `crates/acorn-web/src/components/card.rs`, `crates/acorn-web/src/components/field.rs`, `crates/acorn-web/src/components/toggle_group.rs` (new), `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/component-parity.mjs`, `.wiki/migrations/acorn-web-shadcn-components.md`
- Primary integration lock: Portfolio UI
- Outcome: Use the component boundary for filters, list/cards toggle, and project cards. Preserve native select semantics unless a replacement matches the browser popup and keyboard contract; retain the activity table and specialized swipe carousel.
- Focused validation: Synthetic catalog search/sort/filter and list/cards parity; equal card gaps, dense list rows, one visible slide, swipe/arrows, missing-image placeholder, green links and dark/light captures; focused index smoke.
- Handoff: Record select decision with visual and keyboard evidence, prune SCU-05 and release the Portfolio lock.

## SCU-06 — Migrate details tabs and disclosure controls

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SCU-05
- Exact write set: `crates/acorn-web/src/components.rs`, `crates/acorn-web/src/components/tabs.rs` (new), `crates/acorn-web/src/details.rs`, `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/component-parity.mjs`, `.wiki/migrations/acorn-web-shadcn-components.md`
- Primary integration lock: details tabs and loaded-tree disclosure
- Outcome: Replace the hand-wired details tab interaction with a Dioxus tabs component, and adopt collapsible behavior for loaded-tree disclosures only where it preserves native expansion and selected-file state. Keep the current modal drawer in place for this phase.
- Focused validation: Overview/source/checks content, arrow-key tab navigation, focus/announcement behavior, recursive tree expansion, selected file, screenshot pairs, and drawer keyboard smoke.
- Handoff: Record whether native disclosures remain, prune SCU-06 and release the tabs/tree lock.

## SCU-07 — Adopt official shadcn in the React map layer widget

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SCU-01; no active ABW map write overlap; compatible React/shadcn build selected
- Exact write set: `crates/acorn-web/geospatial/package.json`, `crates/acorn-web/geospatial/package-lock.json`, `crates/acorn-web/geospatial/components.json` (new), `crates/acorn-web/geospatial/vite.config.ts`, `crates/acorn-web/geospatial/src/main.tsx`, `crates/acorn-web/geospatial/src/map.tsx`, `crates/acorn-web/geospatial/src/map.css`, `crates/acorn-web/geospatial/src/shadcn.css` (new), `crates/acorn-web/geospatial/src/lib/utils.ts` (new), `crates/acorn-web/geospatial/src/components/ui/radio-group.tsx` (new), `crates/acorn-web/geospatial/src/components/ui/checkbox.tsx` (new), `crates/acorn-web/geospatial/tests/bridge.test.mjs`, `crates/acorn-web/scripts/component-parity.mjs`, `.wiki/migrations/acorn-web-shadcn-components.md`
- Primary integration lock: React geospatial layer widget
- Outcome: Replace the layer widget's radio and checkbox controls with official copied shadcn/ui React source and ACORN-matched styles. Keep native disclosures, Cesium controls, and the Dioxus-to-map bridge. Generate only the utilities required by this island; avoid a global Tailwind reset. If scoped styling cannot preserve the existing widget, leave it native and record a specific no-go rather than change its appearance.
- Focused validation: `npm --prefix crates/acorn-web/geospatial run check`, map tests/build, layer/terrain/token interactions, map fallback, desktop/mobile open-widget captures, and bundle/CSS audit for style leakage outside the widget.
- Handoff: Record adopted official component names and any exceptions, prune SCU-07 and release the map lock.

## SCU-08 — Migrate the modal drawer only if parity holds

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SCU-06; no active ABW drawer write overlap
- Exact write set: `crates/acorn-web/src/components.rs`, `crates/acorn-web/src/components/sheet.rs` (new), `crates/acorn-web/src/details.rs`, `crates/acorn-web/assets/details.js`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/component-parity.mjs`, `.wiki/migrations/acorn-web-shadcn-components.md`
- Primary integration lock: shared modal details drawer
- Outcome: Evaluate the Dioxus sheet/dialog primitive against the current native modal and replace it only if resizable width, Escape/focus return, route close, responsive layout, scroll reset, and source/editor behavior all remain intact. Otherwise retain the native dialog and document the evidence-backed exception; this is an accepted outcome, not a visual compromise.
- Focused validation: Drawer interaction matrix, keyboard and pointer resize, modal focus trap/return, Escape and route navigation, source download, image slide, 390px viewport, dark/light screenshots, reduced motion.
- Handoff: Record the primitive decision and accepted drawer evidence, prune SCU-08 and release the drawer lock.

## SCU-09 — Reconcile remaining controls and accept the migration

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SCU-03 through SCU-08
- Exact write set: `crates/acorn-web/src/components.rs`, `crates/acorn-web/src/results.rs`, `crates/acorn-web/src/geospatial_page.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/component-parity.mjs`, `crates/acorn-web/README.md`, `.wiki/migrations/acorn-web-shadcn-components.md`, `.wiki/migrations/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: browser component integration and final acceptance
- Outcome: Apply the shared component boundary to remaining eligible result/map-page controls, remove only confirmed unused styles, publish a durable component inventory and exception list, and accept visual/behavioral/build parity. Retain specialized editor, carousel, tour, map vendor controls, and native elements backed by failed-equivalence evidence.
- Focused validation: Full fixed-state screenshot review, keyboard/touch/assistive checks, `make web-check`, `make web-test`, `make web-build`, `make web-test-pages`, single-WASM/export and size audit, and wiki validation. Run final broad gates only after overlapping owners finish.
- Handoff: Put durable implementation and exception guidance in `crates/acorn-web/README.md`, delete this completed plan, and remove its index, network and lane entries together.

# Completion handling

After every acceptance criterion passes, keep the exact component inventory, version/pin rationale, CSS ownership, visual parity procedure, and native exceptions in durable browser documentation and tests. Delete this plan and all inbound wiki references. Do not leave completed phase logs in `.wiki`.
