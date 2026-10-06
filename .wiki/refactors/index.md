# Refactor Plans

## Coordination

- [Crate boundary architecture](crate-boundary-architecture.md) - Split `acorn-lib` into explicitly owned portable domains, adapters, and services.

## In progress

- [GitLab CI test runners](ci-runner-pools.md) - Finish specialized routing, isolated storage, concurrency, and deployment validation for the partially configured runner pools.
- [Shared API client contract](api/shared-client-contract.md) - Finish the reusable API boundary before provider extraction.
- [Shared export and OOXML pipeline](core/export-pipeline-hardening.md) - Complete one export projection, parser-backed atomic PowerPoint mutation, and fuzz coverage.

## Planned groups and independent work

- [Portable analysis and service boundary](core/acorn-service-boundary.md) - Extract `acorn-analysis` and `acorn-service` together, then serve the shared operations through JSON-RPC and typed gRPC.
- [Portable diff resource and API hardening](acorn-diff-resource-and-api-hardening.md) - Bound diff work and output, complete the patch API, and add configurable unified and sequence-aware array behavior.
- [ZON codec hardening](zon-codec-hardening.md) - Review the pinned encoder and decoder, consolidate duplicated grammar logic, and strengthen conformance evidence.
- [CLI composition boundary](core/cli-composition-boundary.md) - Normalize parser inputs, isolate startup, share remote serve dispatch, and map errors/exits once.
- [Portability](portability/) - Future portable adapters and browser workflows on top of the implemented feature, WASM, and Darwin size gates.
- [Core refactors](core/) - Export, CLI, schema, path, interpolation, and binding boundaries.
- [API](api/) - Shared API-client contracts that prepare `acorn-api`.

See the [plan dependency network](../plan-network.md) for execution edges, target crate ownership, and shared-file collisions.
