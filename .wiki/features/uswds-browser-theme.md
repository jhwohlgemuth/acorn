---
type: Plan
title: Add an optional USWDS theme to the ACORN browser workspace
description: Add a selectable, persistent USWDS-inspired browser appearance grounded in official design tokens and component guidance, with accessible states across the existing workspace.
difficulty: high
impact: medium
package: [acorn-web]
tags: [browser, theme, uswds, accessibility, design-system]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal and boundary

Add **USWDS** (the U.S. Web Design System; “USDWS” in the request appears to be a transposition) as a fourth appearance choice in the Dioxus browser workspace. It is an opt-in light appearance. Retain Dark, Light, and System as existing choices and defaults. Apply the new appearance to shared chrome, forms, tables, cards, results, overlays, the code editor, and embedded map controls where ACORN owns their styling. Preserve routes, data, operations, and the current light/dark appearance.

This is a **USWDS-informed theme for ACORN**, not a claim that the entire application implements official USWDS components or meets federal conformance requirements. The linked [designer guide](https://designsystem.digital.gov/documentation/getting-started-for-designers/) points to official Figma/Sketch kits, live component HTML, design tokens, and component-specific accessibility notes. Use those as the visual and behavioral reference. The [developer guide](https://designsystem.digital.gov/documentation/getting-started-for-developers/) describes installing and compiling USWDS Sass; decide whether a scoped, pinned Sass build is justified in USW-01 before changing dependencies. A token translation into existing CSS is the initial implementation path because the current browser build has one app stylesheet and no USWDS pipeline. If fidelity requires the actual package, document its version, license, asset weight, and build integration in a revised bounded phase before adoption.

The terminal theme picker, report/PDF/PPTX templates, public documentation site, and a future full React/shadcn migration are outside this plan. Do not add a government banner, agency identifier, or claim of official endorsement to an ACORN research tool merely to make it look federal. Keep the current brand and task content; adapt the style of appropriate controls.

# Current evidence — November 23, 2026

- **Verified repository:** `crates/acorn-web/index.html:25-36` reads `acorn.theme.v1` and recognizes only `dark`, `light`, `system` during boot, also accepting `?theme=dark|light` for preview. `crates/acorn-web/assets/effects.js:95-120,190-221` represents the chosen style as a dark boolean, persists the same values, updates DOM theme state, labels, favicon, and settings controls. `crates/acorn-web/src/ui.rs:525-533,840-858,909` renders three Settings choices and a binary header toggle. The baseline CSS tokens are in `crates/acorn-web/assets/app.css:1-20`; many later selectors explicitly match `[data-theme=light]`. The React map island has its own `geospatial/src/map.css`. These are source observations, not an acceptance run.
- **Verified repository:** `crates/acorn-web/assets/effects.test.mjs`, `scripts/acceptance.test.mjs`, and `scripts/parity-smoke.mjs` exercise theme persistence, token contrast, and browser behavior. Existing parity checks and screenshots assume exactly two rendered palettes; they must be extended without weakening their present checks. `scripts/preload-wasm.mjs:15-21` expects one staged app stylesheet. The current worktree has uncommitted browser files; recheck ownership before implementation and do not treat those edits as this plan's progress.
- **Verified external guidance, opened November 23:** The [designer guide](https://designsystem.digital.gov/documentation/getting-started-for-designers/) says the maintained USWDS 3 design kits are Figma and Sketch, points to live component HTML for measurements, and asks teams to recheck accessibility after customization. [Design tokens](https://designsystem.digital.gov/design-tokens/) organize color, typography, spacing, and other choices; USWDS Sass settings and helpers resolve token names to values. The [accessibility guidance](https://designsystem.digital.gov/documentation/accessibility/) says teams must test their own implementations, including contrast, keyboard use, zoom, and assistive technology. Thus adopting its colors alone does not establish accessibility or conformance.
- **Unresolved:** Exact official token values, fonts and permitted self-hosted assets for the pinned reference release, contrast across ACORN-specific result/chart states, and whether the map/CodeMirror styles need component-specific overrides. USW-01 inventories and freezes these before any visual implementation. No USWDS theme is implemented or validated yet.

# Design contract

1. **Reference and mapping.** Pin a USWDS 3 release and record the official source URL, token keys, resolved values, and target ACORN roles in a compact mapping table: canvas/surface, primary/action/link, text/muted, border, focus, success/warning/error, selected/hover/disabled, and chart series. Include type family/size/line height, spacing, radius, and button/input/table patterns. Record deliberate ACORN exceptions. Prefer local fonts/assets only after license and bundle review; use documented fallbacks. Do not copy a generic color palette and call it USWDS.
2. **Theme state.** Use `uswds` as the stable stored, DOM, and preview URL value. Treat render mode as a named value rather than a dark boolean. `System` continues to resolve to light/dark according to the operating system; USWDS never changes with that preference. A saved USWDS choice survives reload and route changes; URL override is temporary. Invalid stored/URL values retain existing fallback. Reset removes the saved choice. The header light/dark toggle remains a quick action: from USWDS it selects Dark, and Settings remains the place to choose USWDS again. Give the header control an accurate accessible name and state for the current mode; do not present a misleading pressed binary when USWDS is active.
3. **Scope styles.** Activate via `[data-theme=uswds]` on the existing workspace and document root. Build on existing role variables where possible; explicitly audit hard-coded colors and light-only rules. Scope any USWDS styles so baseline themes and external widgets remain stable. Preserve focus order, native controls, reduced-motion behavior, editor diagnostics, alert meaning, chart data equivalents, and map attribution. Avoid a global reset, duplicate stylesheet, or new network dependency.
4. **Meaningful validation.** Test normal, hover, focus, selected, disabled, busy, error, and success states. Measure text/control contrast and visible focus against the chosen WCAG AA criteria, including ACORN-specific badges and charts; verify labels, keyboard actions, 390px mobile, 200% zoom, reduced motion, and browser engines. Manual assistive-technology review is a separate acceptance observation, not inferred from passing automated checks.

# Dependencies and decisions

- USW-01 freezes the reference and inventory; USW-02 implements named theme state; USW-03 styles shared surfaces; USW-04 covers specialized views; USW-05 performs integrated acceptance and docs. The sequence is serial where CSS or Settings files overlap.
- Share the browser UI/style lock with [browser workspace](acorn-browser-workspace/index.md), [shadcn-style component transition](../migrations/acorn-web-shadcn-components.md), [browser performance](../refactors/browser-performance.md), and [frontend build quality](../testing/frontend-build-quality.md). Coordinate exact write sets before touching active web files. SCU's visual parity baseline applies to existing light/dark; its screenshots should use an explicit theme and exclude the new opt-in state from its zero-difference claim. A future React migration must carry forward the `uswds` preference contract, not block this theme.
- No dependency on ABW-19 hosted private-provider acceptance, desktop creation, or the shadcn component migration. Final local acceptance uses synthetic data and a built Pages artifact. Recheck static asset size against existing browser performance budgets; a future hosted publication remains ABW's owner.

# Acceptance criteria

- A documented token/component mapping cites the pinned official USWDS sources, identifies every adapted ACORN surface and exception, and records whether packaged Sass was declined or adopted with evidence.
- Settings chooses USWDS, announces the selected mode, persists it, and restores it before first paint; URL override, reset, system preference, and header toggle follow the design contract. Existing dark/light behavior and screenshots stay stable.
- Shared and specialized views show consistent USWDS styling without unreadable hard-coded states or broken chart/diagnostic distinctions. No package or asset is fetched at runtime solely for the theme.
- Focused unit and browser checks pass, including all four preference modes, root and `/acorn/` builds, mobile and zoom states, and a documented manual keyboard/screen-reader review. Record any unresolved accessibility defect as a bounded repair phase before claiming acceptance.
- `make web-check`, `make web-build`, `make web-test-pages`, and relevant focused JS/map checks pass on a stable tree; inspect CSS/asset size and first-paint behavior. Run broader gates only after shared browser edits settle.

# Execution phases

## USW-01 — Freeze official reference and ACORN surface inventory

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: stable local snapshot for comparison; coordinate current browser write owners
- Exact write set: `crates/acorn-web/README.md`, `.wiki/features/uswds-browser-theme.md`
- Primary integration lock: browser theme specification
- Outcome: Pin a USWDS release and inspect its official token, type, button, form, table, alert, and accessibility pages plus live HTML/Figma kit. Record resolved value/role mapping, asset/license and Sass decision, current theme-specific selectors, and representative screenshot states for ACORN routes. Flag any unsupported or conflicting pattern explicitly.
- Focused validation: Review each mapping against opened official source and inspect baseline Dark/Light/System screenshots at desktop/mobile; verify every listed ACORN state has a current UI owner.
- Handoff: Record pinned links/version, baseline revision, mapping, exceptions, and CSS/build decision; remove USW-01 and make USW-02 ready.

## USW-02 — Add named theme selection and first-paint restoration

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: USW-01
- Exact write set: `crates/acorn-web/index.html`, `crates/acorn-web/assets/effects.js`, `crates/acorn-web/assets/effects.test.mjs`, `crates/acorn-web/src/ui.rs`, `.wiki/features/uswds-browser-theme.md`
- Primary integration lock: browser theme state and Settings
- Outcome: Accept `uswds` in stored and preview choices; render a fourth Settings choice and correct selected-state/header semantics; restore the named mode before paint; preserve existing system, reset, and URL behavior. The CSS can temporarily use light fallback until USW-03, with no public release between phases.
- Focused validation: Run `node --test crates/acorn-web/assets/effects.test.mjs`, focused Rust checks, and a browser boot/reload/URL/reset matrix for all modes; check the initial document theme before Dioxus mounts.
- Handoff: Record mode matrix and first-paint evidence; remove USW-02 and make USW-03 ready.

## USW-03 — Style shared shell, navigation, and forms

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: USW-02
- Exact write set: `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/acceptance.test.mjs`, `crates/acorn-web/scripts/parity-smoke.mjs`, `.wiki/features/uswds-browser-theme.md`
- Primary integration lock: browser shared CSS
- Outcome: Add scoped USWDS role tokens and styles for typography, layout rhythm, sidebar, links, buttons, controls, cards, tables, notifications, and focus/disabled/busy states. Include token contrast assertions and explicit screenshots for Home, Settings, Review, Portfolio, and one tool result.
- Focused validation: Run `node --test crates/acorn-web/scripts/acceptance.test.mjs`, relevant parity scope, computed-style/contrast checks, and visual comparison showing no unexplained Dark/Light changes.
- Handoff: Record token mapping and view evidence; remove USW-03 and make USW-04 ready.

## USW-04 — Finish specialized editor, results, and map surfaces

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: USW-03; no concurrent map/editor CSS owner
- Exact write set: `crates/acorn-web/assets/app.css`, `crates/acorn-web/editor/editor.js`, `crates/acorn-web/geospatial/src/map.css`, `crates/acorn-web/geospatial/src/map.tsx`, `crates/acorn-web/scripts/parity-smoke.mjs`, `.wiki/features/uswds-browser-theme.md`
- Primary integration lock: browser specialized visual surfaces
- Outcome: Resolve inherited and hard-coded colors for CodeMirror, diagnostics, result charts/statuses, details overlays, map layer controls, and map attribution under USWDS mode. Touch JS/TSX only where CSS cannot express the state; keep charts' noncolor meaning and the map's existing behavior.
- Focused validation: Run focused editor/map tests and parity screenshots for diagnostics, errors, charts, drawer, and map; inspect contrast, focus, and reduced-motion states; compare baseline themes for regressions.
- Handoff: Record exceptional styling and measured asset delta; remove USW-04 and make USW-05 ready.

## USW-05 — Accept accessibility, build, and user documentation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: USW-04; stable shared browser tree
- Exact write set: `crates/acorn-web/README.md`, `crates/acorn-web/scripts/parity-smoke.mjs`, `.wiki/features/uswds-browser-theme.md`
- Primary integration lock: browser theme acceptance
- Outcome: Run full named-mode/browser matrix and document how to select, reset, and preview USWDS, the pinned reference, intentional differences, and limits of the claim. Review keyboard and available screen-reader behavior on representative forms, editor, navigation, dialogs, and map controls; record actual results.
- Focused validation: `make web-check`, `make web-build`, `make web-test-pages`, focused JS/map tests, root and `/acorn/` smoke, CSS/asset-size comparison, and manual mobile/200%-zoom/keyboard/assistive-technology pass.
- Handoff: If all acceptance criteria hold, preserve durable details in README/tests and retire this plan and its index/network/lane entries. If a concrete defect remains, add a bounded repair phase with its exact write set.
