# Refactor Plans

## Coordination

- [Crate boundary architecture](crate-boundary-architecture.md) - Coordinate remaining package owners, including the planned `acorn-web` and `acorn-desktop` interface roots, and add atomic consumer/extraction plans after prerequisites land.

## Planned groups and independent work

- [Incremental builds and reload development](blaze-development-pilot/index.md) - Connect all-crate compiler qualification with daemon config reload, paired-binary restart and isolated function hot reload; preserve Cargo verification and browser/release ownership.
- [Shared API client contract](api/shared-client-contract.md) - Finish the reusable API boundary before provider extraction.
- [Portable diff resource and API hardening](acorn-diff-resource-and-api-hardening.md) - Bound diff work and output, complete the patch API, and add configurable unified and sequence-aware array behavior.
- [ZON codec hardening](zon-codec-hardening.md) - Review the pinned encoder and decoder, consolidate duplicated grammar logic, and strengthen conformance evidence.
- [Browser workspace runtime performance](browser-performance.md) - Operations run in a worker; shrink the ROR and WASM payloads and measure binding, build, observer and caching changes. BWP-03 and BWP-06 are ready.
- [Browser JavaScript isolation](browser-script-isolation.md) - Move every `acorn-web` `document::eval` script into one audited module with compile-time placeholder rules, typed shapes, authored files, and an optional parse gate. BSI-01 is ready.
- [Workspace dependency audit](workspace-dependency-audit.md) - Triage and remediate RustSec findings in the workspace lockfile under the Workspace graph lock.
- [CLI composition boundary](core/cli-composition-boundary.md) - Normalize parser inputs, isolate startup, share remote serve dispatch, and map errors/exits once.
- [Portability](portability/) - Future portable adapters and browser workflows on top of the implemented feature, WASM, and Darwin size gates.
- [Core refactors](core/) - Export, CLI, schema, path, interpolation, and binding boundaries.
- [API](api/) - Shared API-client contracts that prepare `acorn-api`.

## Planned with no ready phase

- [GitLab CI test runners](ci-runner-pools.md) - Runner-pool work is blocked on GitLab runner-administrator access and worker capacity before RUN-01 can produce acceptance evidence.

See the [plan dependency network](../plan-network.md) for execution edges, target crate ownership, and shared-file collisions.
