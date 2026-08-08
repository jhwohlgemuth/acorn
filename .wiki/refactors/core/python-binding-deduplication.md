---
type: Plan
title: "Reduce Python binding repetition"
description: "Reduce repetitive PID wrapper code after the exposed data shape stabilizes."
difficulty: medium
impact: high
package: [acorn-py]
tags: [refactor, python, bindings]
status: planned
---

# Evidence

`crates/acorn-py/src/lib.rs` repeats wrapper patterns for PID-backed data.

# Recommendation

Complete the intended data exposure shape first, then introduce narrowly scoped macro generation for repeated wrapper implementations.

# Acceptance criteria

Bindings preserve their public behavior while repeated PID wrapper code is generated from one definition.
