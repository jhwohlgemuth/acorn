---
okf_version: "0.1"
---

# Plan dependency network

This map records execution dependencies rather than topical links. An arrow means the left plan establishes a boundary or contract required by the right plan. It was reconciled against the workspace at version `0.2.0` on 2026-09-27.

Dependency order does not by itself prove that plans are safe to run concurrently. The [parallel execution guide](parallel-execution.md) overlays exact write-set rules, single-writer integration locks, work lanes, and verified concurrent batches on this network.

## Highest-priority RPC and automation path

```text
MCP client
```

This is one serialized delivery lane in the displayed scheduling order. The implemented PowerAutomate workflow has released the shared API integration locks. Its [secret-generation amendment](features/powerapps-gitlab-logbook-reporting.md) is limited to the existing utility, create-command, generated-help, and documentation boundaries while deployment validation remains open. ACP interoperability and CLI/doctor/TUI integration are complete, so the RMCP client can add outbound MCP interoperability next.

## Critical portability path

```text
dependency feature defaults
          |
          v
          v
API/store/export/service boundary stabilization
          |
          v
remaining consumer-proven acorn-* extractions
          |
          v
direct interface composition
          |
          v
extend the implemented portability matrix
```

`acorn-core`, `acorn-schema`, `acorn-cmd`, and the initial `acorn-host` capabilities are extracted and consumed directly. `acorn-lib` has no schema facade: network model resolution, persistence, contextual RAD formatting, OpenCode CST preservation, and remaining host implementations stay explicit. The alloc-only analyzer kernel and current WASM/package matrix are enforced in CI. Further extraction, browser adapters, or host ports require a concrete alternate consumer and a new atomic plan.

## Target ownership network

| Target package | Source boundary to prepare now | Direct prerequisites | Plans unlocked |
|---|---|---|---|
| `acorn-diff` | New generic alloc-only unified-diff and RFC 6902 transforms over supplied values | Dependency-feature contract; concrete CLI and inference consumers | Semantic file comparison and inference patch output |
| `acorn-schema` | Complete: schema, validation, string/byte codecs, vocabulary and prompt assets | `acorn-core` | Schema features, standards crosswalks, `acorn-analysis` |
| `acorn-analysis` | `analyzer/{check,error,readability}.rs` over supplied content | Host isolation; analyzer algebra plans | Service extraction and portable checks |
| `acorn-api` | `io/api/` protocol, JSON-RPC, gRPC, webhook verification, endpoint, request, response, and auth logic | API roadmap and operation-registry contracts | `acorn-service`, `acorn-host`, provider features |
| `acorn-store` | `io/database/`, provider-neutral webhook delivery, operation, effect, and callback state | API DTO/error boundaries | GitLab workflows, service extraction |
| `acorn-export` | PowerPoint/OOXML and template transforms that return bytes | Export context; interpolation; OOXML hardening | Native/browser renderer adapters |
| `acorn-host` | Filesystem, configuration, HTTP, download, source, and terminal slices are complete; document conversion, doctor, and domain adapters remain | Extracted domains and proven consumers | CLI/TUI direct composition |
| `acorn-wasm` | Complete minimal supplied-value validation and readability bindings | Concrete browser workflow before any new adapter plan | None currently |
| `acorn-service` | Discovery, analysis, GitLab bot, MCP/server orchestration | Analysis/API/store/export boundaries | Thin interface composition roots |
| `acorn-cmd` | Complete: `cmd!`, `args!`, command specifications, discovery, checked and bounded execution, and optional shell linting | Command-word parsing remains isolated inside `acorn-macros` | Direct dependency-light command capability |
| `acorn-lib` | Host implementations awaiting extraction and ACORN-specific host extensions | Owning domains directly | Shrinks at each explicit ownership cutover |

## Cross-plan dependency edges

### Portability and crate boundaries

- [Dependency feature defaults](refactors/portability/dependency-feature-defaults.md) precedes every extraction because each new manifest needs an explicit capability set.
- The [portability roadmap](refactors/portability/roadmap.md) records the completed in-tree analyzer boundary after `acorn-schema` extraction.
- The downstream host boundary is implemented as additive `acorn-host` features with a three-crate compile fixture; future native slices require their own atomic plans and proven consumers.
- [Crate boundary architecture](refactors/crate-boundary-architecture.md) coordinates the analyzer, API, export, command, and portability boundaries; schedule the named extraction plan rather than the umbrella.
- Minimal `acorn-wasm` bindings consume the existing portable schema and analyzer boundaries directly. `make check-portable`, dependency-deny checks, and the optional/scheduled portability job are implemented; extend them in the same change as any accepted package or adapter.

### Analyzer and API

- The analyzer baseline provides ordered `CheckBatch` composition, additive `CheckSummary` counts, pure `ValeConfig` normalization, shared result recovery, unified link evidence/evaluation, shared ordered category traversal, and host-owned CLI/WASM summary rendering. Validation tree flattening remains portable through the ordered `ValidationReport`.
- The portable RAD standards baseline has landed: the [field matrix](assets/standards-field-matrix.md), crosswalk/loss contract, DataCite 4.7 audit and fixtures, warning-aware `ResearchOutput <-> DataCite Record`, strict `ResearchActivity <-> RAiD Metadata`, and opt-in `ResearchActivity -> DataCite Project` are available to analyzer workflows and [CKAN/DCAT](migrations/ckan-dcat-crosswalk.md).
- The portable RRID baseline has landed: strict parsing, PID integration, local text/PDF/DOCX discovery, generic lifecycle persistence, typed RAD validation, standards fixtures, non-project identity, and the RRID `TypedRelatedResource <-> DataCite RelatedIdentifier` leaf crosswalk all remain independent of provider access.
- The retained [SciCrunch contract review](assets/rrid-scicrunch-contract-matrix.md) records a **not planned** decision after both lookup and search failed closed. SciCrunch work is not an execution dependency or provider-lane job; a future proposal requires new contract evidence and a new plan.
- [dPID research-object support](features/dpid-research-object-support.md) adds a portable root/version/path identifier and RO-Crate profile before a read-only resolver, gather integration, and explicit import. Its general activity-identifier field and crosswalk must preserve the implemented RAD field and loss contracts; provider, discovery, persistence, and CLI roots remain integration locks. Generic DID resolution, nanopublications, minting, and content materialization are outside its boundary.
- [API roadmap](refactors/api/roadmap.md) records the completed shared authentication builders, `SecretString` credential migration, and reusable webhook contracts. Its active children are [endpoint resolution](refactors/api/endpoint-resolution.md), [request parameters](refactors/api/request-parameter-composition.md), [invoke/parse wrappers](refactors/api/invoke-and-parse-wrappers.md), [environment bootstrap](refactors/api/environment-bootstrap.md), [health helpers](refactors/api/health-check-helpers.md), [error envelopes](refactors/api/api-error-envelopes.md), [response DTOs](refactors/api/response-dtos.md), and [OpenAPI endpoint validation](refactors/api/openapi-endpoint-validation.md). Provider-specific follow-ons include [Hugging Face readiness](refactors/api/huggingface-request-readiness.md) and [RAiD refinement](refactors/api/raid-api-refinement.md).
- The implemented `io::api::json_rpc` runtime establishes typed service registration and direct JSON-RPC 2.0 request/notification dispatch.
- The implemented `io::api::webhooks` runtime consumes the operation registry and provides durable operations and callbacks, shared outcome timestamps, readiness, and graceful draining.
- GitLab repository automation consumes both implemented foundations and proves registered flat-file workflows against exact repository revisions with deployment-owned policy and durable effect reconciliation.
- The implemented [PowerAutomate reporting workflow](../docs/src/automation/powerautomate-reporting.md) reuses those foundations for shared-secret form intake, rolling bucket merge requests, reviewer-controlled graduation, bounded monthly PDF reports, immutable GitLab packages, and SAS-authenticated callbacks. Its [deployment amendment](features/powerapps-gitlab-logbook-reporting.md) adds generic ACORN secret generation without reclaiming the shared API transport lock. All PowerAutomate protocol code remains isolated to `io::api::powerautomate`.
- The implemented Protobuf/gRPC transport follows the stable operation contract and supplies tonic/prost server and client adapters without encoding protobuf inside JSON-RPC.
- [MCP client](features/mcp-client.md) follows the stable registry and ACP service contract, using RMCP for outbound MCP sessions rather than translating MCP envelopes into JSON-RPC or gRPC.

### Scraping and Rhai exec

- The optional semantic-decision experiment in [SLM sidecars](features/slm-sidecars.md) starts with independent SemIf/Open-Jev fixtures and consumes a runnable [Needle 3](migrations/needle-3-upgrade.md) baseline only for its comparison. Production review follows the proposal-only SLM boundary and requires a measured promotion decision. Schemer, persistence, and semantic decisions are deferred follow-ons; none blocks explicit Tongue, Redact, and Title delivery or adds search/tool-execution scope.

- [Needle 3 migration](migrations/needle-3-upgrade.md) now owns only real-engine validation on the remaining Windows and Linux release hosts. [SLM sidecars](features/slm-sidecars.md) may proceed independently, but both plans serialize shared sidecar roots, manifests, OCI materialization, and CI/build locks.
- [SLM sidecars](features/slm-sidecars.md) owns pinned Swift runners and a bounded JSON-lines process protocol for stable, explicitly invoked Tongue, Redact, and Title commands. Artifact publication precedes Rust checksum pinning. License/redistribution approval and a macOS ARM64 GitLab runner are hard release prerequisites, so SLM also takes the CI/build lock when those jobs are wired. It consumes [structured extraction inference](features/structured-extraction-inference.md) only for later Schemer work and the canonical database boundary only for later durable reconciliation.

- The implemented [configured Rhai exec extensions](../docs/src/packages/exec.md) establish capability-free local CLI quality gates with typed `ExecutionResult<T>` outputs. Application configuration maps names relative to the project-root `.acorn/` directory to hooks, while an explicit `absolute: true` escape hatch is warned, hardened, and limited to local callers. Host inference recognizes valid extensionless Rhai through ABI-gated restricted compilation without authorizing unlisted scripts.
- [Deterministic gather scraping](features/deterministic-gather-scraping.md) independently adds bounded HTML fetching and typed CSS extraction under existing HTTP/offline policy.
- [Structured extraction inference](features/structured-extraction-inference.md) follows deterministic scraping and adds one explicit typed provider adapter before any generic registry is considered.
- The implemented [Rhai exec package](../docs/src/packages/exec.md) now includes local capability-brokered workflow hooks, explicit operation effects, digest-pinned inline or deployment-file manifests, typed document and file proposals, transactional application, audit records, and local validation commands. Remote transports remain unable to invoke configured scripts.

### Export and interfaces

- [Portable semantic diff and inference output](features/semantic-diff-output.md) follows [dependency feature defaults](refactors/portability/dependency-feature-defaults.md) for its new `acorn-diff` manifest and the [MCP client](features/mcp-client.md) for shared CLI/inference locks. Its generic alloc-only engine lands before the native ACORN semantic adapter and CLI/suggestion composition; it does not claim that current std-gated codecs are portable or add an `acorn-lib` facade.
- [ZON codec hardening](refactors/zon-codec-hardening.md) is a portable `acorn-schema` leaf with no feature-integration prerequisite. It owns `crates/acorn-schema/src/standard/zon/` while active and coordinates any shared codec-law tests or fixtures with [semantic transformation laws](testing/semantic-transformation-laws.md).
- [Semantic transformation laws](testing/semantic-transformation-laws.md) consumes the canonical classifications in the [standards field matrix](assets/standards-field-matrix.md) and coordinates its direct-link/workflow commuting test with [CLI test coverage phase 1](testing/cli-coverage-phase-1.md). RAD normalization, custom codec/parser, standards wire-codec, linked-data, and merge laws can start as exact-file leaves against current contracts; PID roots, `io/oci.rs`, crate manifests, and any link-command repair serialize on their owning schema, provider, crate, CLI, and export-dispatch locks.
- [Interpolation primitives](refactors/core/interpolation-primitives.md) precede completion of [export context](refactors/core/export-context.md) and parser-backed PowerPoint text/list interpolation. Shared authored-image and draft-placeholder selection has already started the export context independently.
- The structured slide-list, relationship, content-type, and ASPECT transforms in [OOXML hardening](refactors/core/ooxml-interpolation-and-slide-copy.md) are ready for [OOXML fuzzing](refactors/core/ooxml-fuzzing.md). Parser-backed interpolation and the completed export context still precede `acorn-export` extraction.
- [CLI parse/execution models](refactors/core/cli-parse-execution-models.md), [startup separation](refactors/core/cli-startup-separation.md), and [CLI error boundary](refactors/core/cli-error-boundary.md) prepare the CLI as a composition root after service and adapter extraction.
- [Remote container creation](refactors/core/remote-create-container-targets.md) is an in-progress CLI leaf: preflight, rollback, ownership-aware cleanup, failure-path tests, and disposable-host validation remain after remote target selection landed.
- The [CLI coverage initiative](testing/index.md) coordinates its schedulable [phase 1](testing/cli-coverage-phase-1.md); the initiative is not itself a test implementation slice.
- [Workspace duplication consolidation](refactors/workspace-duplication-consolidation.md) must land its JSONC-policy slice after the active core portability work and before the corresponding host boundary moves again. Its PID work follows the schema lane; TUI gather follows deterministic source ownership and coordinates with gather scraping; remote serve dispatch yields to RPC/MCP and CLI composition; Rhai, model-sync, status-bar, and test-helper leaves follow their owning locks. Provider status consolidation remains in [API health-check helpers](refactors/api/health-check-helpers.md).
- [thiserror adoption](migrations/thiserror-adoption.md) is incremental across extracted crates; it must not delay portable extraction, but each new crate must start with owned portable errors.
- The implemented portable `acorn-schema::modelkit` contract establishes the KitOps-matched authoring-YAML and resolved-config boundary consumed by [ModelKit and ModelPack OCI interoperability](features/oci-model-artifact-downloads.md). The OCI plan preserves the normalized registry materialization boundary used by the landed Needle sidecar, reuses the safe archive boundary from the landed BagIt archive import and download path, and exposes only its selected GGUF output to inference.

### Bucket sources

- [Azure Cosmos DB bucket downloads](features/azure-cosmos-buckets.md) requires the source abstraction and adds one read-only variant and adapter for a configured Cosmos DB for NoSQL container or logical partition. It does not add Cosmos to `acorn_core::Repository`, replace SQLite/DuckDB, or introduce uploads, binary storage, change-feed synchronization, or a generic provider registry.

### Database-backed artifacts

- The implemented [PowerAutomate reporting workflow](../docs/src/automation/powerautomate-reporting.md) uses merged bucket files and establishes bounded reporting-window and artifact-manifest contracts.
- [Database-backed artifact generation](features/database-backed-artifact-generation.md) consumes the implemented canonical database contract and later reuses those reporting contracts.

### Build, verification, and performance

- [Runner-pool isolation](refactors/ci-runner-pools.md) is in progress: repository tag/config support has landed, while specialized routing, isolated build storage, concurrency, and deployment validation remain.
- [macOS compact-unwind reduction](refactors/portability/macos-compact-unwind-size.md), [Miri](testing/miri-quality.md), [Creusot](testing/creusot-pid-checksums.md), and [OOXML fuzzing](refactors/core/ooxml-fuzzing.md) serialize on Makefile and CI integration even when their leaf tests are disjoint.
- The [refactor initiative](refactors/initiative.md), [crate architecture](refactors/crate-boundary-architecture.md), [portability roadmap](refactors/portability/roadmap.md), and [API roadmap](refactors/api/roadmap.md) are coordination records rather than schedulable jobs.

## Scheduling conflicts and shared files

| Shared boundary | Plans that overlap | Coordination rule |
|---|---|---|
| `crates/acorn-schema/src/` | ZON codec hardening, structured enrichment, dPID, CKAN/DCAT | Schema flattening, the portable RRID baseline, the ModelKit/Kitfile contract, OCP infrastructure, typed activity metadata, and ZON serialization have landed. Preserve singular schema ownership and the codec/adapter boundary; feature plans must not reintroduce host access. ZON hardening owns only `standard/zon/` and dedicated fixtures. Plans share `lib.rs`, codec-law tests, enum/module roots, validation, standards adapters, and shared fixtures as integration locks. |
| `crates/acorn-lib/src/analyzer/` | [dPID](features/dpid-research-object-support.md), ACP fixes, [structured enrichment](refactors/core/research-activity-enrichment-integration.md) | Preserve the stable check/result, recovery, traversal, and link-evidence contracts before moving service and host implementations. ACP augments discovery with doctor-related findings; dPID resolution integration and provider enrichment therefore serialize on the same discovery leaf. |
| `crates/acorn-lib/src/io/api/` | PowerAutomate, JSON-RPC, implemented webhook runtime, GitLab, implemented gRPC, API roadmap children, Cosmos buckets, dPID, OCI, inference | The implemented gRPC adapter owns its transport root and shares the stable operation contract. Provider leaves may develop separately only when they do not touch `io/api/mod.rs`, shared errors, operation registration, or transport configuration. |
| `crates/acorn-schema/src/agent/`, `crates/acorn-lib/src/analyzer/fix/`, and `crates/acorn-lib/src/io/{acp,mcp}/` | ACP inference, MCP client, database-backed artifacts, OCI model interoperability, service extraction | Preserve the implemented RMCP server/runtime-policy contracts. ACP uses the official SDK, all fix lifecycle code remains consolidated in `analyzer::fix` beside `analyzer::check`, and MCP uses RMCP; these may adapt to shared services but none owns the JSON-RPC codec. Serialize agent schema, module indexes, manifests, CLI roots, and feature changes. |
| `crates/acorn-lib/src/io/http/mod.rs` | API invocation, transfer/download hardening | Preserve request-level redirect, response-bound, redaction, and anonymous-fallback controls; serialize further service-injection changes. |
| `crates/acorn-lib/src/io/config/` | PowerAutomate project routing, API endpoint/environment, Cosmos buckets, dPID identity persistence, OCI models, BagIt download/import | PowerAutomate claims `application.rs` only for its typed deployment configuration and test snapshots. The landed bucket-source abstraction lives in `bucket/source.rs`; `bucket/mod.rs` owns orchestration, `transfer.rs` owns transfer primitives, and `config/mod.rs` is the shared model/registry and re-export integration root. Cosmos may extend the source leaf but must not reopen repository extraction. Claim leaf files exactly and serialize `config/mod.rs` or shared snapshot changes. |
| `crates/acorn-lib/src/io/database/` | Canonical RAD database, provider-neutral webhook store, database-backed artifact sources | The webhook runtime replaces GitLab-specific bot tables with dedicated generic lifecycle tables. Give each store a leaf module and tests; serialize `database/{mod.rs,schema.rs}`, migrations, and public repository integration. |
| `crates/acorn-lib/src/io/powerpoint/` | OOXML hardening, interpolation, export context, fuzzing | Structured slide/media transforms and focused tests are the baseline. Finish parser-backed interpolation, shared field projection, and fuzzing before moving them to `acorn-export`. |
| `crates/acorn-cli/src/main.rs` and `cli/` | RPC/webhook serving, GitLab bot, ACP inference, MCP client, CLI model/startup/error plans, dPID import | The highest-priority lane has first claim on its CLI integration windows. Preserve implemented MCP commands; register one transport, workflow, or format surface per merge and regenerate help once per integration batch. |
| Workspace manifests and CI | JSON-RPC/webhook dependencies, tonic/prost, ACP SDK, RMCP client, crate extraction, feature defaults, CI pools, portability gates | The highest-priority lane serializes dependency additions in its numbered order. Change one graph concern per merge and validate supported feature combinations after each manifest move. |

## Scheduling projection

- Complete the highest-priority RPC and automation lane before scheduling lower-priority work that needs its module, database, manifest, or CLI locks. The Power Automate deployment amendment remains isolated; ACP and MCP serialize in the numbered order even where later adapter scaffolding could technically begin earlier.
- Critical-path coordination plans are not implementation jobs. Schedule a named contract-stabilization or extraction plan rather than assigning the entire roadmap to an agent.
- Independent leaf modules can fan out after their shared contract lands. Module roots, manifests, CLI parsing, generated help, Makefile, and CI remain single-writer integration points.
- The analyzer extraction decision is deferred: preserve the CI-enforced in-tree boundary while analyzer, API, export, and service contracts stabilize, then reconsider extraction only when a concrete alternate consumer exists.
- Remote-create leaf work, OOXML leaf transformations, and a Hugging Face request-type leaf may proceed concurrently when they obey the locks in the [parallel execution guide](parallel-execution.md); they pause for integration windows if they need shared manifests or roots.
- Schema features, GitLab service work, CLI root refactors, analyzer `mod/check/service` refactors, and build/CI wiring serialize within their lanes.
- dPID's portable parser and provider leaf can be prepared independently after its fixture contract lands. Its general RAD identifier/crosswalk, module registration, endpoint configuration, discovery/persistence integration, and CLI import slices serialize through their named locks.
- CKAN/DCAT consumes the implemented [RAD standards field matrix](assets/standards-field-matrix.md), loss-reporting contract, and analyzer composition baseline. New analyzer work must preserve those shared contracts and coordinate ownership of the analyzer roots.

## Status interpretation

- `in-progress` means code evidence shows a partial implementation with named remaining boundaries.
- `planned` means no accepted implementation slice is present, even when supporting primitives already exist.
- Completed plans are deleted and their index/network edges are removed; durable implementation knowledge belongs in code, tests, architecture documentation, or release history.
