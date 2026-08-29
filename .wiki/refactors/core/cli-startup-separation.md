---
type: Plan
title: "Separate CLI startup concerns"
description: "Extract startup services so the binary entry point remains orchestration glue."
difficulty: medium
impact: high
package: [acorn-cli]
tags: [refactor, cli, startup]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-cli/src/main.rs`, `src/cli/`, logging/runtime/bootstrap helpers, and binary tests.
- **Create:** focused startup/configuration functions or modules within `acorn-cli`.
- **Delete:** orchestration details from `main.rs` after equivalent tests cover initialization order.

# Dependencies

Use [parse/execution models](cli-parse-execution-models.md); exposes the seam for [CLI error boundary](cli-error-boundary.md) and API environment bootstrap.

# Evidence

`crates/acorn-cli/src/main.rs` configures logging, the runtime and thread pool, database state, preflight checks, and command dispatch.

# Recommendation

Extract focused setup functions or modules for logging, runtime/thread-pool configuration, database bootstrap, and command dispatch. Keep one uniform error boundary in `main`.

# Acceptance criteria

Startup units can be tested independently and `main` only coordinates their results.

# Related plans

- [CLI error boundary](cli-error-boundary.md)
- [CLI parse and execution models](cli-parse-execution-models.md)
