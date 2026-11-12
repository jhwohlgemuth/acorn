# Next eight ABW phases — November 11, 2026

Prepared at `d0fe5103` for serial execution. This is an execution brief for the [canonical phases](index.md#execution-phases), not another plan or implementation acceptance. ABW-21 is accepted; ABW-35 has an accepted optimizer repair and retains live acceptance. ABW-20 is accepted and ABW-02 is accepted; ABW-10 is in progress with native Chrome access restored. The November 7 portfolio review is current; this pass checks only ABW currency.

## Remaining sequence and acceptance

| Order | Phase | Entry gate | Mergeable result |
|---:|---|---|---|
| 3 | ABW-10 | Accepted shell | System theme, bundled tour, file drop, notifications and Home/About chrome |
| 4 | ABW-03 | Accepted shell and ABW-10 | Metadata diagnostics and frozen shared input/result presentation |
| 5 | ABW-04 | ABW-03 shared contract accepted | FAIR/readability/diff charts with equivalent visible values and edge-case tests |
| 6 | ABW-05 | ABW-03 shared contract accepted | Identifier cards, candidate ordering, deduplication and bounded-result acceptance |
| 7 | ABW-32 | ABW-03 shared contract accepted | Labeled Big-O illustration and symbolic-result/assumption boundary acceptance |
| 8 | ABW-06 | ABW-03/04/05/32 accepted | Local file failures, explicit copy/download, format-specific exports and lifecycle evidence |

Each phase retains its five-active-hour ceiling and canonical primary lock. The sequence is a dependency-compatible browser acceptance batch; it is not numeric order or a promise that forty hours will be needed. Native Chrome access is restored; ABW-20 is accepted. Mark each subsequent phase ready only when its entry gates pass, and mark only the phase actually running in progress.

ABW-21 skill packaging is accepted. ABW-35 retains watcher/hosted Pages acceptance after the optimizer repair. The optimized local payload supports this batch; full production acceptance still requires ABW-35. Folder/Portfolio/map, WebMCP lifecycle completion, full ABW-08 browser/accessibility acceptance and desktop/publication phases remain outside these eight outcomes. Missing signing identities, channel ownership and native release hosts do not block this batch.

## Verified implementation baseline

- `src/ui.rs` owns `Route`, `Shell`, `Home`, `FileInput`, `ToolWorkspace` and `ResultTree`. The initial implementation uses a flat route and shared components, not the future `pages/` and `components/` layout. Keep changes in the current files unless a bounded extraction is explicitly added to the active write set.
- `src/operations.rs` owns `Tool`, `Request`, UTF-8 byte limits, real WASM dispatch and inline native/WASM tests. All nine tools delegate computation to `acorn-wasm`; the UI must not duplicate domain rules.
- The shell follows the system theme until an explicit session choice. Driver.js is bundled locally; theme/tour lifecycle tests and desktop/phone keyboard checks pass. ABW-10 still requires the remaining live acceptance recorded in the canonical phase. Results retain the generic structured tree for ABW-03.
- `assets/effects.js` implements explicit JSON copy/download and WebMCP install/disposal. Downloads currently use `acorn-result.json` and JSON MIME. Its existing test proves cached-navigation registration lifecycle; it does not prove file, clipboard or download acceptance.
- `build.rs` stages canonical artwork and the adapter. ABW-10 retains that file in scope for additional static assets. Generated asset output remains ignored.
- Preview routes differ from current public tool slugs. The accepted shell preserves the existing route slugs. Preserve existing local/Pages deep links during subsequent presentation changes. Do not silently rename existing URLs during presentation work.
- Fresh preparation checks: `cargo test -p acorn-web --lib` passed six tests; `cargo test -p acorn-web --bin acorn-web` passed the result-tree rendering test; `node --test crates/acorn-web/assets/effects.test.mjs` passed one test. Existing README WASM/build/browser evidence is historical and was not rerun in this preparation pass. Cargo emitted cached macOS SDK/linker sandbox warnings but the native library test completed successfully.

## Ownership and execution gates

One implementation owner serializes `ui.rs`, `operations.rs`, CSS, effects, build staging, README and wiki reconciliation. There are no parallel assignments in this batch. Keep unrelated PA participant-guide edits and generated LaTeX files untouched. Recheck worktree ownership before starting; preparation does not acquire a lasting lock.

The selected phase write sets name current files and reserved new test/vendor files. New browser acceptance tests may live in `crates/acorn-web/scripts/acceptance.test.mjs`; existing native and WASM tests remain inline. Do not edit portable owners, manifests, lockfiles, CI or Makefile for convenience. If a required change exceeds a write set, reconcile its scope and lock before editing. Asset sources, versions, hashes and notices must be recorded before shipping third-party artwork or Driver.js.

For every implementation phase, run `make web-check` and `make web-test` after relevant Rust/effect changes, plus the focused matrix in that phase. Use a built preview for live acceptance; rebuild with `make web-build` when browser source/assets change. Execute the added browser suite with `cd crates/acorn-web && node --test scripts/acceptance.test.mjs` for the asset/contrast checks; it does not automate a browser. Live acceptance uses native UI control.

Render and inspect each changed SVG. Tool phases must exercise real WASM outputs and keyboard/error-focus behavior, not only fixtures or static markup. ABW-06 must record file choice/cancellation/read failure, denied clipboard, exact output bytes/name/MIME, URL revocation and absence of operation network requests. Record browser/version, viewport and exact tested code state. Unavailable browsers and assistive technologies remain unverified; automated checks do not replace the live acceptance rows.

Recheck native file-dialog control before the remaining ABW-10 file acceptance. If unavailable, record the blocked acceptance row and continue only independent eligible work; do not accept a phase by inference. At four active hours, stop adding behavior and use the reserve for validation and a recoverable handoff. Unexpected defects beyond the outcome get a separate bounded repair phase.

After each accepted phase, preserve durable behavior/evidence in code, tests and the browser README, prune that phase, update the cursor in root `progress.md`, reconcile dependent readiness and affected indexes, and run the wiki validator plus `git diff --check`. Run final `make lint`, `make test` and `make check-wasm` serially after the implementation batch stabilizes. Preparation itself requires only focused baseline checks and wiki validation.

ABW-20 is accepted with six rendered exceptional-state previews and an equal-viewport live comparison. The active action is the remaining ABW-10 live acceptance; implementation and focused checks are recorded in the browser README. No product implementation, publication or external deployment occurred during preparation.
