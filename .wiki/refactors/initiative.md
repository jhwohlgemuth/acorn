---
type: Plan
title: "ACORN refactor initiative"
description: "Coordinate cross-crate reliability, maintainability, extensibility, and portability improvements."
difficulty: mixed
impact: critical
package: [workspace]
tags: [refactor, roadmap, architecture]
status: in-progress
---

# Goal

Organize refactoring around reliability, maintainability, extensibility, and compatibility while keeping the execution board grounded in current code.

# Scope

Included work covers `acorn-cli` command flow and export behavior, `acorn-lib` I/O, schemas, interpolation, analyzers and APIs, and cross-crate boundaries. Generated documentation and style-only cleanup are excluded.

# Execution phases

| Phase | Status | Focus | Outcome |
|---|---|---|---|
| Portable boundary hardening | In progress | Remaining schema/analyzer host couplings, dependency features, std isolation | `acorn-schema` is extracted and the `acorn-analysis` boundary becomes mechanical |
| Export boundary hardening | In progress | OOXML, paths, export assets | Export failures become typed errors instead of silent corruption |
| Shared abstractions | Planned | Export context, safe paths, interpolation | Duplicated mappings and path logic are replaced |
| Structural modularization | Planned | CLI startup and API clients | Boundaries become explicit and testable |
| Refinement | Planned | Bindings and portability | Repetition decreases and build targets broaden |

# Project code boundaries

- **Update:** all files named by child plans; the initiative itself does not authorize code outside those explicit boundaries.
- **Create:** only files and crates named by accepted child plans, led by the portability critical path in the [plan dependency network](../plan-network.md).
- **Delete:** completed plan files and implementation explicitly superseded by a child plan; no unscheduled cleanup.

# Dependencies

The [crate boundary architecture](crate-boundary-architecture.md) is the coordinating plan. Its immediate path is dependency classification and std isolation -> [`acorn-schema` extraction](portability/acorn-schema-extraction.md) -> analyzer isolation and `acorn-analysis`. Reliability work may proceed in parallel only when it does not rename the same modules.

# Parallel execution

This initiative is coordination-only and must never be assigned as one implementation task. Schedule an atomic child plan with an exact write set. The coordinator owns shared manifests, module indexes, CLI roots, generated artifacts, and CI/build integration as defined by the [parallel execution guide](../parallel-execution.md).

# Prioritization

Prioritize the portability critical path first, then reliability, security, and correctness impact, then difficulty within each impact tier.

# Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Error changes alter existing behavior | Medium | High | Use focused tests and gradual changes |
| Export changes affect user artifacts | Medium | High | Preserve compatibility and inspect real outputs |
| no-std/WASM architecture expands scope | High | High | Stage feature work and enforce it in CI |

# Acceptance criteria

1. Export paths fail with typed errors rather than silent corruption.
2. Path normalization and containment are centralized.
3. Export field mapping has one shared representation.
4. Cross-crate interpolation is unified and tested.
5. The portable core does not depend on host-only behavior.

# Related plans

- [Crate boundary architecture](crate-boundary-architecture.md)
- [Core refactors](core/)
- [Portability](portability/)
- [Analyzer refactors](analyzer/)
- [API refactors](api/)
- [Plan dependency network](../plan-network.md)
