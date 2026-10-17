---
okf_version: "0.1"
---

# Parallel execution guide

This guide overlays exact write ownership and integration locks on the [plan dependency network](plan-network.md). A plan title is not an implementation assignment: start one named phase only after confirming its exact files or external systems, public contracts, prerequisite baseline, focused validation, and landing order.

## Orthogonality test

Two slices may run concurrently only when all four statements are true:

1. Their exact write sets are disjoint.
2. Neither changes a public type, feature, serialized shape, CLI contract, or generated artifact consumed by the other.
3. They do not both require one integration lock below.
4. Both can be validated against the same baseline.

When an interface connects two slices, land the contract first and rebase the leaf work.

## Single-writer integration locks

| Lock | Files or boundary | Rule |
|---|---|---|
| Workspace graph | root/crate manifests and `Cargo.lock` | One owner for dependency, feature, package, or profile changes. |
| Library roots | crate `lib.rs`, shared `mod.rs`, `acorn-lib`/`acorn-core` roots | Leaf modules do not self-register during a concurrent batch. |
| Analyzer/check contract | analyzer result/category types, check orchestration, shared snapshots | Policy and analysis leaves use a frozen result contract; one owner integrates categories and summaries. |
| Service/RPC contract | service requests/errors, JSON-RPC registry, protobuf definitions/generated code, server lifecycle | Freeze canonical service values first; one owner integrates protocol adapters, generated artifacts, health, and shutdown. |
| API/config roots | `io/api/mod.rs`, `io/config/mod.rs`, endpoint catalog and shared snapshots | Provider leaves use a frozen contract; one owner integrates registration and snapshots. |
| Outbound MCP client | `io/mcp/client.rs`, focused client tests, MCP client documentation | Interface work uses the landed narrow feature graph; it does not edit manifests, configuration roots, server tools, or CLI syntax. |
| HTTP transport | shared request, redirect, retry, response-bound, and redaction code | Provider work consumes policy and does not fork it. |
| Schema roots | PID/research-activity/standards module indexes, validation and shared fixtures | One schema integration owner; codec leaves use dedicated fixtures. |
| CLI contract | `src/cli/`, `main.rs`, dispatch roots and generated help | Command leaves avoid syntax registration until the CLI owner integrates once. |
| Export dispatch | export command root, templates, shared snapshots and assets | PowerPoint leaves may prepare; context/renderer integration is serialized. |
| Transfer/archive | archive, OCI, GGUF, bucket source/transfer and shared fixtures | Claim exact leaf files; one owner changes shared staging or safe-path policy. |
| Ask capability routing | `io/mcp/tools/`, Needle startup/catalog, inference dispatch | SLM-00/RTR own registry/binding/context contracts first. Serialize native registration and adapter integration; do not edit these roots concurrently with Needle or CLI inference work. |
| Sidecar artifacts | sidecar roots, manifests, OCI materialization and inference CLI | Needle and SLM jobs do not edit these concurrently. |
| CI/build | `Makefile`, `.gitlab-ci.yml`, reusable job templates | One owner wires targets/jobs after tool-specific leaves are ready. |
| Documentation generation | `CommandLineHelp.md`, book indexes | Regenerate once after the implementation batch. |

## Coordination-only records

Do not assign these documents as implementation work:

- [Crate boundary architecture](refactors/crate-boundary-architecture.md)

## Work lanes

Plans within a lane normally serialize. Different lanes still check secondary locks above.

| Lane | Plans | Primary surface and rule |
|---|---|---|
| Portable boundaries | [diff](refactors/acorn-diff-resource-and-api-hardening.md), [ZON](refactors/zon-codec-hardening.md) | Portable leaves may run separately; manifests, package roots, and CI integration serialize. |
| Metadata standards | [dPID](features/dpid-research-object-support.md), [structured enrichment](refactors/core/research-activity-enrichment-integration.md) | Standard/schema work serializes shared schema roots and observes each active phase's exact write set. |
| API contract | [shared API client](refactors/api/shared-client-contract.md) | Contract slices land endpoint/request -> invocation -> resolved config -> response/error/health -> OpenAPI/RAiD. Provider files may shard after each contract freezes. |
| MCP client | [outbound MCP interface](refactors/api/outbound-mcp-client-interface.md) | Direct construction, the associated-type client trait, bounded cancellation, and focused tests are implemented. Book integration is separately owned. |
| Provider/model leaves | [model acquisition](features/model-acquisition-hardening.md), [Cosmos](features/azure-cosmos-buckets.md), [dPID](features/dpid-research-object-support.md) | Shard provider leaves only when they avoid shared registration, configuration, discovery enums, archive/GGUF, and CLI roots. |
| Schema and PID | [dPID](features/dpid-research-object-support.md), [structured enrichment](refactors/core/research-activity-enrichment-integration.md), [PID verification](testing/pid-verification.md) | Serialize PID and research-activity roots. ZON remains a disjoint codec leaf unless shared module indexes change. |
| Gather/inference | [gather extraction](features/gather-extraction.md) | Deterministic slice lands before inference. TUI, HTTP, manifests, discovery roots, and CLI syntax each require their lock owner. |
| Policy validation | [CEL validation-profile pilot](features/cel-validation-profile-pilot.md) | Runtime/corpus evaluation may proceed independently. Evaluator integration serializes analyzer results and module indexes; feature wiring, CLI syntax, generated help, and manifests use their named locks. The pilot does not own authorization, Rhai, schema validators, or MCP. |
| Export | [export/OOXML](refactors/core/export-pipeline-hardening.md), [database artifacts](features/database-backed-artifact-generation.md) | Portable interpolation may prepare first; context, renderer, database source, fuzz wiring, and export dispatch follow in order. |
| CLI composition | [CLI composition](refactors/core/cli-composition-boundary.md), [remote create](refactors/core/remote-create-container-targets.md), [CLI coverage](testing/cli-coverage.md), dPID/gather/CEL/SLM CLI leaves | Root/parser work serializes. Command-local tests and rollback logic may proceed without shared syntax/help edits. |
| Agent tools | [Needle validation](migrations/needle-3-upgrade.md), [Needle router and registry parity](features/slm-sidecars.md), including [Julia](features/slm-sidecars.md#julia-1-decision-sidecar-planned-follow-on) | SLM-00/RTR -> PAR native parity -> ADP shared adapters -> model profiles. Native command seams serialize with their domain/CLI owners. MCP/ACP adapter tests reuse existing transports. Swift external gates do not block routing; shared registry, Needle, CLI, features, artifacts, and CI changes serialize. |
| Developer loop | [coverage throughput](testing/coverage-throughput.md) | Owns `Makefile` test/coverage targets, `.config/nextest.toml`, and `[profile.coverage]`. Runs alone against the CI/build and workspace graph locks. Gating a slow test requires agreement from whichever plan owns its acceptance criteria. |
| Build and operations | [runner pools](refactors/ci-runner-pools.md), [Podman](migrations/podman-makefile.md), [Power Automate JSONL intake](features/powerapps-gitlab-logbook-reporting.md) | Operational checks can run independently. The ACRE gate and JSONL source, snapshot, durable admission, cleanup, and poller are landed; their implementation locks are released. PA-24 mixed-project validation and PA-25 local intake are ready; the PA-24 gate blocks PA-05 and reporting rollout. JSONL phases coordinate GitLab dispatch, admission/configuration, durable store, and CLI owners per their write sets; CI/Makefile edits have one owner. |

## Safe preparatory batch

[Beam inference integration](features/beam-inference-integration.md) belongs to the Provider/model leaves lane. Release research and dedicated fixtures may proceed independently; shared API/configuration, CLI, and documentation integration serialize through the named locks.

On the current tree, these leaf slices can proceed together if they honor the lock restrictions:

| Slice | Exact primary ownership | Restriction |
|---|---|---|
| Remote-create preflight/rollback tests | `crates/acorn-cli/src/commands/create/` and dedicated tests | No shared CLI arguments or generated help. |
| Export replacement-plan contract | `crates/acorn-core/src/util/` and dedicated tests | No renderer/template integration. |
| API endpoint/request provider migration | One named provider leaf and its tests | No API/config roots, manifests, or shared snapshots. |
| CEL runtime/corpus gate | Isolated runtime evaluation and dedicated policy fixtures | No manifests, module indexes, analyzer/check integration, CLI roots, or generated help. |
| CLI link coverage | Link command test module and dedicated fixtures | No parser/help or shared snapshot changes. |

## Handoff checklist

Before starting, record the plan, atomic slice, exact update/create/delete set, requested locks, public contracts, baseline, focused test, and landing order.

Before handoff, report changed files, public-contract changes, regenerated artifacts, validation results, and locks still needed. If the slice expands, stop and reschedule it rather than silently taking another lane.

After a phase completes, preserve durable evidence and remove the completed phase or task from its plan. Keep unfinished phase IDs stable, release its lock, update newly eligible phases, and remove any dangling dependency. Do not retain completed checklists as execution history.
