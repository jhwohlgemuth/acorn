---
okf_version: "0.1"
---

# ACORN Planning Wiki

This directory is the authoritative home for ACORN implementation plans. It is an [Open Knowledge Format](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md) bundle.

## Contents

- [Plan dependency network](plan-network.md) - Current contract order and cross-plan blockers.
- [Parallel execution guide](parallel-execution.md) - Work lanes, exact-write-set rules, and single-writer integration locks.
- [Feature plans](features/) - New user-facing capabilities and integrations.
- [Refactor plans](refactors/) - Reliability, maintainability, extensibility, and portability work.
- [Migration plans](migrations/) - Changes that replace an existing dependency, architecture, or workflow.
- [Testing plans](testing/) - Targeted CLI and portable verification work.

## Portfolio review

Reviewed against the current tree on 2026-10-06. The portfolio now contains 22 executable plans and one coordination record. The completed MCP client and dependency-capability plans were removed after their implementations, focused tests, feature and portability checks, Darwin size measurements, and durable documentation landed. The embedder-interface plan covers the remaining direct-construction and external-cancellation contracts required by the first downstream workflow consumer.

Ranking uses impact first, then remaining difficulty, with user/architecture value and readiness as tie-breakers. “Quality” rates the plan rather than the proposed feature:

- **A:** current evidence, coherent ownership, explicit sequencing, and verifiable acceptance gates.
- **B:** useful direction, but feasibility, demand, external prerequisites, or the first concrete consumer needs stronger evidence before implementation.

### Ranked executable plans

| Rank | Plan | Status | Impact | Difficulty | Quality | Value | Assessment |
|---:|---|---|---|---|---|---|---|
| 1 | [Power Apps deployment validation](features/powerapps-gitlab-logbook-reporting.md) | Waiting on validation | Critical | Medium | A | Highest | One deployed, duplicate-safe end-to-end smoke test closes an otherwise implemented critical workflow. |
| 2 | [Shared export and OOXML pipeline](refactors/core/export-pipeline-hardening.md) | In progress | Critical | High | A | Highest | Removes silent/stale template output and partial PPTX mutation while unlocking `acorn-export`. |
| 3 | [Portable analysis and service boundary](refactors/core/acorn-service-boundary.md) | Planned | Critical | High | A | Highest | Creates the first portable service slice and one dual-protocol daemon while preserving JSON-RPC behavior. |
| 4 | [Outbound MCP client embedder interface](refactors/api/outbound-mcp-client-interface.md) | Planned | High | Medium | A | High | Adds direct server-map construction and bounded external cancellation over the narrow MCP graph, unblocking Xylem without changing MCP policy or transports. |
| 5 | [Needle 3 release-host validation](migrations/needle-3-upgrade.md) | Waiting on validation | High | Medium | A | High | A narrow four-host gate closes a shipped migration with little design uncertainty. |
| 6 | [GitLab runner-pool isolation](refactors/ci-runner-pools.md) | In progress | High | Medium | A− | High | Directly addresses observed checkout collisions and protects reporting credentials, but requires administrator evidence. |
| 7 | [Database-backed artifact generation](features/database-backed-artifact-generation.md) | Planned | High | Medium | A− | High | Reuses mature renderers and revisions to provide reproducible current/historical artifacts. |
| 8 | [Structured research-activity enrichment](refactors/core/research-activity-enrichment-integration.md) | Planned | High | Medium | A− | High | Converts already available provider evidence into typed fields with deterministic conflict handling. |
| 9 | [Podman Makefile migration](migrations/podman-makefile.md) | Planned | High | Medium | A− | Medium | Technically well specified, but the broad cross-platform matrix is costly for a development-tooling-only change. |
| 10 | [CEL metadata validation-profile pilot](features/cel-validation-profile-pilot.md) | Planned | High | Medium | B+ | Medium | Strong stop gates and bounded scope, but a named profile owner and enforceable runtime budgets must be proven before adoption. |
| 11 | [Shared API client contract](refactors/api/shared-client-contract.md) | In progress | High | High | A− | Highest | Consolidates the prerequisite mechanics for `acorn-api`; execute in its named contract slices, not as one rewrite. |
| 12 | [CLI composition boundary](refactors/core/cli-composition-boundary.md) | Planned | High | High | A− | High | Makes startup, normalized requests, remote dispatch, and exit behavior testable in one ownership boundary. |
| 13 | [Model acquisition and metadata hardening](features/model-acquisition-hardening.md) | In progress | High | High | A | High | Unifies three overlapping model plans and closes identity, metadata, package, and registry evidence gaps. |
| 14 | [Remote container creation hardening](refactors/core/remote-create-container-targets.md) | In progress | High | High | A | High | Adds preflight and compensating rollback around real external resources; live-host evidence remains necessary. |
| 15 | [Portable diff resource and API hardening](refactors/acorn-diff-resource-and-api-hardening.md) | Planned | High | High | A | High | Adds deterministic resource and output safety to untrusted diff paths while completing a small public API. |
| 16 | [dPID research-object support](features/dpid-research-object-support.md) | Planned | High | High | A | Medium | Exceptionally complete trust and provenance design, but upstream volatility and uncertain demand reduce near-term value. |
| 17 | [Deterministic and assisted gather extraction](features/gather-extraction.md) | Planned | High | High | A− | High | One ordered plan now preserves the deterministic-first security and provenance boundary across CLI and TUI. |
| 18 | [Azure Cosmos DB bucket downloads](features/azure-cosmos-buckets.md) | Planned | High | High | B+ | Medium | Fits the landed source abstraction, but a preview SDK and no recorded deployment demand warrant validation before build-out. |
| 19 | [SLM sidecars](features/slm-sidecars.md) | In progress | High | High | B | Medium | The blocked macOS runner, licensing approval, and upstream runner ownership remain hard prerequisites. |
| 20 | [ZON codec hardening](refactors/zon-codec-hardening.md) | Planned | Medium | Medium | A | Medium | A contained correctness/maintainability leaf with strong tests and low integration risk. |
| 21 | [CLI coverage](testing/cli-coverage.md) | Planned | Medium | Medium | A− | Medium | Targets known command and binary failure gaps without reviving obsolete test layout or chasing raw line coverage. |
| 22 | [Portable PID verification](testing/pid-verification.md) | Planned | Medium | High | B+ | Medium | Consolidation and Miri are practical; full Creusot/solver maintenance should proceed only after toolchain feasibility is proven. |

### Coordination record

| Record | Quality | Impact | Value | Assessment |
|---|---|---|---|---|
| [Crate boundary architecture](refactors/crate-boundary-architecture.md) | A | Critical | Highest | The single package-ownership roadmap; schedule its ranked child plans or a named extraction slice, never the document as a whole. |

## Consolidation decisions

| Canonical plan or record | Consolidated former plans |
|---|---|
| [Shared API client contract](refactors/api/shared-client-contract.md) | API roadmap; endpoint resolution; request composition; invoke/parse; environment bootstrap; health helpers; error envelopes; response DTOs; OpenAPI validation; RAiD refinement |
| [CLI composition boundary](refactors/core/cli-composition-boundary.md) | Parse/execution models; startup separation; CLI error boundary; remote-serve duplication |
| [Shared export and OOXML pipeline](refactors/core/export-pipeline-hardening.md) | Interpolation primitives; export context; OOXML hardening; OOXML fuzzing |
| [Deterministic and assisted gather extraction](features/gather-extraction.md) | Deterministic scraping; structured extraction inference; TUI gather duplication |
| [Model acquisition and metadata hardening](features/model-acquisition-hardening.md) | Hugging Face request readiness; model context metadata; ModelKit/ModelPack OCI interoperability; model-sync option duplication |
| [Portable PID verification](testing/pid-verification.md) | PID parse/percent-validation duplication; Miri quality; Creusot checksum verification |
| [CLI coverage](testing/cli-coverage.md) | CLI coverage initiative; phase 1; repeated CLI log-capture support |
| [Crate boundary architecture](refactors/crate-boundary-architecture.md) | Refactor initiative; portability roadmap; incremental typed-error policy |

The former workspace-duplication umbrella was removed. Its valuable work moved to the owning plans above; the proposed status-bar abstraction was dropped because it had low standalone impact and no demonstrated user or architectural outcome. The separate `thiserror` migration was also removed: the dependency and several owned errors already exist, so conversions now occur only inside the plan that owns each error contract.

## Planning rules

### Adding a plan

1. Create one semantically named Markdown file for one atomic outcome.
2. Add `type: Plan`, `title`, `description`, `difficulty`, `impact`, a non-empty `package` list, `tags`, and `status` to YAML frontmatter.
3. Use `low`, `medium`, `high`, `mixed`, or `not-assessed` for difficulty; use `low`, `medium`, `high`, `critical`, `mixed`, or `not-assessed` for impact.
4. Use `planned`, `in-progress`, or `waiting-on-validation` for status. Reserve `waiting-on-validation` for implemented work whose only remaining acceptance gates are tests, deployment checks, or external validation. Add `priority: next` only when the ranked portfolio and dependency network agree.
5. Document goal, current evidence, exact project boundaries, dependencies, decisions, validation, and acceptance criteria. Cite external claims.
6. Link the concept from the nearest directory index, the dependency network, and one parallel-execution lane.

### Maintaining plans

1. Verify evidence and status against current code before editing.
2. Update frontmatter, boundaries, dependencies, validation, and acceptance criteria together.
3. Delete finished plans and remove every index, network, and lane entry; durable knowledge belongs in code, tests, architecture documentation, or release history.
4. Consolidate overlapping proposals into one canonical concept. Do not preserve an umbrella plan after its work has moved to owners.
5. Before implementation, replace directory/glob boundaries with an exact write set and name each integration lock.
6. After changes, check required frontmatter, internal links, legacy references, whitespace, and plan counts.
