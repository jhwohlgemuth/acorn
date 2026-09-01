---
type: Plan
title: "Extract acorn-schema from the compatibility library"
description: "Move portable schema models, codecs, crosswalks, and validation into acorn-schema while retaining acorn-lib facade paths."
difficulty: high
impact: critical
package: [workspace, acorn-lib, acorn-core, acorn-macros, acorn-schema]
tags: [refactor, crate-extraction, schema, no-std, portability]
status: planned
priority: next
---

# Goal

Create `crates/acorn-schema` as the portable owner of RAD, PID, standards, hardware, agent data models, validation, crosswalks, and string/byte codecs. Preserve existing `acorn::schema` and `acorn::validation` paths through `acorn-lib` re-exports.

# Current evidence

- `acorn-core` already owns portable errors, root values, validation primitives, and core/alloc utilities.
- Schema path/file `InputOutput` implementations, migration, OpenCode file I/O, model-weight persistence, contextual RAD formatting, and RAD Markdown document indexing now live in std-only adapters.
- Schema validation is allocation-compatible and no longer reads environment or database state.
- Remaining extraction blockers are agent source resolution, embedded template rendering, OpenCode CST/path state, presentation side effects, and feature wiring that still assumes `acorn-lib` ownership.
- `acorn-lib` already maps `core` and `alloc` to `acorn-core` and provides the compatibility pattern needed for the next package.

# Project code boundaries

- **Update:** workspace `Cargo.toml`, `Cargo.lock`, `Makefile`, `crates/acorn-lib/Cargo.toml`, `crates/acorn-lib/src/{lib,prelude}.rs`, `crates/acorn-lib/src/{schema,validation}/`, std-only adapters under `crates/acorn-lib/src/io/`, `crates/acorn-macros/src/lib.rs`, first-party manifests/imports, and focused compatibility tests.
- **Create:** `crates/acorn-schema/Cargo.toml`, its library/prelude roots, portable schema and validation modules, package tests, and explicit `core`, `alloc`, and supported `std` feature contracts.
- **Delete:** schema and validation implementation from `acorn-lib` only after facade re-exports and adapter extension points pass compatibility tests; delete no public facade path.

# Dependencies

- **Immediate prerequisite:** [std-only isolation](isolate-std-only-modules.md) removes the named agent/template/CST/presentation blockers.
- **Manifest prerequisite:** [dependency feature defaults](dependency-feature-defaults.md) confirms the extracted package resolves only its selected core/alloc capabilities.
- **Follow-on:** portable analyzer models may depend directly on `acorn-schema`; [host abstraction traits](host-abstraction-traits.md) and `acorn-host` implement proven external capability seams.
- The package graph and release order remain governed by the [crate boundary architecture](../crate-boundary-architecture.md).

# Recommendation

1. Finish the remaining schema std-isolation slices without creating the new package.
2. Freeze the portable schema, validation, codec, and derive-macro contract and record its exact module set.
3. Create `acorn-schema` with explicit core/alloc dependency features, then move one coherent module group at a time without redesigning public types.
4. Replace `acorn-lib` implementations with re-exports and keep host extension implementations in std-only adapter modules.
5. Migrate first-party crates only after facade compatibility and the full portability matrix pass.

# Parallel execution

The extraction owns the workspace-graph, library-root, schema/validation module-index, and CI/build integration locks. Portable leaf moves may fan out only after the package contract is frozen; one integration owner wires manifests, facade re-exports, module indexes, and compatibility tests. Follow the [parallel execution guide](../../parallel-execution.md).

# Validation

Run focused schema and validation tests after each module group, then verify `acorn-schema` and the facade with no default features, `alloc`, supported `std`, and `wasm32-unknown-unknown` combinations. Inspect dependency trees for host reporting, terminal, database, process, native HTTP, async-runtime, and generic embedded-asset leakage. Finish with `make lint`, `make test`, `make wasm`, facade compatibility tests, and package dry runs.

# Acceptance criteria

- `acorn-schema` owns schema and validation implementation and does not depend on `acorn-lib`.
- Minimal and alloc builds contain no filesystem, network, process, database, terminal, native renderer, or generic embedded-asset dependencies.
- String/byte codecs remain available on schema types; path and URL I/O remains in host adapters.
- `acorn-lib` preserves representative `acorn::schema`, `acorn::validation`, derive, serialization, and validation behavior through re-exports.
- First-party interfaces and the supported feature matrix compile and test without behavior or format regressions.

# Related plans

- [Crate boundary architecture](../crate-boundary-architecture.md)
- [Portability roadmap](roadmap.md)
- [Plan dependency network](../../plan-network.md)
