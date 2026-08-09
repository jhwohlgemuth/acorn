---
type: Plan
title: "Split ACORN along layered crate boundaries"
description: "Decompose acorn-lib into portable domain crates, capability adapters, application services, and a compatibility facade."
difficulty: high
impact: critical
package: [workspace, acorn-lib, acorn-core, acorn-schema, acorn-analysis, acorn-api, acorn-store, acorn-export, acorn-host, acorn-wasm, acorn-service, acorn-cmd]
tags: [refactor, architecture, crate-extraction, portability, dependencies]
status: in-progress
priority: next
---

# Goal

Replace the current all-purpose `acorn-lib` package with a directed set of crates whose dependencies follow domain, adapter, service, and interface layers. Retain `acorn-lib` as the supported feature-gated compatibility facade while first-party applications adopt direct dependencies where doing so avoids unrelated capabilities.

# Current evidence

- `acorn-lib` currently exposes analyzer, I/O, schema, utility, and doctor modules from one package.
- Module dependencies are cyclic: schemas use I/O and utility services, I/O uses schemas and analyzers, and analyzers use I/O, schemas, and utilities.
- `schema` types still implement the host-facing `InputOutput` trait and import filesystem operations, preventing a portable schema boundary.
- The portable foundation now exists inside `acorn-lib`: `AcornError` and `AcornResult` are independent of host reporting, schema result ownership no longer points through `io`, and utility code exposes a portable surface separate from terminal output while legacy paths remain available.
- Minimal and alloc builds no longer resolve `color-eyre`, `console`, `owo-colors`, `rust-embed`, or `similar`; immutable validation reference data is compiled directly without the generic embedded-asset runtime.
- Default, minimal, alloc, and `wasm32-unknown-unknown` checks now pass for the in-tree boundary, and the WASM dependency tree also excludes those presentation and generic asset packages.
- Existing ACORN product crates now live under `crates/`; repository-only tooling such as `xtask` and `fuzz` remains at the workspace root. New packages in this plan have a stable nested destination before extraction begins.
- The current no-default build still imports `std` for schema derive output, and `util::portable` still re-exports definitions owned by the mixed root utility module. These are remaining Phase 1 portability boundaries rather than extraction work.
- `analyzer` combines check models and readability transformations with HTTP, filesystem, database, document conversion, and process execution.
- `io` combines provider APIs, databases, document rendering, downloads, configuration, source handling, synchronization, and application workflows.
- The `std` feature enables unrelated host capabilities, so consumers cannot select a narrow dependency graph.
- Existing portability, analyzer, API, export, and command-extraction plans already define prerequisite boundaries but do not provide one target package graph.

# Target package graph

Dependencies flow upward through the following layers:

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

CLI / TUI / Python = service + native adapters
JS / WASM          = service + browser adapters
```

The diagram shows the dominant direction rather than every direct edge. Port traits live with the domain or service that consumes them. `acorn-host`, `acorn-store`, and `acorn-wasm` implement those ports, while interface crates act as composition roots and select concrete adapters. `acorn-service` must not depend on a native adapter. `acorn-lib` sits beside this graph as an optional re-export facade and must not become an implementation layer.

# Repository layout

All ACORN product and library packages use a flat namespace under `crates/`. The directory nesting expresses workspace ownership; dependency direction remains defined by the package graph rather than directory order.

```text
crates/
├── acorn-cli
├── acorn-lib
├── acorn-macros
├── acorn-py
├── acorn-tui
├── acorn-core       (planned)
├── acorn-schema     (planned)
├── acorn-analysis   (planned)
├── acorn-api        (planned)
├── acorn-store      (planned)
├── acorn-export     (planned)
├── acorn-host       (planned)
├── acorn-wasm       (planned)
├── acorn-service    (planned)
└── acorn-cmd        (planned)
```

Workspace-wide support projects that are not published ACORN product crates, including `xtask` and `fuzz`, remain top-level directories. Tests, documentation, notebooks, scripts, and fixtures also remain rooted outside `crates/` and must be addressed through explicit workspace-relative paths.

| Package | Owns | Must not own |
|---|---|---|
| `acorn-core` | Portable errors, location and repository values, MIME/checksum/version types, interpolation, and pure string and collection utilities | Schemas, filesystems, networks, presentation, or embedded assets |
| `acorn-schema` | RAD, PID, standards, hardware, and agent data models; validation; crosswalks; parsing and serialization from strings or bytes | Paths, HTTP, processes, databases, terminal styling, or rich host reports |
| `acorn-analysis` | Check/result types, scoring, summaries, readability, validation folds, and traversal over supplied content | Concrete filesystem, HTTP, database, document, or subprocess implementations |
| `acorn-api` | Endpoint, authentication, request, response, and provider-specific client contracts and DTOs | Persistence, analysis, or bot workflow orchestration |
| `acorn-store` | SQLite and DuckDB adapters, cache and table models, and durable operation persistence | API invocation, presentation, or application workflows |
| `acorn-export` | Shared export context, templates, charts, OOXML/PowerPoint transformations, and renderer contracts that produce artifact bytes | Filesystem writes, browser/process startup, or CLI output |
| `acorn-host` | Native filesystem and source handling, native HTTP transport, document conversion, process execution, downloads, synchronization, doctor checks, and concrete analysis/export adapters | Browser adapters, domain policy, or multi-service orchestration |
| `acorn-wasm` | `wasm-bindgen`/`js-sys` integration, browser HTTP and storage adapters, JS-facing DTO conversion, and the WASM composition root | Native filesystem/process assumptions or portable domain policy |
| `acorn-service` | Artifact discovery, analysis use cases, GitLab bot/webhook/intake flows, MCP/server workflows, and orchestration over injected port traits | Concrete host, browser, provider transport, or database implementations |
| `acorn-cmd` | Command construction and execution helpers plus the public command macro surface | General-purpose ACORN utilities |
| `acorn-lib` | Compatibility modules, feature mapping, and public re-exports | New implementation logic |

Individual schemas, providers, commands, doctor checks, and generic utility collections do not become separate crates. They remain modules within the package that owns their policy.

# Boundary rules

1. Dependencies must not point from a portable layer to a host or service layer.
2. Pure functions accept data and return values or typed errors; filesystem, network, process, clock, entropy, and rendering side effects remain in adapters.
3. A port trait belongs to the domain or service that consumes the capability. Concrete implementations belong to `acorn-host`, `acorn-store`, `acorn-wasm`, or another adapter crate selected by a composition root.
4. `acorn-schema` owns format codecs for strings and bytes. Host-owned extension traits or free functions own path and URL reads and writes.
5. `acorn-api` retains provider-specific protocol policy. GitLab bot decisions, persistence, analysis, and report publication belong to `acorn-service`.
6. `acorn-export` produces in-memory artifacts. `acorn-host` materializes them with native renderers; `acorn-wasm` exposes only browser-supported rendering and delivery adapters.
7. Each crate exposes only capabilities it actually implements. `std` availability must not implicitly enable every host service.
8. `acorn-lib` may map legacy features to new package features, but first-party interfaces should depend directly on the narrow packages they use.
9. Public compatibility shims re-export implementation; they must not duplicate it.
10. Browser dependencies are target-specific to `acorn-wasm`; portable crates must not gain a `wasm-bindgen` or `js-sys` dependency merely because they compile for a WASM target.
11. New ACORN packages are created as `crates/<package-name>`; product crates must not return to the workspace root.

# Migration sequence

## 1. Prepare boundaries inside acorn-lib

Do not create new crates during this phase. First make the intended boundaries explicit and acyclic inside `acorn-lib` so package extraction becomes a mechanical move rather than a simultaneous redesign.

### 1.1. Enforce the portable dependency boundary

Move portable implementations into capability-scoped modules and re-export them from legacy paths, rather than defining them in mixed modules and re-exporting them inward. Introduce strict core, alloc, and std preludes; remove the schema-derive `extern crate std` workaround; and audit the remaining entropy, process, and native-service dependencies in minimal and alloc configurations.

Move the now-WASM-safe immutable validation vocabularies under the schema boundary. Continue validating default, minimal, alloc, and WASM configurations throughout Phase 1. The facade retains its legacy feature names, but extracted portable crates expose only capabilities they implement.

### 1.2. Separate schemas from host I/O

Keep parsing, serialization, validation, and crosswalks with schema types. Move path, URL, and file reads and writes behind host-owned extension traits or free functions. Preserve the existing `acorn::io::InputOutput` surface through compatibility re-exports.

### 1.3. Separate analysis from host adapters

Keep checks, scoring, summaries, readability, validation folds, and traversal over supplied content in the pure analysis boundary. Move Vale execution, HTTP link checking, document loading, database discovery, and filesystem handling into adapters. Supply immutable inputs or focused capability traits to analysis functions.

### 1.4. Separate workflows from provider clients

Keep endpoint, authentication, request, response, and provider protocol mechanics in the API boundary. Move GitLab bot decisions, persistence coordination, analysis invocation, and report publication into a service boundary.

### Phase 1 exit gate

- `schema` no longer imports `io`.
- Pure analysis does not instantiate filesystem, network, process, document, or database services.
- Provider API modules do not orchestrate bot workflows.
- Existing public paths, feature behavior, formats, offline behavior, and diagnostics remain compatible.
- Minimal and alloc dependency trees exclude host reporting, terminal, database, process, native HTTP, async-runtime, and generic embedded-asset dependencies.
- Focused tests, `cargo check -p acorn-lib --no-default-features`, `cargo check -p acorn-lib --no-default-features --features alloc`, `cargo check -p acorn-lib --no-default-features --target wasm32-unknown-unknown --features wasm`, `make lint`, and `make test` pass.

## 2. Extract the portable foundation

Create `crates/acorn-core`, followed by `crates/acorn-schema`. Preserve existing `acorn::schema`, root type, and utility paths through facade re-exports. Give each crate an explicit core, alloc, and std contract; target selection alone must not enable browser dependencies. Retain the facade's legacy `wasm` feature as compatibility wiring.

## 3. Extract domain capabilities

Extract `acorn-analysis` after analyzer result and traversal contracts stabilize. Complete the linked `acorn-cmd` extraction independently. Neither package may depend on the compatibility facade.

## 4. Extract adapters

Extract provider protocol logic into `acorn-api`, persistence into `acorn-store`, and renderers into `acorn-export`. Move native operations and concrete native port implementations into `acorn-host`. Create `acorn-wasm` for target-specific browser adapters and bindings. Split one capability at a time and keep every intermediate workspace state buildable.

## 5. Extract application services

Move artifact discovery, GitLab bot workflows, and agentic/server coordination into `acorn-service` expressed over injected port traits. Keep CLI parsing, terminal presentation, process exit behavior, and concrete adapter selection in interface composition roots.

## 6. Finish the facade and release path

Make `acorn-lib` re-export-only and map its existing feature names and default behavior to the new crates. Migrate first-party interfaces to direct dependencies, retaining the facade where compatibility is being tested. Publish dependency crates in topological order before publishing the facade; Cargo requires published path dependencies to include a registry version and be available to downstream resolution ([Cargo dependency documentation](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-path-dependencies)). Reserve and recheck package names immediately before creating the first public release.

# Compatibility

- Existing `acorn::...` import paths and default CLI behavior remain supported during extraction.
- Re-export host extension traits from the legacy `acorn::io` surface so current trait-based schema reads and writes remain source-compatible.
- Preserve feature names at the facade even when their implementation moves to another crate.
- Map the facade's legacy `wasm` feature to supported portable operations plus `acorn-wasm`; do not propagate it as a feature of every dependency crate.
- Do not remove facade paths or compatibility features without a separately approved breaking-release plan.
- Use aligned workspace versions for the initial package set so facade dependency and release ordering remain explicit.

# Validation

1. Run focused tests after each boundary change and extraction, followed by `make lint` and `make test`.
2. Compile the in-tree Phase 1 boundaries, then `acorn-core` and `acorn-schema`, without default features and with supported alloc combinations; compile browser composition separately through `acorn-wasm` for `wasm32-unknown-unknown`.
3. Inspect resolved feature trees and confirm foundational crates do not contain HTTP, database, process, terminal, embedded-asset, or async-runtime dependencies.
4. Add facade compatibility tests for representative root types, schema paths, analyzer paths, I/O extension traits, API paths, and utility paths.
5. Build and smoke-test the CLI, TUI, Python bindings, and JS/WASM bindings against their direct dependencies and composition roots.
6. Run `cargo package` or `cargo publish --dry-run` for each publishable crate in dependency order.
7. Measure binary and compile-time changes under their existing plans; crate extraction alone is not evidence of a size improvement.

# Acceptance criteria

- Cargo resolves an acyclic package graph matching the documented dependency direction.
- Every ACORN product crate is a direct child of `crates/`, and workspace tooling resolves fixtures and manifests without relying on the former top-level crate paths.
- `acorn-core` and `acorn-schema` compile without host-only services.
- `acorn-wasm` is the only crate that owns browser bindings and concrete browser adapters.
- Schemas do not directly read paths or URLs, and analysis logic does not instantiate concrete host services.
- Provider clients, stores, exporters, host adapters, and workflows have distinct ownership boundaries.
- `acorn-lib` contains compatibility wiring and no independent implementation.
- Existing facade imports, feature behavior, CLI output, file formats, offline behavior, and diagnostics remain compatible.
- First-party interfaces can select their required capabilities without enabling unrelated dependency stacks.
- Service use cases can be composed with either native or browser-supported adapters without depending on `acorn-host` directly.
- Full workspace lint, test, packaging, and supported feature-matrix checks pass.

# Related plans

- [Refactor initiative](initiative.md)
- [Extract acorn-cmd](acorn-cmd-extraction.md)
- [Portability roadmap](portability/roadmap.md)
- [Isolate std-only modules](portability/isolate-std-only-modules.md)
- [Capability preludes](portability/capability-preludes.md)
- [Utility module partitioning](portability/utility-module-partitioning.md)
- [Host abstraction traits](portability/host-abstraction-traits.md)
- [WASM feature path](portability/wasm-feature-path.md)
- [no-std and WASM CI](portability/no-std-wasm-ci.md)
- [Schema I/O validation handoff](core/schema-io-validation-handoff.md)
- [Shared export context](core/export-context.md)
- [Analyzer refactors](analyzer/)
- [API client roadmap](api/roadmap.md)
