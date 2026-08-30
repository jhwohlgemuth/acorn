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

# Project code boundaries

- **Update:** independent validation/check composition in `crates/acorn-lib/src/analyzer/{mod,check,error}.rs` and adjacent tests; later `acorn-analysis`.
- **Create:** a small applicative accumulation helper only if existing validation/report combinators cannot express the contract.
- **Delete:** fail-fast or repeated manual accumulation at migrated call sites; no public check type.

# Dependencies

Build directly on the existing flat, ordered portable validation report.

# Evidence

Analyzer orchestration mixes success and failure branches with repeated matching scaffolding.

# Recommendation

Use applicative-style composition only for independent checks so multiple failures can be retained without obscuring dependent control flow.

# Acceptance criteria

Independent failures are reported together and dependent checks retain their existing short-circuit semantics.
