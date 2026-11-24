---
type: Plan
title: "Adopt shadcn-style components with UI SDK in acorn-web without changing its appearance"
description: "Use locally bundled UI SDK behaviors for eligible Dioxus and React controls while preserving ACORN's visual, accessibility, and single-WASM contracts."
difficulty: high
impact: medium
package: [acorn-web]
tags: [migration, shadcn, ui-sdk, dioxus, react, visual-parity, accessibility]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Adopt reusable, unstyled UI SDK behavior where it improves the existing browser UI, while retaining ACORN's current appearance: layout, typography, colors, borders, spacing, shadows, icons, responsive behavior, and dark/light themes. Preserve the existing routes, operations, loaded workspace, repository credentials, map, accessibility behavior, and single-WASM release contract. “Shadcn-style” describes local component composition and styling here; UI SDK is a distinct library and must be named as such.

The main application remains Dioxus. Test locally bundled `@ui-sdk/web` custom elements behind typed Dioxus wrappers for stateful interactions; Dioxus continues to own application state, routing, and effects. The existing React geospatial island can use `@ui-sdk/react` directly. Keep simple native buttons, fields, cards, tables, and specialized integrations when a component would add no behavior or fail parity. Do not describe UI SDK or a Dioxus wrapper as official shadcn/ui. Converting the whole application to React is a separate architecture migration.

# Current evidence — November 24, 2026

- **Verified:** `crates/acorn-web/Cargo.toml:20` pins Dioxus `=0.7.10`; `crates/acorn-web/src/main.rs:1-20,140-195` composes Rust routes and browser assets. `crates/acorn-web/geospatial/package.json:10-23` pins React `19.3.1` and Resium `1.28.0`; `geospatial/src/map.tsx:80-120` contains native map-layer radio, checkbox, and disclosure controls. Neither npm manifest has UI SDK, shadcn/ui, or Tailwind dependencies. No Dioxus custom-element adapter or UI SDK build exists in the inspected browser source.
- **Verified:** `crates/acorn-web/assets/app.css:1-20,59-60,498-499,618-650,806-829` defines the current theme, cards, Portfolio list/card geometry, button depth, drawer, and tabs. `src/workspace.rs:198-327` uses native search/select controls, project cards, an activity table, and a local carousel. `src/details.rs:184-316` couples a modal `<dialog>`, resize handle, tabs, and source/checks panels to the details drawer. `src/ui.rs:413-479` owns sidebar navigation, theme, notifications, and route focus.
- **Verified:** `crates/acorn-web/scripts/parity-smoke.mjs:73,151-228` already captures browser screenshots and checks catalog/drawer behavior. `Makefile:485-558` provides the web build, checks, tests, and Pages smoke. The worktree contains uncommitted browser/catalog and wiki changes; each phase must recheck its baseline and coordinate exact write sets before editing.
- **Verified external sources (opened November 24):** The November 12 [UI SDK introduction](https://ui-sdk.dev/docs/core/changelog/introducing-ui-sdk) describes unstyled behaviors for keyboard, focus, selection, popups, drag and drop, and ARIA, with React and Web Component adapters. [Web Component installation](https://ui-sdk.dev/docs/web-components/installation) specifies `@ui-sdk/web` plus peer `@ui-sdk/core`, per-component imports, and a self-hosted browser bundle; [state and events](https://ui-sdk.dev/docs/web-components/state-and-events) documents live properties and non-bubbling change events. [Dioxus 0.7 Web Components guidance](https://dioxuslabs.com/learn/0.7/essentials/ui/escape/#web-components) permits dashed custom elements in RSX and recommends typed wrappers. UI SDK's [Web Component catalog](https://ui-sdk.dev/docs/web-components/components) includes carousel, drawer, resizable, tabs, radio, checkbox, select, and toggle. Its [React radio](https://ui-sdk.dev/docs/react/components/radio) and [checkbox](https://ui-sdk.dev/docs/react/components/checkbox) components are unstyled candidates for the map widget. UI SDK labels itself **beta**; package version, license, Dioxus event/property bridge, CSS layering, and bundle cost remain unverified locally.
- **Inference:** the documented Web Component route may let Dioxus use UI SDK behaviors without a second app framework or WASM payload, but documentation alone does not prove event ownership, hydration, focus, CSS, or release compatibility. SCU-02 must demonstrate those contracts in this repository. A vendor default stylesheet or Tailwind preflight would change ACORN's appearance. No UI SDK migration or visual comparison has yet been run.

# Component disposition

| Existing surface | Planned component path | Appearance and behavior constraint |
|---|---|---|
| Simple buttons, cards, fields, and text inputs | Keep native Dioxus markup; use typed wrappers where repeated behavior warrants them | UI SDK's Button adds pending/async behavior; plain click buttons need no wrapper. Preserve types, disabled/busy states, labels, classes, and focus rings. |
| Theme choice and Portfolio list/cards toggle | `@ui-sdk/web` toggle or segmented control through typed Dioxus wrappers after SCU-02 | Rust state remains authoritative; verify non-bubbling events, pressed/selected state, keyboard behavior, and existing CSS geometry. |
| Portfolio filters, project cards, and activity table | Evaluate UI SDK select only if it matches the native closed and open controls; keep cards and semantic table native | Preserve density, gaps, green links, sort announcements, and filter behavior across target browsers. |
| Details tabs and loaded-tree disclosures | Evaluate UI SDK Web Component tabs and collapsible controls | Preserve panel content, focus, announcements, recursive expansion, and selected-file state. |
| Modal details drawer | Evaluate UI SDK drawer and resizable behaviors only after tabs/tree parity | Preserve native modal, width, animation, Escape/focus return, responsive mode, keyboard resize, and scroll behavior. |
| Project/activity image carousel | Compare UI SDK's carousel with the current specialized carousel during Portfolio validation | UI SDK now has a carousel, but adoption still requires one-visible-slide, swipe, buttons, placeholders, and exact visual parity. |
| CodeMirror, Driver.js, and Cesium controls | Retain specialized implementation | Their application-specific behavior remains outside generic UI primitives. |
| React map layer widget | Prefer unstyled `@ui-sdk/react` radio and checkbox where parity holds; retain native disclosure | Keep React/Resium bridge and Cesium controls; scope styles to the map bundle and avoid Tailwind or a global reset. |

After each migration, remove obsolete markup/style/handlers only when the replacement passes parity. Remaining native elements are an intentional result when replacement adds no useful behavior or loses existing behavior. Bundle pinned UI SDK code locally; do not load a remote CDN at runtime, add a vendor theme or global Tailwind reset, register UI SDK's optional agent/WebMCP tools, add a second application runtime, or duplicate Rust application state. Existing ACORN WebMCP tools and permissions remain authoritative.

# Dependencies and coordination

- Lane: [Rust browser and desktop workspace](../parallel-execution.md). SCU phases serialize with ABW, [Research Activity Index](../features/research-activity-index/index.md), [browser performance](../refactors/browser-performance.md), [frontend build quality](../testing/frontend-build-quality.md), and assisted RAD work whenever their exact Rust, CSS, map, package, or test files overlap. No phase silently edits a file held by another active implementation.
- SCU-01 freezes the visual baseline from a stable browser/catalog revision. It can begin as read-only inventory now, but screenshots used for comparison must be recaptured after the current uncommitted catalog work settles. SCU-02 must prove a pinned, locally bundled UI SDK Web Component and its Dioxus event/property bridge before SCU-03–06/08. A failed proof stops the UI SDK route and requires this plan to be revised before downstream implementation. SCU-07 can proceed after SCU-01 on an isolated map write set. SCU-09 follows all component phases.
- The UI SDK proof owns npm manifests, generated-asset staging, and browser component files under one browser integration lock; serialize its `Makefile` recipe with the CI/build owner. It does not assume a Dioxus or CLI upgrade. If a Rust dependency change becomes necessary, coordinate the workspace graph and browser build locks with ABW-11 desktop embedding in a revised bounded phase; do not advance the whole workspace dependency graph solely for this migration.
- ABW-19 hosted catalog acceptance and ABW-44/18 map work retain their own acceptance gates. The migration neither counts as their completion nor changes their data contract. The React layer-widget phase waits for any concurrent ABW map edit to land.
- No hosted publication, private token, or live NSSD bucket is needed for component implementation; final hosted acceptance, if requested later, remains separate from local parity.

# Validation and acceptance

1. Capture a fixed baseline at the same revision, browser version, viewport, zoom, font environment, data fixture, theme, and component state as the candidate. Cover Home, Portfolio list/cards (including images and missing-image placeholder), details overview/source/checks, loaded tree, each tool's input/result shell, Settings, About, and the map layer widget. Include desktop, 390px mobile, 200% zoom, dark/light, hover/focus/open/disabled/error states, and reduced motion where relevant.
2. Require **zero unexplained visual differences** in the reviewed screenshot pairs. Automated diffs flag changed pixels; human review resolves antialiasing or platform noise. No planned restyling is accepted as migration progress. Also compare computed geometry, colors, typography, and visible text for controls that browser screenshots cannot capture reliably, such as native popup menus.
3. Verify keyboard order, Tab/Shift+Tab, Enter/Space, arrows for tabs/radio/select, Escape, focus return, labels, roles, selected/expanded/pressed state, disabled/busy state, screen-reader announcements where available, and pointer/touch controls. Prove that custom-element non-bubbling events and live properties round-trip through Dioxus without stale or duplicated state. Preserve existing mouse/touch slideshow and drawer resizing behavior.
4. Run focused `cargo test -p acorn-web`, WASM/Clippy checks from `make web-check`, existing JS/map tests, the relevant `ACORN_PARITY_SCOPE` browser checks, `make web-build`, and `make web-test-pages` on a disposable available port. The release must retain one WASM payload, nine operation exports, correct `/acorn/` routing and no FOUC regression. Run `make web-test` and any final required workspace gate only after the implementation phases stabilize.
5. Measure compressed/uncompressed Rust WASM, CSS, the local UI SDK JavaScript, and map JavaScript before and after. Any increase must be explained by an adopted component and accepted against browser performance budgets; no duplicate React runtime, remote runtime dependency, or unnecessary Tailwind reset is allowed. Review the pinned UI SDK release, transitive licenses, beta API changes, and generated notice before acceptance.

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

## SCU-02 — Prove a local UI SDK and Dioxus Web Component bridge

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SCU-01; exclusive browser component/asset integration lock
- Exact write set: `crates/acorn-web/package.json`, `crates/acorn-web/package-lock.json`, `crates/acorn-web/scripts/build-ui-sdk.mjs` (new), `crates/acorn-web/assets/generated/ui-sdk.js` (generated), `crates/acorn-web/assets/generated/ui-sdk.LICENSES.txt` (generated), `Makefile`, `crates/acorn-web/src/main.rs`, `crates/acorn-web/src/components.rs` (new), `crates/acorn-web/src/components/toggle.rs` (new), `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/component-parity.mjs`, `.wiki/migrations/acorn-web-shadcn-components.md`
- Primary integration lock: browser component and asset integration
- Outcome: Pin `@ui-sdk/web` and its peer `@ui-sdk/core`, bundle only the required component entries into a local asset with dependency notices, and prove one typed Dioxus toggle wrapper. Verify custom-element upgrade, non-bubbling `pressedchange`, Rust-to-element live state updates, cleanup, and ACORN styling without vendor CSS. If the bridge cannot preserve parity and one-WASM packaging, record a no-go and revise the remaining plan before migration.
- Focused validation: `make web-lint`, `make web-check`, focused Rust/JS bridge tests, `make web-build`, root/prefixed browser captures, license/dependency audit, and SCU-01 toggle keyboard/focus states. Check the local bundle's size and absence of a second app framework, remote import, or global reset.
- Handoff: Record exact UI SDK package pins, beta/API risk, event/property contract, license, JavaScript/WASM/CSS deltas, and parity evidence; remove SCU-02 and make SCU-03 ready only after the proof passes.

## SCU-03 — Migrate stateful shell and Home controls

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SCU-02
- Exact write set: `crates/acorn-web/src/components.rs`, `crates/acorn-web/src/components/toggle.rs`, `crates/acorn-web/src/ui.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/component-parity.mjs`, `.wiki/migrations/acorn-web-shadcn-components.md`
- Primary integration lock: browser shell and Home UI
- Outcome: Use the proven UI SDK wrapper for eligible theme and other stateful shell/Home choices while retaining plain native click buttons and cards. Keep Rust state authoritative and preserve markup hooks, route focus, shadows, icons, and responsive behavior.
- Focused validation: Home/shell parity captures in both themes and viewport sizes; keyboard navigation, theme persistence and tour/focus behavior; focused Rust and browser shell checks.
- Handoff: Record migrated instances and any native exceptions, remove obsolete selectors only when unused, prune SCU-03 and release the shell lock.

## SCU-04 — Evaluate loader, settings, and tool form controls

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SCU-03
- Exact write set: `crates/acorn-web/src/components.rs`, `crates/acorn-web/src/components/field.rs` (new), `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/component-parity.mjs`, `.wiki/migrations/acorn-web-shadcn-components.md`
- Primary integration lock: browser input and settings UI
- Outcome: Add typed, ACORN-styled field/input wrappers only where UI SDK supplies useful validation or accessibility behavior. Keep simple native inputs/buttons, password and repository token handling, file picker, validation timing, labels, status messages, and delete controls unchanged.
- Focused validation: Component parity captures for empty/filled/disabled/error/busy forms; local file and remote mock flows; storage and deletion smoke; focused Rust/JS tests.
- Handoff: Record migrated forms and exceptions, prune SCU-04 and release the form lock.

## SCU-05 — Migrate Portfolio choices and assess carousel

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SCU-04; no active ABW-19 or ABW-64 Portfolio write overlap
- Exact write set: `crates/acorn-web/src/components.rs`, `crates/acorn-web/src/components/field.rs`, `crates/acorn-web/src/components/segmented.rs` (new), `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/component-parity.mjs`, `.wiki/migrations/acorn-web-shadcn-components.md`
- Primary integration lock: Portfolio UI
- Outcome: Use a proven UI SDK segmented/toggle wrapper for list/cards choice and evaluate select parity for filters. Keep project cards and the activity table native. Compare UI SDK carousel behavior with the existing swipe carousel and record whether a separate bounded migration is justified; keep the existing carousel in this phase.
- Focused validation: Synthetic catalog search/sort/filter and list/cards parity; equal card gaps, dense list rows, one visible slide, swipe/arrows, missing-image placeholder, green links and dark/light captures; focused index smoke.
- Handoff: Record select decision with visual and keyboard evidence, prune SCU-05 and release the Portfolio lock.

## SCU-06 — Migrate details tabs and disclosure controls

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SCU-05
- Exact write set: `crates/acorn-web/src/components.rs`, `crates/acorn-web/src/components/tabs.rs` (new), `crates/acorn-web/src/details.rs`, `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/component-parity.mjs`, `.wiki/migrations/acorn-web-shadcn-components.md`
- Primary integration lock: details tabs and loaded-tree disclosure
- Outcome: Replace hand-wired details tab interaction with a typed UI SDK Web Component wrapper after proving Rust/event state synchronization. Use UI SDK collapsible behavior for loaded-tree disclosures only where it preserves native expansion and selected-file state. Keep the current modal drawer in place for this phase.
- Focused validation: Overview/source/checks content, arrow-key tab navigation, focus/announcement behavior, recursive tree expansion, selected file, screenshot pairs, and drawer keyboard smoke.
- Handoff: Record whether native disclosures remain, prune SCU-06 and release the tabs/tree lock.

## SCU-07 — Adopt UI SDK React controls in the map layer widget

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SCU-01; no active ABW map write overlap; compatible pinned UI SDK React build selected
- Exact write set: `crates/acorn-web/geospatial/package.json`, `crates/acorn-web/geospatial/package-lock.json`, `crates/acorn-web/geospatial/src/map.tsx`, `crates/acorn-web/geospatial/src/map.css`, `crates/acorn-web/geospatial/tests/bridge.test.mjs`, `crates/acorn-web/scripts/component-parity.mjs`, `.wiki/migrations/acorn-web-shadcn-components.md`
- Primary integration lock: React geospatial layer widget
- Outcome: Replace eligible layer-widget radio and checkbox controls with unstyled `@ui-sdk/react` parts and ACORN-matched CSS. Keep native disclosure, Cesium controls, and the Dioxus-to-map bridge. If scoped styling or interaction cannot preserve the widget, leave that control native and record a specific no-go.
- Focused validation: `npm --prefix crates/acorn-web/geospatial run check`, map tests/build, layer/terrain/token interactions, map fallback, desktop/mobile open-widget captures, keyboard/ARIA behavior, and bundle/CSS audit for style leakage outside the widget.
- Handoff: Record adopted UI SDK component names, package pins, size change, and any native exceptions; prune SCU-07 and release the map lock.

## SCU-08 — Migrate the modal drawer only if parity holds

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SCU-06; no active ABW drawer write overlap
- Exact write set: `crates/acorn-web/src/components.rs`, `crates/acorn-web/src/components/drawer.rs` (new), `crates/acorn-web/src/details.rs`, `crates/acorn-web/assets/details.js`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/component-parity.mjs`, `.wiki/migrations/acorn-web-shadcn-components.md`
- Primary integration lock: shared modal details drawer
- Outcome: Evaluate UI SDK Web Component drawer plus resizable behavior against the current native modal and replace it only if width, Escape/focus return, route close, responsive layout, scroll reset, and source/editor behavior remain intact. Otherwise retain the native dialog and document the evidence-backed exception.
- Focused validation: Drawer interaction matrix, keyboard and pointer resize, modal focus trap/return, Escape and route navigation, source download, image slide, 390px viewport, dark/light screenshots, reduced motion.
- Handoff: Record the primitive decision and accepted drawer evidence, prune SCU-08 and release the drawer lock.

## SCU-09 — Reconcile remaining controls and accept the migration

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SCU-03 through SCU-08
- Exact write set: `crates/acorn-web/src/components.rs`, `crates/acorn-web/src/results.rs`, `crates/acorn-web/src/geospatial_page.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/component-parity.mjs`, `crates/acorn-web/README.md`, `.wiki/migrations/acorn-web-shadcn-components.md`, `.wiki/migrations/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: browser component integration and final acceptance
- Outcome: Apply the proven component boundary to remaining eligible result/map-page controls, remove only confirmed unused styles, publish a durable component inventory and exception list, and accept visual/behavioral/build parity. Retain native controls when they are simpler or equivalence fails; keep specialized editor, tour, and map vendor controls. Record the carousel decision from SCU-05.
- Focused validation: Full fixed-state screenshot review, keyboard/touch/assistive checks, `make web-check`, `make web-test`, `make web-build`, `make web-test-pages`, single-WASM/export and size audit, and wiki validation. Run final broad gates only after overlapping owners finish.
- Handoff: Put durable implementation and exception guidance in `crates/acorn-web/README.md`, delete this completed plan, and remove its index, network and lane entries together.

# Completion handling

After every acceptance criterion passes, keep the exact component inventory, UI SDK version/pin rationale, local bundle and CSS ownership, visual parity procedure, and native exceptions in durable browser documentation and tests. Delete this plan and all inbound wiki references. Do not leave completed phase logs in `.wiki`.
