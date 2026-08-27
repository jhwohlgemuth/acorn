---
type: Plan
title: "Isolate std-only modules from domain modules"
description: "Complete the boundary between portable transformations and filesystem, process, and network services."
difficulty: medium
impact: critical
package: [acorn-lib]
tags: [refactor, no-std, portability]
status: in-progress
priority: next
---

# Evidence

- `prelude::core`, `prelude::alloc`, and `prelude::std` now expose explicit capability boundaries. Schema and portable utility modules import allocation-backed types directly, while host helpers import the std prelude.
- Phase 1.1 validation isolation is complete. `acorn::validation` provides alloc-compatible contracts, ordered flat reports, built-in scalar/PID/version rules, and indexed collection validation; the derive emits core/alloc paths and the no-default `extern crate std` compatibility block is gone.
- `schema` still owns path-based `InputOutput` implementations, and `analyzer` still coordinates transformations with process, network, filesystem, document, and database services.

# Recommendation

1. Completed: replace `validator` with ACORN-owned allocation-backed validation and remove the no-default compatibility block.
2. Phase 1.2: move schema path and file operations behind std adapters while retaining string/byte codecs with schema types.
3. Separate analyzer models, readability, validation folds, and supplied-content traversal from Vale, HTTP, filesystem, document, and database adapters.
4. Introduce focused host traits only where analysis or schema workflows must consume an external capability.

# Validation

Run `cargo check -p acorn-lib --no-default-features`, `cargo check -p acorn-lib --no-default-features --features alloc`, `make wasm`, and the capability-specific checks in the [portability roadmap](roadmap.md).

# Acceptance criteria

Schemas do not own path or file access, and pure analysis compiles without concrete std services.
