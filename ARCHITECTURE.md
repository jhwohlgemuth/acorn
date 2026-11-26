# Architecture

ACORN is a Rust workspace for maintaining, validating, analyzing, and publishing research activity data (RAD). The architecture is moving from one broad library toward portable domain crates, capability-specific adapters, application services, and thin interface composition roots.

## Design goals

- Keep research metadata, validation, and deterministic transformations usable without a filesystem, network, process runtime, or terminal.
- Make capability requirements explicit: portable code consumes values or narrow ports, while adapters own side effects.
- Preserve the established `acorn::...` API through `acorn-lib` while implementations move to focused `acorn-*` crates.
- Keep offline behavior, serialized formats, diagnostics, and CLI behavior compatible through each extraction.
- Prefer pure, composable transformations and typed errors; materialize files, requests, processes, and rendered artifacts only at host boundaries.
- Keep native and browser targets as peer compositions over the same portable domain rather than making target selection imply capabilities.

## Current workspace

ACORN product crates live under `crates/`; repository-only tooling and shared assets remain at the workspace root.

```text
acorn/
├── crates/
│   ├── acorn-cli/      # Native CLI composition root and embedded presentation assets
│   ├── acorn-analysis/ # Portable findings, FAIR checks, and readability analysis
│   ├── acorn-cmd/      # Portable command specifications and bounded native execution
│   ├── acorn-core/     # Portable errors, root values, validation primitives, and utilities
│   ├── acorn-diff/     # Portable unified-text and RFC 6902 diff primitives
│   ├── acorn-host/     # Feature-gated native filesystem, HTTP, source, and terminal adapters
│   ├── acorn-lib/      # Remaining native analysis and application adapters
│   ├── acorn-macros/   # Validation and command procedural macros
│   ├── acorn-schema/   # Portable schemas, validation, crosswalks, and codecs
│   ├── acorn-server/   # Authenticated JSON-RPC and typed gRPC composition root
│   ├── acorn-service/  # Portable supplied-content application operations
│   ├── acorn-tui/      # Terminal interface
│   └── acorn-wasm/     # In-memory JavaScript validation and analysis bindings
├── docs/               # Public mdBook documentation
├── fuzz/               # Cargo fuzz targets
├── tests/              # Shared fixtures and integration data
├── xtask/              # Repository maintenance commands
└── Cargo.toml          # Workspace dependency, lint, and profile policy
```

The current dependency direction is:

```text
acorn-core ----> acorn-schema ----> acorn-analysis ----> acorn-service
                     |                    |                   |
acorn-diff ----------+----------------> acorn-lib             +----> acorn-server
                                          |                   |
acorn-host -------------------------------+----> acorn-cli    +----> acorn-wasm
```

`acorn-core`, `acorn-diff`, `acorn-schema`, `acorn-analysis`, and `acorn-service` are portable packages consumed directly by first-party crates. `acorn-lib` remains the public library crate named `acorn` and owns native implementations that have not been extracted.

## Portability spectrum

Portability is a capability spectrum, not a native-versus-WASM switch. Both `acorn-core` and `acorn-lib` use `#![no_std]` when their `std` feature is disabled.

| Capability | Intended boundary | Current feature contract |
|---|---|---|
| `core` | Pure computation and APIs that require no host services | Foundation marker; never enables filesystem, network, process, terminal, or browser behavior |
| `alloc` | Heap-backed strings, collections, serialization, schemas, validation, and deterministic transforms | Extends `core`; enables allocation support without enabling `std` |
| `wasm` | Legacy facade compatibility for portable operations on `wasm32-unknown-unknown` | Currently extends `alloc`; it intentionally does not enable browser bindings or adapters |
| `std` | Native runtime support and adapter implementations | Extends `alloc`; enables filesystem, HTTP, database, process, document, terminal, and related host dependencies |
| Capability features | Selectable product behavior such as `analysis`, `agentic`, `acp`, `powerpoint`, and `swhid-compute` | Layer on the minimum runtime they currently need; broad `std` coupling is being split into narrower crates |

Code should import from the narrowest prelude that matches its behavior: `prelude::core` for allocation-free language items, `prelude::alloc` for portable owned data, and `prelude::std` only for operating-system or runtime services. A type compiling for a WASM target is not evidence that it belongs in a browser adapter; JavaScript bindings and future browser APIs live in `acorn-wasm`.

## Current module ownership

### `acorn-core`

`acorn-core` owns portable `AcornError`/`AcornResult`, `Location`, `Repository`, `Release`, `Scheme`, validation primitives, and core/alloc utility implementations. It must not own schemas, filesystems, networks, presentation behavior, or embedded assets.

### `acorn-diff`

`acorn-diff` owns alloc-only unified-text and deterministic RFC 6902 transformations. It accepts strings or `serde_json::Value` instances, performs no file access, and has no dependency on another ACORN crate. Same-standard parsing, canonical serialization, and fallback policy remain in `acorn-lib::analyzer::diff`.

### `acorn-lib`

`acorn-lib` currently contains these boundaries:

- Schema models, crosswalks, codecs, and allocation-compatible validation are owned by `acorn-schema`; `acorn-lib` retains their host adapters and services.
- The portable `LogbookEntry` schema distinguishes dated activities from project status updates. A status update carries significance, health, optional evidence and risk, and an entry-specific or logbook-level reporting range. Power Automate intake normalizes both variants, while report adapters render them separately; only activities can be graduated into canonical metadata.
- `analyzer/`: native context, discovery, trusted fix execution, Vale execution, path services, tabular export, and terminal rendering. Portable check/result models, FAIR calculations, and readability scoring are owned by `acorn-analysis`.
- `io/`: native adapters and services for APIs, databases, documents, archives, downloads, synchronization, source handling, workflows, and PowerPoint.
- `enrichment.rs`: enrichment contracts and transformations shared by schema and service work.
- `util/` and `error.rs`: native helpers and extensions around portable `acorn-core` behavior.

Host behavior already moved out of schema modules includes path/file `InputOutput` implementations, migration, OpenCode file I/O, model-weight persistence, contextual RAD formatting, and RAD Markdown document indexing. `LocationExt` and `RepositoryExt` add native behavior to portable `acorn-core` values without moving that behavior into the foundation crate.

The remaining critical portability edges are agent source resolution, embedded template rendering, OpenCode CST/path state, presentation side effects, and future browser composition. CI prevents the `acorn-analysis` and `acorn-service` alloc graphs from acquiring host-only dependencies.

### `acorn-analysis` and `acorn-service`

`acorn-analysis` owns portable findings, summaries, validation-error folding, FAIR evaluation, and readability metrics over supplied values. `acorn-service` owns the canonical validate-RAD, check-readability, and process-repository operations plus the transport-neutral JSON-RPC registry. Neither crate reads paths, starts processes, performs network requests, or renders terminal output.

### `acorn-server`

`acorn-server` adapts one `acorn-service` instance to authenticated JSON-RPC 2.0 and explicit typed gRPC methods. `acorn.version` is JSON-RPC package metadata only; it is not protocol negotiation and has no gRPC counterpart.

### `acorn-host`

`acorn-host` owns feature-gated native filesystem, configuration, HTTP, download, source, and terminal adapters. Source reads use one policy-aware implementation for paths, file URIs, and HTTP(S); `acorn-lib::io::Source` retains only application selection and materialization metadata. Plain and styled terminal labels share one textual definition so presentation features cannot change their wording.

### `acorn-wasm`

`acorn-wasm` is the JavaScript adapter for portable ACORN behavior. Its initial API parses and validates supplied Research Activity JSON and calculates readability checks over supplied text. It owns `wasm-bindgen` DTO conversion and package generation without enabling native host services. Browser HTTP, storage, document, process, and persistence adapters remain deferred until concrete service workflows prove narrow consumer-owned ports.

### `acorn-web`

`acorn-web` is the Dioxus browser presentation root. It links `acorn-wasm` into one application payload and reuses its portable operations for manual tools and optional WebMCP. It imports neither `acorn-lib` nor `acorn-host`. DOM-local file selection, clipboard and download effects belong to this concrete UI consumer; they do not introduce native capabilities or duplicate domain algorithms. See `crates/acorn-web/README.md` for the browser scope and local Make workflow.

The Graph page consumes `acorn-schema`'s source-backed RAD graph projection. It builds a simple undirected analysis graph in `acorn-web`, using `rustworkx-core` for centrality and a local triangle count for clustering. The authored directed relationships and their RDF export remain in the schema projection. The browser binary hosts the shared unit test suite so it can exercise both the library and browser-only modules.

`acorn-web/src/script` is the sole page-origin JavaScript evaluation boundary. Its named fire, typed query, and channel functions use `acorn-macros` to bind Rust values only as JSON; long bodies are authored under `acorn-web/assets/scripts`. Clippy rejects `document::eval` outside this module. The default-off host `js-lint` feature parses the catalog with OxC and checks referenced `window.acorn*` names against their defining assets. The parser is absent from the default WASM graph; runtime browser tests remain necessary for API behavior.

The bundled RADS starters and their catalog manifest live in `acorn-web/assets/templates`; the library `templates` module embeds and loads the catalog and project files. Home and the main Review page copy them through `Workspace::replace` and the existing browser validation worker before changing the local workspace. The editor popout receives content through window synchronization and omits the content selector. URL entry reuses `window.acornUrl` for parameter reads, subscriptions, and template link updates; a named script channel passes entries into the library `url_intent` module for bounded validation before Dioxus updates tool state. Inline code remains an editor draft until the user explicitly validates or creates a project.

The reusable geospatial model library lives in `acorn-web/assets/models`, following the crate-owned asset pattern used by migrations and prompts. Editable recipes generate self-contained GLBs and a catalog of stable IDs, revisions, hashes, provenance and placement metadata. Model authoring/validation scripts are separate from the runtime. The local `scripts/model-viewer.mjs` server serves the catalog, GLBs and installed Cesium distribution on loopback; its standalone picker and orbit UI live in `scripts/model-viewer/`, reusing the application stylesheet and theme-specific icons. Category navigation, search and a local dark/light preference accompany the preview. `model-detail.py` supplies asset-specific detail geometry; the exporter embeds projected UVs and deterministic roughness texture data. The library now contains 40 locally validated research, landmark, wonder and drone models, with twelve Risk assets in the separately requested `/Users/o9w/dev/risk` directory. Both collections share the ORNL palette with requested natural earth finishes, deterministic source/export contract and validator. The viewer accepts an external model root for browsing Risk. All model files and authoring/viewer tools are excluded from crate packages. Browser release staging and live Cesium object integration remain deferred to the ABW adoption gate. Models are not embedded in the portable schema or WASM payload by this asset addition.

### Interface crates

`acorn-cli` and `acorn-tui` are composition roots. They select concrete runtime capabilities and translate library results into terminal or application behavior. CLI argument parsing and exit/report presentation stay outside portable domain crates.

### Daemon jobs and history

`acorn-service` defines portable job identifiers, schedules, run states, and coalescing. `acorn-store` is the native SQLite adapter for daemon pause state and run history; its versioned schema is separate from existing `acorn-lib` databases. `acorn-daemon` owns fixed check/gather subprocesses and the single-worker file and interval scheduler. It invokes the paired `acorn` binary without a shell, stores bounded reports under the per-user project cache, and keeps host processes and filesystem watching outside portable crates. The daemon composes `acorn-server` with the shared operation catalog and JSON-RPC job control after the scheduler announces readiness. Authenticated `acorn.jobs.reload` validates a complete ACRE document and prepares watchers before publishing a new job generation; listener and TLS settings require restart. The scheduler serializes control requests, rejects stale watcher callbacks by epoch, and lets an in-flight run finish on its original job snapshot. The store replaces active jobs and retires superseded queued runs in one transaction while retaining run history and pause state. It loads a protected local token before starting the worker; listener failure, worker failure, and shutdown drain both components.

On Linux, `acorn-daemon::install::linux` owns an owner-scoped systemd user unit and invokes the foreground `acornd run` process with a canonical config path. The paired `acorn daemon` CLI delegates service lifecycle operations to its sibling `acornd` binary and performs job control through a bounded Unix-socket JSON-RPC client. The daemon creates an owner-only data directory and socket; the CLI verifies that directory and its ancestors before reading the token. The token stays off the loopback TCP connection and out of arguments and logs. The TCP JSON-RPC listener remains bearer authenticated for other clients. The installer can explicitly enable logind lingering; it does not silently change the user's linger setting.

Signed Linux daemon releases contain a manifest and paired binaries in a detached Minisign-signed tar archive. The CLI pins the release public key and checks the signature, target, member names, sizes, and SHA-256 hashes before installation. `acorn daemon install --bundle` copies the archive into an owner-only release directory, verifies and extracts from the same staged file handle, copies the configuration, and creates the protected local token if needed. It activates through the staged `acornd`, checks local JSON-RPC readiness, and restores the prior release on failure. An active-release marker selects the installed binary and default configuration for later CLI control; prior release directories and daemon data remain available for rollback.

On macOS, `acorn-daemon::install::macos` registers a root-owned system LaunchDaemon with a fixed unprivileged `UserName`, explicit home, absolute paired-binary and config paths, and owner-only umask. The user-side preflight validates the selected identity, config, token, and existing managed property list. `acorn-daemon::util::constants::app` owns daemon identity and launchd paths, while `PropertyList` serializes and checks the XML. Narrow `sudo!` calls, implemented in `acorn-macros` and re-exported by `acorn-cmd`, install or remove the property list and manage the system launchd domain; the daemon runtime and its state remain under the selected user's identity. Local job control uses the same protected Unix-socket boundary.

### Export and assets

PowerPoint and OOXML behavior currently lives in `crates/acorn-lib/src/io/powerpoint/`; CLI HTML/template composition lives in `crates/acorn-cli/src/template/`. Native rendering and filesystem writes are host behavior. The planned portable export boundary returns in-memory artifacts, while interface or host adapters decide how to materialize them. Fonts, images, styles, and agent skills are embedded by the CLI where the shipped binary needs them.

## Target package architecture

The planned dependency direction is:

```text
acorn-core
    |
acorn-schema
    +--> acorn-analysis
    +--> acorn-api
    +--> acorn-export
    +--> acorn-service

native adapters:  acorn-host + acorn-store + acorn-cmd
browser adapters: acorn-wasm

CLI / TUI  = services + native adapters
JS / WASM  = services + browser adapters
acorn-lib          = remaining host implementations during migration
```

Port traits belong to the portable domain or service that consumes them. `acorn-host`, `acorn-store`, and `acorn-wasm` provide concrete implementations; `acorn-service` orchestrates use cases without depending on a native adapter. `acorn-lib` shrinks as consumers migrate directly to each owning crate and does not retain ownership-masking re-exports.

`acorn-wasm` consumes `acorn-analysis`, `acorn-schema`, and `acorn-service` directly. It does not import `acorn-lib` or native host adapters.

## Cross-cutting rules

- Portable functions accept data and return values or typed errors.
- Feature selection may add or remove an explicit capability, but it must not change the result of a portable domain operation for the same input.
- Filesystem, network, process, clock, entropy, database, terminal, and renderer effects stay in adapters.
- String/byte codecs live with domain types; path and URL reads/writes live at host boundaries.
- Classification remains a pure data operation; validation or execution that needs a runtime is a separate step.
- Analysis reports distinguish executed categories with no findings from categories skipped by policy or unsupported by the selected standard.
- `color-eyre` and `tracing` provide rich native diagnostics; portable packages own errors that do not require host reporting.
- `rayon` is an optional native analysis implementation detail, not a requirement of portable analysis models.
- Each feature or crate exposes only capabilities it implements; `std` availability must not silently select unrelated services in the final package graph.
- Ownership transfers may break import paths at an explicit semver boundary; legacy re-export adapters are not retained.

The active extraction order, evidence, validation gates, and write-set coordination live in the [planning wiki](.wiki/index.md), especially the [plan dependency network](.wiki/plan-network.md).
