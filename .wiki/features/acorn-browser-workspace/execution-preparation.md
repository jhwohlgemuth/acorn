# acorn-web completion and desktop MVP handoff — November 25, 2026

The [canonical ABW phases](index.md#execution-phases) own outcomes, exact write sets, locks, and acceptance. This document orders the remaining browser work for a stable `acorn-web` build before the first `acorn-desktop` shell. It does not mark any phase accepted or authorize product implementation. The [browser README](../../../crates/acorn-web/README.md) holds durable implementation and test evidence.

## Current evidence and scope

- Baseline: `041577d0` plus the November 25 worktree. `crates/acorn-desktop/` is absent; `crates/acorn-web/src/session.rs` still has one optional `WorkspaceSource`, and `Workspace::replace` replaces the loaded session. The multi-bucket state of ABW-63 is therefore not present. The current geospatial island has organization markers, while ABW-44's source-aware project map remains planned.
- ABW-03 is accepted: the November 25 native, real-WASM, and 50-check built Chromium matrices cover its remaining metadata cases. ABW-19 remains **in progress**. Its local and earlier hosted private-provider evidence does not yet include the new media build's hosted private-provider recheck.
- At review start, the worktree had no `crates/acorn-web` or ABW-plan edits. Another agent currently owns ACD work and has edits in the ACD plan, shared wiki indexes/network/lane, and daemon installer files. Keep this handoff within `acorn-browser-workspace/`; coordinate before any ABW phase touches those shared wiki files or daemon code. No workspace-wide formatting during concurrent implementation.
- The monthly portfolio review dated November 13 is current for November. This is a targeted ABW review, not a new portfolio-wide audit.

## Execution path to a browser build ready for embedding

Each numbered ABW phase retains its canonical five-active-hour budget and exact write set. Recheck code and ownership before starting it; remove it only after its stated acceptance and README/test evidence are durable. The shared Rust UI, effects, CSS, parity scripts, map island, and asset staging each have one owner at a time.

| Order | Phase and handoff | Acceptance gate |
| --- | --- | --- |
| 1 | **Accepted:** ABW-03 metadata diagnostics, fuzzy/seed/conversion cases and preserved editor state. | Native and real-WASM tests and 50 built Chromium parity checks passed on November 25. |
| 2 | Run **ABW-04**, **ABW-05**, and **ABW-32** serially on their overlapping UI/result files. | Readability/FAIR/diff, all supported identifier kinds and 100-result behavior, and Big-O symbolic/boundary matrices pass against portable results. |
| 3 | Run **ABW-06 → ABW-16** on browser effects and folder fallback. **ABW-07** may run after step 2 under the separate WebMCP staging lock, but must serialize shared UI/asset edits. | Record exact Firefox/WebKit versions and observed chooser, cancellation, copy, download, object-URL, nested-folder, fallback, and maximum-session rows. Keep Firefox native chooser and live supported-Chrome WebMCP rows unverified until observed. |
| 4 | Finish **ABW-19 → ABW-63 → ABW-64 → ABW-65** on the loaded-index lock. The current one-source replace model becomes source-aware, then gains controls and browser acceptance. | Hosted private-provider recheck for ABW-19; two-provider, duplicate-path, migration, aggregate-limit, refresh/remove, reload, credential-isolation, and 500-file checks for ABW-63–65. Preserve the current workspace on failed additions. |
| 5 | Run **ABW-44 → ABW-18 → ABW-72** on the geospatial/map lock. ABW-44 must consume the accepted multi-bucket source identity; ABW-18 measures the resulting project map; ABW-72 adds local bounded CZML. | Project marker/table parity, no-network offline fallback, 500-file/1,000-entity resource and response measurements, then CZML limits, lifecycle, time playback, memory, and root/prefixed built-browser checks. ABW-37 is a later feasibility assessment and does not gate embedding. |
| 6 | Run **ABW-08** only after steps 1–5 and ABW-07. Freeze the built browser artifact and route/effect matrix for Tauri. | Chrome/Firefox/WebKit manual-tool and map behavior at 1440×900, 390×844, and 844×390; bounded errors, WebGL fallback, build sizes, one WASM payload, nine operation exports, and no unexpected operation or map network requests. Record unavailable rows. ABW-09 owns final screen-reader and native accessibility acceptance. |

Steps 1–3 and 4 can progress on disjoint files when ownership is explicit; any overlap in `src/ui.rs`, `src/workspace.rs`, `assets/app.css`, or parity scripts serializes. Step 5 waits for ABW-65. ABW-07 can overlap index or map work only when its actual write set is disjoint and the WASM package/Make lock is free. Run focused Rust, real-WASM, Node, geospatial, and built-browser checks at each handoff; reserve `make lint`, `make test`, and portability gates for a stable integration checkpoint.

## Desktop MVP start gate and separate work

The canonical **ABW-11** prerequisite is ABW-18 plus the accepted browser composition/shell. For the requested *finish-web-first* handoff, also complete ABW-72 and ABW-08 before starting ABW-11, so the Tauri shell embeds one reviewed static asset set. The ABW-11 review packet should contain the accepted browser revision, built asset path and hashes, one-payload/export check, map resource measurements, offline/network evidence, dev-server/asset-staging commands, and the known unverified browser rows. Then assign the manifest, lockfile, frontend server, and native shell lock to one owner.

ABW-11 creates only the minimal Tauri shell and restrictive asset/CSP boundary. **ABW-12** adds native single-file open/save after ABW-06; **ABW-17** adds selected-folder loading after ABW-12 and ABW-16. **ABW-45** freezes the broader companion/capability matrix after those native effects. These are successive desktop implementation gates, not work to fold into ABW-08 or ABW-11.

The separate ACD agent owns daemon service registration. Its ACD-11 macOS and ACD-13 Windows service-manager contracts affect **ABW-53** native daemon controls; they do not block ABW-11/12/17. Do not change ACD files or pre-empt its CLI/service lock. Recheck its accepted contract when ABW-53 becomes eligible.

**ABW-36** hosted Pages/HTTPS verification is blocked on the mirror deployment/certificate and does not block local Tauri development. **ABW-22–31**, **ABW-13–15**, **ABW-56**, and final **ABW-09** cover native capability, platform, signed distribution, accessibility, and release acceptance after the MVP shell; they are outside the browser-to-shell gate. **ABW-69** freezes a later React migration handoff after ABW-08 and ABW-11; it does not change the Dioxus frontend being embedded.

## Immediate next action

Continue ABW-19's index acceptance. The current built browser passed five focused index parity checks and the Pages suite's IndexedDB browser tests. `workspace-storage-smoke.mjs` is a legacy script tied to the old four-activity Portfolio DOM and localStorage format; exploratory count-only changes failed at its removed project-grid selector and were reverted. Use the current parity, storage, and remote-loader tests for local acceptance, adding only missing behavioral rows. The remaining hosted private-provider recheck needs an authorized browser session. The connected browser inventory is unavailable and the only accessible Chrome tab is unrelated Power Automate; no private repository was inspected. Preserve dated evidence without storing private records or tokens. Prune ABW-19 only after its complete local and hosted outcomes pass; ABW-63 is next.
