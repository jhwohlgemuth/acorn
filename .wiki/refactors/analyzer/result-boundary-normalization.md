---
type: Plan
title: "Normalize analyzer result boundaries"
description: "Use one conversion and recovery pipeline for analyzer and Vale failures."
difficulty: medium
impact: not-assessed
package: [acorn-lib]
tags: [refactor, analyzer, errors]
status: planned
---

# Evidence

`acorn-lib/src/analyzer/mod.rs` and `vale.rs` contain repeated matches that convert failures into defaults or checks.

# Recommendation

Standardize error mapping and explicit recovery combinators at conversion boundaries.

# Acceptance criteria

Equivalent failures produce equivalent checks or propagated errors from one shared policy.
