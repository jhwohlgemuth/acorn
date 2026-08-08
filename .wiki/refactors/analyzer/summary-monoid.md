---
type: Plan
title: "Introduce a monoid for analyzer summaries"
description: "Fold checks directly into an additive summary record."
difficulty: low
impact: not-assessed
package: [acorn-lib]
tags: [refactor, analyzer, summary]
status: planned
---

# Evidence

`crates/acorn-lib/src/analyzer/check.rs` aggregates category counts through chained mapping and collection.

# Recommendation

Represent a summary as a record whose fields combine by addition and fold checks directly into one value.

# Acceptance criteria

Summary output remains unchanged and aggregation uses one associative combine implementation.
