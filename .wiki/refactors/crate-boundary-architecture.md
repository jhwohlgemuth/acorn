---
type: Plan
title: "Split ACORN along layered crate boundaries"
description: "Decompose acorn-lib into portable domain crates, capability adapters, and application services with explicit ownership."
difficulty: high
impact: critical
package: [workspace, acorn-lib, acorn-core, acorn-schema, acorn-analysis, acorn-api, acorn-store, acorn-export, acorn-host, acorn-wasm, acorn-service, acorn-cmd]
tags: [refactor, architecture, crate-extraction, portability, dependencies]
status: in-progress
---

# Goal

Replace the current all-purpose `acorn-lib` package with a directed set of crates whose dependencies follow domain, adapter, service, and interface layers. Ownership transfers are explicit semver changes: consumers import the owning crate directly, and `acorn-lib` does not mask moves with compatibility re-exports.

# Current priority

This plan is coordination-only. `acorn-schema` extraction is complete with direct first-party imports and no `acorn-lib` schema facade. The in-tree analyzer kernel now compiles independently under `alloc`; reassess its concrete boundary before extracting `acorn-analysis` or introducing generic host traits.

# Current evidence

- `acorn-lib` currently exposes analyzer, I/O, schema, utility, and doctor modules from one package.
- `crates/acorn-core` now owns portable errors, location/repository values, MIME/checksum/version values, and core/alloc utility implementations. First-party consumers import them directly from `acorn-core`; `acorn-lib` retains only host extensions and host utilities.
- Module dependencies are cyclic: schemas use I/O and utility services, I/O uses schemas and analyzers, and analyzers use I/O, schemas, and utilities.
- Schema string/byte codecs remain with schema types, while their path/file `InputOutput` implementations and migration machinery live in the std-only `io::adapter` module.
- The portable foundation is extracted: `AcornError` and `AcornResult` are independent of host reporting, and core/alloc utilities are imported from their owning crate without legacy facade paths.
- Minimal and alloc builds no longer resolve `color-eyre`, `console`, `owo-colors`, `rust-embed`, or `similar`; immutable validation reference data is compiled directly without the generic embedded-asset runtime.
- Default, minimal, alloc, and `wasm32-unknown-unknown` checks pass. The dependency tree excludes the earlier presentation/generic-asset leakage.
- Existing ACORN product crates now live under `crates/`; repository-only tooling such as `xtask` and `fuzz` remains at the workspace root. New packages in this plan have a stable nested destination before extraction begins.
- Utility implementations now have explicit core, alloc, and std ownership; `acorn-lib` no longer exposes `util::core`, `util::portable`, or flat core-utility re-exports.
- ACORN owns one alloc-compatible validation engine and derive; schema validation no longer resolves `validator` or `validator_derive`, generated validation uses core/alloc paths, and deterministic flat reports preserve nested and indexed paths.
- OpenCode path I/O, model-weight persistence, RAD Markdown document indexing, and contextual RAD formatting now live in host adapter modules. Schema license and programming-language validation no longer reads process environment or databases.
- Remaining schema host couplings are concentrated in agent source resolution, templates/CST path state, presentation side effects, and other feature-dependent transformations.
- Analyzer Vale output DTOs are separated from Vale runtime configuration, but path analysis, rendering, discovery, and `analysis -> std` remain host-coupled.
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

The diagram shows the dominant direction rather than every direct edge. Port traits live with the domain or service that consumes them. `acorn-host`, `acorn-store`, and `acorn-wasm` implement those ports, while interface crates act as composition roots and select concrete adapters. `acorn-service` must not depend on a native adapter. During migration, `acorn-lib` remains a host implementation package; it does not sit beside the graph as a re-export facade.

# Repository layout

All ACORN product and library packages use a flat namespace under `crates/`. The directory nesting expresses workspace ownership; dependency direction remains defined by the package graph rather than directory order.

```text
crates/
├── acorn-cli
├── acorn-lib
├── acorn-macros
├── acorn-py
├── acorn-tui
├── acorn-core
├── acorn-schema
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
| `acorn-lib` | Host implementations awaiting extraction and ACORN-specific host extensions | Portable domains, schemas, or re-exports that conceal another crate's ownership |

Individual schemas, providers, commands, doctor checks, and generic utility collections do not become separate crates. They remain modules within the package that owns their policy.

# Project code boundaries

- **Update:** workspace `Cargo.toml`, `Makefile`, `.gitlab-ci.yml`; all current manifests; `crates/acorn-lib/src/{lib,error}.rs`, `analyzer/`, `io/`, `doctor/`, and `util/`; `crates/acorn-core/src/prelude.rs`; first-party crate imports; package/release documentation and `xtask` workspace assumptions.
- **Create:** the remaining planned packages under `crates/` in dependency order, with focused unit and boundary tests and explicit feature contracts.
- **Delete:** implementation from `acorn-lib` after consumers migrate directly to the owning crate; do not retain ownership-masking re-exports.

# Plan dependencies

- **Satisfied prerequisite:** the [portability roadmap](portability/roadmap.md) now proves the analyzer boundary in-tree; that result does not itself authorize package extraction.
- **Boundary inputs:** analyzer plans first prove an in-tree portable kernel; the [API roadmap](api/roadmap.md) prepares `acorn-api`; export context/interpolation/OOXML plans prepare `acorn-export`; [acorn-cmd extraction](acorn-cmd-extraction.md) prepares `acorn-cmd`.
- **Downstream:** [host traits](portability/host-abstraction-traits.md), [WASM](portability/wasm-feature-path.md), and [portable CI](portability/no-std-wasm-ci.md) expand as packages appear.
- The complete ordering and shared-file collision map is maintained in the [plan dependency network](../plan-network.md).

# Parallel execution

This architecture plan is coordination-only except when a named migration phase is scheduled. Before assigning work, divide the phase into contract, leaf migration, and integration slices. One integration owner holds workspace manifests, `acorn-lib` roots, module indexes, ownership-cutover deletions, and feature-matrix validation; leaf agents may move disjoint modules only after the target contract is frozen. See the [parallel execution guide](../parallel-execution.md).

# Boundary rules

1. Dependencies must not point from a portable layer to a host or service layer.
2. Pure functions accept data and return values or typed errors; filesystem, network, process, clock, entropy, and rendering side effects remain in adapters.
3. A port trait belongs to the domain or service that consumes the capability. Concrete implementations belong to `acorn-host`, `acorn-store`, `acorn-wasm`, or another adapter crate selected by a composition root.
4. `acorn-schema` owns format codecs for strings and bytes. Host-owned extension traits or free functions own path and URL reads and writes.
5. `acorn-api` retains provider-specific protocol policy. GitLab bot decisions, persistence, analysis, and report publication belong to `acorn-service`.
6. `acorn-export` produces in-memory artifacts. `acorn-host` materializes them with native renderers; `acorn-wasm` exposes only browser-supported rendering and delivery adapters.
7. Each crate exposes only capabilities it actually implements. `std` availability must not implicitly enable every host service.
8. First-party interfaces depend directly on the narrow packages they use; `acorn-lib` does not map moved ownership back onto legacy paths.
9. Public compatibility shims are not used for ownership transfers; accepted semver changes make the new owner visible.
10. Browser dependencies are target-specific to `acorn-wasm`; portable crates must not gain a `wasm-bindgen` or `js-sys` dependency merely because they compile for a WASM target.
11. New ACORN packages are created as `crates/<package-name>`; product crates must not return to the workspace root.

# Current migration sequence

## 1. Completed: extract acorn-schema

`acorn-schema` owns RAD, PID, standards, hardware and agent models, validation, crosswalks, codecs, vocabulary/geonames data, and prompt templates. First-party crates depend on it directly. Host-only resolution, persistence, filesystem adapters, OpenCode CST state, and presentation remain in `acorn-lib`; no schema compatibility module masks the ownership transfer.

### Phase 1 exit gate

- `acorn-schema` does not import `acorn-lib`, concrete I/O, host reporting, terminal, database, process, native HTTP, async-runtime, or generic embedded-asset dependencies.
- String/byte codecs remain domain-owned; path and URL I/O remains adapter-owned.
- Formats, offline behavior, and diagnostics remain compatible; schema import paths intentionally move to the owning crate.
- Minimal, alloc, supported std, and WASM checks plus focused tests, `make lint`, and `make test` pass.

## 2. Prove analysis isolation in-tree; extract acorn-cmd independently

Within `acorn-lib`, separate checks, scoring, summaries, readability, and validation folds from path traversal, Vale execution, HTTP link checking, document loading, rendering, database discovery, and filesystem handling. Require the model/calculation subset to compile under an alloc-only feature before scheduling `acorn-analysis`. Keep host code concrete during this phase; introduce no generic host trait family. Extract `acorn-cmd` independently after its command contracts stabilize.

## 3. Stabilize protocol and export boundaries

Keep endpoint, authentication, request, response, and provider protocol mechanics in the API boundary. Move GitLab bot decisions, persistence coordination, analysis invocation, and report publication into a service boundary. Make export transformations return in-memory artifacts before extracting provider, store, and export packages.

## 4. Extract adapters

Extract provider protocol logic into `acorn-api`, persistence into `acorn-store`, and renderers into `acorn-export`. Move native operations and concrete native port implementations into `acorn-host`. Create `acorn-wasm` for target-specific browser adapters and bindings. Split one capability at a time and keep every intermediate workspace state buildable.

## 5. Extract application services

Move artifact discovery, GitLab bot workflows, and agentic/server coordination into `acorn-service` expressed over injected port traits. Keep CLI parsing, terminal presentation, process exit behavior, and concrete adapter selection in interface composition roots.

## 6. Finish direct consumer migration and the release path

Migrate first-party interfaces to direct dependencies, then remove extracted implementation and obsolete features from `acorn-lib`. Publish dependency crates in topological order before their consumers; Cargo requires published path dependencies to include a registry version and be available to downstream resolution ([Cargo dependency documentation](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-path-dependencies)). Reserve and recheck package names immediately before creating the first public release.

# Compatibility

- Default CLI behavior remains supported, but accepted ownership transfers may intentionally break legacy import paths.
- Host extension traits remain in their implementing host package until deliberately extracted; domain types are always imported from their owner.
- Remove obsolete feature names when their implementation moves rather than silently forwarding them.
- Do not add schema, core, or future extracted-package facades that obscure ownership; use direct dependencies and semver changes.
- Use aligned workspace versions for the initial package set so dependency and release ordering remain explicit.

# Validation

1. Run focused tests after each boundary change and extraction, followed by `make lint` and `make test`.
2. Compile the in-tree Phase 1 boundaries and `acorn-schema` without default features and with supported alloc combinations; compile browser composition separately through `acorn-wasm` for `wasm32-unknown-unknown`.
3. Inspect resolved feature trees and confirm foundational crates do not contain HTTP, database, process, terminal, embedded-asset, or async-runtime dependencies.
4. Test direct owning-crate imports plus host I/O extension traits, and scan for ownership-masking re-exports.
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
- `acorn-lib` contains no schema implementation or schema re-export facade.
- CLI behavior, file formats, offline behavior, and diagnostics remain compatible; import and feature paths may change at explicit semver boundaries.
- First-party interfaces can select their required capabilities without enabling unrelated dependency stacks.
- Service use cases can be composed with either native or browser-supported adapters without depending on `acorn-host` directly.
- Full workspace lint, test, packaging, and supported feature-matrix checks pass.

# Related plans

- [Refactor initiative](initiative.md)
- [Extract acorn-cmd](acorn-cmd-extraction.md)
- [Portability roadmap](portability/roadmap.md)
- [Host abstraction traits](portability/host-abstraction-traits.md)
- [WASM feature path](portability/wasm-feature-path.md)
- [no-std and WASM CI](portability/no-std-wasm-ci.md)
- [Shared export context](core/export-context.md)
- [Analyzer refactors](analyzer/)
- [API client roadmap](api/roadmap.md)
