---
okf_version: "0.1"
---

# Plan dependency network

This map records implementation dependencies among the active consolidated plans. Portfolio rank is maintained in the [planning index](index.md); dependency order can override rank when a prerequisite contract is not ready.

## Boundary critical path

```text
dependency capabilities and macOS size
       |                 |                    |                    |
       v                 v                    v                    v
 analysis + service   shared API client   outbound MCP client   export and OOXML
       |                 |                    |                    |
       v                 v                    v                    v
 dual-protocol server  acorn-api boundary  Xylem integration   acorn-export boundary
       |
       v
 CLI composition boundary
```

The [crate boundary architecture](refactors/crate-boundary-architecture.md) coordinates this path. It is not a schedulable implementation job.

## Direct edges

### Portable and extraction boundaries

- [Dependency capabilities and macOS unwind size](refactors/portability/dependency-features-and-macos-size.md) precedes new crate extraction and any feature-adding plan that changes workspace manifests. It owns narrowing the landed `mcp-client` graph without reopening MCP protocol behavior.
- [Outbound MCP client embedder interface](refactors/api/outbound-mcp-client-interface.md) follows the narrow `mcp-client` graph. It owns direct server-map construction and bounded caller-driven cancellation while preserving the landed MCP configuration, policy, transport, and result contracts.
- [Portable analysis and service boundary](refactors/core/acorn-service-boundary.md) follows dependency capability cleanup. Its first phase extracts `acorn-analysis` and the supplied-content `acorn-service` catalog together; its second phase hosts the catalog through JSON-RPC and typed gRPC in `acorn-server`.
- [Shared API client contract](refactors/api/shared-client-contract.md) prepares `acorn-api`; endpoint, request, resolved configuration, invocation, response, error, and provider policy move together only after its acceptance gates pass.
- [Shared export and OOXML pipeline](refactors/core/export-pipeline-hardening.md) prepares `acorn-export` by producing one pure projection and fallible in-memory transformations.
- [CLI composition boundary](refactors/core/cli-composition-boundary.md) follows the stable service/server request and error contracts. Command leaves may normalize earlier only when they avoid shared CLI roots.
- [Portable diff hardening](refactors/acorn-diff-resource-and-api-hardening.md) and [ZON codec hardening](refactors/zon-codec-hardening.md) are independent portable leaves.

### Provider and acquisition work

- [Model acquisition and metadata hardening](features/model-acquisition-hardening.md) consumes shared endpoint/authentication mechanics but owns Hugging Face identity, GGUF metadata, OCI package discrimination, and safe materialization.
- [Azure Cosmos bucket downloads](features/azure-cosmos-buckets.md) consumes the shared API policy and landed bucket-source/transfer contracts. It does not extend the repository enum or local database backend.
- [dPID research-object support](features/dpid-research-object-support.md) lands portable parser and RO-Crate slices before resolver, gather, persistence, and CLI integration.
- [Structured research-activity enrichment](refactors/core/research-activity-enrichment-integration.md) and dPID serialize on research-activity schema and standards roots.
- [Database-backed artifact generation](features/database-backed-artifact-generation.md) follows the export-context portion of the export plan and uses the existing canonical database/revision contract.

### Gather, inference, and agent tools

- [Deterministic and assisted gather extraction](features/gather-extraction.md) lands deterministic selector and transport policy before its optional inference adapter. Its TUI cutover follows the shared source contract.
- [SLM sidecars](features/slm-sidecars.md) may build explicit Tongue, Redact, and Title runners independently. Schemer integration, if it becomes available, consumes the gather inference contract.
- [Needle 3 host validation](migrations/needle-3-upgrade.md) and SLM sidecars share sidecar artifact and CI/build locks but are functionally independent.

### Optional policy validation

- [CEL validation-profile pilot](features/cel-validation-profile-pilot.md) builds on the implemented analyzer and validation contracts. Runtime conformance and resource-bound evidence precede evaluator integration, followed by CLI wiring and team acceptance.
- CEL feature wiring coordinates with [dependency capabilities](refactors/portability/dependency-features-and-macos-size.md). Its runtime/corpus gate can proceed without shared locks, but analyzer results, manifests, and CLI integration serialize with their current owners.
- OPA integration remains conditional future work described in the [assessment](assets/opa-cel-review.md), not a scheduled implementation.

### Reliability and operations

- [Power Apps deployment validation](features/powerapps-gitlab-logbook-reporting.md) is an operational closeout over implemented code and can finish independently when deployment credentials and endpoints are available.
- [Runner-pool isolation](refactors/ci-runner-pools.md) precedes claims that Rust/fuzz/reporting jobs are concurrently isolated. Its reporting pool supports the scheduled reporting workflow.
- [Remote container creation](refactors/core/remote-create-container-targets.md) consumes normalized CLI requests when that boundary lands; leaf rollback/preflight work can proceed first.
- [Podman Makefile migration](migrations/podman-makefile.md) changes development/release tooling only and must preserve product-level Docker and Podman support.
- [CLI coverage](testing/cli-coverage.md) may add command-leaf tests at any time; startup/exit tests follow the CLI composition boundary.
- [PID verification](testing/pid-verification.md) serializes PID production helpers with dPID, then adds Miri and Creusot without blocking ordinary builds.

## Shared integration surfaces

| Surface | Plans that serialize |
|---|---|
| Workspace manifests and lockfile | dependency capabilities, analysis/service extraction, model acquisition, Cosmos, gather extraction, CEL, SLM, Miri/Creusot wiring |
| Analyzer results, check orchestration, and module indexes | analysis/service extraction, CEL, gather inference, structured enrichment, diff, CLI coverage |
| JSON-RPC registry, protobuf, and server lifecycle | analysis/service extraction; CLI composition consumes the finished server contract |
| `io/api/mod.rs`, configuration roots, endpoint snapshots | shared API contract, dPID, Cosmos, model acquisition |
| `io/mcp/client.rs`, outbound client tests and documentation | outbound MCP client interface; dependency capabilities owns prerequisite feature and manifest work |
| Research-activity/PID schema roots | dPID, structured enrichment, PID verification |
| `io/{archive,oci}.rs`, `io/model/gguf/`, and model planner | model acquisition, SLM/Needle artifact work |
| CLI parser, `main.rs`, generated help | analysis/service serve cutover, CLI composition, dPID, gather, CEL, remote create, SLM |
| Export dispatch/templates/PowerPoint | export pipeline, database-backed artifacts |
| `Makefile` and `.gitlab-ci.yml` | runner pools, Podman, dependency/size, SLM, Needle, PID verification, export fuzzing |

Exact ownership and safe concurrent slices are maintained in the [parallel execution guide](parallel-execution.md).

## Completion discipline

- A plan closes only after its acceptance criteria and durable documentation are complete.
- Delete completed plan files and remove their index, network, and lane entries immediately.
- If a plan discovers a materially different outcome, create a new atomic concept instead of widening the active write set silently.
