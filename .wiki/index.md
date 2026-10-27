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

The October 26 cleanup against `55484978` plus the current working tree confirms **35 executable plans and two coordination records**. Metadata, phase statuses, asset references and retained archives were checked. No additional whole plan has sufficient evidence for retirement; the manual-ingestion plan remains retired, with its guides and demo fixtures retained. Earlier code reviews remain dated evidence, not a new full-workspace test. See the [cleanup record](features/powerapps-jsonl-intake/test-artifact-cleanup.md#october-26-post-validation-wiki-cleanup).

The preceding live validation accepted PA-19 storage/catalog, PA-20 intake/export and Jason's PA-15 unsafe-tail rejection. Completed phases are pruned from the active form plan; durable proofs remain linked. PA-21 browser acceptance, PA-22 standard-user access, PA-26 exporter repair followed by PA-23 aggregate validation, PA-24 restoration and optional deployment gates remain. Both operational routes were preserved. Browser download and local reporting remain available independently of direct OAuth download and the Vergil bot.

The [SharePoint JSONL download plan](features/sharepoint-jsonl-download/index.md) owns reusable OAuth 2.0/Entra authentication, Graph byte capture and the intake handoff. The [headless CI follow-on](features/sharepoint-headless-ci-download/index.md) requires SPD-26 shared-functionality acceptance plus any one native OS validation; the other OS validations and SPD-11/12 do not gate it. Tenant authentication, GitLab federation, selected application permissions and live download acceptance remain unverified. Neither plan changes the operational pilots.

Current non-PA follow-ups remain substantive: outbound MCP embedder documentation is still missing, release-host validation still needs native hosts, and coverage targets still force clean builds. Existing uncommitted source work was preserved; no plan was closed merely because implementation appeared present.

Ranking uses impact first, then remaining difficulty, with user/architecture value and readiness as tie-breakers. “Quality” rates the plan rather than the proposed feature:

- **A:** current evidence, coherent ownership, explicit sequencing, and verifiable acceptance gates.
- **B:** useful direction, but feasibility, demand, external prerequisites, or the first concrete consumer needs stronger evidence before implementation.

### Ranked executable plans

| Rank | Plan | Status | Impact | Difficulty | Quality | Value | Assessment |
|---:|---|---|---|---|---|---|---|
| 1 | [PowerApp and Power Automate JSONL intake](features/powerapps-jsonl-intake/index.md) | In progress | Critical | Medium | A | Highest | SharePoint is the standard-connector manual path: the owner three-row stage/inspect/apply/report demo passed. PA-22 retains SharePoint participant acceptance; PA-19 storage/catalog and PA-20 intake/export acceptance passed October 26; PA-21 retains browser edge cases. New PA-26 repairs saved exporter failure handling in isolation, then PA-23 accepts collected snapshots. PA-24 covers native restore; PA-25 optional cutover; PA-12 shared deployment. GitLab PA-15–17 remain independent; both pilots stay enabled. |
| 2 | [Shared export and OOXML pipeline](refactors/core/export-pipeline-hardening.md) | In progress | Critical | High | A | Highest | Removes silent/stale template output and partial PPTX mutation while unlocking `acorn-export`. |
| 3 | [Outbound MCP client embedder interface](refactors/api/outbound-mcp-client-interface.md) | In progress | High | Medium | A | High | Client code and focused tests are complete; separately owned book integration remains before plan retirement. |
| 4 | [Needle 3 release-host validation](migrations/needle-3-upgrade.md) | Waiting on validation | High | Medium | A | High | Linux x86_64 passed on Oklahoma; Linux ARM64 can validate on Dante. The Windows job waits for native shell runner readiness. |
| 5 | [GitLab runner-pool isolation](refactors/ci-runner-pools.md) | In progress | High | Medium | A− | High | Directly addresses observed checkout collisions and protects reporting credentials, but requires administrator evidence. |
| 6 | [GitLab shell runner readiness](testing/gitlab-shell-runner-readiness.md) | Waiting on validation | High | Medium | A− | High | Local shell registration is tested and documented; a native Windows host is needed for the tagged job that unblocks three platform test phases. |
| 7 | [Database-backed artifact generation](features/database-backed-artifact-generation.md) | Planned | High | Medium | A− | High | Reuses mature renderers and revisions to provide reproducible current/historical artifacts. |
| 8 | [Automated JSONL management with the Vergil GitLab bot](features/vergil-gitlab-jsonl-automation/index.md) | Planned | High | High | A | High | PA-05/06/07/09 and BOT-02 are Waiting on validation, subject to prerequisites; deployment/integration phases remain planned/blocked. Runtime contracts exist; live Vergil deployment, authenticated callbacks, source handback and review/report acceptance remain. One-shot admission can create a draft MR. Outside the manual demo critical path. |
| 9 | [SharePoint JSONL download for ACORN intake](features/sharepoint-jsonl-download/index.md) | Planned | High | High | A− | High | Add reusable OAuth 2.0/Entra sign-in and Graph byte capture, with SharePoint intake as the first consumer. Twenty-six bounded phases cover ACRE auth profile loading, verified account identity, browser/device authorization, scoped cache inspection/reset, macOS/Windows/Linux protected caching and native acceptance, Confirm-based sign-in UI, verified downloads and SPD-26 single-platform handoff for HCI; no operational app/flow changes. |
| 10 | [Headless CI SharePoint JSONL download](features/sharepoint-headless-ci-download/index.md) | Planned | High | High | A− | High | Seven bounded phases follow SPD-26 shared functionality and one accepted native OS, without waiting for SPD-11/12: approved GitLab federation, ACRE workload profiles, app-only token acquisition, headless intake, opt-in CI template, live acceptance and operations. No user-token cache, automatic apply or operational workflow changes. |
| 11 | [Sponsor highlights clarity](features/sponsor-highlights-clarity.md) | Planned | High | Mixed | B+ | High | Stakeholder feedback supports an editorial pilot; reviewed ASPECT graduation still needs typed evidence and target-specific schema, local CLI and database/MCP/GitLab integration. |
| 12 | [Structured research-activity enrichment](refactors/core/research-activity-enrichment-integration.md) | Planned | High | Medium | A− | High | Converts already available provider evidence into typed fields with deterministic conflict handling. |
| 13 | [Podman Makefile migration](migrations/podman-makefile.md) | Planned | High | Medium | A− | Medium | Technically well specified, but the broad cross-platform matrix is costly for a development-tooling-only change. |
| 14 | [CEL metadata validation-profile pilot](features/cel-validation-profile-pilot.md) | Planned | High | Medium | B+ | Medium | Strong stop gates and bounded scope, but a named profile owner and enforceable runtime budgets must be proven before adoption. |
| 15 | [Shared API client contract](refactors/api/shared-client-contract.md) | In progress | High | High | A− | Highest | Consolidates the prerequisite mechanics for `acorn-api`; execute in its named contract slices, not as one rewrite. |
| 16 | [CLI composition boundary](refactors/core/cli-composition-boundary.md) | Planned | High | High | A− | High | Makes startup, normalized requests, remote dispatch, and exit behavior testable in one ownership boundary. |
| 17 | [Model acquisition and metadata hardening](features/model-acquisition-hardening.md) | In progress | High | High | A | High | Unifies three overlapping model plans and closes identity, metadata, package, and registry evidence gaps. |
| 18 | [Remote container creation hardening](refactors/core/remote-create-container-targets.md) | In progress | High | High | A | High | Adds preflight and compensating rollback around real external resources; live-host evidence remains necessary. |
| 19 | [Portable diff resource and API hardening](refactors/acorn-diff-resource-and-api-hardening.md) | Planned | High | High | A | High | Adds deterministic resource and output safety to untrusted diff paths while completing a small public API. |
| 20 | [dPID research-object support](features/dpid-research-object-support.md) | Planned | High | High | A | Medium | Exceptionally complete trust and provenance design, but upstream volatility and uncertain demand reduce near-term value. |
| 21 | [Deterministic and assisted gather extraction](features/gather-extraction.md) | Planned | High | High | A− | High | One ordered plan now preserves the deterministic-first security and provenance boundary across CLI and TUI. |
| 22 | [Azure Cosmos DB bucket downloads](features/azure-cosmos-buckets.md) | Planned | High | High | B+ | Medium | Fits the landed source abstraction, but a preview SDK and no recorded deployment demand warrant validation before build-out. |
| 23 | [Interactive metadata chat and terminal prompts](features/interactive-metadata-chat.md) | Planned | High | High | A− | High | Confirm visuals and selectable enrichment candidates have landed; routed chat and interactive format work remain. |
| 24 | [Needle routing and native capability parity](features/needle-router-native-parity.md) | In progress | High | High | A− | High | Establish caller-owned Needle routing and measured native registry parity. |
| 25 | [Generic ask executor adapters](features/ask-executor-adapters.md) | Planned | High | High | A− | High | Build configured MCP, ACP, endpoint, and local sidecar bindings after router contracts land; release waits for native parity. |
| 26 | [Installable acornd background service](features/acornd-service.md) | Planned | High | High | B+ | High | Reuses landed service and CLI operations; service registration, signing, and live host evidence gate release. |
| 27 | [Swift SLM capability profile](features/swift-slm-profile.md) | Planned | High | High | B | Medium | Licensing, host, artifact, and quality gates precede a three-capability release. |
| 28 | [Julia decision sidecar profile](features/julia-decision-sidecar.md) | Planned | High | High | B | Medium | Independent parity, quality, and resource gates precede an advisory decision release. |
| 29 | [Grouped report separators](features/grouped-report-separators.md) | Planned | Medium | Medium | A | High | Extend the existing local --combine renderer with explicit group order and office divider pages; retain the current output API. |
| 30 | [RON support for ACRE configuration](features/acre-ron-configuration.md) | Planned | Medium | Medium | A− | High | Adds the requested configuration format through a map-based adapter that preserves ACRE semantics; staged codec, discovery, CLI, and formatting gates protect existing formats. |
| 31 | [ZON codec hardening](refactors/zon-codec-hardening.md) | Planned | Medium | Medium | A | Medium | A contained correctness/maintainability leaf with strong tests and low integration risk. |
| 32 | [CLI coverage](testing/cli-coverage.md) | Planned | Medium | Medium | A− | Medium | Targets known command and binary failure gaps without reviving obsolete test layout or chasing raw line coverage. |
| 33 | [Portable PID verification](testing/pid-verification.md) | Planned | Medium | High | B+ | Medium | Consolidation and Miri are practical; full Creusot/solver maintenance should proceed only after toolchain feasibility is proven. |
| 34 | [Beam inference integration](features/beam-inference-integration.md) | Planned | Medium | High | B | Medium | Release and serving evidence gate an explicit endpoint integration and ACORN evaluation. |
| 35 | [Coverage throughput](testing/coverage-throughput.md) | Planned | Medium | Medium | A− | Medium | Developer-workflow only; the forced clean stage is the confirmed cost, but every speed claim still needs a measured baseline. |

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
