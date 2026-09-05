---
okf_version: "0.1"
---

# Plan dependency network

This map records execution dependencies rather than topical links. An arrow means the left plan establishes a boundary or contract required by the right plan. It was reconciled against the workspace at version `0.1.76` on 2026-09-01.

Dependency order does not by itself prove that plans are safe to run concurrently. The [parallel execution guide](parallel-execution.md) overlays exact write-set rules, single-writer integration locks, work lanes, and verified concurrent batches on this network.

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
    acorn-host              acorn-wasm
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

`acorn-core` and `acorn-schema` are extracted and their domain types are consumed directly. `acorn-lib` has no schema facade: path/file ownership, network model resolution, persistence, contextual RAD formatting, and OpenCode CST preservation remain explicit host extensions. Active portability work now tightens analyzer document/rendering/service couplings and must produce an alloc-only in-tree kernel before analysis extraction is reconsidered.

## Target ownership network

| Target package | Source boundary to prepare now | Direct prerequisites | Plans unlocked |
|---|---|---|---|
| `acorn-schema` | Complete: schema, validation, string/byte codecs, vocabulary and prompt assets | `acorn-core` | Schema features, standards crosswalks, `acorn-analysis` |
| `acorn-analysis` | `analyzer/{check,error,readability}.rs` over supplied content | Host isolation; analyzer algebra plans | Service extraction and portable checks |
| `acorn-api` | `io/api/` protocol, endpoint, request, response, and auth logic | API roadmap contracts | `acorn-service`, `acorn-host`, provider features |
| `acorn-store` | `io/database/`, GitLab durable operation state | API DTO/error boundaries | GitLab workflows, service extraction |
| `acorn-export` | PowerPoint/OOXML and template transforms that return bytes | Export context; interpolation; OOXML hardening | Native/browser renderer adapters |
| `acorn-host` | Native files, HTTP, processes, downloads, document conversion, doctor | Host traits plus extracted domains | CLI/TUI/Python direct composition |
| `acorn-wasm` | Browser bindings, HTTP/storage adapters, JS DTO conversion | Portable domains and host traits | Consumable WASM artifact |
| `acorn-service` | Discovery, analysis, GitLab bot, MCP/server orchestration | Analysis/API/store/export boundaries | Thin interface composition roots |
| `acorn-cmd` | `util/cmd.rs`, command macros and tests | `acorn-core`; command extraction plan | Removes command runtime from `acorn-lib` defaults |
| `acorn-lib` | Host implementations awaiting extraction and ACORN-specific host extensions | Owning domains directly | Shrinks at each explicit ownership cutover |

## Cross-plan dependency edges

### Portability and crate boundaries

- [Dependency feature defaults](refactors/portability/dependency-feature-defaults.md) precedes every extraction because each new manifest needs an explicit capability set.
- The [portability roadmap](refactors/portability/roadmap.md) records the completed in-tree analyzer boundary after `acorn-schema` extraction.
- [Host abstraction traits](refactors/portability/host-abstraction-traits.md) is explicitly deferred until an independently compiling kernel and concrete alternate consumers prove each seam.
- [Crate boundary architecture](refactors/crate-boundary-architecture.md) coordinates the analyzer, API, export, command, and portability boundaries; schedule the named extraction plan rather than the umbrella.
- [WASM feature path](refactors/portability/wasm-feature-path.md) requires portable crates and consumer-owned ports. [no-std and WASM CI](refactors/portability/no-std-wasm-ci.md) begins with current package checks and expands as crates appear.

### Analyzer and API

- [Check batch monoid](refactors/analyzer/check-batch-monoid.md) reduces duplicated assembly before [standards traversal](refactors/analyzer/standards-traversal.md); validation tree flattening is already complete through the portable ordered `ValidationReport`.
- [RAD metadata standards alignment](features/rad-metadata-standards-alignment.md) provides the portable `acorn-schema` crosswalk/loss contract and DataCite 4.7 type audit consumed by [standards traversal](refactors/analyzer/standards-traversal.md), [CKAN/DCAT](migrations/ckan-dcat-crosswalk.md), and [research activity changelog](features/research-activity-changelog.md); its Slice 0 (loss reporting + 4.7 fixtures) is the gate for those consumers. The changelog's portable leaf types may land after the matrix, but `ResearchActivity`/`ResearchOutput` contexts, renderers, and shared fixtures remain schema-lane integration locks.
- [Link resolution checks](refactors/analyzer/link-resolution-checks.md) must consume the implemented calculation/execution/report context split without introducing a generic host trait or presuming package extraction.
- [API roadmap](refactors/api/roadmap.md) records the completed shared authentication builders, `SecretString` credential migration, and reusable webhook contracts, then orders the remaining endpoint/parameter, invoke/parse, environment, health, error, and DTO contracts before `acorn-api` extraction.
- [GitLab bot workflows](refactors/api/gitlab-webhook-bot-workflows.md) consume the landed opaque claimed-operation settlement and provider-neutral webhook verification/dispatch boundaries before moving orchestration into `acorn-service`.

### Export and interfaces

- [Interpolation primitives](refactors/core/interpolation-primitives.md) and [export context](refactors/core/export-context.md) precede extraction of pure export transforms.
- [OOXML hardening](refactors/core/ooxml-interpolation-and-slide-copy.md) precedes [OOXML fuzzing](refactors/core/ooxml-fuzzing.md) and `acorn-export` extraction.
- [CLI parse/execution models](refactors/core/cli-parse-execution-models.md), [startup separation](refactors/core/cli-startup-separation.md), and [CLI error boundary](refactors/core/cli-error-boundary.md) prepare the CLI as a composition root after service and adapter extraction.
- [Workspace duplication consolidation](refactors/workspace-duplication-consolidation.md) must land its JSONC-policy slice before the corresponding host boundary moves again. Its PID, TUI, API-status, and test-helper slices follow the locks of their owning lanes.
- [thiserror adoption](migrations/thiserror-adoption.md) is incremental across extracted crates; it must not delay portable extraction, but each new crate must start with owned portable errors.
- [Portable ModelKit and Kitfile schema model](features/modelkit-schema-model.md) (KitOps `kitfile.md` + `kitfile.go` at the pinned commit, including `LayerInfo yaml:"-"` codec split and `Validate`/`GetPathType`/`IsModelKitReference` parity with `testdata/validation` + `testdata/parameters` fixtures) establishes the authoring-YAML and resolved-config contract consumed by [ModelKit and ModelPack OCI interoperability](features/oci-model-artifact-downloads.md). The OCI plan preserves the normalized registry materialization boundary used by the landed Needle sidecar, reuses the safe archive boundary from [unified import and download with BagIt archives](features/unified-import-download-bagit.md), and exposes only its selected GGUF output to inference.

## Scheduling conflicts and shared files

| Shared boundary | Plans that overlap | Coordination rule |
|---|---|---|
| `crates/acorn-schema/src/schema/` | Metadata alignment, changelog, OCP, SWHID, ZON, CKAN/DCAT, ModelKit/Kitfile | Preserve singular schema ownership and the codec/adapter boundary; feature plans must not reintroduce host access. ModelKit/Kitfile owns a leaf module (`schema/modelkit`) but shares the `schema/mod.rs` integration lock. |
| `crates/acorn-lib/src/analyzer/` | Analyzer algebra plans, link resolution, ACP fixes, metadata standards | Stabilize pure check/result types before moving service and host implementations. |
| `crates/acorn-lib/src/io/api/` | API roadmap children, GitLab, SWHID, OCI, inference | Preserve the landed authentication and webhook boundaries, separate provider protocol from workflow orchestration before crate moves, and avoid parallel renames of the same modules. |
| `crates/acorn-schema/src/schema/agent/` and `crates/acorn-lib/src/io/mcp/` | ACP inference, OCI model interoperability, service extraction | Preserve the landed provider-neutral tool definition/executor contract; serialize the agent schema contract, module-index, and feature changes. |
| `crates/acorn-lib/src/io/http/mod.rs` | API invocation, transfer/download hardening | Preserve request-level redirect, response-bound, redaction, and anonymous-fallback controls; serialize further service-injection changes. |
| `crates/acorn-lib/src/io/config/` | API endpoint/environment, OCI models, BagIt/import | `application.rs` owns application-file I/O and endpoint lists, `bucket.rs` owns bucket orchestration and token-backed repository access, `transfer.rs` owns transfer primitives, and `mod.rs` is the shared model/registry and re-export integration root. Claim leaf files exactly and serialize `mod.rs` or shared snapshot changes. |
| `crates/acorn-lib/src/io/powerpoint/` | OOXML hardening, interpolation, export context, fuzzing | Finish structured transforms and tests before moving them to `acorn-export`. |
| `crates/acorn-cli/src/main.rs` and `cli/` | CLI model/startup/error plans, ACP inference | Complete parse/execution separation before adding a large new command surface. |
| Workspace manifests and CI | Crate extraction, feature defaults, Python extraction, CI pools, WASM gates | Change one graph concern per merge and validate all supported feature combinations after each manifest move. |

## Scheduling projection

- Critical-path coordination plans are not implementation jobs. Schedule an analyzer std-isolation slice or named extraction rather than assigning the entire roadmap to an agent.
- Independent leaf modules can fan out after their shared contract lands. Module roots, manifests, CLI parsing, generated help, Makefile, and CI remain single-writer integration points.
- The alloc-only analyzer-kernel gate is complete; the next portability decision is whether proven consumer needs justify package extraction or host traits.
- Remote-create leaf work, OOXML leaf transformations, and a Hugging Face request-type leaf may proceed beside the isolation slice when they obey the locks in the [parallel execution guide](parallel-execution.md); they pause for integration windows if they need shared manifests or roots.
- Schema features, GitLab service work, CLI root refactors, analyzer `mod/check/service` refactors, and build/CI wiring serialize within their lanes.
- The [research activity changelog](features/research-activity-changelog.md) and [OCP schema alignment](features/ocp-infrastructure-schema-alignment.md) consume the implemented `not` combinator, `timestamp`/`nonempty` primitives, and consolidated `schema::validate` module. The changelog and CKAN/DCAT now also consume the RAD standards field-matrix and loss-reporting contract from [RAD metadata standards alignment](features/rad-metadata-standards-alignment.md); its Slice 0 must land before changelog's loss table and CKAN's target-contract work. Portable leaf types may land separately, but `ResearchActivity`, contexts, renderers, and shared fixtures remain schema-lane integration locks.

## Status interpretation

- `in-progress` means code evidence shows a partial implementation with named remaining boundaries.
- `planned` means no accepted implementation slice is present, even when supporting primitives already exist.
- Completed plans are deleted and their index/network edges are removed; durable implementation knowledge belongs in code, tests, architecture documentation, or release history.
