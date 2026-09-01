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
│   ├── acorn-core/     # Portable errors, root values, validation primitives, and utilities
│   ├── acorn-lib/      # Compatibility facade plus schemas, analysis, and host services not yet extracted
│   ├── acorn-macros/   # Validation and command procedural macros
│   ├── acorn-cli/      # Native CLI composition root and embedded presentation assets
│   ├── acorn-tui/      # Terminal interface
│   └── acorn-py/       # Python bindings
├── docs/               # Public mdBook documentation
├── fuzz/               # Cargo fuzz targets
├── tests/              # Shared fixtures and integration data
├── xtask/              # Repository maintenance commands
└── Cargo.toml          # Workspace dependency, lint, and profile policy
```

The current dependency direction is:

```text
acorn-core
    |
    v
acorn-lib <----- acorn-macros
    |
    +----------> acorn-cli <----- acorn-tui
    +----------> acorn-tui
    +----------> acorn-py
```

`acorn-core` is the first extracted portable package. `acorn-lib` remains the public library crate named `acorn`; it re-exports core types and still owns implementation that will move into the planned crates below.

## Portability spectrum

Portability is a capability spectrum, not a native-versus-WASM switch. Both `acorn-core` and `acorn-lib` use `#![no_std]` when their `std` feature is disabled.

| Capability | Intended boundary | Current feature contract |
|---|---|---|
| `core` | Pure computation and APIs that require no host services | Foundation marker; never enables filesystem, network, process, terminal, or browser behavior |
| `alloc` | Heap-backed strings, collections, serialization, schemas, validation, and deterministic transforms | Extends `core`; enables allocation support without enabling `std` |
| `wasm` | Legacy facade compatibility for portable operations on `wasm32-unknown-unknown` | Currently extends `alloc`; it intentionally does not enable browser bindings or adapters |
| `std` | Native runtime support and adapter implementations | Extends `alloc`; enables filesystem, HTTP, database, process, document, terminal, and related host dependencies |
| Capability features | Selectable product behavior such as `analysis`, `agentic`, `doctor`, `powerpoint`, and `swhid-compute` | Layer on the minimum runtime they currently need; broad `std` coupling is being split into narrower crates |

Code should import from the narrowest prelude that matches its behavior: `prelude::core` for allocation-free language items, `prelude::alloc` for portable owned data, and `prelude::std` only for operating-system or runtime services. A type compiling for a WASM target is not evidence that it belongs in a browser adapter; browser APIs will live in `acorn-wasm`.

## Current module ownership

### `acorn-core`

`acorn-core` owns portable `AcornError`/`AcornResult`, `Location`, `Repository`, `Release`, `Scheme`, validation primitives, and core/alloc utility implementations. It must not own schemas, filesystems, networks, presentation behavior, or embedded assets.

### `acorn-lib`

`acorn-lib` currently contains these boundaries:

- `schema/` and `validation/`: RAD, PID, standards, hardware, agent models, crosswalks, codecs, and allocation-compatible validation.
- `analyzer/`: check/result models, readability, scoring, Vale integration, discovery, rendering, and analysis services. The `analysis` feature currently implies `std`.
- `io/`: native adapters and services for APIs, databases, documents, archives, downloads, synchronization, source handling, workflows, and PowerPoint.
- `doctor/`: native environment diagnostics behind the `doctor` feature.
- `enrichment.rs`: enrichment contracts and transformations shared by schema and service work.
- `util/`, `prelude.rs`, and `error.rs`: compatibility paths that re-export or extend portable `acorn-core` behavior and retain native helpers.

Host behavior already moved out of schema modules includes path/file `InputOutput` implementations, migration, OpenCode file I/O, model-weight persistence, contextual RAD formatting, and RAD Markdown document indexing. `LocationExt` and `RepositoryExt` add native behavior to portable `acorn-core` values without moving that behavior into the foundation crate.

The remaining critical portability edges are agent source resolution, embedded template rendering, OpenCode CST/path state, presentation side effects, and analyzer dependencies on document paths, rendering, files, processes, HTTP, and databases.

### Interface crates

`acorn-cli`, `acorn-tui`, and `acorn-py` are composition roots. They select concrete runtime capabilities and translate library results into terminal, Python, or application behavior. CLI argument parsing and exit/report presentation stay outside portable domain crates.

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

CLI / TUI / Python = services + native adapters
JS / WASM          = services + browser adapters
acorn-lib          = compatibility feature mapping and re-exports
```

Port traits belong to the portable domain or service that consumes them. `acorn-host`, `acorn-store`, and `acorn-wasm` provide concrete implementations; `acorn-service` orchestrates use cases without depending on a native adapter. `acorn-lib` must become a facade rather than a second implementation layer.

The next extraction is `acorn-schema`, after its remaining std-only state and side effects move behind adapters. `acorn-analysis` follows once check/result contracts no longer depend on document, renderer, filesystem, network, process, or database implementations. Provider, store, export, host, browser, service, and command packages follow one accepted boundary at a time.

## Cross-cutting rules

- Portable functions accept data and return values or typed errors.
- Filesystem, network, process, clock, entropy, database, terminal, and renderer effects stay in adapters.
- String/byte codecs live with domain types; path and URL reads/writes live at host boundaries.
- `color-eyre` and `tracing` provide rich native diagnostics; portable packages own errors that do not require host reporting.
- `rayon` is an optional native analysis implementation detail, not a requirement of portable analysis models.
- Each feature or crate exposes only capabilities it implements; `std` availability must not silently select unrelated services in the final package graph.
- Existing `acorn::...` paths remain compatibility shims until a separately approved breaking release.

The active extraction order, evidence, validation gates, and write-set coordination live in the [planning wiki](.wiki/index.md), especially the [plan dependency network](.wiki/plan-network.md).
