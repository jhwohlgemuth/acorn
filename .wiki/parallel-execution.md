---
okf_version: "0.1"
---

> **Sanitized copy:** Account labels and related assets have been redacted. Historical checksums and import results describe the originals; see [privacy and reuse limits](privacy.md).

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
| Daemon runtime | `acorn-daemon` runner, scheduler, control, and process lifecycle | Land job/config values and store contract before composing worker and listeners; serialize shared daemon roots. |
| Host service registration | `acorn-daemon/src/install/` and installed service definitions | Platform leaves may prepare separately, but shared registration, staging, and rollback have one integration owner. |
| API/config roots | `io/api/mod.rs`, `io/config/mod.rs`, endpoint catalog and shared snapshots | Provider leaves use a frozen contract; one owner integrates registration and snapshots. |
| Outbound MCP client | `io/mcp/client.rs`, focused client tests, MCP client documentation | Interface work uses the landed narrow feature graph; it does not edit manifests, configuration roots, server tools, or CLI syntax. |
| Host authentication capability | `acorn-host/src/auth/`, ACRE auth profile translation | SPD establishes the reusable interface first; headless HCI-02/03/04 extend it after accepted SPD-26. Shared files serialize with API/configuration and intake owners. |
| HTTP transport | shared request, redirect, retry, response-bound, and redaction code | Provider work consumes policy and does not fork it. |
| Schema roots | PID/research-activity/standards module indexes, validation and shared fixtures | One schema integration owner; codec leaves use dedicated fixtures. |
| CLI interactive controls | `acorn-cli/src/io/interactive/`, OAuth presentation and shared renderer tests | SharePoint SPD-13 and interactive metadata chat serialize renderer changes; preserve current callers and machine-readable stdout. |
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
| ACRE formats | [RON configuration](features/acre-ron-configuration.md) | RON-01 owns dependency/format registration; RON-02/03 serialize configuration roots; RON-04/05 serialize CLI discovery, help, and formatting; RON-06 owns final documentation and gates. Coordinate exact files with API configuration, export core utilities, and CLI composition. No change to daemon configuration or generic RAD/export formats. |
| MCP client | [outbound MCP interface](refactors/api/outbound-mcp-client-interface.md) | Complete embedder documentation and portable/WASM evidence before retiring the plan; implementation contracts are landed. |
| Provider/model leaves | [model acquisition](features/model-acquisition-hardening.md), [Cosmos](features/azure-cosmos-buckets.md), [dPID](features/dpid-research-object-support.md) | Shard provider leaves only when they avoid shared registration, configuration, discovery enums, archive/GGUF, and CLI roots. |
| Schema and PID | [dPID](features/dpid-research-object-support.md), [structured enrichment](refactors/core/research-activity-enrichment-integration.md), [PID verification](testing/pid-verification.md), [sponsor highlights](features/sponsor-highlights-clarity.md) | Serialize PID and research-activity roots. Sponsor editorial input and ASPECT graduation phases HIL-09 through HIL-14 claim this lock. ZON remains a disjoint codec leaf unless shared module indexes change. |
| Gather/inference | [gather extraction](features/gather-extraction.md), [interactive metadata chat](features/interactive-metadata-chat.md) | Gather's deterministic slice lands before its inference adapter. Chat history, inline controls, shared slash commands, local analysis, `ask --interactive`, and reviewed document fixes have landed. Shared inference, TUI, HTTP, discovery, and CLI syntax require their lock owner. |
| Policy validation | [CEL validation-profile pilot](features/cel-validation-profile-pilot.md) | Runtime/corpus evaluation may proceed independently. Evaluator integration serializes analyzer results and module indexes; feature wiring, CLI syntax, generated help, and manifests use their named locks. The pilot does not own authorization, Rhai, schema validators, or MCP. |
| Export | [database artifacts](features/database-backed-artifact-generation.md), [grouped report separators](features/grouped-report-separators.md), [sponsor highlights](features/sponsor-highlights-clarity.md) | Serialize shared report files and validation with grouped separators and sponsor highlights where surfaces overlap. |
| CLI composition | [CLI composition](refactors/core/cli-composition-boundary.md), [remote create](refactors/core/remote-create-container-targets.md), [CLI coverage](testing/cli-coverage.md), [sponsor highlights](features/sponsor-highlights-clarity.md), [interactive metadata chat](features/interactive-metadata-chat.md), dPID/gather/CEL/SLM CLI leaves | Root/parser work serializes. IMC-10 owns its format syntax and help window; other chat leaves avoid those files. CLI-11 retires the composition plan after the command and startup contracts pass. |
| Agent tools | [Needle validation](migrations/needle-3-upgrade.md), [native routing](features/needle-router-native-parity.md), [generic adapters](features/ask-executor-adapters.md), [Swift profile](features/swift-slm-profile.md), [Julia profile](features/julia-decision-sidecar.md) | The 72-row CLI parity inventory, caller-owned Needle registry, frozen 200-case routing corpus, versioned binding contract, terminal execution, opt-in CLI route syntax, bounded native inputs, and local check binding are in place. PAR-02 completes nonlocal and fix-preview check parity next. ADP configuration, feature, adapter, and CLI leaves may consume the landed contracts. PAR native parity must pass before ADP-06/07 release. Swift/Julia feasibility may proceed earlier; their releases wait for ADP-07. SLM-17/JUL-11 own registry integration; SLM-23/JUL-16 own CLI syntax. Linux x86_64 Needle validation passed on Oklahoma and Linux ARM64 passed on Dante. The Windows x86_64 job is blocked on GSR-02. Runner 55645 cannot finish a tagged job. Shared documentation, registry, Needle, CLI, features, artifacts, and CI changes serialize. Native command seams serialize with their domain/CLI owners. MCP/ACP adapter tests reuse existing transports. Swift external gates do not block routing. |
| Metadata feedback | [interactive metadata chat](features/interactive-metadata-chat.md), [structured enrichment](refactors/core/research-activity-enrichment-integration.md) | Selectable provider and vocabulary candidates and chat fix review have landed; IMC-10/11 own format feedback. Serialize their exact workflow files with structured enrichment implementation. |
| Developer loop | [coverage throughput](testing/coverage-throughput.md) | Owns `Makefile` test/coverage targets, `.config/nextest.toml`, and `[profile.coverage]`. Runs alone against the CI/build and workspace graph locks. Gating a slow test requires agreement from whichever plan owns its acceptance criteria. |
| Build and operations | [runner pools](refactors/ci-runner-pools.md), [GitLab shell runner readiness](testing/gitlab-shell-runner-readiness.md), [Podman](migrations/podman-makefile.md) | GSR-02 is blocked and owns the stuck Windows runner. NDL-02, MOD-11, and POD-11 do not start until a tagged job finishes. Repository CI/Makefile integration has one owner. The external pilot reporting include and schedule belong to the Vergil automation lane. |
| Background service | [acornd](features/acornd-service.md) | Portable service values and strict daemon configuration are landed. ACD-03 next claims the workspace graph while ACD-07 may independently claim the service/RPC lock. ACD-12 and ACD-14 later claim workspace graph changes separately. ACD-04 through ACD-06 and ACD-08 serialize daemon runtime; ACD-09, ACD-18, and ACD-19 serialize CLI syntax with CLI composition. ACD-10, ACD-11, and ACD-13 own platform registration one at a time; ACD-20 through ACD-22 own separate external host checks. ACD-15 through ACD-17 and ACD-24 claim CI/build one at a time alongside Podman and runner work. Signing-key and native-host availability gate release without blocking pure service/store work. |
| SharePoint CLI capture | [SharePoint JSONL download](features/sharepoint-jsonl-download/index.md) | SPD-01 owns read-only tenant evidence and is blocked on an approved public-client registration and Graph consent. The contract, fixtures, feature-gated seams, ACRE `auth` profiles, validated Entra endpoints, account identity verifier and fixture-tested device grant are landed. The CLI resolver stays test-only until SPD-09 or SPD-21 supplies its first caller; those phases serialize shared `io/auth` edits. Coordinate later API/config root edits with API-06/API-14 and the ACRE format owner. SPD-24 owns shared cache lifecycle; SPD-04 owns the browser grant. SPD-05/14/15 independently implement native store leaves against SPD-24. SPD-20 serializes cache/grant lifecycle composition; SPD-21 reserves CLI parser/dispatch/help. SPD-13 owns Confirm/Spin stderr rendering and serializes with interactive metadata chat. SPD-08/09 serialize intake persistence/routing with CLI and graduation owners. SPD-16/17/18 own separate native acceptance after SPD-21; SPD-22 owns live device/reset acceptance. Native checks may proceed independently, but writes to the shared operations guide serialize. SPD-10 never injects faults into operational exports. SPD-26 follows SPD-10/22 and any one of SPD-16/17/18 to hand off shared functionality to HCI; SPD-11/12 still require all-platform acceptance for documentation and final closeout. Remaining native tests can run alongside HCI only with disjoint systems/files; shared auth and guide edits serialize. No Power Platform mutations or Vergil prerequisite. |
| Headless SharePoint CI | [Headless CI download](features/sharepoint-headless-ci-download/index.md) | Blocked until SPD-26: shared functionality, SPD-10/22 and any one native validation (SPD-16 OR SPD-17 OR SPD-18). Does not wait for other OS validation or SPD-11/12; HCI-06 separately proves the Linux CI runner. HCI-01 owns isolated federation feasibility; HCI-02 serializes ACRE/API configuration; HCI-03 owns host auth; HCI-04 serializes intake routing; HCI-05 owns an opt-in template under the CI/build lock without changing active jobs; HCI-06 owns isolated pilot inclusion/live acceptance; HCI-07 owns durable documentation and final gates. All phases are sequential. No operational form/exporter/credential changes, user cache, automatic apply or Vergil coupling. |
| Power Platform intake | [PowerApp/Power Automate highlight intake](features/powerapps-jsonl-intake/index.md) | SharePoint is the accepted default, including contributor access, browser recovery, collected export, native restore and immutable submission editing. GitLab intake/catalog remain Off. PA-32 is accepted and pruned: monthly coverage/history is published, a serialized five-minute refresh is On, counts match original receipts, and desktop/mobile views passed. Populated month/project cards link to counted original submissions using existing reviewer access. Contributor-account October/September navigation is accepted and PA-33 is pruned; dashboard/page/list Read grants are verified; Entra groups remain deferred; PA-29 owns standalone general-logbook intake/reporting without a project index; PA-12 waits for shared-target/group/capacity approval. Independent GitLab PA-15–17 retain isolated acceptance blockers; OAuth download and Vergil are separate. PA-15/16 serialize on the isolated intake route; PA-16 waits for PA-15. PA-24 is accepted and pruned; native archives and restore evidence are retained. PA-17 retains the packaging lock for its independent unfinished GitLab checks. PA-12 owns the deployment lock. |
| Vergil JSONL automation | [Vergil GitLab bot](features/vergil-gitlab-jsonl-automation/index.md) | PA-05/06/07/09 and BOT-02 are Waiting on validation, subject to prerequisites. BOT-01 owns approvals/runtime references; PA-03 owns service boot; PA-04 owns external callback integration; PA-05/BOT-02 serialize with form mutations on the intake route; PA-05 can create a draft MR, and PA-06/07 accept its review/delivery and human graduation/merge; PA-08/BOT-03 own external reporting publication; PA-09 records final operations. Never run bot ingestion and manual handback against the same GitLab inbox concurrently; the SharePoint retained list/snapshot is not that inbox and has no bot handback in scope. |
| Logbook graduation adapters | [sponsor highlights](features/sponsor-highlights-clarity.md) | HIL-15 owns database/MCP graduation tests, then HIL-16 owns GitLab automation review tests using the implemented admission contract. The Vergil plan's PA-07 is an external pilot, not a prerequisite for local schema or adapter work. Local `intake run --index` already consumes the current schema contract. HIL-19 owns its future target-specific adapter and CLI/help integration. PA-05 owns live baseline approval using the verified handback procedure; it does not own schema or CLI changes. |

SPD-24 restores `keyring-core` only with its shared store interface. SPD-05/14/15 restore their respective native store dependencies with working adapters. These phases may implement platform code independently, but their edits to `Cargo.toml`, `Cargo.lock`, `crates/acorn-host/Cargo.toml` and the cache module root serialize through the host authentication lock; each integration runs `make check-dependencies`.

## Safe preparatory batch

[Beam inference integration](features/beam-inference-integration.md) belongs to the Provider/model leaves lane. Release research and dedicated fixtures may proceed independently; shared API/configuration, CLI, and documentation integration serialize through the named locks.

On the current tree, these leaf slices can proceed together if they honor the lock restrictions:

| Slice | Exact primary ownership | Restriction |
|---|---|---|
| Remote-create preflight/rollback tests | `crates/acorn-cli/src/commands/create/` and dedicated tests | No shared CLI arguments or generated help. |
| HTML export adapter | `crates/acorn-cli/src/template/` and dedicated tests | Consume the landed context; no PowerPoint or export dispatch edits. |
| PowerPoint export adapter | `crates/acorn-lib/src/io/powerpoint/` and dedicated tests | Consume the landed context; no CLI template or export dispatch edits. |
| API endpoint/request provider migration | One named provider leaf and its tests | No API/config roots, manifests, or shared snapshots. |
| CEL runtime/corpus gate | Isolated runtime evaluation and dedicated policy fixtures | No manifests, module indexes, analyzer/check integration, CLI roots, or generated help. |
| CLI link coverage | Link command test module and dedicated fixtures | No parser/help or shared snapshot changes. |

## Handoff checklist

Before starting, record the plan, atomic slice, exact update/create/delete set, requested locks, public contracts, baseline, focused test, and landing order.

Before handoff, report changed files, public-contract changes, regenerated artifacts, validation results, and locks still needed. If the slice expands, stop and reschedule it rather than silently taking another lane.

After a phase completes, preserve durable evidence and remove the completed phase or task from its plan. Keep unfinished phase IDs stable, release its lock, update newly eligible phases, and remove any dangling dependency. Do not retain completed checklists as execution history.
