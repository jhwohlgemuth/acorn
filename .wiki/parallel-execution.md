---
okf_version: "0.1"
---

# Parallel execution guide

This guide turns the dependency network into a scheduling contract for concurrent agents. It was reconciled against the active plan boundaries and workspace layout at version `0.1.76` on 2026-09-01.

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
| Module indexes | Any shared `mod.rs`, especially `schema/mod.rs`, `io/mod.rs`, `io/api/mod.rs`, `analyzer/mod.rs` | Parallel leaf modules need imports/re-exports | Leaf agents do not edit the index; one integration owner performs module wiring after leaf changes land. |
| HTTP transport | `crates/acorn-lib/src/io/http/mod.rs` | API invocation, webhook redaction/dispatch, transfer retries, host extraction | One transport owner per batch; land logging, redaction, retry, or service-injection changes before dependent provider migrations. |
| CLI contract | `crates/acorn-cli/src/cli/`, `main.rs`, generated help | New commands, parse/execution split, startup/error work | One CLI-contract owner; command leaf modules can proceed separately against stable arguments. |
| Export dispatch | `crates/acorn-cli/src/commands/export/mod.rs`, `template/mod.rs` | Schema codecs, ZON, export context, PowerPoint | Renderer leaf work may proceed; dispatch/context integration serializes. |
| Test/fixture registry | Shared snapshots and the same fixture directory | Independent plans overwrite snapshots or reinterpret fixtures | Give each slice dedicated fixtures; one owner accepts shared snapshot changes. |
| CI/build entry points | `.gitlab-ci.yml`, `Makefile` | Runner pools, portable gates, Miri, Creusot, fuzz | Tool-specific scripts may be separate; one CI/build integration owner wires jobs and targets. |
| Documentation generation | `CommandLineHelp.md`, shared book indexes | Multiple CLI features regenerate the same output | Regenerate once after the batch, owned by the CLI integration slice. |
| Application configuration | `crates/acorn-lib/src/io/config/mod.rs` and shared config snapshots | API endpoint/environment work, OCI model profiles, and bucket transfer changes need shared re-exports or snapshots | Treat `config/mod.rs` and shared snapshots as integration-owned. Leaf work must claim `application.rs`, `bucket.rs`, or `transfer.rs` explicitly; `bucket.rs` changes serialize between API environment and transfer/archive slices. |

## Coordination-only plans

These plans describe architecture and ordering. Do not assign an agent to implement one as a whole:

- [Refactor initiative](refactors/initiative.md)
- [Crate boundary architecture](refactors/crate-boundary-architecture.md)
- [Portability roadmap](refactors/portability/roadmap.md)
- [API roadmap](refactors/api/roadmap.md)

Schedule their atomic child plans or a named migration phase with an exact write set. The coordinator owns shared integration locks and updates the roadmap after child slices land.

## Work lanes

Plans within one lane generally serialize unless the notes define leaf-module sharding. Plans in different lanes may run concurrently after checking secondary locks.

| Lane | Plans | Primary write surface | Parallel rule |
|---|---|---|---|
| Portable domain boundary | [portability roadmap](refactors/portability/roadmap.md) | `acorn-lib` analyzer roots and feature checks | The in-tree analyzer kernel is green; package-graph work still requires an explicit extraction decision. |
| Schema features | [OCP](features/ocp-infrastructure-schema-alignment.md), [RAD standards](features/rad-metadata-standards-alignment.md), [changelog](features/research-activity-changelog.md), [ZON](features/zon-format-support.md), [CKAN/DCAT](migrations/ckan-dcat-crosswalk.md), [lifecycle/credit](refactors/core/research-activity-lifecycle-credit-fields.md), [ModelKit and Kitfile](features/modelkit-schema-model.md) | `crates/acorn-schema/src/schema/research_activity`, `crates/acorn-schema/src/schema/standard`, `crates/acorn-schema/src/schema/modelkit`, `crates/acorn-schema/src/validation`, `crates/acorn-macros`, crosswalks, `tests/fixtures/schema/`, `tests/fixtures/modelkit/` | The portable validation-rule and derive-macro contract is implemented, and whole-value validators now use the report form; OCP, changelog, and ModelKit/Kitfile consume it. RAD standards Slice 0 (crosswalk loss + DataCite 4.7 audit) is the gate for changelog's loss table and CKAN's target-contract work. ModelKit/Kitfile can own its leaf and dedicated fixtures independently but must serialize `schema/mod.rs`, shared validation changes, `util/constants/app.rs` media-type additions (`mcpb`), and its final `io/oci.rs` migration. Serialize changes to ResearchActivity/output types, contexts, renderers, crosswalk registry, validation, and serialized fixtures. |
| PID/SWHID | [SWHID](features/swhid-persistent-identifier.md), [Creusot](testing/creusot-pid-checksums.md), [Miri](testing/miri-quality.md) | `schema/pid`, SWHID host/API modules, verification wiring | Proof/test work can run beside unrelated lanes. SWHID and proof agents must not edit the same PID leaf; Makefile/CI wiring uses the integration lock. |
| Analyzer | [Applicative validation](refactors/analyzer/applicative-validation.md), [check batch](refactors/analyzer/check-batch-monoid.md), [link checks](refactors/analyzer/link-resolution-checks.md), [result boundaries](refactors/analyzer/result-boundary-normalization.md), [standards traversal](refactors/analyzer/standards-traversal.md), [summary](refactors/analyzer/summary-monoid.md), [Vale normalization](refactors/analyzer/vale-config-normalization.md) | `analyzer/` | A Vale leaf can run independently. Plans touching `mod.rs`, `check.rs`, or `host/service.rs` serialize in the documented dependency order. |
| API shared contract | [Endpoint resolution](refactors/api/endpoint-resolution.md), [request parameters](refactors/api/request-parameter-composition.md), [invoke/parse](refactors/api/invoke-and-parse-wrappers.md), [environment](refactors/api/environment-bootstrap.md), [health](refactors/api/health-check-helpers.md), [errors](refactors/api/api-error-envelopes.md), [DTOs](refactors/api/response-dtos.md), [OpenAPI validation](refactors/api/openapi-endpoint-validation.md), [OpenAPI codegen](refactors/api/openapi-module-codegen.md), [capability matrix](refactors/api/provider-capability-matrix.md) | `io/api/mod.rs`, provider modules, `io/config/{application,bucket,mod}.rs` | Shared `io/api/mod.rs` and `io/config/mod.rs` changes serialize. Preserve the landed `SecretString` authentication contract. Endpoint work owns `config/application.rs`; environment work must claim `application.rs` or `bucket.rs` explicitly. Provider leaf migrations may run concurrently when each owns a different provider file. Codegen waits for the extracted layout. |
| Provider leaves | [Hugging Face readiness](refactors/api/huggingface-request-readiness.md), [RAiD refinement](refactors/api/raid-api-refinement.md), [OCI models](features/oci-model-artifact-downloads.md) | `io/api/{huggingface,raid,harbor}.rs`, `io/oci.rs`, `io/config/mod.rs`, distinct CLI planner files | Safe to shard by provider unless the shared API root, model/registry configuration in `config/mod.rs`, download planner, or manifests change. |
| Agent tools and sidecars | [ACP inference](features/opencode-acp-inference.md) | `schema/agent`, `io/{mcp,sidecar}`, inference API leaves, export/serve CLI composition | Preserve the landed Needle tool contract, MCP adapter, and sidecar boundary. Serialize schema/API indexes, manifests, CLI roots, generated help, and OCI integration with their lock owners. |
| GitLab service/store | [Bot workflows](refactors/api/gitlab-webhook-bot-workflows.md) | `io/api/webhooks/`, `io/http/mod.rs`, `io/api/gitlab/`, analyzer service, bot commands | Build on the landed opaque-claim settlement and reusable webhook contracts. Preserve those boundaries while remaining workflow migration proceeds; GitLab review additions from ZON also serialize on the touched review file. |
| Export/OOXML | [Interpolation](refactors/core/interpolation-primitives.md), [OOXML hardening](refactors/core/ooxml-interpolation-and-slide-copy.md), [OOXML fuzzing](refactors/core/ooxml-fuzzing.md), [export context](refactors/core/export-context.md), [safe path](refactors/core/safe-path.md) | `util`, `io/powerpoint`, CLI export/template, fuzz | Follow interpolation -> hardening -> fuzzing. A pure safe-path leaf may run separately, but PowerPoint/export integration and snapshots serialize. |
| Transfer/archive | [BagIt/import](features/unified-import-download-bagit.md), [OCI models](features/oci-model-artifact-downloads.md), [safe path](refactors/core/safe-path.md) | `io/{archive,bagit,download,source,oci}.rs`, `io/config/{bucket,transfer}.rs`, import/download/export commands | Library leaf work can shard by module. `config/bucket.rs` is shared with API environment work; CLI download/export integration, path policy, `config/mod.rs`, shared snapshots, and manifests are locks. |
| CLI composition | [Parse/execution](refactors/core/cli-parse-execution-models.md), [startup](refactors/core/cli-startup-separation.md), [error boundary](refactors/core/cli-error-boundary.md), [remote create](refactors/core/remote-create-container-targets.md), [CLI coverage](testing/cli-coverage-phase-1.md), [ACP inference](features/opencode-acp-inference.md) | CLI root/parse plus command leaf modules | Root/parse/error plans serialize. Remote-create and command-local tests can run concurrently if they do not touch shared arguments/help. ACP waits for the composition contracts. |
| Python/repository | [acorn-py extraction](migrations/acorn-py-repository-extraction.md) | `crates/acorn-py`, workspace/CI during cutover | Binding deduplication is complete. Repository extraction owns the remaining Python workspace, manifest, and CI cutover. |
| Crate/host adapters | [acorn-cmd](refactors/acorn-cmd-extraction.md), [host traits](refactors/portability/host-abstraction-traits.md), [WASM](refactors/portability/wasm-feature-path.md), [thiserror](migrations/thiserror-adoption.md) | New crates, ownership roots, manifests | Host traits and analysis extraction wait for a green in-tree portable kernel and proven alternate consumers. Apply thiserror within the owning slice, not as a workspace-wide concurrent rewrite. |
| Duplication consolidation | [Workspace duplication consolidation](refactors/workspace-duplication-consolidation.md) | Schema PID leaves, core JSONC policy, CLI model synchronization, TUI widgets, provider leaves, test utilities | Treat each recommendation as a separate slice in its owning lane. TUI and distinct provider/test leaves may run independently after exact write sets are declared. |
| Build/CI/performance | [Portable CI](refactors/portability/no-std-wasm-ci.md), [runner pools](refactors/ci-runner-pools.md), [macOS size](refactors/portability/macos-compact-unwind-size.md), [Miri](testing/miri-quality.md), [Creusot](testing/creusot-pid-checksums.md), [OOXML fuzzing](refactors/core/ooxml-fuzzing.md) | manifests, Makefile, GitLab CI | Measurement/tool scripts may be prepared independently; CI and Makefile integration serialize through one owner. |

## Concurrent batch before the next extraction

The following slices have disjoint primary write sets on the current tree. They can run at the same time if each obeys the named lock restriction:

| Slice | Exact primary ownership | Lock restriction |
|---|---|---|
| Remote-create hardening leaf | `acorn-cli/src/commands/create/` and dedicated tests | Do not edit shared CLI arguments/help; hand required argument changes to the CLI lock owner. |
| OOXML structured mutation leaf | `io/powerpoint/{mod,ooxml}.rs`, PowerPoint-specific command/test files | Do not edit export dispatch/templates or shared snapshots owned by another slice. |
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
