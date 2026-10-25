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
| Ask capability routing | `io/mcp/tools/`, Needle startup/catalog, inference dispatch | Caller-owned registry injection has landed; RTR owns binding/context contracts next. Serialize native registration and adapter integration; do not edit these roots concurrently with Needle or CLI inference work. |
| Sidecar artifacts | sidecar roots, manifests, OCI materialization and inference CLI | Needle and SLM jobs do not edit these concurrently. |
| CI/build | `Makefile`, `.gitlab-ci.yml`, reusable job templates | One owner wires targets/jobs after tool-specific leaves are ready. |
| Documentation generation | `CommandLineHelp.md`, book indexes | Regenerate once after the implementation batch. |

## Coordination-only records

Do not assign these documents as implementation work:

- [Crate boundary architecture](refactors/crate-boundary-architecture.md)
- [Ask routing and sidecar coordination](features/slm-sidecars.md)

## Work lanes

Plans within a lane normally serialize. Different lanes still check secondary locks above.

| Lane | Plans | Primary surface and rule |
|---|---|---|
| Portable boundaries | [diff](refactors/acorn-diff-resource-and-api-hardening.md), [ZON](refactors/zon-codec-hardening.md) | Portable leaves may run separately; manifests, package roots, and CI integration serialize. |
| Metadata standards | [dPID](features/dpid-research-object-support.md), [structured enrichment](refactors/core/research-activity-enrichment-integration.md) | Standard/schema work serializes shared schema roots and observes each active phase's exact write set. |
| API contract | [shared API client](refactors/api/shared-client-contract.md) | Contract slices land endpoint/request -> invocation -> resolved config -> response/error/health -> OpenAPI/RAiD. Provider files may shard after each contract freezes. |
| MCP client | [outbound MCP interface](refactors/api/outbound-mcp-client-interface.md) | Complete embedder documentation and portable/WASM evidence before retiring the plan; implementation contracts are landed. |
| Provider/model leaves | [model acquisition](features/model-acquisition-hardening.md), [Cosmos](features/azure-cosmos-buckets.md), [dPID](features/dpid-research-object-support.md) | Shard provider leaves only when they avoid shared registration, configuration, discovery enums, archive/GGUF, and CLI roots. |
| Schema and PID | [dPID](features/dpid-research-object-support.md), [structured enrichment](refactors/core/research-activity-enrichment-integration.md), [PID verification](testing/pid-verification.md), [sponsor highlights](features/sponsor-highlights-clarity.md) | Serialize PID and research-activity roots. Sponsor editorial input and ASPECT graduation phases HIL-09 through HIL-14 claim this lock. ZON remains a disjoint codec leaf unless shared module indexes change. |
| Gather/inference | [gather extraction](features/gather-extraction.md), [interactive metadata chat](features/interactive-metadata-chat.md) | Gather's deterministic slice lands before its inference adapter. Chat history, inline controls including Confirm, the shared slash catalog, confirmed local metadata analysis, and the grounded chat driver have landed. IMC-06 exposes the driver using the landed route and local check contracts. Shared inference, TUI, HTTP, discovery, and CLI syntax require their lock owner. |
| Policy validation | [CEL validation-profile pilot](features/cel-validation-profile-pilot.md) | Runtime/corpus evaluation may proceed independently. Evaluator integration serializes analyzer results and module indexes; feature wiring, CLI syntax, generated help, and manifests use their named locks. The pilot does not own authorization, Rhai, schema validators, or MCP. |
| Export | [export/OOXML](refactors/core/export-pipeline-hardening.md), [database artifacts](features/database-backed-artifact-generation.md), [grouped report separators](features/grouped-report-separators.md), [sponsor highlights](features/sponsor-highlights-clarity.md) | Portable interpolation may prepare first; context, renderer, database source, fuzz wiring, and export dispatch follow in order. Grouped separators extend the existing --combine path; sponsor synopsis work shares `export/html.rs`, report wiring and styles, so renderer integration is serialized. |
| CLI composition | [CLI composition](refactors/core/cli-composition-boundary.md), [remote create](refactors/core/remote-create-container-targets.md), [CLI coverage](testing/cli-coverage.md), [sponsor highlights](features/sponsor-highlights-clarity.md), [interactive metadata chat](features/interactive-metadata-chat.md), dPID/gather/CEL/SLM CLI leaves | Root/parser work serializes. IMC-06/10 own their syntax and help windows; other chat leaves avoid those files. CLI-11 retires the composition plan after the command and startup contracts pass. |
| Agent tools | [Needle validation](migrations/needle-3-upgrade.md), [native routing](features/needle-router-native-parity.md), [generic adapters](features/ask-executor-adapters.md), [Swift profile](features/swift-slm-profile.md), [Julia profile](features/julia-decision-sidecar.md) | The 72-row CLI parity inventory, caller-owned Needle registry, frozen 200-case routing corpus, versioned binding contract, terminal execution, opt-in CLI route syntax, bounded native inputs, and local check binding are in place. PAR-02 completes nonlocal and fix-preview check parity next. ADP configuration, feature, adapter, and CLI leaves may consume the landed contracts. PAR native parity must pass before ADP-06/07 release. Swift/Julia feasibility may proceed earlier; their releases wait for ADP-07. SLM-17/JUL-11 own registry integration; SLM-23/JUL-16 own CLI syntax. Linux x86_64 Needle validation passed on Oklahoma; Linux ARM64 may validate directly on Dante. The Windows x86_64 job awaits a native runner. Shared documentation, registry, Needle, CLI, features, artifacts, and CI changes serialize. Native command seams serialize with their domain/CLI owners. MCP/ACP adapter tests reuse existing transports. Swift external gates do not block routing. |
| Metadata feedback | [interactive metadata chat](features/interactive-metadata-chat.md), [structured enrichment](refactors/core/research-activity-enrichment-integration.md) | Selectable provider and vocabulary candidates have landed; IMC-10/11 own format feedback. Serialize their exact workflow files with structured enrichment implementation; chat fix and terminal leaves may proceed while that lock is held. |
| Developer loop | [coverage throughput](testing/coverage-throughput.md) | Owns `Makefile` test/coverage targets, `.config/nextest.toml`, and `[profile.coverage]`. Runs alone against the CI/build and workspace graph locks. Gating a slow test requires agreement from whichever plan owns its acceptance criteria. |
| Build and operations | [runner pools](refactors/ci-runner-pools.md), [GitLab shell runner readiness](testing/gitlab-shell-runner-readiness.md), [Podman](migrations/podman-makefile.md) | GSR-01 claims the CLI contract; GSR-02 owns native Windows runner readiness. Repository CI/Makefile integration has one owner. The external pilot reporting include and schedule belong to the Vergil automation lane. |
| Power Platform intake | [PowerApp/Power Automate JSONL intake](features/powerapps-jsonl-intake/index.md) | PA-14/15/16 serialize on the intake route; PA-02 is access/licensing evidence; PA-17 is solution packaging; PA-12 is later shared deployment. Do not write the live inbox while another lane owns its cleanup or handback. No Vergil prerequisite. |
| Manual JSONL handback | [Manual ingestion and graduation](features/manual-jsonl-ingestion-graduation/index.md) | URI staging, consolidated local run modes and isolated CLI/PDF rehearsals are complete. MAN-06 owns the isolated handback fixture and, only when authorized, the pilot reconciliation branch. It must serialize with bot activation and remote inbox cleanup. |
| Vergil JSONL automation | [Vergil GitLab bot](features/vergil-gitlab-jsonl-automation/index.md) | BOT-01 owns approvals/runtime references; PA-03 owns service boot; PA-04 owns external callback integration; PA-05/BOT-02 serialize with form mutations on the intake route; PA-05 can create a draft MR, and PA-06/07 accept its review/delivery and human graduation/merge; PA-08/BOT-03 own external reporting publication; PA-09 records final operations. Never run bot ingestion and manual handback against the same inbox concurrently. |
| Logbook graduation adapters | [sponsor highlights](features/sponsor-highlights-clarity.md) | HIL-15 owns database/MCP graduation tests, then HIL-16 owns GitLab automation review tests using the implemented admission contract. The Vergil plan's PA-07 is an external pilot, not a prerequisite for local schema or adapter work. Local `intake run --index` already consumes the current schema contract. HIL-19 owns its future target-specific adapter and CLI/help integration; MAN-06 owns only handback, not schema or CLI changes. |

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
