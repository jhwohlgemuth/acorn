---
okf_version: "0.1"
---

# Plan dependency network

This map records execution dependencies rather than topical links. An arrow means the left plan establishes a boundary or contract required by the right plan. It was reconciled against the workspace at version `0.1.75` on 2026-08-29.

Dependency order does not by itself prove that plans are safe to run concurrently. The [parallel execution guide](parallel-execution.md) overlays exact write-set rules, single-writer integration locks, work lanes, and verified concurrent batches on this network.

## Critical portability path

```text
dependency feature defaults
          |
          v
isolate std-only modules --> schema I/O validation handoff
          |                           |
          v                           v
host abstraction traits      acorn-core + acorn-schema extraction
          |                           |
          +-------------+-------------+
                        v
       analyzer/API/export boundary stabilization
                        |
                        v
          remaining crate extractions
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
               acorn-lib facade only
                        |
                        v
               no-std/WASM CI gates
```

The highest-value immediate slice is Phase 1.2. Its first commit moves the data-only `License` enum out of std-gated `io` and re-exports it for compatibility, restoring the currently broken minimal, alloc, and WASM checks. The remainder removes path/file ownership from schema modules without changing the legacy `InputOutput` surface. This edge blocks both portable crate extraction and narrower host traits.

## Target ownership network

| Target package | Source boundary to prepare now | Direct prerequisites | Plans unlocked |
|---|---|---|---|
| `acorn-core` | `error.rs`, portable root values, `util/{core,alloc,portable}.rs` | Dependency defaults; std isolation | `acorn-schema`, `acorn-analysis`, `acorn-cmd` |
| `acorn-schema` | `validation/`, `schema/`, string/byte codecs | `acorn-core`; schema I/O handoff | Schema features, standards crosswalks, `acorn-analysis` |
| `acorn-analysis` | `analyzer/{check,error,readability}.rs` over supplied content | Host isolation; analyzer algebra plans | Service extraction and portable checks |
| `acorn-api` | `io/api/` protocol, endpoint, request, response, and auth logic | API roadmap contracts | `acorn-service`, `acorn-host`, provider features |
| `acorn-store` | `io/database/`, GitLab durable operation state | API DTO/error boundaries | GitLab workflows, service extraction |
| `acorn-export` | PowerPoint/OOXML and template transforms that return bytes | Export context; interpolation; OOXML hardening | Native/browser renderer adapters |
| `acorn-host` | Native files, HTTP, processes, downloads, document conversion, doctor | Host traits plus extracted domains | CLI/TUI/Python direct composition |
| `acorn-wasm` | Browser bindings, HTTP/storage adapters, JS DTO conversion | Portable domains and host traits | Consumable WASM artifact |
| `acorn-service` | Discovery, analysis, GitLab bot, MCP/server orchestration | Analysis/API/store/export boundaries | Thin interface crates and facade completion |
| `acorn-cmd` | `util/cmd.rs`, command macros and tests | `acorn-core`; command extraction plan | Removes command runtime from facade defaults |
| `acorn-lib` | Compatibility feature mapping and re-exports only | All extractions | Compatibility release and topological publish |

## Cross-plan dependency edges

### Portability and crate boundaries

- [Dependency feature defaults](refactors/portability/dependency-feature-defaults.md) precedes every extraction because each new manifest needs an explicit capability set.
- [Isolate std-only modules](refactors/portability/isolate-std-only-modules.md) and [schema I/O validation handoff](refactors/core/schema-io-validation-handoff.md) are the immediate prerequisites for `acorn-core` and `acorn-schema`.
- [Host abstraction traits](refactors/portability/host-abstraction-traits.md) follows domain/adapter separation; traits are introduced at actual consumer seams, not speculatively.
- [Crate boundary architecture](refactors/crate-boundary-architecture.md) consumes the analyzer, API, export, command, and portability boundary plans and governs extraction order.
- [WASM feature path](refactors/portability/wasm-feature-path.md) requires portable crates and consumer-owned ports. [no-std and WASM CI](refactors/portability/no-std-wasm-ci.md) begins with current facade checks and expands as crates appear.

### Analyzer and API

- [Check batch monoid](refactors/analyzer/check-batch-monoid.md) reduces duplicated assembly before [standards traversal](refactors/analyzer/standards-traversal.md); validation tree flattening is already complete through the portable ordered `ValidationReport`.
- [Check options context](refactors/analyzer/check-options-context.md) and [link resolution checks](refactors/analyzer/link-resolution-checks.md) establish the injected analysis boundary used by `acorn-analysis` and `acorn-host`.
- [API roadmap](refactors/api/roadmap.md) records completed shared authentication builders and orders the remaining endpoint/parameter, invoke/parse, environment, health, error, and DTO contracts before `acorn-api` extraction.
- [GitLab claimed-operation capabilities](refactors/api/gitlab-claimed-operation-capabilities.md) must stabilize durable settlement before the remaining [GitLab bot workflows](refactors/api/gitlab-webhook-bot-workflows.md) move into `acorn-service`.

### Export and interfaces

- [Interpolation primitives](refactors/core/interpolation-primitives.md) and [export context](refactors/core/export-context.md) precede extraction of pure export transforms.
- [OOXML hardening](refactors/core/ooxml-interpolation-and-slide-copy.md) precedes [OOXML fuzzing](refactors/core/ooxml-fuzzing.md) and `acorn-export` extraction.
- [CLI parse/execution models](refactors/core/cli-parse-execution-models.md), [startup separation](refactors/core/cli-startup-separation.md), and [CLI error boundary](refactors/core/cli-error-boundary.md) prepare the CLI as a composition root after service and adapter extraction.
- [thiserror adoption](migrations/thiserror-adoption.md) is incremental across extracted crates; it must not delay Phase 1.2, but each new crate must start with owned portable errors.

## Scheduling conflicts and shared files

| Shared boundary | Plans that overlap | Coordination rule |
|---|---|---|
| `crates/acorn-lib/src/schema/` | Schema I/O, metadata alignment, changelog, OCP, SWHID, ZON, CKAN/DCAT | Land I/O separation first; feature plans target `acorn-schema` after extraction and must not reintroduce host access. |
| `crates/acorn-lib/src/analyzer/` | Analyzer algebra plans, link resolution, ACP fixes, metadata standards | Stabilize pure check/result types before moving service and host implementations. |
| `crates/acorn-lib/src/io/api/` | API roadmap children, GitLab, SWHID, OCI, inference | Separate provider protocol from workflow orchestration before crate moves; avoid parallel renames of the same modules. |
| `crates/acorn-lib/src/io/powerpoint/` | OOXML hardening, interpolation, export context, fuzzing | Finish structured transforms and tests before moving them to `acorn-export`. |
| `crates/acorn-cli/src/main.rs` and `cli/` | CLI model/startup/error plans, ACP inference | Complete parse/execution separation before adding a large new command surface. |
| Workspace manifests and CI | Crate extraction, feature defaults, Python extraction, CI pools, WASM gates | Change one graph concern per merge and validate all supported feature combinations after each manifest move. |

## Scheduling projection

- Critical-path coordination plans are not implementation jobs. Schedule Phase 1.2 or a named extraction rather than assigning the entire roadmap to an agent.
- Independent leaf modules can fan out after their shared contract lands. Module roots, manifests, CLI parsing, generated help, Makefile, and CI remain single-writer integration points.
- The current pre-extraction batch can combine the Phase 1.2 `License`/schema slice with typed scoring, remote-create leaf work, Python binding deduplication, OOXML leaf transformations, and a Hugging Face request-type leaf, subject to the locks in the [parallel execution guide](parallel-execution.md).
- Schema features, GitLab service work, CLI root refactors, analyzer `mod/check/service` refactors, and build/CI wiring serialize within their lanes.
- The [research activity changelog](features/research-activity-changelog.md) follows the RAD standards field-matrix and loss-reporting contract; its portable leaf types may land separately, but `ResearchActivity`, contexts, renderers, and shared fixtures remain schema-lane integration locks.

## Status interpretation

- `in-progress` means code evidence shows a partial implementation with named remaining boundaries.
- `planned` means no accepted implementation slice is present, even when supporting primitives already exist.
- Completed plans are deleted and their index/network edges are removed; durable implementation knowledge belongs in code, tests, architecture documentation, or release history.
