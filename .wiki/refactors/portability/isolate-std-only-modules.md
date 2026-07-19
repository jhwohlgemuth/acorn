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

# Context

Pure JSON and JSONC detection no longer imports the std-gated I/O module, but broader analyzer and I/O boundaries still mix domain logic with host behavior.

# Evidence

`acorn-lib/src/analyzer/mod.rs` still coordinates process, network, and filesystem-like behavior alongside transformations.

# Recommendation

1. Move host-heavy logic behind std adapters and keep pure transformations in portable modules.
2. Keep schema parsing, validation, and formatting independent of command, process, and network concerns.
3. Route host operations through explicit traits.

# Validation

Run `cargo check -p acorn-lib --no-default-features` and the capability-specific checks in the [portability roadmap](roadmap.md).

# Acceptance criteria

Portable modules compile without importing std-only services.
