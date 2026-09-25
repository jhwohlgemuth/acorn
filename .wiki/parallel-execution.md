---
okf_version: "0.1"
---

# Parallel execution guide

This guide turns the dependency network into a scheduling contract for concurrent agents. It was reconciled against the active plan boundaries and workspace layout at version `0.2.0` on 2026-09-23.

## Verification result

- Every active plan is assigned to at least one work lane.
- Umbrella plans are explicitly coordination-only and cannot masquerade as concurrent implementation tasks.
- Shared integration surfaces have single-writer locks.
- Active schema features and the next consumer-proven extraction phases have explicit landing order and lock ownership.
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
| Export dispatch | `crates/acorn-cli/src/commands/export/mod.rs`, `template/mod.rs` | Schema codecs, export context, PowerPoint | Renderer leaf work may proceed; dispatch/context integration serializes. |
| Test/fixture registry | Shared snapshots and the same fixture directory | Independent plans overwrite snapshots or reinterpret fixtures | Give each slice dedicated fixtures; one owner accepts shared snapshot changes. |
| CI/build entry points | `.gitlab-ci.yml`, `Makefile` | Runner pools, portable gates, Miri, Creusot, fuzz | Tool-specific scripts may be separate; one CI/build integration owner wires jobs and targets. |
| Documentation generation | `CommandLineHelp.md`, shared book indexes | Multiple CLI features regenerate the same output | Regenerate once after the batch, owned by the CLI integration slice. |
| Application configuration | `crates/acorn-lib/src/io/config/mod.rs` and shared config snapshots | PowerAutomate routing, API endpoint/environment work, bucket-source abstraction, Cosmos bucket sources, OCI model profiles, and bucket transfer changes need shared re-exports or snapshots | Treat `config/mod.rs` and shared snapshots as integration-owned. PowerAutomate claims `application.rs` only for its typed deployment configuration; the bucket-source plan owns `bucket_source.rs` and its compatibility snapshots before Cosmos extends that leaf; other work must claim `application.rs`, `bucket.rs`, or `transfer.rs` explicitly. |
| Operation registry and RPC | `crates/acorn-lib/src/io/api/{json_rpc,grpc}/`, shared operation handlers, protocol configuration | PowerAutomate, JSON-RPC, webhook dispatch, tonic/prost, ACP, and MCP all consume callable services | The highest-priority lane owner lands one stable operation contract before adapters. No transport may fork handler DTOs or edit another transport's framing. |
| Webhook lifecycle | `crates/acorn-lib/src/io/api/webhooks/`, its database repository/migrations, worker composition | The implemented runtime, GitLab bot, and PowerAutomate provider share delivery, claim, retry, effect, callback, readiness, signing, and drain state | Preserve the implemented generic lifecycle contract. Provider-specific GitLab and PowerAutomate modules must not recreate readiness, retry, callback, or drain coordination. |

## Coordination-only plans

These plans describe architecture and ordering. Do not assign an agent to implement one as a whole:

- [Refactor initiative](refactors/initiative.md)
- [Crate boundary architecture](refactors/crate-boundary-architecture.md)
- [Portability roadmap](refactors/portability/roadmap.md)
- [API roadmap](refactors/api/roadmap.md)
- [CLI test coverage initiative](testing/index.md)

Schedule their atomic child plans or a named migration phase with an exact write set. The coordinator owns shared integration locks and updates the roadmap after child slices land.

## Highest-priority lane execution order

This lane has priority over every other lane and is intentionally serialized. Later steps may prepare read-only design evidence, but they do not edit shared implementation surfaces until the preceding exit gate lands.

| Priority | Plan | Exit gate before the next plan |
|---:|---|---|
| 1 | [ACP inference and verified fixes](features/opencode-acp-inference.md) | Bounded suggestion-only fixes, trusted WinGet/Homebrew/Vale/Chromium recipes, shared doctor policy, opt-in smoke tests, and CLI/TUI integration pass without replacing ACP wire/session semantics. |
| 2 | [MCP client](features/mcp-client.md) | RMCP client sessions, capability negotiation, policy, cancellation, and shared-service integration pass without changing the existing MCP server contract. |

## Work lanes

Plans within one lane generally serialize unless the notes define leaf-module sharding. Plans in different lanes may run concurrently after checking secondary locks.

| Lane | Plans | Primary write surface | Parallel rule |
|---|---|---|---|
| RPC and repository automation — highest priority | [Power Automate deployment amendment](features/powerapps-gitlab-logbook-reporting.md), [ACP inference](features/opencode-acp-inference.md), [MCP client](features/mcp-client.md) | Implemented `io/api/{json_rpc,webhooks,gitlab,grpc}/`, provider-neutral webhook store, `io/{acp,mcp}/`, `fix/`, doctor/TUI views, inference services, manifests, and CLI integration | The Power Automate amendment owns only utility, create-command, generated-help, and documentation files while its smoke test remains. Preserve the implemented gRPC and inference contracts; keep one integration owner across ACP phases 5-8, then serialize MCP CLI-root integration after the ACP exit gate. |
| Portable domain boundary | [portability roadmap](refactors/portability/roadmap.md) | `acorn-lib` analyzer roots and feature checks | The in-tree analyzer kernel is green (`CheckBatch`/`CheckSummary`/`ValidationReport`/`ValeConfig` endomorphisms implemented and gated); package-graph work still requires an explicit extraction decision. |
| Schema features | [dPID](features/dpid-research-object-support.md), [CKAN/DCAT](migrations/ckan-dcat-crosswalk.md), [structured enrichment](refactors/core/research-activity-enrichment-integration.md) | `crates/acorn-schema/src/{lib.rs,research_activity,standard,validation}`, `crates/acorn-macros`, crosswalks, `.wiki/assets/standards-field-matrix.md`, `tests/fixtures/schema/`, `tests/fixtures/rrid/` | The portable RAD, RRID, ModelKit, OCP infrastructure, DataCite 4.7, RAiD, typed activity metadata, ZON serialization, and structured-export baselines have landed. dPID, standards adapters, enrichment merge rules, and logbook graduation serialize on the implemented RAD/standards roots. |
| Schema codecs | [ZON codec hardening](refactors/zon-codec-hardening.md), [JSON Lines recognition and export crosswalk](features/json-lines-export-crosswalk.md) | ZON owns `crates/acorn-schema/src/standard/zon/{decode.rs,encode.rs,mod.rs,tests/}`; JSON Lines owns `crates/acorn-core/src/util/alloc.rs`, `crates/acorn-schema/src/types.rs`, and dedicated fixtures during its portable codec slice | The codec leaves may run concurrently only while they avoid shared codec-law tests, module roots, manifests, and fixture indexes. JSON Lines source-inference, CLI, export-dispatch, documentation, and generated-help integration lands later under the corresponding single-writer locks. |
| Semantic transforms | [Transformation laws](testing/semantic-transformation-laws.md) | Research-activity, custom codec/parser, standards wire-codec, crosswalk, adapter, workflow, candidate-merge, and focused CLI link tests | Pure test leaves may proceed independently after each slice declares exact files. PID roots coordinate with the PID/schema lane; `io/oci.rs` coordinates with provider and transfer lanes; crate manifests are locks. Direct-link/workflow integration serializes with CLI coverage and owns the CLI/export-dispatch locks only for the narrow repair window. |
| PID/schema | [dPID](features/dpid-research-object-support.md), [Creusot](testing/creusot-pid-checksums.md), [Miri](testing/miri-quality.md) | `crates/acorn-schema/src/pid`, PID validation, and verification wiring | Serialize PID roots, constants, validation registries, and proof/CI targets. The dPID portable parser leaf lands here before its provider work. No SciCrunch credential, DTO, remote-search, or TUI work belongs here. |
| API shared contract | [Endpoint resolution](refactors/api/endpoint-resolution.md), [request parameters](refactors/api/request-parameter-composition.md), [invoke/parse](refactors/api/invoke-and-parse-wrappers.md), [environment](refactors/api/environment-bootstrap.md), [health](refactors/api/health-check-helpers.md), [errors](refactors/api/api-error-envelopes.md), [DTOs](refactors/api/response-dtos.md), [OpenAPI validation](refactors/api/openapi-endpoint-validation.md) | `io/api/mod.rs`, provider modules, `io/config/{application,bucket,mod}.rs` | Shared `io/api/mod.rs` and `io/config/mod.rs` changes serialize. Preserve the landed `SecretString` authentication contract. Endpoint work owns `config/application.rs`; environment work must claim `application.rs` or `bucket.rs` explicitly. Provider leaf migrations may run concurrently when each owns a different provider file. |
| Provider leaves | [dPID](features/dpid-research-object-support.md), [Hugging Face readiness](refactors/api/huggingface-request-readiness.md), [RAiD refinement](refactors/api/raid-api-refinement.md), [OCI models](features/oci-model-artifact-downloads.md) | `io/api/{dpid,huggingface,raid,harbor}.rs`, `io/oci.rs`, `io/config/mod.rs`, provider entity/enums, optional CLI/TUI leaves | Safe to shard by provider unless the shared API root, configuration, discovery provider/entity enums, CLI/TUI roots, or manifests change. The dPID provider leaf and fixtures may develop after its portable contract lands; module registration, endpoint configuration, and discovery integration serialize. SciCrunch is not scheduled; its retained contract review records the failed lookup and search gates. |
| Agent tools and sidecars | [Needle 3 migration](migrations/needle-3-upgrade.md), [SLM sidecars](features/slm-sidecars.md), existing MCP server maintenance | `io/{mcp,sidecar,sync}`, Needle API/constants, `scripts/{needle,slm}`, dedicated sidecar fixtures/docs, and focused CLI leaves | The single Kitfile-versioned ModelKit asset contract has landed. Needle engine work and SLM runner/artifact leaves may proceed independently. Serialize shared sidecar roots, CLI, manifests, OCI contracts, generated help, `.gitlab-ci.yml`, and `Makefile`; no concurrent CI/build owner may provision the required macOS runner or wire SLM jobs. Preserve MCP runtime-policy, tool projection, and `TargetConfig`/`SyncConfig` contracts. Schemer, enrichment persistence, and semantic-decision review remain deferred to their owning boundaries; ACP and MCP-client changes remain in the highest-priority lane. |
| Deterministic scraping | [Gather scraping](features/deterministic-gather-scraping.md) | `scrape/`, focused HTTP/discovery integration, `cli/commands/gather`, focused CLI integration, manifests | Independent of Rhai. Serialize HTTP, manifest, module-index, analyzer discovery, and CLI locks; use dedicated fixtures. |
| Extraction inference | [Structured inference](features/structured-extraction-inference.md) | `inference/`, one explicit provider adapter, focused scrape/gather integration, manifests | Follows deterministic scraping and coordinates with ACP/provider owners. Do not create a sidecar registry or take RPC/webhook locks in the first slice. |
| Database-backed artifacts | [Database-backed artifacts](features/database-backed-artifact-generation.md) | `io/database/{candidate,logbook,migrations,schema}.rs`, database tests, typed artifact-source leaf, focused CLI/MCP consumers | Database migrations, `database/{mod.rs,schema.rs}`, and repository contracts are single-writer surfaces. Database artifact work reuses the implemented bounded reporting-window and manifest contracts. |
| Export/OOXML | [JSON Lines recognition and export crosswalk](features/json-lines-export-crosswalk.md), [interpolation](refactors/core/interpolation-primitives.md), [OOXML hardening](refactors/core/ooxml-interpolation-and-slide-copy.md), [OOXML fuzzing](refactors/core/ooxml-fuzzing.md), [export context](refactors/core/export-context.md) | JSON Lines structured-export integration, `acorn-core` interpolation utilities, `io/powerpoint`, CLI export/template, schema image selection, fuzz | Portable codec and slide/media leaf work may prepare without taking export dispatch or snapshot locks. JSON Lines and PowerPoint/export integration serialize on `crates/acorn-cli/src/commands/export/mod.rs`, shared snapshots, and generated documentation. |
| Transfer/archive | [BagIt/import](features/unified-import-download-bagit.md), [bucket source abstraction](refactors/bucket-source-abstraction.md), [Cosmos buckets](features/azure-cosmos-buckets.md), [OCI models](features/oci-model-artifact-downloads.md) | `io/{archive,bagit,download,source,oci}.rs`, provider API leaves, `io/config/{bucket,bucket_source,transfer}.rs`, import/download/export commands | Library leaf work can shard by module while preserving the landed `SafePath` policy. Land the bucket-source type, shorthand parser, repository adapter, and compatibility suite after the shared transfer baseline and before Cosmos integration. Its config snapshots and import/download parser changes are integration locks. A Cosmos client leaf may prepare independently, but its source variant and provider registration wait for the prerequisite; manifests, module indexes, shared snapshots, and CLI download integration serialize with BagIt/import, API environment, and OCI owners. |
| CLI composition | [Parse/execution](refactors/core/cli-parse-execution-models.md), [startup](refactors/core/cli-startup-separation.md), [error boundary](refactors/core/cli-error-boundary.md), [remote create](refactors/core/remote-create-container-targets.md), [CLI coverage](testing/cli-coverage-phase-1.md), [dPID import](features/dpid-research-object-support.md), [JSON Lines export](features/json-lines-export-crosswalk.md) | CLI root/parse plus command leaf modules | Yield CLI roots and generated help to the highest-priority lane during its integration windows. Otherwise root/parse/error and JSON Lines argument-registration work serialize; command and portable-codec leaves may prepare independently without argument registration. |
| Crate/host adapters | [dependency features](refactors/portability/dependency-feature-defaults.md), [thiserror](migrations/thiserror-adoption.md), and named extraction slices coordinated by the [crate architecture](refactors/crate-boundary-architecture.md) | New crates, ownership roots, manifests | `acorn-cmd` and the initial additive `acorn-host` features are complete foundations. Further slices require a concrete consumer and exact ownership cutover. Apply thiserror within the owning slice, not as a workspace-wide concurrent rewrite. |
| Duplication consolidation | [Workspace duplication consolidation](refactors/workspace-duplication-consolidation.md) | Core JSONC policy, schema PID leaves, CLI model synchronization and remote serve dispatch, TUI gather/widgets, Rhai projection, CLI test utilities | Treat each recommendation as a separate slice in its owning lane. JSONC waits for active core portability work; TUI gather follows deterministic source ownership; PID work serializes with the schema lane; remote serve dispatch yields to RPC/MCP and CLI composition. Independent TUI widget, Rhai, model-sync, and test-helper leaves may proceed after exact write sets are declared. |
| Build/CI/performance | [runner pools](refactors/ci-runner-pools.md), [macOS size](refactors/portability/macos-compact-unwind-size.md), [Miri](testing/miri-quality.md), [Creusot](testing/creusot-pid-checksums.md), [OOXML fuzzing](refactors/core/ooxml-fuzzing.md) | manifests, Makefile, GitLab CI, reusable reporting template | CI and Makefile changes serialize through one integration owner; preserve the implemented portability job and dependency gates. |

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
