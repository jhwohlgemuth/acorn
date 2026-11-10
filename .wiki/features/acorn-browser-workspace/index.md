---
type: Plan
title: Rust browser and Tauri desktop workspace for ACORN tools
description: Ship one accessible local-first Dioxus workspace, patterned after the OKF Plan Browser shell, in browsers and a least-privilege Tauri desktop app for all eight current ACORN WebMCP operations plus a Resium/CesiumJS research-organization map.
difficulty: high
impact: high
package: [workspace, acorn-web, acorn-desktop, acorn-wasm, acorn-host, acorn-service, acorn-analysis, acorn-schema]
tags: [feature, browser, desktop, tauri, rust, wasm, webmcp, dioxus, resium, cesiumjs, geospatial, accessibility, ornl, os:windows, os:macos, os:linux]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Add one Rust-authored ACORN workspace that turns the current eight-operation WebAssembly demo into a coherent local product in two delivery modes: a browser application and a local Tauri 2 desktop application. Both modes use the same Dioxus Web route tree, components, state model, ACORN artwork, and OKF-inspired design. Tauri embeds the built static frontend and adds only explicit user-mediated local file, folder, and save capabilities; it does not fork the UI or domain logic.

The application uses the [OKF Plan Browser home](https://www.jasonwohlgemuth.com/okf-plan-browser/home) as its layout reference: a dark full-height shell, collapsible grouped sidebar, concise home dashboard, status and metric cards, routed detail views, and compact responsive behavior. ACORN content and workflows replace all planning-specific concepts.

The first release provides manual interfaces for Research Activity validation, metadata-standard validation, persistent-identifier discovery and inspection, content diff, FAIR assessment, metadata crosswalk, readability, and a geospatial view of organization locations referenced by loaded Research Activity Data. The same page explicitly installs the existing eight read-only WebMCP tools when `document.modelContext` is supported; every manual workflow remains available when WebMCP is absent.

# Design preview

## Home workspace

![Latest ACORN browser and Tauri desktop home preview](_assets/acorn-browser-workspace-preview.svg)

## About page

![Latest ACORN browser and Tauri desktop About preview](_assets/acorn-browser-about-preview.svg)

These SVGs are the canonical plan-level Home and About previews. Any browser or desktop UX, branding, navigation, theme, responsive-layout, or native-shell decision that changes either intended view must update the corresponding asset in the same planning change so the embedded images never trail the plan.

# Current evidence

- Repository baseline `10b17f22` has a clean worktree on November 7, 2026. `crates/acorn-wasm/src/lib.rs` exposes the eight browser operations and `crates/acorn-wasm/web/webmcp.js` registers their bounded read-only WebMCP adapters without import-time registration or network side effects.
- `crates/acorn-wasm/examples/webmcp/` already supplies accessible manual controls, adapter invocation feedback, an unsupported-browser fallback, and a native smoke route. It proves the operations, but it is a test/demo page rather than a routed product workspace.
- `docs/src/_assets/acorn.svg`, `banner_dark_mode.png`, and `banner_light_mode.png` are the current repository-owned ACORN mark and theme banners. The shared workspace reuses those assets rather than drawing a substitute logo. The live OKF Plan Browser audit on November 8, 2026 confirms transferable chrome: collapsible sidebar, icon-rich navigation, version display, guided tour, theme choices, notification region, Settings/About routes, local file/drop and sample-data entry points, and the made-with-heart credit. Its About route uses a narrow left-aligned content column, prominent source/contribution card, short labeled narrative sections and cards, a three-column technology grid, and a bordered AI-disclosure card; ACORN adopts that hierarchy for its applicable project, local-first, delivery-target, source, technology, and disclosure content while keeping the tour entry point on Home rather than copying every OKF control.
- `docs/src/_assets/schema-logos/rust.svg` and `webassembly.svg` already provide repository-held Rust and WebAssembly marks. The technology credit may also use the unchanged icon-only Tauri artwork from the official `create-tauri-app` repository and the unchanged `dioxus_color.svg` from the official Dioxus brand repository, subject to retaining their provenance and license notices and without implying endorsement: [Tauri template asset](https://github.com/tauri-apps/create-tauri-app/blob/dev/templates/_assets_/tauri.svg), [Tauri trademark guidance](https://v2.tauri.app/about/trademark/), and [Dioxus brand resources](https://github.com/DioxusLabs/brand).
- The About-page AI-use disclosure identifies the tools with the official Codex login mark from the [`openai/codex` login asset](https://github.com/openai/codex/blob/main/codex-rs/login/src/assets/error.html), the official [GitHub Copilot Octicon](https://github.com/primer/octicons/blob/main/icons/copilot-24.svg), and the repository-held `docs/src/_assets/artifact-logos/microsoft.svg` vendor mark beside the Microsoft 365 Copilot label. Imported marks remain unchanged, linked labels identify the exact products, provenance and license notices are retained, and the disclosure does not imply endorsement. Home and non-About chrome identify only the human author; About has no made-with-heart line.
- The official [AI Assisted badge](https://www.aihonestybadge.com/ai-assisted-badge) describes work led and shaped by a person with AI helping, including cases where AI produced most of the code. The publisher allows personal or commercial use, hotlinking without attribution, and provides a single light/dark-compatible image with no script or tracking. ACORN bundles the unchanged [small SVG](https://www.aihonestybadge.com/badges/ai-assisted-small.svg) for offline/static delivery, links it to the badge explanation, gives it an accessible name, and keeps the more specific agent/tool disclosure beside it. The preview copy retrieved November 8, 2026 has SHA-256 `E5342204B418C42AE9F5E981E558CCEE71754252E52FCEF9A9B7ED8B4A52D5A6`.
- Driver.js 1.10.2 is the selected guided-tour implementation. Its official configuration exposes ordered steps, progress, keyboard control, close labels, reduced-animation control, and destroy hooks; the application bundles a pinned local copy of its JavaScript and CSS so the tour does not introduce a runtime CDN dependency: [Driver.js configuration](https://driverjs.com/docs/configuration), [installation](https://driverjs.com/docs/installation), and [changelog](https://driverjs.com/docs/changelog).
- `crates/acorn-service` owns portable supplied-content validation, readability, and repository-processing contracts; `acorn-analysis`, `acorn-schema`, `acorn-core`, and `acorn-diff` provide the remaining in-memory behavior. `ARCHITECTURE.md` keeps browser composition in `acorn-wasm` and excludes native host adapters from the portable graph.
- The active [crate-boundary architecture](../../refactors/crate-boundary-architecture.md) is the ownership authority for this plan: portable computation stays in `acorn-core`, `acorn-schema`, `acorn-analysis`, `acorn-diff`, and `acorn-service`; reusable JavaScript bindings stay in `acorn-wasm`; native filesystem effects stay in `acorn-host`; and interface crates select those capabilities without importing `acorn-lib`. The [portability coordination record](../../refactors/portability/index.md) requires a concrete browser workflow before adding a narrow browser adapter contract; this workspace is that concrete consumer and must extend the matrix rather than bypass it.
- The current WebMCP adapter bounds metadata documents to 256 KiB, each diff side to 128 KiB, prose and discovery text to 16 KiB, identifier/resource values to 2 KiB, output to 256 KiB, diagnostics to 100, and discovered identifiers to 100. Both delivery modes consume those limits rather than inventing broader ones.
- The active [WebMCP and MCP interoperability plan](../../testing/webmcp-evaluations.md) owns independent evaluation data, external evaluator tooling, agent-selection evidence, layered MCP checks, and compatible-runner CI. It does not own the ACORN product workspace.
- Dioxus 0.7 documents Web as its best-supported target, compiles to `wasm32-unknown-unknown`, renders DOM nodes, and exposes browser APIs through `wasm-bindgen`: [Dioxus Web](https://dioxuslabs.com/learn/0.7/guides/platforms/web/). This matches ACORN's current target and accessibility requirements.
- Freya 0.5 added an experimental canvas-based browser backend and states that not every desktop integration works in the browser: [Freya 0.5](https://freyaui.dev/posts/0.5). The shared semantic Dioxus Web frontend plus Tauri shell is the selected cross-delivery path; Freya is not a first-release renderer.
- Resium supplies React lifecycle wrappers for CesiumJS and requires `react-dom`; CesiumJS itself requires DOM, WebGL, Web Workers, and static runtime assets. Resium recommends Vite plus `vite-plugin-cesium`, which copies those assets and sets `CESIUM_BASE_URL`: [Resium guide](https://resium.reearth.io/guide/) and [installation](https://resium.reearth.io/installation/). This makes Resium suitable only as a route-local React island with an explicit bridge to Dioxus, not as a replacement application shell.
- CesiumJS can render against its local ellipsoid without Cesium ion, while ion terrain and imagery require a public-client access token and runtime network requests. The first release therefore bundles Cesium workers/widgets and uses a token-free ellipsoid scene with local organization markers; remote terrain, imagery, geocoding, and ion credentials are optional future work, not hidden defaults: [CesiumJS quickstart](https://cesium.com/learn/cesiumjs-learn/cesiumjs-quickstart/), [Ion reference](https://cesium.com/learn/cesiumjs/ref-doc/Ion.html), and [offline guide](https://github.com/CesiumGS/cesium/blob/main/Documentation/OfflineGuide/README.md).
- ROR Schema 2.1 records latitude and longitude under `locations[].geonames_details`, and the versioned ROR data dump is CC0. A pinned, generated location-only subset can resolve ROR identifiers without a runtime registry call, provided the build records the dump version, digest, license, generation command, and output size: [ROR data structure](https://ror.readme.io/docs/ror-data-structure) and [data dump](https://ror.readme.io/docs/data-dump).
- Rune is a pre-release, revision-pinned native GPU-rendered Rust UI framework with its own renderer, text, layout, state, animation, platform shells, and headless agent-driven rendering. Those capabilities overlap rather than complement Dioxus/Tauri and do not solve Resium's React requirement. Rune is therefore not added to this plan; its headless PNG/element-tree testing model remains a useful idea for a later bounded spike if the selected stack fails measured performance or testability gates: [Rune README](https://github.com/anomalyco/rune) and [Rune plan](https://github.com/anomalyco/rune/blob/dev/PLAN.md).
- Tauri 2 is frontend-agnostic and can embed a static HTML/CSS/JavaScript/WASM directory through `frontendDist`, so the built Dioxus Web output can be reused instead of creating a second renderer: [Tauri frontend configuration](https://v2.tauri.app/start/frontend/) and [Tauri configuration](https://v2.tauri.app/reference/config/). Its capability files constrain which windows may invoke commands, and production builds generate platform-specific bundles: [Tauri capabilities](https://v2.tauri.app/security/capabilities/) and [Tauri distribution](https://v2.tauri.app/distribute/).
- ORNL publicly identifies green as its signature color, but the public page does not publish a machine-readable ACORN token: [OLCF media assets](https://www.olcf.ornl.gov/about-olcf/media-assets/). The initial theme therefore uses `#00662C` only as a provisional ORNL-inspired accent unless an approved ORNL source confirms a production value. No ORNL logo or implied official endorsement is in scope.

# Boundaries and decisions

## In scope

- A new `crates/acorn-web` Dioxus Web composition root targeting `wasm32-unknown-unknown`.
- A new `crates/acorn-desktop` Tauri 2 shell that embeds the release output from `acorn-web`, opens the same routes in a system webview, and contains no second ACORN UI or computation implementation.
- A light/dark/system-aware OKF-inspired application shell using the repository ACORN mark and matching theme banner, grouped routes, collapsible desktop sidebar, accessible narrow-screen drawer, home callout, runtime status, and one icon-labeled entry per tool. The persistent chrome presents theme state as a real native toggle button with a visible switch track, light/dark label, and accessible pressed state rather than two decorative icon buttons; the Settings route may restore system preference.
- Applicable OKF Plan Browser affordances adapted to ACORN: icon-rich navigation, sidebar version and runtime summary, a Home-only Driver.js guided tour with **Try sample data** immediately to its right, Settings and About routes, session-only theme control, polite notification region, one combined **Open file or folder** action beside the retained drop-zone loader, bundled sample data, visible WebMCP status, tooltips for collapsed icon controls, and the linked Home/non-About credit “Made with ❤️ by [**jhwohlgemuth**](https://github.com/jhwohlgemuth)”. About has no tour control or made-with-heart line; its AI-use disclosure uses the unchanged linked AI Assisted honesty badge and incorporates linked Codex, GitHub Copilot, and Microsoft 365 Copilot names and marks alongside the human-review explanation. The About technology cards use the Rust, WebAssembly, Tauri, and Dioxus marks with visible names, without a second “official project marks” strip. Domain-specific portfolio, graph, repository-token, install, and planning-skill workflows do not carry over.
- Manual routed workflows for the eight tool names frozen by the current adapter:
  - `acorn_assess_fair_metadata`
  - `acorn_check_readability`
  - `acorn_crosswalk_metadata`
  - `acorn_diff_content`
  - `acorn_discover_persistent_identifiers`
  - `acorn_inspect_persistent_identifier`
  - `acorn_validate_metadata_standard`
  - `acorn_validate_research_activity`
- Typed Rust state and result presentation over the same portable operations used by the JavaScript bindings. UI mappings may be new; domain algorithms, validation rules, limits, and result semantics are reused.
- One generated Dioxus/WASM application module that links the `acorn-wasm` rlib and retains its `wasm-bindgen` exports. ABW-01 must prove this packaging seam before product pages build on it; a silent second copy of the ACORN WASM graph is not an accepted fallback.
- Explicit `createAcornWebMcpAdapter(...)` installation, status, invocation feedback, and disposal. Importing the application or adapter remains side-effect free until the page installs it.
- Platform effects behind one typed UI boundary: native browser file/folder selection and download in browser builds and user-mediated Tauri file/folder open plus file-save dialogs in desktop builds. A loaded folder becomes a deterministic session-only tree of supported text files; selecting a leaf supplies that file to a manual tool and never runs tools automatically. The desktop backend traverses only the root returned by its native folder dialog inside a bounded Rust command, so the webview receives sanitized relative paths and content but never a reusable filesystem capability, absolute path, persisted directory grant, or ambient home-directory access.
- Structured results before optional raw JSON: ordered diagnostics, FAIR profiles/findings, readability checks, conversion warnings, unified diff or JSON Patch, and normalized identifier states.
- A `/geospatial` route that extracts ROR identifiers from loaded Research Activity Data, resolves them against a lazy-loaded, versioned ROR 2.1 location subset, and plots deduplicated organizations in a token-free CesiumJS globe. Dioxus owns route state, filters, selection details, the accessible organization table, empty/error/WebGL-unavailable states, and all ACORN data handling; a route-local React/Resium island owns only the Cesium viewer and marker lifecycle.
- A semantic DOM, keyboard navigation, visible focus, route-title/focus management, restrained live regions, reduced motion, 200% zoom, and desktop/phone portrait/phone landscape acceptance.
- An ORNL-inspired green accent system with accessible solid, subtle, hover, selected, and focus tokens. The production documentation must distinguish an approved brand value from a provisional inspired value.

## Out of scope

- Reimplementing ACORN rules, codecs, crosswalks, identifier logic, FAIR checks, readability, or diff behavior in UI code.
- Replacing `webmcp.js`, its contract tests, the existing native smoke, or the independent WebMCP evaluation plan.
- Unprompted filesystem access, traversal outside the explicitly selected folder, symlink following, directory watching, PDF or PowerPoint rendering, databases, shell/process execution, inference, `doctor`, authenticated repository operations, a server requirement, or remote persistence.
- A TypeScript/React application shell, server-side rendering, a separate desktop UI, a Freya or Rune production renderer, or a generic dynamic-form framework. The bounded Resium map island is the only React-owned surface.
- Cesium ion credentials, remote terrain or imagery, geocoding, automatic ROR API calls, network-fetched map data, user tracking, or making the WebGL globe the only way to inspect organization locations.
- Auto-update, background startup, tray integration, global shortcuts, deep links, remote content, mobile Tauri targets, store publication, production code signing, and macOS notarization. The first desktop release produces local unsigned developer bundles; distribution credentials and governed release channels require a separate authorized plan.
- Automatic clipboard writes, automatic downloads, tool execution on page load, telemetry, or ACORN-originated network requests.
- Copying OKF Plan Browser source, planning content, portfolio concepts, or product name. The reference governs information architecture and visual rhythm only.
- Using an ORNL logo, DOE mark, trademarked asset, or claim of official branding without an approved repository asset and authorization.
- Adding Big-O UI to this eight-tool workspace scope before its navigation and preview are reconciled with the now nine-tool adapter.

## Product and architecture decisions

1. **Use Dioxus Web and the DOM.** The product requires native inputs, semantic navigation, browser file controls, text-heavy results, and screen-reader evidence. Canvas-first rendering is not the first-release accessibility risk profile.
2. **Keep one UI composition root.** `acorn-web` owns routes, presentation state, and a typed platform-effect interface; portable ACORN crates own computation; `acorn-wasm` owns JavaScript DTO conversion and WebMCP bindings. `acorn-desktop` embeds the compiled frontend and implements only the native side of approved effects. Browser, webview, and Tauri types do not cross into portable crates.
3. **Prove one WASM payload before page work.** Link the existing `acorn-wasm` rlib into the Dioxus module and verify the generated JavaScript exports all eight bindings. If current tooling cannot preserve those exports, ABW-01 remains incomplete and a bounded repair phase must decide the packaging change; do not ship two unmeasured copies.
4. **Manual controls are the baseline.** WebMCP status is `initializing`, `installed`, `unsupported`, `failed`, or `disposed`; unsupported WebMCP never disables the manual tools.
5. **Use structured Rust presentation state.** Distinguish idle, running, valid/success, invalid input, bounded rejection, operation failure, and unsupported capability. Invalid metadata is not an application crash.
6. **Stay local by default.** Session state may survive route changes in memory. The first release uses no cookies, local storage, IndexedDB, service worker cache, remote API, or analytics.
7. **Use a provisional green honestly.** Start with `#00662C` as `--ornl-green` only when the UI and documentation call it ORNL-inspired. Replace or confirm it from an approved brand source before claiming official ORNL branding. Derive accessible light/dark variants rather than using the base green for every foreground.
8. **Do not treat blue or purple as the primary accent.** Informational states may use a distinct accessible color, but actions, active navigation, focus treatment, and brand highlights use the green token family.
9. **Carry over applicable OKF chrome deliberately.** Reuse the interaction ideas—collapse/drawer, icon labels and tooltips, tour, theme choices, version, Settings/About, notifications, local open/drop/sample entry points, runtime status, and made-with-heart credit—while replacing OKF-specific portfolio and remote-repository behavior with ACORN's bounded local workflows.
10. **Keep the desktop authority narrow and visible.** The main Tauri window loads bundled assets, uses an explicit Content Security Policy, and receives only custom open-text, open-folder, and save-text commands. Those commands show a native dialog and complete bounded reads, selected-root traversal, or writes in Rust through `acorn-host`. Do not grant the webview the filesystem, shell, process, updater, persisted-scope, or remote-URL capabilities. HTML file drops remain content-only; do not enable path-wide scope as a shortcut.
11. **Do not emulate WebMCP with privileged IPC.** The desktop app reports the actual `document.modelContext` result from its system webview. Manual tools remain the baseline when it is unsupported. Tauri commands never masquerade as WebMCP tools, and WebMCP invocations never inherit native file authority.
12. **Make the chrome theme control a toggle.** The header uses one native button with `aria-pressed` to express explicit dark/light state and a visible switch track plus label. It initializes from the system preference when no explicit session choice exists; activating it selects the opposite explicit theme. Settings may return to system preference, after which the toggle reflects the resolved theme without misrepresenting persistence.
13. **Use Driver.js for the tour.** Bundle pinned Driver.js 1.10.2 JavaScript, CSS, license, and provenance with the static application; do not load it from a CDN. A thin application adapter owns stable target IDs, ordered steps, progress, keyboard controls, close/done handling, focus return, teardown, theme tokens, and reduced-motion behavior. Driver.js never owns routes, product state, analytics, or persistence.
14. **Treat a folder as bounded supplied content.** Browser and desktop folder pickers may recursively enumerate only the selected root. Keep at most 500 supported regular files, 20 relative-path segments, 256 KiB per loaded file, and 16 MiB total decoded UTF-8 content; accept `.json`, `.yaml`, `.yml`, `.md`, and `.txt`; sort normalized relative paths lexicographically; never follow symlinks; and report skipped, rejected, and truncated entries. Operation-specific bounds still apply when a leaf is sent to a tool.
15. **Isolate Resium as a map island.** Resium requires React, so a small Vite-built module mounts only inside the `/geospatial` viewer host. Dioxus passes an immutable, bounded marker DTO and receives selection/camera/error events through one versioned JavaScript bridge. The island does not parse RAD, resolve ROR, own application navigation, duplicate theme state, call ACORN operations, register WebMCP tools, or import Tauri APIs.
16. **Keep geospatial resolution local and inspectable.** Generate a minimal ROR-id/display-name/latitude/longitude/country index from a pinned Schema 2.1 data dump, record its digest and CC0 provenance, and lazy-load it only on `/geospatial`. Deduplicate by normalized ROR ID, preserve every source reference count, report unresolved and locationless identifiers, and provide the same ordered organization table when WebGL is unavailable. Cesium starts without ion, geocoder, remote imagery, or terrain.
17. **Do not add Rune to the first-release stack.** Rune's renderer, layout, state, and platform shells compete with Dioxus/Tauri rather than supplying missing components. Reconsider it only in a separate time-boxed spike with explicit parity, accessibility, Web/desktop, Resium integration, binary-size, and headless-test gates after a measured failure of the selected architecture; borrowing its headless-testing concepts does not require a dependency.

## Crate-boundary contract

| Package | Role in this plan | Forbidden ownership |
|---|---|---|
| `acorn-core`, `acorn-schema`, `acorn-analysis`, `acorn-diff`, `acorn-service` | Existing portable values, codecs, operations, findings, and orchestration consumed directly by first-party interfaces | DOM, Tauri, filesystem, WebMCP registration, presentation state, or duplicated UI-specific result models |
| `acorn-wasm` | Reusable `wasm-bindgen` ABI, JavaScript DTO conversion, and the existing eight-tool WebMCP adapter | Dioxus routes/components, CSS, Tauri commands, native host effects, or a dependency on `acorn-web` |
| `acorn-web` | Static Dioxus interface composition root: routes, components, presentation state, accessibility, ephemeral DOM effects, and the narrow bridge to a route-local Resium map island | Portable domain policy, duplicate JS DTO/adapter logic, a React application shell, ACORN logic inside the map island, native filesystem/process dependencies, `acorn-host`, or `acorn-lib` |
| `acorn-host` | Existing narrow native filesystem owner; ABW-12 may add the missing explicit text-write primitive beside `read_text`, and ABW-17 may add bounded selected-root traversal, each with focused tests | Tauri window/dialog types, Dioxus state, product routing, or WebMCP policy |
| `acorn-desktop` | Thin Tauri interface composition root that embeds `acorn-web`, owns native file/folder dialog selection and IPC authorization, and delegates selected-file reads/writes plus selected-root traversal to `acorn-host` | Reimplemented ACORN operations, direct `std::fs` policy, a second UI tree, reusable browser adapters, reusable path authority, or privileged WebMCP bridging |
| `acorn-lib` | No role in the new browser or desktop dependency graphs | Compatibility re-exports or host shortcuts that conceal the narrow owning crates |

`acorn-web` is a new interface root, not a replacement browser-adapter owner. Its DOM effects are presentation mechanics over user-supplied in-memory values; reusable JavaScript ABI and WebMCP behavior remain in `acorn-wasm`. If a required operation is absent from the portable owners, the implementing phase stops and opens a bounded extraction/repair phase under the crate-boundary architecture instead of adding the operation to either UI crate.

# Routes and first-release behavior

| Route | Manual workflow | Required presentation |
|---|---|---|
| `/home` | Choose a tool, geospatial view, local file, recursively loaded local folder, dropped content, or bundled sample and inspect runtime state | ACORN artwork, adjacent **Take a tour** and **Try sample data** actions in that order, privacy callout, combined file-or-folder loader, retained drop zone, session file tree, eight icon-labeled WebMCP tools plus geospatial navigation, theme toggle button, notification region, WebMCP status, version, and author-only made-with-heart credit |
| `/validate/activity` | Validate Research Activity JSON and optional local file | Validity, effective fuzzy policy, ordered/truncated diagnostics |
| `/validate/standard` | Validate supplied content/format/standard | Validity, supported-combination errors, ordered diagnostics |
| `/crosswalk` | Run a supported typed crosswalk | Converted content, output format, mandatory loss warnings, copy/download |
| `/fair` | Assess supported JSON metadata | 41 findings, profile and per-principle counts, no-resolution notice |
| `/readability` | Run one supported metric | Score, threshold, acceptable/excessive state, returned summary |
| `/diff` | Compare before/after content | Unified or RFC 6902 result, changed state, copy/download |
| `/identifiers/discover` | Discover identifiers in text | Ordered normalized identifiers, count, truncation |
| `/identifiers/inspect` | Inspect one identifier and optional kind | Valid, invalid, or ambiguous state and deterministic candidates |
| `/geospatial` | Map organizations referenced by ROR identifiers in loaded Research Activity Data | Token-free Resium/CesiumJS globe, deduplicated local markers, unresolved/locationless summary, selected-organization details, synchronized accessible table, WebGL fallback, and ROR dataset version/provenance |
| `/settings` | Configure session-only appearance and reset transient state | Dark, light, and system choices; current version/runtime; no persistence claim |
| `/about` | Explain boundaries, authorship, AI use, and implementation technology | Narrow OKF-inspired content column with source card and no tour control, project narrative, local-first and delivery-target cards, three-column technology grid with labeled marks and no duplicate mark strip, version/runtime and WebMCP boundaries, bordered AI-use disclosure with the linked AI Assisted badge plus linked agent names and marks, and no made-with-heart footer |

# Dependencies and coordination

- The [crate-boundary architecture](../../refactors/crate-boundary-architecture.md) is an upstream coordination dependency, and this plan is its concrete atomic browser/desktop consumer. That record remains authoritative for ownership direction; this plan owns the bounded implementation phases. The [portability coordination record](../../refactors/portability/index.md) supplies the existing WASM/feature gates and gains the accepted `acorn-web` composition row when ABW-01 lands.
- ABW-01 is ready against the current eight-operation contract. It claims the workspace manifest/lockfile and browser build seam, so it must not overlap another phase editing `Cargo.toml` or `Cargo.lock`.
- ABW-02 follows ABW-01 and freezes the branded shell; ABW-10 then completes the applicable OKF home/chrome affordances. ABW-03 follows both. ABW-04 and ABW-05 own disjoint page and operation modules and may proceed in parallel only after ABW-03 freezes the shared result/input components and their listed files remain disjoint.
- ABW-06 follows the tool pages because single-file/copy/download behavior must be accepted against real result types, not speculative abstractions. ABW-16 then adds the bounded recursive browser folder workspace over that frozen effect contract.
- ABW-18 follows ABW-03, ABW-05, and ABW-16 so it can consume the accepted RAD input, normalized ROR identifiers, and session-tree contracts. It exclusively owns the React/Resium/CesiumJS island, Vite staging, and generated ROR location index; no other phase may introduce React or change final static-asset assembly concurrently.
- ABW-07 follows ABW-01 and all tool groups. It serializes WebMCP staging and `Makefile` changes with WME-02, WME-04, and any other owner of `webmcp.js` or WASM package assembly. It consumes the adapter unchanged unless a separate evidence-backed repair expands its write set.
- ABW-08 requires the complete manual and geospatial route matrix, including the map island's static assets, WebGL fallback, and accessible table. Live WebMCP registration requires a supported Chrome build and flag/configuration; missing support blocks only that matrix row, not manual browser acceptance.
- ABW-11 follows the frozen browser build seam and shell, then claims the workspace manifest/lockfile alone to add the Tauri shell. ABW-12 follows the accepted browser effects, serializes the `acorn-host` filesystem leaf, and adds bounded native single-file open/save commands without a general filesystem permission or direct filesystem implementation in `acorn-desktop`. ABW-17 then adds one selected-folder command against the ABW-16 tree contract and the same `acorn-host` ownership boundary.
- ABW-13, ABW-14, and ABW-15 validate local bundles on Windows, macOS, and Linux respectively after ABW-17 and ABW-08. Their platform-specific files are disjoint and may proceed in parallel on authorized native hosts; each unavailable host remains unverified rather than passing by inference.
- ABW-09 follows all browser and desktop behavior and acceptance phases. It owns final documentation, broad gates, and plan retirement.
- The [WebMCP and MCP interoperability plan](../../testing/webmcp-evaluations.md) remains authoritative for external evaluator, model-selection, MCP Inspector/conformance, public MCP control, and required-CI decisions. This plan owns the product page and manual parity.
- BGO has added a ninth Big-O tool and passed its interactive Chrome smoke. Before ABW implementation, reconcile this plan's eight-tool UI scope and previews with the nine-tool adapter in a bounded planning update.

# Validation strategy

## Focused Rust and package checks

```text
cargo test -p acorn-web
cargo check -p acorn-web --target wasm32-unknown-unknown
cargo check -p acorn-wasm --target wasm32-unknown-unknown
cargo test -p acorn-host --features fs
cargo test -p acorn-desktop
cargo check -p acorn-desktop
(from crates/acorn-desktop) cargo tauri build --no-bundle
(from crates/acorn-web/geospatial) npm ci && npm test && npm run build
make test-webmcp
make test-webmcp-package
make build-wasm
```

The Dioxus production build must record generated JavaScript/WASM sizes and prove that the eight `wasm-bindgen` exports and routed application share one payload. Generated output stays under `target/`.

The Tauri build must consume that exact release asset directory through `frontendDist`; it must not fetch a remote page or stage another UI. `cargo tauri build` runs on each native desktop host because platform bundles are host-specific. Generated binaries, installers, and Tauri schemas stay under ignored build output.

## Browser matrix

- Desktop at 1440×900.
- Phone portrait at 390×844.
- Phone landscape at 844×390; do not apply desktop sidebar behavior merely because the width exceeds a small-phone breakpoint.
- Chrome, Firefox, and Safari/WebKit for manual tools when available. WebMCP registration is a separate supported-Chrome row, not a universal-browser expectation.
- Keyboard-only route, drawer, form, result, disclosure, copy, and download flows.
- Visible focus, skip link, route heading focus, Escape/return focus for the drawer and guided tour, 200% zoom, reduced motion, status announcements, icon tooltips, and no color-only validity or active state.
- Dark, light, and system theme states; collapsed desktop sidebar; narrow-screen drawer; version/runtime display; tour steps; local open/drop/sample entry points; notification lifecycle; Settings/About routes; and made-with-heart footer.
- Geospatial route with representative duplicate, invalid, unresolved, and locationless ROR values; marker/table selection parity; no-ion/offline mode; WebGL-unavailable fallback; theme and resize changes; lazy loading; and no external map, registry, terrain, imagery, geocoder, or telemetry request.
- Valid, invalid, empty, oversized, operation-error, and unsupported-capability states for each applicable tool.
- Browser console free of application errors; no ACORN operation request observed on the network panel.

Static, unit, or headless checks do not count as live desktop/portrait/landscape or screen-reader evidence. Record unavailable browsers or assistive technologies as unverified rather than passing them by inference.

## Desktop matrix

- Windows 11 native WebView2, macOS native WebKit, and a supported Linux WebKitGTK host; record OS, architecture, webview version, bundle type, and exact unsigned artifact path for every observed row.
- Launch from `cargo tauri dev` and from the locally built bundle with networking unavailable; both must show the same routes, ACORN assets, theme states, keyboard behavior, and bounded operation results as the browser build.
- Native open and save dialogs cover success, cancel, invalid extension, invalid UTF-8, oversized input, safe suggested filename, write failure, and preservation of current editor/result state.
- Capability inspection proves the bundled main window only loads local assets and cannot invoke filesystem, shell, process, updater, or remote-content APIs. The three custom file/folder commands reject forged paths because callers never submit paths; the folder result exposes only normalized relative paths beneath the dialog-selected root.
- WebMCP status reflects the embedded webview's real support. Unsupported is an accepted desktop state; no native IPC shim or elevated tool registration is present.
- Platform-specific packaging checks run on their native hosts. Missing signing or notarization is documented as an intentional local-developer limitation, not silently reported as a distributable release.

# Acceptance criteria

- `crates/acorn-web` is the documented Rust/Dioxus composition root and compiles for `wasm32-unknown-unknown` without native ACORN host dependencies; `crates/acorn-desktop` embeds that exact static build in Tauri 2 without duplicating routes, components, or domain logic.
- `cargo tree` and target checks prove `acorn-web` has no `acorn-host` or `acorn-lib` edge, `acorn-wasm` has no UI or native-host edge, and `acorn-desktop` delegates selected-path filesystem work to the narrowly enabled `acorn-host/fs` capability rather than owning direct filesystem policy.
- The application uses an OKF-inspired light/dark/system shell, grouped collapsible icon-rich sidebar, centered home workflow, responsive drawer, tool entries, Settings/About routes, locally bundled Driver.js tour, notification region, and routed detail pages without copying OKF planning content or source. The persistent theme control is one native toggle button with visible state, switch geometry, and correct `aria-pressed`; Settings can restore system preference.
- The repository `acorn.svg`, dark banner, and light banner are reused with appropriate accessible alternatives; `v0.6.3`-style version placement derives from the built package rather than hard-coded UI state. Home and non-About footers display exactly “Made with ❤️ by [**jhwohlgemuth**](https://github.com/jhwohlgemuth)” and contain no agent names or marks. About contains no made-with-heart line. Its AI-use disclosure displays the unchanged, locally bundled [AI Assisted badge](https://www.aihonestybadge.com/ai-assisted-badge) with an accessible name and link, then names and links [**Codex**](https://openai.com/codex/), [**GitHub Copilot**](https://github.com/features/copilot), and [**Microsoft 365 Copilot**](https://www.microsoft.com/microsoft-365/copilot) beside their marks and the human direction/review/testing explanation. Each linked label and adjacent mark belongs to one accessible external link, wraps without clipping at narrow widths, and retains a readable text-only meaning when images are unavailable.
- The About page displays unchanged Rust, WebAssembly, Tauri, and Dioxus marks inside their existing technology cards with adjacent text labels, useful alternative text, provenance, and retained license notices. It does not repeat them in a separate “official project marks” strip. The marks do not appear in the persistent footer, remain secondary technology credits, are not recolored or combined into a new logo, and do not imply project endorsement.
- The About page follows the live OKF reference hierarchy closely while using ACORN content: narrow left-aligned column, source/contribution card, project and local-first sections, delivery-target cards, three-column technology grid, and bordered AI-use disclosure. It contains no tour action; the guided-tour entry point remains on Home. The disclosure leads with the AI Assisted honesty badge, incorporates linked names and marks for the three agent products, explains the maintainer's direction/review/testing role, and advises review of AI-generated code; no separate attribution footer is present.
- The accent system uses an approved ORNL signature-green value or explicitly labels `#00662C` and its derived variants as ORNL-inspired/provisional. Purple and blue are not the primary interactive accent; no unapproved logo is present.
- All eight current manual operations are reachable from the home page and sidebar, use existing ACORN computation, enforce current bounds, and render structured results before optional raw JSON.
- One Dioxus/WASM payload contains the application and all eight JavaScript bindings, with recorded release size and no silent duplicated ACORN module.
- `/geospatial` plots deduplicated ROR-backed organization locations from loaded RAD using the isolated Resium/CesiumJS island while Dioxus retains all ACORN state and an equivalent accessible table. The production build includes Cesium workers/widgets and a lazy, pinned, digest-verified ROR 2.1 location subset with license/provenance, makes no external map or ROR request by default, requires no ion token, and reports unresolved/locationless records plus WebGL failure without losing the table.
- The page explicitly installs exactly eight WebMCP tools when supported, reports installation state, routes invocation feedback safely, disposes registrations, and keeps manual controls functional when unsupported.
- Local file or folder selection never uploads content, failed reads preserve the current editor, and the session tree contains only normalized relative paths and supported UTF-8 content under the 500-file/20-segment/256-KiB-per-file/16-MiB-total limits. Unsupported and rejected entries are summarized; no operation runs until the user selects a leaf and invokes it. Copy/download/save actions require explicit user action, object URLs are revoked, and generated filenames are safe. Desktop file/folder open and save occur inside bounded Rust commands after native dialog selection, delegate filesystem work to `acorn-host`, never follow symlinks, and grant no ambient filesystem or absolute-path access to the webview.
- Route changes set a useful document title and focus; navigation, tools, disclosures, results, copy/download, and the responsive drawer pass keyboard acceptance with visible focus and semantic DOM controls.
- Desktop, phone portrait, and phone landscape pass without horizontal page overflow. The globe yields space or collapses behind the accessible organization list at narrow sizes without hiding data. Manual-tool acceptance records the browsers actually observed; WebMCP acceptance records the exact supported Chrome version/configuration.
- Unsigned local Tauri bundles launch and pass the shared manual-tool contract on observed Windows, macOS, and Linux hosts. Each platform row records native evidence; an unavailable host remains a named blocker. The desktop app works offline, loads only bundled assets, exposes only the three approved file/folder dialog commands, and does not elevate WebMCP.
- Focused Rust, WASM, adapter, browser, Tauri, capability, and native-package checks pass. Final `make lint`, `make test`, `make check-wasm`, documentation build, `git diff --check`, and wiki validation pass without unrelated changes.
- Durable user/developer documentation records browser installation and serving, Tauri prerequisites and local build commands, static-host base paths, privacy/network/native-capability boundaries, supported tools, WebMCP limitations, browser and native-platform evidence, build sizes, unsigned-bundle limitations, and brand-token status.
- After all criteria pass, ABW-09 deletes this plan and removes its feature-index, portfolio, dependency-network, and parallel-lane entries.

# Execution phases

## ABW-01 — Prove the Rust browser composition and single-WASM seam

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: current eight-operation `acorn-wasm` contract and installed `wasm32-unknown-unknown` target
- Exact write set: `Cargo.toml`, `Cargo.lock`, `ARCHITECTURE.md`, `crates/acorn-web/Cargo.toml` (new), `crates/acorn-web/Dioxus.toml` (new), `crates/acorn-web/index.html` (new), `crates/acorn-web/src/main.rs` (new), `crates/acorn-web/src/lib.rs` (new), `crates/acorn-web/README.md` (new), `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/refactors/crate-boundary-architecture.md`, `.wiki/refactors/portability/index.md`
- Primary integration lock: workspace manifest and browser build graph
- Outcome: Add the minimal Dioxus Web interface composition root, consume portable owners directly plus the `acorn-wasm` rlib, build one browser payload, and update the crate-boundary/portability records to show `acorn-web` above rather than inside the reusable adapter layer. Prove all eight generated JavaScript bindings remain exported without `acorn-lib`, native host dependencies, reversed package edges, or a second staged ACORN WASM module.
- Focused validation: `cargo test -p acorn-web`; `cargo check -p acorn-web --target wasm32-unknown-unknown`; rerun `cargo check -p acorn-wasm --target wasm32-unknown-unknown`; inspect `cargo tree -p acorn-web --target wasm32-unknown-unknown` for direct narrow portable owners and absence of `acorn-host`/`acorn-lib`; run the documented Dioxus development and release builds; inspect generated exports, network-free blank shell, and JS/WASM sizes; validate the wiki and dependency diagrams together.
- Handoff: Remove ABW-01, preserve the accepted build/export commands and measured baseline in `crates/acorn-web/README.md`, and mark ABW-02 ready. If the single-module seam fails, keep the phase open and add a bounded repair phase instead of silently staging duplicate payloads.

## ABW-02 — Build the accessible OKF-inspired shell and green theme

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-01
- Exact write set: `crates/acorn-web/assets/app.css` (new), `crates/acorn-web/assets/acorn.svg` (new from `docs/src/_assets/acorn.svg`), `crates/acorn-web/assets/banner_dark_mode.png` (new from docs), `crates/acorn-web/assets/banner_light_mode.png` (new from docs), `crates/acorn-web/assets/technology/rust.svg` (new from docs), `crates/acorn-web/assets/technology/webassembly.svg` (new from docs), `crates/acorn-web/assets/technology/tauri.svg` (new from official `create-tauri-app`), `crates/acorn-web/assets/technology/dioxus.svg` (new from official Dioxus brand resources), `crates/acorn-web/assets/technology/README.md` (new provenance/license record), `crates/acorn-web/src/app.rs` (new), `crates/acorn-web/src/route.rs` (new), `crates/acorn-web/src/components/mod.rs` (new), `crates/acorn-web/src/components/app_shell.rs` (new), `crates/acorn-web/src/components/sidebar.rs` (new), `crates/acorn-web/src/components/tool_card.rs` (new), `crates/acorn-web/src/pages/mod.rs` (new), `crates/acorn-web/src/pages/home.rs` (new), `crates/acorn-web/src/pages/about.rs` (new), `crates/acorn-web/src/tests.rs` (new), `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-workspace-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-about-preview.svg`
- Primary integration lock: `acorn-web` shell and theme contract
- Outcome: Deliver the routed shell with actual repository ACORN artwork, grouped icon-rich collapsible sidebar, responsive drawer, version/runtime block, centered home workflow, privacy callout, eight WebMCP tool entries plus a geospatial route placeholder, an About page containing a labeled Rust/WebAssembly/Tauri/Dioxus technology grid using unchanged sourced marks inside the cards without a duplicate marks strip, semantic landmarks, skip link, route-title/focus behavior, provisional green token family, and no blue/purple primary accent or unapproved mark.
- Focused validation: Component/state tests for routes, active navigation, icon labels/tooltips, collapse/drawer semantics, version/runtime truth, artwork selection, technology-logo labels/alternatives/provenance, absence of a duplicate technology-mark strip, document titles, eight-tool coverage, and the geospatial navigation placeholder; inspect upstream mark hashes and retained notices; Dioxus browser build; keyboard smoke at desktop and both phone orientations; automated contrast checks for normal, hover, selected, disabled, and focus tokens; compare the implementation with the canonical SVG and update that SVG in the same change if the accepted design changes.
- Handoff: Remove ABW-02, preserve the branded shell and navigation contract in code and README, and mark ABW-10 ready.

## ABW-10 — Complete applicable OKF home and chrome affordances

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-02
- Exact write set: `crates/acorn-web/assets/app.css`, `crates/acorn-web/assets/sample_research_activity.json` (new), `crates/acorn-web/assets/vendor/driver.js` (new, pinned 1.10.2 distribution), `crates/acorn-web/assets/vendor/driver.css` (new, pinned 1.10.2 distribution), `crates/acorn-web/assets/vendor/driver.LICENSE` (new), `crates/acorn-web/assets/vendor/README.md` (new provenance record), `crates/acorn-web/assets/credits/ai-assisted-badge.svg` (new, unchanged from AI Honesty Badge), `crates/acorn-web/assets/credits/codex.svg` (new from `openai/codex`), `crates/acorn-web/assets/credits/github-copilot.svg` (new from Primer Octicons), `crates/acorn-web/assets/credits/microsoft.svg` (new from docs), `crates/acorn-web/assets/credits/README.md` (new provenance/license record), `crates/acorn-web/src/theme.rs` (new), `crates/acorn-web/src/components/tour.rs` (new), `crates/acorn-web/src/components/theme_toggle.rs` (new), `crates/acorn-web/src/components/notification.rs` (new), `crates/acorn-web/src/components/sidebar.rs`, `crates/acorn-web/src/pages/home.rs`, `crates/acorn-web/src/pages/settings.rs` (new), `crates/acorn-web/src/pages/about.rs`, `crates/acorn-web/src/route.rs`, `crates/acorn-web/src/tests/chrome.rs` (new), `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/acorn-browser-workspace/_assets/ai-assisted-badge.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-workspace-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-about-preview.svg`
- Primary integration lock: browser home and application-chrome behavior
- Outcome: Add a native session-only dark/light toggle button that initializes from and can return to system preference through Settings; integrate locally bundled Driver.js 1.10.2 for the guided tour with stable targets, progress, dismiss/done teardown, return focus, themed popovers, and reduced-motion configuration; place **Try sample data** immediately to the right of **Take a tour** at desktop and narrow widths; and add polite notifications, local open/drop entry points, collapsed-icon tooltips, WebMCP/version display, the author-only Home/non-About credit, and one About AI-use disclosure containing the unchanged AI Assisted honesty badge plus linked Codex/GitHub Copilot/Microsoft 365 Copilot names and marks, without persistent storage or runtime network access.
- Focused validation: Component/state tests for toggle semantics (`button`, visible label, switch state, `aria-pressed`), both explicit themes, system initialization/reset/change, and banner/token synchronization; Driver.js asset version/license/provenance and offline loading; tour target order, progress, keyboard controls, close labels, dismissal/done reason, teardown, missing target, focus return, and reduced motion; notification replacement/expiry, adjacent Home tour/sample action order and responsive wrapping, absence of a tour control on About, file/drop/sample entry routing, Settings/About links, version derivation, collapsed tooltips, WebMCP status, exact author-only non-About footer text, absence of agent names/marks outside About, absence of any made-with-heart line on About, exact AI Assisted badge asset/hash/link/accessible name, exact AI-disclosure agent labels and URLs, inline-mark alternatives/provenance, safe external-link behavior, responsive wrapping, and proof that the badge loads from the local static bundle without a runtime request; keyboard smoke in desktop and both phone orientations; compare every applicable chrome element with the live OKF reference and both canonical SVGs.
- Handoff: Remove ABW-10, preserve the accepted chrome behavior and asset provenance in code/README, and mark ABW-03 ready.

## ABW-03 — Add validation and crosswalk workspaces

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-02, ABW-10
- Exact write set: `crates/acorn-web/src/operations/mod.rs` (new), `crates/acorn-web/src/operations/metadata.rs` (new), `crates/acorn-web/src/components/content_input.rs` (new), `crates/acorn-web/src/components/diagnostic_list.rs` (new), `crates/acorn-web/src/components/result_panel.rs` (new), `crates/acorn-web/src/pages/validate_activity.rs` (new), `crates/acorn-web/src/pages/validate_standard.rs` (new), `crates/acorn-web/src/pages/crosswalk.rs` (new), `crates/acorn-web/src/pages/mod.rs`, `crates/acorn-web/src/route.rs`, `crates/acorn-web/src/tests/metadata.rs` (new), `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: browser metadata operation and shared result contract
- Outcome: Add complete manual Research Activity validation, metadata-standard validation, and loss-aware crosswalk pages with typed Rust state, current input bounds, ordered diagnostics, supported-combination feedback, explicit conversion warnings, and optional raw results.
- Focused validation: Tests for valid/invalid/empty/oversized input, fuzzy policy, ordered and truncated diagnostics, missing paths, unsupported format/standard pairs, required crosswalk seeds, loss warnings, and operation errors; focused Rust/WASM checks; live keyboard and result-announcement smoke for the three routes.
- Handoff: Remove ABW-03 and preserve stable shared input/result component behavior for the remaining tool groups.

## ABW-04 — Add FAIR, readability, and diff workspaces

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-02; ABW-03 shared result contract frozen
- Exact write set: `crates/acorn-web/src/operations/analysis.rs` (new), `crates/acorn-web/src/pages/fair.rs` (new), `crates/acorn-web/src/pages/readability.rs` (new), `crates/acorn-web/src/pages/diff.rs` (new), `crates/acorn-web/src/pages/mod.rs`, `crates/acorn-web/src/route.rs`, `crates/acorn-web/src/tests/analysis.rs` (new), `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: browser analysis pages
- Outcome: Add FAIR, readability, and diff pages that retain all 41 FAIR findings and profile counts, all seven readability metrics and returned thresholds/summaries, and both unified and RFC 6902 diff modes with changed state and safe structured rendering.
- Focused validation: Tests for every metric, non-finite/error handling, 41 ordered findings, supported FAIR standards, no-resolution notice, equal/changed diff states, invalid JSON Patch inputs, escaped unified output, input limits, and optional raw results; focused Rust/WASM checks and live keyboard smoke.
- Handoff: Remove ABW-04 and record any rendering performance threshold needed by ABW-08 without changing portable result semantics.

## ABW-05 — Add identifier discovery and inspection workspaces

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-02; ABW-03 shared result contract frozen
- Exact write set: `crates/acorn-web/src/operations/identifiers.rs` (new), `crates/acorn-web/src/pages/discover_identifiers.rs` (new), `crates/acorn-web/src/pages/inspect_identifier.rs` (new), `crates/acorn-web/src/pages/mod.rs`, `crates/acorn-web/src/route.rs`, `crates/acorn-web/src/tests/identifiers.rs` (new), `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: browser identifier pages
- Outcome: Add discovery and one-identifier inspection pages with supported-kind selection, normalized ordered results, deduplication/count/truncation state, and visibly distinct valid, invalid, and deterministic ambiguous outcomes without resolving identifiers.
- Focused validation: Representative ARK, arXiv, DOI, Handle, ISBN, ISNI, ORCID, patent, RAiD, ROR, RRID, and SWHID cases; invalid and explicit-kind mismatch; ambiguous ISNI/ORCID candidate ordering; discovery deduplication and 100-result limit; focused Rust/WASM checks and live keyboard smoke.
- Handoff: Remove ABW-05 and confirm all eight home/sidebar entries resolve to complete manual pages before ABW-06/07.

## ABW-06 — Add single-file, copy, and download effects

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-03, ABW-04, ABW-05
- Exact write set: `crates/acorn-web/src/effects/mod.rs` (new), `crates/acorn-web/src/effects/browser_file.rs` (new), `crates/acorn-web/src/effects/clipboard.rs` (new), `crates/acorn-web/src/effects/download.rs` (new), `crates/acorn-web/src/components/content_input.rs`, `crates/acorn-web/src/components/result_panel.rs`, `crates/acorn-web/src/pages/validate_activity.rs`, `crates/acorn-web/src/pages/validate_standard.rs`, `crates/acorn-web/src/pages/crosswalk.rs`, `crates/acorn-web/src/pages/diff.rs`, `crates/acorn-web/src/tests/effects.rs` (new), `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: browser-local presentation-effect boundary
- Outcome: Add explicit local file selection where meaningful, user-initiated copy feedback, and safe converted/diff downloads as UI-local DOM mechanics without upload, persistence, automatic clipboard writes, leaked object URLs, browser types in portable crates, or a competing reusable adapter surface beside `acorn-wasm`.
- Focused validation: Unit tests for accepted extensions, UTF-8/size rejection, failure preservation, filename normalization, MIME selection, and effect state; live file chooser, failed read, copy success/failure, download bytes/name, object-URL cleanup, and network-panel observation in supported browsers.
- Handoff: Remove ABW-06, retain the accepted browser-effect boundary, and mark ABW-16 ready.

## ABW-16 — Add a bounded browser file-and-folder workspace

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-06
- Exact write set: `crates/acorn-web/src/effects/browser_file.rs`, `crates/acorn-web/src/effects/browser_folder.rs` (new), `crates/acorn-web/src/components/content_input.rs`, `crates/acorn-web/src/components/file_tree.rs` (new), `crates/acorn-web/src/pages/home.rs`, `crates/acorn-web/src/tests/effects.rs`, `crates/acorn-web/src/tests/file_tree.rs` (new), `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-workspace-preview.svg`
- Primary integration lock: browser supplied-content and session-tree contract
- Outcome: Present one left-side **Open file or folder** loader beside the existing right-side drop zone. The single visible loader owns both file and directory selection; when a browser cannot expose a combined native picker, activating it opens one accessible File/Folder choice without adding a second home-page button. Retain drop loading for supported files and folder entries where the browser exposes them. Capability-detect recursive directory selection, normalize and lexicographically sort relative paths, and render an accessible expandable session tree for nested folders. A leaf click loads that file into the active content input without automatically invoking a tool. Accept only `.json`, `.yaml`, `.yml`, `.md`, and `.txt` UTF-8 regular files under the 500-file, 20-segment, 256-KiB-per-file, and 16-MiB-total limits; summarize skipped/rejected/truncated entries; preserve the prior session after cancellation or total failure; and keep all content local with no directory handle persistence.
- Focused validation: Unit tests for the single loader's file/folder branches, drop-zone preservation, nested relative paths, deterministic ordering, duplicate normalized paths, traversal names, depth/file/individual/total limits, invalid UTF-8, unsupported extensions, empty folders, partial acceptance summaries, cancellation, failure preservation, and leaf-to-input selection; live Chromium and Safari/WebKit directory input where available plus Firefox capability fallback; keyboard tree and loader-choice navigation, 200% zoom, phone portrait/landscape reflow, file/folder drop behavior, and network-panel observation.
- Handoff: Remove ABW-16, preserve the accepted supplied-tree DTO and capability fallback in tests/README, make ABW-08 browser acceptance and ABW-17 native folder integration eligible, and record unavailable browser directory-picker rows without weakening single-file workflows.

## ABW-18 — Add the local ROR geospatial workspace

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-03, ABW-05, and ABW-16; no concurrent owner of `acorn-web` route assembly, static-asset staging, or JavaScript package manifests
- Exact write set: `crates/acorn-web/geospatial/package.json` (new), `crates/acorn-web/geospatial/package-lock.json` (new), `crates/acorn-web/geospatial/vite.config.ts` (new), `crates/acorn-web/geospatial/src/main.tsx` (new), `crates/acorn-web/geospatial/src/bridge.ts` (new), `crates/acorn-web/geospatial/src/map.tsx` (new), `crates/acorn-web/geospatial/tests/map.test.tsx` (new), `crates/acorn-web/scripts/build-ror-location-index.mjs` (new), `crates/acorn-web/assets/geospatial/ror-locations.json` (new, generated), `crates/acorn-web/assets/geospatial/ROR_PROVENANCE.md` (new), `crates/acorn-web/src/geospatial.rs` (new), `crates/acorn-web/src/pages/geospatial.rs` (new), `crates/acorn-web/src/pages/mod.rs`, `crates/acorn-web/src/route.rs`, `crates/acorn-web/src/components/sidebar.rs`, `crates/acorn-web/src/tests/geospatial.rs` (new), `crates/acorn-web/Dioxus.toml`, `Makefile`, `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: Resium/CesiumJS island, ROR location dataset, and final static-asset assembly
- Outcome: Add `/geospatial` as a Dioxus-owned RAD workflow with a route-local React 19/Resium/CesiumJS island built by pinned Vite tooling. Extract and normalize ROR identifiers from the selected RAD, deduplicate organization markers, join them to a lazy-loaded minimal ROR Schema 2.1 location index, and expose source-reference count, name, ROR link, country, latitude, and longitude. Start Cesium without ion, terrain, imagery, geocoder, or network services; bundle its workers/widgets into the same relative-base-safe static output used by GitHub Pages, GitLab Pages, and Tauri. Keep filters, selected details, loading/error state, dataset provenance, unresolved/locationless summaries, and a synchronized sortable table in semantic Dioxus DOM. The bridge accepts only a bounded immutable marker DTO and emits selection/camera/error events; it owns no ACORN parsing, routing, theme persistence, WebMCP, or native capability. If the phase reaches its budget before both the data/table contract and map bridge are accepted, preserve the data/table work and split the island into a new bounded phase rather than weakening tests.
- Focused validation: Pin and audit npm versions/licenses; regenerate the ROR index from the recorded dump version and SHA-256 and compare deterministic output; test valid/duplicate/invalid/unresolved/locationless ROR values, coordinate bounds, stable ordering, source counts, empty RAD, oversized marker sets, lazy-load failure, and escaped labels; React bridge contract tests for mount/update/select/error/unmount and theme changes; Cesium asset/base-path inspection for root and nested Pages paths; prove no ion token or external request; WebGL-on and forced-WebGL-off browser smokes; marker/table selection parity; keyboard access to every organization through the table; 200% zoom and phone reflow; Dioxus/WASM build plus Tauri offline smoke; measure the route's compressed/uncompressed JS, worker, widget, and ROR-index sizes and record a reviewed budget before acceptance.
- Handoff: Remove ABW-18, preserve the bridge schema, dataset refresh/provenance procedure, measured asset budget, no-network Cesium configuration, and accessible-table fallback in `crates/acorn-web/README.md`; make ABW-08 and native bundle acceptance eligible. Any request for ion, remote tiles, geocoding, or a broader React surface requires a separate privacy/security plan.

## ABW-07 — Install the existing eight-tool WebMCP adapter in the workspace

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-01, ABW-03, ABW-04, ABW-05, ABW-10; no concurrent WME-02/WME-04 or other WASM package-staging owner
- Exact write set: `crates/acorn-web/index.html`, `crates/acorn-web/assets/webmcp-bootstrap.js` (new), `crates/acorn-web/src/webmcp.rs` (new), `crates/acorn-web/src/app.rs`, `crates/acorn-web/src/components/sidebar.rs`, `crates/acorn-web/src/pages/home.rs`, `crates/acorn-web/tests/webmcp-workspace.test.mjs` (new), `Makefile`, `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: WebMCP application installation and WASM package staging
- Outcome: Initialize the generated bindings, create and explicitly install the unchanged adapter, expose `initializing/installed/unsupported/failed/disposed` status, route invocation feedback without overwriting unrelated pages, dispose on page teardown, and keep all manual routes usable without `document.modelContext`.
- Focused validation: `make test-webmcp`; new workspace integration test with fake supported/unsupported contexts; exactly eight registered names and schemas; representative manual/adapter parity for every operation; busy, cancellation, bounded rejection, failure-envelope, duplicate-install, and disposal cases; `make test-webmcp-package`; Dioxus build export inspection; live supported-Chrome registration/discovery/invocation/cleanup when available.
- Handoff: Remove ABW-07 and record the exact supported Chrome/configuration evidence. Missing native support remains an explicit ABW-08 matrix blocker, not a reason to weaken adapter tests.

## ABW-08 — Accept responsive, accessible, and bounded browser behavior

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-02 through ABW-07, ABW-10, ABW-16, and ABW-18
- Exact write set: `crates/acorn-web/assets/app.css`, `crates/acorn-web/src/theme.rs`, `crates/acorn-web/src/components/app_shell.rs`, `crates/acorn-web/src/components/sidebar.rs`, `crates/acorn-web/src/components/tour.rs`, `crates/acorn-web/src/components/theme_toggle.rs`, `crates/acorn-web/src/components/notification.rs`, `crates/acorn-web/src/components/content_input.rs`, `crates/acorn-web/src/components/file_tree.rs`, `crates/acorn-web/src/effects/browser_folder.rs`, `crates/acorn-web/src/components/result_panel.rs`, `crates/acorn-web/src/pages/home.rs`, `crates/acorn-web/src/pages/geospatial.rs`, `crates/acorn-web/src/pages/settings.rs`, `crates/acorn-web/geospatial/src/map.tsx`, `crates/acorn-web/tests/browser-smoke.mjs` (new), `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-workspace-preview.svg`
- Primary integration lock: browser workspace acceptance matrix
- Outcome: Close responsive, keyboard, focus, zoom, reduced-motion, semantic, contrast, performance, WebGL-fallback, and bounded-error defects across the complete route matrix at desktop, phone portrait, and phone landscape without changing operation or geospatial bridge contracts.
- Focused validation: Run the complete manual-tool and geospatial matrix in available Chrome, Firefox, and Safari/WebKit; supported-Chrome WebMCP matrix separately; keyboard-only navigation, organization-table use, and drawer focus return; 200% zoom; reduced motion; automated accessibility/contrast scan; no horizontal overflow at all three viewports; forced WebGL failure; console/network inspection proving no external map/ROR request; maximum-bound timing and interaction responsiveness; rerun focused Rust/WASM, Resium bridge, and WebMCP tests after repairs.
- Handoff: Remove ABW-08 and record dated browser/OS/viewport/assistive-technology evidence plus every unavailable row. Any acceptance defect outside the exact write set becomes a bounded repair phase.

## ABW-11 — Embed the shared Dioxus workspace in a minimal Tauri shell

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-01, ABW-02, and ABW-10; no concurrent workspace-manifest or lockfile owner
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-desktop/Cargo.toml` (new), `crates/acorn-desktop/build.rs` (new), `crates/acorn-desktop/tauri.conf.json` (new), `crates/acorn-desktop/capabilities/main.json` (new), `crates/acorn-desktop/icons/` (new, derived from repository ACORN artwork), `crates/acorn-desktop/src/main.rs` (new), `crates/acorn-desktop/README.md` (new), `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-workspace-preview.svg`
- Primary integration lock: workspace manifest and Tauri application shell
- Outcome: Add a flat interface workspace member for Tauri 2, point `frontendDist` and the development URL at the accepted `acorn-web` outputs, open one labeled main window with the repository-derived ACORN icon, inject a restrictive Content Security Policy, and prove browser and desktop load the same route/component build. The shell exposes no custom command or plugin permission yet, imports neither `acorn-lib` nor portable operation crates, and never loads a remote production URL.
- Focused validation: `cargo test -p acorn-desktop`; `cargo check -p acorn-desktop`; build `acorn-web`, launch `cargo tauri dev` from `crates/acorn-desktop`, and run `cargo tauri build --no-bundle`; inspect the embedded asset manifest, window label, CSP, enabled capability list, binary provenance, offline launch, route parity, version/runtime display, and JS/WASM hash equality with the accepted web build; compare the implementation with the canonical SVG.
- Handoff: Remove ABW-11, preserve the exact dev/build prerequisites and shared-asset proof in `crates/acorn-desktop/README.md`, and mark ABW-12 ready. Any need for a second frontend or remote URL fails the phase and requires an architecture repair.

## ABW-12 — Add least-privilege native open and save effects

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-06 and ABW-11; no concurrent workspace-manifest, lockfile, or `acorn-web` effect-boundary owner
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-host/src/fs.rs`, `crates/acorn-host/src/tests/fs.rs`, `crates/acorn-desktop/Cargo.toml`, `crates/acorn-desktop/build.rs`, `crates/acorn-desktop/capabilities/main.json`, `crates/acorn-desktop/src/main.rs`, `crates/acorn-desktop/src/commands.rs` (new), `crates/acorn-desktop/src/tests.rs` (new), `crates/acorn-web/src/effects/mod.rs`, `crates/acorn-web/src/effects/desktop_file.rs` (new), `crates/acorn-web/src/components/content_input.rs`, `crates/acorn-web/src/components/result_panel.rs`, `crates/acorn-web/src/tests/effects.rs`, `crates/acorn-desktop/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: Tauri IPC capability and shared platform-effect contract
- Outcome: Add exactly two typed, bounded commands: open a user-selected UTF-8 text file and save user-provided text to a user-selected destination. Each Tauri command owns its native dialog and bounds, delegates the selected-path read/write to `acorn-host::fs`, returns cancellation as a non-error state, and reveals no reusable path authority. Add only the missing focused host text-write primitive beside the existing `read_text`; do not duplicate direct filesystem policy in `acorn-desktop`. Explicit capability entries expose only those commands to the bundled `main` window; do not add Tauri filesystem, shell, process, updater, persisted-scope, or remote-content plugins.
- Focused validation: `cargo test -p acorn-host --features fs`; unit tests for the host write primitive, DTO serialization, accepted extensions, oversize and invalid UTF-8 rejection, cancellation, safe suggested names, write failure, and forged path absence; `cargo tree -p acorn-desktop` proves only the narrow `acorn-host/fs` capability and no `acorn-lib`; capability-schema inspection; negative invoke tests for unlisted commands, wrong window/origin, arbitrary caller paths, directories, traversal, and remote content; live dialog open/cancel/save flows; offline operation; browser effect regression; verify HTML drops do not widen native scope and WebMCP requests cannot invoke native commands.
- Handoff: Remove ABW-12, preserve the single-file command/capability threat model plus the `acorn-host` ownership decision in `crates/acorn-desktop/README.md`, and mark ABW-17 ready.

## ABW-17 — Add least-privilege native folder loading

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-12 and ABW-16; no concurrent `acorn-host` filesystem, Tauri command/capability, or shared platform-effect owner
- Exact write set: `crates/acorn-host/src/fs.rs`, `crates/acorn-host/src/tests/fs.rs`, `crates/acorn-desktop/capabilities/main.json`, `crates/acorn-desktop/src/main.rs`, `crates/acorn-desktop/src/commands.rs`, `crates/acorn-desktop/src/tests.rs`, `crates/acorn-web/src/effects/desktop_file.rs`, `crates/acorn-web/src/components/content_input.rs`, `crates/acorn-web/src/components/file_tree.rs`, `crates/acorn-web/src/tests/effects.rs`, `crates/acorn-desktop/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: selected-root traversal and Tauri IPC capability contract
- Outcome: Add exactly one `open_folder` command beside the accepted `open_text` and `save_text` commands. The command owns a native directory dialog, delegates deterministic selected-root traversal to a focused `acorn-host::fs` primitive, applies the ABW-16 extension/count/depth/per-file/total bounds, never follows symlinks, and returns only sanitized relative paths, UTF-8 content, and rejection summaries. Cancellation is non-error; absolute paths, reusable handles, arbitrary caller paths, and persisted scopes never cross IPC. The shared Dioxus session tree remains the only folder UI.
- Focused validation: `cargo test -p acorn-host --features fs`; host tests for nested ordering, root confinement, symlink exclusion, depth/count/size/UTF-8/extension limits, read races, and partial summaries; desktop DTO and cancellation tests; capability-schema inspection proving only three approved commands; negative invokes for unlisted commands, wrong window/origin, arbitrary paths, traversal, symlinks, directories outside the selected root, remote content, and WebMCP callers; live native file/folder open and save dialogs; browser effect regression; offline operation; `cargo tree -p acorn-desktop` proving narrow `acorn-host/fs` use and no `acorn-lib`.
- Handoff: Remove ABW-17, preserve the selected-root threat model and limits in `crates/acorn-desktop/README.md`, and mark ABW-13, ABW-14, and ABW-15 ready.

## ABW-13 — Accept the local Windows desktop bundle

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-08 and ABW-17; authorized Windows 11 x86_64 or ARM64 host with current Tauri prerequisites
- Exact write set: `crates/acorn-desktop/tauri.windows.conf.json` (new), `crates/acorn-desktop/acceptance/windows.md` (new), `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: Windows Tauri bundle acceptance
- Outcome: Produce and launch an unsigned local Windows bundle, prove shared UI/manual-tool parity and bounded native dialogs under WebView2, and record OS, architecture, WebView2, artifact, size, capability, accessibility, and offline evidence without claiming production signing or Store readiness.
- Focused validation: Native `cargo tauri build`; clean install/launch/uninstall smoke for the selected local bundle type; complete eight-tool and geospatial representative matrix; keyboard, 200% zoom, theme toggle/system reset, Driver.js tour, notification, icon, footer, offline Resium/Cesium assets and ROR markers/table, forced WebGL fallback, file/folder open/cancel/partial/error and save flows, nested tree selection, symlink exclusion, offline, console, network, capability-negative, and actual WebMCP-support checks; record hashes and artifact paths without committing generated bundles.
- Handoff: Remove ABW-13 and retain the dated Windows evidence in `crates/acorn-desktop/acceptance/windows.md`. Defects outside the write set open a bounded repair phase; a missing authorized host keeps this phase blocked.

## ABW-14 — Accept the local macOS desktop bundle

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-08 and ABW-17; authorized macOS Apple Silicon or x86_64 host with current Tauri prerequisites
- Exact write set: `crates/acorn-desktop/tauri.macos.conf.json` (new), `crates/acorn-desktop/acceptance/macos.md` (new), `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: macOS Tauri bundle acceptance
- Outcome: Produce and launch an unsigned local macOS application bundle, prove shared UI/manual-tool parity and bounded native dialogs under the system webview, and record OS, architecture, WebKit, artifact, size, capability, accessibility, and offline evidence without claiming signing, notarization, or App Store readiness.
- Focused validation: Native `cargo tauri build --bundles app`; launch/quarantine limitation smoke for the local app bundle; complete eight-tool and geospatial representative matrix; keyboard, 200% zoom, theme toggle/system reset, Driver.js tour, notification, icon, footer, offline Resium/Cesium assets and ROR markers/table, forced WebGL fallback, file/folder open/cancel/partial/error and save flows including security-scoped selection, nested tree selection, symlink exclusion, offline, console, network, capability-negative, and actual WebMCP-support checks; record hashes and artifact paths without committing generated bundles.
- Handoff: Remove ABW-14 and retain the dated macOS evidence in `crates/acorn-desktop/acceptance/macos.md`. Defects outside the write set open a bounded repair phase; a missing authorized host keeps this phase blocked.

## ABW-15 — Accept the local Linux desktop bundle

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-08 and ABW-17; authorized supported Linux x86_64 or ARM64 host with current WebKitGTK/Tauri prerequisites
- Exact write set: `crates/acorn-desktop/tauri.linux.conf.json` (new), `crates/acorn-desktop/acceptance/linux.md` (new), `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: Linux Tauri bundle acceptance
- Outcome: Produce and launch one unsigned local Linux bundle, prove shared UI/manual-tool parity and bounded native dialogs under WebKitGTK, and record distribution, architecture, WebKitGTK, artifact, size, capability, accessibility, and offline evidence without claiming repository, package-manager, or store publication.
- Focused validation: Native `cargo tauri build` for the selected local bundle type; clean launch/removal smoke; complete eight-tool and geospatial representative matrix; keyboard, 200% zoom, theme toggle/system reset, Driver.js tour, notification, icon, footer, offline Resium/Cesium assets and ROR markers/table, forced WebGL fallback, file/folder open/cancel/partial/error and save flows, nested tree selection, symlink exclusion, offline, console, network, capability-negative, and actual WebMCP-support checks; record hashes, dynamic-library prerequisites, and artifact paths without committing generated bundles.
- Handoff: Remove ABW-15 and retain the dated Linux evidence in `crates/acorn-desktop/acceptance/linux.md`. Defects outside the write set open a bounded repair phase; a missing authorized host keeps this phase blocked.

## ABW-09 — Publish the browser and desktop workspace workflows and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-01 through ABW-08 and ABW-10 through ABW-18
- Exact write set: `Makefile`, `crates/acorn-web/README.md`, `crates/acorn-desktop/README.md`, `docs/src/packages/acorn-web.md`, `docs/src/packages/acorn-desktop.md` (new), `docs/src/SUMMARY.md`, `.wiki/features/acorn-browser-workspace/index.md` (delete after acceptance), `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-workspace-preview.svg` (delete after acceptance), `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-about-preview.svg` (delete after acceptance), `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: browser/desktop build documentation and wiki retirement
- Outcome: Publish copy/paste browser and Tauri development/build workflows, local serving and local-bundle prerequisites, static-host base paths, privacy and capability boundaries, supported tools, WebMCP behavior, geospatial data/provenance/no-network behavior, ROR refresh procedure, browser/native evidence, measured WASM/map/dataset and bundle sizes, unsigned-distribution limitations, and brand-token guidance; add stable focused Make targets; run final integration gates; remove the completed plan from every wiki surface.
- Focused validation: Clean focused and release Dioxus builds; `cargo test -p acorn-web`; `cargo check -p acorn-web --target wasm32-unknown-unknown`; `npm ci && npm test && npm run build` from `crates/acorn-web/geospatial`; deterministic ROR-index regeneration/digest check; `cargo test -p acorn-host --features fs`; `cargo test -p acorn-desktop`; `cargo check -p acorn-desktop`; dependency-tree checks for every forbidden edge in the crate-boundary contract; local `cargo tauri build --no-bundle`; inspect retained Windows/macOS/Linux native evidence; `make test-webmcp`; `make test-webmcp-package`; `make build-wasm`; `make lint`; `make test`; `make check-wasm`; documentation build and link check; `git diff --check`; planning-wiki validation; verify generated files remain under ignored build output and no ion token, unapproved brand asset, signing material, or secret is committed.
- Handoff: Delete the plan and its index/network/lane entries after preserving all durable browser/desktop commands, native evidence, measured asset and bundle data, privacy/capability boundaries, unsigned-distribution limitations, and brand status in product documentation and tests.
