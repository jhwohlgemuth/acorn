---
type: Plan
title: Isolate browser JavaScript behind one audited script module
description: Move every acorn-web `document::eval` script into a single module with compile-time placeholder rules, typed result shapes, an optional JavaScript parse gate, and authored files for long bodies.
difficulty: medium
impact: high
package: [acorn-web, acorn-macros]
tags: [browser, javascript, safety, static-analysis, refactor, boundary]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal and boundary

`acorn-web` composes JavaScript as Rust strings and hands it to `dioxus::document::eval`. These strings are executable code in the page's own origin, but they are the only executable surface in the repository with no parser, no linter, no type checking, no test coverage, and no compile-time rule about interpolation. Rust string literals and `format!` are the wrong level of fear for them.

Give every one of these scripts a single owner: one `script` module in `acorn-web` that is the only code permitted to name `document::eval`. Inside it, each script is a named item with a declared result shape, its placeholders are bound through a compile-time-checked macro that can only emit JSON values, and long bodies live in authored `.js` files that ordinary frontend tooling can read. Add an optional, host-only JavaScript parse gate over the whole catalog, mirroring the existing `shell-lint` feature in `acorn-cmd`.

This plan changes where scripts live and how they are built, not what they do. Preserve current browser behavior, the `window.acorn*` runtime contract, every route, operation, and persisted preference. The runtime asset scripts (`assets/*.js`), the CodeMirror bundle, the map island, the WebMCP adapter, and minification ownership stay with their existing plans. The two `dangerous_inner_html` sites are an HTML sink, not a script sink, and remain outside this plan.

# Current evidence — November 23, 2026

- **Verified repository**, `crates/acorn-web/src` at `09194049` plus the current uncommitted browser worktree: **101** `document::eval` call sites across eight binary modules — `ui.rs` (40), `workspace.rs` (20), `syntax.rs` (14), `details.rs` (10), `main.rs` (7), `skill.rs` (4), `geospatial_page.rs` (4), `results.rs` (2). By construction: **63** take a static string literal, **35** take `&format!(...)`, and **3** take a separately built `&script` binding. These are source counts from the current tree, not a behavioral run.
- **Verified repository:** by result shape, **70** sites drop the returned evaluator (fire-and-forget), **25** call `.join::<T>()` for a typed value, and **6** open a `dioxus.send`/`dioxus.recv` channel bridge. The 70 dropped evaluators mean a malformed script, a renamed host function, or a thrown exception produces no Rust-side error and no failing test. **Unresolved:** the exact failure behavior of a dropped `Evaluator` in the pinned Dioxus 0.7.10; BSI-01 reads the pinned source and records it rather than assuming it.
- **Verified repository:** the scripts reference **19** distinct `window.acorn*` host APIs — `acornCelebrate`, `acornChrome`, `acornCopy`, `acornDashboardLayout`, `acornDetails`, `acornDownload`, `acornEditorSyncVersions`, `acornEditors`, `acornInitialReviewFragment`, `acornMap`, `acornOperations`, `acornPendingEditorReveal`, `acornRemote`, `acornSkill`, `acornSyntax`, `acornTabId`, `acornUrl`, `acornWorkspace`, `acornWorkspaceStorage` — defined across `assets/effects.js`, `details.js`, `remote.js`, `syntax.js`, `url-state.js`, `workspace-storage.js`, `dashboard-layout.js`, and `editor/editor.js`. Nothing checks that a name spelled in a Rust string still exists in the asset that defines it; the optional-chaining style (`window.acornChrome?.…`) converts a renamed API into silence rather than an error.
- **Verified repository:** the safe interpolation habit is already present and dominant — `serde_json::to_string(&args)` followed by a `...{args}` spread or a destructuring assignment appears throughout (`syntax.rs:8,63`, `skill.rs:46,76,158`, `workspace.rs:291,641,657`, `details.rs:257`, `results.rs:580`, `main.rs:78-79`, `geospatial_page.rs:55`). It is a convention, not an enforced rule.
- **Verified repository:** **15** sites instead interpolate a bare placeholder *inside* a JavaScript string or attribute literal — `syntax.rs:72,73,74,75,77,78,81,86,88,96`, `ui.rs:533,876,1139,1304`, `workspace.rs:641`. Every one currently carries a constrained value: `CodeEditor`'s `id: &'static str`, `ChoiceField`'s `label: &'static str`, a `Tool::slug()` enum discriminant, a `match`-produced literal fragment, a numeric line/character offset, or a `&'static str` notification kind. None is attacker-controlled today. That safety rests entirely on reading the surrounding Rust; a future prop widened from `&'static str` to `String` would silently turn one of these into a quote-breaking injection with no diagnostic. This is precisely the shape `cmd!` already rejects at compile time.
- **Verified repository:** `crates/acorn-macros/src/command.rs:47-133` parses a `cmd!` literal at compile time and fails the build on `interpolation must occupy an unquoted shell word`, `interpolation must occupy a full shell word`, unterminated quotes, and trailing escapes; `expand_cmd_sh_words` then emits each placeholder through `AsRef<OsStr>` so a value can never re-enter the grammar as syntax. `crates/acorn-macros/tests/ui/` holds the compile-fail cases. This is a working, tested precedent in this repository for the rule this plan applies to JavaScript.
- **Verified repository:** `crates/acorn-cmd/Cargo.toml:30` defines `shell-lint = ["dep:shuck-linter", "dep:shuck-parser"]`, off by default; `src/runtime.rs:131-166` parses the script with the matching dialect, lints it, filters by a severity floor read from `ACORN_SHELL_LINT_MIN_SEVERITY`, and returns a rendered `io::Error` capped at five diagnostics; `Makefile:202` checks the feature in `check-host`, and `crates/acorn-cli/Cargo.toml:28` enables it for the host binary. The optional-feature shape keeps a heavy native analyzer out of the default graph. The same shape keeps a JavaScript parser out of the `wasm32-unknown-unknown` payload.
- **Verified repository:** the embedded-asset precedent is `crates/acorn-web/build.rs` staging canonical sources into `assets/generated/` with `cargo:rerun-if-changed` on both ends, `asset!()` declarations in `main.rs:156-175` and `syntax.rs:4`, and `rust_embed::Embed` folders in `acorn-cli` and `acorn-schema`. Authored script files get rebuild tracking, a single canonical copy, license notices, and node test coverage (`assets/*.test.mjs`); inline Rust strings get none of it.
- **Verified repository:** [frontend build quality](../testing/frontend-build-quality.md) records in its own evidence that "Inline JavaScript in Rust `document::eval` strings is not covered by ordinary file linting and must remain an explicit boundary backed by runtime tests." FBQ-02 adds pinned Oxlint over authored `.js`/`.mjs`/`.ts` files and explicitly declines to claim `.rs` strings are linted. This plan fills that documented gap from the Rust side and feeds FBQ newly authored files rather than competing with it.
- **Verified repository:** `clippy.toml` currently holds only `allow-unwrap-in-tests` and `allow-expect-in-tests`; there is no `disallowed-methods` list, and `[workspace.lints.clippy]` in `Cargo.toml` does not enable `disallowed_methods`. **Unresolved:** `disallowed_methods` has no module-scoped exemption, so the owning module needs one documented `#[allow]`. BSI-01 confirms that the lint fires on the Dioxus path as written before BSI-04 depends on it.
- **Verified repository:** `cargo test -p acorn-web` runs on the host (`Makefile:546`) and `make web-check` already runs host Clippy alongside the `wasm32-unknown-unknown` check (`Makefile:493-497`), so a host-only catalog test and an optional host-only parse gate are both reachable from the existing targets. No new runner or CI entry point is required.
- **Unresolved:** which JavaScript parser crate to pin for the gate, and whether it parses the dialect actually used here (top-level `await`, optional chaining, logical assignment `||=`, `catch {}` without a binding). All four appear in current scripts — `ui.rs:461`, `ui.rs:541`, `main.rs:78`, `ui.rs:599`. BSI-01 freezes the choice against the workspace license allowlist in `deny.toml` and rejects any candidate that cannot parse the existing catalog.

# Design contract

1. **One owner.** A `script` module in `acorn-web` is the only code that names `document::eval`. Every other module calls a named function on it. Call sites stop containing JavaScript syntax, so a reader of `ui.rs` sees `script::focus_content_editor()`, not a selector string. Enforce the boundary with `clippy::disallowed_methods`, with exactly one documented `#[allow]` inside the module.
2. **Declared shape.** Each script states what it is: fire-and-forget, a query returning `T`, or a channel bridge. The shape is part of the item, not an incidental consequence of whether a caller remembered `.join::<T>()`. Fire-and-forget stops meaning "errors are invisible": the module logs or surfaces a failure in one place rather than at 70 sites.
3. **Placeholders are values, never syntax.** A `js!` macro parses the literal at compile time and rejects a `{placeholder}` that falls inside a string literal, template literal, comment, or regular expression — the direct analogue of `cmd!`'s "interpolation must occupy an unquoted shell word". Every binding is emitted as a `serde_json` value, so interpolated data can only ever land as a JavaScript value. Rust types already constrained to `&'static str` stay constrained; the rule stops relying on that being true.
4. **Long bodies are files.** A script body above the threshold frozen in BSI-01 lives in an authored `assets/scripts/*.js` file, read at compile time with rebuild tracking, and takes its data through a single JSON argument rather than textual placeholders. These files enter FBQ's Oxlint surface and can carry their own node tests, exactly as `assets/effects.js` does today. Short one-expression scripts stay inline as named items; moving `window.acornDetails?.close()` into a file would be ceremony, not safety.
5. **A real parser, off by default.** An optional `script-lint` feature parses every catalog entry with a pinned JavaScript parser and fails on syntax errors and the configured diagnostic floor, mirroring `acorn-cmd`'s `shell-lint` down to the severity environment variable and capped error rendering. It is host-only and never enters the `wasm32-unknown-unknown` graph or the shipped WASM payload.
6. **Behavior is preserved, not assumed.** Every phase that moves call sites ends with the existing browser checks, not with a compile. A script that parses is not a script that works.

# Dependencies and decisions

- Phase order is BSI-01 freeze → BSI-02 macro and module → BSI-03 fire-and-forget and quoted-placeholder sites → BSI-04 query and channel sites plus the Clippy boundary → BSI-05 authored files → BSI-06 parse gate → BSI-07 acceptance. BSI-03 and BSI-04 are serial because both edit the same eight modules and the shared `script` module.
- Share the browser Rust/UI lock with [browser workspace](../features/acorn-browser-workspace/index.md), [shadcn-style component transition](../migrations/acorn-web-shadcn-components.md), [browser performance](../refactors/browser-performance.md), [optional USWDS theme](../features/uswds-browser-theme.md), and [frontend build quality](../testing/frontend-build-quality.md). BSI-03 and BSI-04 touch `ui.rs`, `workspace.rs`, `syntax.rs`, `details.rs`, `results.rs`, `skill.rs`, `geospatial_page.rs`, and `main.rs`, which several of those plans also edit. Confirm exact write-set ownership before starting either phase; do not run BSI-03 or BSI-04 concurrently with a phase that rewrites the same component bodies.
- FBQ is a consumer, not a prerequisite. BSI-05 adds authored `.js` files that FBQ-02's discovery rule should pick up automatically; coordinate the glob with FBQ-01's source-inventory decision if that phase has already run. Neither plan blocks the other, and this plan does not add npm to any Rust-only check.
- BSI-02 adds an `acorn-macros` dependency to `acorn-web`. It is a proc macro with no runtime component and no effect on the WASM payload. The compile-time file read in BSI-05 requires a `proc-macro` crate reading relative to `CARGO_MANIFEST_DIR` and emitting `include_str!` for rebuild tracking; confirm that pattern against the existing macro tests before relying on it.
- The parser crate for BSI-06 is deliberately not named here. BSI-01 selects it against `deny.toml`'s license allowlist, the workspace MSRV, and the dialect features the current catalog actually uses, and records why the rejected candidates failed. Do not add a general bundler, a Node step, or a second type engine to satisfy this gate.
- This plan does not change the `window.acorn*` runtime contract, add a network dependency, alter persisted keys, or claim that the browser's JavaScript is now type-checked. A parse gate establishes that a script is syntactically valid JavaScript and that its placeholders cannot escape a value position. It does not establish that a referenced host API exists; BSI-06 adds a name cross-check against the defining assets as a separate, explicitly bounded check.

# Acceptance criteria

- No module outside `acorn-web`'s `script` module names `document::eval`, and `clippy::disallowed_methods` fails the build if one does. The single exemption inside the module carries a comment explaining the boundary.
- All 101 current scripts are named items in the catalog with a declared shape. Each of the 25 typed queries and 6 channel bridges keeps its existing deserialization target, and each of the 70 previously silent sites has a defined failure behavior recorded in the module.
- The `js!` macro rejects, with a compile error naming the rule, a placeholder inside a JavaScript string, template literal, comment, or regular expression, and rejects a binding that is not serializable. `trybuild` compile-fail cases cover each rule, in the style of `crates/acorn-macros/tests/ui/`.
- Every script body above the frozen threshold is an authored file under `crates/acorn-web/assets/scripts/`, tracked for rebuilds, taking data through a single JSON argument, and visible to the repository's frontend lint surface.
- `cargo check -p acorn-cmd --features shell-lint` style parity holds: `cargo check -p acorn-web --features script-lint` succeeds on the host, the feature is absent from the default and `wasm32-unknown-unknown` graphs, and a deliberately broken catalog entry fails the gate with a capped, readable diagnostic.
- Behavior is unchanged: `make web-check`, `cargo test -p acorn-web`, `make web-test`, `make web-build`, and `make web-test-pages` pass, and the built payload size is compared against the pre-refactor baseline with any delta explained.

# Execution phases

## BSI-01 — Freeze the catalog, result shapes, and tool choices

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: stable local snapshot of `crates/acorn-web/src`; confirm current browser write owners
- Exact write set: `.wiki/_assets/browser-script-inventory.json`, `.wiki/refactors/browser-script-isolation.md`
- Primary integration lock: browser script specification
- Outcome: Record every `document::eval` site as a catalog row — file, line, shape, deserialization target where present, each interpolated binding with its Rust type and how it is encoded, and the `window.acorn*` names it touches with the asset file that defines each one. Read the pinned Dioxus 0.7.10 `eval` implementation and record what a dropped `Evaluator` actually does. Freeze the inline-versus-file threshold, the macro's name and crate placement, the candidate JavaScript parser with its license and dialect check against the existing catalog, and whether `clippy::disallowed_methods` fires on the Dioxus path as written. Name the rejected alternatives and why.
- Focused validation: Reconcile the catalog row count against `rg` counts for the current tree; trace every listed `window.acorn*` name to a defining asset; parse a representative sample of current script bodies with each candidate parser, including the top-level `await`, `?.`, `||=`, and bare `catch {}` cases cited in the evidence; confirm the chosen crate's license appears in `deny.toml`.
- Handoff: Record the baseline revision, catalog, threshold, parser decision, and the Clippy and dropped-evaluator findings; remove BSI-01 and make BSI-02 ready.

## BSI-02 — Add the `js!` macro and the script module skeleton

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BSI-01
- Exact write set: `crates/acorn-macros/src/lib.rs`, `crates/acorn-macros/src/script.rs`, `crates/acorn-macros/src/tests.rs`, `crates/acorn-macros/tests/script.rs`, `crates/acorn-macros/tests/ui/`, `crates/acorn-web/Cargo.toml`, `crates/acorn-web/src/script/mod.rs`, `crates/acorn-web/src/main.rs`
- Primary integration lock: browser script module
- Outcome: Add a compile-time literal parser in `acorn-macros` modeled on `parse_cmd_words`, rejecting a placeholder inside a JavaScript string, template literal, comment, or regular expression, and emitting each binding through `serde_json` so it can only land as a value. Add the `script` module declaring the fire, query, and channel shapes with their failure behavior in one place. Migrate no call sites in this phase; port two or three representative scripts only to prove each shape.
- Focused validation: `cargo test -p acorn-macros` including new `trybuild` compile-fail cases for each rejection rule; `cargo check -p acorn-web` and `cargo check -p acorn-web --target wasm32-unknown-unknown`; confirm the ported scripts still run in a local browser session.
- Handoff: Record the accepted and rejected placeholder grammar with its test names; remove BSI-02 and make BSI-03 ready.

## BSI-03 — Move the fire-and-forget and quoted-placeholder sites

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BSI-02; no concurrent owner of the eight browser modules
- Exact write set: `crates/acorn-web/src/script/mod.rs`, `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/src/syntax.rs`, `crates/acorn-web/src/details.rs`, `crates/acorn-web/src/results.rs`, `crates/acorn-web/src/skill.rs`
- Primary integration lock: browser UI script call sites
- Outcome: Convert the fire-and-forget sites and all 15 quoted-placeholder sites into named catalog items. The quoted placeholders become typed bindings; where a value is genuinely a DOM identifier or enum discriminant, keep the narrow Rust type and pass it as a value rather than splicing it into a selector. Call sites in the seven modules no longer contain JavaScript syntax.
- Focused validation: `make web-check`; `cargo test -p acorn-web`; `node --test` over the affected asset tests; a local browser pass covering theme choice, sidebar collapse, editor toolbar actions, details open and close, downloads, and notifications — the behaviors behind the converted sites.
- Handoff: Record converted site count against the BSI-01 catalog and any behavior difference found; remove BSI-03 and make BSI-04 ready.

## BSI-04 — Move the query and channel sites and close the boundary

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BSI-03
- Exact write set: `crates/acorn-web/src/script/mod.rs`, `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/src/main.rs`, `crates/acorn-web/src/geospatial_page.rs`, `crates/acorn-web/src/details.rs`, `clippy.toml`, `Cargo.toml`
- Primary integration lock: browser UI script call sites
- Outcome: Convert the 25 typed queries and 6 channel bridges, preserving each deserialization target and the existing send/recv lifecycle, including workspace restore and save, tool broadcast, remote loading, storage status, review-fragment sync, map connect and update, and dashboard layout. Then enable `clippy::disallowed_methods` for the Dioxus eval path with the one documented module exemption.
- Focused validation: `make web-check` with the new lint active; `cargo test -p acorn-web`; `make web-test`; a two-tab browser pass covering workspace save and restore, cross-tab broadcast, storage-change handling, map selection, and remote repository loading.
- Handoff: Record that no module outside `script` names `document::eval` and that the lint fails when one does; remove BSI-04 and make BSI-05 ready.

## BSI-05 — Move long script bodies into authored files

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BSI-04; coordinate with FBQ-01 if its source-discovery rule has landed
- Exact write set: `crates/acorn-web/assets/scripts/`, `crates/acorn-web/src/script/mod.rs`, `crates/acorn-macros/src/script.rs`, `crates/acorn-macros/tests/script.rs`, `crates/acorn-web/assets/README.md`
- Primary integration lock: browser authored script assets
- Outcome: Move every body above the BSI-01 threshold into `assets/scripts/*.js`, read at compile time with `include_str!`-backed rebuild tracking and applying the same parse rules as the inline macro. These files take data through one JSON argument and carry no textual placeholders. Record the file set and its tooling ownership in the assets README.
- Focused validation: `cargo check -p acorn-web` and a touch-then-rebuild check proving an edited `.js` file actually triggers recompilation; `make web-check`; `make web-test`; browser confirmation of the moved scripts, which are the workspace storage, broadcast channel, remote loading, and editor sync bridges.
- Handoff: Record the moved file list, the rebuild-tracking evidence, and the glob FBQ should adopt; remove BSI-05 and make BSI-06 ready.

## BSI-06 — Add the optional `script-lint` parse gate

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BSI-05; parser crate frozen in BSI-01
- Exact write set: `crates/acorn-web/Cargo.toml`, `crates/acorn-web/src/script/lint.rs`, `crates/acorn-web/src/script/mod.rs`, `Cargo.toml`, `Makefile`, `deny.toml`
- Primary integration lock: browser script lint feature
- Outcome: Add a default-off `script-lint` feature that parses every catalog entry, inline and authored, and fails on syntax errors and the configured diagnostic floor, following `acorn-cmd`'s `shell-lint` shape including the severity environment variable and five-diagnostic capped rendering. Add a host test that enumerates the catalog so a new script cannot skip the gate, plus a bounded cross-check that each referenced `window.acorn*` name appears in a defining asset. Wire the feature check into `web-check` beside the existing Rust checks.
- Focused validation: `cargo test -p acorn-web --features script-lint`; `cargo check -p acorn-web --target wasm32-unknown-unknown` confirming the parser is absent from that graph; a deliberately broken catalog entry and a deliberately misspelled host API each failing with a readable message; `make audit` and `cargo deny check licenses` for the new dependency.
- Handoff: Record the gate's coverage, its explicit limits, and the measured host check time; remove BSI-06 and make BSI-07 ready.

## BSI-07 — Accept the boundary and document it

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BSI-06; stable shared browser tree
- Exact write set: `crates/acorn-web/README.md`, `crates/acorn-web/src/script/mod.rs`, `ARCHITECTURE.md`, `.wiki/refactors/browser-script-isolation.md`
- Primary integration lock: browser script acceptance
- Outcome: Run the full browser matrix against the refactored tree and document the boundary: where scripts live, how to add one, which rules the macro enforces, how to enable the lint feature, and what the gate does not establish. Compare the built payload against the pre-refactor baseline and explain any size delta.
- Focused validation: `make web-check`, `make web-test`, `make web-build`, `make web-test-pages`, and `make web-test-engines`; payload and asset size comparison against the BSI-01 baseline revision; a browser pass over the routes exercising the converted scripts.
- Handoff: If every acceptance criterion holds, preserve the durable boundary description in the README, `ARCHITECTURE.md`, and the module's own documentation, then retire this plan with its index, network, and lane entries. If a concrete defect remains, add one bounded repair phase with its exact write set.
