# ACORN browser

ACORN Browser is a local Dioxus workspace for nine ACORN operations, loaded Research Activities, and organization locations. It inherits the workspace version and links `acorn-wasm` into the UI's single WASM payload. Its dependency graph excludes `acorn-lib` and `acorn-host`. Desktop/Tauri delivery remains planned.

## Develop and validate

```sh
make web-setup       # Pinned Dioxus CLI and WASM target
make web-dev         # Dioxus :8080 and Vite map Fast Refresh :8083
make web-check       # Package formatting, native/WASM checks and strict Clippy
make web-test        # Packaging, native projections, real WASM, effects and assets
make web-build       # Static release and single-WASM/export verification
make web-preview     # Serve release assets on loopback :8080
npm --prefix crates/acorn-web/geospatial test
node crates/acorn-web/scripts/parity-smoke.mjs
```

Prerequisites: Rust/Cargo, Python 3, Node.js 22.17+ and `wasm-pack` (tested with 0.15.0). `web-map`, included in dev/release builds, runs `npm ci --ignore-scripts` against the pinned map lockfile. Install browser-test dependencies with `npm ci --prefix crates/acorn-web` and `crates/acorn-web/node_modules/.bin/playwright install chromium` once. The parity script accepts an optional public-directory path, screenshot-output directory and base prefix; defaults are `target/dx/acorn-web/release/web/public` and `target/abw-parity`.

`WEB_PORT=8084` changes the Dioxus or preview port; Vite uses loopback port 8083. The coordinator passes the selected frontend origin to Vite; CORS permits only that port on `127.0.0.1` and `localhost`. Both names work when opening the development frontend. For standalone `npm run dev` with a custom frontend port, set `ACORN_WEB_DEV_ORIGIN=http://127.0.0.1:<port>`. `DIOXUS=/path/to/dx` overrides `target/tools/dioxus/bin/dx`. The development coordinator waits for Vite readiness and stops both process groups on exit; a port conflict fails without terminating another server. This coordinator currently targets POSIX hosts. Compatible map TSX/CSS edits use Vite Fast Refresh/HMR; `main.tsx`, dependency/config changes and Rust changes can require a reload/rebuild. Initial Vite dependency optimization may reload once and clear the in-memory session. `node crates/acorn-web/scripts/dev-smoke.mjs http://127.0.0.1:8084` checks map refresh with temporary source edits restored in `finally`; run it only with no concurrent map editor. Dioxus owns the surrounding RSX/CSS/static watcher. Production embeds local static assets without either development client.

Cargo stages canonical artwork, the unchanged WebMCP adapter and the verified skill bundle before asset macros expand. Generated staging is ignored. Use `make web-map` before a direct `dx` build; ordinary Cargo checks do not build the JavaScript map. Release builds use the `web-release` profile and `--debug-symbols false` to avoid the Dioxus 0.7.3/Binaryen DWARF failure. The package check rejects a second WASM payload, missing operation exports or retained DWARF sections. The crate has `publish = false`; its distribution is the website.

## Local workspace and tools

Home provides the bundled tour, **Try sample data**, Skill entry and paired file/folder and drop loaders. The sample action explicitly loads and validates three illustrative RAD examples, then opens Portfolio. **Use example** on a tool only fills its editor. ACRE 2.0.0 is a separate CLI configuration download, not a browser validation tool.

Files remain in tab memory. The loader accepts `.json`, `.yaml`, `.yml`, `.md` and `.txt`, with 500 files, 20 path segments, 256 KiB per file and 16 MiB total. It rejects invalid UTF-8, unsafe paths and unsupported files, sorts relative paths, reports skipped entries and preserves the prior session on cancellation or total failure. Folder selection is capability-detected; unsupported browsers retain single-file loading. The expandable tree selects text without running a tool. Portfolio validates loaded JSON only when requested and derives filters, sortable records, deduplicated metrics and distribution bars from successful portable validation results. Replacing or clearing the session removes stale validation.

All nine tools use the existing `acorn-wasm` operations. Results show ordered diagnostics, crosswalk loss warnings, FAIR counts/findings, readability scores and configured thresholds, literal unified diffs or nested JSON Patch, identifier cards/candidates, and symbolic Big-O comparisons. The n/n² plot is explicitly an illustrative example, not a general evaluator or evidence of symbolic dominance. Every chart retains visible numeric/text equivalents. Missing/non-finite/off-scale values have explicit presentation. Raw JSON remains available.

Run is explicit and focuses the result heading. Invalid operations preserve editor text. **Copy result** and **Download result** export the accepted converted content, unified diff, JSON Patch or JSON result with a safe filename/MIME. Adapter limits remain metadata 256 KiB, each diff side 128 KiB, readability/discovery 16 KiB, identifier 2 KiB and Big-O expression 256 bytes; results are capped at 256 KiB and displayed lists at 100 entries. Navigation seeds tools from the shared document; independent per-route drafts are not persisted.

Settings offers Dark/Light/System, actual file/activity/byte counts, clear-session and bundled skill identity. System preference changes apply until an explicit choice. No theme or content is persisted. The drawer makes main content inert only while open and clears that state on desktop resize. Home/About reuse canonical banners and unchanged credited artwork; [asset notices](assets/NOTICE.md) record licenses and hashes. Driver.js 1.10.2 is bundled locally with teardown, reduced-motion and focus-return handling. External project links require a click. The read-only WebMCP adapter is unchanged; unsupported browsers retain all manual tools.

## Skill and Install

The Skill route verifies the bundled schema/version, raw-definition digest and ZIP digest before enabling acquisition. It presents source/revision/digests, raw/copy/archive actions, four copyable workflows and manual install/update/rollback/removal guidance. Exported copies remain user-managed; the app never scans or overwrites agent folders. Settings displays the same verified identity. Install shows all five GUI channels as pending until actual release identities and published artifacts exist; no CLI package is presented as a GUI installer.

`make web-skill` stages the canonical tracked `crates/acorn-cli/assets/skills/acorn/` folder in `target/acorn-skill/`. `make web-build` verifies the release `acorn-skill/` copy; Cargo also stages route-local raw/manifest/archive assets. Manifest schema 1 records app/CLI versions, source revision/dirty state, deterministic skill revision and file hashes. ZIP entries have fixed ordering, timestamps and permissions. `python3 scripts/package-acorn-skill.py --check` verifies staging; pass `--output target/dx/acorn-web/release/web/public/acorn-skill` for the release copy. Only tracked authored files are packaged; add new resources to Git before packaging. Seven packaging tests cover deterministic bytes and stale/corrupt/unsafe inputs.

## Offline organization map

Geospatial lazy-loads the [pinned ROR 2.15 location index](assets/geospatial/ROR_PROVENANCE.md), extracts validated ROR references, deduplicates organizations and retains unresolved/locationless entries. The first country/coordinate-sorted location is used; this is not a headquarters inference. Dioxus owns filters, selection, source counts, related activities and the semantic table. At most 1,000 matching markers/rows cross the bridge, with a visible truncation message. The route-local React/Resium island owns only the Cesium viewer. A WebGL 2 capability failure leaves the selectable table available.

Cesium uses bundled Natural Earth II imagery, workers/widgets and ellipsoid terrain, with no ion token, remote imagery/terrain, geocoder or ROR request. Releases carry upstream license notices. Four unused optional Cesium WASM decoders are excluded because this viewer exposes no 3D Tiles, KTX, splat or ZIP import. Enabling those features requires revisiting the asset contract.

Initial static budget (uncompressed / sum of per-file gzip bytes): map entry 5.75 MB / 1.43 MB; CSS 24 KB / 5.4 KB; workers 1.03 MB / 327 KB; widgets 515 KB / 429 KB; Natural Earth II tiles 476 KB / 470 KB; complete map graph about 11.95 MB / 5.32 MB. ROR adds 22.28 MB / 4.96 MB and loads only on Geospatial. These are measured transfer/storage sizes, not browser memory budgets. Dataset regeneration from the pinned source hash is deterministic. Maximum-session performance and native-webview resource acceptance remain open.

## Validation evidence — November 12, 2026

The current parity work has passed `make web-check`, `make web-test`, map TypeScript/bridge tests, read-only strict workspace Clippy, `make test` (3,066 passed; 18 skipped) and a release build with one 6,463,864-byte WASM payload and all nine exports (the final package command reports exact sizes). Isolated Chromium 158 checks cover all nine example operations/result focus, invalid-input preservation, literal diff-download bytes, Skill ZIP digest parity, theme changes, local imagery and table selection, file/folder pickers, file drops, cancellation and UTF-8/size rejection, 390×844 and 844×390 route reflow, 200% CSS zoom, tour focus return, session clearing and mobile-to-desktop inert cleanup. All 20 checks passed at the origin root and `/acorn/` prefix, including forced WebGL-off table fallback and zero unexpected external requests. Screenshots and machine-readable checks are generated under `target/abw-parity/` and `target/abw-parity-prefix/`. Map TSX and CSS hot reload preserve the Dioxus route and organization selection; source edits are restored after the check.

Review regressions: ROR HTTP/HTTPS/bare values now deduplicate to the HTTPS index key; unified diff counts use hunk content and retain changed `---`/`+++` separators. Live Chromium checks pass on `127.0.0.1:9010` and `localhost:9010`, including exact-port CORS allow/deny cases, map loading and Fast Refresh. Rejected map URLs deliver the table fallback. Focused Rust/JavaScript/TypeScript/Clippy checks and `make test` (3,067 passed, 18 skipped) pass after these fixes.

Pages direct-route refresh under `/acorn/`, portable WASM gates, read-only workspace formatting and wiki validation also pass. The final root build is restored for `make web-preview`.

These automated Chromium checks do not establish Safari/Firefox folder behavior, screen-reader announcements, supported-Chrome WebMCP invocation, native dialogs or published desktop artifacts. The ABW plan retains those acceptance rows, maximum-bound timing, native integration and hosted deployment checks. Earlier native Chrome shell checks established keyboard navigation and browser 200% zoom; CSS-zoom automation is additional evidence, not a substitute for every browser's zoom behavior.

## Application publishing lifecycle

`acorn-web` and `acorn-desktop` are application distributions. `packaging/applications.json` records both independently of the crates.io release list. Rust release commands live in `xtask/src/application_release/`. `make check-release-apps` runs `cargo xtask check-release-apps`, checks coverage, workspace version inheritance, `publish = false` and pending desktop work, and runs the Rust packaging/upload regression tests.

`make release-version VERSION=X.Y.Z` runs that application check before changing the workspace version, then updates dependency pins, `CITATION.cff` and `Cargo.lock` and verifies crate/version consistency. The browser package, UI version, archive name and embedded release metadata derive from the workspace version. After the version change reaches GitLab main, CI builds and publishes the matching browser ZIP and checksum. The command itself does not build or upload artifacts. Desktop remains an explicitly planned distribution until its native crate and packaging jobs exist; it has no artifact to release yet.

`make web-package` builds the browser, verifies the single WASM payload and nine operation exports, then writes `bin/acorn-web-v<VERSION>.zip` and its `.sha256` sidecar. Extract the ZIP and serve it over HTTP at the origin root. `release.json` inside the archive records the source revision, workspace version, file hashes and application publication states. Archive bytes are reproducible for identical inputs.

On merge requests and relevant default-branch changes, `build-web` checks, tests and packages the browser. On the existing default-branch `Cargo.toml` release trigger, it also runs `make web-publish`, using `CI_JOB_TOKEN` and `PKG_REGISTRY_URL`. Publication verifies the existing archive before uploading it to `acorn-web/v<VERSION>/` in the GitLab generic package registry. The release job waits for this build/upload and exposes ZIP and checksum links labeled **browser preview**. Existing identical remote bytes are accepted on retries; different bytes at the same URL are rejected. `make web-package` delegates to `cargo xtask package-web`; `make web-publish` delegates to `cargo xtask publish-web`. The latter performs a real upload and does not rebuild. GitHub Pages deployment is handled separately by the mirror workflow below.

The browser remains a preview while its documented acceptance and optimizer limits remain open. These registry URLs use the project's access policy; anonymous public access has not been established. The desktop crate does not yet exist. Its inventory entry remains `planned`, and no native download is advertised. ABW-11 must add the crate with workspace version inheritance and `publish = false`, move the inventory to `development`, and add native build checks and the Cargo packaging exclusion. ABW-23–30 must add signed native artifacts, release links and package-manager publication using the same immutable upload path, with actual public-download/signing evidence. Desktop packaging must retain the shared frontend version and digest and use identifiers distinct from the CLI.

For an existing browser build, run `cargo xtask package-web` and `cargo xtask verify-web` without rebuilding. Both accept `--output DIR`; packaging also accepts `--source DIR`. The CLI artifact uploader uses the same `cargo xtask upload-artifact FILE HTTPS_URL` implementation. Release checks, packaging, verification and HTTP uploads require no Python runtime.

## GitHub Pages mirror deployment

GitLab remains the source repository. Its enabled push mirror updates protected branches in `jhwohlgemuth/acorn`. `.github/workflows/acorn-web-pages.yml` deploys the browser preview after each mirrored push to `main`; it also supports a manual run on `main`. Other branches and forks cannot deploy through this workflow. Pages is a continuously updated preview of mirrored main, independent of the versioned GitLab ZIP releases.

The build reads the Pages `base_path`, runs `make check-release-apps`, builds with `make web-pages`, and runs `make web-test-pages` in Chromium before uploading only the generated public directory. The deploy job uses GitHub's Pages artifact and OIDC actions with `pages: write` and `id-token: write`; no GitLab token, personal deployment token, generated branch or additional backend is needed. Deployments are serialized so an active deployment is not interrupted by a later mirror push.

`web-pages` adds an identical `404.html` and a `.nojekyll` marker. A direct tool URL or refresh receives GitHub Pages' HTTP 404 response, then boots the application at that route. This preserves the URL and navigation; it does not change the server response to HTTP 200. The smoke test checks this behavior, stylesheet and image loading, local WASM execution, router links, and failed or unexpected network requests.

Reproduce the project-site build and browser check locally:

```sh
npm ci --prefix crates/acorn-web
# One-time browser installation; Linux CI also passes --with-deps.
crates/acorn-web/node_modules/.bin/playwright install --only-shell chromium
make web-pages WEB_BASE_PATH=/acorn
make web-test-pages WEB_BASE_PATH=/acorn
```

The GitHub repository is configured with Actions enabled, Pages' publishing source set to **GitHub Actions**, and the `github-pages` environment restricted to `main`. GitHub assigns the project site to the account's existing domain at `www.jasonwohlgemuth.com/acorn/`. The first hosted deployment is pending the workflow reaching the mirror. HTTPS enforcement must be enabled once GitHub provisions the site certificate; the setup API currently reports that the certificate does not exist yet. The GitLab mirror credential must be allowed to push workflow files (the `workflow` scope for a classic GitHub PAT). Keep workflow changes in GitLab so later mirroring does not overwrite GitHub-only edits. The first deployment requires this workflow and its supporting files to reach the mirror; the resulting live URL is recorded by the `github-pages` deployment. This setup does not change `acorn.ornl.gov` or publish desktop installers.

References: [GitHub custom Pages workflows](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages), [Dioxus base-path configuration](https://dioxuslabs.com/learn/0.7/guides/tools/configure/).

Local hosting evidence: release builds and Chromium 158 smoke tests passed for `/acorn/` and an empty root base path. Both loaded assets, ran Research Activity validation, navigated within the configured prefix, and refreshed a direct Readability route through the Pages-style 404 response. The optimizer fallback was subsequently removed by the verified debug-symbol repair; this local hosting evidence does not claim a live GitHub deployment or a full browser matrix.
