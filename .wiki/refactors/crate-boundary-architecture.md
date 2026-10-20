---
type: Plan
title: "Split ACORN along layered crate boundaries"
description: "Coordinate the remaining package ownership boundaries and their prerequisite plans."
difficulty: high
impact: critical
package: [workspace, acorn-lib, acorn-core, acorn-schema, acorn-analysis, acorn-host, acorn-wasm, acorn-service, acorn-server, acorn-cmd]
tags: [refactor, architecture, crate-extraction, portability, dependencies]
status: in-progress
execution: coordination
---

# Goal

Coordinate the remaining extraction from `acorn-lib` into portable domains, adapters, services, and interfaces. This record defines package direction and ownership; an atomic executable plan owns each future extraction. Ownership transfers are explicit semver changes: consumers import the owning crate directly, and `acorn-lib` does not mask moves with compatibility re-exports.

# Current evidence

- `acorn-core`, `acorn-schema`, `acorn-analysis`, `acorn-cmd`, `acorn-host`, `acorn-service`, and `acorn-server` exist under `crates/`. Portable values, schema codecs, analysis data, command execution, initial native capabilities, application policy, and protocol listeners have distinct owners.
- `acorn-lib` still combines analyzer host execution, provider clients, persistence, export, and workflows. Its broad `std` feature enables unrelated native capabilities.
- The outbound MCP client still combines service policy with native child-process and HTTP transports; extraction must preserve its landed contract.
- The [shared API client](api/shared-client-contract.md) and [export pipeline](core/export-pipeline-hardening.md) plans prepare contracts for `acorn-api` and `acorn-export`. Neither creates those target packages; `acorn-store` has no executable extraction plan yet.

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
├── acorn-analysis
├── acorn-api        (planned)
├── acorn-store      (planned)
├── acorn-export     (planned)
├── acorn-host
├── acorn-wasm
├── acorn-service
├── acorn-server
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

# Coordination boundaries

This record owns package direction, not implementation files or phases. The [shared API client plan](api/shared-client-contract.md) prepares `acorn-api`; the [export pipeline plan](core/export-pipeline-hardening.md) prepares `acorn-export`; the [CLI composition plan](core/cli-composition-boundary.md) owns interface cutover. Create an atomic extraction plan with current evidence, exact write sets, one primary lock per phase, and five-hour outcomes when a prerequisite contract lands. Do not treat a preparation plan as the extraction assignment.

# Plan dependencies

- **Satisfied prerequisite:** the implemented portability matrix proves the analyzer boundary in-tree; that result does not itself authorize package extraction.
- **Boundary inputs:** the analyzer/service/server extraction, `acorn-cmd`, and initial `acorn-host` boundaries are implemented; the [shared API client contract](api/shared-client-contract.md) prepares `acorn-api`; the [shared export and OOXML pipeline](core/export-pipeline-hardening.md) prepares `acorn-export`. No active plan owns package creation or `acorn-store` extraction.
- **Downstream:** the implemented portability matrix expands with each accepted extraction. Browser adapters or consumer-owned host ports require a concrete alternate implementation and a new atomic plan.
- The complete ordering and shared-file collision map is maintained in the [plan dependency network](../plan-network.md).

# Parallel execution

This architecture record is coordination-only. Before assigning extraction work, create an atomic executable child plan with contract, leaf migration, and integration slices. One integration owner holds workspace manifests, `acorn-lib` roots, module indexes, ownership-cutover deletions, and feature-matrix validation; leaf agents may move disjoint modules only after the target contract is frozen. See the [parallel execution guide](../parallel-execution.md).

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

# Active child plans

- [Shared API client contract](api/shared-client-contract.md) stabilizes endpoint, request, response, error, and provider mechanics before a separate `acorn-api` extraction plan.
- [Shared export and OOXML pipeline](core/export-pipeline-hardening.md) establishes pure in-memory transformations before a separate `acorn-export` extraction plan.
- [CLI composition boundary](core/cli-composition-boundary.md) keeps parsing and presentation at the interface root.
- [Structured research-activity enrichment](core/research-activity-enrichment-integration.md) owns its schema/service seam.
- [Portable diff hardening](acorn-diff-resource-and-api-hardening.md) and [ZON codec hardening](zon-codec-hardening.md) remain independent portable leaves.

No package-extraction work is scheduled by this record. Create a separate atomic plan with current code evidence, one outcome, exact ownership, and focused validation before promoting it to the portfolio.

# Compatibility

- Default CLI behavior remains supported, but accepted ownership transfers may intentionally break legacy import paths.
- Host extension traits remain in their implementing host package until deliberately extracted; domain types are always imported from their owner.
- Remove obsolete feature names when their implementation moves rather than silently forwarding them.
- Do not add schema, core, or future extracted-package facades that obscure ownership; use direct dependencies and semver changes.
- Use aligned workspace versions for the initial package set so dependency and release ordering remain explicit.

# Extraction-plan requirements

Each future package extraction must name the exact current modules and their target owner, move one independently useful boundary, preserve direct first-party imports, and pass focused portability and behavior checks. Its final integration phase owns manifests, feature matrices, package checks, CLI/TUI/WASM consumers, and retirement of that executable plan. This coordination record remains until the target ownership map has no remaining extraction work.

# Related plans

- [Shared export and OOXML pipeline](core/export-pipeline-hardening.md)
- [Shared API client contract](api/shared-client-contract.md)
