---
type: Plan
title: "Tighten schema I/O validation handoff"
description: "Make CLI entry points rely consistently on library-owned strict format validation."
difficulty: high
impact: low
package: [acorn-cli, acorn-lib]
tags: [refactor, schema, validation, io]
status: planned
---

# Evidence

- `crates/acorn-lib/src/io/mod.rs`
- `crates/acorn-lib/src/schema/research_activity/mod.rs`
- `crates/acorn-cli/src/commands/export/mod.rs`

# Recommendation

Extend the library I/O boundary with strict read-and-validate operations and make CLI consumers use that boundary consistently.

# Acceptance criteria

User-facing reads apply the same format and schema validation regardless of the invoking command.
