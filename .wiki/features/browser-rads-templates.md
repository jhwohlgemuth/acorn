---
type: Plan
title: Browser RADS project templates and editor URL intake
description: Let web and desktop users start from validated bundled RADS projects or a bounded URL supplied editor draft.
difficulty: medium
impact: high
package: [acorn-web, acorn-schema, acorn-desktop]
tags: [browser, desktop, rads, templates, editor, deep-links]
status: planned
execution: phased
phase_budget_hours: 5
---

# Outcome

Offer a small catalog of pre-populated Research Activity Data Schema (RADS) projects that users can open, edit, validate, and export immediately in the shared Dioxus frontend. Put a styled template dropdown in the controls bar of the code editor opened in a new tab, so that users can start a project without returning to Home. Accept bounded editor text from a URL search parameter, with the same behavior in the hosted browser and bundled desktop webview. Template links use stable IDs instead of putting an entire project in the URL.

# Current evidence — November 24, 2026

- At `4fc72c19` plus concurrent uncommitted browser changes, `crates/acorn-web/src/ui.rs:277–303` has one Home **Try Sample Data** action. It loads `sample::files()` into `Workspace::replace`, calls `validate_files`, selects an activity, and opens Portfolio. `src/sample.rs:4–170` embeds a 40-file fictional bucket and tests its records. This is a demonstration dataset, not a catalog of editable starter projects.
- `src/session.rs:12–33,65–124` defines a 500-file, 2 MiB-per-file, 32 MiB-total content boundary, normalizes supplied paths, and makes only explicitly validated records visible through `Workspace::activities`. `src/main.rs:28–80` restores and revalidates the saved workspace before routing; subsequent workspace changes trigger persistence. Template imports must enter this same path and must not claim validity from a successful JSON parse alone.
- `src/operations.rs:63–102,116–135,266–283` holds user-owned tool form text in `Request.input` and serves intentionally flawed examples from `example_request`. `src/ui.rs:838–910,974–991,1094–1105` synchronizes session content into tool drafts and supports local files, URL-loaded files, examples, and direct editor changes. The existing examples are operation teaching material; they are not project templates.
- `assets/url-state.js:1–56` already exposes `window.acornUrl.getParameters`, `setParameters`, `getFragment`, `setFragment`, and `subscribe`. Its subscription observes `popstate`, `hashchange`, and its own `acorn-url-change` event, deduplicated by full `href`; `setParameters` preserves path, fragment, and other parameters. The existing `assets/url-state.test.mjs:1–46` verifies these behaviors under `/acorn/review`. `assets/scripts/review-fragments-channel.js:1–22` and `src/script/mod.rs:384,418` consume that helper for Review tab fragments. `src/ui.rs:53–59` routes `/` and `/:page`; `src/main.rs:147–160` mounts the router after storage recovery. `src/ui.rs:1180–1230` has a public HTTPS file URL loader with an explicit Load action and bounded result handling. No observed URL parameter currently initializes `Request.input` or selects a project template.
- `crates/acorn-web/README.md:187–207` documents root and `/acorn/` Pages builds, direct-route fallback, one WASM payload, and worker-backed validation. The [desktop workspace plan](acorn-browser-workspace/index.md#abw-11--embed-the-shared-dioxus-workspace-in-a-minimal-tauri-shell) places the same static frontend in Tauri; `crates/acorn-desktop/` is not yet present. Desktop acceptance therefore follows ABW-11 rather than creating another frontend.
- `src/syntax.rs:34–104` owns the shared `CodeEditor` toolbar; its **Open code editor in new tab** control is at line 78. `src/ui.rs:856–860,1121–1130` identifies editor popouts and passes the Review editor value through `Request.input`; `editor/popout.js:4–50` creates the popout using `acorn-editor` URL/window state; `src/window_sync.rs:1–69` carries tool snapshots across windows. `assets/app.css:958–1023,1113–1129` already styles toolbar, popout, fullscreen, and narrow layouts. The requested dropdown should be a Rust-rendered toolbar control with one shared project-creation action, not a separate CodeMirror or popout-only data store.

These are inspected source observations. No template or URL-intake test has run for this proposal. Existing dirty `acorn-web` and `.wiki` files are concurrent work; implementation phases claim their named write sets only after those owners release them.

# Product contract

1. **Catalog.** Start with three distinct, fictional, locally bundled starter projects: a minimal single-activity RADS file, a multi-activity publication project, and a collaboration/project narrative. Each card states its purpose, file count, included fields, and what the user should replace. The exact record contents are authored against current `acorn-schema` validation before release. Keep the existing broad fictional sample as a separate demo action.
2. **Open and edit.** Selecting a template shows a preview and file list; **Create from template** copies its files into a new local workspace, validates them through the existing worker, selects the first RADS file, and opens its source in Review. The template remains immutable. A failed load leaves the prior workspace untouched. If the current workspace contains files or unsaved editor text, show an explicit replacement choice before changing it. The new project is saved by the existing workspace persistence path. Do not silently overwrite a remote bucket or another local project.
3. **Validity.** Each shipped starter is a real valid RADS project under the current schema and browser validation operation: all included RADS files report `valid: true`, no unsupported or skipped files, and Portfolio shows the expected activity count. Placeholder values must themselves be schema-valid, visibly fictional, and documented on the card. Validation after user edits is normal; a user can make the draft invalid while working.
4. **Exit paths.** Users can edit, revalidate, copy/download a file, and download the complete project using existing bounded browser effects or an explicit new local ZIP export if no project export exists. Desktop uses the shared UI initially; optional native save is a separate explicit action through ABW-12/17, never automatic disk mutation. The template catalog works offline without a service or credentials.
5. **URL search contract.** `?code=<URL-encoded UTF-8 text>` on a supported tool route (initially `/review`) initializes one *editor draft*, selecting RADS/JSON defaults for Review. Cap decoded bytes at the existing tool input limit and the encoded URL at a conservative documented limit (proposed 8 KiB); reject duplicate `code`, malformed encoding, empty text, and conflicts with `template`. A rejected link displays an accessible error and preserves current content. No execution, validation, persistence, or workspace replacement occurs merely because `code` is present. **Run check** or **Create project from draft** is explicit. The draft wins only on first navigation to that URL; ordinary rerenders and storage restore must not clobber subsequent edits. Changing to a distinct URL must not overwrite a dirty draft without a choice.
6. **Template links.** `?template=<stable-id>` opens the catalog entry/preview. **Create from template** is explicit; it uses bundled files, not remote fetch or a serialized project in the query. Unknown IDs show the catalog with a non-destructive error. Generate and clear catalog links with the existing `window.acornUrl.setParameters` so path, Review fragment, and unrelated search parameters including `theme` survive. Support both `/` and `/acorn/` deployments. Define the precedence of query, route, saved workspace, editor popout state, and cross-tab sync in a focused test matrix before wiring effects.
7. **Remote source links.** `?url=<encoded HTTPS file URL>` may prefill the existing File URL field on Review, but the user must click **Load URL**. It inherits that loader's size, redirect, CORS, and cancellation rules. Do not add automatic network reads, arbitrary protocols, credentials, local paths, or a desktop IPC bridge to URL startup. `code`, `template`, and `url` are mutually exclusive; surface conflicts rather than silently picking one.
8. **New-tab editor dropdown.** In the Review `content` editor opened by **Open code editor in new tab**, add a visibly styled **Start from template** dropdown in `.editor-toolbar`. It lists the same three catalog IDs with short descriptions and a placeholder choice. Selecting a template uses the catalog's preview/replacement confirmation and **Create from template** path: it loads the full project into the bounded workspace and shows the selected primary RADS file in the editor. It must not copy only the primary file or edit CodeMirror directly. Cancel leaves the editor and workspace unchanged; successful creation updates the opener and popout through existing workspace/tool synchronization. The dropdown remains keyboard and screen-reader usable, fits narrow popout widths and fullscreen, and is absent from unrelated editors (`after`, assumptions, contact, details). A `?template=` link may preselect its choice, but opening a popout from that URL must not create the project twice. Do not write a new `?template=` parameter merely because the dropdown selection changes; offer a share link explicitly if needed later.

## Acceptance matrix

| Surface | Required observation |
|---|---|
| Catalog | Keyboard and screen-reader reachable cards/preview; all three projects validate and show expected Portfolio counts; existing sample still works. |
| Hosted web | Direct and refreshed root and `/acorn/` URLs for `template`, `code`, and `url`; no unexpected network requests for bundled templates or inline code. |
| Editor | Unicode/reserved-character round trip, empty/duplicate/conflicting/oversized input rejection, dirty-draft protection, browser Back/Forward, tab switch, popout, restored-workspace behavior, and preservation of Review `#fair`/`#readability` fragments. |
| New-tab toolbar | Dropdown appears beside existing editor controls only in the Review content popout; keyboard and screen-reader selection, narrow/fullscreen layout, cancel/no-change, full-project creation, opener/popout synchronization, and no duplicate creation from inherited `?template=` all pass. |
| Persistence/export | Explicit create replaces only after confirmation, saves/reloads the resulting workspace, never stores the URL text as template provenance, and exports files that pass the same RADS check. |
| Desktop | After ABW-11, packaged Tauri window opens the catalog and routes query links through the same frontend; explicit native save is checked only after ABW-12/17. |

# Boundaries and dependencies

- Reuse portable RADS validation and `Workspace::replace`; no schema extension, remote template API, generic URL fetch, or new server is in scope. Do not confuse project templates with CLI report templates, prompt templates, or intentionally invalid operation examples.
- Extend `window.acornUrl` instead of introducing another `location.search` reader, history wrapper, or URL event bus. Read repeated values through `getParameters`, listen through its existing `subscribe`, and write template links through `setParameters`. Add only a small raw-encoding check to that helper if `URLSearchParams` cannot distinguish malformed percent escapes; keep decoding/byte-policy decisions in a pure bounded function. Leave the current `subscribe(fragment)` callback contract intact for `review-fragments-channel.js`, and add a separate named Dioxus channel that consumes the same helper if needed. Browser location/history access belongs to the existing `script` boundary; editor updates belong to Dioxus `Request` signals. Avoid driving CodeMirror directly from query JavaScript, since `Request.input` owns the source of truth.
- Template creation and query startup serialize with ABW-19/63–65 on workspace state, SCU and USW on shared UI/CSS, FMT on Review route/editor, FBQ on assets, and ACK on drafts. No dependency on those plans' final release gates is implied; only overlapping file ownership is serialized. The desktop check depends on ABW-11; native save depends on ABW-12/17.
- The shared `CodeEditor` component owns toolbar markup and styling; `ToolWorkspace` owns which template choices and callback apply to Review's primary content editor. Reuse the TPL-04 project-creation function in Home and the popout. `editor/popout.js` should keep its existing window-opening and `acorn-editor` parameter behavior; only change it if a focused test proves a missing synchronization or URL-precedence case.
- Before implementation, confirm current `Workspace::replace` total-failure semantics, whether an existing project download already covers multi-file export, and the current `Tool::Standard` input limit. Update exact phase write sets if current code changed; do not broaden a phase silently.

# Execution phases

## TPL-01 — Freeze valid starter assets and catalog data

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: no concurrent owner of the named example assets or catalog leaf
- Exact write set: `crates/acorn-web/assets/templates/` (new), `crates/acorn-web/src/templates.rs` (new), `crates/acorn-web/src/main.rs`, `crates/acorn-web/scripts/template-fixtures.mjs` (new), `.wiki/features/browser-rads-templates.md`
- Primary integration lock: browser bundled template catalog
- Outcome: Ship three stable IDs with immutable metadata and bounded file arrays; confirm every file path/content passes `Workspace::replace` and every RADS file passes the same portable validation used by the app. Keep fixtures fictional and visibly editable.
- Focused validation: `cargo test -p acorn-web template`; `node crates/acorn-web/scripts/template-fixtures.mjs`; `cargo fmt --all -- --check`.
- Handoff: Record IDs, file counts, validation outputs, schema version/commit, and any project export gap; release the catalog leaf for UI integration.

## TPL-02 — Parse bounded URL entry intents

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: TPL-01 IDs are frozen; no concurrent URL-state or script boundary owner
- Exact write set: `crates/acorn-web/assets/url-state.js`, `crates/acorn-web/assets/url-state.test.mjs`, `crates/acorn-web/assets/scripts/url-entry-channel.js` (new), `crates/acorn-web/src/script/mod.rs`, `crates/acorn-web/src/url_intent.rs` (new), `crates/acorn-web/src/main.rs`, `.wiki/features/browser-rads-templates.md`
- Primary integration lock: browser URL entry contract
- Outcome: Use `window.acornUrl.getParameters` and its existing `subscribe` notification to decode mutually exclusive `code`, `template`, and `url` into a bounded typed intent with explicit errors. Add only the helper capability needed to reject malformed raw encoding. Preserve `setParameters` and `subscribe(fragment)` behavior so Review fragments and other URL consumers keep working. Deliver intents to Dioxus through the named script/channel boundary after storage recovery; no content mutation or fetch in this phase.
- Focused validation: extend and run the existing `node --test crates/acorn-web/assets/url-state.test.mjs` for root and `/acorn/` paths, duplicate/conflicting keys, `%` escapes, Unicode, Back/Forward, unchanged-href deduplication, `setParameters` preservation, and `#fair` Review fragments; `cargo test -p acorn-web url_intent`; `make web-lint`.
- Handoff: Freeze precedence, error text, byte limits, and event lifetime for UI consumption.

## TPL-03 — Open URL supplied editor drafts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: TPL-02; exclusive Review/editor signal owner
- Exact write set: `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/window_sync.rs`, `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/README.md`, `.wiki/features/browser-rads-templates.md`
- Primary integration lock: browser Review editor state
- Outcome: Apply accepted `code` once to `Request.input` and prefill accepted `url` in the existing loader. Handle dirty draft replacement, Back/Forward, popout/cross-tab interactions, and error announcements. Leave workspace and result untouched until the user acts.
- Focused validation: `cargo test -p acorn-web`; built-app Chromium URL/editor cases in `parity-smoke.mjs` at root and `/acorn/`.
- Handoff: Record URL examples and source precedence; release Review editor state before catalog UI work.

## TPL-04 — Present the catalog and create a local project

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: TPL-01–03; exclusive Home/workspace UI owner
- Exact write set: `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/templates.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/README.md`, `.wiki/features/browser-rads-templates.md`
- Primary integration lock: browser Home and workspace loading
- Outcome: Add discoverable template cards and preview, `?template=` selection, explicit creation/replacement, worker validation, first-file Review focus, save/reload, and an explicit complete-project download path. Preserve the existing sample button.
- Focused validation: `cargo test -p acorn-web`; `make web-test-pages` including root and `/acorn/` catalog/create/export/reload cases; keyboard focus check.
- Handoff: Document the project creation/export flow, expose one reusable confirmation/creation action for the editor dropdown, and record validation/count evidence for all starters.

## TPL-05 — Add templates to the new-tab editor controls bar

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: TPL-04; exclusive CodeEditor toolbar and Review tool-state owner
- Exact write set: `crates/acorn-web/src/syntax.rs`, `crates/acorn-web/src/ui.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/tab-sync-smoke.mjs`, `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/README.md`, `.wiki/features/browser-rads-templates.md`
- Primary integration lock: browser editor toolbar and popout
- Outcome: Render a styled, labeled template dropdown in the new-tab Review content editor toolbar. Route selection through the TPL-04 preview/confirm/create action, loading every project file and selecting its primary RADS file. Keep the toolbar control out of unrelated editors and preserve CodeMirror value, focus, and cross-tab synchronization on cancel or create.
- Focused validation: `cargo test -p acorn-web`; `make web-test-pages` after extending its existing `tab-sync-smoke.mjs` row for popout template selection, opener/popout state parity, and narrow/fullscreen keyboard cases.
- Handoff: Record popout and opener behavior, responsive layout, and accessibility observations; release the toolbar lock.

## TPL-06 — Accept shared web and packaged desktop behavior

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: TPL-05 and accepted ABW-11 shared Tauri shell; ABW-12/17 only for native save rows
- Exact write set: `crates/acorn-web/scripts/pages-smoke.mjs`, `crates/acorn-web/README.md`, `crates/acorn-desktop/acceptance/templates.md` (new), `.wiki/features/browser-rads-templates.md`
- Primary integration lock: browser and desktop template acceptance
- Outcome: Verify hosted direct links, offline catalog and new-tab toolbar dropdown, fresh/dirty/restored sessions, exported project validity, and identical catalog/editor behavior in packaged desktop. Record native-save follow-up only if its host capability is accepted.
- Focused validation: `make web-test-pages`, `make web-check`, packaged Tauri smoke on an available host, and manual keyboard/screen-reader review of catalog and confirmation controls.
- Handoff: Preserve durable acceptance evidence in product docs/tests and retire this plan when all required rows pass.
