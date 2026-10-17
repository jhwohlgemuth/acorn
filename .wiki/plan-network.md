---
okf_version: "0.1"
---

# Plan dependency network

This map records implementation dependencies among the active consolidated plans. Portfolio rank is maintained in the [planning index](index.md); dependency order can override rank when a prerequisite contract is not ready.

Executable detail lives in each plan's `# Execution phases` section. Cross-plan edges name stable phase IDs; completed phases are pruned without renumbering unfinished phases. A plan-level edge is not permission to start every phase in that plan.

## Boundary critical path

```text
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

- [Outbound MCP client embedder interface](refactors/api/outbound-mcp-client-interface.md) builds on the landed narrow `mcp-client` graph. It owns direct server-map construction and bounded caller-driven cancellation while preserving the landed MCP configuration, policy, transport, and result contracts.

- [Shared API client contract](refactors/api/shared-client-contract.md) prepares `acorn-api`; endpoint, request, resolved configuration, invocation, response, error, and provider policy move together only after its acceptance gates pass.
- [Shared export and OOXML pipeline](refactors/core/export-pipeline-hardening.md) prepares `acorn-export` by producing one pure projection and fallible in-memory transformations.
- [CLI composition boundary](refactors/core/cli-composition-boundary.md) follows the stable service/server request and error contracts. Command leaves may normalize earlier only when they avoid shared CLI roots.
- [Portable diff hardening](refactors/acorn-diff-resource-and-api-hardening.md) and [ZON codec hardening](refactors/zon-codec-hardening.md) are independent portable leaves.

### Provider and acquisition work

- [Beam inference integration](features/beam-inference-integration.md) requires published release/deployment evidence and a verified serving contract before promotion. It consumes landed shared API mechanics; gather adoption follows the gather contract. It is independent of local SLM sidecars.

- [Model acquisition and metadata hardening](features/model-acquisition-hardening.md) consumes shared endpoint/authentication mechanics but owns Hugging Face identity, GGUF metadata, OCI package discrimination, and safe materialization.
- [Azure Cosmos bucket downloads](features/azure-cosmos-buckets.md) consumes the shared API policy and landed bucket-source/transfer contracts. It does not extend the repository enum or local database backend.
- [dPID research-object support](features/dpid-research-object-support.md) lands portable parser and RO-Crate slices before resolver, gather, persistence, and CLI integration.
- [Structured research-activity enrichment](refactors/core/research-activity-enrichment-integration.md) and dPID serialize on research-activity schema and standards roots.
- [Database-backed artifact generation](features/database-backed-artifact-generation.md) follows the export-context portion of the export plan and uses the existing canonical database/revision contract.

### Gather, inference, and agent tools

- [Deterministic and assisted gather extraction](features/gather-extraction.md) lands deterministic selector and transport policy before its optional inference adapter. Its TUI cutover follows the shared source contract.
- [Needle router and capability registry parity](features/slm-sidecars.md) starts with SLM-00 caller-owned registry injection, then PAR-00 and RTR-01/02/03 contracts. PAR-01 through PAR-16 deliver measured native parity before ADP-01 through ADP-07 add generic executor configuration, MCP, ACP, endpoints, and sidecars. SLM-17 and JUL-11 consume ADP-05. Native parity and the generic release do not depend on model licensing or macOS hosts.
- Router native adapters consume existing operation services and coordinate command seams with CLI composition; export/gather/model-acquisition plans retain their domain behavior ownership. MCP and ACP adapters reuse landed client/inference contracts; outbound MCP documentation closeout is not a code prerequisite. No automatic inbound MCP re-export is implied.
- The [Julia-1 decision slice](features/slm-sidecars.md#julia-1-decision-sidecar-planned-follow-on) follows PAR-16 and consumes ADP-05 for registration, plus landed acquisition contracts. JUL-15 depends on Julia quality/resource gates, not SLM-22; Swift and Julia releases are independent.
- [Needle 3 host validation](migrations/needle-3-upgrade.md) and the router plan serialize Needle roots, tool-index/artifact, and CI/build changes. Router implementation uses existing contracts; PAR-15 still needs an installed pinned runner on a supported host for real routing evaluation.

### Optional policy validation

- [CEL validation-profile pilot](features/cel-validation-profile-pilot.md) builds on the implemented analyzer and validation contracts. Runtime conformance and resource-bound evidence precede evaluator integration, followed by CLI wiring and team acceptance.
- CEL feature wiring must preserve the landed dependency-capability boundaries. Its runtime/corpus gate can proceed without shared locks, but analyzer results, manifests, and CLI integration serialize with their current owners.
- OPA integration remains conditional future work described in the [assessment](assets/opa-cel-review.md), not a scheduled implementation.

### Reliability and operations

- [Power Automate GitLab JSONL intake](features/powerapps-gitlab-logbook-reporting.md) requires a conflict-safe flow appending Logbook envelopes and a Vergil poller that durably ingests/removes items, then multiplexes records by project ID or configured folder alias to augment the associated `index.json`. The ACRE 2.0.0 contract, source snapshots, durable admission, conflict-safe cleanup, independent poller, mixed-project review publication, one-shot manual intake, diff export, native executable, and explicit callback command are landed and locally validated. Standalone local intake has a separate proposal/apply command. PA-01 tester notifications passed; PA-02 ownership and access decisions gate PA-03 and PA-04, which gate PA-05. Final PA-09 acceptance requires PA-08. No gateway is required for the selected GitLab-mediated path.
- [Runner-pool isolation](refactors/ci-runner-pools.md) precedes claims that Rust/fuzz/reporting jobs are concurrently isolated. Its reporting pool supports the scheduled reporting workflow.
- [Remote container creation](refactors/core/remote-create-container-targets.md) consumes normalized CLI requests when that boundary lands; leaf rollback/preflight work can proceed first.
- [Podman Makefile migration](migrations/podman-makefile.md) changes development/release tooling only and must preserve product-level Docker and Podman support.
- [CLI coverage](testing/cli-coverage.md) may add command-leaf tests at any time; startup/exit tests follow the CLI composition boundary.
- [PID verification](testing/pid-verification.md) serializes PID production helpers with dPID, then adds Miri and Creusot without blocking ordinary builds.
- [Coverage throughput](testing/coverage-throughput.md) is developer-workflow only and owns no product contract. It serializes on the `Makefile` and root-manifest locks, so it must not overlap Podman migration, runner-pool, PID verification, or export fuzzing phases. Its bounded-test-time phase must observe the exact write sets of concurrent CLI and MCP test phases.

## Shared integration surfaces

| Surface | Plans that serialize |
|---|---|
| Workspace manifests and lockfile | analysis/service extraction, model acquisition, Cosmos, gather extraction, CEL, SLM, Miri/Creusot wiring |
| Analyzer results, check orchestration, and module indexes | analysis/service extraction, CEL, gather inference, structured enrichment, diff, CLI coverage |
| JSON-RPC registry, protobuf, and server lifecycle | analysis/service extraction; CLI composition consumes the finished server contract |
| `io/api/mod.rs`, configuration roots, endpoint snapshots | shared API contract, dPID, Cosmos, model acquisition |
| `io/mcp/client.rs`, outbound client tests and documentation | outbound MCP client interface; the landed feature graph and manifest boundary remain fixed |
| Research-activity/PID schema roots | Dublin Core, dPID, structured enrichment, PID verification |
| `io/{archive,oci}.rs`, `io/model/gguf/`, and model planner | model acquisition, SLM/Needle artifact work |
| Tool registry, Needle startup/catalog, inference dispatch | router/parity, Needle validation, CLI composition; adapter leaves consume the frozen binding contract |
| CLI parser, `main.rs`, generated help | analysis/service serve cutover, CLI composition, dPID, gather, CEL, remote create, SLM |
| Export dispatch/templates/PowerPoint | export pipeline, database-backed artifacts |
| `Makefile` and `.gitlab-ci.yml` | runner pools, Podman, SLM, Needle, PID verification, export fuzzing, coverage throughput |
| Developer test and coverage workflow (`Makefile` test targets, `.config/nextest.toml`, `[profile.coverage]`) | coverage throughput, CLI coverage | Slow-test gating and coverage scoping must not silently drop tests that another plan's acceptance criteria depend on. |

Exact ownership and safe concurrent slices are maintained in the [parallel execution guide](parallel-execution.md).

## Completion discipline

- A plan closes only after its acceptance criteria and durable documentation are complete.
- Remove each completed section, phase, and task after reconciling durable evidence. Do not retain checked lists or completed phase records.
- Preserve the IDs of unfinished phases when earlier phases are pruned, and remove dangling phase dependencies in the same change.
- Delete completed plan files and remove their index, network, and lane entries immediately.
- If a plan discovers a materially different outcome, create a new atomic concept instead of widening the active write set silently.
