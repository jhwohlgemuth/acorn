---
okf_version: "0.1"
type: Plan
title: Browser workspace runtime performance
description: Shrink the ROR and WASM payloads, and measure build, observer and caching changes for the deployed acorn-web app.
difficulty: high
impact: high
package: [acorn-web, acorn-wasm, acorn-schema]
tags: [browser, performance, wasm, web-worker, dioxus, geospatial]
status: planned
execution: phased
phase_budget_hours: 5
---

# Outcome

The deployed RAD Workspace stays responsive while any of its nine local operations runs, loads quickly on first and repeat visits, and opens the organization map without transferring and re-serializing the full ROR registry. Each change is measured against the release build served by `make web-pages`; no claimed saving is accepted without a recorded before/after number.

# Current evidence

Targeted review on November 14, 2026 at HEAD `1fe6df4d` plus the uncommitted performance pass described here. Native timings come from a release-profile scratch benchmark of `Identifier::find_all`; browser numbers come from `make web-test-pages`.

- **Landed in the same pass, not remaining work:** PID parsers use precompiled anchored regexes (`RE_*_EXACT` in [constants](../../crates/acorn-schema/src/util/constants/mod.rs), `regex_captures` in [acorn-core](../../crates/acorn-core/src/util/alloc/mod.rs)). On the bundled discovery example, native `find_all` fell from 84 ms to 10 ms; at the 16 KiB editor limit it fell from 556 ms to 11 ms. Discovery de-duplicates with sets, Discover rows no longer run one highlight `eval` each, diagnostic rows share `Rc` source/result values, raw JSON renders only when opened, the ROR index lives in app context and is read without cloning, `ToolWorkspace` reads signals instead of cloning `Request`/`Workspace`, the result-size guard is allocation-free, `index.html` shows a loading shell, the release build preloads the WASM, and the Pages smoke enforces a 3 s Discover budget at the editor limit; release builds measured 67 ms, and 77 ms with the busy-state frame yield, in Chromium.
- **Measured against the deployed site:** on November 14, 2026, Chromium ran the same Discover input five times, each on a fresh page, against `www.jasonwohlgemuth.com/acorn/` (old build) and the local release build. Both returned 36 identifiers. Medians: bundled example 258 → 117 ms (2.2×); 16 KiB editor limit 780 → 71 ms (11×). The example run is the first operation after load and includes one-time regex compilation. Load times (2.75 s live versus 76 ms on loopback) mix network, CDN and Cloudflare Rocket Loader effects, so they are not a code speedup. The live site's CodeMirror editor sometimes failed to mount after navigation, and one asset returned 404.
- **Operation worker landed (BWP-01/02):** a module worker ([worker.js](../../crates/acorn-web/assets/worker.js)) imports the same release loader and WASM; [main.rs](../../crates/acorn-web/src/main.rs) launches Dioxus only when `document` exists, so the release still ships one payload. The `runOperation` export wraps `operations::handle`. `execute` in [ui.rs](../../crates/acorn-web/src/ui.rs) sends each request through `window.acornOperations` in [effects.js](../../crates/acorn-web/assets/effects.js) and falls back in-thread when the worker cannot load. Home sample loading and Portfolio validation use the same path. WebMCP still calls the page's bindings. The release Pages smoke at `/acorn` recorded 5 worker-served operations, 63 ms for 16 KiB discovery and a longest main-thread frame gap of 17 ms; node tests cover fallback, load failure and trap restart. Against the deployed in-thread build (`dc5d79c7`), five fresh-page runs gave a first operation of 122 → 212 ms (the worker's WASM instance is still starting) and 71 → 89 ms at 16 KiB (median message overhead). The worker's second WASM instance roughly doubles memory while it runs.
- [operations.rs](../../crates/acorn-web/src/operations.rs) calls the `#[wasm_bindgen]` exports in [acorn-wasm](../../crates/acorn-wasm/src/lib.rs), which serialize to `JsValue` before `serde_wasm_bindgen` converts back to `serde_json::Value`; this now happens inside the worker.
- Release payloads at `75cc88ec` (raw / gzip -9 / brotli): WASM 6,786,150 / 2,300,331 / 1,592,431; `ror-locations.json` 22,282,929 / 4,880,305 / 2,624,487; `map.js` 4,998,435 / 1,325,894 / 1,027,597; `editor.js` 572,076 / 198,279 / 167,353. The shared release profile uses `opt-level = "z"`, `overflow-checks = true`, LTO and one codegen unit; `web-release` inherits it. [Dioxus.toml](../../crates/acorn-web/Dioxus.toml) sets no `wasm_opt` level.
- [geospatial_page.rs](../../crates/acorn-web/src/geospatial_page.rs) fetches the full registry, `JSON.parse`s it in JS, and returns it through `document::eval(...).join::<LocationIndex>()`, which serializes it again for Rust deserialization. Only referenced RORs are used.
- [syntax.js](../../crates/acorn-web/assets/syntax.js) observes `document.documentElement` with `subtree: true` and calls `querySelectorAll(".code-viewport")` on every DOM mutation outside line-number gutters.
- GitHub Pages serves gzip and cannot be configured for brotli or long-lived cache headers. Asset names are already content-hashed by Dioxus.

# Decisions

- **Measurement first.** Build-profile and WASM size work records raw, gzip and brotli bytes plus Discover/Validate timings before changing defaults. Readability or correctness regressions reject a faster setting.
- **Hosting.** Brotli and cache headers need a host other than GitHub Pages; that is an external decision, not a phase. BWP-08 provides repeat-visit caching that works on Pages.

# Dependencies and boundaries

BWP-03 is independent and reduces per-operation work in the worker and fallback. BWP-04 replaces the ROR dataset format and serializes with [ABW-18](../features/acorn-browser-workspace/index.md#abw-18--accept-map-resource-and-performance-budgets), which owns the ROR dataset and map budgets; BWP-04 lands first or as an ABW-18 input, not concurrently. BWP-05 and BWP-07 change build configuration and serialize with [frontend build quality](../testing/frontend-build-quality.md) FBQ-04/05 and other CI/build owners. BWP-06 owns only `syntax.js` and its tests. BWP-08 adds a service worker and serializes static-asset assembly with ABW-18/35 and FBQ-04.

WebMCP adapter behavior and its nine-tool contract are unchanged; coordinate any loader change with [WebMCP evaluations](../testing/webmcp-evaluations.md). Preserve unrelated working-tree changes.

# Acceptance criteria

- The map page transfers a recorded fraction of today's ROR bytes, opens in a measured time on the Pages smoke, and keeps the offline, no-network and accessible-table behavior.
- Every build or payload change records raw/gzip/brotli sizes and Discover/Validate timings before and after in `crates/acorn-web/README.md`.
- Durable measurements and decisions live in the browser README or tests before this plan is retired.

# Execution phases

## BWP-03 — Return results without the JsValue round trip

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none; serialize `crates/acorn-wasm/src/lib.rs` with WME owners
- Exact write set: `crates/acorn-wasm/src/lib.rs`, `crates/acorn-wasm/src/tests.rs`, `crates/acorn-web/src/operations.rs`
- Primary integration lock: acorn-wasm public bindings
- Outcome: acorn-wasm exposes Rust functions returning serializable results that both its `#[wasm_bindgen]` exports and acorn-web call, so acorn-web builds `serde_json::Value` directly. The JS export signatures are unchanged.
- Focused validation: `cargo nextest run -p acorn-wasm`, `make web-test-wasm`, `make test-webmcp`.
- Handoff: Record the measured Validate time change on the bundled examples.

## BWP-04 — Ship a compact ROR location index

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: not concurrent with ABW-18
- Exact write set: `crates/acorn-web/scripts/build-ror-location-index.mjs`, `crates/acorn-web/assets/geospatial/` (generated index files), `crates/acorn-web/src/geospatial.rs`, `crates/acorn-web/src/geospatial_page.rs`, `crates/acorn-web/README.md`
- Primary integration lock: ROR location dataset
- Outcome: The index keeps only name and first usable location, is split into shards by ROR identifier prefix, and the map fetches only the shards for referenced RORs. Results reach Rust without a second full serialization.
- Focused validation: `cargo test -p acorn-web` geospatial tests, `npm --prefix crates/acorn-web/geospatial test`, and the parity-smoke ROR map flow; record transferred bytes before and after.
- Handoff: Update provenance and regeneration steps; hand the measured budget to ABW-18.

## BWP-05 — Measure web-release optimization settings

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CI/build lock available
- Exact write set: `Cargo.toml` (`[profile.web-release]` only), `crates/acorn-web/Dioxus.toml`, `crates/acorn-web/README.md`
- Primary integration lock: CI/build
- Outcome: Compare `opt-level` `"z"`, `"s"` and `3`, overflow checks, and explicit `wasm_opt` levels for the web profile. Keep the setting with the best recorded Discover/Validate time within an agreed size increase, or keep the current defaults with the numbers that justify them.
- Focused validation: `make web-pages` and `make web-test-pages` per candidate; record sizes and timings.
- Handoff: Record the measurement table and chosen setting.

## BWP-06 — Scope the line-number observer

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none; serialize `syntax.js` with FBQ-02
- Exact write set: `crates/acorn-web/assets/syntax.js`, `crates/acorn-web/assets/syntax.test.mjs`, `crates/acorn-web/src/syntax.rs`
- Primary integration lock: browser syntax assets
- Outcome: Code blocks register their viewport explicitly, or the observer watches only code containers; unrelated DOM changes no longer trigger a document-wide query.
- Focused validation: `node --test crates/acorn-web/assets/syntax.test.mjs` and `node crates/acorn-web/scripts/syntax-smoke.mjs`.
- Handoff: Remove BWP-06.

## BWP-07 — Analyze and reduce WASM size

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BWP-05 setting recorded
- Exact write set: `crates/acorn-web/Cargo.toml`, `crates/acorn-web/README.md`; feature changes to other crates become separate bounded phases
- Primary integration lock: workspace graph
- Outcome: A `twiggy` report of the release payload names the largest contributors (schemas, vocabularies, regex engines, standards). Low-risk feature trims for the browser land here; larger ones become new phases with exact write sets.
- Focused validation: `make web-pages`, `make web-test-wasm`, `make web-test-pages`; record raw/gzip/brotli sizes.
- Handoff: Record the report summary and follow-up phases.

## BWP-08 — Cache hashed assets for repeat visits

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: not concurrent with ABW-18/35 or FBQ-04 asset assembly
- Exact write set: `crates/acorn-web/assets/service-worker.js` (new), `crates/acorn-web/assets/effects.js`, `crates/acorn-web/scripts/pages-smoke.mjs`, `crates/acorn-web/README.md`
- Primary integration lock: final static-asset assembly
- Outcome: A cache-first service worker for content-hashed assets and network-first `index.html`, scoped to the deployment base path, with old caches cleared on activation. It adds no network destinations.
- Focused validation: `make web-test-pages` with a second visit served from cache and a base-path deployment; confirm no unexpected requests.
- Handoff: Record repeat-visit timing and the update/rollback behavior; retire the plan when no phases remain.
