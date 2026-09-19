---
okf_version: "0.1"
---

# Parallel execution guide

This guide turns the dependency network into a scheduling contract for concurrent agents. It was reconciled against the active plan boundaries and workspace layout at version `0.2.0` on 2026-09-13.

## Verification result

- Every active plan is assigned to at least one work lane.
- Umbrella plans are explicitly coordination-only and cannot masquerade as concurrent implementation tasks.
- Shared integration surfaces have single-writer locks.
- The next schema isolation and extraction phases have explicit landing order and lock ownership.
- Broad plans are not considered concurrency-ready until an agent replaces directory/glob ownership with an exact atomic write set.

This verifies that the wiki can produce a safe concurrent batch; it does not claim every pair of plans is orthogonal. Orthogonality is a property of concrete implementation slices on a shared baseline, not of plan titles.

## Orthogonality test

Two implementation slices are orthogonal only when all four conditions hold:

1. Their exact write sets are disjoint. A directory or brace-expanded path in a plan is a planning boundary, not an implementation claim; the agent must expand it to named files before starting.
2. Neither slice changes a public type, trait, feature, serialized shape, CLI contract, or generated artifact consumed by the other.
3. They do not both require a single-writer integration file from the table below.
4. Each slice can be validated on the same baseline before either is merged.

Disjoint files alone are insufficient when one slice changes an interface consumed by the other. In that case, land a contract-only commit first, then rebase dependent slices.

## Single-writer integration locks

Only one active slice may own an integration lock. Other agents may edit leaf modules and tests, but they leave integration to the lock owner or a follow-up commit.

| Lock | Files | Typical collision | Rule |
|---|---|---|---|
| Workspace graph | `Cargo.toml`, crate `Cargo.toml` files, `Cargo.lock` | Feature defaults, crate extraction, dependency migration, WASM, size work | One manifest owner per batch; dependency additions and crate moves serialize. |
| Library roots | `crates/acorn-lib/src/{lib,error}.rs`, `crates/acorn-core/src/{lib,prelude}.rs` | Portable extraction, ownership cutovers, error migration | The active portability/extraction slice owns these files. |
| Module indexes | Any shared `mod.rs`, especially `lib.rs`, `io/mod.rs`, `io/api/mod.rs`, `analyzer/mod.rs` | Parallel leaf modules need imports/re-exports | Leaf agents do not edit the index; one integration owner performs module wiring after leaf changes land. |
| HTTP transport | `crates/acorn-lib/src/io/http/mod.rs` | API invocation, webhook redaction/dispatch, transfer retries, host extraction | One transport owner per batch; land logging, redaction, retry, or service-injection changes before dependent provider migrations. |
| CLI contract | `crates/acorn-cli/src/cli/`, `main.rs`, generated help | New commands, parse/execution split, startup/error work | One CLI-contract owner; command leaf modules can proceed separately against stable arguments. |
| Export dispatch | `crates/acorn-cli/src/commands/export/mod.rs`, `template/mod.rs` | Schema codecs, ZON, export context, PowerPoint | Renderer leaf work may proceed; dispatch/context integration serializes. |
| Test/fixture registry | Shared snapshots and the same fixture directory | Independent plans overwrite snapshots or reinterpret fixtures | Give each slice dedicated fixtures; one owner accepts shared snapshot changes. |
| CI/build entry points | `.gitlab-ci.yml`, `Makefile` | Runner pools, portable gates, Miri, Creusot, fuzz | Tool-specific scripts may be separate; one CI/build integration owner wires jobs and targets. |
| Documentation generation | `CommandLineHelp.md`, shared book indexes | Multiple CLI features regenerate the same output | Regenerate once after the batch, owned by the CLI integration slice. |
| Application configuration | `crates/acorn-lib/src/io/config/mod.rs` and shared config snapshots | PowerAutomate routing, API endpoint/environment work, OCI model profiles, and bucket transfer changes need shared re-exports or snapshots | Treat `config/mod.rs` and shared snapshots as integration-owned. PowerAutomate claims `application.rs` only for its typed deployment configuration; other leaf work must claim `application.rs`, `bucket.rs`, or `transfer.rs` explicitly. |
| Operation registry and RPC | `crates/acorn-lib/src/io/api/{json_rpc,grpc}/`, shared operation handlers, protocol configuration | PowerAutomate, JSON-RPC, webhook dispatch, tonic/prost, ACP, and MCP all consume callable services | The highest-priority lane owner lands one stable operation contract before adapters. No transport may fork handler DTOs or edit another transport's framing. |
| Webhook lifecycle | `crates/acorn-lib/src/io/api/webhooks/`, its database repository/migrations, worker composition | The implemented runtime, GitLab bot, and PowerAutomate provider share delivery, claim, retry, effect, callback, readiness, signing, and drain state | Preserve the implemented generic lifecycle contract. Provider-specific GitLab and PowerAutomate modules must not recreate readiness, retry, callback, or drain coordination. |

## Coordination-only plans

These plans describe architecture and ordering. Do not assign an agent to implement one as a whole:

- [Refactor initiative](refactors/initiative.md)
- [Crate boundary architecture](refactors/crate-boundary-architecture.md)
- [Portability roadmap](refactors/portability/roadmap.md)
- [API roadmap](refactors/api/roadmap.md)

Schedule their atomic child plans or a named migration phase with an exact write set. The coordinator owns shared integration locks and updates the roadmap after child slices land.

## Highest-priority lane execution order

This lane has priority over every other lane and is intentionally serialized. Later steps may prepare read-only design evidence, but they do not edit shared implementation surfaces until the preceding exit gate lands.

| Priority | Plan | Exit gate before the next plan |
|---:|---|---|
| 1 | [Protobuf/gRPC transport](refactors/api/protobuf-grpc-transport.md) | Committed proto, vendored generation, tonic server/client, limits, cancellation, and JSON-RPC equivalence tests pass. |
| 2 | [ACP inference and verified fixes](features/opencode-acp-inference.md) | Official ACP SDK adapter invokes registered inference services without replacing ACP wire/session semantics; suggestion-only fix policy is enforced. |
| 3 | [MCP client](features/mcp-client.md) | RMCP client sessions, capability negotiation, policy, cancellation, and shared-service integration pass without changing the existing MCP server contract. |

## Work lanes

Plans within one lane generally serialize unless the notes define leaf-module sharding. Plans in different lanes may run concurrently after checking secondary locks.

| Lane | Plans | Primary write surface | Parallel rule |
|---|---|---|---|
| RPC and repository automation — highest priority | [Power Automate deployment amendment](features/powerapps-gitlab-logbook-reporting.md), [Protobuf/gRPC](refactors/api/protobuf-grpc-transport.md), [ACP inference](features/opencode-acp-inference.md), [MCP client](features/mcp-client.md) | `io/api/{json_rpc,webhooks,gitlab,grpc}/`, provider-neutral webhook store, `io/{acp,mcp}/`, inference services, manifests, and CLI integration | The Power Automate amendment owns only utility, create-command, generated-help, and documentation files while its smoke test remains. Serialize later CLI-root integration with the numbered transport plans under one integration owner. |
| Portable domain boundary | [portability roadmap](refactors/portability/roadmap.md) | `acorn-lib` analyzer roots and feature checks | The in-tree analyzer kernel is green (`CheckBatch`/`CheckSummary`/`ValidationReport`/`ValeConfig` endomorphisms implemented and gated); package-graph work still requires an explicit extraction decision. |
| Schema features | [dPID](features/dpid-research-object-support.md), [ZON](features/zon-format-support.md), [CKAN/DCAT](migrations/ckan-dcat-crosswalk.md), [lifecycle/credit](refactors/core/research-activity-lifecycle-credit-fields.md) | `crates/acorn-schema/src/{lib.rs,research_activity,standard,validation}`, `crates/acorn-macros`, crosswalks, `.wiki/assets/standards-field-matrix.md`, `tests/fixtures/schema/`, `tests/fixtures/rrid/` | The portable RAD, RRID, ModelKit, OCP infrastructure, DataCite 4.7, RAiD, and structured-export baselines have landed. Remaining plans preserve the independent `hardware` and `standard::ocp` modules and compose them only through `infrastructure`. dPID's general activity identifier and RO-Crate adapter serialize on the implemented RAD/standards roots. |
| Semantic transforms | [Transformation laws](testing/semantic-transformation-laws.md) | Research-activity and crosswalk law tests, `io/adapter` and `io/workflow` tests, candidate-merge tests, focused CLI link tests | Pure test leaves may proceed independently against the landed RAD loss contract after each slice declares exact files. Direct-link/workflow integration serializes with CLI coverage and owns the CLI/export-dispatch locks only for the narrow repair window. |
| PID/schema | [dPID](features/dpid-research-object-support.md), [Creusot](testing/creusot-pid-checksums.md), [Miri](testing/miri-quality.md) | `crates/acorn-schema/src/pid`, PID validation, and verification wiring | Serialize PID roots, constants, validation registries, and proof/CI targets. The dPID portable parser leaf lands here before its provider work. No SciCrunch credential, DTO, remote-search, or TUI work belongs here. |
| Analyzer | [Analyzer composition](refactors/analyzer/analyzer-composition.md) (consolidated: link evidence, result boundaries, traversal, applicative audit, summary rendering) | `analyzer/{check.rs,error,service,host/{context,runtime,service,vale}.rs,readability/**}` + `analyzer/tests` + CLI `check/gather` consumers | Single-writer lane. Land slices in the order listed in the plan (recovery → link evaluator → traversal → applicative clarity → rendering split). Pure calculation slices may prepare in parallel when write sets are disjoint, but integration into `check.rs`/`service.rs`/`host/runtime.rs` serializes. |
| API shared contract | [Endpoint resolution](refactors/api/endpoint-resolution.md), [request parameters](refactors/api/request-parameter-composition.md), [invoke/parse](refactors/api/invoke-and-parse-wrappers.md), [environment](refactors/api/environment-bootstrap.md), [health](refactors/api/health-check-helpers.md), [errors](refactors/api/api-error-envelopes.md), [DTOs](refactors/api/response-dtos.md), [OpenAPI validation](refactors/api/openapi-endpoint-validation.md), [OpenAPI codegen](refactors/api/openapi-module-codegen.md), [capability matrix](refactors/api/provider-capability-matrix.md) | `io/api/mod.rs`, provider modules, `io/config/{application,bucket,mod}.rs` | Shared `io/api/mod.rs` and `io/config/mod.rs` changes serialize. Preserve the landed `SecretString` authentication contract. Endpoint work owns `config/application.rs`; environment work must claim `application.rs` or `bucket.rs` explicitly. Provider leaf migrations may run concurrently when each owns a different provider file. Codegen waits for the extracted layout. |
| Provider leaves | [dPID](features/dpid-research-object-support.md), [Hugging Face readiness](refactors/api/huggingface-request-readiness.md), [RAiD refinement](refactors/api/raid-api-refinement.md), [OCI models](features/oci-model-artifact-downloads.md) | `io/api/{dpid,huggingface,raid,harbor}.rs`, `io/oci.rs`, `io/config/mod.rs`, provider entity/enums, optional CLI/TUI leaves | Safe to shard by provider unless the shared API root, configuration, discovery provider/entity enums, CLI/TUI roots, or manifests change. The dPID provider leaf and fixtures may develop after its portable contract lands; module registration, endpoint configuration, and discovery integration serialize. SciCrunch is not scheduled; its retained contract review records the failed lookup and search gates. |
| Agent tools and sidecars | Needle and existing MCP server maintenance | `io/{mcp,sidecar,sync}` outside files claimed by the priority lane | Preserve implemented MCP runtime-policy, bundle, Needle projection, and shared `TargetConfig`/`SyncConfig` contracts. ACP and MCP-client changes belong only to the highest-priority lane. |
| Deterministic scraping | [Gather scraping](features/deterministic-gather-scraping.md) | `scrape/`, focused HTTP/discovery integration, `cli/commands/gather`, focused CLI integration, manifests | Independent of Rhai. Serialize HTTP, manifest, module-index, analyzer discovery, and CLI locks; use dedicated fixtures. |
| Extraction inference | [Structured inference](features/structured-extraction-inference.md) | `inference/`, one explicit provider adapter, focused scrape/gather integration, manifests | Follows deterministic scraping and coordinates with ACP/provider owners. Do not create a sidecar registry or take RPC/webhook locks in the first slice. |
| Database-backed artifacts | [Database-backed artifacts](features/database-backed-artifact-generation.md) | `io/database/{candidate,logbook,migrations,schema}.rs`, database tests, typed artifact-source leaf, focused CLI/MCP consumers | Database migrations, `database/{mod.rs,schema.rs}`, and repository contracts are single-writer surfaces. Database artifact work reuses the implemented bounded reporting-window and manifest contracts. |
| Export/OOXML | [interpolation](refactors/core/interpolation-primitives.md), [OOXML hardening](refactors/core/ooxml-interpolation-and-slide-copy.md), [OOXML fuzzing](refactors/core/ooxml-fuzzing.md), [export context](refactors/core/export-context.md) | `acorn-core` interpolation utilities, `io/powerpoint`, CLI export/template, schema image selection, fuzz | Slide/media leaf work may prepare without taking export dispatch or snapshot locks; PowerPoint/export integration and snapshots serialize. |
| Transfer/archive | [BagIt/import](features/unified-import-download-bagit.md), [OCI models](features/oci-model-artifact-downloads.md) | `io/{archive,bagit,download,source,oci}.rs`, `io/config/{bucket,transfer}.rs`, import/download/export commands | Library leaf work can shard by module while preserving the landed `SafePath` policy. `config/bucket.rs` is shared with API environment work; CLI download/export integration, `config/mod.rs`, shared snapshots, and manifests are locks. |
| CLI composition | [Parse/execution](refactors/core/cli-parse-execution-models.md), [startup](refactors/core/cli-startup-separation.md), [error boundary](refactors/core/cli-error-boundary.md), [remote create](refactors/core/remote-create-container-targets.md), [CLI coverage](testing/cli-coverage-phase-1.md), [dPID import](features/dpid-research-object-support.md) | CLI root/parse plus command leaf modules | Yield CLI roots and generated help to the highest-priority lane during its integration windows. Otherwise root/parse/error plans serialize; command leaves may prepare independently without argument registration. |
| Crate/host adapters | [acorn-cmd](refactors/acorn-cmd-extraction.md), [host traits](refactors/portability/host-abstraction-traits.md), [WASM](refactors/portability/wasm-feature-path.md), [thiserror](migrations/thiserror-adoption.md) | New crates, ownership roots, manifests | Host traits and analysis extraction wait for a green in-tree portable kernel and proven alternate consumers. Apply thiserror within the owning slice, not as a workspace-wide concurrent rewrite. |
| Duplication consolidation | [Workspace duplication consolidation](refactors/workspace-duplication-consolidation.md) | Schema PID leaves, core JSONC policy, CLI model synchronization, TUI widgets, provider leaves, test utilities | Treat each recommendation as a separate slice in its owning lane. TUI and distinct provider/test leaves may run independently after exact write sets are declared. |
| Build/CI/performance | [portable CI](refactors/portability/no-std-wasm-ci.md), [runner pools](refactors/ci-runner-pools.md), [macOS size](refactors/portability/macos-compact-unwind-size.md), [Miri](testing/miri-quality.md), [Creusot](testing/creusot-pid-checksums.md), [OOXML fuzzing](refactors/core/ooxml-fuzzing.md) | manifests, Makefile, GitLab CI, reusable reporting template | CI and Makefile changes serialize through one integration owner. |

## Concurrent batch before the next extraction

The following slices have disjoint primary write sets on the current tree. They can run at the same time if each obeys the named lock restriction:

| Slice | Exact primary ownership | Lock restriction |
|---|---|---|
| Remote-create hardening leaf | `acorn-cli/src/commands/create/` and dedicated tests | Do not edit shared CLI arguments/help; hand required argument changes to the CLI lock owner. |
| Shared interpolation contract leaf | `crates/acorn-core/src/util/` and dedicated contract tests | Do not integrate into PowerPoint or templates until the export lock owner schedules the parser and renderer adapter slice. |
| Hugging Face request-type leaf | `io/api/huggingface.rs` and dedicated tests | Do not edit `io/api/mod.rs`, manifests, or shared CLI import/download planners until integration. |

The CLI link coverage plan may join this batch when it adds only a dedicated test module and avoids shared CLI roots.

## Contract-first extraction pattern

The completed `acorn-schema` move established the pattern: freeze the portable/adapter contract, migrate consumers directly, assign one integration owner for manifests and module indexes, remove the old implementation instead of adding an ownership-masking facade, and run the full feature matrix before beginning the next extraction.

The same pattern applies to API providers: land transport/request/response contracts first, then shard provider files, then integrate through `io/api/mod.rs` or the extracted `acorn-api` root.

## Agent start and handoff checklist

Before work starts, record:

- the plan and atomic slice;
- the exact files to update/create/delete;
- integration locks requested;
- public contracts changed;
- prerequisite commit or branch baseline;
- focused validation command;
- expected landing order.

Before handoff, report changed files, new public contracts, regenerated artifacts, validation results, and any lock still needed. A slice that expands beyond its claimed write set stops and is rescheduled rather than silently taking another lane's files.
