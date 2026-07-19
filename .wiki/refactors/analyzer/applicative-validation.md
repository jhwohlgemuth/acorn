---
type: Plan
title: "Compose independent checks applicatively"
description: "Accumulate independent analyzer failures in one pass where computations do not depend on one another."
difficulty: medium
impact: not-assessed
package: [acorn-lib]
tags: [refactor, analyzer, validation]
status: planned
---

# Evidence

Analyzer orchestration mixes success and failure branches with repeated matching scaffolding.

# Recommendation

Use applicative-style composition only for independent checks so multiple failures can be retained without obscuring dependent control flow.

# Acceptance criteria

Independent failures are reported together and dependent checks retain their existing short-circuit semantics.
