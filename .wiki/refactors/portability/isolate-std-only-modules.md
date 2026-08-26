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

- The no-default build still imports `std` for validator derive output.
- `schema` still owns path-based `InputOutput` implementations, and `analyzer` still coordinates transformations with process, network, filesystem, document, and database services.

# Recommendation

1. Establish matching `prelude::core`, `prelude::alloc`, and `prelude::std` imports and resolve the validator derive dependency on `std`.
2. Move schema path and file operations behind std adapters while retaining string/byte codecs with schema types.
3. Separate analyzer models, readability, validation folds, and supplied-content traversal from Vale, HTTP, filesystem, document, and database adapters.
4. Introduce focused host traits only where analysis or schema workflows must consume an external capability.

# Validation

Run `cargo check -p acorn-lib --no-default-features` and the capability-specific checks in the [portability roadmap](roadmap.md).

# Acceptance criteria

Schemas do not own path or file access, and pure analysis compiles without concrete std services.
