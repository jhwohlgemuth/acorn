# ACORN browser

The browser is a local Dioxus workspace for nine existing ACORN operations. The browser inherits the version in `[workspace.package]` from the root `Cargo.toml`; the UI displays that version from Cargo package metadata. All computation uses `acorn-wasm` in the same WASM payload as the UI. The browser dependency graph excludes `acorn-lib` and `acorn-host`.

## Develop, review and test

From the repository root:

```sh
make web-setup       # One-time: pinned Dioxus CLI and WASM target
make web-dev         # http://127.0.0.1:8080, Dioxus hot reload
make web-check       # Formatting, native/WASM checks and strict Clippy
make web-test        # Bounds, nine real WASM operations and WebMCP contracts
make web-build       # Release assets and single-payload export check
make web-preview     # Serve release assets at http://127.0.0.1:8080
```

Prerequisites are Rust/Cargo, Python 3, Node.js and `wasm-pack` (validated with 0.15.0). `web-setup` installs the Dioxus CLI and WASM target; it does not install those general-purpose tools.

`WEB_PORT=8081` selects another loopback port. `DIOXUS=/path/to/dx` selects an existing Dioxus CLI; it must be Dioxus, not Deno's `dx` alias. The default executable is `target/tools/dioxus/bin/dx`. Stop a server with Ctrl-C before starting another on the same port. `web-preview` serves only `target/dx/acorn-web/release/web/public`, with SPA route fallback. It exposes no repository source or upload endpoint. Development uses Dioxus's own watcher; release preview requires an explicit rebuild.

Cargo stages the canonical adapter (`crates/acorn-wasm/web/webmcp.js`) and artwork (`docs/src/_assets/acorn.svg`) into ignored crate-local assets before compilation. This also works for direct workspace Cargo commands and CI from a clean checkout. `make web-prepare` remains an explicit staging convenience. Source changes and deleted generated files trigger restaging; no adapter or artwork source is forked.

The browser crate has `publish = false`: its release artifact is the built website, and it is excluded from crates.io publication. CI explicitly excludes `acorn-web` from `cargo package --workspace`, alongside other non-published workspace members.

## Try the browser

Choose a research-activity example on Home, then **Try sample data** and **Run Research Activity**. The examples cover a concise research summary, publications/organizations/dates/outputs, and GPU resources/software identifiers. All use illustrative data. Home also offers an ACRE 2.0.0 configuration download (`assets/examples/.acorn.json`) for CLI use. ACRE configures an application and is not RAD; browser validation of ACRE is not available. Validate it with `acorn check crates/acorn-web/assets/examples/.acorn.json --standard acre`. Each research tool also has **Use example**. Inputs do not run automatically. The nine routes provide Research Activity and metadata-standard validation, crosswalk, FAIR assessment, readability, content diff, identifier discovery/inspection, and Big-O comparison. Unsupported conversions produce errors with the original editor preserved.

The UI displays structured result fields, including nested JSON Patch values, diagnostics and lists before optional raw JSON. Unified diffs accept empty and whitespace-only documents on either side within the same byte limits. **Copy JSON** and **Download JSON** require a click. File reading accepts UTF-8 content only and enforces the selected tool's byte limit before reading. Readability/discovery use 16 KiB; inspection 2 KiB; Big-O expressions 256 bytes; each diff side 128 KiB; metadata 256 KiB. Results are bounded to 256 KiB and display at most 100 entries per list. Download contains the complete accepted result.

Files and edits live only in tab memory. Reloading clears the session. Navigation seeds tools with the last explicitly run document; it does not persist independent drafts for every route. External project links open only when clicked. WebMCP installs the unchanged read-only nine-tool adapter if supported; manual tools work when unsupported. Cached back/forward navigation retains registrations; final page unload disposes them. No filesystem permission, remote identifier resolution, telemetry or service endpoint is introduced.

## Review boundaries

The route shell and operations are implemented; full Portfolio and folder loading, offline geospatial map, release-matched skill downloads, production desktop channels, Driver.js guided tour, system theme synchronization and full browser/assistive-technology matrices remain in the ABW plan. Resource pages describe pending functionality explicitly. The quick-start dialog is a quick-start guide, not accepted Driver.js tour implementation. Desktop artifacts are not built by these targets.

The static build currently serves from the origin root; subpath hosting requires a separate accepted base-path check. Do not advertise root-only preview evidence as arbitrary-prefix deployment acceptance.

## Validation evidence

November 11, 2026, macOS local worktree:

- `make web-check`: native/WASM strict Clippy and package formatting passed. Workspace all-target/all-feature Clippy passed as well; workspace-wide formatting was intentionally avoided during concurrent work.
- Seven native bounds/route/rendering tests, four real WASM tests exercising all nine operations, invalid RAD and all bundled RAD examples, and 26 WebMCP adapter/lifecycle tests passed.
- `make test`: 3,048 passed, 18 skipped (nextest marked one passing test leaky). The first sandboxed attempt could not bind local mock servers; the authorized loopback rerun passed.
- `make web-build`: one 7,200,409-byte WASM payload and 86,364-byte bindings module, all nine exports. The WASM dependency tree excludes `acorn-lib` and `acorn-host`.
- `make web-dev WEB_PORT=8082`: built and served successfully; watcher logs show Rust source and staged adapter hot reload. Full observed RSX/CSS/static-edit acceptance remains in ABW-35.
- Safari release preview: sample RAD validation, Big-O comparison, successful RAD-to-DataCite conversion with loss warnings, empty-input feedback, route-result reset, theme toggle and mobile drawer open/route-close observed. Portrait 390×844 visually reflows without apparent horizontal overflow. Chrome showed WebMCP installed; Safari showed unsupported while manual operations remained usable. These are smoke observations, not full WebMCP invocation or accessibility acceptance.

The native file chooser opened, but end-to-end selection could not be established before the computer-use service failed; file/copy/download cross-browser acceptance remains ABW-06. Paste and bundled examples are the verified input paths. Landscape, screen reader, automated contrast, full console/network and browser-matrix checks remain ABW-08.

The dedicated `web-release` Cargo profile preserves wasm-bindgen metadata (`strip = false`). With the pinned Dioxus CLI, Binaryen currently reports an unsupported-DWARF optimizer failure and Dioxus emits the valid unoptimized payload; browser execution and export inspection passed. ABW-35 tracks optimizer and base-path hardening before production release. The final local test servers are stopped after verification; run either preview or dev above to review. The continuation cursor is in root `progress.md`.

## Review changes

The initial review replaces duplicated title/variant lists with `Display` and `EnumIter`, stages canonical documentation artwork, and adds three schema-checked RAD examples plus an independently validated ACRE configuration. Focused checks passed: eight SharePoint resolver tests, three artifact-source tests, one ACRE example test, 14 server tests, four native browser tests, three WASM tests (including all nine operations and all RAD examples), and 25 WebMCP adapter tests. Browser native/WASM Clippy and the release asset/export inspection passed. The documentation logo and staged asset are byte-identical.

Follow-up review: the browser inherits the workspace version and the UI reads Cargo package metadata. `make web-check` and `make web-build` passed after this change; the existing Binaryen fallback described above remains.

Review regressions: clean workspace checks regenerate absent assets and match their canonical sources byte-for-byte. The release-list check and CI's offline Cargo packaging command pass with the browser excluded. Native rendering tests preserve nested JSON Patch values; real WASM tests accept empty and whitespace-only unified diffs while native checks enforce byte limits and nonempty JSON Patch inputs. The JavaScript lifecycle test uses the real adapter with a model-context fixture to verify nine registrations and successful execution after two cached navigation cycles, then disposal on final unload. This is automated lifecycle evidence, not a new cross-browser observation.
