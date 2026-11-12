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

### Release skill bundle

`make web-skill` stages the canonical tracked `crates/acorn-cli/assets/skills/acorn/` folder in `target/acorn-skill/`. `make web-build` stages and verifies it under the release public directory's `acorn-skill/` after Dioxus finishes. This is the bundle contract for the later shared Skill route and desktop embed; the current UI does not yet expose acquisition actions.

Each bundle contains `SKILL.md`, the complete `acorn/` folder, `acorn-skill.zip` and `manifest.json`. Manifest schema 1 records `appVersion`, `sourceRevision` (Git HEAD), `sourceDirty` (skill/version worktree changes), `skillRevision` (SHA-256 of sorted compact JSON mapping relative file names to their SHA-256 hashes), `files`, `definition`, `archive`, source repository/path and compatibility facts. Browser remains preview and desktop planned; the CLI compatibility version matches the workspace. No timestamp or absolute local path enters the archive or manifest. ZIP entries have fixed timestamps, ordering and permissions and use stored bytes for cross-version reproducibility.

Run `python3 scripts/package-acorn-skill.py --check` to reject missing/stale staging, changed versions, source revisions, hashes, extra files or corrupted bytes. For the release directory, pass `--output target/dx/acorn-web/release/web/public/acorn-skill`. The packager includes only Git-tracked authored files, rejects symlinks, hidden/cache/key files and missing/out-of-bundle Markdown references, and never scans user agent directories. Add new authored resources to Git before packaging. Source edits must regenerate the bundle; do not hand-edit staging. `python3 -m unittest discover -s scripts/tests -p 'test_package_acorn_skill.py'` checks packaging invariants and is included in `make web-test`.

The route shell and operations are implemented; full Portfolio and folder loading, offline geospatial map, release-matched skill downloads, production desktop channels, Driver.js guided tour, system theme synchronization and full browser/assistive-technology matrices remain in the ABW plan. Resource pages describe pending functionality explicitly. The quick-start dialog is a quick-start guide, not accepted Driver.js tour implementation. Desktop artifacts are not built by these targets.

The default static build serves from the origin root. `WEB_BASE_PATH=/acorn` builds for a project-site prefix, including asset URLs and router links. Use the same prefix for the Pages smoke test. The local `web-preview` server is for root builds; rebuild with `make web-build` after testing a prefixed Pages build.

## Validation evidence

November 11, 2026, macOS local worktree:

- `make web-check`: native/WASM strict Clippy and package formatting passed. Workspace all-target/all-feature Clippy passed as well; workspace-wide formatting was intentionally avoided during concurrent work.
- Seven native bounds/route/rendering tests, four real WASM tests exercising all nine operations, invalid RAD and all bundled RAD examples, and 26 WebMCP adapter/lifecycle tests passed.
- `make test`: 3,048 passed, 18 skipped (nextest marked one passing test leaky). The first sandboxed attempt could not bind local mock servers; the authorized loopback rerun passed.
- `make web-build`: one 7,200,409-byte WASM payload and 86,364-byte bindings module, all nine exports. The WASM dependency tree excludes `acorn-lib` and `acorn-host`.
- `make web-dev WEB_PORT=8082`: built and served successfully; watcher logs show Rust source and staged adapter hot reload. Full observed RSX/CSS/static-edit acceptance remains in ABW-35.
- Safari release preview: sample RAD validation, Big-O comparison, successful RAD-to-DataCite conversion with loss warnings, empty-input feedback, route-result reset, theme toggle and mobile drawer open/route-close observed. Portrait 390×844 visually reflows without apparent horizontal overflow. Chrome showed WebMCP installed; Safari showed unsupported while manual operations remained usable. These are smoke observations, not full WebMCP invocation or accessibility acceptance.

The native file chooser opened, but end-to-end selection could not be established before the computer-use service failed; file/copy/download cross-browser acceptance remains ABW-06. Paste and bundled examples are the verified input paths. Landscape, screen reader, automated contrast, full console/network and browser-matrix checks remain ABW-08.

The dedicated `web-release` Cargo profile preserves wasm-bindgen metadata (`strip = false`). Release builds pass `--debug-symbols false` to Dioxus 0.7.3: its CLI otherwise enables WASM debug symbols by default, which reproduced a Binaryen DWARF abort. The corrected build completes without optimizer fallback and produces one 6,259,734-byte payload with 74,128-byte bindings and all nine exports. The package check compiles the WASM and rejects retained DWARF sections. ABW-35 still requires observed hot reload and hosted Pages/HTTPS acceptance. The final local test servers are stopped after verification; run either preview or dev above to review. The continuation cursor is in root `progress.md`.

## Review changes

The initial review replaces duplicated title/variant lists with `Display` and `EnumIter`, stages canonical documentation artwork, and adds three schema-checked RAD examples plus an independently validated ACRE configuration. Focused checks passed: eight SharePoint resolver tests, three artifact-source tests, one ACRE example test, 14 server tests, four native browser tests, three WASM tests (including all nine operations and all RAD examples), and 25 WebMCP adapter tests. Browser native/WASM Clippy and the release asset/export inspection passed. The documentation logo and staged asset are byte-identical.

Follow-up review: the browser inherits the workspace version and the UI reads Cargo package metadata. `make web-check` and `make web-build` passed after this change; the existing Binaryen fallback described above remains.

Review regressions: clean workspace checks regenerate absent assets and match their canonical sources byte-for-byte. The release-list check and CI's offline Cargo packaging command pass with the browser excluded. Native rendering tests preserve nested JSON Patch values; real WASM tests accept empty and whitespace-only unified diffs while native checks enforce byte limits and nonempty JSON Patch inputs. The JavaScript lifecycle test uses the real adapter with a model-context fixture to verify nine registrations and successful execution after two cached navigation cycles, then disposal on final unload. This is automated lifecycle evidence, not a new cross-browser observation.

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

Skill bundle acceptance (November 11): seven packaging tests and the skill validator passed. The source definition, fresh `cargo run --bin acorn -- skill` extraction, standalone bundle and browser release bundle are byte-identical (SHA-256 `e30c68a947641430cb58c556040e6aabf5f19b777f34f36ca58601d7bce406d5`). Two packaging runs produce identical archive bytes; stale version, corruption, missing references and unsafe staging are rejected. `make web-build` included the verified bundle and retained one payload with nine exports.

Batch regression checks: `make web-test` passed seven packaging tests, seven native tests, four real WASM tests and 26 adapter/lifecycle tests. Read-only workspace formatting and strict all-features/all-targets Clippy passed. `make test` passed 3,059 tests with 18 skipped after the approved loopback-capable rerun; the sandboxed attempt could not bind mock-server ports. `make check-wasm` passed all portable target checks. Live browser access was unavailable, so this adds no visual, keyboard, hot-reload or hosted-site acceptance.

## Shell acceptance — November 11, 2026

The shell preserves existing route slugs and uses the design specification's local outlined icons, grouped navigation, 220 px sidebar and 62 px collapsed rail. Every link retains an accessible name and title when its visible label is hidden; the active route uses `aria-current`. Route changes, including Home, focus the destination heading. The mobile drawer focuses Home on open, makes content inert only while open, closes on route selection, and returns focus to its button on Escape. Omit the HTML `inert` attribute when closed: `inert="false"` still disables interaction.

Home uses byte-identical repository dark/light banners. About has one locally bundled technology grid, with unchanged Rust/WebAssembly/Dioxus/Tauri mark bytes, visible linked names and decorative images. Tauri is labeled planned. [Artwork notices](assets/NOTICE.md) retain source URLs, hashes, Dioxus usage guidance and Tauri's MIT notice. Run `node --test crates/acorn-web/scripts/acceptance.test.mjs` after the build to check artwork integrity, canonical banner staging, 4.5:1 text contrast across normal/hover/selected/disabled token pairs and 3:1 focus contrast in both themes.

Live evidence: Chrome 156.0.8078.80 arm64 on macOS 26.7.2, native UI control, local release at `http://127.0.0.1:8082`. Observed dark/light banners, collapse with keyboard Home activation, Home→About heading focus and subsequent Tab to the source card, portrait 390×844 drawer open/Escape/keyboard Portfolio selection, landscape 844×390 traversal of all sixteen links and About activation, and desktop 200% zoom outside device emulation with visible focus and contained technology cards. The local WASM payload was 6,271,559 bytes with nine exports; bindings were 74,128 bytes. Build, seven native tests, host Clippy and three asset/contrast tests passed. Full screen-reader, other-browser and hosted-prefix acceptance remains in the owning release phases. The subsequent chrome implementation and its remaining acceptance are recorded below.


## Home and chrome implementation — November 11, 2026

The Home actions are Take a tour and Try sample data, followed by a three-example selector, skill entry and bounded local-file entry. File selection and drop share one reader: it checks the 256 KiB byte limit before and after reading and rejects invalid UTF-8 without replacing session content. Loading a sample or file routes to Research Activity without executing validation. Folder loading remains explicitly unavailable.

Themes initialize from `prefers-color-scheme`, follow changes until an explicit session choice, and reset from Settings. No theme preference is persisted. Notifications replace the prior message, dismiss explicitly or expire after six seconds. Driver.js 1.10.2 is bundled with its MIT license and source hashes. The four-step tour has progress, keyboard navigation, an accessible close control, reduced-motion support, completion/dismissal feedback, and focus return; route teardown and late module imports cannot reopen it. About has six linked technology cards and the exact local AI Assisted badge and three credited agent marks; source hashes and licenses are in [the asset notices](assets/NOTICE.md). All external links use `noopener noreferrer`.

Focused evidence: `make web-check` passed host and WASM checks and strict Clippy. `make web-test` passed seven packaging, seven native, four real WASM and 30 adapter/lifecycle tests. Four additional asset/contrast tests passed and are now part of `make web-test`. The optimized release contains one 6,281,724-byte WASM payload, 74,128-byte bindings and all nine operation exports. The selected sample now uses explicit option selection after live review found that the browser initially displayed the first option independently of state. The focused tour close style also covers hover/focus so the vendor stylesheet cannot darken it on a dark popover.

Live evidence on Chrome 156.0.8078.80 arm64/macOS 26.7.2: desktop dark→light choice and Settings system reset synchronized the toggle and labels; all four tour steps advanced by arrow keys, Done returned focus to Take a tour and showed completion feedback; Escape returned focus and showed dismissal feedback. At emulated 393×852 and 852×393 the tour controls remained visible, keyboard entry/navigation/dismissal worked, and the sample action opened the selected publications example with results still idle. About visibly loaded the badge, agent marks and six technology cards locally. Notification expiry was observed on subsequent navigation.

ABW-10 is **not yet accepted**. Remaining live checks are file chooser/drop routing, system-preference changes and reduced-motion emulation, light-tour and narrow About wrapping, offline/network inspection, and final comparison against the canonical Home/About previews. Native file-chooser control repeatedly returned the Go to Folder sheet to `/` after selecting the repository fixture instead of opening it; the browser tab connector was unavailable. Shared reader code and unit tests do not substitute for these live rows. Subsequent metadata presentation work retains ABW-10 as a prerequisite.

Final integration gates after the chrome implementation: `make lint`, `make test` (3,059 passed, 18 skipped), `make check-wasm`, planning-wiki validation and whitespace checks passed. Lint and the mock-server tests ran with approved loopback access after the sandbox denied the local listener. These gates do not close the outstanding live acceptance rows.
