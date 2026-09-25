# Refactor Plans

## Coordination

- [Crate boundary architecture](crate-boundary-architecture.md) - Split `acorn-lib` into explicitly owned portable domains, adapters, and services.
- [Portability roadmap](portability/roadmap.md) - Coordinate the critical feature, extraction, adapter, WASM, and CI path.
- [Refactor initiative](initiative.md) - Overall goals, phases, risks, scope, and completion criteria.

## In progress

- [GitLab CI test runners](ci-runner-pools.md) - Finish specialized routing, isolated storage, concurrency, and deployment validation for the partially configured runner pools.

## Planned groups and independent work

- [ZON codec hardening](zon-codec-hardening.md) - Review the pinned encoder and decoder, consolidate duplicated grammar logic, and strengthen conformance evidence.
- [Workspace duplication consolidation](workspace-duplication-consolidation.md) - Assign one explicit owner to JSONC policy, PID helpers, model synchronization, TUI gather and controls, remote serve dispatch, Rhai check projection, and CLI test utilities.
- [Portability](portability/) - Dependency-feature narrowing and measured macOS size work on top of the implemented portability matrix.
- [Core refactors](core/) - Export, CLI, schema, path, interpolation, and binding boundaries.
- [API](api/) - Shared API-client contracts that prepare `acorn-api`.

See the [plan dependency network](../plan-network.md) for execution edges, target crate ownership, and shared-file collisions.
