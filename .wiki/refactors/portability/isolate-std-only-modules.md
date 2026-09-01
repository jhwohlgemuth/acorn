---
type: Plan
title: "Isolate std-only modules from domain modules"
description: "Complete the boundary between portable transformations and filesystem, process, and network services."
difficulty: medium
impact: critical
package: [acorn-lib, acorn-core, acorn-schema, acorn-analysis, acorn-host]
tags: [refactor, no-std, portability]
status: in-progress
priority: next
---

# Evidence

- `prelude::core`, `prelude::alloc`, and `prelude::std` now expose explicit capability boundaries. Schema and portable utility modules import allocation-backed types directly, while host helpers import the std prelude.
- `acorn::validation` provides alloc-compatible contracts, ordered flat reports, built-in scalar/PID/version rules, and indexed collection validation; the derive emits core/alloc paths.
- Portable string/byte codecs remain on schema types, while path/file `InputOutput` implementations, migration machinery, RAiD read/conversion, and `format_with` live in the std-only `io::adapter` module.
- `acorn-core` owns portable errors, location/repository values, and core/alloc utility implementations. `acorn-lib` supplies native `LocationExt` and `RepositoryExt` adapters.
- OpenCode `InputOutput`, model-weight persistence, RAD contextual formatting, and RAD Markdown document indexing have moved out of schema modules. License/programming-language schema validation is deterministic and database-independent.
- Remaining schema host coupling includes agent source resolution, embedded prompt rendering, OpenCode CST/path state, and presentation side effects.
- `analyzer/mod.rs`, `check.rs`, `error.rs`, and `service.rs` still coordinate transformations with processes, async sleep, network checks, files, documents, and databases. The entire analyzer feature currently implies `std`.
- Vale output DTOs now have analyzer-model ownership and are re-exported from the existing `analyzer::vale` path; Vale runtime/configuration remains host-owned.

# Project code boundaries

- **Update:** `crates/acorn-lib/Cargo.toml`; `src/lib.rs`, `prelude.rs`, and `error.rs`; `src/schema/`, `src/validation/`, `src/analyzer/`, and `src/io/`; focused tests adjacent to each moved boundary.
- **Create:** adapter boundaries for the remaining schema and analyzer host capabilities; later `crates/acorn-schema/`, `crates/acorn-analysis/`, and native implementations in `crates/acorn-host/`.
- **Delete:** remaining portable-to-host imports after compatibility adapters exist; no legacy public path is removed.

# Dependencies

- **Prerequisite:** keep [dependency feature defaults](dependency-feature-defaults.md) green for minimal and alloc builds.
- **Follow-on:** introduce [host abstraction traits](host-abstraction-traits.md) at proven seams, then extract crates through the [crate boundary architecture](../crate-boundary-architecture.md).

# Parallel execution

Analyzer isolation is a separate batch and owns `analyzer/mod.rs`, `check.rs`, and `service.rs`. Schema isolation owns its remaining std-gated signatures and host-coupled modules. Leaf work outside those files may proceed according to the [parallel execution guide](../../parallel-execution.md).

# Recommendation

1. Move agent source resolution and embedded prompt/template rendering out of portable agent DTOs.
2. Split OpenCode's portable serializable configuration from CST preservation and source-path state; keep file dispatch and comment-preserving writes in std adapters.
3. Move presentation/rendering side effects out of schema types, leaving deterministic view or artifact inputs at the portable boundary.
4. Freeze the resulting schema/validation/codec contract and hand it to the [`acorn-schema` extraction](acorn-schema-extraction.md).
5. Continue analyzer isolation as the following slice: separate `Check` and analyzer issue models from document paths/rendering, then move Vale, HTTP, filesystem, process, and database execution behind adapters.

# Validation

Run `cargo check -p acorn-lib --no-default-features`, `cargo check -p acorn-lib --no-default-features --features alloc`, `make wasm`, and the capability-specific checks in the [portability roadmap](roadmap.md).

# Acceptance criteria

Portable schema modules compile without concrete std services and satisfy the entry gate for `acorn-schema`; analyzer model modules compile independently of concrete document, renderer, filesystem, network, process, and database services.

# Related plans

- [Plan dependency network](../../plan-network.md)
- [Extract acorn-schema](acorn-schema-extraction.md)
