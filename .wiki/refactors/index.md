# Refactor Plans

## Coordination

- [Crate boundary architecture](crate-boundary-architecture.md) - Split `acorn-lib` into explicitly owned portable domains, adapters, and services.
- [Portability roadmap](portability/roadmap.md) - Coordinate the critical feature, extraction, adapter, WASM, and CI path.
- [Refactor initiative](initiative.md) - Overall goals, phases, risks, scope, and completion criteria.

## Planned groups and independent work

- [Workspace duplication consolidation](workspace-duplication-consolidation.md) - Assign one explicit owner to duplicated model parsing, gather output, JSONC policy, PID helpers, synchronization, TUI controls, API status calls, and test utilities.
- [Portability](portability/) - Host traits, WASM adapters, portable CI, and size work downstream of the critical path.
- [Core refactors](core/) - Export, CLI, schema, path, interpolation, and binding boundaries.
- [Analyzer](analyzer/) - Composable analysis and validation boundaries developed in-tree; package extraction remains deferred.
- [API](api/) - Shared API-client contracts that prepare `acorn-api`.
- [acorn-cmd extraction](acorn-cmd-extraction.md) - Extract command macros and runtime helpers after portable shared values have a home.
- [GitLab CI runner pools](ci-runner-pools.md) - Isolate Rust, fuzz, and Python test workloads without blocking portability.

See the [plan dependency network](../plan-network.md) for execution edges, target crate ownership, and shared-file collisions.
