---
type: Plan
title: "Introduce a typed scoring model"
description: "Make readability score combination, thresholds, and severity interpretation explicit."
difficulty: high
impact: not-assessed
package: [acorn-lib]
tags: [refactor, analyzer, readability, scoring]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/analyzer/readability/`, check/report score consumers, and tests; later `acorn-analysis`.
- **Create:** constrained score/severity value types and pure combination functions.
- **Delete:** raw numeric score interpretation duplicated at migrated call sites.

# Dependencies

Can follow extraction of portable readability logic; does not block schema/core extraction.

# Parallel execution

The value types, pure combination functions, and tests under `analyzer/readability/` are orthogonal to portable extraction. Claim only that leaf directory in the initial slice. Defer `analyzer/mod.rs`, shared `check.rs` report wiring, and public re-exports to the analyzer integration owner.

# Evidence

`crates/acorn-lib/src/analyzer/readability/mod.rs` and `check.rs` map scores and severity through ad hoc conditionals.

# Recommendation

Define typed score values with explicit combination and threshold operations, then interpret them into `CheckSeverity` in one layer.

# Acceptance criteria

Current scoring results remain stable and threshold behavior has focused boundary tests.
