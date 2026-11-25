---
type: Plan
title: Browser RADS project templates and editor URL intake
description: Accept bundled RADS templates and URL entry in the shared packaged desktop frontend.
difficulty: medium
impact: high
package: [acorn-web, acorn-desktop]
tags: [browser, desktop, rads, templates, editor, deep-links]
status: planned
execution: phased
phase_budget_hours: 5
---

# Outcome

Three validated, fictional RADS starters and bounded URL editor entry are implemented in the shared Dioxus frontend. The remaining outcome is packaged desktop acceptance after ABW-11 supplies the Tauri shell. The browser controls use no host-specific template store: the Home catalog and Review popout dropdown copy bundled files through `Workspace::replace` and worker validation; `?code`, `?template`, and `?url` use the existing `window.acornUrl` helper. This plan stays active solely for desktop acceptance.

# Current evidence — November 24, 2026

- `crates/acorn-web/src/templates.rs` embeds four fictional JSON records across `single-activity`, `publication-project`, and `collaboration-project`. Native and real WASM tests assert workspace bounds and RADS validation for every file. Home presents purpose, included fields, replacement guidance, file preview, explicit creation, and a bounded complete-project ZIP download.
- `assets/url-state.js` retains its `getParameters`, `setParameters`, and `subscribe` behavior while exposing raw-encoding and href reads. `assets/scripts/url-entry-channel.js` delivers typed entries to Dioxus; `src/url_intent.rs` rejects malformed, duplicate, conflicting, empty, and oversized inputs. Review protects dirty drafts, creates a project only after validation and confirmation, and prefills the existing HTTPS loader without fetching. The template link action uses `setParameters`.
- `src/syntax.rs` renders a labeled, styled dropdown only in the new-tab Review content editor. The shared preview/create path loads every template file and uses existing window synchronization. `scripts/template-smoke.mjs` exercises catalog, validation counts, URL entry, persistence, ZIP, focus restoration, and popout/opener behavior on a built Pages bundle; `make web-test-pages` includes it.
- `cargo test -p acorn-web`, the URL/ZIP Node tests, `make web-lint`, and `make web-check` passed. Final `make web-pages` and `make web-test-pages` passed for both root and `/acorn/` base paths, including direct-route refresh, template creation and ZIP, URL entry, editor popout sync, and storage checks.
- `crates/acorn-desktop/` does not exist. The [browser workspace plan](acorn-browser-workspace/index.md#abw-11--embed-the-shared-dioxus-workspace-in-a-minimal-tauri-shell) owns ABW-11, so no packaged desktop window, deep-link route, or native-save acceptance can currently be observed.

# Remaining acceptance

The packaged Tauri app must display the same three template cards and the Review popout dropdown, create each starter offline with expected activity counts, accept direct `template`, `code`, and `url` links, retain the explicit replacement and network actions, and export a valid project. Check fresh, dirty, and restored sessions; keyboard and screen-reader behavior; and opener/popout synchronization. Native save is accepted separately after ABW-12/17 provides its host capability. The browser root and `/acorn/` Pages tests provide the web half of the shared-front-end evidence.

# Dependencies and integration

- TPL-06 waits for ABW-11. Native save rows additionally wait for ABW-12/17. No TPL source edit should duplicate the Tauri frontend or introduce a native URL bridge.
- Serialize future edits to shared `acorn-web` UI, toolbar, script, and CSS files with ABW, SCU, USW, FMT, ACK, and FBQ owners. The ACD agent owns separate daemon/store work.

# Execution phases

## TPL-06 — Accept shared web and packaged desktop behavior

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: accepted ABW-11 shared Tauri shell; ABW-12/17 only for native save rows
- Exact write set: `crates/acorn-web/scripts/template-smoke.mjs`, `crates/acorn-web/README.md`, `crates/acorn-desktop/acceptance/templates.md` (new), `.wiki/features/browser-rads-templates.md`
- Primary integration lock: browser and desktop template acceptance
- Outcome: Run the accepted browser matrix in the packaged Tauri window, document route/deep-link and offline behavior, and verify native save only when its host capability exists.
- Focused validation: `make web-test-pages`, `make web-check`, packaged Tauri smoke on an available host, and manual keyboard/screen-reader review.
- Handoff: Preserve desktop evidence in product tests/docs and retire this plan when every required row passes.
