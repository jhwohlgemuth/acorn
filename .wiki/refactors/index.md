# Refactor Plans

## Next

- [Isolate std-only modules](portability/isolate-std-only-modules.md) - Remove the remaining schema host state and prepare analyzer model isolation.
- [Extract acorn-schema](portability/acorn-schema-extraction.md) - Create the next portable package after its std-isolation gate passes.

## Coordination

- [Crate boundary architecture](crate-boundary-architecture.md) - Split `acorn-lib` into portable domains, adapters, services, and a compatibility facade.
- [Portability roadmap](portability/roadmap.md) - Coordinate the critical feature, extraction, adapter, WASM, and CI path.
- [Refactor initiative](initiative.md) - Overall goals, phases, risks, scope, and completion criteria.

## Planned groups and independent work

- [Portability](portability/) - Host traits, WASM adapters, portable CI, and size work downstream of the critical path.
- [Core refactors](core/) - Export, CLI, schema, path, interpolation, and binding boundaries.
- [Analyzer](analyzer/) - Composable analysis and validation abstractions that prepare `acorn-analysis`.
- [API](api/) - Shared API-client contracts that prepare `acorn-api`.
- [acorn-cmd extraction](acorn-cmd-extraction.md) - Extract command macros and runtime helpers after portable shared values have a home.
- [GitLab CI runner pools](ci-runner-pools.md) - Isolate Rust, fuzz, and Python test workloads without blocking portability.

See the [plan dependency network](../plan-network.md) for execution edges, target crate ownership, and shared-file collisions.
