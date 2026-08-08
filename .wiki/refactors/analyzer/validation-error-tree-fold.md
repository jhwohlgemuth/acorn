---
type: Plan
title: "Fold validation error trees with one algebra"
description: "Centralize recursive validation-error traversal and path-context handling."
difficulty: medium
impact: not-assessed
package: [acorn-lib]
tags: [refactor, analyzer, validation]
status: planned
---

# Evidence

`crates/acorn-lib/src/analyzer/error.rs` recursively handles field, struct, and list errors with repeated path logic.

# Recommendation

Define one fold over validation error nodes that carries path context and interprets every node variant through a shared algebra.

# Acceptance criteria

Flattened user-facing errors preserve all paths and messages without variant-specific path duplication.
