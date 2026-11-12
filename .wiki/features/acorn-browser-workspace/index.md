---
type: Plan
title: Rust browser and Tauri desktop workspace for ACORN tools
description: Ship an OKF Browser-aligned Dioxus workspace and Tauri desktop app with coordinated ACORN skill releases, WinGet, Scoop, Debian, AppImage, and Homebrew cask publication, loaded Research Activity views, and local ACORN tools.
difficulty: high
impact: high
package: [workspace, acorn-web, acorn-desktop, acorn-wasm, acorn-host, acorn-service, acorn-analysis, acorn-schema, acorn-cli]
tags: [feature, browser, desktop, tauri, rust, wasm, webmcp, dioxus, resium, cesiumjs, geospatial, accessibility, ornl, os:windows, os:macos, os:linux, os:debian, packaging, winget, scoop, appimage, homebrew, skills]
status: in-progress
execution: phased
phase_budget_hours: 5
---

# Goal

Add one Rust-authored ACORN workspace that turns the existing WebAssembly demo into a coherent local product in two delivery modes: a browser application and a local Tauri 2 desktop application. Both modes use the same Dioxus Web route tree, components, state model, ACORN artwork, and the OKF Browser design. Tauri embeds the built static frontend and adds only explicit user-mediated local file, folder, and save capabilities; it does not fork the UI or domain logic.

The application uses the [OKF Plan Browser home](https://www.jasonwohlgemuth.com/okf-plan-browser/home) as its design authority: the same shell proportions, typography hierarchy, spacing, navigation treatment, centered Home loading flow, restrained cards, and responsive behavior, adapted to ACORN content and green branding. The [November 11 design review](design-review.md) records the revised design specifications and accepted normal and exceptional-state specifications. ACORN content and workflows replace all planning-specific concepts.

The first release provides a visual portfolio for loaded Research Activity Data, manual interfaces for Research Activity validation, metadata-standard validation, persistent-identifier discovery and inspection, content diff, FAIR assessment, metadata crosswalk, readability, Big-O comparison, and a geospatial view of referenced organization locations. The same application explicitly installs the existing read-only WebMCP tools when `document.modelContext` is supported; every manual workflow remains available when WebMCP is absent.

# Design preview

**Updated November 11:** Every one of the 16 sidebar tabs has an individual editable SVG at 1440×900, plus a 32 px sample-state annotation strip. All use the OKF Browser shell and page hierarchy. Settings also has a separate light-theme SVG; responsive, drawer and collapsed-navigation boards supplement the desktop views. These are static specifications; Six exceptional-state previews and the dated live comparison are accepted in the design review.

| Tab | Route | Individual SVG |
|---|---|---|
| Home | `/home` | [Home](_assets/acorn-browser-workspace-preview.svg) |
| Portfolio | `/portfolio` | [Portfolio](_assets/acorn-browser-portfolio-preview.svg) |
| Research activity | `/validate/activity` | [Research activity](_assets/acorn-browser-research-activity-preview.svg) |
| Metadata standard | `/validate/standard` | [Metadata standard](_assets/acorn-browser-metadata-standard-preview.svg) |
| Crosswalk | `/crosswalk` | [Crosswalk](_assets/acorn-browser-crosswalk-preview.svg) |
| FAIR metadata | `/fair` | [FAIR metadata](_assets/acorn-browser-fair-preview.svg) |
| Readability | `/readability` | [Readability](_assets/acorn-browser-readability-preview.svg) |
| Content diff | `/diff` | [Content diff](_assets/acorn-browser-diff-preview.svg) |
| Big-O comparison | `/big-o` | [Big-O comparison](_assets/acorn-browser-big-o-preview.svg) |
| Geospatial | `/geospatial` | [Geospatial](_assets/acorn-browser-geospatial-preview.svg) |
| Discover | `/identifiers/discover` | [Discover](_assets/acorn-browser-discover-preview.svg) |
| Inspect | `/identifiers/inspect` | [Inspect](_assets/acorn-browser-inspect-preview.svg) |
| Install | `/install` | [Install](_assets/acorn-browser-install-preview.svg) |
| ACORN Skill | `/skill` | [ACORN Skill](_assets/acorn-browser-skill-preview.svg) |
| Settings | `/settings` | [Settings](_assets/acorn-browser-settings-dark-preview.svg) |
| About | `/about` | [About](_assets/acorn-browser-about-preview.svg) |

## Home

![ACORN Home tab](_assets/acorn-browser-workspace-preview.svg)

## Portfolio

![ACORN Portfolio tab](_assets/acorn-browser-portfolio-preview.svg)

## Research activity

![ACORN Research activity tab](_assets/acorn-browser-research-activity-preview.svg)

## Metadata standard

![ACORN Metadata standard tab](_assets/acorn-browser-metadata-standard-preview.svg)

## Crosswalk

![ACORN Crosswalk tab](_assets/acorn-browser-crosswalk-preview.svg)

## FAIR metadata

![ACORN FAIR metadata tab](_assets/acorn-browser-fair-preview.svg)

## Readability

![ACORN Readability tab](_assets/acorn-browser-readability-preview.svg)

## Content diff

![ACORN Content diff tab](_assets/acorn-browser-diff-preview.svg)

## Big-O comparison

![ACORN Big-O comparison tab](_assets/acorn-browser-big-o-preview.svg)

## Geospatial

![ACORN Geospatial tab](_assets/acorn-browser-geospatial-preview.svg)

## Discover

![ACORN Discover tab](_assets/acorn-browser-discover-preview.svg)

## Inspect

![ACORN Inspect tab](_assets/acorn-browser-inspect-preview.svg)

## Install

![ACORN Install tab](_assets/acorn-browser-install-preview.svg)

## ACORN Skill

![ACORN ACORN Skill tab](_assets/acorn-browser-skill-preview.svg)

## Settings

![ACORN Settings tab](_assets/acorn-browser-settings-dark-preview.svg)

## About

![ACORN About tab](_assets/acorn-browser-about-preview.svg)

## Charts and diagrams

Portfolio shows activity-type and status counts from the same filtered records as its table, plus a selected-record diagram linking contributor and organization counts. FAIR uses proportion bars with explicit satisfied/available counts. Readability plots the returned score against the selected metric’s configured target. Content diff pairs a before/after diagram with field or line counts appropriate to the selected diff mode. Big-O includes a labeled illustration of `n` and `n²` alongside the authoritative symbolic comparison.

Keep labels and numeric or tabular equivalents visible; do not require color, hover or a chart to understand a result. Stack charts on narrow screens. Missing values, zero denominators, off-scale scores and unsupported plot inputs need explicit states. Big-O finite samples illustrate growth and never determine symbolic dominance; the initial plot is a bundled `n`/`n²` example, not a new general expression evaluator. Other comparisons retain their structured symbolic result without suggesting the example plots their inputs.

## Theme and responsive variants

![ACORN Settings in light mode](_assets/acorn-browser-settings-light-preview.svg)

![Portrait and landscape ACORN layouts](_assets/acorn-browser-responsive-preview.svg)

![Collapsed desktop navigation](_assets/acorn-browser-collapsed-preview.svg)

![Mobile navigation drawer](_assets/acorn-browser-drawer-preview.svg)

Grouped comparison boards remain available for [validation and crosswalk](_assets/acorn-browser-validation-preview.svg), [analysis](_assets/acorn-browser-analysis-preview.svg), [identifiers](_assets/acorn-browser-identifiers-preview.svg), and [Settings themes](_assets/acorn-browser-settings-preview.svg). Update the matching individual and grouped SVGs together when a design changes.

The SVGs specify a 220 px desktop sidebar, 420 px Home column, 848 px tool/Install/Skill content column, 624 px About column and 464 px Settings column. Shared navigation, outlined icons, muted charcoal surfaces, green selection and action styles, 10 px card corners and 24 px section spacing follow the reference with documented ACORN adaptations. Version numbers and loaded data are examples, not release or runtime acceptance evidence.

# Current evidence

- ABW-21 accepted November 11: deterministic tracked-source skill folder/raw/ZIP/manifest staging is wired into browser releases. Seven packaging tests, skill validation, source/CLI extraction/staged byte parity and `make web-build` passed. The subsequent ABW-35 optimizer repair passes with one 6,259,734-byte payload and nine exports; only live reload/hosting acceptance remains in ABW-35. Durable bundle commands and metadata are in the browser README.

- Initial browser review: `acorn-web` inherits the workspace package version. `Display` and `EnumIter` own tool presentation/iteration, documentation artwork is staged from its canonical source, and bundled RAD examples cover summary, publication/organization/output and compute-resource fields. ACRE 2.0.0 is a separate downloadable CLI configuration, not a browser validation tool.

- November 11 implementation evidence: `acorn-web` now provides a local nine-operation Dioxus initial browser release with one WASM payload and unchanged `acorn-wasm` computation/adapter ownership. `acorn-desktop` remains planned. [Browser README](../../../crates/acorn-web/README.md) records commands, measured sizes, passing tests, observed Safari/Chrome behavior and unverified release rows. Concurrent design and unrelated implementation edits are preserved. Native Chrome access is restored; ABW-02 is accepted; ABW-10 is active and ABW-35 retains live reload/hosting acceptance; tool phases retain only presentation and acceptance gaps.
- `crates/acorn-wasm/examples/webmcp/` already supplies accessible manual controls, adapter invocation feedback, an unsupported-browser fallback, and a native smoke route. It proves the operations, but it is a test/demo page rather than a routed product workspace.
- `docs/src/_assets/acorn.svg`, `banner_dark_mode.png`, and `banner_light_mode.png` are the current repository-owned ACORN mark and theme banners. The shared workspace reuses those assets rather than drawing a substitute logo, and every canonical route preview embeds the exact brown vector mark from `acorn.svg`. The live OKF Plan Browser audit on November 8, 2026 confirms transferable chrome: collapsible sidebar, icon-rich navigation, version display, guided tour, theme choices, notification region, Settings/About routes, local file/drop and sample-data entry points, and the made-with-heart credit. Its About route uses a narrow left-aligned content column, prominent source/contribution card, short labeled narrative sections and cards, a three-column technology grid, and a bordered AI-disclosure card; ACORN adopts that hierarchy for its applicable project, local-first, delivery-target, source, technology, and disclosure content while keeping the tour entry point on Home rather than copying every OKF control.
- `docs/src/_assets/schema-logos/rust.svg` and `webassembly.svg` already provide repository-held Rust and WebAssembly marks. The technology credit may also use the unchanged icon-only Tauri artwork from the official `create-tauri-app` repository and the unchanged `dioxus_color.svg` from the official Dioxus brand repository, subject to retaining their provenance and license notices and without implying endorsement: [Tauri template asset](https://github.com/tauri-apps/create-tauri-app/blob/dev/templates/_assets_/tauri.svg), [Tauri trademark guidance](https://v2.tauri.app/about/trademark/), and [Dioxus brand resources](https://github.com/DioxusLabs/brand).
- The About-page AI-use disclosure identifies the tools with the official Codex login mark from the [`openai/codex` login asset](https://github.com/openai/codex/blob/main/codex-rs/login/src/assets/error.html), the official [GitHub Copilot Octicon](https://github.com/primer/octicons/blob/main/icons/copilot-24.svg), and the repository-held `docs/src/_assets/artifact-logos/microsoft.svg` vendor mark beside the Microsoft 365 Copilot label. Imported marks remain unchanged, linked labels identify the exact products, provenance and license notices are retained, and the disclosure does not imply endorsement. Home and non-About chrome identify only the human author; About has no made-with-heart line.
- The official [AI Assisted badge](https://www.aihonestybadge.com/ai-assisted-badge) describes work led and shaped by a person with AI helping, including cases where AI produced most of the code. The publisher allows personal or commercial use, hotlinking without attribution, and provides a single light/dark-compatible image with no script or tracking. ACORN bundles the unchanged [small SVG](https://www.aihonestybadge.com/badges/ai-assisted-small.svg) for offline/static delivery, links it to the badge explanation, gives it an accessible name, and keeps the more specific agent/tool disclosure beside it. The preview copy retrieved November 8, 2026 has SHA-256 `E5342204B418C42AE9F5E981E558CCEE71754252E52FCEF9A9B7ED8B4A52D5A6`.
- Driver.js 1.10.2 is the selected guided-tour implementation. Its official configuration exposes ordered steps, progress, keyboard control, close labels, reduced-animation control, and destroy hooks; the application bundles a pinned local copy of its JavaScript and CSS so the tour does not introduce a runtime CDN dependency: [Driver.js configuration](https://driverjs.com/docs/configuration), [installation](https://driverjs.com/docs/installation), and [changelog](https://driverjs.com/docs/changelog).
- `crates/acorn-service` owns portable supplied-content validation, readability, and repository-processing contracts; `acorn-analysis`, `acorn-schema`, `acorn-core`, and `acorn-diff` provide the remaining in-memory behavior. `ARCHITECTURE.md` keeps reusable browser adapters in `acorn-wasm` and UI composition in `acorn-web` and excludes native host adapters from the portable graph.
- The active [crate-boundary architecture](../../refactors/crate-boundary-architecture.md) is the ownership authority for this plan: portable computation stays in `acorn-core`, `acorn-schema`, `acorn-analysis`, `acorn-diff`, and `acorn-service`; reusable JavaScript bindings stay in `acorn-wasm`; native filesystem effects stay in `acorn-host`; and interface crates select those capabilities without importing `acorn-lib`. The [portability coordination record](../../refactors/portability/index.md) requires a concrete browser workflow before adding a narrow browser adapter contract; this workspace is that concrete consumer and must extend the matrix rather than bypass it.
- The current WebMCP adapter bounds metadata documents to 256 KiB, each diff side to 128 KiB, prose and discovery text to 16 KiB, identifier/resource values to 2 KiB, output to 256 KiB, diagnostics to 100, and discovered identifiers to 100. Both delivery modes consume those limits rather than inventing broader ones.
- The active [WebMCP and MCP interoperability plan](../../testing/webmcp-evaluations.md) owns independent evaluation data, external evaluator tooling, agent-selection evidence, layered MCP checks, and compatible-runner CI. It does not own the ACORN product workspace.
- Dioxus 0.7 documents Web as its best-supported target, compiles to `wasm32-unknown-unknown`, renders DOM nodes, and exposes browser APIs through `wasm-bindgen`: [Dioxus Web](https://dioxuslabs.com/learn/0.7/guides/platforms/web/). This matches ACORN's current target and accessibility requirements.
- Freya 0.5 added an experimental canvas-based browser backend and states that not every desktop integration works in the browser: [Freya 0.5](https://freyaui.dev/posts/0.5). The shared semantic Dioxus Web frontend plus Tauri shell is the selected cross-delivery path; Freya is not a first-release renderer.
- Resium supplies React lifecycle wrappers for CesiumJS and requires `react-dom`; CesiumJS itself requires DOM, WebGL, Web Workers, and static runtime assets. Resium recommends a Vite React project plus `vite-plugin-cesium`, which copies those assets and sets `CESIUM_BASE_URL`; Vite's official React plugin provides React Fast Refresh over its HMR transport: [Resium guide](https://resium.reearth.io/guide/), [Resium installation](https://resium.reearth.io/installation/), [Vite HMR](https://vite.dev/guide/features.html#hot-module-replacement), and [Vite plugins](https://vite.dev/plugins/#official-plugins). Preserving that map-specific Fast Refresh loop is the primary reason to select Resium instead of hand-managing the Cesium viewer from unstructured JavaScript. Resium remains suitable only as a route-local React island with an explicit bridge to Dioxus, not as a replacement application shell.
- Dioxus CLI supplies integrated development serving and hot reload for RSX, CSS, and static assets, while Tauri's `devUrl` and `beforeDevCommand` are designed to load and start a frontend development server with hot reload/HMR: [Dioxus hot reload](https://dioxuslabs.com/learn/0.7/essentials/ui/hotreload/), [Dioxus Web](https://dioxuslabs.com/learn/0.7/guides/platforms/web/), and [Tauri build configuration](https://v2.tauri.app/reference/config/#buildconfig). The accepted development workflow composes the Dioxus and Vite servers on fixed loopback-only ports; production still consumes static, relative-base-safe assets and contains no HMR client or development URL.
- CesiumJS ships a low-resolution Natural Earth II tileset under `Assets/Textures/NaturalEarthII`, and its current offline guide configures that local tileset as the viewer base layer while disabling network-backed picker and geocoder behavior. The first release therefore bundles Cesium workers/widgets plus those Natural Earth II tiles, renders local organization markers on a recognizable token-free Earth, and makes no runtime imagery, terrain, geocoding, or ion request. Remote terrain/imagery and ion credentials remain optional future work, not hidden defaults: [CesiumJS imagery](https://cesium.com/learn/cesiumjs-learn/cesiumjs-imagery/), [Ion reference](https://cesium.com/learn/cesiumjs/ref-doc/Ion.html), and [offline guide](https://github.com/CesiumGS/cesium/blob/main/Documentation/OfflineGuide/README.md).
- ROR Schema 2.1 records latitude and longitude under `locations[].geonames_details`, and the versioned ROR data dump is CC0. A pinned, generated location-only subset can resolve ROR identifiers without a runtime registry call, provided the build records the dump version, digest, license, generation command, and output size: [ROR data structure](https://ror.readme.io/docs/ror-data-structure) and [data dump](https://ror.readme.io/docs/data-dump).
- Rune is a pre-release, revision-pinned native GPU-rendered Rust UI framework with its own renderer, text, layout, state, animation, platform shells, and headless agent-driven rendering. Those capabilities overlap rather than complement Dioxus/Tauri and do not solve Resium's React requirement. Rune is therefore not added to this plan; its headless PNG/element-tree testing model remains a useful idea for a later bounded spike if the selected stack fails measured performance or testability gates: [Rune README](https://github.com/anomalyco/rune) and [Rune plan](https://github.com/anomalyco/rune/blob/dev/PLAN.md).
- Tauri 2 is frontend-agnostic and can embed a static HTML/CSS/JavaScript/WASM directory through `frontendDist`, so the built Dioxus Web output can be reused instead of creating a second renderer: [Tauri frontend configuration](https://v2.tauri.app/start/frontend/) and [Tauri configuration](https://v2.tauri.app/reference/config/). Its capability files constrain which windows may invoke commands, and production builds generate platform-specific bundles: [Tauri capabilities](https://v2.tauri.app/security/capabilities/) and [Tauri distribution](https://v2.tauri.app/distribute/).
- ORNL publicly identifies green as its signature color, but the public page does not publish a machine-readable ACORN token: [OLCF media assets](https://www.olcf.ornl.gov/about-olcf/media-assets/). The initial theme therefore uses `#00662C` only as a provisional ORNL-inspired accent unless an approved ORNL source confirms a production value. No ORNL logo or implied official endorsement is in scope.

- The November 11 [design review](design-review.md) inspected all eight rendered ACORN boards and the live OKF Home, About, Install and Planning Skill pages. The reference now makes skill discovery and app/skill acquisition prominent. The revised SVGs now include those routes. A follow-up live comparison also inspected loaded Portfolio, Settings in both themes, 390×844 portrait and 844×390 landscape. Full exceptional-state and interaction acceptance remains separate.
- `crates/acorn-cli/assets/skills/acorn/SKILL.md` is the existing canonical authored ACORN skill. `crates/acorn-cli/src/commands/skill/mod.rs` embeds it and extracts it to the platform cache; [CLI documentation](../../../docs/src/cli-reference/commands/skill.md) explains this behavior. The canonical skill bundle and compatibility manifest are now staged by the browser build; UI acquisition, desktop delivery and publication remain pending.
- `.gitlab-ci.yml` defines CLI release publication including Debian and AppImage links; this review did not run that pipeline or verify published binaries. Those entries do not establish publication of the proposed Tauri GUI. Desktop package identity, launch target and artifact provenance must be distinct from the CLI while remaining compatible when installed together.
- Primary distribution references inspected November 11: [WinGet submission](https://learn.microsoft.com/en-us/windows/package-manager/package/) requires a package manifest and repository submission; [Scoop manifests](https://github.com/ScoopInstaller/Scoop/wiki/App-Manifests) support versioned URLs, hashes, architecture and lifecycle fields; [Homebrew casks](https://docs.brew.sh/Cask-Cookbook) describe version, hash, URL and app artifacts. Tauri documents [Debian bundles](https://v2.tauri.app/distribute/debian/), [AppImage bundles and Linux baseline constraints](https://v2.tauri.app/distribute/appimage/), and [macOS signing/notarization](https://v2.tauri.app/distribute/sign/macos/). Publication credentials, public artifact access, signing identities, bucket/tap ownership and native release hosts remain unverified prerequisites.

# Boundaries and decisions

## In scope

- A new `crates/acorn-web` Dioxus Web composition root targeting `wasm32-unknown-unknown`.
- A new `crates/acorn-desktop` Tauri 2 shell that embeds the release output from `acorn-web`, opens the same routes in a system webview, and contains no second ACORN UI or computation implementation.
- A light/dark/system-aware OKF-inspired application shell using the repository ACORN mark and matching theme banner, grouped routes, collapsible desktop sidebar, accessible narrow-screen drawer, home callout, runtime status, and one icon-labeled entry per tool. The persistent chrome presents theme state as a real native toggle button with a visible switch track, light/dark label, and accessible pressed state rather than two decorative icon buttons; the Settings route may restore system preference.
- Applicable OKF Plan Browser affordances adapted to ACORN: icon-rich navigation, sidebar version and runtime summary, a Home-only Driver.js guided tour with **Try sample data** immediately to its right, an ACORN-specific Research Activity Portfolio, Settings and About routes, session-only theme control, polite notification region, one combined **Open file or folder** action beside the retained drop-zone loader, bundled sample data, visible WebMCP status, tooltips for collapsed icon controls, and the linked Home/non-About credit “Made with ❤️ by [**jhwohlgemuth**](https://github.com/jhwohlgemuth)”. About has no tour control or made-with-heart line; its AI-use disclosure uses the unchanged linked AI Assisted honesty badge and incorporates linked Codex, GitHub Copilot, and Microsoft 365 Copilot names and marks alongside the human-review explanation. The About technology cards use the Rust, WebAssembly, Tauri, and Dioxus marks with visible names, without a second “official project marks” strip. OKF planning-domain semantics, dependency graphs, repository tokens, and planning-specific prompts do not carry over. Its Install and Skill presentation patterns do carry over, with ACORN-specific distribution and workflows.
- Manual routed workflows for the nine tool names frozen by the current adapter:
  - `acorn_assess_fair_metadata`
  - `acorn_check_readability`
  - `acorn_compare_big_o`
  - `acorn_crosswalk_metadata`
  - `acorn_diff_content`
  - `acorn_discover_persistent_identifiers`
  - `acorn_inspect_persistent_identifier`
  - `acorn_validate_metadata_standard`
  - `acorn_validate_research_activity`
- Typed Rust state and result presentation over the same portable operations used by the JavaScript bindings. UI mappings may be new; domain algorithms, validation rules, limits, and result semantics are reused.
- One generated Dioxus/WASM application module that links the `acorn-wasm` rlib and retains its `wasm-bindgen` exports. The initial browser release export inspection proves this packaging seam; a silent second copy of the ACORN WASM graph is not an accepted fallback.
- Explicit `createAcornWebMcpAdapter(...)` installation, status, invocation feedback, and disposal. Importing the application or adapter remains side-effect free until the page installs it.
- Platform effects behind one typed UI boundary: native browser file/folder selection and download in browser builds and user-mediated Tauri file/folder open plus file-save dialogs in desktop builds. A loaded folder becomes a deterministic session-only tree of supported text files; selecting a leaf supplies that file to a manual tool and never runs tools automatically. The desktop backend traverses only the root returned by its native folder dialog inside a bounded Rust command, so the webview receives sanitized relative paths and content but never a reusable filesystem capability, absolute path, persisted directory grant, or ambient home-directory access.
- Structured results before optional raw JSON: ordered diagnostics, FAIR profiles/findings, readability checks, conversion warnings, unified diff or JSON Patch, and normalized identifier states.
- A `/portfolio` route that derives a deterministic, read-only view of the Research Activity Data loaded in the current session: activity, organization, contributor, and persistent-identifier metrics; activity-type distribution; text and status/type filters; a sortable semantic activity table; and selected-record details. The visual summary never becomes the sole representation, invents no new domain state, and performs no network or persistence work.
- A `/geospatial` route that extracts ROR identifiers from loaded Research Activity Data, resolves them against a lazy-loaded, versioned ROR 2.1 location subset, and plots deduplicated organizations in a token-free CesiumJS globe. Dioxus owns route state, filters, selection details, the accessible organization table, empty/error/WebGL-unavailable states, and all ACORN data handling; a route-local React/Resium island owns only the Cesium viewer and marker lifecycle. During development, that island is served by Vite with React Fast Refresh while the surrounding Dioxus application retains its own hot-reload loop; release builds replace both development clients with one static asset graph.
- A semantic DOM, keyboard navigation, visible focus, route-title/focus management, restrained live regions, reduced motion, 200% zoom, and desktop/phone portrait/phone landscape acceptance.
- An ORNL-inspired green accent system with accessible solid, subtle, hover, selected, and focus tokens. The production documentation must distinguish an approved brand value from a provisional inspired value.

## Out of scope

- Reimplementing ACORN rules, codecs, crosswalks, identifier logic, FAIR checks, readability, or diff behavior in UI code.
- Replacing `webmcp.js`, its contract tests, the existing native smoke, or the independent WebMCP evaluation plan.
- Unprompted filesystem access, traversal outside the explicitly selected folder, symlink following, directory watching, PDF or PowerPoint rendering, databases, shell/process execution, inference, `doctor`, authenticated repository operations, a server requirement, or remote persistence.
- A TypeScript/React application shell, server-side rendering, a separate desktop UI, a Freya or Rune production renderer, or a generic dynamic-form framework. The bounded Resium map island is the only React-owned surface.
- Cesium ion credentials, remote terrain or imagery, geocoding, automatic ROR API calls, network-fetched map data, user tracking, or making the WebGL globe the only way to inspect organization locations.
- In-app auto-update, background startup, tray integration, global shortcuts, deep links, remote content, mobile Tauri targets, Microsoft Store/App Store publication, and admission to the Debian archive. Production signing/notarization and the five requested distribution channels are in scope; credentials and authorized release hosts are external prerequisites. Unsigned local bundles are development evidence only. Package-manager updates do not require an updater plugin.
- Automatic clipboard writes, automatic downloads, tool execution on page load, telemetry, or ACORN-originated network requests.
- Copying OKF Plan Browser source, planning content, portfolio concepts, or product name. The reference governs information architecture and visual rhythm only.
- Using an ORNL logo, DOE mark, trademarked asset, or claim of official branding without an approved repository asset and authorization.

## Product and architecture decisions

1. **Use Dioxus Web and the DOM.** The product requires native inputs, semantic navigation, browser file controls, text-heavy results, and screen-reader evidence. Canvas-first rendering is not the first-release accessibility risk profile.
2. **Keep one UI composition root.** `acorn-web` owns routes, presentation state, and a typed platform-effect interface; portable ACORN crates own computation; `acorn-wasm` owns JavaScript DTO conversion and WebMCP bindings. `acorn-desktop` embeds the compiled frontend and implements only the native side of approved effects. Browser, webview, and Tauri types do not cross into portable crates.
3. **Prove one WASM payload before page work.** Link the existing `acorn-wasm` rlib into the Dioxus module and verify the generated JavaScript exports all nine bindings. If a future build cannot preserve those exports, a bounded repair phase must decide the packaging change; do not ship two unmeasured copies.
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
15. **Isolate Resium as a hot-reloadable map island.** The primary reason for Resium is a productive geospatial development loop: a small Vite-built React module mounts only inside the `/geospatial` viewer host and receives React Fast Refresh/HMR without rebuilding the Dioxus WASM application for compatible TSX or map-style edits. Dioxus passes an immutable, bounded marker DTO and receives selection/camera/error events through one versioned JavaScript bridge. The island does not parse RAD, resolve ROR, own application navigation, duplicate theme state, call ACORN operations, register WebMCP tools, or import Tauri APIs. Rust, bridge-schema, package, and Vite-configuration changes may trigger their normal rebuild or full reload; the plan does not claim that every edit preserves state.
16. **Keep geospatial resolution local and inspectable.** Generate a minimal ROR-id/display-name/latitude/longitude/country index from a pinned Schema 2.1 data dump, record its digest and CC0 provenance, and lazy-load it only on `/geospatial`. Deduplicate by normalized ROR ID, preserve every source reference count, report unresolved and locationless identifiers, and provide the same ordered organization table when WebGL is unavailable. Cesium starts with its bundled Natural Earth II TMS imagery and without ion, geocoder, remote imagery, or terrain.
17. **Do not add Rune to the first-release stack.** Rune's renderer, layout, state, and platform shells compete with Dioxus/Tauri rather than supplying missing components. Reconsider it only in a separate time-boxed spike with explicit parity, accessibility, Web/desktop, Resium integration, binary-size, and headless-test gates after a measured failure of the selected architecture; borrowing its headless-testing concepts does not require a dependency.

## Crate-boundary contract

| Package | Role in this plan | Forbidden ownership |
|---|---|---|
| `acorn-core`, `acorn-schema`, `acorn-analysis`, `acorn-diff`, `acorn-service` | Existing portable values, codecs, operations, findings, and orchestration consumed directly by first-party interfaces | DOM, Tauri, filesystem, WebMCP registration, presentation state, or duplicated UI-specific result models |
| `acorn-wasm` | Reusable `wasm-bindgen` ABI, JavaScript DTO conversion, and the existing nine-tool WebMCP adapter | Dioxus routes/components, CSS, Tauri commands, native host effects, or a dependency on `acorn-web` |
| `acorn-web` | Static Dioxus interface composition root: routes, components, presentation state, accessibility, ephemeral DOM effects, and the narrow bridge to a route-local Resium map island | Portable domain policy, duplicate JS DTO/adapter logic, a React application shell, ACORN logic inside the map island, native filesystem/process dependencies, `acorn-host`, or `acorn-lib` |
| `acorn-host` | Existing narrow native filesystem owner; ABW-12 may add the missing explicit text-write primitive beside `read_text`, and ABW-17 may add bounded selected-root traversal, each with focused tests | Tauri window/dialog types, Dioxus state, product routing, or WebMCP policy |
| `acorn-desktop` | Thin Tauri interface composition root that embeds `acorn-web`, owns native file/folder dialog selection and IPC authorization, and delegates selected-file reads/writes plus selected-root traversal to `acorn-host` | Reimplemented ACORN operations, direct `std::fs` policy, a second UI tree, reusable browser adapters, reusable path authority, or privileged WebMCP bridging |
| `acorn-lib` | No role in the new browser or desktop dependency graphs | Compatibility re-exports or host shortcuts that conceal the narrow owning crates |

`acorn-web` is a new interface root, not a replacement browser-adapter owner. Its DOM effects are presentation mechanics over user-supplied in-memory values; reusable JavaScript ABI and WebMCP behavior remain in `acorn-wasm`. If a required operation is absent from the portable owners, the implementing phase stops and opens a bounded extraction/repair phase under the crate-boundary architecture instead of adding the operation to either UI crate.

# Routes and first-release behavior

| Route | Manual workflow | Required presentation |
|---|---|---|
| `/home` | Choose the Portfolio, a tool, geospatial view, local file, recursively loaded local folder, dropped content, or bundled sample and inspect runtime state | ACORN artwork, adjacent **Take a tour** and **Try sample data** actions in that order, prominent ACORN Skill card, Install/Skill navigation, supporting privacy copy, combined file-or-folder loader, retained drop zone, session file tree, Portfolio entry, nine icon-labeled WebMCP tools plus geospatial navigation, theme toggle button, notification region, WebMCP status, version, and author-only made-with-heart credit |
| `/portfolio` | Explore and filter the Research Activity Data loaded in this session | Deterministic activity, organization, contributor, and PID metrics; activity-type/status bar charts and selected-record relationship diagram with equivalent text/table values; type/status/text filters; sortable semantic activity table; selected-record details; empty and single-record states; narrow-screen list-first reflow |
| `/validate/activity` | Validate Research Activity JSON and optional local file | Validity, effective fuzzy policy, ordered/truncated diagnostics |
| `/validate/standard` | Validate supplied content/format/standard | Validity, supported-combination errors, ordered diagnostics |
| `/crosswalk` | Run a supported typed crosswalk | Converted content, output format, mandatory loss warnings, copy/download |
| `/fair` | Assess supported JSON metadata | 41 findings, profile and per-principle proportion bars with counts, no-resolution notice |
| `/readability` | Run one supported metric | Score, metric-specific target gauge, threshold, acceptable/excessive state, returned summary |
| `/big-o` | Compare two Big-O expressions with explicit symbol assumptions | Normalized expressions, assumptions, left/right/equivalent/incomparable/undetermined dominance, structured invalid-input feedback, and optional raw result; 256 UTF-8 bytes per expression/assumption name and at most 64 assumptions |
| `/diff` | Compare before/after content | Unified or RFC 6902 result, before/after diagram and mode-appropriate change counts, changed state, copy/download |
| `/identifiers/discover` | Discover identifiers in text | Ordered normalized identifiers, count, truncation |
| `/identifiers/inspect` | Inspect one identifier and optional kind | Valid, invalid, or ambiguous state and deterministic candidates |
| `/geospatial` | Map organizations referenced by ROR identifiers in loaded Research Activity Data | Token-free Resium/CesiumJS globe, deduplicated local markers, unresolved/locationless summary, selected-organization details, synchronized accessible table, WebGL fallback, and ROR dataset version/provenance |
| `/install` | Acquire or update the desktop app and matching skill | OKF-style app/skill entry cards, five-channel platform table, exact supported architectures, versioned artifact links/checksums, explicit publication status, upgrade/removal instructions, and a link to `/skill` |
| `/skill` | Read, copy or download the release-matched ACORN skill | Canonical source link, app/skill version and digest, workflow guide, raw definition, complete folder download, compatibility and manual install/update/removal instructions; works offline from bundled content |
| `/settings` | Configure session-only appearance and reset transient state | Dark, light, and system choices; current version/runtime; no persistence claim |
| `/about` | Explain boundaries, authorship, AI use, and implementation technology | Narrow OKF-inspired content column with source card and no tour control, project narrative, local-first and delivery-target cards, three-column technology grid with labeled marks and no duplicate mark strip, version/runtime and WebMCP boundaries, bordered AI-use disclosure with the linked AI Assisted badge plus linked agent names and marks, and no made-with-heart footer |

# Dependencies and coordination

- The [crate-boundary architecture](../../refactors/crate-boundary-architecture.md) is an upstream coordination dependency, and this plan is its concrete atomic browser/desktop consumer. That record remains authoritative for ownership direction; this plan owns the bounded implementation phases. The [portability coordination record](../../refactors/portability/index.md) supplies the existing WASM/feature gates records the accepted single-payload `acorn-web` composition.
- ABW-20 is accepted with six rendered exceptional-state previews and a dated 1440×900 live comparison; the shared shell, Install/Skill pages and nine-tool inventory are already reflected in the revised SVGs. The browser composition is accepted; exceptional-state design does not block local review. Future manifest/lockfile edits still require exclusive ownership.
- The branded shell is accepted; ABW-10 completes the applicable OKF home/chrome affordances. ABW-03 follows both. ABW-04 and ABW-05 currently share `src/ui.rs` and `src/operations.rs`, so they serialize. Parallel leaf work requires an explicit module extraction and named disjoint write sets first.
- ABW-06 follows the tool pages because single-file/copy/download behavior must be accepted against real result types, not speculative abstractions. ABW-16 then adds the bounded recursive browser folder workspace over that frozen effect contract. ABW-19 follows ABW-03 and ABW-16 so the Portfolio consumes accepted validated RAD and session-tree state rather than creating a parallel loader or domain model.
- ABW-18 follows ABW-05 and ABW-19 so it can consume the accepted portfolio state and normalized ROR identifiers. It exclusively owns the React/Resium/CesiumJS island, coordinated Dioxus/Vite development workflow, Vite staging, and generated ROR location index; no other phase may introduce React or change `Makefile`, frontend watchers, or final static-asset assembly concurrently.
- ABW-07 completes workspace integration acceptance after all tool groups. It serializes WebMCP staging and `Makefile` changes with WME-02, WME-04, and any other owner of `webmcp.js` or WASM package assembly. It consumes the adapter unchanged unless a separate evidence-backed repair expands its write set.
- ABW-08 requires the complete manual and geospatial route matrix, including the map island's static assets, WebGL fallback, and accessible table. Live WebMCP registration requires a supported Chrome build and flag/configuration; missing support blocks only that matrix row, not manual browser acceptance.
- ABW-11 follows ABW-18 as well as the frozen browser build seam and shell, then claims the workspace manifest/lockfile alone to add the Tauri shell and connect `cargo tauri dev` to the accepted coordinated hot-reload workflow. ABW-12 follows the accepted browser effects, serializes the `acorn-host` filesystem leaf, and adds bounded native single-file open/save commands without a general filesystem permission or direct filesystem implementation in `acorn-desktop`. ABW-17 then adds one selected-folder command against the ABW-16 tree contract and the same `acorn-host` ownership boundary.
- ABW-13, ABW-14, and ABW-15 validate local bundles on Windows, macOS, and Linux respectively after ABW-17 and ABW-08. Their platform-specific files are disjoint and may proceed in parallel on authorized native hosts; each unavailable host remains unverified rather than passing by inference.
- The canonical skill bundle is accepted against the browser composition; ABW-22 exposes it after the shared chrome and browser/native file effects exist. ABW-23 freezes release identities and manifests after ABW-11 and the accepted skill bundle; ABW-31 consumes that contract for Install UI before ABW-08. ABW-24/25 produce signed Windows/macOS releases after their native acceptance; ABW-26/27 publish separate Debian/AppImage artifacts after Linux acceptance. ABW-28/29 consume the Windows release for WinGet/Scoop; ABW-30 consumes the notarized macOS release for Homebrew cask. Missing hosts, signing, public URLs or channel ownership block only the dependent publication phases.
- ABW-09 follows all browser and desktop behavior, skill lifecycle, and five-channel publication acceptance phases. It owns final documentation, broad gates, and plan retirement.
- The [WebMCP and MCP interoperability plan](../../testing/webmcp-evaluations.md) remains authoritative for external evaluator, model-selection, MCP Inspector/conformance, public MCP control, and required-CI decisions. This plan owns the product page and manual parity.
- ABW-32 completes Big-O visualization and edge-case acceptance after ABW-03. It must pass before file/result effects ABW-06 and WebMCP product installation ABW-07; the existing adapter is unchanged.

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

The Dioxus production build must record generated JavaScript/WASM sizes and prove that the nine `wasm-bindgen` exports and routed application share one payload. Generated output stays under `target/`.

The Tauri build must consume that exact release asset directory through `frontendDist`; it must not fetch a remote page or stage another UI. `cargo tauri build` runs on each native desktop host because platform bundles are host-specific. Generated binaries, installers, and Tauri schemas stay under ignored build output.

## Development feedback loop

- `make web-dev` starts `dx serve` for the Dioxus application and `npm run dev` for the route-local geospatial island together on documented, fixed loopback-only ports. Startup fails clearly if either server fails; interruption tears both down without leaving a watcher behind.
- Development-only island resolution loads the Vite entry from its local server so TSX and map-style edits use React Fast Refresh/HMR without a Dioxus/WASM rebuild. Dioxus RSX, CSS, and static assets keep the framework's own hot-reload behavior. Changes outside those supported boundaries may rebuild or reload normally.
- `cargo tauri dev` uses `beforeDevCommand` to start the same composite frontend workflow and points `devUrl` at the Dioxus development server, so the system webview observes both Dioxus reloads and Resium Fast Refresh. It does not use `frontendDist` until a production build.
- Release builds fail acceptance if generated HTML, JavaScript, configuration, or the Tauri bundle contains a localhost development URL, Vite HMR client, React Refresh preamble, or development-only cross-origin/CSP allowance.

## Browser matrix

- Desktop at 1440×900.
- Phone portrait at 390×844.
- Phone landscape at 844×390; do not apply desktop sidebar behavior merely because the width exceeds a small-phone breakpoint.
- Chrome, Firefox, and Safari/WebKit for manual tools when available. WebMCP registration is a separate supported-Chrome row, not a universal-browser expectation.
- Keyboard-only route, drawer, form, result, disclosure, copy, and download flows.
- Visible focus, skip link, route heading focus, Escape/return focus for the drawer and guided tour, 200% zoom, reduced motion, status announcements, icon tooltips, and no color-only validity or active state.
- Dark, light, and system theme states; collapsed desktop sidebar; narrow-screen drawer; version/runtime display; tour steps; local open/drop/sample entry points; notification lifecycle; Settings/About routes; and made-with-heart footer.
- Geospatial route with representative duplicate, invalid, unresolved, and locationless ROR values; marker/table selection parity; bundled Natural Earth II imagery; no-ion/offline mode; WebGL-unavailable fallback; theme and resize changes; lazy loading; and no external map, registry, terrain, imagery, geocoder, or telemetry request.
- Development smoke proves Dioxus RSX/CSS/asset hot reload and Vite React Fast Refresh for representative geospatial TSX and map-style edits without manually rebuilding the full application; expected full-reload boundaries are documented rather than disguised as HMR failures.
- Valid, invalid, empty, oversized, operation-error, and unsupported-capability states for each applicable tool.
- Browser console free of application errors; no ACORN operation request observed on the network panel.

Static, unit, or headless checks do not count as live desktop/portrait/landscape or screen-reader evidence. Record unavailable browsers or assistive technologies as unverified rather than passing them by inference.

## Desktop matrix

- Windows 11 native WebView2, macOS native WebKit, and a supported Linux WebKitGTK host; record OS, architecture, webview version, bundle type, and exact unsigned artifact path for every observed row.
- Launch from `cargo tauri dev` and from the locally built bundle with networking unavailable; both must show the same routes, ACORN assets, theme states, keyboard behavior, and bounded operation results as the browser build.
- Under `cargo tauri dev`, representative Dioxus and geospatial-island edits reach the existing webview through their respective hot-reload channels. The production bundle launches without either development server and contains no development client or loopback dependency.
- Native open and save dialogs cover success, cancel, invalid extension, invalid UTF-8, oversized input, safe suggested filename, write failure, and preservation of current editor/result state.
- Capability inspection proves the bundled main window only loads local assets and cannot invoke filesystem, shell, process, updater, or remote-content APIs. The three custom file/folder commands reject forged paths because callers never submit paths; the folder result exposes only normalized relative paths beneath the dialog-selected root.
- WebMCP status reflects the embedded webview's real support. Unsupported is an accepted desktop state; no native IPC shim or elevated tool registration is present.
- Platform-specific packaging checks run on their native hosts. Unsigned local acceptance does not satisfy publication. Production releases need the channel-specific signing/notarization, immutable public downloads, manifest validation and installed-app evidence defined below.

# Acceptance criteria

- ABW-20 closes the [design-review findings](design-review.md): all route previews match the live OKF shell/layout at equal viewports with documented ACORN adaptations, Install and Skill are included, and remaining exceptional-state behavior is accepted before full release.
- The canonical ACORN skill is bundled from the same release revision into CLI, browser and desktop artifacts. Build checks prove byte/hash and version compatibility; install, upgrade, rollback and removal follow the lifecycle contract below without silently changing user-managed agent folders.
- The desktop GUI is published and installable through WinGet, Scoop, a Debian `.deb`, an AppImage, and a Homebrew cask. Each channel records public artifact/manifest URLs, exact version/architecture/checksums and native install/launch/update/removal evidence. A local bundle, pending upstream PR, or inaccessible URL does not satisfy publication.

- `crates/acorn-web` is the documented Rust/Dioxus composition root and compiles for `wasm32-unknown-unknown` without native ACORN host dependencies; `crates/acorn-desktop` embeds that exact static build in Tauri 2 without duplicating routes, components, or domain logic.
- `cargo tree` and target checks prove `acorn-web` has no `acorn-host` or `acorn-lib` edge, `acorn-wasm` has no UI or native-host edge, and `acorn-desktop` delegates selected-path filesystem work to the narrowly enabled `acorn-host/fs` capability rather than owning direct filesystem policy.
- The application uses an OKF-inspired light/dark/system shell, grouped collapsible icon-rich sidebar, centered home workflow, responsive drawer, tool entries, Settings/About routes, locally bundled Driver.js tour, notification region, and routed detail pages without copying OKF planning content or source. The persistent theme control is one native toggle button with visible state, switch geometry, and correct `aria-pressed`; Settings can restore system preference.
- The repository `acorn.svg`, dark banner, and light banner are reused with appropriate accessible alternatives; `v0.6.3`-style version placement derives from the built package rather than hard-coded UI state. Home and non-About footers display exactly “Made with ❤️ by [**jhwohlgemuth**](https://github.com/jhwohlgemuth)” and contain no agent names or marks. About contains no made-with-heart line. Its AI-use disclosure displays the unchanged, locally bundled [AI Assisted badge](https://www.aihonestybadge.com/ai-assisted-badge) with an accessible name and link, then names and links [**Codex**](https://openai.com/codex/), [**GitHub Copilot**](https://github.com/features/copilot), and [**Microsoft 365 Copilot**](https://www.microsoft.com/microsoft-365/copilot) beside their marks and the human direction/review/testing explanation. Each linked label and adjacent mark belongs to one accessible external link, wraps without clipping at narrow widths, and retains a readable text-only meaning when images are unavailable.
- The About page displays unchanged Rust, WebAssembly, Tauri, and Dioxus marks inside their existing technology cards with adjacent text labels, useful alternative text, provenance, and retained license notices. It does not repeat them in a separate “official project marks” strip. The marks do not appear in the persistent footer, remain secondary technology credits, are not recolored or combined into a new logo, and do not imply project endorsement.
- The About page follows the live OKF reference hierarchy closely while using ACORN content: narrow left-aligned column, source/contribution card, project and local-first sections, delivery-target cards, three-column technology grid, and bordered AI-use disclosure. It contains no tour action; the guided-tour entry point remains on Home. The disclosure leads with the AI Assisted honesty badge, incorporates linked names and marks for the three agent products, explains the maintainer's direction/review/testing role, and advises review of AI-generated code; no separate attribution footer is present.
- The accent system uses an approved ORNL signature-green value or explicitly labels `#00662C` and its derived variants as ORNL-inspired/provisional. Purple and blue are not the primary interactive accent; no unapproved logo is present.
- All manual operations accepted in ABW-20 are reachable from the home page and sidebar, use existing ACORN computation, enforce current bounds, and render structured results before optional raw JSON.
- The Portfolio is reachable from Home and the sidebar, derives all counts, filters, distribution, table rows, and selected details from the same loaded Research Activity session state, provides semantic equivalents for every visual summary, and handles empty, single-record, multi-record, and narrow-screen states without network requests or persistence.
- One Dioxus/WASM payload contains the application and all nine JavaScript bindings, with recorded release size and no silent duplicated ACORN module.
- `/geospatial` plots deduplicated ROR-backed organization locations from loaded RAD using the isolated Resium/CesiumJS island while Dioxus retains all ACORN state and an equivalent accessible table. The production build includes Cesium workers/widgets, its locally served Natural Earth II imagery, and a lazy, pinned, digest-verified ROR 2.1 location subset with license/provenance; makes no external map or ROR request by default; requires no ion token; and reports unresolved/locationless records plus WebGL failure without losing the table.
- The documented browser and Tauri development commands start one coordinated Dioxus/Vite feedback loop: supported Dioxus UI/style/asset edits hot reload, and compatible Resium TSX/map-style edits use React Fast Refresh without rebuilding the WASM application. Watcher startup, failure, and shutdown are deterministic; production output contains no HMR client, refresh preamble, localhost URL, or development-only CSP allowance.
- The page explicitly installs exactly nine WebMCP tools when supported, reports installation state, routes invocation feedback safely, disposes registrations, and keeps manual controls functional when unsupported.
- Local file or folder selection never uploads content, failed reads preserve the current editor, and the session tree contains only normalized relative paths and supported UTF-8 content under the 500-file/20-segment/256-KiB-per-file/16-MiB-total limits. Unsupported and rejected entries are summarized; no operation runs until the user selects a leaf and invokes it. Copy/download/save actions require explicit user action, object URLs are revoked, and generated filenames are safe. Desktop file/folder open and save occur inside bounded Rust commands after native dialog selection, delegate filesystem work to `acorn-host`, never follow symlinks, and grant no ambient filesystem or absolute-path access to the webview.
- Route changes set a useful document title and focus; navigation, tools, disclosures, results, copy/download, and the responsive drawer pass keyboard acceptance with visible focus and semantic DOM controls.
- Desktop, phone portrait, and phone landscape pass without horizontal page overflow. The globe yields space or collapses behind the accessible organization list at narrow sizes without hiding data. Manual-tool acceptance records the browsers actually observed; WebMCP acceptance records the exact supported Chrome version/configuration.
- Unsigned local Tauri bundles launch and pass the shared manual-tool contract on observed Windows, macOS, and Linux hosts. Each platform row records native evidence; an unavailable host remains a named blocker. The desktop app works offline, loads only bundled assets, exposes only the three approved file/folder dialog commands, and does not elevate WebMCP.
- Focused Rust, WASM, adapter, browser, Tauri, capability, and native-package checks pass. Final `make lint`, `make test`, `make check-wasm`, documentation build, `git diff --check`, and wiki validation pass without unrelated changes.
- Durable user/developer documentation records browser installation and serving, Tauri prerequisites and local build commands, static-host base paths, privacy/network/native-capability boundaries, supported tools, WebMCP limitations, browser and native-platform evidence, build sizes, unsigned-bundle limitations, and brand-token status, plus public channel commands and the coordinated skill lifecycle.
- After all criteria pass, ABW-09 deletes this plan and removes its feature-index, portfolio, dependency-network, and parallel-lane entries.

# Desktop distribution and ACORN skill lifecycle

## Release contract

Publish the **Tauri desktop GUI**, using one workspace-versioned frontend/skill payload. Keep the desktop package name, application identifier, executable and install paths distinct from the existing CLI artifacts; installing or removing the GUI must not replace `acorn` on PATH or remove a separately installed CLI. ABW-23 freezes collision-checked channel identifiers, concrete public download origin, supported architecture/OS rows, publisher identity, maintainer-owned Scoop bucket and Homebrew tap URLs, and the release manifest schema before packaging proceeds. Prefer existing ACORN release infrastructure after proving anonymous access; private GitLab package URLs cannot be advertised as public downloads.

| Channel | Required artifact and publication outcome | Acceptance evidence |
|---|---|---|
| WinGet | Signed Windows installer, versioned installer URLs and SHA-256 values, application identity and silent install/uninstall metadata; accepted manifest in `microsoft/winget-pkgs` | Manifest validation, public source discovery, clean install/launch, upgrade from a prior accepted version and uninstall without CLI collision |
| Scoop | Windows desktop archive with the complete runtime/resources and Start Menu shortcut in a maintainer-owned public bucket; pinned hashes and architecture mapping | Bucket add/install/update/uninstall; GUI launch and WebView2 prerequisites; `checkver`/`autoupdate` resolve verified assets rather than mutable bytes |
| Debian | Desktop `.deb` and checksum published as immutable release assets; GUI desktop entry/icon, runtime dependencies, architecture and license metadata | Install with dependency resolution on the oldest declared Debian baseline, offline launch, upgrade and remove/purge behavior; no claim of Debian archive or apt-repository inclusion |
| AppImage | Desktop `.AppImage` and checksum published as immutable release assets per supported architecture | Launch on the oldest declared Linux baseline and a second declared distribution, documented FUSE/extraction requirements, executable permission, manual replacement/rollback/removal; no universal-portability claim |
| Homebrew cask | Signed and notarized macOS app in a DMG/ZIP, published through a maintainer-owned public tap with version/architecture hashes and `app` stanza | Cask audit/style, tap install/launch, upgrade and uninstall with Gatekeeper enabled; no assumption that admission to `homebrew/cask` is automatic |

ABW-13/14/15 retain their value as local functional acceptance. ABW-24–30 add production artifacts and public channel acceptance. Signing keys, Apple credentials, public release access, bucket/tap ownership, native hosts and upstream review are external prerequisites; do not invent their availability or count waiting as active implementation time. A pending WinGet submission remains a blocker to that channel's completion. Additional architecture rows require their own native evidence; x86_64/ARM64 are never implied merely by listing an OS.

Release jobs verify the shared frontend and skill digests before publishing versioned artifacts; channel manifests must reference those bytes. Publish channel availability separately from intended support so `/install` can represent pending/unavailable channels truthfully. Upgrades use the package manager or explicit replacement; no background network checks, updater plugin, agent execution, credential collection or extra IPC capability is added.

The application publication inventory is `packaging/applications.json`, enforced by `make check-release-apps` through Rust `xtask` commands and tests. Packaging, verification and immutable uploads also live in `xtask/src/application_release/`; future desktop release validation extends that module. Browser preview builds, checks, deterministic ZIP/checksum packaging, immutable generic-registry upload and GitLab release links are wired into the existing default-branch version-change lifecycle. The archive carries workspace version, source revision, content hashes and the desktop's planned state. GitHub Pages is enabled with a main-only deployment environment. The mirror-triggered workflow builds and smoke-tests the preview, with local root and `/acorn/` Chromium evidence; first hosted deployment and HTTPS enforcement await the workflow reaching the mirror and GitHub certificate provisioning. Anonymous access to the GitLab release archives and full release acceptance remain unverified. The desktop crate is absent and no native download is advertised. Both GUI distributions are application artifacts only, with `publish = false` required when their crates exist.

## Skill lifecycle contract

1. **Author and review:** retain `crates/acorn-cli/assets/skills/acorn/` as the single authored source. Update the skill whenever UI routes, available operations, limits, WebMCP behavior or desktop capabilities change. Distinguish browser manual operations, optional WebMCP and CLI-only host operations; the GUI does not execute the agent skill.
2. **Build and release:** derive the raw definition, complete skill archive and a manifest containing app version, source revision, skill digest and compatibility information from that source at build time. Bundle the same bytes in browser and Tauri output, retain the CLI embedding, and publish the archive with the same release. CI rejects stale staging, missing referenced files, mismatched hashes/versions and undocumented operation drift. Skill-only changes still produce a coordinated app/skill release; no independently floating skill download.
3. **Discover and install:** Home links prominently to `/skill`; shared navigation includes `/install` and `/skill`; About explains their relationship. The skill page reads bundled content offline, offers explicit copy/download actions and complete repository-skill installation instructions, and links to canonical source and the matching versioned release. Browser builds may download the full archive through the existing content-only download boundary; desktop builds export the current raw definition through the existing `save_text` dialog and link to the matching public archive for full-folder acquisition. Do not add binary-save, shell, updater or agent-install commands to IPC. A downloaded skill may be used independently in a compatible agent; it grants no additional permission or app access.
4. **Upgrade and rollback:** replacing the app replaces its bundled skill atomically. User-exported or agent-installed copies remain user-managed; show their version/digest and instructions to compare and explicitly replace them. Do not claim the UI can detect external skill copies without user-supplied evidence. Retain prior versioned app/skill artifacts for rollback; never pair an older app with a silently fetched latest skill.
5. **Remove:** uninstall removes package-owned bundled skill resources with the app. Preserve separately exported skills, repository copies, the separately installed CLI and user data; describe explicit optional cleanup rather than deleting agent directories from package scripts. Browser session clearing does not remove or reinstall any skill.

## Additional validation

For every native channel, record the exact OS/architecture, artifact URL/hash, app version, skill manifest/digest, GUI launch, offline manual operation and file-dialog smoke, install/upgrade/removal result, and preservation of unrelated CLI/agent files. Exercise version mismatch and rollback from a previous release. Check the skill source, CLI-embedded definition, browser download, desktop-bundled download and published archive for equality. The browser and desktop Install/Skill routes also enter the normal keyboard, focus, dark/light and responsive matrix. Availability of a build or manifest alone is not successful publication.

# Browser execution milestone — November 11, 2026

The authorized implementation milestone is a locally runnable browser for use, review and testing. It includes the shared Dioxus shell, nine manual operations, structured results, explicit local input and result download, theme/responsive navigation, and optional WebMCP installation. `make web-dev`, `make web-build`, `make web-preview`, `make web-check` and `make web-test` provide reproducible entry points. Product completion still requires the remaining exceptional-state design, full Portfolio/folder/map, skill lifecycle, desktop and publication phases below; none is implied by initial browser release acceptance.

The single-payload composition and nine-operation initial browser release are implemented and exercised. Durable evidence is in [the browser README](../../../crates/acorn-web/README.md). Completed initial browser release phases have been pruned; remaining phase outcomes below cover full-release work only. The browser milestone does not wait for desktop signing, channel identities, native save dialogs or geospatial assets. ABW-22's desktop-only export and ABW-31's published-channel metadata remain release prerequisites, while initial browser release navigation can truthfully identify their pending status.

One integration owner edits manifests/lockfile, `Makefile`, route and module registration, shared components and planning indexes. Parallel page assignments own only their named operation/page/test leaves after ABW-03 freezes the shared result contract; ABW-04 and ABW-05 must not concurrently edit `pages/mod.rs`, `route.rs` or this plan. ABW-11/12 serialize with ACD-03, SPD-24 and dependency-audit manifest work. ABW-07/18 serialize with WebMCP and other build owners. Final broad gates serialize after the implementation batch.

# Prepared execution batch — November 11, 2026

The [execution brief](execution-preparation.md) prepares **ABW-10 → ABW-03 → ABW-04 → ABW-05 → ABW-32 → ABW-06** against `d0fe5103`. Execute serially through the current shared UI and operation files; readiness remains conditional on each phase's prerequisites. ABW-21 is now accepted; ABW-35 retains live acceptance after its optimizer repair. ABW-20 is accepted; native Chrome access is restored and ABW-02 is accepted; ABW-10 is in progress. The brief records inspected code, validation limits, ownership, live-evidence gates and the next action.

# Execution phases

## ABW-22 — Expose the release-matched ACORN Skill workflow

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-06, ABW-10, ABW-12 and accepted skill bundle
- Exact write set: `crates/acorn-web/src/pages/skill.rs` (new), `crates/acorn-web/src/pages/mod.rs`, `crates/acorn-web/src/pages/home.rs`, `crates/acorn-web/src/pages/about.rs`, `crates/acorn-web/src/pages/settings.rs`, `crates/acorn-web/src/components/sidebar.rs`, `crates/acorn-web/src/route.rs`, `crates/acorn-web/src/tests/skill.rs` (new), `crates/acorn-web/assets/app.css`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-skill-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-workspace-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-about-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-settings-preview.svg`
- Primary integration lock: shared Skill route and application chrome
- Outcome: Implement the OKF-aligned skill guide, prominent Home card and shared navigation using bundled content; expose app/skill version, source and digest, explicit raw/copy/archive actions, and manual install/update/rollback/removal instructions. Show only bundled compatibility facts; never inspect or overwrite agent folders or run the skill.
- Focused validation: Offline route and download smoke; browser archive and raw bytes match the accepted bundle manifest; desktop raw export uses `save_text` and full-folder acquisition links to the exact versioned archive; explicit-action clipboard/download tests; missing/corrupt/mismatched manifest rejection; keyboard/focus and dark/light responsive checks; About/Settings links and Home hierarchy match accepted previews; browser and Tauri use the same route.
- Handoff: Remove ABW-22 and retain lifecycle documentation plus parity tests; include Skill in ABW-08 and every native package acceptance row.

## ABW-23 — Freeze desktop release identity and publication metadata

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-11 and accepted skill bundle; maintainer-confirmed public release origin, publisher identity, Scoop bucket and Homebrew tap destinations; no concurrent CI/build owner
- Exact write set: `packaging/applications.json`, `xtask/Cargo.toml`, `xtask/src/application_release/`, `Makefile`, `crates/acorn-desktop/release/contract.md` (new), `crates/acorn-desktop/release/manifest.schema.json` (new), `xtask/src/application_release/desktop.rs` (new), `.gitlab-ci.yml`, `crates/acorn-desktop/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: desktop release manifest and CI integration
- Outcome: Freeze collision-checked desktop identities, exact supported OS/architecture rows, immutable asset naming/URLs, channel ownership, signing requirements and publication-state metadata. Extend the checked application lifecycle and existing immutable artifact uploader with native build/upload dependencies, per-platform checksums and conditional GitLab release links. Retain development/pending states until actual artifact and channel acceptance. Wire validation of app/frontend/skill version and digest consistency into existing release CI without relabeling CLI packages as GUI artifacts. Identify native runners and protected signing inputs without storing credentials in the repository.
- Focused validation: Schema tests reject mismatched versions, missing skill digests, mutable or authenticated asset URLs, duplicate identifiers and unsupported architecture claims; verify public origin anonymously and bucket/tap ownership; inspect existing CLI release behavior for regressions; CI configuration validation and release dry run.
- Handoff: Remove ABW-23 only with concrete destinations and release contract recorded; make platform publishing and ABW-31 eligible. Missing external identities/access keep this phase blocked, without speculative credentials or made-up package IDs.

## ABW-24 — Publish signed Windows desktop release artifacts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-13 and ABW-23; authorized native Windows release host, provisioned signing identity and public release access
- Exact write set: `crates/acorn-desktop/tauri.windows.conf.json`, `crates/acorn-desktop/release/windows.ps1` (new), `crates/acorn-desktop/acceptance/windows-release.md` (new), `.gitlab-ci.yml`, `.wiki/features/acorn-browser-workspace/index.md`; external system: ACORN public desktop release origin frozen by ABW-23
- Primary integration lock: Windows desktop release publication
- Outcome: Build/sign/publish a Windows installer for WinGet and a complete signed desktop archive for Scoop, with versioned URLs, SHA-256 sidecars and matching frontend/skill metadata. Preserve the current CLI package and PATH behavior.
- Focused validation: Signature and timestamp verification; anonymous download/hash check; clean installer and archive launch on each declared architecture; WebView2 prerequisites; offline tool/skill and native-dialog smoke; upgrade/rollback/removal and coexistence with installed CLI; complete resource coverage.
- Handoff: Remove ABW-24 after recording public artifact and installed-app evidence; release ABW-28/29. Missing signing, hosts or publication access remain explicit blockers.

## ABW-25 — Publish signed and notarized macOS desktop artifacts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-14 and ABW-23; authorized native macOS hosts for declared architectures, Apple signing/notarization credentials and public release access
- Exact write set: `crates/acorn-desktop/tauri.macos.conf.json`, `crates/acorn-desktop/release/macos.sh` (new), `crates/acorn-desktop/acceptance/macos-release.md` (new), `.gitlab-ci.yml`, `.wiki/features/acorn-browser-workspace/index.md`; external systems: Apple notarization service and ACORN public desktop release origin frozen by ABW-23
- Primary integration lock: macOS desktop release publication
- Outcome: Build/sign/notarize and publish the macOS app in the release contract's DMG/ZIP format with architecture-specific hashes and the matching skill payload. Retain verification evidence and package identity for the cask.
- Focused validation: Code-signature verification, notarization result, stapling where applicable, Gatekeeper assessment and normal downloaded-app launch without bypasses; public hash verification; offline tool/skill/dialog smoke; upgrade/rollback/removal and preservation of unrelated CLI/agent files.
- Handoff: Remove ABW-25 after accepted public macOS artifacts exist; release ABW-30. Apple service waiting and unavailable credentials do not count as active phase time.

## ABW-26 — Publish the Debian desktop package

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-15 and ABW-23; authorized declared Debian baseline/architecture hosts and public release access
- Exact write set: `crates/acorn-desktop/tauri.debian.conf.json` (new), `crates/acorn-desktop/release/debian.sh` (new), `crates/acorn-desktop/acceptance/debian-release.md` (new), `.gitlab-ci.yml`, `.wiki/features/acorn-browser-workspace/index.md`; external system: ACORN public desktop release origin frozen by ABW-23
- Primary integration lock: Debian desktop package publication
- Outcome: Build and publish the Tauri GUI `.deb` and checksum with correct dependencies, architecture, desktop entry/icon, license, package identity and bundled skill, independently of existing CLI Debian packages.
- Focused validation: Package metadata/content inspection and Debian package lint; anonymous fetch/hash check; clean dependency-resolving install on the oldest supported baseline; GUI/offline skill/manual-tool/native-dialog smoke; upgrade, rollback, remove and purge with unrelated CLI/agent files preserved. Record the exact distribution and architecture tested.
- Handoff: Remove ABW-26 after published package and native acceptance evidence are retained; do not claim an apt repository or Debian archive acceptance from a downloadable `.deb`.

## ABW-27 — Publish the portable desktop AppImage

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-15 and ABW-23; native build hosts for declared architectures, oldest supported Linux build baseline, second declared Linux runtime host and public release access
- Exact write set: `crates/acorn-desktop/tauri.appimage.conf.json` (new), `crates/acorn-desktop/release/appimage.sh` (new), `crates/acorn-desktop/acceptance/appimage-release.md` (new), `.gitlab-ci.yml`, `.wiki/features/acorn-browser-workspace/index.md`; external system: ACORN public desktop release origin frozen by ABW-23
- Primary integration lock: desktop AppImage publication
- Outcome: Publish versioned GUI AppImages and checksums containing the accepted frontend, local map resources and matching skill. Document minimum runtime, FUSE/extraction requirements and explicit replacement/rollback/removal behavior.
- Focused validation: Anonymous download/hash check; execute on the oldest declared baseline and a second declared distribution per supported architecture; offline GUI/skill/map/dialog smoke, resource paths and executable permissions; replace with newer and retained prior versions; remove without changing exported skills or the CLI. No inference of universal Linux support.
- Handoff: Remove ABW-27 after public assets and exact tested runtime rows are recorded; retain all unsupported/unavailable rows honestly in release documentation.

## ABW-28 — Publish the desktop app through WinGet

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-24; collision-checked publisher/package ID from ABW-23; authorized submission account and native Windows host; upstream review completion is external waiting
- Exact write set: `crates/acorn-desktop/release/winget/` (new, generated versioned manifest set), `crates/acorn-desktop/acceptance/winget.md` (new), `.wiki/features/acorn-browser-workspace/index.md`; external system: `microsoft/winget-pkgs`
- Primary integration lock: WinGet desktop package publication
- Outcome: Generate validated version/installer/locale manifests against the signed release bytes, submit them to WinGet and establish public installability of the ACORN GUI with correct silent install, upgrade and uninstall metadata.
- Focused validation: `winget validate` on the manifest set; local-manifest installation before submission; after acceptance refresh the public source, find the exact ID/version, install/launch/upgrade/uninstall and verify app/skill version plus preservation of the standalone CLI. Record submission URL, accepted revision and public source result.
- Handoff: Remove ABW-28 only after public-source installation succeeds. A submitted or pending PR remains incomplete and blocked on upstream acceptance; preserve the exact next check without claiming publication.

## ABW-29 — Publish the desktop app through Scoop

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-24; authorized maintainer-owned public bucket frozen by ABW-23; native Windows host
- Exact write set: `crates/acorn-desktop/release/scoop/acorn-desktop.json` (new), `crates/acorn-desktop/acceptance/scoop.md` (new), `.wiki/features/acorn-browser-workspace/index.md`; external system: ACORN public Scoop bucket frozen by ABW-23
- Primary integration lock: Scoop desktop package publication
- Outcome: Publish an architecture-aware desktop manifest for the complete Windows archive, verified SHA-256 values, GUI shortcut, WebView2 prerequisite and pinned update metadata. Keep executable shims distinct from `acorn` and avoid scripts that edit user agent folders.
- Focused validation: Manifest/schema validation and update detection; add the public bucket on a clean Windows account, install, launch, update and uninstall; check published hashes, app/skill version, offline operations, resource paths, shortcut cleanup and preservation of CLI/exported skills.
- Handoff: Remove ABW-29 after public bucket installation and lifecycle evidence are recorded; document bucket add and exact package commands without claiming Scoop main/extras admission.

## ABW-30 — Publish a Homebrew cask for the desktop app

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-25; authorized maintainer-owned public tap and collision-checked cask token frozen by ABW-23; native macOS hosts for declared architectures
- Exact write set: `crates/acorn-desktop/release/homebrew/acorn-desktop.rb` (new), `crates/acorn-desktop/acceptance/homebrew.md` (new), `.wiki/features/acorn-browser-workspace/index.md`; external system: ACORN public Homebrew tap frozen by ABW-23
- Primary integration lock: Homebrew desktop cask publication
- Outcome: Publish a cask with immutable versioned DMG/ZIP URLs, architecture-specific SHA-256 values, app identity, supported macOS floor and update metadata. Preserve the CLI formula and avoid unrequested agent-directory cleanup.
- Focused validation: `brew style` and `brew audit --cask` for the exact token; fresh public tap install, normal Gatekeeper-enabled app launch, offline skill/tool/dialog smoke, upgrade and uninstall on each supported architecture; verify version/hash parity and retained CLI/user-managed skill files.
- Handoff: Remove ABW-30 after public tap installability and lifecycle evidence are recorded; document exact tap/cask commands and distinguish this publication from admission to the Homebrew core cask repository.

## ABW-31 — Present desktop and skill installation channels in the UI

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-10, ABW-22 and ABW-23; accepted Install preview
- Exact write set: `crates/acorn-web/src/pages/install.rs` (new), `crates/acorn-web/src/pages/mod.rs`, `crates/acorn-web/src/pages/skill.rs`, `crates/acorn-web/src/pages/about.rs`, `crates/acorn-web/src/components/sidebar.rs`, `crates/acorn-web/src/route.rs`, `crates/acorn-web/src/tests/install.rs` (new), `crates/acorn-web/assets/app.css`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-install-preview.svg`
- Primary integration lock: shared Install route and release presentation
- Outcome: Implement OKF-style desktop/skill entry cards and a platform/channel table for WinGet, Scoop, Debian, AppImage and Homebrew cask, driven by bundled release metadata. Show exact published versions/architectures, checksum and install/update/removal instructions and pending/unavailable states. The page does not execute installation commands or make background release queries.
- Focused validation: Available/pending/unavailable fixtures for all five channels; exact version/URL/hash rendering; explicit copy actions and safe external links; no fictitious successful install state; offline load, keyboard/focus, dark/light and all three viewports; shared browser/Tauri route parity. Recheck published metadata during ABW-09 after channel acceptance.
- Handoff: Remove ABW-31 after route and fixture checks pass; add Install to ABW-08. Final release acceptance still waits for real channel publication and verified metadata, not UI fixtures.

## ABW-32 — Complete Big-O illustration and acceptance

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-03 shared input/result components
- Exact write set: `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/acceptance.test.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: browser Big-O operation and route
- Outcome: Add the planned labeled n/n² example illustration with distinct line styles and visible sample values. Complete equivalent/incomparable/undetermined, malformed assumptions, maximum-bound and keyboard/error-focus coverage against the working comparison form; preserve existing portable semantics and keep sampled curves separate from symbolic dominance.
- Focused validation: Compare manual and adapter results for left/right/equivalent/incomparable/undetermined cases; reject malformed expressions, invalid assumptions and oversized inputs; keyboard/error focus checks; verify the illustration values and labels, preserve structured results for inputs outside that example, and inspect the accepted SVG and prove no network or native capability is added.
- Handoff: Remove ABW-32 after focused tests pass and record manual/adapter parity; unblock ABW-06 and ABW-07 once their other prerequisites pass.


## ABW-10 — Complete home and chrome release behavior

- Status: in-progress
- Budget: 5 active hours maximum
- Prerequisites: accepted shell
- Exact write set: `Makefile`, `crates/acorn-web/assets/ai-assisted.svg`, `crates/acorn-web/assets/agent-marks.svg`, `crates/acorn-web/scripts/pages-smoke.mjs`, `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/acceptance.test.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`, `crates/acorn-web/build.rs`, `crates/acorn-web/assets/effects.js`, `crates/acorn-web/assets/effects.test.mjs`, `crates/acorn-web/assets/vendor/driver.js`, `crates/acorn-web/assets/vendor/driver.css`, `crates/acorn-web/assets/vendor/driver.LICENSE`, `crates/acorn-web/assets/NOTICE.md`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-workspace-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-about-preview.svg`
- Primary integration lock: browser home and application-chrome behavior
- Outcome: Add system-theme initialization/reset/change handling, locally bundled Driver.js tour with teardown/focus/reduced-motion support, file drop, notification lifecycle, final Home action order and sourced About badges/credits. Preserve the working sample action, quick-start dialog, explicit dark/light toggle and version/WebMCP status until replacements pass acceptance.
- Focused validation: Component/state tests for toggle semantics (`button`, visible label, switch state, `aria-pressed`), both explicit themes, system initialization/reset/change, and banner/token synchronization; Driver.js asset version/license/provenance and offline loading; tour target order, progress, keyboard controls, close labels, dismissal/done reason, teardown, missing target, focus return, and reduced motion; notification replacement/expiry, adjacent Home tour/sample action order and responsive wrapping, absence of a tour control on About, file/drop/sample entry routing, Settings/About links, version derivation, collapsed tooltips, WebMCP status, exact author-only non-About footer text, absence of agent names/marks outside About, absence of any made-with-heart line on About, exact AI Assisted badge asset/hash/link/accessible name, exact AI-disclosure agent labels and URLs, inline-mark alternatives/provenance, safe external-link behavior, responsive wrapping, and proof that the badge loads from the local static bundle without a runtime request; keyboard smoke in desktop and both phone orientations; compare every applicable chrome element with the live OKF reference and both canonical SVGs.
- Remaining acceptance (November 11): local file chooser/drop routing, live system-preference changes/reduced motion, light-tour and narrow About wrapping, offline/network inspection, and final canonical-preview comparison. Native Chrome theme reset, desktop/phone keyboard tour, sample routing and local About assets passed. The native file chooser repeatedly returns its path sheet to `/` instead of opening the selected fixture; the browser tab connector remains unavailable. Do not infer file/drop acceptance from the shared reader implementation.
- Handoff: Remove ABW-10, preserve the accepted chrome behavior and asset provenance in code/README, and mark ABW-03 ready.

## ABW-03 — Complete metadata presentation acceptance

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted shell; ABW-10
- Exact write set: `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/acceptance.test.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: browser metadata operation and shared result contract
- Outcome: Refine the working validation/crosswalk forms and structured result tree into the accepted diagnostic presentation. Complete ordered/truncated/missing-path diagnostics, unsupported combinations, fuzzy policy, conversion-seed and loss-warning coverage, and live result-announcement/error-focus acceptance.
- Focused validation: Tests for valid/invalid/empty/oversized input, fuzzy policy, ordered and truncated diagnostics, missing paths, unsupported format/standard pairs, required crosswalk seeds, loss warnings, and operation errors; focused Rust/WASM checks; live keyboard and result-announcement smoke for the three routes.
- Handoff: Remove ABW-03 and preserve stable shared input/result component behavior for the remaining tool groups.

## ABW-04 — Complete analysis visualization and acceptance

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted shell; ABW-03 shared result contract frozen
- Exact write set: `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/acceptance.test.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: browser analysis pages
- Outcome: Add the planned FAIR proportion bars, metric-specific readability target gauge and diff before/after diagram with numeric equivalents. Preserve the working FAIR/readability/diff operations and complete every metric, 41-finding, non-finite, mode, escaping and boundary case in the focused matrix.
- Focused validation: Tests for every metric, non-finite/error handling, 41 ordered findings, supported FAIR standards, no-resolution notice, equal/changed diff states, invalid JSON Patch inputs, escaped unified output, input limits, optional raw results, chart/result consistency, zero denominators, missing or non-finite scores, unclipped off-scale score labels, non-color equivalents and narrow reflow; focused Rust/WASM checks and live keyboard smoke.
- Handoff: Remove ABW-04 and record any rendering performance threshold needed by ABW-08 without changing portable result semantics.

## ABW-05 — Complete identifier presentation acceptance

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted shell; ABW-03 shared result contract frozen
- Exact write set: `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/acceptance.test.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: browser identifier pages
- Outcome: Refine the working discovery/inspection result tree into the accepted identifier cards with explicit deduplication/count/truncation and ambiguous-candidate presentation. Complete the supported-kind, invalid, mismatch, ordering and 100-result boundary matrix without identifier resolution.
- Focused validation: Representative ARK, arXiv, DOI, Handle, ISBN, ISNI, ORCID, patent, RAiD, ROR, RRID, and SWHID cases; invalid and explicit-kind mismatch; ambiguous ISNI/ORCID candidate ordering; discovery deduplication and 100-result limit; focused Rust/WASM checks and live keyboard smoke.
- Handoff: Remove ABW-05 and confirm all nine home/sidebar entries resolve to complete manual pages before ABW-06/07.

## ABW-06 — Complete browser file and output acceptance

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-03, ABW-04, ABW-05 and ABW-32
- Exact write set: `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/acceptance.test.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`, `crates/acorn-web/assets/effects.js`, `crates/acorn-web/assets/effects.test.mjs`
- Primary integration lock: browser-local presentation-effect boundary
- Outcome: Complete failure and lifecycle acceptance for the implemented local file reader, copy feedback and JSON download. Verify UTF-8/size failure preservation, copy denial, downloaded bytes/name/MIME and object-URL cleanup across supported browsers. Add format-specific converted/diff exports where required by accepted previews; preserve user-mediated local-only effects.
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

## ABW-19 — Add the Research Activity portfolio

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-03 and ABW-16; accepted validated RAD and session-tree state; no concurrent owner of shared route assembly or portfolio state
- Exact write set: `crates/acorn-web/assets/app.css`, `crates/acorn-web/src/state/mod.rs` (new), `crates/acorn-web/src/state/portfolio.rs` (new), `crates/acorn-web/src/components/metric_card.rs` (new), `crates/acorn-web/src/components/activity_table.rs` (new), `crates/acorn-web/src/pages/portfolio.rs`, `crates/acorn-web/src/pages/mod.rs`, `crates/acorn-web/src/route.rs`, `crates/acorn-web/src/components/sidebar.rs`, `crates/acorn-web/src/pages/home.rs`, `crates/acorn-web/src/tests/portfolio.rs` (new), `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-portfolio-preview.svg`
- Primary integration lock: loaded Research Activity session state and Portfolio route
- Outcome: Replace the Portfolio placeholder with a deterministic, read-only view model derived from the same validated Research Activity records already loaded in the session. Present activity, organization, contributor, and persistent-identifier metrics; activity-type/status bar charts and selected-record contributor/organization relationship diagram; text, type, and status filters; a sortable semantic activity table; and selected-record details. Visual bars and summary cards supplement rather than replace text and table values. Preserve source ordering only where semantically required, use stable normalized keys for deduplication and selection, expose empty and single-record states, and add no persistence, network access, dashboard dependency, new domain state, or automatic tool execution.
- Focused validation: Unit tests for empty, single-record, multi-record, duplicate organization/contributor/PID, missing optional fields, mixed activity types/statuses, stable metric and distribution counts, deterministic filter/sort/selection behavior, selected-record removal, and bounded rendering from the 500-file session limit; component tests for semantic headings, table headers and sort state, text equivalents for charts, route title/focus, Home/sidebar entry, and empty-state recovery; keyboard-only selection, 200% zoom, phone portrait/landscape list-first reflow, no horizontal overflow, no raw absolute paths, no external requests, and visual comparison with the canonical SVG.
- Handoff: Remove ABW-19, preserve the accepted portfolio view-model and semantic-visualization contract in tests/README, and make ABW-18 and ABW-08 eligible without duplicating loaded RAD state.

## ABW-18 — Add the local ROR geospatial workspace

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-05 and ABW-19; no concurrent owner of `acorn-web` route assembly, static-asset staging, or JavaScript package manifests
- Exact write set: `crates/acorn-web/geospatial/package.json` (new), `crates/acorn-web/geospatial/package-lock.json` (new), `crates/acorn-web/geospatial/vite.config.ts` (new), `crates/acorn-web/geospatial/src/main.tsx` (new), `crates/acorn-web/geospatial/src/bridge.ts` (new), `crates/acorn-web/geospatial/src/map.tsx` (new), `crates/acorn-web/geospatial/tests/map.test.tsx` (new), `crates/acorn-web/scripts/build-ror-location-index.mjs` (new), `crates/acorn-web/assets/geospatial/ror-locations.json` (new, generated), `crates/acorn-web/assets/geospatial/ROR_PROVENANCE.md` (new), `crates/acorn-web/src/geospatial.rs` (new), `crates/acorn-web/src/pages/geospatial.rs` (new), `crates/acorn-web/src/pages/mod.rs`, `crates/acorn-web/src/route.rs`, `crates/acorn-web/src/components/sidebar.rs`, `crates/acorn-web/src/tests/geospatial.rs` (new), `crates/acorn-web/Dioxus.toml`, `Makefile`, `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-geospatial-preview.svg`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: Resium/CesiumJS island, ROR location dataset, and final static-asset assembly
- Outcome: Add `/geospatial` as a Dioxus-owned RAD workflow with a route-local React 19/Resium/CesiumJS island built by pinned Vite tooling. Fast Refresh for compatible TSX and map-style edits is the primary reason for the Resium/Vite island: `npm run dev` serves a development entry with Vite HMR, while `make web-dev` coordinates it with `dx serve` without forcing a Dioxus/WASM rebuild for every map presentation change. Extract and normalize ROR identifiers from the selected RAD, deduplicate organization markers, join them to a lazy-loaded minimal ROR Schema 2.1 location index, and expose source-reference count, name, ROR link, country, latitude, and longitude. Start Cesium with its locally served `Assets/Textures/NaturalEarthII` base layer and without ion, remote terrain/imagery, geocoder, or network services; bundle its workers, widgets, and Natural Earth II tiles into the same relative-base-safe static output used by GitHub Pages, GitLab Pages, and Tauri. Keep filters, selected details, loading/error state, dataset provenance, unresolved/locationless summaries, and a synchronized sortable table in semantic Dioxus DOM. The bridge accepts only a bounded immutable marker DTO and emits selection/camera/error events; it owns no ACORN parsing, routing, theme persistence, WebMCP, or native capability. If the phase reaches its budget before both the data/table contract and map bridge are accepted, preserve the data/table work and split the island into a new bounded phase rather than weakening tests.
- Focused validation: Pin and audit npm versions/licenses; regenerate the ROR index from the recorded dump version and SHA-256 and compare deterministic output; test valid/duplicate/invalid/unresolved/locationless ROR values, coordinate bounds, stable ordering, source counts, empty RAD, oversized marker sets, lazy-load failure, and escaped labels; React bridge contract tests for mount/update/select/error/unmount and theme changes; start `make web-dev`, verify both servers become ready and stop together, then edit representative map TSX and CSS and observe React Fast Refresh in the existing route without a Dioxus/WASM rebuild or navigation reset; document edits that correctly require a full reload; Cesium asset/base-path inspection for root and nested Pages paths, including successful Natural Earth II `tilemapresource.xml` and tile loads; prove no ion token or external request; WebGL-on and forced-WebGL-off browser smokes; marker/table selection parity; keyboard access to every organization through the table; 200% zoom and phone reflow; Dioxus/WASM build plus Tauri offline smoke; inspect release output for absence of Vite HMR/React Refresh/localhost development artifacts; measure the route's compressed/uncompressed JS, worker, widget, Natural Earth II, and ROR-index sizes and record a reviewed budget before acceptance.
- Handoff: Remove ABW-18, preserve the bridge schema, hot-reload/full-reload boundary, coordinated watcher commands, dataset refresh/provenance procedure, measured asset budget, no-network Cesium configuration, and accessible-table fallback in `crates/acorn-web/README.md`; make ABW-08 and ABW-11 eligible. Any request for ion, remote tiles, geocoding, or a broader React surface requires a separate privacy/security plan.

## ABW-07 — Complete WebMCP workspace lifecycle acceptance

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted browser composition, ABW-03, ABW-04, ABW-05, ABW-10, ABW-32; no concurrent WME-02/WME-04 or other WASM package-staging owner
- Exact write set: `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/assets/`, `crates/acorn-web/tests/`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`, `Makefile`
- Primary integration lock: WebMCP application installation and WASM package staging
- Outcome: Complete fake-context application integration tests and live supported-Chrome discovery/invocation/cleanup for the implemented unchanged nine-tool adapter installation. Verify busy/cancellation/failure/disposal and duplicate-install behavior at the workspace boundary, keeping manual routes available when unsupported.
- Focused validation: `make test-webmcp`; new workspace integration test with fake supported/unsupported contexts; exactly nine registered names and schemas; representative manual/adapter parity for every operation; busy, cancellation, bounded rejection, failure-envelope, duplicate-install, and disposal cases; `make test-webmcp-package`; Dioxus build export inspection; live supported-Chrome registration/discovery/invocation/cleanup when available.
- Handoff: Remove ABW-07 and record the exact supported Chrome/configuration evidence. Missing native support remains an explicit ABW-08 matrix blocker, not a reason to weaken adapter tests.

## ABW-08 — Accept responsive, accessible, and bounded browser behavior

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-03 through ABW-07, ABW-10, ABW-16, ABW-18, ABW-19, ABW-22, ABW-31 and ABW-32
- Exact write set: `crates/acorn-web/assets/app.css`, `crates/acorn-web/src/theme.rs`, `crates/acorn-web/src/components/app_shell.rs`, `crates/acorn-web/src/components/sidebar.rs`, `crates/acorn-web/src/components/tour.rs`, `crates/acorn-web/src/components/theme_toggle.rs`, `crates/acorn-web/src/components/notification.rs`, `crates/acorn-web/src/components/content_input.rs`, `crates/acorn-web/src/components/file_tree.rs`, `crates/acorn-web/src/effects/browser_folder.rs`, `crates/acorn-web/src/components/result_panel.rs`, `crates/acorn-web/src/pages/home.rs`, `crates/acorn-web/src/pages/portfolio.rs`, `crates/acorn-web/src/pages/geospatial.rs`, `crates/acorn-web/src/pages/settings.rs`, `crates/acorn-web/src/pages/install.rs`, `crates/acorn-web/src/pages/skill.rs`, `crates/acorn-web/geospatial/src/map.tsx`, `crates/acorn-web/tests/browser-smoke.mjs` (new), `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-workspace-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-portfolio-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-validation-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-analysis-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-identifiers-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-geospatial-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-settings-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-about-preview.svg`
- Primary integration lock: browser workspace acceptance matrix
- Outcome: Close responsive, keyboard, focus, zoom, reduced-motion, semantic, contrast, performance, WebGL-fallback, and bounded-error defects across the complete route matrix at desktop, phone portrait, and phone landscape without changing operation or geospatial bridge contracts.
- Focused validation: Run the complete manual-tool and geospatial matrix in available Chrome, Firefox, and Safari/WebKit; supported-Chrome WebMCP matrix separately; keyboard-only navigation, organization-table use, and drawer focus return; 200% zoom; reduced motion; automated accessibility/contrast scan; no horizontal overflow at all three viewports; forced WebGL failure; console/network inspection proving no external map/ROR request; maximum-bound timing and interaction responsiveness; rerun focused Rust/WASM, Resium bridge, and WebMCP tests after repairs.
- Handoff: Remove ABW-08 and record dated browser/OS/viewport/assistive-technology evidence plus every unavailable row. Any acceptance defect outside the exact write set becomes a bounded repair phase.

## ABW-11 — Embed the shared Dioxus workspace in a minimal Tauri shell

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted browser composition and shell, ABW-10, and ABW-18; no concurrent workspace-manifest, frontend-development-server, or lockfile owner
- Exact write set: `packaging/applications.json`, `xtask/Cargo.toml`, `xtask/src/application_release/`, `.gitlab-ci.yml`, `Makefile`, `Cargo.toml`, `Cargo.lock`, `crates/acorn-desktop/Cargo.toml` (new), `crates/acorn-desktop/build.rs` (new), `crates/acorn-desktop/tauri.conf.json` (new), `crates/acorn-desktop/capabilities/main.json` (new), `crates/acorn-desktop/icons/` (new, derived from repository ACORN artwork), `crates/acorn-desktop/src/main.rs` (new), `crates/acorn-desktop/README.md` (new), `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-workspace-preview.svg`
- Primary integration lock: workspace manifest and Tauri application shell
- Outcome: Add a flat interface workspace member for Tauri 2 with `version.workspace = true` and `publish = false`; transition its publication inventory from `planned` to `development`, add native CI/build checks and explicitly exclude it from Cargo workspace packaging. Then point `frontendDist` at the accepted static `acorn-web` output, set `devUrl` to the loopback Dioxus development server, and use `beforeDevCommand` to start the accepted coordinated Dioxus/Vite workflow. Open one labeled main window with the repository-derived ACORN icon, inject restrictive production and development Content Security Policies, and prove browser and desktop load the same route/component build. Development policy permits only the documented loopback HMR origins; production permits neither. The shell exposes no custom command or plugin permission yet, imports neither `acorn-lib` nor portable operation crates, and never loads a remote production URL.
- Focused validation: `make check-release-apps`; CI configuration and offline Cargo workspace packaging checks; `cargo test -p acorn-desktop`; `cargo check -p acorn-desktop`; build `acorn-web`, launch `cargo tauri dev` from `crates/acorn-desktop`, verify `beforeDevCommand` starts and stops both frontend watchers, and observe representative Dioxus RSX/CSS and Resium TSX/map-style edits update the existing webview through their respective hot-reload channels; run `cargo tauri build --no-bundle`; inspect the embedded asset manifest, window label, development and production CSPs, enabled capability list, binary provenance, offline launch, route parity, version/runtime display, JS/WASM hash equality with the accepted web build, and absence of HMR clients, localhost URLs, and development-only CSP sources from release output; compare the implementation with the canonical SVG.
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
- Focused validation: Native `cargo tauri build`; clean install/launch/uninstall smoke for the selected local bundle type; complete nine-tool and geospatial representative matrix; keyboard, 200% zoom, theme toggle/system reset, Driver.js tour, notification, icon, footer, offline Resium/Cesium assets and ROR markers/table, forced WebGL fallback, file/folder open/cancel/partial/error and save flows, nested tree selection, symlink exclusion, offline, console, network, capability-negative, and actual WebMCP-support checks; record hashes and artifact paths without committing generated bundles.
- Handoff: Remove ABW-13 and retain the dated Windows evidence in `crates/acorn-desktop/acceptance/windows.md`. Defects outside the write set open a bounded repair phase; a missing authorized host keeps this phase blocked.

## ABW-14 — Accept the local macOS desktop bundle

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-08 and ABW-17; authorized macOS Apple Silicon or x86_64 host with current Tauri prerequisites
- Exact write set: `crates/acorn-desktop/tauri.macos.conf.json` (new), `crates/acorn-desktop/acceptance/macos.md` (new), `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: macOS Tauri bundle acceptance
- Outcome: Produce and launch an unsigned local macOS application bundle, prove shared UI/manual-tool parity and bounded native dialogs under the system webview, and record OS, architecture, WebKit, artifact, size, capability, accessibility, and offline evidence without claiming signing, notarization, or App Store readiness.
- Focused validation: Native `cargo tauri build --bundles app`; launch/quarantine limitation smoke for the local app bundle; complete nine-tool and geospatial representative matrix; keyboard, 200% zoom, theme toggle/system reset, Driver.js tour, notification, icon, footer, offline Resium/Cesium assets and ROR markers/table, forced WebGL fallback, file/folder open/cancel/partial/error and save flows including security-scoped selection, nested tree selection, symlink exclusion, offline, console, network, capability-negative, and actual WebMCP-support checks; record hashes and artifact paths without committing generated bundles.
- Handoff: Remove ABW-14 and retain the dated macOS evidence in `crates/acorn-desktop/acceptance/macos.md`. Defects outside the write set open a bounded repair phase; a missing authorized host keeps this phase blocked.

## ABW-15 — Accept the local Linux desktop bundle

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-08 and ABW-17; authorized supported Linux x86_64 or ARM64 host with current WebKitGTK/Tauri prerequisites
- Exact write set: `crates/acorn-desktop/tauri.linux.conf.json` (new), `crates/acorn-desktop/acceptance/linux.md` (new), `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: Linux Tauri bundle acceptance
- Outcome: Produce and launch one unsigned local Linux bundle, prove shared UI/manual-tool parity and bounded native dialogs under WebKitGTK, and record distribution, architecture, WebKitGTK, artifact, size, capability, accessibility, and offline evidence without claiming repository, package-manager, or store publication.
- Focused validation: Native `cargo tauri build` for the selected local bundle type; clean launch/removal smoke; complete nine-tool and geospatial representative matrix; keyboard, 200% zoom, theme toggle/system reset, Driver.js tour, notification, icon, footer, offline Resium/Cesium assets and ROR markers/table, forced WebGL fallback, file/folder open/cancel/partial/error and save flows, nested tree selection, symlink exclusion, offline, console, network, capability-negative, and actual WebMCP-support checks; record hashes, dynamic-library prerequisites, and artifact paths without committing generated bundles.
- Handoff: Remove ABW-15 and retain the dated Linux evidence in `crates/acorn-desktop/acceptance/linux.md`. Defects outside the write set open a bounded repair phase; a missing authorized host keeps this phase blocked.

## ABW-09 — Publish the browser and desktop workspace workflows and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-03 through ABW-08 and ABW-10 through ABW-32 and ABW-35
- Exact write set: `Makefile`, `crates/acorn-web/README.md`, `crates/acorn-desktop/README.md`, `docs/src/packages/acorn-web.md`, `docs/src/packages/acorn-desktop.md` (new), `docs/src/SUMMARY.md`, `.wiki/features/acorn-browser-workspace/index.md` (delete after acceptance), `.wiki/features/acorn-browser-workspace/_assets/` (delete plan-specific previews after acceptance), `.wiki/features/acorn-browser-workspace/design-review.md` (delete after preserving durable design requirements), `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: browser/desktop build documentation and wiki retirement
- Outcome: Publish copy/paste browser and Tauri development/build workflows, including the coordinated Dioxus/Vite hot-reload loop, supported hot-reload versus full-reload boundaries, deterministic watcher shutdown, local serving and local-bundle prerequisites, static-host base paths, privacy and capability boundaries, Portfolio and supported-tool behavior, WebMCP behavior, geospatial data/provenance/no-network behavior, ROR refresh procedure, browser/native evidence, measured WASM/map/dataset and bundle sizes, published install/update/removal commands, skill compatibility and rollback, signing/notarization status, and brand-token guidance; add stable focused Make targets; run final integration gates; remove the completed plan from every wiki surface.
- Focused validation: Clean focused and release Dioxus builds; `cargo test -p acorn-web`; `cargo check -p acorn-web --target wasm32-unknown-unknown`; run the documented `make web-dev` and `cargo tauri dev` hot-reload smokes and confirm watcher cleanup; `npm ci && npm test && npm run build` from `crates/acorn-web/geospatial`; deterministic ROR-index regeneration/digest check; `cargo test -p acorn-host --features fs`; `cargo test -p acorn-desktop`; `cargo check -p acorn-desktop`; dependency-tree checks for every forbidden edge in the crate-boundary contract; local `cargo tauri build --no-bundle`; inspect retained Windows/macOS/Linux native and all five public-channel evidence; verify source/CLI/browser/desktop/published skill hashes, manifest versions, upgrade/rollback/removal checks and Install availability metadata; `make test-webmcp`; `make test-webmcp-package`; `make build-wasm`; `make lint`; `make test`; `make check-wasm`; documentation build and link check; `git diff --check`; planning-wiki validation; verify generated files remain under ignored build output and no HMR client, React Refresh preamble, loopback URL, development-only CSP source, ion token, unapproved brand asset, signing material, or secret is committed.
- Handoff: Delete the plan and its index/network/lane entries after preserving all durable browser/desktop commands, published channel evidence, skill lifecycle, measured asset and bundle data, privacy/capability boundaries, signing and compatibility requirements, and brand status in product documentation and tests.

## ABW-35 — Harden the browser build and development loop

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: accepted browser build; exclusive browser staging ownership
- Exact write set: `crates/acorn-web/Dioxus.toml`, `crates/acorn-web/package.json`, `crates/acorn-web/package-lock.json`, `crates/acorn-web/scripts/`, `crates/acorn-web/README.md`, `.github/workflows/acorn-web-pages.yml`, `Makefile`, `.wiki/features/acorn-browser-workspace/index.md`; external system: `jhwohlgemuth/acorn` GitHub Pages
- Primary integration lock: browser build, watcher and static staging
- Outcome: Finish representative RSX, CSS and static-asset hot-reload acceptance and live mirrored GitHub Pages/HTTPS acceptance. The optimizer defect is resolved; durable commands, payload sizes and validation are in the browser README.
- Remaining evidence (November 11): native Chrome can render the local release, but observed RSX/CSS/static reload, first mirror deployment and HTTPS enforcement are still unverified; the browser tab connector remains unavailable.
- Focused validation: Preserve the accepted optimized single-payload build; actual browser reload observations for each edit category; successful mirror-triggered deployment, anonymous hosted route refresh and HTTPS enforcement; focused browser checks.
- Handoff: Preserve tool versions, commands, sizes and observed watcher/base-path behavior in the README and remove this phase.
