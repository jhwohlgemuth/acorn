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

# Evidence

`acorn-lib/src/analyzer/readability/mod.rs` and `check.rs` map scores and severity through ad hoc conditionals.

# Recommendation

Define typed score values with explicit combination and threshold operations, then interpret them into `CheckSeverity` in one layer.

# Acceptance criteria

Current scoring results remain stable and threshold behavior has focused boundary tests.
