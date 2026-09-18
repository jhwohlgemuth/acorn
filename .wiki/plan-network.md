---
okf_version: "0.1"
---

# Plan dependency network

This map records execution dependencies rather than topical links. An arrow means the left plan establishes a boundary or contract required by the right plan. It was reconciled against the workspace at version `0.2.0` on 2026-09-13.

Dependency order does not by itself prove that plans are safe to run concurrently. The [parallel execution guide](parallel-execution.md) overlays exact write-set rules, single-writer integration locks, work lanes, and verified concurrent batches on this network.

## Highest-priority RPC and automation path

```text
PowerApps logbook intake and sponsor reporting

             then

Protobuf/gRPC transport
          |
          v
ACP inference and verified fixes
          |
          v
MCP client
```

This is one serialized delivery lane in the displayed scheduling order. PowerApps reporting consumes the completed JSON-RPC operation registry, webhook runtime, GitLab repository automation, canonical RAD persistence, logbook graduation, and updates PDF foundations; it does not make gRPC a functional dependency. After its MVP exit gate releases the shared integration locks, tonic/prost adds a sibling binary transport, ACP maps official SDK sessions to the same services, and the RMCP client adds outbound MCP interoperability without replacing either protocol.

## Critical portability path

```text
dependency feature defaults
          |
          v
isolate std-only modules   host abstraction traits
          |                    |
          +----------+---------+
                     v
   analyzer/API/export boundary stabilization
                     |
                     v
       remaining acorn-* extractions
                     |
          +----------+----------+
          v                     v
    acorn-host       acorn-wasm browser services
          \                     /
           +---------+---------+
                     v
             acorn-service
                     |
                     v
       direct interface composition
                     |
                     v
            no-std/WASM CI gates
```

`acorn-core` and `acorn-schema` are extracted and their domain types are consumed directly. `acorn-lib` has no schema facade: path/file ownership, network model resolution, persistence, contextual RAD formatting, and OpenCode CST preservation remain explicit host extensions. Shared API, runtime, and synchronization composition is owned by `acorn-core::options`; provider, analyzer, and sync-target modules retain only their domain extensions. The alloc-only analyzer kernel is enforced in CI; `acorn-analysis` and generic host traits remain deferred until a concrete alternate consumer proves their need.

## Target ownership network

| Target package | Source boundary to prepare now | Direct prerequisites | Plans unlocked |
|---|---|---|---|
| `acorn-schema` | Complete: schema, validation, string/byte codecs, vocabulary and prompt assets | `acorn-core` | Schema features, standards crosswalks, `acorn-analysis` |
| `acorn-analysis` | `analyzer/{check,error,readability}.rs` over supplied content | Host isolation; analyzer algebra plans | Service extraction and portable checks |
| `acorn-api` | `io/api/` protocol, JSON-RPC, gRPC, webhook verification, endpoint, request, response, and auth logic | API roadmap and operation-registry contracts | `acorn-service`, `acorn-host`, provider features |
| `acorn-store` | `io/database/`, provider-neutral webhook delivery, operation, effect, and callback state | API DTO/error boundaries | GitLab workflows, service extraction |
| `acorn-export` | PowerPoint/OOXML and template transforms that return bytes | Export context; interpolation; OOXML hardening | Native/browser renderer adapters |
| `acorn-host` | Native files, HTTP, processes, downloads, document conversion, doctor | Host traits plus extracted domains | CLI/TUI direct composition |
| `acorn-wasm` | Minimal bindings are complete; browser HTTP/storage adapters remain | Concrete browser workflow and narrow consumer-owned ports | Browser service composition |
| `acorn-service` | Discovery, analysis, GitLab bot, MCP/server orchestration | Analysis/API/store/export boundaries | Thin interface composition roots |
| `acorn-cmd` | `util/cmd.rs`, command macros and tests | `acorn-core`; command extraction plan | Removes command runtime from `acorn-lib` defaults |
| `acorn-lib` | Host implementations awaiting extraction and ACORN-specific host extensions | Owning domains directly | Shrinks at each explicit ownership cutover |

## Cross-plan dependency edges

### Portability and crate boundaries

- [Dependency feature defaults](refactors/portability/dependency-feature-defaults.md) precedes every extraction because each new manifest needs an explicit capability set.
- The [portability roadmap](refactors/portability/roadmap.md) records the completed in-tree analyzer boundary after `acorn-schema` extraction.
- [Host abstraction traits](refactors/portability/host-abstraction-traits.md) is explicitly deferred until an independently compiling kernel and concrete alternate consumers prove each seam.
- [Crate boundary architecture](refactors/crate-boundary-architecture.md) coordinates the analyzer, API, export, command, and portability boundaries; schedule the named extraction plan rather than the umbrella.
- Minimal `acorn-wasm` bindings consume the existing portable schema and analyzer boundaries directly. [WASM browser services](refactors/portability/wasm-feature-path.md) require a concrete workflow and proven consumer-owned ports; [no-std and WASM CI](refactors/portability/no-std-wasm-ci.md) expands with each supported adapter.

### Analyzer and API

- The implemented ordered `CheckBatch` contract (`crates/acorn-lib/src/analyzer/check.rs:279`) and additive `CheckSummary` monoid (`check.rs:313`) plus `ValeConfig` endomorphisms (`host/vale.rs:89`) precede [analyzer composition and traversal](refactors/analyzer/analyzer-composition.md); validation tree flattening is already complete through the portable ordered `ValidationReport`. The single consolidated plan replaces `link-resolution-checks`, `result-boundary-normalization`, and `standards-traversal`.
- The portable RAD standards baseline has landed: the [field matrix](assets/standards-field-matrix.md), crosswalk/loss contract, DataCite 4.7 audit and fixtures, warning-aware `ResearchOutput <-> DataCite Record`, strict `ResearchActivity <-> RAiD Metadata`, and opt-in `ResearchActivity -> DataCite Project` are available to [analyzer composition and traversal](refactors/analyzer/analyzer-composition.md) and [CKAN/DCAT](migrations/ckan-dcat-crosswalk.md).
- The portable RRID baseline has landed: strict parsing, PID integration, local text/PDF/DOCX discovery, generic lifecycle persistence, typed RAD validation, standards fixtures, non-project identity, and the RRID `TypedRelatedResource <-> DataCite RelatedIdentifier` leaf crosswalk all remain independent of provider access.
- The retained [SciCrunch contract review](assets/rrid-scicrunch-contract-matrix.md) records a **not planned** decision after both lookup and search failed closed. SciCrunch work is not an execution dependency or provider-lane job; a future proposal requires new contract evidence and a new plan.
- [dPID research-object support](features/dpid-research-object-support.md) adds a portable root/version/path identifier and RO-Crate profile before a read-only resolver, gather integration, and explicit import. Its general activity-identifier field and crosswalk must preserve the implemented RAD field and loss contracts; provider, discovery, persistence, and CLI roots remain integration locks. Generic DID resolution, nanopublications, minting, and content materialization are outside its boundary.
- [Analyzer composition and traversal](refactors/analyzer/analyzer-composition.md) must consume the implemented calculation/execution/report context split without introducing a generic host trait or presuming package extraction.
- [API roadmap](refactors/api/roadmap.md) records the completed shared authentication builders, `SecretString` credential migration, and reusable webhook contracts, then orders the remaining endpoint/parameter, invoke/parse, environment, health, error, and DTO contracts before `acorn-api` extraction.
- The implemented `io::api::json_rpc` runtime establishes typed service registration and direct JSON-RPC 2.0 request/notification dispatch.
- The implemented `io::api::webhooks` runtime consumes the operation registry and provides durable operations and callbacks, shared outcome timestamps, readiness, and graceful draining.
- GitLab repository automation consumes both implemented foundations and proves registered flat-file workflows against exact repository revisions with deployment-owned policy and durable effect reconciliation.
- [PowerApps logbook intake and sponsor reporting](features/powerapps-gitlab-logbook-reporting.md) reuses those foundations for signed form intake, rolling bucket merge requests, reviewer-controlled graduation, bounded monthly PDF reports, immutable GitLab packages, and signed PowerAutomate callbacks. All PowerAutomate protocol code remains isolated to `io::api::powerautomate`.
- [Protobuf/gRPC transport](refactors/api/protobuf-grpc-transport.md) follows the stable operation contract and adds tonic/prost server and client adapters without encoding protobuf inside JSON-RPC.
- [ACP inference and verified fixes](features/opencode-acp-inference.md) registers inference services while leaving ACP wire ownership with the official SDK; ACP request/session IDs remain distinct from shared operation IDs.
- [MCP client](features/mcp-client.md) follows the stable registry and ACP service contract, using RMCP for outbound MCP sessions rather than translating MCP envelopes into JSON-RPC or gRPC.

### Scraping and Rhai exec

- The implemented [configured Rhai exec extensions](../docs/src/packages/exec.md) establish capability-free local CLI quality gates with typed `ExecutionResult<T>` outputs. Application configuration maps names relative to the project-root `.acorn/` directory to hooks, while an explicit `absolute: true` escape hatch is warned, hardened, and limited to local callers. Host inference recognizes valid extensionless Rhai through ABI-gated restricted compilation without authorizing unlisted scripts.
- [Deterministic gather scraping](features/deterministic-gather-scraping.md) independently adds bounded HTML fetching and typed CSS extraction under existing HTTP/offline policy.
- [Structured extraction inference](features/structured-extraction-inference.md) follows deterministic scraping and adds one explicit typed provider adapter before any generic registry is considered.
- The implemented [Rhai exec package](../docs/src/packages/exec.md) now includes local capability-brokered workflow hooks, explicit operation effects, digest-pinned inline or deployment-file manifests, typed document and file proposals, transactional application, audit records, and local validation commands. Remote transports remain unable to invoke configured scripts.

### Export and interfaces

- [Semantic transformation laws](testing/semantic-transformation-laws.md) consumes the canonical classifications in the [standards field matrix](assets/standards-field-matrix.md) and coordinates its direct-link/workflow commuting test with [CLI test coverage phase 1](testing/cli-coverage-phase-1.md). Normalization, codec, linked-data, and merge laws can start independently against current contracts; any link-command repair serializes on the CLI and export-dispatch locks.
- [Interpolation primitives](refactors/core/interpolation-primitives.md) precede completion of [export context](refactors/core/export-context.md) and parser-backed PowerPoint text/list interpolation. Shared authored-image and draft-placeholder selection has already started the export context independently.
- The structured slide-list, relationship, content-type, and ASPECT transforms in [OOXML hardening](refactors/core/ooxml-interpolation-and-slide-copy.md) are ready for [OOXML fuzzing](refactors/core/ooxml-fuzzing.md). Parser-backed interpolation and the completed export context still precede `acorn-export` extraction.
- [CLI parse/execution models](refactors/core/cli-parse-execution-models.md), [startup separation](refactors/core/cli-startup-separation.md), and [CLI error boundary](refactors/core/cli-error-boundary.md) prepare the CLI as a composition root after service and adapter extraction.
- [Workspace duplication consolidation](refactors/workspace-duplication-consolidation.md) must land its JSONC-policy slice before the corresponding host boundary moves again. Its PID, TUI, API-status, and test-helper slices follow the locks of their owning lanes.
- [thiserror adoption](migrations/thiserror-adoption.md) is incremental across extracted crates; it must not delay portable extraction, but each new crate must start with owned portable errors.
- The implemented portable `acorn-schema::modelkit` contract establishes the KitOps-matched authoring-YAML and resolved-config boundary consumed by [ModelKit and ModelPack OCI interoperability](features/oci-model-artifact-downloads.md). The OCI plan preserves the normalized registry materialization boundary used by the landed Needle sidecar, reuses the safe archive boundary from [unified import and download with BagIt archives](features/unified-import-download-bagit.md), and exposes only its selected GGUF output to inference.

### Database-backed artifacts

- [PowerApps logbook intake and sponsor reporting](features/powerapps-gitlab-logbook-reporting.md) uses merged bucket files for its MVP and establishes bounded reporting-window and artifact-manifest contracts.
- [Database-backed artifact generation](features/database-backed-artifact-generation.md) consumes the implemented canonical database contract and later reuses those reporting contracts; it is not an MVP prerequisite for PowerApps reporting.

## Scheduling conflicts and shared files

| Shared boundary | Plans that overlap | Coordination rule |
|---|---|---|
| `crates/acorn-schema/src/` | Metadata alignment, dPID, OCP, ZON, CKAN/DCAT, SciCrunch resource entity | Schema flattening, the portable RRID baseline, and the ModelKit/Kitfile contract have landed in ACORN. Preserve singular schema ownership and the codec/adapter boundary; feature plans must not reintroduce host access. Plans share `lib.rs`, enum/module roots, validation, standards adapters, and fixtures as integration locks. dPID's general activity identifier and RO-Crate crosswalk serialize with metadata alignment. |
| `crates/acorn-lib/src/analyzer/` | [Analyzer composition](refactors/analyzer/analyzer-composition.md), [dPID](features/dpid-research-object-support.md), ACP fixes, metadata standards | Stabilize pure check/result types before moving service and host implementations. Single analyzer plan owns `analyzer/{check,error,service,host/*}` renames; dPID resolution integration takes the discovery leaf lock for its landing slice. |
| `crates/acorn-lib/src/io/api/` | PowerAutomate, JSON-RPC, implemented webhook runtime, GitLab, gRPC, API roadmap children, dPID, OCI, inference | The highest-priority lane first owns the isolated `powerautomate` provider, GitLab/package integration, shared operation/callback seams, and module root. After its MVP, gRPC owns the transport root. Provider leaves may develop separately only when they do not touch `io/api/mod.rs`, shared errors, operation registration, or transport configuration. |
| `crates/acorn-schema/src/agent/` and `crates/acorn-lib/src/io/{acp,mcp}/` | ACP inference, MCP client, database-backed artifacts, OCI model interoperability, service extraction | Preserve the implemented RMCP server/runtime-policy contracts. ACP uses the official SDK; MCP uses RMCP; both may adapt to shared services but neither owns the JSON-RPC codec. Serialize agent schema, module indexes, manifests, CLI roots, and feature changes. |
| `crates/acorn-lib/src/io/http/mod.rs` | API invocation, transfer/download hardening | Preserve request-level redirect, response-bound, redaction, and anonymous-fallback controls; serialize further service-injection changes. |
| `crates/acorn-lib/src/io/config/` | PowerAutomate project routing, API endpoint/environment, dPID identity persistence, OCI models, BagIt/import | PowerAutomate claims `application.rs` only for its typed deployment configuration and test snapshots. `bucket.rs` owns bucket orchestration and token-backed repository access, `transfer.rs` owns transfer primitives, and `mod.rs` is the shared model/registry and re-export integration root. Claim leaf files exactly and serialize `mod.rs` or shared snapshot changes. |
| `crates/acorn-lib/src/io/database/` | Canonical RAD database, provider-neutral webhook store, database-backed artifact sources | The webhook runtime replaces GitLab-specific bot tables with dedicated generic lifecycle tables. Give each store a leaf module and tests; serialize `database/{mod.rs,schema.rs}`, migrations, and public repository integration. |
| `crates/acorn-lib/src/io/powerpoint/` | OOXML hardening, interpolation, export context, fuzzing | Structured slide/media transforms and focused tests are the baseline. Finish parser-backed interpolation, shared field projection, and fuzzing before moving them to `acorn-export`. |
| `crates/acorn-cli/src/main.rs` and `cli/` | RPC/webhook serving, GitLab bot, ACP inference, MCP client, CLI model/startup/error plans, dPID import | The highest-priority lane has first claim on its CLI integration windows. Preserve implemented MCP commands; register one transport or workflow surface per merge and regenerate help once per integration batch. |
| Workspace manifests and CI | JSON-RPC/webhook dependencies, tonic/prost, ACP SDK, RMCP client, crate extraction, feature defaults, CI pools, WASM gates | The highest-priority lane serializes dependency additions in its numbered order. Change one graph concern per merge and validate supported feature combinations after each manifest move. |

## Scheduling projection

- Complete the highest-priority RPC and automation lane before scheduling lower-priority work that needs its module, database, manifest, or CLI locks. Its four remaining plans serialize in the numbered order even where later adapter scaffolding could technically begin earlier.
- Critical-path coordination plans are not implementation jobs. Schedule a named contract-stabilization or extraction plan rather than assigning the entire roadmap to an agent.
- Independent leaf modules can fan out after their shared contract lands. Module roots, manifests, CLI parsing, generated help, Makefile, and CI remain single-writer integration points.
- The analyzer extraction decision is deferred: preserve the CI-enforced in-tree boundary while analyzer, API, export, and service contracts stabilize, then reconsider extraction only when a concrete alternate consumer exists.
- Remote-create leaf work, OOXML leaf transformations, and a Hugging Face request-type leaf may proceed concurrently when they obey the locks in the [parallel execution guide](parallel-execution.md); they pause for integration windows if they need shared manifests or roots.
- Schema features, GitLab service work, CLI root refactors, analyzer `mod/check/service` refactors, and build/CI wiring serialize within their lanes.
- dPID's portable parser and provider leaf can be prepared independently after its fixture contract lands. Its general RAD identifier/crosswalk, module registration, endpoint configuration, discovery/persistence integration, and CLI import slices serialize through their named locks.
- [OCP schema alignment](features/ocp-infrastructure-schema-alignment.md) consumes the implemented manual `Resource` validator, indexed `OneOrMany<T>` validation, portable validation primitives, and the RAD standards field/loss contract. It keeps `hardware` and `standard::ocp` independent, composes them only in `infrastructure`, and reports the new RAD field as local-only until a target mapping is approved. CKAN/DCAT also consumes the implemented [RAD standards field matrix](assets/standards-field-matrix.md) and loss-reporting contract. The single [analyzer composition](refactors/analyzer/analyzer-composition.md) plan now serializes all `analyzer/` algebra and link work; do not reintroduce leaf plans that overlap its shared files.

## Status interpretation

- `in-progress` means code evidence shows a partial implementation with named remaining boundaries.
- `planned` means no accepted implementation slice is present, even when supporting primitives already exist.
- Completed plans are deleted and their index/network edges are removed; durable implementation knowledge belongs in code, tests, architecture documentation, or release history.
