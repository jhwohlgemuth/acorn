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

# Project code boundaries

- **Update:** `crates/acorn-py/src/lib.rs`, binding tests/docs, and macros only if repeated stable shapes justify them.
- **Create:** narrowly scoped wrapper-generation helpers inside `acorn-py`.
- **Delete:** repeated PID wrapper implementations replaced by generated/shared code; no Python-visible type or method without an explicit compatibility decision.

# Dependencies

Complete before [acorn-py repository extraction](../../migrations/acorn-py-repository-extraction.md); coordinate direct dependencies with the crate split.

# Parallel execution

Wrapper deduplication confined to `crates/acorn-py/src/lib.rs` and binding tests is orthogonal to portable extraction. Do not change workspace/crate manifests or depend on new facade paths in the concurrent slice; those changes wait for the workspace-graph integration owner.

# Evidence

`crates/acorn-py/src/lib.rs` repeats wrapper patterns for PID-backed data.

# Recommendation

Complete the intended data exposure shape first, then introduce narrowly scoped macro generation for repeated wrapper implementations.

# Acceptance criteria

Bindings preserve their public behavior while repeated PID wrapper code is generated from one definition.
