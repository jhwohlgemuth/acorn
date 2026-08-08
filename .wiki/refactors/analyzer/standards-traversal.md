---
type: Plan
title: "Unify traversal across analyzed standards"
description: "Share traversal strategy while retaining per-standard reading and URL extraction adapters."
difficulty: high
impact: not-assessed
package: [acorn-lib]
tags: [refactor, analyzer, standards]
status: planned
---

# Evidence

`crates/acorn-lib/src/analyzer/mod.rs` has near-isomorphic check flows for CFF, Research Activity, DOCX, and text inputs.

# Recommendation

Define a generic traversal over readable artifacts and check categories after the smaller check-batch and error-fold abstractions are stable.

# Acceptance criteria

Standard implementations retain only reading and extraction differences while common check orchestration is shared.

# Related plans

- [Check batch monoid](check-batch-monoid.md)
- [Validation error tree fold](validation-error-tree-fold.md)
