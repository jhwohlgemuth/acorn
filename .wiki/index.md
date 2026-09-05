---
okf_version: "0.1"
---

# ACORN Planning Wiki

This directory is the authoritative home for ACORN implementation plans. It is an [Open Knowledge Format](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md) bundle.

## Contents

- [Plan dependency network](plan-network.md) - Current execution order, crate ownership, and cross-plan blockers.
- [Parallel execution guide](parallel-execution.md) - Orthogonal work lanes, single-writer integration locks, and safe concurrent batches.
- [Feature plans](features/) - New user-facing capabilities and integrations.
- [Refactor plans](refactors/) - Reliability, maintainability, extensibility, and portability work.
- [Migration plans](migrations/) - Changes that replace an existing dependency, architecture, or workflow.
- [Testing plans](testing/) - CLI test coverage, integration testing, and quality assurance work.

## Next execution phase

1. Complete the active [portable `CheckBatch` contract](refactors/analyzer/check-batch-monoid.md) without extracting a crate or adding host traits.
2. Continue with [standards traversal](refactors/analyzer/standards-traversal.md) once check batching is stable.
3. Continue narrowing broad dependency feature defaults while keeping minimal, alloc, analyzer, native, and WASM checks green.
4. Revisit consumer-owned ports only after concrete native and alternate consumers prove a narrow seam.

The [crate boundary architecture](refactors/crate-boundary-architecture.md), [portability roadmap](refactors/portability/roadmap.md), and [refactor initiative](refactors/initiative.md) are coordination records, not schedulable implementation jobs.

## Adding a plan

1. Create one semantically named Markdown file for one atomic topic.
2. Add `type: Plan`, `title`, `description`, `difficulty`, `impact`, a non-empty `package` list, `tags`, and `status` to its YAML frontmatter.
3. Use `low`, `medium`, `high`, `mixed`, or `not-assessed` for difficulty; use `low`, `medium`, `high`, `critical`, `mixed`, or `not-assessed` for impact.
4. Use `planned` or `in-progress` for status. Add `priority: next` when the plan is an immediate candidate.
5. Document the goal, current evidence, project code boundaries, dependencies, recommendation, validation, and acceptance criteria. Add citations for externally sourced claims.
6. In project code boundaries, identify existing files or directories to update, planned files or directories to create, and files to delete. Write `None` when a category is intentionally empty.
7. Link related concepts with relative Markdown links, distinguish prerequisites from follow-on work, and add the new concept to the nearest directory index and the [plan dependency network](plan-network.md).
8. Assign the plan to a lane in the [parallel execution guide](parallel-execution.md). Before implementation, narrow broad directory boundaries to an atomic write set and name every shared integration file.

## Maintaining plans

1. Verify evidence and status against the current code before editing a plan.
2. Update frontmatter and body together when scope, package, priority, or status changes.
3. Delete finished plans and remove their index entries.
4. Consolidate overlapping proposals into one canonical concept instead of duplicating them.
5. Keep external claims cited and verify that internal links resolve.
6. Preserve OKF frontmatter and the reserved `index.md` and `log.md` filename conventions.
7. After changes, check frontmatter, required fields, links, legacy references, and whitespace.
8. Reconcile every plan's code boundaries, dependency edges, lane, and integration locks whenever modules move or a plan completes.
9. Treat umbrella roadmaps as coordination records, not independently schedulable implementation tasks.
