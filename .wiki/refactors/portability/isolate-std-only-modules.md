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
- `schema/mod.rs` still exposes std-only `PathBuf` signatures, and `lib.rs` gives `Location` filesystem-aware methods under `std`.
- `analyzer/mod.rs`, `check.rs`, `error.rs`, and `service.rs` still coordinate transformations with processes, async sleep, network checks, files, documents, and databases. The entire analyzer feature currently implies `std`.

# Project code boundaries

- **Update:** `crates/acorn-lib/Cargo.toml`; `src/lib.rs`, `prelude.rs`, and `error.rs`; `src/schema/`, `src/validation/`, `src/analyzer/`, and `src/io/`; focused tests adjacent to each moved boundary.
- **Create:** adapter boundaries for the remaining schema and analyzer host capabilities; later `crates/acorn-core/`, `crates/acorn-schema/`, `crates/acorn-analysis/`, and native implementations in `crates/acorn-host/`.
- **Delete:** remaining portable-to-host imports after compatibility adapters exist; no legacy public path is removed.

# Dependencies

- **Prerequisite:** keep [dependency feature defaults](dependency-feature-defaults.md) green for minimal and alloc builds.
- **Follow-on:** introduce [host abstraction traits](host-abstraction-traits.md) at proven seams, then extract crates through the [crate boundary architecture](../crate-boundary-architecture.md).

# Parallel execution

Analyzer isolation is a separate batch and owns `analyzer/mod.rs`, `check.rs`, and `service.rs`. Schema isolation owns its remaining std-gated signatures and host-coupled modules. Leaf work outside those files may proceed according to the [parallel execution guide](../../parallel-execution.md).

# Recommendation

1. Move the remaining `schema/mod.rs` `PathBuf` signatures, `schema/agent` and `schema/validate` I/O couplings, and `research_activity/markdown.rs` document parsing behind adapters.
2. Separate analyzer models, readability, validation folds, and supplied-content traversal from Vale, HTTP, filesystem, document, and database adapters.
3. Introduce focused host traits only where analysis or schema workflows must consume an external capability.

# Validation

Run `cargo check -p acorn-lib --no-default-features`, `cargo check -p acorn-lib --no-default-features --features alloc`, `make wasm`, and the capability-specific checks in the [portability roadmap](roadmap.md).

# Acceptance criteria

Portable schema modules and pure analysis compile without concrete std services.

# Related plans

- [Plan dependency network](../../plan-network.md)
