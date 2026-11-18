---
type: Plan
title: Rust browser and Tauri desktop workspace for ACORN tools
description: Ship a shared Dioxus browser workspace and Tauri desktop app with bounded native ACORN host capabilities, coordinated skill releases, and verified desktop distribution.
difficulty: high
impact: high
package: [workspace, acorn-web, acorn-desktop, acorn-wasm, acorn-host, acorn-service, acorn-analysis, acorn-schema, acorn-cli, acorn-daemon, acorn-server]
tags: [feature, browser, desktop, tauri, rust, wasm, webmcp, dioxus, resium, cesiumjs, geospatial, accessibility, ornl, os:windows, os:macos, os:linux, os:debian, packaging, winget, scoop, appimage, homebrew, skills]
status: in-progress
execution: phased
phase_budget_hours: 5
---

# Goal

Add one Rust-authored ACORN workspace that turns the existing WebAssembly demo into a coherent local product in two delivery modes: a browser application and a local Tauri 2 desktop application. Both modes use the same Dioxus Web route tree, components, state model, ACORN artwork, and the OKF Browser design. Tauri embeds the built static frontend and adds explicit, typed host capabilities for selected files, native exports, local workflows, daemon management, and configured agent communication without forking UI or domain logic.

The application uses the [OKF Plan Browser home](https://www.jasonwohlgemuth.com/okf-plan-browser/home) as its design authority: the same shell proportions, typography hierarchy, spacing, navigation treatment, centered Home loading flow, restrained cards, and responsive behavior, adapted to ACORN content and green branding. The [November 11 design review](design-review.md) records the revised design specifications and accepted normal and exceptional-state specifications. ACORN content and workflows replace all planning-specific concepts.

The first release provides a visual portfolio for loaded Research Activity Data, manual interfaces for Research Activity validation, metadata-standard validation, persistent-identifier discovery and inspection, content diff, FAIR assessment, metadata crosswalk, readability, Big-O comparison, and a geospatial view of referenced organization locations and projects. The Portfolio adapts the research index catalog interaction model at [research-int.ornl.gov](https://research-int.ornl.gov) and consults the public [OKF Plan Browser source](https://github.com/jhwohlgemuth/okf-plan-browser) for bounded interaction and credential-lifecycle patterns without importing its planning-domain model or application shell. A follow-on browser phase loads multiple GitHub/GitLab buckets into one bounded index. The same application explicitly installs the existing read-only WebMCP tools when `document.modelContext` is supported; every manual workflow remains available when WebMCP is absent.

# Design preview

**Updated November 11:** Every one of the 16 sidebar tabs has an individual editable SVG at 1440×900, plus a 32 px sample-state annotation strip. All use the OKF Browser shell and page hierarchy. Settings also has a separate light-theme SVG; responsive, drawer and collapsed-navigation boards supplement the desktop views. These are static specifications; Six exceptional-state previews and the dated live comparison are accepted in the design review.

**November 16 live Portfolio:** [Desktop index](_assets/portfolio-index-desktop-20261116.png) and [390px index](_assets/portfolio-index-mobile-20261116.png) show the release browser's validated sample catalog. These Chromium captures supersede the static Portfolio board for the implemented index layout.

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

- November 16 implementation and live acceptance evidence: `session.rs` retains the 500-file/256-KiB/16-MiB bounds and rechecks persisted files on restore. `main.rs` revalidates restored JSON and holds loading controls until revalidation finishes. `workspace-storage.js` now saves a versioned complete workspace snapshot automatically after a load and restores it until **Delete all data**; a failed later save reports failure and retains the previous snapshot. Credentials remain separate from localStorage. ABW-38 was accepted against the earlier opt-in flow; this November 17 worktree behavior needs ABW-19 reacceptance. ABW-39 established secure-context sessionStorage credentials with masked controls and removal; ABW-40 accepted private GitLab loading with exact provider/host token scope; ABW-41 sends a scoped GitHub PAT only in provider headers, rejects redirects, and preserves anonymous loading. The focused remote loader suite passed 15 tests, `make web-pages` built one 7,000,573-byte WASM payload with nine operation exports, `make web-test-pages` passed, and `workspace-storage-smoke.mjs` passed mocked private GitLab loading plus authenticated/anonymous token lifecycle in Chromium. On the hosted HTTPS origin `https://www.jasonwohlgemuth.com/acorn/`, the user completed a private NSSD load on November 16; Chromium showed 38 files loaded, 40 entries skipped, zero validated activities, and a masked token configured for `code.ornl.gov`. Successful browser-side fetches verify the private request and CORS path. The app displayed its `read_api` requirement before the user entered the token; the raw scope and commit ID are intentionally not exposed in the UI. Source and tests verify revision pinning, host isolation, redirect rejection, redaction, token removal, and exclusion from workspace/history storage. ABW-42/43 catalog projection and navigation are accepted with focused Chromium and 500-file evidence; Cesium project associations and 3D model feasibility remain open.
- November 17 catalog media evidence: ABW-60–62 now accept signature-matched PNG/JPEG/WebP local files and pinned GitHub/GitLab image bytes as canonical data URLs under the stored-byte limit. Rust restore rechecks signatures and excludes images from text tools. The Portfolio and activity overview render mixed SVG/raster slides, retain touch controls and a placeholder for missing references, and restore images from localStorage until **Delete all data**. `make web-test`, `make web-check`, `make web-build`, 38 focused JavaScript tests, focused Rust raster tests, a four-check index parity run, and the Chromium workspace smoke passed locally. That smoke exercised a 500-file index in about 1.8 seconds, token-backed mocked private GitLab raster loading, reload, and token exclusion. The new build has not yet had its hosted private-provider recheck; ABW-19 remains active for that acceptance step.
- November 17 live development check: the already-running `make web-dev` app at `127.0.0.1:8080` loaded and restored the four-project fictional sample; search, Cards/table parity, equal gaps, and image controls worked. A synthetic PNG/JSON pair loaded through the real file picker, validated as one activity, rendered its image and a missing-reference placeholder, and restored after reload without browser errors. Dark Portfolio title links now use a calmer sage green and underline on hover or keyboard focus. The one-record label is singular. A second dev coordinator refused the occupied Vite port as designed.
- November 17 multi-bucket planning baseline: `Workspace::replace` accepts one selection and resets the prior files, validation results, and single `source`; the Home remote-load handler calls it for every successful repository load and labels the action as replacing the session. `workspace-storage.js` serializes that single workspace. No multi-bucket append, per-bucket removal, or aggregate-source identity is implemented. ABW-63/64 add those capabilities after ABW-19's initial single-bucket acceptance; ABW-65 accepts the combined behavior.

- November 12 desktop icon selection: [ACORN – Brown](../../../docs/src/_assets/acorn-app-icon-brown.svg) is the canonical app-icon source, preserved from the selected `docs/book/_assets/acorn-app-icon-brown.svg` preview because `docs/book/` is ignored generated output. The SVG uses the original acorn geometry in `#80350E`, dark details and a `#1D2922` rounded-square background, with a 512×512 viewBox and 112-unit corner radius. Native icon generation and Tauri wiring remain ABW-11 work.

- November 12 parity items 1–5 are implemented: Home/About/chrome and corrected previews, nine tool-specific results, bounded browser file/folder state, Portfolio, local ROR/Resium map, explicit Settings, verified browser Skill and pending Install channels. ABW-10 is accepted and pruned. Other affected phases retain only unfinished acceptance or native/publication integration, with stable IDs. Durable implementation, asset budgets and test commands are in the [browser README](../../../crates/acorn-web/README.md). Chromium 158 parity automation covers 20 checks; native/real-WASM/effect/map tests and the 3,066-test workspace run pass. This is not Safari/Firefox, assistive-technology or Tauri acceptance.

- ABW-21 accepted November 11: deterministic tracked-source skill folder/raw/ZIP/manifest staging is wired into browser releases. Seven packaging tests, skill validation, source/CLI extraction/staged byte parity and `make web-build` passed. The subsequent optimizer repair passes with one 6,259,734-byte payload and nine exports; local reload acceptance remains in ABW-35 and hosted Pages/HTTPS acceptance in ABW-36. Durable bundle commands and metadata are in the browser README.

- Initial browser review: `acorn-web` inherits the workspace package version. `Display` and `EnumIter` own tool presentation/iteration, documentation artwork is staged from its canonical source, and bundled RAD examples cover summary, publication/organization/output and compute-resource fields. ACRE 2.0.0 is a separate downloadable CLI configuration, not a browser validation tool.

- November 11 implementation evidence: `acorn-web` now provides a local nine-operation Dioxus initial browser release with one WASM payload and unchanged `acorn-wasm` computation/adapter ownership. `acorn-desktop` remains planned. [Browser README](../../../crates/acorn-web/README.md) records commands, measured sizes, passing tests, observed Safari/Chrome behavior and unverified release rows. Concurrent design and unrelated implementation edits are preserved. ABW-02 and ABW-10 are accepted; the November 12 implementation and remaining acceptance below supersede the earlier gaps. ABW-35/36 separately own local watcher and hosted acceptance.
- November 16 desktop scope review at the current worktree: `crates/acorn-desktop/` is absent. `crates/acorn-cli/src/cli/mod.rs` exposes host commands for check, doctor, download, export, format, gather, import, infer/ask, link, MCP, report, serve, and SWHID. `FileFormat` includes PDF, PowerPoint, DOCX, BagIt and structured formats; `ReportFormat` includes PDF and PowerPoint. `crates/acorn-lib/Cargo.toml` gates native PDF/PowerPoint and ACP/MCP work; `crates/acorn-server/src/client/mod.rs` has a typed JSON-RPC client; `crates/acorn-daemon/src/install/mod.rs` only has target-path discovery, while [ACD-09–13](../acornd-service.md#acd-09--add-local-cli-daemon-management) still own daemon control and OS service registration. The [native parity inventory](../../../tests/fixtures/routing/parity.json) covers 72 CLI behavior rows, 46 with bounded native bindings and 26 explicit exclusions; desktop support is not established by that CLI evidence. ABW-45 freezes the desktop-facing host capability matrix against those sources before implementation.
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
- CesiumJS ships Natural Earth II local imagery, and ACORN bundles local map assets. Current browser behavior also offers opt-in live imagery/buildings and an in-memory ion token in the map widget, as recorded in the [browser README](../../../crates/acorn-web/README.md). Preserve explicit opt-in for remote requests and never embed/persist the ion token by default: [CesiumJS imagery](https://cesium.com/learn/cesiumjs-learn/cesiumjs-imagery/), [Ion reference](https://cesium.com/learn/cesiumjs/ref-doc/Ion.html), and [offline guide](https://github.com/CesiumGS/cesium/blob/main/Documentation/OfflineGuide/README.md).
- ROR Schema 2.1 records latitude and longitude under `locations[].geonames_details`, and the versioned ROR data dump is CC0. A pinned, generated location-only subset can resolve ROR identifiers without a runtime registry call, provided the build records the dump version, digest, license, generation command, and output size: [ROR data structure](https://ror.readme.io/docs/ror-data-structure) and [data dump](https://ror.readme.io/docs/data-dump).
- Rune is a pre-release, revision-pinned native GPU-rendered Rust UI framework with its own renderer, text, layout, state, animation, platform shells, and headless agent-driven rendering. Those capabilities overlap rather than complement Dioxus/Tauri and do not solve Resium's React requirement. Rune is therefore not added to this plan; its headless PNG/element-tree testing model remains a useful idea for a later bounded spike if the selected stack fails measured performance or testability gates: [Rune README](https://github.com/anomalyco/rune) and [Rune plan](https://github.com/anomalyco/rune/blob/dev/PLAN.md).
- Tauri 2 is frontend-agnostic and can embed a static HTML/CSS/JavaScript/WASM directory through `frontendDist`, so the built Dioxus Web output can be reused instead of creating a second renderer: [Tauri frontend configuration](https://v2.tauri.app/start/frontend/) and [Tauri configuration](https://v2.tauri.app/reference/config/). Its capability files constrain which windows may invoke commands, and production builds generate platform-specific bundles: [Tauri capabilities](https://v2.tauri.app/security/capabilities/) and [Tauri distribution](https://v2.tauri.app/distribute/).
- ORNL publicly identifies green as its signature color, but the public page does not publish a machine-readable ACORN token: [OLCF media assets](https://www.olcf.ornl.gov/about-olcf/media-assets/). The initial theme therefore uses `#00662C` only as a provisional ORNL-inspired accent unless an approved ORNL source confirms a production value. No ORNL logo or implied official endorsement is in scope.

- The November 11 [design review](design-review.md) inspected all eight rendered ACORN boards and the live OKF Home, About, Install and Planning Skill pages. The reference now makes skill discovery and app/skill acquisition prominent. The revised SVGs now include those routes. A follow-up live comparison also inspected loaded Portfolio, Settings in both themes, 390×844 portrait and 844×390 landscape. Full exceptional-state and interaction acceptance remains separate.
- `crates/acorn-cli/assets/skills/acorn/SKILL.md` is the existing canonical authored ACORN skill. `crates/acorn-cli/src/commands/skill/mod.rs` embeds it and extracts it to the platform cache; [CLI documentation](../../../docs/src/cli-reference/commands/skill.md) explains this behavior. The canonical skill bundle and compatibility manifest are now staged by the browser build; Browser acquisition is implemented and verified; desktop delivery and publication remain pending.
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
- Platform effects behind one typed UI boundary: native browser file/folder selection and download in browser builds; user-mediated Tauri file/folder open, save, export, and explicitly configured host operations in desktop builds. A loaded folder becomes a deterministic tree of supported text and image files, automatically saved as a bounded localStorage snapshot until explicit deletion. Selecting a text leaf supplies that file to a manual tool without running it; selecting an image opens a preview. The desktop backend traverses only the root returned by its native folder dialog inside a bounded Rust command, so the webview receives sanitized relative paths and content but never a reusable filesystem capability, absolute path, persisted directory grant, or ambient home-directory access.
- Structured results before optional raw JSON: ordered diagnostics, FAIR profiles/findings, readability checks, conversion warnings, unified diff or JSON Patch, and normalized identifier states.
- A `/portfolio` route that derives a deterministic view of the Research Activity Data in the current or automatically restored workspace: activity, organization, contributor, and persistent-identifier metrics; activity-type distribution; text and status/type filters; a sortable semantic activity catalog; selected-record details; and visible saved-data deletion and failure status. The visual summary never becomes the sole representation and invents no new domain state.
- Portfolio catalog behavior that lets users explore a research index of loaded projects/activities, with searchable and filterable records, useful catalog summaries, and record-to-organization detail navigation. Use the accessible information hierarchy and interaction flow of the ORNL Research Index Catalog, while honoring its terms, linking it as inspiration, and keeping ACORN-specific loaded-data boundaries.
- Automatic localStorage persistence for loaded project/activity data, associated images, and the selected workspace state across reloads until **Delete all data**. Keep schema/version handling, quota/error feedback, bounded encoded size, credential exclusion, and a visible deletion control. Preserve the prior saved snapshot after a failed replacement save.
- Optional GitHub and GitLab personal access token entry for repository loading, with provider-specific scopes kept minimal, explicit user consent, masked fields, validation/error feedback, and save/delete controls. Browser saves are restricted to provider/host-scoped sessionStorage so they survive a same-tab reload but end with the tab session; PATs never enter localStorage, URLs, logs, history, errors, or redirects. Desktop host operations use a separate protected, user-controlled credential store after ABW-48; a token reaches only its selected provider request, never a generic tool or agent context.
- A `/geospatial` route that extracts ROR identifiers from loaded Research Activity Data, resolves them against a lazy-loaded, versioned ROR 2.1 location subset, and plots deduplicated organizations and associated projects on the Cesium globe. Dioxus owns route state, filters, selection details, the accessible organization/project table, empty/error/WebGL-unavailable states, and all ACORN data handling; a route-local React/Resium island owns only the Cesium viewer and marker/model lifecycle. During development, that island is served by Vite with React Fast Refresh while the surrounding Dioxus application retains its own hot-reload loop; release builds replace both development clients with one static asset graph.
- A semantic DOM, keyboard navigation, visible focus, route-title/focus management, restrained live regions, reduced motion, 200% zoom, and desktop/phone portrait/phone landscape acceptance.
- An ORNL-inspired green accent system with accessible solid, subtle, hover, selected, and focus tokens. The production documentation must distinguish an approved brand value from a provisional inspired value.

## Out of scope

- Reimplementing ACORN rules, codecs, crosswalks, identifier logic, FAIR checks, readability, or diff behavior in UI code.
- Replacing `webmcp.js`, its contract tests, the existing native smoke, or the independent WebMCP evaluation plan.
- Unprompted filesystem access, traversal outside an explicitly selected root, symlink following, automatic directory watching or daemon installation, arbitrary shell/process execution, direct webview host APIs, unattended inference, a required server for ordinary tools, or remote persistence. Desktop-only PDF/PPTX export, diagnostics, selected repository/model work, daemon controls, and configured agent/protocol connections are explicit opt-in phases below.
- A TypeScript/React application shell, server-side rendering, a separate desktop UI, a Freya or Rune production renderer, or a generic dynamic-form framework. The bounded Resium map island is the only React-owned surface.
- Automatic ROR API calls, user tracking, or making the WebGL globe the only way to inspect organization/project locations. Cesium ion credentials and remote imagery/terrain retain their separate explicit opt-in controls.
- In-app auto-update, background startup, tray integration, global shortcuts, deep links, remote content, mobile Tauri targets, Microsoft Store/App Store publication, and admission to the Debian archive. Production signing/notarization and the five requested distribution channels are in scope; credentials and authorized release hosts are external prerequisites. Unsigned local bundles are development evidence only. Package-manager updates do not require an updater plugin.
- Automatic clipboard writes, automatic downloads, tool execution on page load, telemetry, or unconsented ACORN-originated network requests.
- Copying the OKF Plan Browser application wholesale, its planning content, or its product name. Its public source may be consulted for the bounded token lifecycle and Portfolio interaction patterns, while ACORN retains its Rust/Dioxus shell, domain model, and styling ownership.
- Using an ORNL logo, DOE mark, trademarked asset, or claim of official branding without an approved repository asset and authorization.

## Product and architecture decisions

1. **Use Dioxus Web and the DOM.** The product requires native inputs, semantic navigation, browser file controls, text-heavy results, and screen-reader evidence. Canvas-first rendering is not the first-release accessibility risk profile.
2. **Keep one UI composition root.** `acorn-web` owns routes, presentation state, and a typed platform-effect interface; portable ACORN crates own computation; `acorn-wasm` owns JavaScript DTO conversion and WebMCP bindings. `acorn-desktop` embeds the compiled frontend and implements only the native side of approved effects. Browser, webview, and Tauri types do not cross into portable crates.
3. **Prove one WASM payload before page work.** Link the existing `acorn-wasm` rlib into the Dioxus module and verify the generated JavaScript exports all nine bindings. If a future build cannot preserve those exports, a bounded repair phase must decide the packaging change; do not ship two unmeasured copies.
4. **Manual controls are the baseline.** WebMCP status is `initializing`, `installed`, `unsupported`, `failed`, or `disposed`; unsupported WebMCP never disables the manual tools.
5. **Use structured Rust presentation state.** Distinguish idle, running, valid/success, invalid input, bounded rejection, operation failure, and unsupported capability. Invalid metadata is not an application crash.
6. **Retain loaded workspaces until deletion.** Save loaded data and workspace selection automatically to localStorage, display stored size and save failures, bound encoded data, handle schema upgrades/quota failures, and offer **Delete all data**. Repository suggestions already use localStorage. Newly saved browser PATs use a separate provider/host-scoped sessionStorage record only after explicit save action, are masked in the UI, and have visible deletion controls; legacy project-bound tab entries stay narrow until replaced. They never enter the loaded-data localStorage record.
7. **Use a provisional green honestly.** Start with `#00662C` as `--ornl-green` only when the UI and documentation call it ORNL-inspired. Replace or confirm it from an approved brand source before claiming official ORNL branding. Derive accessible light/dark variants rather than using the base green for every foreground.
8. **Do not treat blue or purple as the primary accent.** Informational states may use a distinct accessible color, but actions, active navigation, focus treatment, and brand highlights use the green token family.
9. **Carry over applicable OKF chrome deliberately.** Reuse the interaction ideas—collapse/drawer, icon labels and tooltips, tour, theme choices, version, Settings/About, notifications, local open/drop/sample entry points, runtime status, and made-with-heart credit. Shape Portfolio catalog behavior after the linked ORNL research index and OKF Plan Browser reference, and support explicitly authenticated GitHub/GitLab loads within the bounded repository loader.
10. **Keep the desktop authority narrow and visible.** The main Tauri window loads bundled assets and uses an explicit Content Security Policy. ABW-12/17 first add only open-text, open-folder, and save-text commands; later desktop phases add separately allowlisted commands for native exports, host operations, daemon control, and agent connections. Every effect has typed inputs, explicit user action, bounded output, and a named native owner. The webview receives no generic filesystem, shell, process, updater, persisted-scope, or remote-URL capability. HTML file drops remain content-only; do not enable path-wide scope as a shortcut.
11. **Do not emulate WebMCP with privileged IPC.** The desktop app reports the actual `document.modelContext` result from its system webview. Manual tools remain the baseline when it is unsupported. Tauri commands never masquerade as WebMCP tools, and WebMCP invocations never inherit native file authority. Outbound ACP/MCP/inference sessions are separate, explicitly configured desktop actions; the assisted RAD plan owns its own consented conversational experience.
12. **Make the chrome theme control a toggle.** The header uses one native button with `aria-pressed` to express explicit dark/light state and a visible switch track plus label. It initializes from the system preference when no explicit session choice exists; activating it selects the opposite explicit theme. Settings may return to system preference, after which the toggle reflects the resolved theme without misrepresenting persistence.
13. **Use Driver.js for the tour.** Bundle pinned Driver.js 1.10.2 JavaScript, CSS, license, and provenance with the static application; do not load it from a CDN. A thin application adapter owns stable target IDs, ordered steps, progress, keyboard controls, close/done handling, focus return, teardown, theme tokens, and reduced-motion behavior. Driver.js never owns routes, product state, analytics, or persistence.
14. **Treat a folder as bounded supplied content.** Browser and desktop folder pickers may recursively enumerate only the selected root. Keep at most 500 supported regular files, 20 relative-path segments, 256 KiB per loaded file, and 16 MiB total decoded UTF-8 content; accept `.json`, `.yaml`, `.yml`, `.md`, and `.txt`; sort normalized relative paths lexicographically; never follow symlinks; and report skipped, rejected, and truncated entries. Operation-specific bounds still apply when a leaf is sent to a tool.
15. **Isolate Resium as a hot-reloadable map island.** The primary reason for Resium is a productive geospatial development loop: a small Vite-built React module mounts only inside the `/geospatial` viewer host and receives React Fast Refresh/HMR without rebuilding the Dioxus WASM application for compatible TSX or map-style edits. Dioxus passes an immutable, bounded marker DTO and receives selection/camera/error events through one versioned JavaScript bridge. The island does not parse RAD, resolve ROR, own application navigation, duplicate theme state, call ACORN operations, register WebMCP tools, or import Tauri APIs. Rust, bridge-schema, package, and Vite-configuration changes may trigger their normal rebuild or full reload; the plan does not claim that every edit preserves state.
16. **Keep geospatial resolution local and inspectable.** Generate a minimal ROR-id/display-name/latitude/longitude/country index from a pinned Schema 2.1 data dump, record its digest and CC0 provenance, and lazy-load it only on `/geospatial`. Deduplicate by normalized ROR ID, preserve every source reference count, report unresolved and locationless identifiers, and provide the same ordered organization table when WebGL is unavailable. Cesium starts with its bundled Natural Earth II TMS imagery and without ion, geocoder, remote imagery, or terrain.
17. **Do not add Rune to the first-release stack.** Rune's renderer, layout, state, and platform shells compete with Dioxus/Tauri rather than supplying missing components. Reconsider it only in a separate time-boxed spike with explicit parity, accessibility, Web/desktop, Resium integration, binary-size, and headless-test gates after a measured failure of the selected architecture; borrowing its headless-testing concepts does not require a dependency.
18. **Assess organization models before adopting them.** A bounded feasibility phase must identify authoritative 3D organization assets or a viable lawful creation path, attribution/license terms, Cesium glTF/glb compatibility, coordinate/orientation/scale metadata, download/packaging size, rendering cost, and a 2.5D camera/selection prototype. Recommend a go/no-go and fallback marker/footprint representation; do not bundle or fetch models until that evidence supports a separate implementation phase.
19. **Use a fixed native companion contract.** ABW-45 inventories the CLI/service host surface and packages a version-matched `acorn` companion for allowlisted operations that still live in `acorn-lib`. The desktop backend owns typed requests and result limits; `acorn-host` owns fixed executable invocation and cancellation. No user-supplied command, executable path, argument vector, or ambient directory crosses IPC. A missing typed result contract is an extraction task in the owning crate, not permission for the webview to run the CLI. The desktop bundle verifies companion version and digest at startup and remains usable for browser-equivalent tools when a host capability is unavailable.

## Crate-boundary contract

| Package | Role in this plan | Forbidden ownership |
|---|---|---|
| `acorn-core`, `acorn-schema`, `acorn-analysis`, `acorn-diff`, `acorn-service` | Existing portable values, codecs, operations, findings, and orchestration consumed directly by first-party interfaces | DOM, Tauri, filesystem, WebMCP registration, presentation state, or duplicated UI-specific result models |
| `acorn-wasm` | Reusable `wasm-bindgen` ABI, JavaScript DTO conversion, and the existing nine-tool WebMCP adapter | Dioxus routes/components, CSS, Tauri commands, native host effects, or a dependency on `acorn-web` |
| `acorn-web` | Static Dioxus interface composition root: routes, components, presentation state, accessibility, ephemeral DOM effects, and the narrow bridge to a route-local Resium map island | Portable domain policy, duplicate JS DTO/adapter logic, a React application shell, ACORN logic inside the map island, native filesystem/process dependencies, `acorn-host`, or `acorn-lib` |
| `acorn-host` | Native filesystem owner for selected paths and, after ABW-45, fixed allowlisted companion execution with cancellation and resource limits | Tauri window/dialog types, Dioxus state, product routing, or WebMCP policy |
| `acorn-desktop` | Tauri composition root that embeds `acorn-web`, authorizes typed IPC, selects native files/destinations, checks companion provenance, and delegates filesystem/process effects to `acorn-host` | Reimplemented ACORN operations, direct `std::fs` or generic process policy, a second UI tree, reusable path authority, or privileged WebMCP bridging |
| `acorn-cli`, `acorn-daemon`, `acorn-server` | Existing host operation, daemon lifecycle, and typed service-client contracts consumed through narrowly packaged native adapters | Desktop-only forks of command behavior or a browser dependency on native crates |
| `acorn-lib` | No direct role in the new browser or desktop dependency graphs; existing CLI companion may continue to depend on it | Compatibility re-exports or direct desktop host shortcuts that conceal the narrow owning crates |

`acorn-web` is a new interface root, not a replacement browser-adapter owner. Its DOM effects are presentation mechanics over user-supplied in-memory values; reusable JavaScript ABI and WebMCP behavior remain in `acorn-wasm`. If a required operation is absent from the portable owners, the implementing phase stops and opens a bounded extraction/repair phase under the crate-boundary architecture instead of adding the operation to either UI crate.

## Desktop host capability inventory

ABW-45 freezes one versioned, testable matrix against `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/cli/arguments/mod.rs`, the 72-row native parity fixture, and the daemon/agent plans. These rows are the minimum desktop scope; new host-only CLI capabilities require a matrix decision and a bounded implementation phase before ABW-56 can pass. A command-line flag does not require a separate GUI control when a typed workflow carries the same user intent and result. Browser-only delivery remains limited to the existing portable operations.

| Host capability family | Desktop phase | Required result |
|---|---|---|
| Selected local content, folders, and save | ABW-12/17 | Bounded dialogs and normalized content without reusable path authority |
| PDF export and report rendering | ABW-46 | Existing ACORN PDF output from an explicit selected source/destination, with Chromium availability shown honestly |
| PowerPoint and remaining export/report formats | ABW-47 | Existing PPTX and applicable structured/DOCX/BagIt outputs with format-specific validation |
| Local diagnostics and checks | ABW-49 | Explicit check and doctor output with bounded results and no automatic fix |
| Local creation and mutation | ABW-50 | Create, format, link, SWHID and reviewed check fixes with source freshness and preview before write |
| Repository, gather, import, and intake | ABW-51 | Consent-bound network/source selection, bounded download and clear offline/unavailable states |
| Model assets and native prerequisites | ABW-52 | Explicit model, Needle, Chromium and Vale acquisition plus import/sync, verified assets, quotas and cleanup |
| `acornd` lifecycle and job control | ABW-53 | Local status/history/run/pause/resume/reload plus supported install/start/stop/uninstall after ACD ownership lands |
| Configured ACP, MCP, and endpoint inference | ABW-54 | Explicit outbound connection/session controls, bounded turns/results, cancellation, no implicit tool or filesystem authority |
| Inbound ACORN protocol servers | ABW-55 | Explicit start/stop/status for supported `serve` modes without unsolicited listening or public bind |
| Constrained and trusted Rhai extensions | ABW-57 | Explicit static check, manifest/hook validation, bounded script execution, and separately consented trusted automation |
| Native integration provisioning | ABW-58 | Reviewed `create mcp/runner/bot/secret` output with deployment and clipboard effects disclosed before execution |
| Report publication | ABW-59 | Preview and explicitly publish bounded report artifacts to a named authorized GitLab target |
| Native credentials and configuration | ABW-48 | Protected user-controlled secrets, scoped provider/endpoint use, redaction and deletion |
| Terminal-only TUI | ABW-45/56 | Record that the shared Dioxus UI is the desktop interaction equivalent; no nested terminal emulator or hidden command invocation |
| Complete host matrix and native bundles | ABW-56 then ABW-13/14/15 | Every row implemented or explicitly unavailable for a verified platform prerequisite; no silent omission |

# Routes and first-release behavior

| Route | Manual workflow | Required presentation |
|---|---|---|
| `/home` | Choose the Portfolio, a tool, geospatial view, local file, recursively loaded local folder, dropped content, or bundled sample and inspect runtime state | ACORN artwork, adjacent **Take a tour** and **Try sample data** actions in that order, prominent ACORN Skill card, Install/Skill navigation, supporting privacy copy, combined file-or-folder loader, retained drop zone, session file tree, Portfolio entry, nine icon-labeled WebMCP tools plus geospatial navigation, theme toggle button, notification region, WebMCP status, version, and author-only made-with-heart credit |
| `/portfolio` | Explore and filter the saved or current-session Research Activity Data as a research index catalog | Research-index-inspired project/activity catalog; deterministic activity, organization, contributor, and PID metrics; activity-type/status bar charts and selected-record relationship diagram with equivalent text/table values; type/status/text filters; sortable semantic catalog; selected-record details; empty and single-record states; narrow-screen list-first reflow; automatic restore and **Delete all data** controls |
| `/validate/activity` | Validate Research Activity JSON and optional local file | Validity, effective fuzzy policy, ordered/truncated diagnostics |
| `/validate/standard` | Validate supplied content/format/standard | Validity, supported-combination errors, ordered diagnostics |
| `/crosswalk` | Run a supported typed crosswalk | Converted content, output format, mandatory loss warnings, copy/download |
| `/fair` | Assess supported JSON metadata | 41 findings, profile and per-principle proportion bars with counts, no-resolution notice |
| `/readability` | Run one supported metric | Score, metric-specific target gauge, threshold, acceptable/excessive state, returned summary |
| `/big-o` | Compare two Big-O expressions with explicit symbol assumptions | Normalized expressions, assumptions, left/right/equivalent/incomparable/undetermined dominance, structured invalid-input feedback, and optional raw result; 256 UTF-8 bytes per expression/assumption name and at most 64 assumptions |
| `/diff` | Compare before/after content | Unified or RFC 6902 result, before/after diagram and mode-appropriate change counts, changed state, copy/download |
| `/identifiers/discover` | Discover identifiers in text | Ordered normalized identifiers, count, truncation |
| `/identifiers/inspect` | Inspect one identifier and optional kind | Valid, invalid, or ambiguous state and deterministic candidates |
| `/geospatial` | Map organizations and projects referenced by loaded Research Activity Data | Resium/CesiumJS globe, project-aware deduplicated locations, unresolved/locationless summary, selected-organization/project details, synchronized accessible table, WebGL fallback, and ROR dataset version/provenance; optional organization models only after feasibility gate |
| `/install` | Acquire or update the desktop app and matching skill | OKF-style app/skill entry cards, five-channel platform table, exact supported architectures, versioned artifact links/checksums, explicit publication status, upgrade/removal instructions, and a link to `/skill` |
| `/skill` | Read, copy or download the release-matched ACORN skill | Canonical source link, app/skill version and digest, workflow guide, raw definition, complete folder download, compatibility and manual install/update/removal instructions; works offline from bundled content |
| `/settings` | Configure appearance, saved workspace data, and repository credentials | Dark, light, and system choices; current version/runtime; automatic bounded workspace save/restore status and **Delete all data**; masked GitHub/GitLab token save/delete controls with provider, host, scope, and tab-only disclosure |
| `/about` | Explain boundaries, authorship, AI use, and implementation technology | Narrow OKF-inspired content column with source card and no tour control, project narrative, local-first and delivery-target cards, three-column technology grid with labeled marks and no duplicate mark strip, version/runtime and WebMCP boundaries, bordered AI-use disclosure with the linked AI Assisted badge plus linked agent names and marks, and no made-with-heart footer |

# Dependencies and coordination

- [Frontend build quality](../../testing/frontend-build-quality.md) owns CSS/JS/TS analysis and verified minification for the shared frontend. Its FBQ-01–06 phases serialize overlapping assets, adapter staging and Make/CI files with ABW-07/18/35; ABW retains product/performance acceptance. ABW-11 must consume the accepted build contract when embedding the frontend, without creating a separate lint/minification pipeline.

- The [crate-boundary architecture](../../refactors/crate-boundary-architecture.md) is an upstream coordination dependency, and this plan is its concrete atomic browser/desktop consumer. That record remains authoritative for ownership direction; this plan owns the bounded implementation phases. The [portability coordination record](../../refactors/portability/index.md) supplies the existing WASM/feature gates records the accepted single-payload `acorn-web` composition.
- ABW-20 is accepted with six rendered exceptional-state previews and a dated 1440×900 live comparison; the shared shell, Install/Skill pages and nine-tool inventory are already reflected in the revised SVGs. The browser composition is accepted; exceptional-state design does not block local review. Future manifest/lockfile edits still require exclusive ownership.
- The branded shell and Home/chrome contract are accepted. ABW-03 retains metadata acceptance. ABW-04 and ABW-05 currently share `src/ui.rs` and `src/operations.rs`, so they serialize. Parallel leaf work requires an explicit module extraction and named disjoint write sets first.
- ABW-06 follows the tool pages because single-file/copy/download behavior must be accepted against real result types, not speculative abstractions. ABW-16 accepts the broader cross-engine folder matrix independently of the catalog's bounded folder contract. ABW-39 provides the independent credential boundary, ABW-41 accepts scoped GitHub loading, and ABW-38 supplied the separate full workspace snapshot. ABW-42/43 index projection and catalog navigation are accepted. ABW-40 private GitLab loading and CORS were accepted on the authorized private NSSD project and hosted HTTPS origin on November 16. ABW-60–62 bounded image intake and presentation were accepted with local and mocked private-provider browser checks on November 17. ABW-19 now owns initial single-bucket catalog acceptance and the hosted recheck of the new build; ABW-63 adds multi-bucket state/persistence, ABW-64 adds the load and catalog controls, and ABW-65 accepts the integrated browser behavior. ABW-03/16 are separate tool/folder cross-engine matrices.
- ABW-44 follows ABW-65 to project all retained buckets onto the existing organization map. ABW-18 follows ABW-44 for maximum project/marker resource and performance acceptance. These phases exclusively own project-aware changes to the React/Resium/CesiumJS island and its Dioxus bridge; no other phase may change those surfaces concurrently. ABW-37 follows ABW-18 and is assessment-only; it does not block marker-based geospatial acceptance.
- ABW-07 completes workspace integration acceptance after all tool groups. It serializes WebMCP staging and `Makefile` changes with WME-02, WME-04, and any other owner of `webmcp.js` or WASM package assembly. It consumes the adapter unchanged unless a separate evidence-backed repair expands its write set.
- ABW-08 requires the complete manual and geospatial route matrix, including the map island's static assets, WebGL fallback, and accessible table. Live WebMCP registration requires a supported Chrome build and flag/configuration; missing support blocks only that matrix row, not manual browser acceptance.
- ABW-11 follows ABW-18 as well as the frozen browser build seam and shell, then claims the workspace manifest/lockfile alone to add the Tauri shell and connect `cargo tauri dev` to the accepted coordinated hot-reload workflow. ABW-12 follows the accepted browser effects, serializes the `acorn-host` filesystem leaf, and adds bounded native single-file open/save commands without a general filesystem permission or direct filesystem implementation in `acorn-desktop`. ABW-17 then adds one selected-folder command against the ABW-16 tree contract and the same `acorn-host` ownership boundary.
- ABW-45 follows ABW-17 and freezes the complete desktop host matrix and version-matched fixed companion contract. ABW-46/47 own PDF then PowerPoint/other export flows; ABW-48 owns protected credentials; ABW-49/50 own local diagnostics and reviewed local mutation; ABW-51/52 own repository, model and native-prerequisite operations. Their shared host command, Tauri capability, and Dioxus host-view files serialize. ABW-53 consumes ACD-09 plus the relevant ACD-10/11/13 service-manager contracts without implementing the daemon again. ABW-54 consumes the landed configured MCP/ACP/endpoint adapters; ABW-55 adds separately consented inbound server control. ABW-57/58/59 cover trusted extensions, native integration provisioning and report publication. The [assisted RAD plan](../assisted-rad-workspace.md) retains the distinct conversational review experience and does not substitute for host transport acceptance. ABW-56 checks every matrix row and packaged companion before native OS acceptance.
- ABW-13, ABW-14, and ABW-15 validate local bundles on Windows, macOS, and Linux respectively after ABW-56 and ABW-08. Their platform-specific files are disjoint and may proceed in parallel on authorized native hosts; each unavailable host remains unverified rather than passing by inference.
- The canonical skill bundle is accepted against the browser composition; ABW-22 exposes it after the shared chrome and browser/native file effects exist. ABW-23 freezes release identities and manifests after ABW-45 has specified the companion and the accepted skill bundle; ABW-31 consumes that contract for Install UI before ABW-08. ABW-24/25 produce signed Windows/macOS releases after their native acceptance; ABW-26/27 publish separate Debian/AppImage artifacts after Linux acceptance. ABW-28/29 consume the Windows release for WinGet/Scoop; ABW-30 consumes the notarized macOS release for Homebrew cask. Missing hosts, signing, public URLs or channel ownership block only the dependent publication phases.
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
cargo test -p acorn-host --features process
cargo test -p acorn-desktop
cargo test -p acorn-daemon
cargo test -p acorn-server
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
- Capability inspection proves the bundled main window only loads local assets and cannot invoke generic filesystem, shell, process, updater, or remote-content APIs. Each custom command is enumerated in the frozen matrix, bound to the main window, and rejects forged paths/identities; the folder result exposes only normalized relative paths beneath the dialog-selected root.
- PDF, PowerPoint, structured/DOCX/BagIt export and report flows use an explicit selected source and destination, retain CLI-equivalent bytes or documented format semantics, and show missing Chromium, template, companion or platform prerequisites instead of silently dropping a format.
- Local checks, doctor, creation/format/link/SWHID, repository/gather/import/intake, and model setup each have a visible start, cancel or unsupported state and a bounded outcome. Mutations show a preview and require confirmation; remote calls show provider, host, scope and data before consent.
- `acornd` shows authenticated local health/status/history and supported job/lifecycle controls only after its CLI and OS service-manager contracts land. ACP/MCP/endpoint sessions and inbound ACORN servers are separately opt-in, bounded, cancellable, redacted and offline-aware; neither receives selected files or native effects by default.
- Installed bundles verify the pinned companion and skill versions/digests, work for portable manual tools offline, and never require an existing system `acorn` on PATH. Platform-specific protected-store and service prerequisites are recorded as unverified until exercised on their native host.
- WebMCP status reflects the embedded webview's real support. Unsupported is an accepted desktop state; no native IPC shim or elevated tool registration is present.
- Platform-specific packaging checks run on their native hosts. Unsigned local acceptance does not satisfy publication. Production releases need the channel-specific signing/notarization, immutable public downloads, manifest validation and installed-app evidence defined below.

# Acceptance criteria

- ABW-20 closes the [design-review findings](design-review.md): all route previews match the live OKF shell/layout at equal viewports with documented ACORN adaptations, Install and Skill are included, and remaining exceptional-state behavior is accepted before full release.
- The canonical ACORN skill is bundled from the same release revision into CLI, browser and desktop artifacts. Build checks prove byte/hash and version compatibility; install, upgrade, rollback and removal follow the lifecycle contract below without silently changing user-managed agent folders.
- The desktop GUI is published and installable through WinGet, Scoop, a Debian `.deb`, an AppImage, and a Homebrew cask. Each channel records public artifact/manifest URLs, exact version/architecture/checksums and native install/launch/update/removal evidence. A local bundle, pending upstream PR, or inaccessible URL does not satisfy publication.

- `crates/acorn-web` is the documented Rust/Dioxus composition root and compiles for `wasm32-unknown-unknown` without native ACORN host dependencies; `crates/acorn-desktop` embeds that exact static build in Tauri 2 without duplicating routes, components, or domain logic.
- Desktop acceptance requires the ABW-45 capability matrix to account for every host-only CLI/service family with an implemented typed path, a documented shared-UI equivalent, or an explicit, evidenced platform prerequisite. ABW-56 proves export, diagnostics, local mutations, repository/model and native-prerequisite work, daemon control, agent communication, inbound service lifecycle, trusted extensions, integration provisioning and report publication with negative permission/secret tests before ABW-13/14/15 native bundle acceptance; an omitted family is not complete.
- `cargo tree` and target checks prove `acorn-web` has no `acorn-host` or `acorn-lib` edge, `acorn-wasm` has no UI or native-host edge, and `acorn-desktop` delegates selected-path filesystem work to the narrowly enabled `acorn-host/fs` capability rather than owning direct filesystem policy.
- The application uses an OKF-inspired light/dark/system shell, grouped collapsible icon-rich sidebar, centered home workflow, responsive drawer, tool entries, Settings/About routes, locally bundled Driver.js tour, notification region, and routed detail pages without copying OKF planning content or source. The persistent theme control is one native toggle button with visible state, switch geometry, and correct `aria-pressed`; Settings can restore system preference.
- The repository `acorn.svg`, dark banner, and light banner are reused with appropriate accessible alternatives; `v0.6.3`-style version placement derives from the built package rather than hard-coded UI state. Home and non-About footers display exactly “Made with ❤️ by [**jhwohlgemuth**](https://github.com/jhwohlgemuth)” and contain no agent names or marks. About contains no made-with-heart line. Its AI-use disclosure displays the unchanged, locally bundled [AI Assisted badge](https://www.aihonestybadge.com/ai-assisted-badge) with an accessible name and link, then names and links [**Codex**](https://openai.com/codex/), [**GitHub Copilot**](https://github.com/features/copilot), and [**Microsoft 365 Copilot**](https://www.microsoft.com/microsoft-365/copilot) beside their marks and the human direction/review/testing explanation. Each linked label and adjacent mark belongs to one accessible external link, wraps without clipping at narrow widths, and retains a readable text-only meaning when images are unavailable.
- The About page displays unchanged Rust, WebAssembly, Tauri, and Dioxus marks inside their existing technology cards with adjacent text labels, useful alternative text, provenance, and retained license notices. It does not repeat them in a separate “official project marks” strip. The marks do not appear in the persistent footer, remain secondary technology credits, are not recolored or combined into a new logo, and do not imply project endorsement.
- The About page follows the live OKF reference hierarchy closely while using ACORN content: narrow left-aligned column, source/contribution card, project and local-first sections, delivery-target cards, three-column technology grid, and bordered AI-use disclosure. It contains no tour action; the guided-tour entry point remains on Home. The disclosure leads with the AI Assisted honesty badge, incorporates linked names and marks for the three agent products, explains the maintainer's direction/review/testing role, and advises review of AI-generated code; no separate attribution footer is present.
- The accent system uses an approved ORNL signature-green value or explicitly labels `#00662C` and its derived variants as ORNL-inspired/provisional. Purple and blue are not the primary interactive accent; no unapproved logo is present.
- All manual operations accepted in ABW-20 are reachable from the home page and sidebar, use existing ACORN computation, enforce current bounds, and render structured results before optional raw JSON.
- The Portfolio is reachable from Home and the sidebar, follows the catalog exploration pattern of the linked ORNL Research Index and OKF Plan Browser reference, derives its catalog and details from loaded Research Activity data, provides semantic equivalents for every visual summary, and handles empty, single-record, multi-record, and narrow-screen states. Users can retain multiple GitHub/GitLab buckets in one index, identify the source of every activity and image, refresh or remove one bucket without altering the others, and see aggregate file/byte limits before adding another. Same-named paths or project slugs from different buckets remain distinct. Failed, cancelled, or over-limit loads leave the existing index intact. Loaded data is saved automatically to localStorage with bounded/versioned data and a visible **Delete all data** control. Loading controls remain unavailable until saved validation finishes; a failed automatic save is reported, retains the previous snapshot, and leaves the current workspace open.
- One Dioxus/WASM payload contains the application and all nine JavaScript bindings, with recorded release size and no silent duplicated ACORN module.
- `/geospatial` plots deduplicated ROR-backed organization locations and associated loaded projects from RAD using the isolated Resium/CesiumJS island while Dioxus retains all ACORN state and an equivalent accessible table. The production build includes Cesium workers/widgets, locally bundled imagery, and a lazy, pinned, digest-verified ROR location subset with license/provenance; optional external Cesium layers require explicit user choice. It reports unresolved/locationless records plus WebGL failure without losing the table. Organization models remain gated on ABW-37 feasibility evidence.
- GitHub/GitLab PAT entry uses provider-minimum scopes and explicit consent, with newly saved browser credentials bound to one provider and HTTPS host; legacy project-bound entries stay project-bound until replaced. A user may explicitly save a masked credential in sessionStorage so it survives reloads in that tab; visible deletion clears both memory and sessionStorage. Tokens never enter localStorage, URLs, logs, history, errors, redirects, or ACORN services, and anonymous loading remains available. Desktop protected credentials use the separate ABW-48 contract.
- The documented browser and Tauri development commands start one coordinated Dioxus/Vite feedback loop: supported Dioxus UI/style/asset edits hot reload, and compatible Resium TSX/map-style edits use React Fast Refresh without rebuilding the WASM application. Watcher startup, failure, and shutdown are deterministic; production output contains no HMR client, refresh preamble, localhost URL, or development-only CSP allowance.
- The page explicitly installs exactly nine WebMCP tools when supported, reports installation state, routes invocation feedback safely, disposes registrations, and keeps manual controls functional when unsupported.
- Local file or folder selection never uploads content, failed reads preserve the current editor, and the session tree contains only normalized relative paths and supported UTF-8 content under the 500-file/20-segment/256-KiB-per-file/16-MiB-total limits. Unsupported and rejected entries are summarized; no operation runs until the user selects a leaf and invokes it. Copy/download/save actions require explicit user action, object URLs are revoked, and generated filenames are safe. Desktop file/folder open and save occur inside bounded Rust commands after native dialog selection, delegate filesystem work to `acorn-host`, never follow symlinks, and grant no ambient filesystem or absolute-path access to the webview.
- Route changes set a useful document title and focus; navigation, tools, disclosures, results, copy/download, and the responsive drawer pass keyboard acceptance with visible focus and semantic DOM controls.
- Desktop, phone portrait, and phone landscape pass without horizontal page overflow. The globe yields space or collapses behind the accessible organization list at narrow sizes without hiding data. Manual-tool acceptance records the browsers actually observed; WebMCP acceptance records the exact supported Chrome version/configuration.
- Desktop bundle, window, launcher/taskbar/Dock and installer icons use native assets derived from [ACORN – Brown](../../../docs/src/_assets/acorn-app-icon-brown.svg). ABW-13/14/15 record visual icon checks on their respective native hosts; regeneration instructions and the canonical source remain in desktop documentation.
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

Release jobs verify the shared frontend, skill and fixed companion digests before publishing versioned artifacts; channel manifests must reference those bytes. Publish channel availability separately from intended support so `/install` can represent pending/unavailable channels truthfully. Upgrades use the package manager or explicit replacement; no background network checks, updater plugin, automatic agent execution, or capability outside the ABW-45 matrix is added.

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

# Prepared execution batch — November 12, 2026

The user authorized parity-review items 1–5: shared chrome, all tool presentations, browser file/folder workspace, Portfolio, Geospatial, Settings, and browser Skill/Install. Execute serially through the actual shared UI and operation files. Browser-only implementation does not depend on native export or published desktop artifacts. Live acceptance remains required and must not be inferred from unit tests; unavailable browser control is an external validation prerequisite, not a blocker for independent implementation. ABW-08 consolidates the cross-route live matrix while each phase retains its outstanding focused evidence.

The integration owner may use `src/ui.rs`, `src/main.rs`, `src/lib.rs`, new focused `src/presentation.rs`, `src/session.rs`, `src/workspace.rs`, `src/skill.rs` and `src/results.rs` modules, `assets/app.css`, `assets/effects.js`, `assets/effects.test.mjs`, `scripts/acceptance.test.mjs`, the browser README, and the ABW planning/progress surfaces instead of the speculative page/component paths below. Geospatial retains its separate bounded data/bridge/build write set. Settings theme/count/clear/browser-identity behavior is implemented and included in the Chromium parity suite; broader browser acceptance belongs to ABW-08 and native skill export to ABW-22.

# Execution phases

## ABW-22 — Accept native Skill export

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-12; accepted browser Skill implementation and bundle; verified archive metadata for native full-folder links
- Exact write set: `crates/acorn-web/src/skill.rs`, `crates/acorn-web/src/ui.rs`, `crates/acorn-web/assets/effects.js`, `crates/acorn-web/assets/effects.test.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`; native save boundary owned by ABW-12
- Primary integration lock: shared Skill route and application chrome
- Outcome: Connect the implemented shared Skill route to native raw-text save after ABW-12 and a verified exact-version full-folder archive after release metadata exists. Preserve browser digest verification and manual lifecycle guidance.
- Focused validation: Native save/cancel/failure and exported-byte parity; browser/Tauri shared-route identity; exact release archive URL/digest with no agent-folder inspection.
- Handoff: Remove ABW-22 and retain lifecycle documentation plus parity tests; include Skill in ABW-08 and every native package acceptance row.

## ABW-23 — Freeze desktop release identity and publication metadata

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-45 and accepted skill bundle; maintainer-confirmed public release origin, publisher identity, Scoop bucket and Homebrew tap destinations; no concurrent CI/build owner
- Exact write set: `packaging/applications.json`, `xtask/Cargo.toml`, `xtask/src/application_release/`, `Makefile`, `crates/acorn-desktop/release/contract.md` (new), `crates/acorn-desktop/release/manifest.schema.json` (new), `xtask/src/application_release/desktop.rs` (new), `.gitlab-ci.yml`, `crates/acorn-desktop/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: desktop release manifest and CI integration
- Outcome: Freeze collision-checked desktop identities, exact supported OS/architecture rows, immutable asset naming/URLs, channel ownership, signing requirements and publication-state metadata. Include the bundled ACORN companion in native artifact/version/digest and signing rules without replacing a separately installed CLI. Extend the checked application lifecycle and existing immutable artifact uploader with native build/upload dependencies, per-platform checksums and conditional GitLab release links. Retain development/pending states until actual artifact and channel acceptance. Wire app/frontend/skill/companion consistency into release CI and identify native runners/protected signing inputs without storing credentials in the repository.
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

## ABW-31 — Populate verified Install release metadata

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-23 and verified channel publication metadata; accepted pending-channel browser UI
- Exact write set: `crates/acorn-web/src/skill.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: shared Install route and release presentation
- Outcome: Replace pending-only GUI channel rows with bundled, schema-checked publication metadata when ABW-23 and the actual channels supply it. Add available/pending/unavailable fixtures without fabricating installability.
- Focused validation: Validate all five channel versions, architecture rows, immutable URLs and hashes; explicit copy actions and safe links; shared browser/Tauri and offline rendering.
- Handoff: Remove ABW-31 after route and fixture checks pass; add Install to ABW-08. Final release acceptance still waits for real channel publication and verified metadata, not UI fixtures.

## ABW-32 — Accept Big-O boundary cases

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-03 shared input/result components
- Exact write set: `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/src/presentation.rs`, `crates/acorn-web/src/results.rs`, `crates/acorn-web/src/session.rs`, `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/assets/effects.js`, `crates/acorn-web/assets/effects.test.mjs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: browser Big-O operation and route
- Outcome: Complete equivalent/incomparable/undetermined, invalid assumptions and maximum-bound manual/adapter parity against the implemented symbolic result and labeled n/n² illustration.
- Focused validation: Test remaining symbolic outcomes and invalid assumptions through the real WASM adapter; verify accessible output and illustrative-only labels.
- Handoff: Remove ABW-32 after focused tests pass and record manual/adapter parity; unblock ABW-06 and ABW-07 once their other prerequisites pass.



## ABW-03 — Accept metadata edge cases and announcements

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted shell and accepted Home/chrome contract
- Exact write set: `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/src/presentation.rs`, `crates/acorn-web/src/results.rs`, `crates/acorn-web/src/session.rs`, `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/assets/effects.js`, `crates/acorn-web/assets/effects.test.mjs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: browser metadata operation and shared result contract
- Outcome: Complete ordered/truncated/missing-path diagnostic, fuzzy-policy, required seed and unsupported conversion matrices against the implemented structured presentation; verify assistive-technology result announcements.
- Focused validation: Use native/WASM fixtures for every remaining metadata case and a screen reader for announcements; preserve editor content and heading focus.
- Handoff: Remove ABW-03 and preserve stable shared input/result component behavior for the remaining tool groups.

## ABW-04 — Accept analysis edge cases

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted shell; ABW-03 shared result contract frozen
- Exact write set: `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/src/presentation.rs`, `crates/acorn-web/src/results.rs`, `crates/acorn-web/src/session.rs`, `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/assets/effects.js`, `crates/acorn-web/assets/effects.test.mjs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: browser analysis pages
- Outcome: Complete all-metric readability, FAIR-standard/41-finding and equal/changed/invalid JSON Patch matrices against the implemented bars, gauge and diff presentation.
- Focused validation: Exercise each remaining metric/standard and boundary fixture; compare visible values with portable results and verify keyboard/assistive-technology output.
- Handoff: Remove ABW-04 and record any rendering performance threshold needed by ABW-08 without changing portable result semantics.

## ABW-05 — Accept identifier boundary cases

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted shell; ABW-03 shared result contract frozen
- Exact write set: `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/src/presentation.rs`, `crates/acorn-web/src/results.rs`, `crates/acorn-web/src/session.rs`, `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/assets/effects.js`, `crates/acorn-web/assets/effects.test.mjs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: browser identifier pages
- Outcome: Complete the supported-kind, mismatch, ambiguous-candidate ordering, deduplication and 100-result matrix against the implemented cards and inspection drill-down.
- Focused validation: Use representative ARK, arXiv, DOI, Handle, ISBN, ISNI, ORCID, patent, RAiD, ROR, RRID and SWHID fixtures; retain no-resolution behavior.
- Handoff: Remove ABW-05 and confirm all nine home/sidebar entries resolve to complete manual pages before ABW-06/07.

## ABW-06 — Accept browser effects across engines

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-03, ABW-04, ABW-05 and ABW-32
- Exact write set: `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/src/presentation.rs`, `crates/acorn-web/src/results.rs`, `crates/acorn-web/src/session.rs`, `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/assets/effects.js`, `crates/acorn-web/assets/effects.test.mjs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: browser-local presentation-effect boundary
- Outcome: Verify file/copy/download permission failures and cleanup in Safari/WebKit and Firefox against the implemented bounded reader and format-specific exports.
- Focused validation: Record browser/version, chooser cancellation, copy success/denial, download bytes/name/MIME, object-URL cleanup and no external requests. Chromium file/folder/drop and diff-byte checks are already covered by parity-smoke.mjs.
- Handoff: Remove ABW-06, retain the accepted browser-effect boundary, and mark ABW-16 ready.

## ABW-16 — Accept folder capability fallbacks across engines

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-06
- Exact write set: `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/src/presentation.rs`, `crates/acorn-web/src/results.rs`, `crates/acorn-web/src/session.rs`, `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/assets/effects.js`, `crates/acorn-web/assets/effects.test.mjs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: browser supplied-content and session-tree contract
- Outcome: Verify nested directory selection/drop and the capability fallback in Safari/WebKit and Firefox, plus maximum-session interaction timing.
- Focused validation: Exercise folder batches, partial acceptance, cancellation and keyboard tree navigation in each remaining engine; record unsupported directory APIs without weakening file loading.
- Handoff: Remove ABW-16, preserve the accepted supplied-tree DTO and capability fallback in tests/README, make ABW-08 browser acceptance and ABW-17 native folder integration eligible, and record unavailable browser directory-picker rows without weakening single-file workflows.

## ABW-19 — Accept the initial Research Activity Index capability

- Status: in-progress
- Budget: 5 active hours maximum
- Prerequisites: accepted bounded image pipeline, validated/restored workspace, authenticated loaders including private GitLab access, index projection and catalog navigation; no concurrent owner of shared Portfolio state
- Exact write set: `crates/acorn-web/src/session.rs`, `crates/acorn-web/src/research_index.rs`, `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/src/details.rs`, `crates/acorn-web/src/ui.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/scripts/workspace-storage-smoke.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/research-activity-index/index.md`, `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: loaded Research Activity Index state and Portfolio route
- Outcome: Accept the initial token-backed Research Activity Index end to end: load a private GitLab or GitHub bucket with associated images, automatically restore the bounded workspace until **Delete all data**, project only validated RAD, search and count results, switch alphabetical/relevance sort and project List/Cards while matching activities remain tabular, navigate project/activity details and source links, and preserve correct behavior through replacement, deletion, empty, single-record, mixed-record, narrow-screen, and 500-file sessions.
- Focused validation: Use fictional public and private-provider fixtures plus restored-workspace fixtures; compare projection/catalog/detail counts, exercise keyboard search/sort/layout/navigation, image and placeholder controls, verify token redaction and semantic equivalents, and record maximum-session timing in Chromium with unavailable cross-engine rows named rather than inferred. Recheck the hosted private-provider path after publishing the new media build without retaining private records in the repository.
- Handoff: Remove ABW-19 after preserving the accepted initial index contract and dated evidence in tests/README; make ABW-63 eligible. Keep the Research Activity Index epic until ABW-65 accepts multiple buckets.

## ABW-63 — Compose multiple repository buckets in one bounded workspace

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-19 accepted; no concurrent owner of workspace state, catalog projection, or browser storage schema
- Exact write set: `crates/acorn-web/src/session.rs`, `crates/acorn-web/src/research_index.rs`, `crates/acorn-web/src/media.rs`, `crates/acorn-web/src/tests/mod.rs`, `crates/acorn-web/assets/workspace-storage.js`, `crates/acorn-web/assets/workspace-storage.test.mjs`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: multi-bucket workspace and persisted source identity
- Outcome: Add an atomic append/refresh/remove contract for several GitHub/GitLab buckets. Retain provider, host, project, path, and pinned revision per bucket; namespace same-named files and project identities so validation, media, details, and source links resolve to the correct bucket. Apply the existing 500-file, 256-KiB-per-file, and 16-MiB stored limits to the combined workspace. Keep existing data on failed, cancelled, or over-limit additions, migrate a valid single-bucket snapshot, and keep credentials out of persistent workspace data. Define local-file/sample replacement behavior explicitly without broadening this phase into local multi-source intake.
- Focused validation: Rust state/index/media tests for two providers, duplicate relative paths and project slugs, refresh/removal, validation isolation, combined limits, and failure atomicity; JavaScript storage tests for versioned single-bucket migration, multi-bucket restore/delete, corrupt entries, and token exclusion.
- Handoff: Preserve the source-aware state and storage contract in tests; make ABW-64 eligible without changing its UI write set.

## ABW-64 — Load and explore multiple buckets in the browser

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-63; no concurrent owner of Home loader, Portfolio, shared CSS, or browser parity scripts
- Exact write set: `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/details.rs`, `crates/acorn-web/assets/remote.js`, `crates/acorn-web/assets/remote.test.mjs`, `crates/acorn-web/assets/app.css`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: Home repository loader and Portfolio source controls
- Outcome: Let users add another public or token-backed GitHub/GitLab bucket, refresh one, and remove one while keeping other loaded buckets, their validation results, and selected activity intact. Show each source, aggregate usage, and per-activity provenance in Home, Portfolio, and details; make append versus replacement clear. Keep provider/host token scoping, cancellation, partial-file reporting, and explicit **Delete all data** behavior. Preserve single-file and local-folder workflows.
- Focused validation: Focused Rust and remote-loader tests plus a local Chromium two-bucket smoke for add, refresh, remove, and source labels; verify keyboard operation and no token exposure. Leave the complete cross-state/browser acceptance matrix to ABW-65.
- Handoff: Preserve UI behavior in focused tests; make ABW-65 eligible and retain the Research Activity Index epic.

## ABW-65 — Accept multiple-bucket index behavior

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-64; built browser with multi-bucket state and controls; no concurrent owner of browser parity scripts or documentation
- Exact write set: `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/scripts/workspace-storage-smoke.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/research-activity-index/index.md`
- Primary integration lock: multi-bucket browser acceptance matrix
- Outcome: Accept combined GitHub/GitLab buckets across loading, validation, Portfolio, persistence, and source links without silent replacement, provenance loss, credential leakage, or aggregate-limit bypass. Record unsupported browser or hosted-provider rows explicitly; any product defect outside this write set opens a bounded repair phase.
- Focused validation: Chromium flows for two public buckets, mixed public/private hosts, duplicate paths and slugs, refresh, per-bucket removal, reload, cancellation/error/limit rollback, keyboard controls, 390px layout, source links, token redaction, and a 500-file aggregate. Recheck the hosted private-provider path if loader or credential behavior changed; run focused Rust/JavaScript tests and compare saved-workspace behavior after reload and **Delete all data**.
- Handoff: Record dated evidence and limits in the README, remove ABW-65, retire the Research Activity Index epic, and make ABW-44 and ABW-08 eligible.

## ABW-44 — Visualize the loaded project buckets on Cesium

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-65; accepted index projection and existing organization/ROR map bridge; no concurrent owner of geospatial route or island
- Exact write set: `crates/acorn-web/src/research_index.rs`, `crates/acorn-web/src/geospatial.rs`, `crates/acorn-web/src/geospatial_page.rs`, `crates/acorn-web/geospatial/src/bridge.ts`, `crates/acorn-web/geospatial/src/map.tsx`, `crates/acorn-web/geospatial/src/map.css`, `crates/acorn-web/geospatial/tests/bridge.test.mjs`, `crates/acorn-web/geospatial/tests/home.test.mjs`, `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-geospatial-preview.svg`
- Primary integration lock: project-aware geospatial DTO and Cesium selection state
- Outcome: Project all retained buckets into the existing Cesium route. Associate validated projects/activities with resolved ROR locations, deduplicate organization coordinates without losing source-aware project membership, render project-aware markers or clusters, add project filters and synchronized selection/details, and retain an equivalent ordered table for unresolved, locationless, narrow-screen, and WebGL-unavailable states. Add no new runtime registry or model request.
- Focused validation: Run focused Rust projection tests and geospatial bridge tests, then Chromium checks for multiple projects per organization, projects spanning organizations, duplicate/missing ROR identifiers, filter/selection parity, replacement/restoration, accessible-table equivalence, no-network default, and bounded payload size.
- Handoff: Preserve the versioned project DTO and fallback behavior in tests/README; make ABW-18 eligible.

## ABW-18 — Accept project-aware map resource and performance budgets

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-44; no concurrent owner of `acorn-web` geospatial route, island, or final static-asset staging
- Exact write set: `crates/acorn-web/src/geospatial.rs`, `crates/acorn-web/src/geospatial_page.rs`, `crates/acorn-web/geospatial/src/bridge.ts`, `crates/acorn-web/geospatial/src/map.tsx`, `crates/acorn-web/geospatial/src/map.css`, `crates/acorn-web/geospatial/tests/bridge.test.mjs`, `crates/acorn-web/geospatial/tests/home.test.mjs`, `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-geospatial-preview.svg`
- Primary integration lock: Resium/CesiumJS project rendering and measured resource budget
- Outcome: Complete maximum-project/marker timing and memory measurements, lazy-index failure/retry, and remaining browser selection/fallback acceptance for the project-aware offline map. Review the measured static and runtime budgets before release. Native-webview acceptance belongs to ABW-11 and ABW-13–15.
- Focused validation: Exercise the 500-file workspace at up to 1,000 project/location entities, absent/corrupt local index, duplicate/unresolved/locationless organizations, keyboard/table/marker selection, filtering, WebGL failure, and replacement/restoration across available engines; measure memory and interaction response and preserve the no-network and Fast Refresh checks.
- Handoff: Remove ABW-18 after preserving the project DTO, bridge schema, measured budget, no-network configuration, and accessible fallback in `crates/acorn-web/README.md`; make ABW-37, ABW-08, and ABW-11 eligible.

## ABW-37 — Assess 3D research-organization models

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-18 project-aware map measurements accepted
- Exact write set: `.wiki/features/acorn-browser-workspace/organization-model-feasibility.md` (new), `.wiki/features/acorn-browser-workspace/_assets/organization-model-spike/index.html` (new, optional), `.wiki/features/acorn-browser-workspace/_assets/organization-model-spike/spike.js` (new, optional), `.wiki/features/acorn-browser-workspace/_assets/organization-model-spike/README.md` (new, optional measurements/provenance), `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: Cesium organization/project visualization contract
- Outcome: Produce a source-backed go/no-go assessment for displaying research-organization 3D models in a 2.5D Cesium view. Discover authoritative assets or a lawful creation path; evaluate provenance/license/attribution, glTF/glb compatibility, coordinate/orientation/scale metadata, loading/caching and bundle budgets, WebGL/device cost, camera/selection behavior, accessibility, and marker/footprint fallback. No model asset or runtime request is adopted in this phase.
- Focused validation: Inspect official Cesium model/scene guidance and exact candidate licenses/metadata. If evidence is otherwise insufficient, build only the smallest disposable 2.5D prototype and record asset size, load/render observations, camera behavior, attribution, accessible alternative, and comparison with ABW-18 budgets. Mark missing evidence unresolved.
- Handoff: Open a separately bounded implementation phase only for a supported go decision; otherwise preserve the no-go/conditional assessment and retain accessible project-aware markers and table.

## ABW-07 — Complete WebMCP workspace lifecycle acceptance

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted browser composition, ABW-03, ABW-04, ABW-05, accepted Home/chrome, ABW-32; no concurrent WME-02/WME-04 or other WASM package-staging owner
- Exact write set: `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/assets/`, `crates/acorn-web/tests/`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`, `Makefile`
- Primary integration lock: WebMCP application installation and WASM package staging
- Outcome: Complete fake-context application integration tests and live supported-Chrome discovery/invocation/cleanup for the implemented unchanged nine-tool adapter installation. Verify busy/cancellation/failure/disposal and duplicate-install behavior at the workspace boundary, keeping manual routes available when unsupported.
- Focused validation: `make test-webmcp`; new workspace integration test with fake supported/unsupported contexts; exactly nine registered names and schemas; representative manual/adapter parity for every operation; busy, cancellation, bounded rejection, failure-envelope, duplicate-install, and disposal cases; `make test-webmcp-package`; Dioxus build export inspection; live supported-Chrome registration/discovery/invocation/cleanup when available.
- Handoff: Remove ABW-07 and record the exact supported Chrome/configuration evidence. Missing native support remains an explicit ABW-08 matrix blocker, not a reason to weaken adapter tests.

## ABW-08 — Accept responsive, accessible, and bounded browser behavior

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-03 through ABW-07, ABW-16, ABW-18, ABW-19, ABW-32, and ABW-65; browser Skill and pending Install contracts are implemented; native/publication acceptance remains independent
- Exact write set: `crates/acorn-web/assets/app.css`, `crates/acorn-web/src/theme.rs`, `crates/acorn-web/src/components/app_shell.rs`, `crates/acorn-web/src/components/sidebar.rs`, `crates/acorn-web/src/components/tour.rs`, `crates/acorn-web/src/components/theme_toggle.rs`, `crates/acorn-web/src/components/notification.rs`, `crates/acorn-web/src/components/content_input.rs`, `crates/acorn-web/src/components/file_tree.rs`, `crates/acorn-web/src/effects/browser_folder.rs`, `crates/acorn-web/src/components/result_panel.rs`, `crates/acorn-web/src/pages/home.rs`, `crates/acorn-web/src/pages/portfolio.rs`, `crates/acorn-web/src/pages/geospatial.rs`, `crates/acorn-web/src/pages/settings.rs`, `crates/acorn-web/src/pages/install.rs`, `crates/acorn-web/src/pages/skill.rs`, `crates/acorn-web/geospatial/src/map.tsx`, `crates/acorn-web/tests/browser-smoke.mjs` (new), `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-workspace-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-portfolio-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-validation-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-analysis-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-identifiers-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-geospatial-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-settings-preview.svg`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-about-preview.svg`
- Primary integration lock: browser workspace acceptance matrix
- Outcome: Close responsive, keyboard, focus, zoom, reduced-motion, semantic, contrast, performance, WebGL-fallback, and bounded-error defects across the complete route matrix at desktop, phone portrait, and phone landscape without changing operation or geospatial bridge contracts.
- Focused validation: Run the complete manual-tool and geospatial matrix in available Chrome, Firefox, and Safari/WebKit; supported-Chrome WebMCP matrix separately; keyboard-only navigation, organization-table use, and drawer focus return; 200% zoom; reduced motion; automated accessibility/contrast scan; no horizontal overflow at all three viewports; forced WebGL failure; console/network inspection proving no external map/ROR request; maximum-bound timing and interaction responsiveness; rerun focused Rust/WASM, Resium bridge, and WebMCP tests after repairs.
- Handoff: Remove ABW-08 and record dated browser/OS/viewport/assistive-technology evidence plus every unavailable row. Any acceptance defect outside the exact write set becomes a bounded repair phase.

## ABW-11 — Embed the shared Dioxus workspace in a minimal Tauri shell

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted browser composition and shell, accepted Home/chrome, and ABW-18; no concurrent workspace-manifest, frontend-development-server, or lockfile owner
- Exact write set: `packaging/applications.json`, `xtask/Cargo.toml`, `xtask/src/application_release/`, `.gitlab-ci.yml`, `Makefile`, `Cargo.toml`, `Cargo.lock`, `crates/acorn-desktop/Cargo.toml` (new), `crates/acorn-desktop/build.rs` (new), `crates/acorn-desktop/tauri.conf.json` (new), `crates/acorn-desktop/capabilities/main.json` (new), `crates/acorn-desktop/icons/` (new, derived from `docs/src/_assets/acorn-app-icon-brown.svg`), `crates/acorn-desktop/src/main.rs` (new), `crates/acorn-desktop/README.md` (new), `.wiki/features/acorn-browser-workspace/index.md`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-workspace-preview.svg`
- Primary integration lock: workspace manifest and Tauri application shell
- Outcome: Add a flat interface workspace member for Tauri 2 with `version.workspace = true` and `publish = false`; transition its publication inventory from `planned` to `development`, add native CI/build checks and explicitly exclude it from Cargo workspace packaging. Then point `frontendDist` at the accepted static `acorn-web` output, set `devUrl` to the loopback Dioxus development server, and use `beforeDevCommand` to start the accepted coordinated Dioxus/Vite workflow. Generate the native PNG, ICO and ICNS icon assets from [ACORN – Brown](../../../docs/src/_assets/acorn-app-icon-brown.svg), preserving its colors, dark details, rounded background and transparent outer corners; configure the Tauri bundle and main-window icon from those assets and document the regeneration command. Open one labeled main window with that icon, inject restrictive production and development Content Security Policies, and prove browser and desktop load the same route/component build. Development policy permits only the documented loopback HMR origins; production permits neither. The shell exposes no custom command or plugin permission yet, imports neither `acorn-lib` nor portable operation crates, and never loads a remote production URL.
- Focused validation: `make check-release-apps`; CI configuration and offline Cargo workspace packaging checks; `cargo test -p acorn-desktop`; `cargo check -p acorn-desktop`; build `acorn-web`, launch `cargo tauri dev` from `crates/acorn-desktop`, verify `beforeDevCommand` starts and stops both frontend watchers, and observe representative Dioxus RSX/CSS and Resium TSX/map-style edits update the existing webview through their respective hot-reload channels; run `cargo tauri build --no-bundle`; inspect the embedded asset manifest, window label, development and production CSPs, enabled capability list, binary provenance, offline launch, route parity, version/runtime display, JS/WASM hash equality with the accepted web build, and absence of HMR clients, localhost URLs, and development-only CSP sources from release output; inspect configured PNG/ICO/ICNS assets against the selected brown SVG at small and large sizes, checking colors, dark details, transparency and clipping; compare the implementation with the canonical UI SVG.
- Handoff: Remove ABW-11, preserve the canonical icon source, regeneration command, exact dev/build prerequisites and shared-asset proof in `crates/acorn-desktop/README.md`, and mark ABW-12 ready. Any need for a second frontend or remote URL fails the phase and requires an architecture repair.

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
- Handoff: Remove ABW-17, preserve the selected-root threat model and limits in `crates/acorn-desktop/README.md`, and mark ABW-45 ready.

## ABW-45 — Freeze the native capability matrix and companion boundary

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-11, ABW-12, and ABW-17; exclusive workspace manifest and native command owner
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-host/Cargo.toml`, `crates/acorn-host/src/lib.rs`, `crates/acorn-host/src/command.rs` (new), `crates/acorn-host/src/tests/command.rs` (new), `crates/acorn-desktop/Cargo.toml`, `crates/acorn-desktop/tauri.conf.json`, `crates/acorn-desktop/src/commands.rs`, `crates/acorn-desktop/src/companion.rs` (new), `crates/acorn-desktop/src/tests.rs`, `crates/acorn-desktop/acceptance/capability-matrix.md` (new), `crates/acorn-desktop/README.md`, `crates/acorn-web/src/host.rs` (new), `crates/acorn-web/src/main.rs`, `crates/acorn-web/src/ui.rs`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: versioned desktop companion and host process contract
- Outcome: Freeze every host-only CLI/service behavior against the current command inventory and 72-row native parity fixture, with owner, input/output, effect, platform prerequisite, consent, and ABW phase. Establish the shared desktop host route with browser-unavailable states. Bundle a version-matched `acorn` companion under a fixed app-owned path, verify its version/digest, and add a bounded `acorn-host` invocation port that accepts only named operations from the desktop backend. Never accept executable paths, arbitrary arguments, directories, or shell text from the webview; ordinary WASM tools work if the companion is unavailable.
- Focused validation: Fake-executable and corrupted/mismatched-companion tests; cancellation, timeout, output and environment limits; package and capability inspection; no `acorn-lib` desktop edge, PATH fallback, generic Tauri process permission, or browser behavior change.
- Handoff: Remove ABW-45 only after the matrix and companion contract are recorded; make ABW-46/48/49 ready and ABW-23 eligible for its release-manifest work.

## ABW-46 — Export PDF from a selected desktop source

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-45 and ABW-12
- Exact write set: `crates/acorn-desktop/src/commands.rs`, `crates/acorn-desktop/src/commands/export.rs` (new), `crates/acorn-desktop/src/tests.rs`, `crates/acorn-desktop/capabilities/main.json`, `crates/acorn-web/src/host.rs`, `crates/acorn-web/src/host/export.rs` (new), `crates/acorn-web/src/ui.rs`, `crates/acorn-desktop/acceptance/capability-matrix.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: typed native export IPC
- Outcome: Present PDF export/report options over selected content, show destination and renderer readiness, and invoke the existing ACORN PDF path through the fixed companion only after the user confirms. Preserve current PDF settings and exact output-file semantics; no renderer or arbitrary path access crosses IPC.
- Focused validation: Fixture PDF and report output, cancel/failure/oversize cases, missing Chromium state, selected-source freshness, wrong-window invoke denial, and browser route regression.
- Handoff: Record renderer and artifact evidence, remove ABW-46, and make ABW-47 ready.

## ABW-47 — Export PowerPoint and remaining native formats

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-46
- Exact write set: `crates/acorn-desktop/src/commands.rs`, `crates/acorn-desktop/src/commands/export.rs`, `crates/acorn-desktop/src/tests.rs`, `crates/acorn-web/src/host.rs`, `crates/acorn-web/src/host/export.rs`, `crates/acorn-desktop/acceptance/capability-matrix.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: native export format contract
- Outcome: Add PPTX export/report plus applicable DOCX, BagIt, CFF, JSON, JSON-LD, JSONL, Markdown, RDF/XML, Turtle, YAML, and ZON outputs from the existing CLI format contract. Keep format-specific inputs, artifact naming and binary/text save behavior explicit; never present an unsupported format as complete.
- Focused validation: One golden artifact per format family, PPTX structure/openability, malformed source, cancellation, destination collision, size bounds and source/CLI equivalence; matrix records any host-only prerequisite.
- Handoff: Record the supported format/renderer matrix and remove ABW-47.

## ABW-48 — Store desktop credentials with explicit scope

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-45
- Exact write set: `crates/acorn-desktop/src/commands.rs`, `crates/acorn-desktop/src/credentials.rs` (new), `crates/acorn-desktop/src/commands/credentials.rs` (new), `crates/acorn-desktop/src/tests.rs`, `crates/acorn-desktop/capabilities/main.json`, `crates/acorn-web/src/host.rs`, `crates/acorn-web/src/host/credentials.rs` (new), `crates/acorn-web/src/ui.rs`, `crates/acorn-desktop/acceptance/capability-matrix.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: native protected-store and secret IPC
- Outcome: Add masked save/replace/delete/status controls for provider-host PATs and named daemon/agent/endpoint profiles using an OS-protected user store. Keep secrets out of workspace persistence, companion arguments, process output, logs, errors and agent context; expose only scoped handles to later host operations.
- Focused validation: Per-profile isolation, delete/restart, unavailable protected store, wrong-host/agent use, redaction and negative IPC tests on each available native OS; browser sessionStorage behavior remains independent.
- Handoff: Record store backend and platform prerequisites, remove ABW-48, and make remote/agent integration eligible.

## ABW-49 — Expose local checks and diagnostics

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-45
- Exact write set: `crates/acorn-desktop/src/commands.rs`, `crates/acorn-desktop/src/commands/local.rs` (new), `crates/acorn-desktop/src/tests.rs`, `crates/acorn-desktop/capabilities/main.json`, `crates/acorn-web/src/host.rs`, `crates/acorn-web/src/host/local.rs` (new), `crates/acorn-web/src/ui.rs`, `crates/acorn-desktop/acceptance/capability-matrix.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: local diagnostic command contract
- Outcome: Run explicit `check` and read-only `doctor` workflows on selected local content or app diagnostics through the fixed companion. Show structured findings, missing tool prerequisites, progress and cancellation without automatically fixing or exposing an ambient working directory.
- Focused validation: CLI-equivalent fixture findings, absent tool, wrong source, timeout, oversized output, cancellation, offline operation and forged-command rejection.
- Handoff: Record diagnostic parity rows, remove ABW-49, and make ABW-50 ready.

## ABW-50 — Review local creation and mutation workflows

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-49 and ABW-12
- Exact write set: `crates/acorn-desktop/src/commands.rs`, `crates/acorn-desktop/src/commands/local.rs`, `crates/acorn-desktop/src/tests.rs`, `crates/acorn-web/src/host.rs`, `crates/acorn-web/src/host/local.rs`, `crates/acorn-desktop/acceptance/capability-matrix.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: reviewed local write contract
- Outcome: Add the host-only local creation, `format`, `link`, SWHID, check-fix and doctor-fix rows with typed selection, dry run or diff preview where mutation is possible, explicit confirmation, source freshness and bounded artifact destinations. Pure in-memory equivalents continue to use WASM; integration provisioning from `create mcp/runner/bot/secret` belongs to ABW-58.
- Focused validation: Per-command fixture results, denied/stale/changed source, cancel, selected-root confinement, preview versus applied bytes, rollback on failure, and no arbitrary path or command invocation.
- Handoff: Record row-level local parity, remove ABW-50, and make ABW-51 ready.

## ABW-51 — Add consented repository and intake workflows

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-48 and ABW-45
- Exact write set: `crates/acorn-desktop/src/commands.rs`, `crates/acorn-desktop/src/commands/repository.rs` (new), `crates/acorn-desktop/src/tests.rs`, `crates/acorn-desktop/capabilities/main.json`, `crates/acorn-web/src/host.rs`, `crates/acorn-web/src/host/repository.rs` (new), `crates/acorn-web/src/ui.rs`, `crates/acorn-desktop/acceptance/capability-matrix.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: desktop source and network authority
- Outcome: Expose the existing host `download`, `gather`, API-spec `import`, and intake stage/inspect/apply/export/deliver/run rows through explicit source, destination, provider, host and scope selection. Use existing ACORN repository/authorization policy through the companion; never share a saved token with another host or persist downloaded content without user choice. Model metadata import belongs to ABW-52.
- Focused validation: Offline, private/public source, wrong-host token, redirect, cancellation, partial download, input/output bounds and CLI-equivalent fixture outcomes; consent shown before network access.
- Handoff: Record provider and offline evidence, remove ABW-51, and make ABW-52 ready.

## ABW-52 — Manage native model assets and configuration

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-48 and ABW-45
- Exact write set: `crates/acorn-desktop/src/commands.rs`, `crates/acorn-desktop/src/commands/model.rs` (new), `crates/acorn-desktop/src/tests.rs`, `crates/acorn-desktop/capabilities/main.json`, `crates/acorn-web/src/host.rs`, `crates/acorn-web/src/host/model.rs` (new), `crates/acorn-web/src/ui.rs`, `crates/acorn-desktop/acceptance/capability-matrix.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: model acquisition and native configuration
- Outcome: Add user-started model and Needle download/verification, model metadata import, Chromium/Vale prerequisite acquisition, and `sync` setup using the existing ACORN host policy, with license/size/destination disclosure, progress, cancellation and cleanup. Do not start inference or fetch assets at app launch.
- Focused validation: Fake registry and cached assets, checksum/license refusal, quota, offline, cancel/partial cleanup, per-prerequisite readiness, exact destination and no credential or arbitrary command leak.
- Handoff: Record supported model/profile rows and remove ABW-52.

## ABW-53 — Manage the local acornd service

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-45, ACD-09, and the relevant ACD-10/11/13 OS service-manager contract; no concurrent daemon CLI/service owner
- Exact write set: `crates/acorn-desktop/src/commands.rs`, `crates/acorn-desktop/src/commands/daemon.rs` (new), `crates/acorn-desktop/src/tests.rs`, `crates/acorn-desktop/capabilities/main.json`, `crates/acorn-web/src/host.rs`, `crates/acorn-web/src/host/daemon.rs` (new), `crates/acorn-web/src/ui.rs`, `crates/acorn-desktop/acceptance/capability-matrix.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: daemon control adapter
- Outcome: Present authenticated local health/status/history and explicit run/pause/resume/reload plus install/start/stop/uninstall only where the acornd plan has implemented that host manager. Use its CLI/client and protected local token; never implement a second scheduler or silently install/start the service.
- Focused validation: Absent/stopped/unauthorized daemon, bounded history, job control and lifecycle fixtures, failure rollback, secret redaction, offline local loopback and native service-manager evidence per available OS.
- Handoff: Record ACD version/host dependency, remove ABW-53, and leave unavailable OS rows open for native acceptance.

## ABW-54 — Connect configured agents and outbound tools

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-45 and ABW-48; configured ACP/MCP/endpoint adapter contracts remain available
- Exact write set: `crates/acorn-desktop/src/commands.rs`, `crates/acorn-desktop/src/commands/agent.rs` (new), `crates/acorn-desktop/src/tests.rs`, `crates/acorn-desktop/capabilities/main.json`, `crates/acorn-web/src/host.rs`, `crates/acorn-web/src/host/agent.rs` (new), `crates/acorn-web/src/ui.rs`, `crates/acorn-desktop/acceptance/capability-matrix.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: desktop agent transport and consent
- Outcome: Expose configured `infer`/Ask, ACP, outbound MCP list/call, and named endpoint sessions with explicit target, content sharing, bounded turns/results, cancellation, offline and mutation policy. Preserve agent provenance; never treat model text or an MCP description as native authority. ACK owns its distinct conversational RAD review panel.
- Focused validation: Fake ACP/MCP/endpoint transports, wrong profile/host, refusal, timeout, malformed result, cancellation, denied mutation, no selected-file transfer before consent, and no reusable path or Tauri handle in agent context.
- Handoff: Record supported profiles and transport evidence, remove ABW-54, and keep unavailable external agents explicit.

## ABW-55 — Control opt-in local ACORN protocol servers

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-45 and ABW-48
- Exact write set: `crates/acorn-desktop/src/commands.rs`, `crates/acorn-desktop/src/commands/server.rs` (new), `crates/acorn-desktop/src/tests.rs`, `crates/acorn-desktop/capabilities/main.json`, `crates/acorn-web/src/host.rs`, `crates/acorn-web/src/host/server.rs` (new), `crates/acorn-web/src/ui.rs`, `crates/acorn-desktop/acceptance/capability-matrix.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: inbound service lifecycle
- Outcome: Offer explicit start/stop/status for supported ACORN `serve` MCP, RPC, gRPC and bot modes through the fixed companion, with loopback default, authenticated/non-loopback policy, visible bind address and clean shutdown. The shared UI's manual tools never depend on a running server.
- Focused validation: No listener before consent, bind/auth refusal, address collision, status/cancel/exit, bounded logs, redacted token and process cleanup; remote/bot prerequisites remain visible and unavailable without configuration.
- Handoff: Record accepted protocol modes and remove ABW-55.

## ABW-57 — Review and run native Rhai extensions

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-45, ABW-48, and ABW-50; accepted executable-hook manifest policy
- Exact write set: `crates/acorn-desktop/src/commands.rs`, `crates/acorn-desktop/src/commands/extension.rs` (new), `crates/acorn-desktop/src/tests.rs`, `crates/acorn-desktop/capabilities/main.json`, `crates/acorn-web/src/host.rs`, `crates/acorn-web/src/host/extension.rs` (new), `crates/acorn-web/src/ui.rs`, `crates/acorn-desktop/acceptance/capability-matrix.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: selected-script trust and extension-effect contract
- Outcome: Offer `exec check-safe`, manifest and hook checks, then bounded `exec check` on an explicitly selected script and input. Require a separate trust decision, effect preview and confirmation for `exec automation`; preserve the CLI's trusted-local-script distinction, cancellation and output limits without granting the webview a generic script or process primitive.
- Focused validation: Static and manifest fixtures, denied manifest, script/input freshness, timeout and resource limits, extension error redaction, automation refusal and confirmation, mutation preview, wrong-window denial and no script evaluation before consent.
- Handoff: Record extension policy and fixture equivalence, then remove ABW-57.

## ABW-58 — Provision native integration resources

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-45 and ABW-48; configured GitLab and container prerequisites when applicable
- Exact write set: `crates/acorn-desktop/src/commands.rs`, `crates/acorn-desktop/src/commands/provision.rs` (new), `crates/acorn-desktop/src/tests.rs`, `crates/acorn-desktop/capabilities/main.json`, `crates/acorn-web/src/host.rs`, `crates/acorn-web/src/host/provision.rs` (new), `crates/acorn-web/src/ui.rs`, `crates/acorn-desktop/acceptance/capability-matrix.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: integration provisioning and external mutation consent
- Outcome: Expose reviewed `create mcp`, `create runner`, `create bot` and `create secret` workflows through typed forms. Show generated configuration or target changes before applying them, require explicit authorization for GitLab/container mutations, and keep generated secrets in protected storage or the user's explicit clipboard action.
- Focused validation: CLI-equivalent generated artifacts, absent Docker/GitLab, wrong-host credential, refused deployment, idempotent retry, secret redaction, clipboard failure, cancellation and no network or container action before consent.
- Handoff: Record per-resource outcome and prerequisite evidence, then remove ABW-58.

## ABW-59 — Publish reviewed native reports

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-45, ABW-47, and ABW-48; configured GitLab publication target
- Exact write set: `crates/acorn-desktop/src/commands.rs`, `crates/acorn-desktop/src/commands/report.rs` (new), `crates/acorn-desktop/src/tests.rs`, `crates/acorn-desktop/capabilities/main.json`, `crates/acorn-web/src/host.rs`, `crates/acorn-web/src/host/report.rs` (new), `crates/acorn-web/src/ui.rs`, `crates/acorn-desktop/acceptance/capability-matrix.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: report artifact and publication authority
- Outcome: Preview the bounded artifacts, commit/source identity, time range and immutable GitLab package target represented by `acorn report`; require an explicit publish action with a matching protected credential. Separate local artifact preview from external publication and show the final package identity and failure state.
- Focused validation: Fixture report equivalence, missing or wrong-host token, changed source/commit, package collision, retry, upload failure, cancellation, redaction and proof that preview produces no remote write.
- Handoff: Record artifact and target evidence, then remove ABW-59.

## ABW-56 — Accept the desktop host capability matrix

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-46 through ABW-55, ABW-57 through ABW-59, ABW-22, and ABW-17; accepted versioned companion and native host prerequisites
- Exact write set: `crates/acorn-desktop/acceptance/capability-matrix.md`, `crates/acorn-desktop/src/tests.rs`, `crates/acorn-web/src/host.rs`, `crates/acorn-web/src/host/tests.rs` (new), `crates/acorn-desktop/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: complete desktop capability acceptance
- Outcome: Reconcile each host-only CLI/service row with a reachable typed desktop workflow, an explicitly recorded shared-UI equivalent such as `tui`, or an evidenced platform prerequisite; prove exact frontend/skill/companion version and digest, browser isolation, negative IPC cases, offline baseline, and no silent capability omissions. Preserve OS-specific live evidence for ABW-13/14/15 rather than claiming it from fixtures.
- Focused validation: `cargo test -p acorn-desktop`, `cargo test -p acorn-host`, `cargo check -p acorn-desktop`, focused web host-view tests, matrix-to-command inventory check, local `cargo tauri build --no-bundle`, capability/CSP inspection and `git diff --check`.
- Handoff: Remove ABW-56 only with row-level evidence and hand the complete native matrix to ABW-13/14/15; missing prerequisite or oversized row remains an explicit bounded phase.

## ABW-13 — Accept the local Windows desktop bundle

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-08 and ABW-56; authorized Windows 11 x86_64 or ARM64 host with current Tauri prerequisites
- Exact write set: `crates/acorn-desktop/tauri.windows.conf.json` (new), `crates/acorn-desktop/acceptance/windows.md` (new), `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: Windows Tauri bundle acceptance
- Outcome: Produce and launch an unsigned local Windows bundle, prove shared UI plus the complete ABW-56 host capability matrix under WebView2, and record OS, architecture, WebView2, artifact, companion, size, capability, accessibility, and offline evidence without claiming production signing or Store readiness.
- Focused validation: Native `cargo tauri build`; clean install/launch/uninstall smoke for the selected local bundle type; complete nine-tool, geospatial and ABW-56 host capability matrix; keyboard, 200% zoom, theme toggle/system reset, Driver.js tour, notification, icon, footer, offline Resium/Cesium assets and ROR markers/table, forced WebGL fallback, file/folder open/cancel/partial/error and save flows, nested tree selection, symlink exclusion, offline, console, network, capability-negative, and actual WebMCP-support checks; record hashes and artifact paths without committing generated bundles.
- Handoff: Remove ABW-13 and retain the dated Windows evidence in `crates/acorn-desktop/acceptance/windows.md`. Defects outside the write set open a bounded repair phase; a missing authorized host keeps this phase blocked.

## ABW-14 — Accept the local macOS desktop bundle

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-08 and ABW-56; authorized macOS Apple Silicon or x86_64 host with current Tauri prerequisites
- Exact write set: `crates/acorn-desktop/tauri.macos.conf.json` (new), `crates/acorn-desktop/acceptance/macos.md` (new), `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: macOS Tauri bundle acceptance
- Outcome: Produce and launch an unsigned local macOS application bundle, prove shared UI plus the complete ABW-56 host capability matrix under the system webview, and record OS, architecture, WebKit, artifact, companion, size, capability, accessibility, and offline evidence without claiming signing, notarization, or App Store readiness.
- Focused validation: Native `cargo tauri build --bundles app`; launch/quarantine limitation smoke for the local app bundle; complete nine-tool, geospatial and ABW-56 host capability matrix; keyboard, 200% zoom, theme toggle/system reset, Driver.js tour, notification, icon, footer, offline Resium/Cesium assets and ROR markers/table, forced WebGL fallback, file/folder open/cancel/partial/error and save flows including security-scoped selection, nested tree selection, symlink exclusion, offline, console, network, capability-negative, and actual WebMCP-support checks; record hashes and artifact paths without committing generated bundles.
- Handoff: Remove ABW-14 and retain the dated macOS evidence in `crates/acorn-desktop/acceptance/macos.md`. Defects outside the write set open a bounded repair phase; a missing authorized host keeps this phase blocked.

## ABW-15 — Accept the local Linux desktop bundle

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-08 and ABW-56; authorized supported Linux x86_64 or ARM64 host with current WebKitGTK/Tauri prerequisites
- Exact write set: `crates/acorn-desktop/tauri.linux.conf.json` (new), `crates/acorn-desktop/acceptance/linux.md` (new), `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: Linux Tauri bundle acceptance
- Outcome: Produce and launch one unsigned local Linux bundle, prove shared UI plus the complete ABW-56 host capability matrix under WebKitGTK, and record distribution, architecture, WebKitGTK, artifact, companion, size, capability, accessibility, and offline evidence without claiming repository, package-manager, or store publication.
- Focused validation: Native `cargo tauri build` for the selected local bundle type; clean launch/removal smoke; complete nine-tool, geospatial and ABW-56 host capability matrix; keyboard, 200% zoom, theme toggle/system reset, Driver.js tour, notification, icon, footer, offline Resium/Cesium assets and ROR markers/table, forced WebGL fallback, file/folder open/cancel/partial/error and save flows, nested tree selection, symlink exclusion, offline, console, network, capability-negative, and actual WebMCP-support checks; record hashes, dynamic-library prerequisites, and artifact paths without committing generated bundles.
- Handoff: Remove ABW-15 and retain the dated Linux evidence in `crates/acorn-desktop/acceptance/linux.md`. Defects outside the write set open a bounded repair phase; a missing authorized host keeps this phase blocked.

## ABW-09 — Publish the browser and desktop workspace workflows and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-03 through ABW-08, accepted Home/chrome through ABW-32, ABW-13/14/15, ABW-22 through ABW-31, ABW-35/36, and ABW-56
- Exact write set: `Makefile`, `crates/acorn-web/README.md`, `crates/acorn-desktop/README.md`, `docs/src/packages/acorn-web.md`, `docs/src/packages/acorn-desktop.md` (new), `docs/src/SUMMARY.md`, `.wiki/features/acorn-browser-workspace/index.md` (delete after acceptance), `.wiki/features/acorn-browser-workspace/_assets/` (delete plan-specific previews after acceptance), `.wiki/features/acorn-browser-workspace/design-review.md` (delete after preserving durable design requirements), `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: browser/desktop build documentation and wiki retirement
- Outcome: Publish copy/paste browser and Tauri development/build workflows, including the coordinated Dioxus/Vite hot-reload loop, supported hot-reload versus full-reload boundaries, deterministic watcher shutdown, local serving and local-bundle prerequisites, static-host base paths, privacy and capability boundaries, Portfolio and supported-tool behavior, desktop host capability matrix and companion/renderer/service/agent prerequisites, WebMCP behavior, geospatial data/provenance/no-network behavior, ROR refresh procedure, browser/native evidence, measured WASM/map/dataset and bundle sizes, published install/update/removal commands, skill compatibility and rollback, signing/notarization status, and brand-token guidance; add stable focused Make targets; run final integration gates; remove the completed plan from every wiki surface.
- Focused validation: Clean focused and release Dioxus builds; `cargo test -p acorn-web`; `cargo check -p acorn-web --target wasm32-unknown-unknown`; run the documented `make web-dev` and `cargo tauri dev` hot-reload smokes and confirm watcher cleanup; `npm ci && npm test && npm run build` from `crates/acorn-web/geospatial`; deterministic ROR-index regeneration/digest check; `cargo test -p acorn-host --features fs`; `cargo test -p acorn-desktop`; `cargo check -p acorn-desktop`; dependency-tree checks for every forbidden edge in the crate-boundary contract; local `cargo tauri build --no-bundle`; inspect retained Windows/macOS/Linux native and all five public-channel evidence; verify source/CLI/browser/desktop/published skill hashes, manifest versions, upgrade/rollback/removal checks and Install availability metadata; `make test-webmcp`; `make test-webmcp-package`; `make build-wasm`; `make lint`; `make test`; `make check-wasm`; documentation build and link check; `git diff --check`; planning-wiki validation; verify generated files remain under ignored build output and no HMR client, React Refresh preamble, loopback URL, development-only CSP source, ion token, unapproved brand asset, signing material, or secret is committed.
- Handoff: Delete the plan and its index/network/lane entries after preserving all durable browser/desktop commands, published channel evidence, skill lifecycle, measured asset and bundle data, privacy/capability boundaries, signing and compatibility requirements, and brand status in product documentation and tests.

## ABW-35 — Accept local Dioxus hot reload

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: accepted browser build; exclusive browser staging ownership
- Exact write set: `crates/acorn-web/scripts/dev-smoke.mjs`, `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: browser development watcher acceptance
- Outcome: Observe representative RSX, CSS and static-asset hot reload in the local Dioxus development loop. The optimizer defect is resolved; the measured release payload and durable commands already appear in the browser README.
- Remaining evidence (November 13): map TSX/CSS Fast Refresh and coordinated watcher teardown are verified, while direct RSX/CSS/static reload observations remain unverified. No hosted deployment is needed for this local phase.
- Focused validation: Run the documented development coordinator and observe a reversible edit in each category, route/session behavior, watcher cleanup, and no development client in a release build. Preserve the accepted single-payload build.
- Handoff: Record observed watcher behavior in the README, remove ABW-35, and leave hosted deployment to ABW-36.

## ABW-36 — Accept mirrored GitHub Pages and HTTPS

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: a successful workflow build on the GitHub mirror and a provisioned Pages certificate; access to the hosted preview and its deployment record
- Exact write set: `crates/acorn-web/README.md`, `.wiki/features/acorn-browser-workspace/index.md`; external system: `jhwohlgemuth/acorn` GitHub Actions and Pages
- Primary integration lock: hosted browser deployment acceptance
- Outcome: Verify the first mirrored GitHub Pages deployment, anonymous route refresh, asset loading, and HTTPS enforcement against the actual hosted origin. Mirror run 43971897844 reached the browser smoke but timed out because WebMCP setup ran before its status element mounted. Waiting for that element and adding timeout diagnostics repaired the failure locally: `make web-test-pages WEB_BASE_PATH=/acorn` passed on November 16. The repaired workflow still needs to reach the mirror; hosted deployment and HTTPS remain unverified.
- Focused validation: Inspect the mirror commit and deployment result, open the live direct tool route, confirm HTTPS/certificate state and unexpected request count, and record the deployed revision and result.
- Handoff: Record live hosted evidence in the README, remove ABW-36, or retain the exact mirror/certificate blocker without holding local development acceptance.
