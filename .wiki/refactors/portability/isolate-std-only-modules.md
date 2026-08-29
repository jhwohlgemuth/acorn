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
- Phase 1.1 validation isolation is complete. `acorn::validation` provides alloc-compatible contracts, ordered flat reports, built-in scalar/PID/version rules, and indexed collection validation; the derive emits core/alloc paths and the no-default `extern crate std` compatibility block is gone.
- The minimal, alloc, and WASM checks currently fail at `schema/research_activity/output.rs` because it imports data-only `License` from std-gated `io`; `License` is presently defined in `io/mod.rs` despite being serialized schema data.
- `schema` still owns path-based `InputOutput` implementations in `research_activity/mod.rs`, `standard/{cff,datacite,dcat,huwise,invenio,text}.rs`, and `pid/raid/mod.rs`.
- `schema/mod.rs` still exposes std-only `PathBuf` behavior, while `lib.rs` gives `Location` filesystem-aware methods under `std`.
- `analyzer/mod.rs`, `check.rs`, `error.rs`, and `service.rs` still coordinate transformations with processes, async sleep, network checks, files, documents, and databases. The entire analyzer feature currently implies `std`.

# Project code boundaries

- **Update:** `crates/acorn-lib/Cargo.toml`; `src/lib.rs`, `prelude.rs`, and `error.rs`; `src/schema/`, `src/validation/`, `src/analyzer/`, and `src/io/`; focused tests adjacent to each moved boundary.
- **Create:** internal codec/adapter modules under the existing crate during Phase 1; later `crates/acorn-core/`, `crates/acorn-schema/`, `crates/acorn-analysis/`, and native implementations in `crates/acorn-host/`.
- **Delete:** schema-owned path/file implementations and portable-to-host imports after compatibility adapters exist; no legacy public path is removed.

# Dependencies

- **Prerequisite:** keep [dependency feature defaults](dependency-feature-defaults.md) green for minimal and alloc builds.
- **Immediate coupled plan:** [schema I/O validation handoff](../core/schema-io-validation-handoff.md) is the first implementation slice.
- **Follow-on:** introduce [host abstraction traits](host-abstraction-traits.md) at proven seams, then extract crates through the [crate boundary architecture](../crate-boundary-architecture.md).

# Parallel execution

Do not schedule this broad plan beside the schema I/O handoff; Phase 1.2 is its current implementation slice. Later analyzer isolation is a separate batch and owns `analyzer/mod.rs`, `check.rs`, and `service.rs`. Leaf work outside those files may proceed according to the [parallel execution guide](../../parallel-execution.md).

# Recommendation

1. Completed: replace `validator` with ACORN-owned allocation-backed validation and remove the no-default compatibility block.
2. Phase 1.2 first unblocker: move `License` to schema/core ownership, retain `acorn::io::License` as a compatibility re-export, and restore all three portable checks.
3. Move schema path and file operations behind std adapters while retaining string/byte codecs with schema types.
4. Separate analyzer models, readability, validation folds, and supplied-content traversal from Vale, HTTP, filesystem, document, and database adapters.
5. Introduce focused host traits only where analysis or schema workflows must consume an external capability.

# Validation

Run `cargo check -p acorn-lib --no-default-features`, `cargo check -p acorn-lib --no-default-features --features alloc`, `make wasm`, and the capability-specific checks in the [portability roadmap](roadmap.md).

# Acceptance criteria

Schemas do not own path or file access, and pure analysis compiles without concrete std services.

# Related plans

- [Plan dependency network](../../plan-network.md)
