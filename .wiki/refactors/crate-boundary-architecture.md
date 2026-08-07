---
type: Plan
title: "Split ACORN along layered crate boundaries"
description: "Decompose acorn-lib into portable domain crates, capability adapters, application services, and a compatibility facade."
difficulty: high
impact: critical
package: [workspace, acorn-lib, acorn-core, acorn-schema, acorn-analysis, acorn-api, acorn-store, acorn-export, acorn-host, acorn-service, acorn-cmd]
tags: [refactor, architecture, crate-extraction, portability, dependencies]
status: planned
---

# Goal

Replace the current all-purpose `acorn-lib` package with a directed set of crates whose dependencies follow domain, adapter, service, and interface layers. Retain `acorn-lib` as the supported feature-gated compatibility facade while first-party applications adopt direct dependencies where doing so avoids unrelated capabilities.

# Current evidence

- `acorn-lib` currently exposes analyzer, I/O, schema, utility, and doctor modules from one package.
- Module dependencies are cyclic: schemas use I/O and utility services, I/O uses schemas and analyzers, and analyzers use I/O, schemas, and utilities.
- `schema` types implement the host-facing `InputOutput` trait and import filesystem operations and the `color-eyre`-backed `ApiResult`, preventing a portable schema boundary.
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
    |
    +--> acorn-analysis
    +--> acorn-api
    +--> acorn-store
    +--> acorn-export
              |
        acorn-host <--- acorn-cmd
              |
        acorn-service
              |
   CLI / TUI / Python bindings
```

The diagram shows the dominant direction rather than every direct edge. `acorn-host` may depend on the analysis, API, and export contracts whose concrete adapters it implements. `acorn-lib` sits beside this graph as an optional re-export facade and must not become an implementation layer.

| Package | Owns | Must not own |
|---|---|---|
| `acorn-core` | Portable errors, location and repository values, MIME/checksum/version types, interpolation, and pure string and collection utilities | Schemas, filesystems, networks, presentation, or embedded assets |
| `acorn-schema` | RAD, PID, standards, hardware, and agent data models; validation; crosswalks; parsing and serialization from strings or bytes | Paths, HTTP, processes, databases, terminal styling, or rich host reports |
| `acorn-analysis` | Check/result types, scoring, summaries, readability, validation folds, and traversal over supplied content | Concrete filesystem, HTTP, database, document, or subprocess implementations |
| `acorn-api` | Endpoint, authentication, request, response, and provider-specific client contracts and DTOs | Persistence, analysis, or bot workflow orchestration |
| `acorn-store` | SQLite and DuckDB adapters, cache and table models, and durable operation persistence | API invocation, presentation, or application workflows |
| `acorn-export` | Shared export context, templates, charts, OOXML/PowerPoint transformations, and renderer contracts that produce artifact bytes | Filesystem writes, browser/process startup, or CLI output |
| `acorn-host` | Filesystem and source handling, HTTP transport, document conversion, process execution, downloads, synchronization, doctor checks, and concrete analysis/export adapters | Domain policy or multi-service orchestration |
| `acorn-service` | Artifact discovery, analysis use cases, GitLab bot/webhook/intake flows, MCP/server workflows, and coordination across APIs, stores, and host adapters | Provider protocol details or database mechanics |
| `acorn-cmd` | Command construction and execution helpers plus the public command macro surface | General-purpose ACORN utilities |
| `acorn-lib` | Compatibility modules, feature mapping, and public re-exports | New implementation logic |

Individual schemas, providers, commands, doctor checks, and generic utility collections do not become separate crates. They remain modules within the package that owns their policy.

# Boundary rules

1. Dependencies must not point from a portable layer to a host or service layer.
2. Pure functions accept data and return values or typed errors; filesystem, network, process, clock, entropy, and rendering side effects remain in adapters.
3. A port trait belongs to the domain or service that consumes the capability. Concrete implementations belong to `acorn-host`, `acorn-store`, or another adapter crate.
4. `acorn-schema` owns format codecs for strings and bytes. Host-owned extension traits or free functions own path and URL reads and writes.
5. `acorn-api` retains provider-specific protocol policy. GitLab bot decisions, persistence, analysis, and report publication belong to `acorn-service`.
6. `acorn-export` produces in-memory artifacts. `acorn-host` materializes them and provides browser- or process-backed renderers.
7. Each crate exposes only capabilities it actually implements. `std` availability must not implicitly enable every host service.
8. `acorn-lib` may map legacy features to new package features, but first-party interfaces should depend directly on the narrow packages they use.
9. Public compatibility shims re-export implementation; they must not duplicate it.

# Migration sequence

## 1. Remove module-level cycles

Complete the portable error and utility separation, move presentation helpers out of pure utilities, separate schema codecs from host reads and writes, and split pure analyzer operations from their host adapters. Move GitLab workflow logic out of the provider client boundary. Keep these changes inside existing packages until the intended dependency graph is acyclic.

## 2. Extract the portable foundation

Extract `acorn-core`, followed by `acorn-schema`. Preserve existing `acorn::schema`, root type, and utility paths through facade re-exports. Establish independent no-std, alloc, std, and WASM checks at this point.

## 3. Extract domain capabilities

Extract `acorn-analysis` after analyzer result and traversal contracts stabilize. Complete the linked `acorn-cmd` extraction independently. Neither package may depend on the compatibility facade.

## 4. Extract adapters

Extract provider clients into `acorn-api`, persistence into `acorn-store`, and renderers into `acorn-export`. Move general host operations and concrete port implementations into `acorn-host`. Split one capability at a time and keep every intermediate workspace state buildable.

## 5. Extract application services

Move artifact discovery, GitLab bot workflows, and agentic/server coordination into `acorn-service`. Keep CLI parsing, terminal presentation, and process exit behavior in the existing interface crates.

## 6. Finish the facade and release path

Make `acorn-lib` re-export-only and map its existing feature names and default behavior to the new crates. Migrate first-party interfaces to direct dependencies, retaining the facade where compatibility is being tested. Publish dependency crates in topological order before publishing the facade; Cargo requires published path dependencies to include a registry version and be available to downstream resolution ([Cargo dependency documentation](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-path-dependencies)). Reserve and recheck package names immediately before creating the first public release.

# Compatibility

- Existing `acorn::...` import paths and default CLI behavior remain supported during extraction.
- Re-export host extension traits from the legacy `acorn::io` surface so current trait-based schema reads and writes remain source-compatible.
- Preserve feature names at the facade even when their implementation moves to another crate.
- Do not remove facade paths or compatibility features without a separately approved breaking-release plan.
- Use aligned workspace versions for the initial package set so facade dependency and release ordering remain explicit.

# Validation

1. Run focused tests after each boundary change and extraction, followed by `make lint` and `make test`.
2. Compile `acorn-core` and `acorn-schema` without default features and with the supported alloc and WASM combinations.
3. Inspect resolved feature trees and confirm foundational crates do not contain HTTP, database, process, terminal, embedded-asset, or async-runtime dependencies.
4. Add facade compatibility tests for representative root types, schema paths, analyzer paths, I/O extension traits, API paths, and utility paths.
5. Build and smoke-test the CLI, TUI, and Python bindings against their direct dependencies.
6. Run `cargo package` or `cargo publish --dry-run` for each publishable crate in dependency order.
7. Measure binary and compile-time changes under their existing plans; crate extraction alone is not evidence of a size improvement.

# Acceptance criteria

- Cargo resolves an acyclic package graph matching the documented dependency direction.
- `acorn-core` and `acorn-schema` compile without host-only services.
- Schemas do not directly read paths or URLs, and analysis logic does not instantiate concrete host services.
- Provider clients, stores, exporters, host adapters, and workflows have distinct ownership boundaries.
- `acorn-lib` contains compatibility wiring and no independent implementation.
- Existing facade imports, feature behavior, CLI output, file formats, offline behavior, and diagnostics remain compatible.
- First-party interfaces can select their required capabilities without enabling unrelated dependency stacks.
- Full workspace lint, test, packaging, and supported feature-matrix checks pass.

# Related plans

- [Refactor initiative](initiative.md)
- [Extract acorn-cmd](acorn-cmd-extraction.md)
- [Portability roadmap](portability/roadmap.md)
- [Isolate std-only modules](portability/isolate-std-only-modules.md)
- [Portable core error surface](portability/core-error-surface.md)
- [Utility module partitioning](portability/utility-module-partitioning.md)
- [Schema I/O validation handoff](core/schema-io-validation-handoff.md)
- [Shared export context](core/export-context.md)
- [Analyzer refactors](analyzer/)
- [API client roadmap](api/roadmap.md)

