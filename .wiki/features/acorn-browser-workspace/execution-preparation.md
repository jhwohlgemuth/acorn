# Browser parity execution handoff — November 24, 2026

The [canonical ABW phases](index.md#execution-phases) own scope, write sets, and acceptance. This handoff prepares ABW-03–08, ABW-16, and ABW-32. It does not claim those phases are implemented or accepted. [The browser README](../../../crates/acorn-web/README.md) retains durable behavior and earlier validation evidence.

## Verified baseline and ownership

- Baseline: `4fc72c19` plus the current uncommitted worktree. The plan records the shell and Home/chrome as accepted, so ABW-03 is ready. The current Review route, shared result presentation, nine operation bindings, folder effects, and WebMCP adapter exist.
- `cargo test -p acorn-web` passed 44 library and 28 binary tests. `node --test crates/acorn-web/assets/effects.test.mjs` passed 21 tests. `make test-webmcp` passed 46 adapter/effects tests. These runs use the current worktree; they are not real-WASM, browser, or assistive-technology acceptance.
- Existing tests check diagnostic ordering/grouping, operation bounds, some result shapes, WebMCP registration and lifecycle, folder effects, and example routes. The remaining phase matrices are wider than those tests.
- Uncommitted edits overlap the selected phases in `crates/acorn-web/src/ui.rs`, `src/results.rs`, `assets/app.css`, and `scripts/parity-smoke.mjs`; `scripts/pages-smoke.mjs` also has edits relevant to final browser acceptance. `src/location.rs` and its tests feed diagnostic and identifier location behavior. Preserve these changes and establish one owner for the shared browser files before ABW-03 edits. Do not run workspace-wide formatting while another implementation is editing.

## Execution order

1. **ABW-03:** Check ordered/truncated/missing-path metadata diagnostics, fuzzy policy, required seed, unsupported conversions, preserved editor state, and screen-reader announcements. Freeze the shared input/result behavior after focused native/WASM and browser evidence.
2. **ABW-04, ABW-05, ABW-32:** After ABW-03, accept analysis metrics/FAIR/diff, identifier kind/ambiguity/100-result cases, and Big-O symbolic/boundary cases. These phases share UI/result files; run them serially under the browser integration lock. They may open bounded repair phases when a defect falls outside a phase's write set.
3. **ABW-06 → ABW-16:** After all four operation phases, accept Firefox/WebKit file, copy, download, and object-URL behavior; then nested folder selection/drop, fallback, partial acceptance, keyboard navigation, and maximum-session timing. The existing cross-engine smoke is a baseline; Firefox's actual native chooser and screen-reader behavior still need observed evidence or an explicit unavailable row.
4. **ABW-07:** After ABW-03/04/05/32, accept fake-context workspace integration, nine-tool parity and lifecycle, package exports, and live supported-Chrome discovery/invocation/cleanup if available. Serialize package staging and `Makefile` with WME owners. ABW-07 can precede or follow ABW-06/16 once its own prerequisites hold.
5. **ABW-08:** Final browser acceptance follows this batch *and* ABW-19, ABW-65, ABW-18, and ABW-72. Prepare the viewport/engine/assistive-technology matrix now, but do not mark it ready until the index, map, and CZML phases pass. ABW-36 hosted Pages/HTTPS acceptance is a separate blocked gate.

## Next action

Recheck the current shared-file diff and owner, then start ABW-03 with one explicit fixture/result matrix for each remaining metadata case. Run focused Rust and real-WASM checks before browser and screen-reader acceptance. Preserve dated browser/version, assistive-technology, command, and fixture evidence in the browser README; remove a phase only after its full outcome passes. No product code, commit, deployment, or publication changed in this preparation pass.
