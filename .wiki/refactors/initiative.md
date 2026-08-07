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
| Boundary hardening | In progress | OOXML, paths, export assets | Export failures become typed errors instead of silent corruption |
| Shared abstractions | Planned | Export context, safe paths, interpolation | Duplicated mappings and path logic are replaced |
| Structural modularization | Planned | CLI startup, API clients, schema I/O | Boundaries become explicit and testable |
| Refinement | Planned | Bindings and portability | Repetition decreases and build targets broaden |

# Prioritization

Prioritize reliability, security, and correctness impact first, then difficulty within each impact tier.

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
