---
type: Plan
title: "Split ACORN along layered crate boundaries"
description: "Decompose acorn-lib into portable domain crates, capability adapters, and application services with explicit ownership."
difficulty: high
impact: critical
package: [workspace, acorn-lib, acorn-core, acorn-schema, acorn-analysis, acorn-api, acorn-store, acorn-export, acorn-host, acorn-wasm, acorn-service, acorn-server, acorn-cmd]
tags: [refactor, architecture, crate-extraction, portability, dependencies]
status: in-progress
execution: coordination
---

# Goal

Replace the current all-purpose `acorn-lib` package with a directed set of crates whose dependencies follow domain, adapter, service, and interface layers. Ownership transfers are explicit semver changes: consumers import the owning crate directly, and `acorn-lib` does not mask moves with compatibility re-exports.

# Current priority

This plan is coordination-only. `acorn-schema`, `acorn-cmd`, the initial `acorn-host` capabilities, `acorn-analysis`, `acorn-service`, and the `acorn-server` composition root are extracted with direct first-party imports. The analyzer and service kernels compile independently under portable feature configurations and have direct, WASM, JSON-RPC, and gRPC consumers. The current boundary-consistency pass established schema-owned string/byte codecs, one portable operation/effect model and JSON-RPC registry, portable RAD Markdown decoding, real repository action semantics, and an unconditional gRPC server implementation; the ranked slices below capture the remaining parity work.

# Current evidence

- `acorn-lib` now exposes analyzer, I/O, host utility, and doctor modules; schema implementation has moved to `acorn-schema`, command construction/execution to `acorn-cmd`, and initial filesystem, configuration, HTTP, download, source, and terminal capabilities to `acorn-host`.
- `acorn-schema` now owns the portable `DocumentCodec` contract and RAD Markdown decoder. Service validation, repository normalization, configuration parsing, and host path adapters call those codecs instead of maintaining format-specific serde branches.
- `acorn-core` now owns invocation context, principal, capability effects, and authorization. `acorn-service` owns the sole JSON-RPC framing, method catalog, schema definition, dispatch, and policy implementation; `acorn-lib` retains only host handler and durable-operation adapters.
- Repository `validate`, `check`, `format`, and `link` actions now have distinct behavior, invalid RAD Markdown cannot silently downgrade to generic Markdown, and duplicate or empty action requests fail explicitly. Enrichment remains an explicit denied host capability until its port exists.
- The gRPC command always compiles its real server path; the former empty `grpc` feature and success-returning shell have been removed.
- `crates/acorn-core` now owns portable errors, location/repository values, MIME/checksum/version values, and core/alloc utility implementations. First-party consumers import them directly from `acorn-core`; `acorn-lib` retains only host extensions and host utilities.
- Remaining coupling is concentrated inside `acorn-lib`: analyzer host execution, provider APIs, persistence, export, and workflows still share I/O and utility boundaries.
- The outbound MCP client now combines configuration and operation policy with concrete child-process and HTTP transports in `io::mcp`; extraction must separate service orchestration from native transport without changing the landed MCP contract.
- Schema string/byte codecs remain with schema types, while their path/file `InputOutput` implementations and migration machinery live in the std-only `io::adapter` module.
- The portable foundation is extracted: `AcornError` and `AcornResult` are independent of host reporting, and core/alloc utilities are imported from their owning crate without legacy facade paths.
- Minimal and alloc builds no longer resolve `color-eyre`, `console`, `owo-colors`, `rust-embed`, or `similar`; immutable validation reference data is compiled directly without the generic embedded-asset runtime.
- Default, minimal, alloc, and `wasm32-unknown-unknown` checks pass. The dependency tree excludes the earlier presentation/generic-asset leakage.
- Existing ACORN product crates now live under `crates/`; repository-only tooling such as `xtask` and `fuzz` remains at the workspace root. New packages in this plan have a stable nested destination before extraction begins.
- Utility implementations now have explicit core, alloc, and std ownership; `acorn-lib` no longer exposes `util::core`, `util::portable`, or flat core-utility re-exports.
- ACORN owns one alloc-compatible validation engine and derive; schema validation no longer resolves `validator` or `validator_derive`, generated validation uses core/alloc paths, and deterministic flat reports preserve nested and indexed paths.
- OpenCode path I/O, model-weight persistence, RAD Markdown document indexing, and contextual RAD formatting now live in host adapter modules. Schema license and programming-language validation no longer reads process environment or databases.
- Remaining schema-boundary cleanup is concentrated in agent prompt asset hydration and other feature-dependent transformations; path/file adapters and OpenCode CST preservation already remain outside `acorn-schema`.
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

CLI / TUI  = service + native adapters
JS / WASM  = service + browser adapters
acorn-server = service + selected native adapters + protocol listeners
```

The diagram shows the dominant direction rather than every direct edge. Port traits live with the domain or service that consumes them. `acorn-host`, `acorn-store`, and `acorn-wasm` implement those ports, while interface crates act as composition roots and select concrete adapters. `acorn-service` must not depend on a native adapter. `acorn-server` owns listener, authentication, TLS, health, and shutdown composition for JSON-RPC and gRPC without absorbing application policy. During migration, `acorn-lib` remains a host implementation package; it does not sit beside the graph as a re-export facade.

# Repository layout

All ACORN product and library packages use a flat namespace under `crates/`. The directory nesting expresses workspace ownership; dependency direction remains defined by the package graph rather than directory order.

```text
crates/
├── acorn-cli
├── acorn-lib
├── acorn-macros
├── acorn-tui
├── acorn-core
├── acorn-schema
├── acorn-analysis   (planned)
├── acorn-api        (planned)
├── acorn-store      (planned)
├── acorn-export     (planned)
├── acorn-host
├── acorn-wasm
├── acorn-service    (planned)
├── acorn-server     (planned)
└── acorn-cmd
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
| `acorn-wasm` | `wasm-bindgen` integration, JS-facing DTO conversion, and future browser adapters and composition | Native filesystem/process assumptions or portable domain policy |
| `acorn-service` | Artifact discovery, analysis use cases, GitLab bot/webhook/intake flows, MCP client/server workflows, and orchestration over injected port traits | Concrete host, browser, provider transport, or database implementations |
| `acorn-server` | JSON-RPC and gRPC listeners, authentication, TLS, health, request bounds, tracing setup, and graceful process lifecycle | Portable analysis, application policy, provider logic, or persistence implementation |
| `acorn-cmd` | Command construction and execution helpers plus the public command macro surface; command-word parsing is isolated inside `acorn-macros` | General-purpose ACORN utilities |
| `acorn-lib` | Host implementations awaiting extraction and ACORN-specific host extensions | Portable domains, schemas, or re-exports that conceal another crate's ownership |

Individual schemas, providers, commands, doctor checks, and generic utility collections do not become separate crates. They remain modules within the package that owns their policy.

# Project code boundaries

- **Update:** workspace `Cargo.toml`, `Makefile`, `.gitlab-ci.yml`; all current manifests; `crates/acorn-lib/src/{lib,error}.rs`, `analyzer/`, `io/`, `doctor/`, and `util/`; `crates/acorn-core/src/prelude.rs`; first-party crate imports; package/release documentation and `xtask` workspace assumptions.
- **Create:** the remaining planned packages under `crates/` in dependency order, including the `acorn-server` interface/composition package, with focused unit and boundary tests and explicit feature contracts.
- **Delete:** implementation from `acorn-lib` after consumers migrate directly to the owning crate; do not retain ownership-masking re-exports.

# Plan dependencies

- **Satisfied prerequisite:** the implemented portability matrix proves the analyzer boundary in-tree; that result does not itself authorize package extraction.
- **Boundary inputs:** the analyzer/service/server extraction, `acorn-cmd`, and initial `acorn-host` boundaries are implemented; the [shared API client contract](api/shared-client-contract.md) prepares `acorn-api`; the [shared export and OOXML pipeline](core/export-pipeline-hardening.md) prepares `acorn-export`.
- **Downstream:** the implemented portability matrix expands with each accepted extraction. Browser adapters or consumer-owned host ports require a concrete alternate implementation and a new atomic plan.
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
12. Reusable operations own capability-specific typed errors, using the existing workspace `thiserror` dependency when it reduces handwritten boilerplate. Host and application composition retain `color-eyre`; error conversion is performed within the owning slice rather than a workspace-wide migration.

# Current migration sequence

## 1. Extract the portable analysis and supplied-content service slice

Move the proven analyzer kernel into `acorn-analysis` and build the initial `acorn-service` catalog over supplied content in the same phase. Host those shared operations through retained JSON-RPC semantics and explicit typed gRPC methods in `acorn-server`. Keep native path discovery, persistence implementation, and other adapters outside the portable crates.

## 2. Stabilize provider and export boundaries

Keep endpoint, authentication, request, response, and provider protocol mechanics in the API boundary. Move GitLab bot decisions, persistence coordination, analysis invocation, and report publication into a service boundary. Make export transformations return in-memory artifacts before extracting provider, store, and export packages.

## 3. Extract adapters

Extract provider protocol logic into `acorn-api`, persistence into `acorn-store`, and renderers into `acorn-export`. Move remaining native operations and concrete native port implementations into the existing feature-gated `acorn-host`. Extend the minimal `acorn-wasm` bindings with target-specific browser adapters only as service ports stabilize. Split one capability at a time and keep every intermediate workspace state buildable.

## 4. Expand application services

Move artifact discovery, GitLab bot workflows, and agentic/server coordination into `acorn-service` expressed over injected port traits. Keep CLI parsing, terminal presentation, process exit behavior, and concrete adapter selection in interface composition roots.

## 5. Finish direct consumer migration and the release path

Migrate first-party interfaces to direct dependencies, then remove extracted implementation and obsolete features from `acorn-lib`. Publish dependency crates in topological order before their consumers; Cargo requires published path dependencies to include a registry version and be available to downstream resolution ([Cargo dependency documentation](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-path-dependencies)). Reserve and recheck package names immediately before creating the first public release.

# Ranked remaining consistency slices

These are independently schedulable child slices, ordered by impact and then lower implementation difficulty. Before implementation, each slice must receive an exact write set and an integration owner under the parallel-execution rules.

| Rank | Slice | Impact | Difficulty | Project boundary and current evidence | Acceptance gate |
|---:|---|---|---|---|---|
| 1 | Differential portability parity harness | High | Medium | Exercise identical schema codec and service requests through alloc/native service, JSON-RPC, gRPC, and WASM adapters; current tests are per-surface and can miss semantic drift. | Shared fixtures assert identical values, diagnostics, ordering, file changes, and policy errors across every supported surface in the feature matrix. |
| 2 | Canonical diagnostics and readability contracts | High | Medium | Move remaining analyzer/host diagnostic and readability translations behind the `acorn-analysis` and `acorn-service` DTOs; current transport-specific projections can still diverge. | Direct, JSON-RPC, gRPC, CLI, and WASM consumers serialize one owned model with snapshot coverage and no duplicate public DTO. |
| 3 | Explicit validation context | High | Medium | Replace ambient time and other environment-sensitive validation inputs with a portable `ValidationContext` supplied by callers. | Temporal checks are deterministic under a fixed context, browser/native results match, and schema validation does not read the system clock or process environment. |
| 4 | Transport catalog parity | High | Medium | Generate or verify JSON-RPC and gRPC exposure from the service catalog, including `ValidateStandard`; current listeners still register methods separately. | A catalog test fails on missing, extra, renamed, or schema-incompatible operations, and every intended typed service operation is reachable through both transports. |
| 5 | Canonical exact and host-fuzzy algorithms | High | Medium | Keep deterministic exact matching in portable owners and isolate optional fuzzy/search dependencies in host adapters; similar matching logic remains split across layers. | Portable builds have dependency-free deterministic results, host fuzzy behavior is explicitly selected, and shared fixtures document intentional result differences. |
| 6 | Narrow schema standard-library features | High | High | Split the broad `acorn-schema/std` switch into capability-scoped features so codecs and standards do not enable unrelated host-adjacent dependency stacks. | Each supported schema capability compiles in isolation, the resolved tree matches its documented needs, and the default feature set preserves behavior. |
| 7 | End-to-end offline propagation | Medium | Low | Carry the offline decision through service context and every host network adapter instead of relying on command-local checks. | Offline integration tests prove that CLI, service, JSON-RPC, and gRPC entry points perform no network-capable operation and return the same typed denial. |
| 8 | Endpoint security policy consolidation | Medium | Low | Centralize bind-address, TLS, authentication, request-size, and exposure defaults in `acorn-server`; current listener composition retains protocol-specific decisions. | JSON-RPC and gRPC consume one reviewed policy, insecure non-loopback exposure requires explicit configuration, and tests cover default and rejected configurations. |
| 9 | Boundary residue cleanup | Low | Low | Remove unused `ServiceClient` and durable marker abstractions, no-op aliases, stale feature/docs references, and compatibility names after direct consumers are confirmed. | Reference scans find no consumers, public docs name current owners and features, and workspace/package checks pass after deletion without replacement shells. |

The differential harness precedes model and catalog convergence because it supplies the evidence needed to distinguish intentional adapter behavior from drift. Schema feature splitting follows the portable contract work so dependency changes do not obscure semantic changes. Residue cleanup runs last, after the parity gates prove that no compatibility shell remains necessary.

# Compatibility

- Default CLI behavior remains supported, but accepted ownership transfers may intentionally break legacy import paths.
- Host extension traits remain in their implementing host package until deliberately extracted; domain types are always imported from their owner.
- Remove obsolete feature names when their implementation moves rather than silently forwarding them.
- Do not add schema, core, or future extracted-package facades that obscure ownership; use direct dependencies and semver changes.
- Use aligned workspace versions for the initial package set so dependency and release ordering remain explicit.

# Validation

1. Run focused tests after each boundary change and extraction, followed by `make lint` and `make test`.
2. Compile the in-tree portable boundaries and `acorn-schema` without default features and with supported alloc combinations; compile browser composition separately through `acorn-wasm` for `wasm32-unknown-unknown`.
3. Inspect resolved feature trees and confirm foundational crates do not contain HTTP, database, process, terminal, embedded-asset, or async-runtime dependencies.
4. Test direct owning-crate imports plus host I/O extension traits, and scan for ownership-masking re-exports.
5. Build and smoke-test the CLI, TUI, and JS/WASM bindings against their direct dependencies and composition roots.
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

- [Shared export and OOXML pipeline](core/export-pipeline-hardening.md)
- [Shared API client contract](api/shared-client-contract.md)
