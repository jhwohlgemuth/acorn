---
okf_version: "0.1"
type: Plan
title: Frontend static analysis and release minification
description: Gate browser and shared desktop frontend builds on CSS, JavaScript and TypeScript analysis, then produce verified minified release assets.
difficulty: medium
impact: medium
package: [acorn-web, acorn-wasm]
tags: [frontend, static-analysis, css, javascript, typescript, minification, build]
status: planned
execution: phased
phase_budget_hours: 5
---

# Outcome

Every supported ACORN frontend build entry point checks authored CSS, JavaScript and TypeScript before generating or replacing build artifacts. Release builds deliver verified minified runtime assets with preserved browser behavior, module exports, license notices and asset URLs. Development retains readable sources and working hot reload. The future Tauri app consumes this same frontend build contract.

This plan's remaining type-checking and minification work covers `acorn-web` and its canonical `acorn-wasm/web` adapter. The accepted lint gate covers all authored repository JS/TS and CSS, including docs helpers, WebMCP examples, print styles and the design catalog. Rust-only Cargo checks remain usable without installing Node packages. Generated WASM bindings, vendored libraries and dependency files are excluded from authored-source lint. Generated and vendored runtime assets still appear in the release inventory with their processing owner and any explicit exception.

# Current evidence

On November 24, 2026, FBQ-OX1 and FBQ-CSS1 landed and were pruned. `make web-lint` runs pinned Oxlint 1.95.0 over 72 authored JS/TS files and Stylelint 18.1.0 with its recommended error preset over all six authored CSS files. File selection matched the repository inventory after generated/vendor exclusions. `make web-fix` exposes Oxlint's safe fixes and `make web-format` exposes pinned Oxfmt 0.80.0 as explicit source-writing actions; the existing formatting baseline is not yet Oxfmt-clean. A deliberate invalid CSS value stopped parallel `web-map`/`web-editor-bundle` before either bundle command; the restored parallel build, `make web-check`, and `make web-build` passed. Four browser acceptance tests, eight map tests, and 49 release browser parity checks passed. The release retained one WASM payload and nine operation exports. Separate TypeScript compiler coverage, minification, and final release acceptance remain open below.

Targeted inspection on November 12, 2026 used HEAD `0d0cc366` plus the existing uncommitted browser implementation. These are source observations, not a new passing build or browser acceptance run. The November 7 portfolio-wide currency review remains current.

- [Makefile](../../Makefile): `web-lint` is an ordered predecessor of `web-map` and `web-editor-bundle`; `web-build`, Pages, package, dev, check and test inherit the gate. The map build still installs its own npm lockfile and runs TypeScript/Vite.
- [Browser npm manifest](../../crates/acorn-web/package.json) pins the accepted lint tools. The [map manifest](../../crates/acorn-web/geospatial/package.json) pins TypeScript 7.0.2 and Vite 8.3.8; its build runs `tsc --noEmit`. The [TS project](../../crates/acorn-web/geospatial/tsconfig.json) includes `src`, leaving `vite.config.ts` outside compiler checking.
- [Vite configuration](../../crates/acorn-web/geospatial/vite.config.ts) emits an ES library, relies on default minifiers and disables production sourcemaps. Cesium resources are copied separately. The [release checker](../../crates/acorn-web/scripts/check-package.mjs) checks one WASM payload, absent debug sections and nine operation exports, but does not establish JS/CSS minification coverage.
- [Main asset declarations](../../crates/acorn-web/src/main.rs) and [UI asset declarations](../../crates/acorn-web/src/ui.rs) use Manganis for shell CSS/JS, vendor assets and the generated adapter. Installed `manganis-core` 0.7.10 `src/css.rs` and `src/js.rs` expose minification options that default to true; generic asset defaults use an unknown variant. **Unresolved:** which options the pinned Dioxus pipeline applies to each final artifact. Measure this before adding another transform.
- [Build script](../../crates/acorn-web/build.rs) and `web-prepare` copy the canonical WebMCP source into generated assets. A transform inserted at the wrong point can be overwritten by staging or invalidate already hashed asset references. Inline JavaScript in Rust `document::eval` strings is not covered by ordinary file linting and must remain an explicit boundary backed by runtime tests.
- GitHub [Pages CI](../../.github/workflows/acorn-web-pages.yml) and [GitLab CI](../../.gitlab-ci.yml) reach different Make entry points. Both must consume one gate; a local npm script alone is insufficient.

# Decisions

## Analysis tools and coverage

The accepted JS/TS/TSX correctness gate uses pinned **Oxlint** and the CSS gate uses pinned **Stylelint** with `stylelint-config-recommended`. The remaining phases extend the existing **TypeScript compiler** for type checking. Keep tool dependencies in the browser npm package and map-specific compiler dependencies in the map package; retain both lockfiles.

[Oxlint documents JS/TS/TSX support and correctness-focused defaults](https://oxc.rs/docs/guide/usage/linter.html). Use its non-type-aware rules initially; `tsc` remains the type-checking authority. This avoids adding a second type engine to the first rollout. [typescript-eslint currently publishes TypeScript support below 6.1](https://typescript-eslint.io/users/dependency-versions/), outside this repository's pin; do not downgrade TypeScript or suppress compatibility warnings to force adoption. Recheck versions during FBQ-01.

[Stylelint's recommended preset](https://github.com/stylelint/stylelint-config-recommended) enables rules aimed at likely errors. The standard preset produced 971 existing diagnostics, mostly formatting and selector ordering, so the accepted gate uses the recommended preset. Two selector-order/duplicate rules are disabled for intentional source-order theme and component overrides; one PDF margin-box background property has a local exception. Formatting migrations, selector purging and broad automatic fixes remain outside scope.

The initial source inventory is:

| Surface | Files | Required analysis |
|---|---|---|
| Shell | `crates/acorn-web/assets/effects.js`, `effects.test.mjs`, `app.css` | Browser/classic-script globals for effects; Node globals for tests; CSS correctness |
| Browser automation and packaging | Authored `crates/acorn-web/scripts/*.mjs` | Node correctness; explicit browser contexts inside Playwright callbacks |
| Map | `crates/acorn-web/geospatial/src/*.{ts,tsx,css}`, `vite.config.ts`, `tests/*.mjs` | TS/React correctness, separate browser and Node compiler environments, CSS and test lint |
| Shared adapter | `crates/acorn-wasm/web/webmcp.js`, `webmcp.d.ts`, `webmcp.test.mjs` | ES-module/browser lint, declaration checking and Node test lint |
| New frontend tooling | The lint/minification configurations and authored scripts added by this plan | Same appropriate checks, without generated-fixture false positives |

The accepted lint command discovers newly added authored files inside its named source directories. It excludes `node_modules`, generated assets, build outputs, Cesium copies and vendor files; the canonical adapter is checked once. Rust-embedded scripts remain under the separate `js-lint` feature.

## Minification ownership

Use existing build owners: Manganis for individual Dioxus shell/adapter assets and Vite for map JS/CSS. Make release options explicit after verifying the pinned implementation. Do not introduce a second general bundler merely to repeat an existing transform.

[Vite's build options](https://vite.dev/config/build-options.html#build-minify) distinguish JS and CSS minification and document that ES library mode preserves whitespace. The map is a final browser-consumed module: FBQ-04 must configure a fully minified final-consumer output using the pinned Vite APIs, preserve its public ES exports and relative Cesium resource paths, and verify the resulting bytes. A configuration flag alone is not acceptance.

Minify before asset hashing/copying and package assembly. Never rewrite final hashed assets in place. Keep canonical source files readable; record source-to-output ownership, byte sizes and hashes in a build report. Preserve externally accessed `window.acorn*` APIs, WebMCP exports, CSS custom properties, cascade order, media queries and required notices. No property mangling or unused-CSS removal. Retain current production sourcemap policy and readable development sources. Existing vendor minification and generated binding ownership are documented exceptions, not new source rewrite targets.

## Build contract

The accepted `make web-lint` runs JS/TS/CSS analysis before the existing browser build paths. The remaining work exposes `make web-typecheck` for compiler checks and `make web-preflight` for both. Dependency preparation is explicit and lockfile-based; analysis itself does not fetch packages or rewrite sources. Install each npm tree once per build invocation and fail clearly when tooling is unavailable.

The ordered release path is dependency preparation → preflight → map build/minification and source staging → Dioxus build/asset minification → artifact verification → Pages/package assembly. Encode real dependency edges, including direct `web-map` calls; test parallel Make execution. `web-check` and `web-test` include preflight; `web-dev` performs it before starting servers and retains HMR. Document that preflight runs at startup, with explicit reruns during editing; a continuous lint watcher is outside this initial scope. `web-prepare` remains a narrow Cargo-compatible source staging operation. Direct npm map builds must retain their analysis gate, too.

# Dependencies and boundaries

FBQ-01 → FBQ-02 → FBQ-04 → FBQ-05 → FBQ-06. Independent phase eligibility does not authorize concurrent edits to shared configurations.

Coordinate with [browser/desktop workspace](../features/acorn-browser-workspace/index.md) ABW-07 adapter staging, ABW-18 asset budgets and ABW-35 development/hosting acceptance. This plan owns analysis and minification mechanics; ABW retains product, accessibility, performance and native release acceptance. The future ABW-11 desktop consumer must invoke the accepted preflight/build path. None of those entire plans is a prerequisite for FBQ-01.

Serialize adapter/test changes with [WebMCP evaluation](webmcp-evaluations.md), especially WME-02/WME-04 tooling. Serialize Make/CI integration with [mise integration](../migrations/mise-toolchain-integration.md), [coverage throughput](coverage-throughput.md) and other CI/build owners. Native desktop availability and hosted deployment are not needed to establish these frontend gates.

All phases reserve this plan file for lifecycle maintenance. At each handoff, serialize updates to `.wiki/testing/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md` and the ABW dependency reference. Preserve unrelated working-tree changes. If a phase exceeds its five-hour boundary, split the remaining work before execution continues.

# Acceptance criteria

- A deliberate JS correctness error, TS type error and invalid CSS declaration each stop every applicable build entry point before output cleanup, bundling or packaging. The test also checks nonzero exit status and useful file/line diagnostics.
- Authored files and exclusions are enumerable, browser and Node environments are distinct, and map config/declarations are checked. The clean baseline requires no broad ignore, unsupported parser or silent warning suppression.
- Release runtime assets have a documented minifier or explicit generated/vendor exception. Controlled fixtures demonstrate minification; actual artifact checks verify parseability, expected exports, notices and source/output mapping. Record raw and compressed byte baselines; do not invent a percentage saving before measurement.
- Repeated asset transforms with identical source, tool versions and configuration are deterministic. Whole-package hashes may contain build metadata and are not falsely claimed deterministic.
- Shell behavior, WebMCP tools, map loading, themes, responsive navigation and relative asset URLs work against the release output. One WASM payload and nine operation exports remain enforced.
- Root and prefixed hosting, `localhost` and `127.0.0.1`, a custom frontend port and development HMR pass focused existing smoke tests. CI checks the same commands as local builds, with both lockfiles represented in cache keys.
- Durable commands, exclusions, tool ownership and measured asset results live in the browser README before this plan is retired. No new production implementation is authorized by creating this plan.

# Execution phases

## FBQ-01 — Inspect compiler coverage and asset processing ownership

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: frontend asset inspection lock available
- Exact write set: `crates/acorn-web/scripts/{frontend-quality.mjs,frontend-quality.test.mjs}`, `crates/acorn-web/README.md`
- Primary integration lock: frontend asset inventory
- Outcome: Inventory remaining TypeScript compiler gaps and per-artifact processing ownership. Preserve the accepted `web-lint` gate.
- Focused validation: Inspect actual release assets and pinned Manganis/Vite options; verify type-check project boundaries and identify generated/vendor exceptions without rewriting sources.
- Handoff: Freeze the compiler and artifact inventory in README; FBQ-02 becomes eligible. Unsupported tooling is a named blocker, not a waived gate.

## FBQ-02 — Extend TypeScript compiler coverage

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: FBQ-01
- Exact write set: `crates/acorn-web/geospatial/{package.json,package-lock.json,tsconfig.json,tsconfig.node.json,vite.config.ts}`, `crates/acorn-web/geospatial/src/{bridge.ts,home.ts,main.tsx,map.tsx}`, `crates/acorn-wasm/web/webmcp.d.ts`, `crates/acorn-web/tsconfig.webmcp.json`, `crates/acorn-web/scripts/{frontend-quality.mjs,frontend-quality.test.mjs}`
- Primary integration lock: browser JavaScript and adapter
- Outcome: Map sources, Node config and adapter declarations compile in appropriate TypeScript projects. Preserve the accepted Oxlint baseline without a source reformat.
- Focused validation: Repository JS/TS lint remains clean; all compiler projects and map bridge tests pass. Inject a type error in a temporary fixture and prove nonzero failure.
- Handoff: Record passing commands and justified narrow exceptions; freeze source/API compatibility for FBQ-04.

## FBQ-04 — Make release minification explicit and verifiable

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: FBQ-02; inspected pinned Manganis/Vite behavior from FBQ-01
- Exact write set: `crates/acorn-web/src/{main.rs,ui.rs,geospatial_page.rs}`, `crates/acorn-web/build.rs`, `crates/acorn-web/geospatial/vite.config.ts`, `crates/acorn-web/scripts/{stage-map.mjs,check-package.mjs,frontend-assets.test.mjs}`, `crates/acorn-web/frontend-assets.json`, `crates/acorn-web/README.md`
- Primary integration lock: frontend asset pipeline
- Outcome: Explicit processing owners produce minified shell, adapter and final map assets before Dioxus hashing, with a build report and tests for preserved public contracts. Existing generated/vendor exceptions are enumerated. Production source maps and development staging remain intentional.
- Focused validation: Controlled minification fixture, repeated identical transforms, parse/export checks, source-versus-built adapter contract tests and release package checker. Verify canonical adapter staging cannot overwrite transformed release output and copied Cesium URLs/notices remain valid.
- Handoff: Record raw/compressed baseline and deterministic-transform evidence; FBQ-05 may integrate only the verified path.

## FBQ-05 — Wire ordered local and CI pre-build gates

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: FBQ-04; CI/build lock available
- Exact write set: `Makefile`, `.github/workflows/acorn-web-pages.yml`, `.gitlab-ci.yml`, `crates/acorn-web/package.json`, `crates/acorn-web/geospatial/package.json`, `crates/acorn-web/scripts/{dev.py,frontend-quality.mjs,frontend-quality.test.mjs}`, `crates/acorn-web/README.md`
- Primary integration lock: CI/build
- Outcome: Complete the typecheck/preflight targets and direct map npm gate while preserving the accepted `web-lint` Make ordering. Both npm lockfiles and compatible Node/native tooling are prepared once; CI fails before uploading or assembling invalid artifacts. Rust-only Cargo checks acquire no npm dependency.
- Focused validation: Temporary-worktree fault injection at direct map npm, web-map/build/dev/check/test/pages/package entry points; sentinel output survives failure. Exercise `make -j` ordering with instrumented build commands, then one real clean release build. Validate CI syntax and changed-path/cache coverage; confirm direct Cargo check still works without frontend node_modules.
- Handoff: Record the command/dependency matrix and install behavior; FBQ-06 owns final acceptance.

## FBQ-06 — Accept the delivered artifacts and document the workflow

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: FBQ-05; isolated local ports and supported browser test engine available
- Exact write set: `crates/acorn-web/scripts/{acceptance.test.mjs,dev-smoke.mjs,pages-smoke.mjs,parity-smoke.mjs,frontend-assets.test.mjs}`, `crates/acorn-web/README.md`, `docs/src/packages/acorn-web.md`; plan/index/network/lane files named above for retirement
- Primary integration lock: frontend release acceptance
- Outcome: Existing browser smoke tests consume actual minified release output and cover root/prefixed paths, the adapter/map APIs, responsive/theme behavior and custom-port loopback development. Durable documentation explains checks, fixes, exclusions and artifact ownership.
- Focused validation: `make web-preflight`, `make web-check`, `make web-test`, `make web-pages`, `make web-test-pages`, release parity and development smoke; one serialized repository `make test` after implementation stabilizes; wiki validation. Preserve tests that compare canonical source copies, and separately validate transformed delivered artifacts rather than weakening source provenance assertions.
- Handoff: Record measured results and remaining platform limits outside the wiki. Acceptance failures become bounded repair phases. Once criteria pass, retire this plan and its index/network/lane entries, replacing the ABW reference with the durable build contract.
