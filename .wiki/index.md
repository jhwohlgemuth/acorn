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

The full portfolio was reviewed against the tree on 2026-10-20. On 2026-10-24, the three PA lifecycle plans and their sponsor-content, grouped-report, database-artifact and runner-isolation dependencies were reviewed against `49b4700d` plus current uncommitted code. This was a code/saved-evidence review; live deployment was not revalidated. The manual plan was retired October 25 after its offline handback rehearsal passed; actual deployment baseline approval remains in PA-05. The installable acornd plan was added October 25 from current service, CLI, and release evidence. The ACRE RON plan was added the same day from configuration, CLI, and upstream RON documentation. The portfolio contains 33 executable plans and two coordination records. Remaining work is tracked in the plans below; completed outcomes live in code, tests, and durable documentation.

Ranking uses impact first, then remaining difficulty, with user/architecture value and readiness as tie-breakers. “Quality” rates the plan rather than the proposed feature:

- **A:** current evidence, coherent ownership, explicit sequencing, and verifiable acceptance gates.
- **B:** useful direction, but feasibility, demand, external prerequisites, or the first concrete consumer needs stronger evidence before implementation.

### Ranked executable plans

| Rank | Plan | Status | Impact | Difficulty | Quality | Value | Assessment |
|---:|---|---|---|---|---|---|---|
| 1 | [PowerApp and Power Automate JSONL intake](features/powerapps-jsonl-intake/index.md) | In progress | Critical | Medium | A | Highest | PA-02 closed by owner decision with intermittent GitLab connectivity recorded as a known limitation; Jason-only pilot licensing is documented. PA-15/16/17 remain Waiting on validation for boundaries, concurrency and full restoration. PA-15 field limits and rejections passed in both environments. The 34-run five-attempt series added ORNL count, unsafe-tail and oversized-file acceptance. Each still-open case received five attempts with GitLab 403/504 failures. Mixed-line preservation and exact-cap append remain in both environments; unsafe-tail and oversized rejection remain in Jason. Isolated baselines were restored and completed simulation artifacts removed. Native v6 app/catalog restoration passed in both environments; fresh-save acceptance and ORNL callback-table creation remain unaccepted. PA-12 is later shared deployment. Preserve operational components and use isolated test copies. |
| 2 | [Shared export and OOXML pipeline](refactors/core/export-pipeline-hardening.md) | In progress | Critical | High | A | Highest | Removes silent/stale template output and partial PPTX mutation while unlocking `acorn-export`. |
| 3 | [Outbound MCP client embedder interface](refactors/api/outbound-mcp-client-interface.md) | In progress | High | Medium | A | High | Client code and focused tests are complete; separately owned book integration remains before plan retirement. |
| 4 | [Needle 3 release-host validation](migrations/needle-3-upgrade.md) | Waiting on validation | High | Medium | A | High | Linux x86_64 passed on Oklahoma; Linux ARM64 can validate on Dante. The Windows job waits for native shell runner readiness. |
| 5 | [GitLab runner-pool isolation](refactors/ci-runner-pools.md) | In progress | High | Medium | A− | High | Directly addresses observed checkout collisions and protects reporting credentials, but requires administrator evidence. |
| 6 | [GitLab shell runner readiness](testing/gitlab-shell-runner-readiness.md) | Waiting on validation | High | Medium | A− | High | Local shell registration is tested and documented; a native Windows host is needed for the tagged job that unblocks three platform test phases. |
| 7 | [Database-backed artifact generation](features/database-backed-artifact-generation.md) | Planned | High | Medium | A− | High | Reuses mature renderers and revisions to provide reproducible current/historical artifacts. |
| 8 | [Automated JSONL management with the Vergil GitLab bot](features/vergil-gitlab-jsonl-automation/index.md) | Planned | High | High | A | High | PA-05/06/07/09 and BOT-02 are Waiting on validation, subject to prerequisites; deployment/integration phases remain planned/blocked. Runtime contracts exist; live Vergil deployment, authenticated callbacks, source handback and review/report acceptance remain. One-shot admission can create a draft MR. Outside the manual demo critical path. |
| 9 | [Sponsor highlights clarity](features/sponsor-highlights-clarity.md) | Planned | High | Mixed | B+ | High | Stakeholder feedback supports an editorial pilot; reviewed ASPECT graduation still needs typed evidence and target-specific schema, local CLI and database/MCP/GitLab integration. |
| 10 | [Structured research-activity enrichment](refactors/core/research-activity-enrichment-integration.md) | Planned | High | Medium | A− | High | Converts already available provider evidence into typed fields with deterministic conflict handling. |
| 11 | [Podman Makefile migration](migrations/podman-makefile.md) | Planned | High | Medium | A− | Medium | Technically well specified, but the broad cross-platform matrix is costly for a development-tooling-only change. |
| 12 | [CEL metadata validation-profile pilot](features/cel-validation-profile-pilot.md) | Planned | High | Medium | B+ | Medium | Strong stop gates and bounded scope, but a named profile owner and enforceable runtime budgets must be proven before adoption. |
| 13 | [Shared API client contract](refactors/api/shared-client-contract.md) | In progress | High | High | A− | Highest | Consolidates the prerequisite mechanics for `acorn-api`; execute in its named contract slices, not as one rewrite. |
| 14 | [CLI composition boundary](refactors/core/cli-composition-boundary.md) | Planned | High | High | A− | High | Makes startup, normalized requests, remote dispatch, and exit behavior testable in one ownership boundary. |
| 15 | [Model acquisition and metadata hardening](features/model-acquisition-hardening.md) | In progress | High | High | A | High | Unifies three overlapping model plans and closes identity, metadata, package, and registry evidence gaps. |
| 16 | [Remote container creation hardening](refactors/core/remote-create-container-targets.md) | In progress | High | High | A | High | Adds preflight and compensating rollback around real external resources; live-host evidence remains necessary. |
| 17 | [Portable diff resource and API hardening](refactors/acorn-diff-resource-and-api-hardening.md) | Planned | High | High | A | High | Adds deterministic resource and output safety to untrusted diff paths while completing a small public API. |
| 18 | [dPID research-object support](features/dpid-research-object-support.md) | Planned | High | High | A | Medium | Exceptionally complete trust and provenance design, but upstream volatility and uncertain demand reduce near-term value. |
| 19 | [Deterministic and assisted gather extraction](features/gather-extraction.md) | Planned | High | High | A− | High | One ordered plan now preserves the deterministic-first security and provenance boundary across CLI and TUI. |
| 20 | [Azure Cosmos DB bucket downloads](features/azure-cosmos-buckets.md) | Planned | High | High | B+ | Medium | Fits the landed source abstraction, but a preview SDK and no recorded deployment demand warrant validation before build-out. |
| 21 | [Interactive metadata chat and terminal prompts](features/interactive-metadata-chat.md) | Planned | High | High | A− | High | Confirm visuals and selectable enrichment candidates have landed; routed chat and interactive format work remain. |
| 22 | [Needle routing and native capability parity](features/needle-router-native-parity.md) | In progress | High | High | A− | High | Establish caller-owned Needle routing and measured native registry parity. |
| 23 | [Generic ask executor adapters](features/ask-executor-adapters.md) | Planned | High | High | A− | High | Build configured MCP, ACP, endpoint, and local sidecar bindings after router contracts land; release waits for native parity. |
| 24 | [Installable acornd background service](features/acornd-service.md) | Planned | High | High | B+ | High | Reuses landed service and CLI operations; service registration, signing, and live host evidence gate release. |
| 25 | [Swift SLM capability profile](features/swift-slm-profile.md) | Planned | High | High | B | Medium | Licensing, host, artifact, and quality gates precede a three-capability release. |
| 26 | [Julia decision sidecar profile](features/julia-decision-sidecar.md) | Planned | High | High | B | Medium | Independent parity, quality, and resource gates precede an advisory decision release. |
| 27 | [Grouped report separators](features/grouped-report-separators.md) | Planned | Medium | Medium | A | High | Extend the existing local --combine renderer with explicit group order and office divider pages; retain the current output API. |
| 28 | [RON support for ACRE configuration](features/acre-ron-configuration.md) | Planned | Medium | Medium | A− | High | Adds the requested configuration format through a map-based adapter that preserves ACRE semantics; staged codec, discovery, CLI, and formatting gates protect existing formats. |
| 29 | [ZON codec hardening](refactors/zon-codec-hardening.md) | Planned | Medium | Medium | A | Medium | A contained correctness/maintainability leaf with strong tests and low integration risk. |
| 30 | [CLI coverage](testing/cli-coverage.md) | Planned | Medium | Medium | A− | Medium | Targets known command and binary failure gaps without reviving obsolete test layout or chasing raw line coverage. |
| 31 | [Portable PID verification](testing/pid-verification.md) | Planned | Medium | High | B+ | Medium | Consolidation and Miri are practical; full Creusot/solver maintenance should proceed only after toolchain feasibility is proven. |
| 32 | [Beam inference integration](features/beam-inference-integration.md) | Planned | Medium | High | B | Medium | Release and serving evidence gate an explicit endpoint integration and ACORN evaluation. |
| 33 | [Coverage throughput](testing/coverage-throughput.md) | Planned | Medium | Medium | A− | Medium | Developer-workflow only; the forced clean stage is the confirmed cost, but every speed claim still needs a measured baseline. |

### Coordination record

| Record | Quality | Impact | Value | Assessment |
|---|---|---|---|---|
| [Crate boundary architecture](refactors/crate-boundary-architecture.md) | A | Critical | Highest | Coordinates the remaining package map; existing child plans prepare API, export, and CLI contracts, while each future extraction needs its own atomic plan. |
| [Ask routing and sidecar coordination](features/slm-sidecars.md) | A− | High | High | Orders native routing, generic adapters, and independently gated Swift and Julia releases. |

## Planning rules

### Adding a plan

1. Create one semantically named Markdown file for one atomic outcome.
2. Add `type: Plan`, `title`, `description`, `difficulty`, `impact`, a non-empty `package` list, `tags`, and `status` to YAML frontmatter.
3. Add `execution: phased` and `phase_budget_hours: 5`. Use `execution: coordination` without phases or a phase budget only for a portfolio coordination record.
4. Use `low`, `medium`, `high`, `mixed`, or `not-assessed` for difficulty; use `low`, `medium`, `high`, `critical`, `mixed`, or `not-assessed` for impact.
5. Use `planned`, `in-progress`, or `waiting-on-validation` for status. Reserve `waiting-on-validation` for implemented work whose only remaining acceptance gates are tests, deployment checks, or external validation. Add `priority: next` only when the ranked portfolio and dependency network agree.
6. Document goal, current evidence, exact project boundaries, dependencies, decisions, validation, acceptance criteria, and at least one remaining five-hour execution phase. Cite external claims.
7. Link the concept from the nearest directory index, the dependency network, and one parallel-execution lane.

### Maintaining plans

1. Verify evidence and status against current code before editing.
2. Use phase status `waiting-on-validation` when implementation exists and only acceptance evidence remains. Keep unmet prerequisites and external blockers explicit; this status does not imply readiness to execute. Keep only remaining work. After completion, remove the completed section, phase, or task; do not retain checked boxes or `done` phase records. Preserve stable IDs for unfinished phases.
3. Update frontmatter, boundaries, dependencies, validation, acceptance criteria, remaining phases, and handoffs together.
4. Delete finished plans and remove every index, network, and lane entry; durable knowledge belongs in code, tests, architecture documentation, or release history.
5. Consolidate overlapping proposals into one canonical concept. Do not preserve an umbrella plan after its work has moved to owners.
6. Every phase must fit five active hours, name one outcome, use an exact write set or named external systems, claim at most one primary integration lock, and include focused validation plus a mergeable handoff.
7. Treat external waiting as a prerequisite. Create a new bounded repair phase for an unexpected defect instead of expanding the active phase.
8. Run `python3 .agents/skills/planning/scripts/validate_wiki.py` after changes, then check internal links, legacy references, whitespace, and plan counts.
