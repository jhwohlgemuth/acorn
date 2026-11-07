---
okf_version: "0.1"
---

> **Sanitized copy:** Account labels and related assets have been redacted. Historical checksums and import results describe the originals; see [privacy and reuse limits](privacy.md).

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

The October 27, 2026 [review inventory](_assets/portfolio-review-20261027.json) records the historical plan set against `bde5df3f`. The November 7, 2026 currency review uses repository baseline `02843b3d` and the official remote planning-skill package downloaded that day. After retiring completed work and consolidating the Power Platform intake portfolio, the current portfolio has **43 executable plans, two coordination records, and one epic**. The intake epic separates shared experience, approval ledger, approved export, pilot release, retained GitLab fallback, and shared deployment into atomic plans while preserving stable PA phase IDs. This review checked every plan's metadata, remaining phases, links, asset placement, dependency/readiness claims, shared locks, and related-plan boundaries; source evidence was spot-checked for the recommended next slices. It was not a new product implementation, full test suite, cloud acceptance run, or proof that external prerequisites are available. The same-day planning update at `401ab463` adds the independently evidenced WebMCP evaluation plan without reopening the completed portfolio-wide review.

SharePoint is the accepted default. Contributor access, recovery, exporter repair, native restore and immutable editing are accepted; GitLab intake/catalog remain Off. Ease of use is the next Power Platform priority: PA-34 freezes one shared toggle/button-bar/radio-checkbox-menu and field-to-report contract. Bounded PA-35 and PA-36 tracks build the report-shaped app and review ledger in parallel; PA-37 freezes approved projections; PA-38 gates the collected export; PA-39 aligns contributor intake; and PA-40/41 accept and cut over both consistent apps. PA-12 then requires PA-41A plus an approved shared target, groups, capacity, owners, connections and baseline policy. The current form has report placement first, a distinct `general` destination and My Submissions between the banner and form. PA-29 standalone general-logbook intake and title-page reporting passed local acceptance and is pruned. PA-32 project coverage/history is published and accepted, with a serialized five-minute refresh and verified October totals. Month/project cards link to original submissions using existing reviewer access. Contributor-account current/history navigation is accepted; PA-33 is pruned and dashboard-only Read grants are verified. GitLab PA-15–17 retain their isolated validation blockers and are not SharePoint release gates.

[SharePoint OAuth download](features/sharepoint-jsonl-download/index.md) has landed ACRE profiles, verified identity and fixture-tested device authorization. Its next implementation phases are ready, but the live tenant feasibility phase remains blocked on an approved public-client registration and Graph consent. Browser authorization, protected cache backends, Graph capture, CLI wiring and live tenant acceptance remain. [Headless CI](features/sharepoint-headless-ci-download/index.md) waits for SPD-26 plus one accepted native platform, not all three. Browser download and local stage/inspect/apply/report work independently of both plans and Vergil.

Non-PA plans retain substantive work. Source spot-checks confirm coverage targets still force clean builds; grouped report separators and native cache adapters are not implemented. Native release-host checks still require their named hosts. Existing uncommitted implementation work is preserved; module stubs do not count as completed behavior.

ADP-05 and ADP-09 have landed. [ADP](features/ask-executor-adapters.md) ADP-06 waits for PAR-16 native parity before heterogeneous validation. IMC-12 remains ready for the later TUI consumer seam. This scheduling update does not change the portfolio ranking below.

The [current guides and retained recovery sources](features/powerapps-jsonl-intake/source-bundle-readme.md) supersede old source/demo distribution ZIPs and redundant PDF extracts. Historical acceptance and native archives remain subject to [sanitization limits](privacy.md). The [cleanup record](features/powerapps-jsonl-intake/test-artifact-cleanup.md#november-5-power-platform-cleanup) identifies the November app renames, removed test artifacts and unsupported self-chat poller, retained PA-15/17 evidence, and preserved premium app dependencies.

Ranking uses impact first, then remaining difficulty, with user/architecture value and readiness as tie-breakers. “Quality” rates the plan rather than the proposed feature:

- **A:** current evidence, coherent ownership, explicit sequencing, and verifiable acceptance gates.
- **B:** useful direction, but feasibility, demand, external prerequisites, or the first concrete consumer needs stronger evidence before implementation.

### Recommended next independently mergeable phases

First finish and land the already active PAR-34 routing repair; do not start another phase against its Needle evaluation lock. After this portfolio update lands, the following four ready phases form the strongest next wave. Their declared write sets and primary locks are disjoint; each remains a separate merge and five-hour acceptance boundary. Any central index reconciliation is serialized at handoff.

| Order | Phase | Why now | Primary lock | Independence boundary |
|---:|---|---|---|---|
| 1 | [PA-34 — Freeze the shared form and report-preview contract](features/powerapps-jsonl-intake/approval-experience/index.md#pa-34--freeze-the-shared-form-and-report-preview-contract) | Highest-value intake prerequisite; it turns the approved ease-of-use direction into one accessible component and field-to-report contract before either app or ledger builds against it. | Power Apps shared UI contract | Owns only its intake-plan assets and TEST ONLY component library; no ACORN code or operational form mutation. |
| 2 | [ACD-07 — Expose composable server lifecycle](features/acornd-service.md#acd-07--expose-composable-server-lifecycle) | The current server already has standalone lifecycle, readiness, TLS, and drain seams; this slice makes them injectable without taking the workspace graph. | Service/RPC contract | Restricted to `acorn-server` configuration/runtime/lib and focused tests. |
| 3 | [SPD-06 — Resolve the configured SharePoint document](features/sharepoint-jsonl-download/index.md#spd-06--resolve-the-configured-sharepoint-document) | The provider leaf is fixture-testable without tenant consent and advances the high-value SharePoint downloader while SPD-01 remains externally blocked. | SharePoint provider leaf | Restricted to `sharepoint/resolve.rs` and dedicated fixtures; no auth, cache, CLI, or Power Platform writes. |
| 4 | [DBA-01 — Define the database artifact selector](features/database-backed-artifact-generation.md#dba-01--define-the-database-artifact-selector) | Small, consumer-backed API slice that unlocks current/revision artifact resolution while reusing mature renderers. | Database artifact API | Adds one database leaf and focused tests; no renderer, export-dispatch, schema, or CLI edits. |

PA-34 should be reviewed first because it governs the user-facing intake direction. ACD-07, SPD-06, and DBA-01 can be prepared concurrently on separate branches or worktrees. Merge each independently, then remove only its completed phase and promote the newly eligible successor. ACD-03 and SPD-24 are good follow-ups but both claim workspace manifests/lockfile work, so they must not run together. Schema-root candidates BGO-01, ENR-01, PID-01, and later Sponsor Highlights schema phases likewise serialize rather than joining this wave.

### Ranked executable plans

| Rank | Plan | Status | Impact | Difficulty | Quality | Value | Assessment |
|---:|---|---|---|---|---|---|---|
| 1 | [Shared intake and approval experience](features/powerapps-jsonl-intake/approval-experience/index.md) | Planned | Critical | High | A | Highest | PA-34 is ready and freezes one accessible component/token and field-to-report contract. Bounded PA-35 slices build the report-shaped reviewer canvas; bounded PA-39 slices refit contributor intake and prove cross-app consistency without changing receipt semantics. |
| 2 | [Intake approval ledger and finalization](features/powerapps-jsonl-intake/approval-ledger/index.md) | Planned | Critical | High | A | Highest | Build indexed/paged closed-period batches, ETag-safe drafts, deterministic approval identities, correction mappings, exact frozen bytes and one Approved visibility commit. PA-36 can start after PA-34 in parallel with the fixture-backed canvas. |
| 3 | [Approval-gated collected JSONL export](features/powerapps-jsonl-intake/approval-export/index.md) | Planned | Critical | High | A | Highest | Preserve the exact pre-approval baseline and append only deterministic frozen projections from complete Approved batches, retaining current limits, failure preservation, readback and freeze-only rollback. |
| 4 | [Intake approval pilot acceptance and cutover](features/powerapps-jsonl-intake/approval-release/index.md) | Planned | Critical | High | A | Highest | Join the consistent apps, review ledger, approval exporter and actual ACORN PDF in technical and uncoached user acceptance; operational PA-41/41A require explicit authorization, fresh backup/baseline and tested freeze rollback. |
| 5 | [Shared Power Platform intake deployment](features/powerapps-jsonl-intake/shared-deployment/index.md) | Planned | Critical | High | A− | High | PA-12/12A wait for PA-41A plus a named environment, groups, owners, licensing/capacity, DLP, storage, notification destination and baseline policy; isolated target acceptance precedes any future-link cutover. |
| 6 | [Retained GitLab intake fallback hardening](features/powerapps-jsonl-intake/gitlab-fallback-hardening/index.md) | Waiting on validation | Medium | High | A− | Medium | Complete remaining isolated byte limits, concurrent writes and native restore from retained backups. Operational GitLab flows remain Off; passing evidence does not authorize restoration and does not block SharePoint approval work. |
| 7 | [Needle 3 release-host validation](migrations/needle-3-upgrade.md) | Waiting on validation | High | Medium | A | High | Linux x86_64 passed on Oklahoma and Linux ARM64 passed on Dante. NDL-02 is blocked on GSR-02 because runner 55645 cannot finish a tagged job. |
| 8 | [GitLab runner-pool isolation](refactors/ci-runner-pools.md) | Planned | High | Medium | A− | High | Directly addresses observed checkout collisions and protects reporting credentials, but RUN-01 cannot start without runner-administrator access and worker capacity. |
| 9 | [GitLab shell runner readiness](testing/gitlab-shell-runner-readiness.md) | Waiting on validation | High | Medium | A− | High | Local shell registration is tested. GSR-02 is blocked: runner 55645 starts a tagged job, then the final status update returns 403. NDL-02 and MOD-11 stay blocked; planned POD-11 cannot start until that proof passes. |
| 10 | [Database-backed artifact generation](features/database-backed-artifact-generation.md) | Planned | High | Medium | A− | High | Reuses mature renderers and revisions to provide reproducible current/historical artifacts. |
| 11 | [Automated JSONL management with the Vergil GitLab bot](features/vergil-gitlab-jsonl-automation/index.md) | Planned | High | High | A | High | PA-05/06/07/09 and BOT-02 are Waiting on validation, subject to prerequisites; deployment/integration phases remain planned/blocked. Runtime contracts exist; live Vergil deployment, authenticated callbacks, source handback and review/report acceptance remain. One-shot admission can create a draft MR. Outside the manual demo critical path. |
| 12 | [Power Automate Teams workflow bot](features/powerautomate-teams-bot/index.md) | Planned | High | High | A− | High | The owner-only Workflows request/reply summarization slice is repaired and accepted. Eleven bounded remaining phases first gate interaction privacy, correlation and native Workflows UX, then package durable Dataverse state, shared routing, centralized responses, bounded capabilities, recovery and governed pilot release. Sponsor Highlights intake, Vergil and ACORN code retain separate ownership. |
| 13 | [SharePoint JSONL download for ACORN intake](features/sharepoint-jsonl-download/index.md) | Planned | High | High | A− | High | Add reusable OAuth 2.0/Entra sign-in and Graph byte capture, with SharePoint intake as the first consumer. ACRE auth profiles, verified account identity and fixture-tested device authorization are landed. Twenty-one remaining bounded phases cover tenant feasibility, browser authorization, scoped cache inspection/reset, macOS/Windows/Linux protected caching and native acceptance, Confirm-based sign-in UI, verified downloads and SPD-26 single-platform handoff for HCI; no operational app/flow changes. |
| 14 | [Headless CI SharePoint JSONL download](features/sharepoint-headless-ci-download/index.md) | Planned | High | High | A− | High | Seven bounded phases follow SPD-26 shared functionality and one accepted native OS, without waiting for SPD-11/12: approved GitLab federation, ACRE workload profiles, app-only token acquisition, headless intake, opt-in CI template, live acceptance and operations. No user-token cache, automatic apply or operational workflow changes. |
| 15 | [Sponsor highlights clarity](features/sponsor-highlights-clarity.md) | Planned | High | Mixed | B+ | High | Stakeholder feedback supports an editorial pilot; reviewed ASPECT graduation still needs typed evidence and target-specific schema, local CLI and database/MCP/GitLab integration. |
| 16 | [Structured research-activity enrichment](refactors/core/research-activity-enrichment-integration.md) | Planned | High | Medium | A− | High | Converts already available provider evidence into typed fields with deterministic conflict handling. |
| 17 | [Big-O metadata validation and agent comparison](features/big-o-agent-validation-and-comparison.md) | Planned | High | Medium | A | High | Converts landed parser/schema primitives into fixture-proven RAD validation and deterministic MCP, native, WASM, and WebMCP comparison; MCP/native registration waits for the active router release lock. |
| 18 | [Podman Makefile migration](migrations/podman-makefile.md) | Planned | High | Medium | A− | Medium | Technically well specified, but the broad cross-platform matrix is costly for a development-tooling-only change. |
| 19 | [CEL metadata validation-profile pilot](features/cel-validation-profile-pilot.md) | Planned | High | Medium | B+ | Medium | Strong stop gates and bounded scope, but a named profile owner and enforceable runtime budgets must be proven before adoption. |
| 20 | [Shared API client contract](refactors/api/shared-client-contract.md) | Planned | High | High | A− | Highest | Consolidates the prerequisite mechanics for `acorn-api`; execute in its named contract slices, not as one rewrite. |
| 21 | [CLI composition boundary](refactors/core/cli-composition-boundary.md) | Planned | High | High | A− | High | Makes startup, normalized requests, remote dispatch, and exit behavior testable in one ownership boundary. |
| 22 | [Model acquisition and metadata hardening](features/model-acquisition-hardening.md) | Planned | High | High | A | High | Unifies three overlapping model plans and closes identity, metadata, package, and registry evidence gaps. |
| 23 | [Remote container creation hardening](refactors/core/remote-create-container-targets.md) | Planned | High | High | A | High | Adds preflight and compensating rollback around real external resources; live-host evidence remains necessary. |
| 24 | [Portable diff resource and API hardening](refactors/acorn-diff-resource-and-api-hardening.md) | Planned | High | High | A | High | Adds deterministic resource and output safety to untrusted diff paths while completing a small public API. |
| 25 | [dPID research-object support](features/dpid-research-object-support.md) | Planned | High | High | A | Medium | Exceptionally complete trust and provenance design, but upstream volatility and uncertain demand reduce near-term value. |
| 26 | [Deterministic and assisted gather extraction](features/gather-extraction.md) | Planned | High | High | A− | High | One ordered plan now preserves the deterministic-first security and provenance boundary across CLI and TUI. |
| 27 | [Azure Cosmos DB bucket downloads](features/azure-cosmos-buckets.md) | Planned | High | High | B+ | Medium | Fits the landed source abstraction, but a preview SDK and no recorded deployment demand warrant validation before build-out. |
| 28 | [Interactive metadata chat and terminal prompts](features/interactive-metadata-chat.md) | Planned | High | High | A− | High | Interactive format review has landed; the headless TUI consumer seam, multi-select control, and release validation remain. |
| 29 | [Needle routing and native capability parity](features/needle-router-native-parity.md) | In progress | High | High | A− | High | Native routes cover 46/46 bounded rows; PAR-34 repairs failed real Needle selection before PAR-15 evaluation and PAR-16 release validation. |
| 30 | [Generic ask executor adapters](features/ask-executor-adapters.md) | Planned | High | High | A− | High | Configured MCP, ACP, endpoint, and generic sidecar registration plus CLI composition are landed. Both remaining phases wait for PAR-16 native parity acceptance. |
| 31 | [Installable acornd background service](features/acornd-service.md) | Planned | High | High | B+ | High | Portable jobs and strict daemon configuration are landed; SQLite history and composable server lifecycle are ready, while registration, signing, and live host evidence still gate release. |
| 32 | [Swift SLM capability profile](features/swift-slm-profile.md) | Planned | High | High | B | Medium | Licensing, host, artifact, and quality gates precede a three-capability release. |
| 33 | [Julia decision sidecar profile](features/julia-decision-sidecar.md) | Planned | High | High | B | Medium | Independent parity, quality, and resource gates precede an advisory decision release. |
| 34 | [Grouped report separators](features/grouped-report-separators.md) | Planned | Medium | Medium | A | High | Extend the existing local --combine renderer with explicit group order and office divider pages; retain the current output API. |
| 35 | [RON support for ACRE configuration](features/acre-ron-configuration.md) | Planned | Medium | Medium | A− | High | Dependency/format registration and the map-based adapter are landed; lifecycle, CLI discovery, formatting, and final documentation gates remain. |
| 36 | [Report PowerPoint visual regression](testing/powerpoint-report-visual-regression.md) | Planned | Medium | Medium | A− | High | Existing OOXML tests and PDF snapshots make the harness tractable; Linux and native Windows baselines require pinned renderers and fonts, and Windows CI waits for GSR-02. |
| 37 | [ZON codec hardening](refactors/zon-codec-hardening.md) | Planned | Medium | Medium | A | Medium | A contained correctness/maintainability leaf with strong tests and low integration risk. |
| 38 | [CLI coverage](testing/cli-coverage.md) | Planned | Medium | Medium | A− | Medium | Targets known command and binary failure gaps without reviving obsolete test layout or chasing raw line coverage. |
| 39 | [WebMCP evaluations](testing/webmcp-evaluations.md) | Planned | Medium | Medium | A− | Medium | Adds an independent deterministic live-browser signal for all eight tools plus explicit model-selection evidence; CI waits for a proven Chrome/WebMCP runner. |
| 40 | [Portable PID verification](testing/pid-verification.md) | Planned | Medium | High | B+ | Medium | Consolidation and Miri are practical; full Creusot/solver maintenance should proceed only after toolchain feasibility is proven. |
| 41 | [Beam inference integration](features/beam-inference-integration.md) | Planned | Medium | High | B | Medium | Release and serving evidence gate an explicit endpoint integration and ACORN evaluation. |
| 42 | [Coverage throughput](testing/coverage-throughput.md) | Planned | Medium | Medium | A− | Medium | Developer-workflow only; the forced clean stage is the confirmed cost, but every speed claim still needs a measured baseline. |
| 43 | [Mise development-tool integration](migrations/mise-toolchain-integration.md) | Planned | Medium | Medium | A− | Medium | A locked cross-platform inventory can remove local/CI version drift, but Make, Docker image, CI, and native-host adoption must land serially under the CI/build lock. |

### Epics

| Epic | Quality | Impact | Value | Assessment |
|---|---|---|---|---|
| [PowerApp and Power Automate intake portfolio](features/powerapps-jsonl-intake/index.md) | A | Critical | Highest | Coordinates the accepted SharePoint baseline and six atomic remaining plans; defines the UI, immutable-receipt, approval, export, rollback and adjacent-plan boundaries without owning phases. |

### Coordination records

| Record | Quality | Impact | Value | Assessment |
|---|---|---|---|---|
| [Crate boundary architecture](refactors/crate-boundary-architecture.md) | A | Critical | Highest | Coordinates the remaining package map; existing child plans prepare API, export, and CLI contracts, while each future extraction needs its own atomic plan. |
| [Ask routing and sidecar coordination](features/slm-sidecars.md) | A− | High | High | Orders native routing, generic adapters, and independently gated Swift and Julia releases. |

## Planning rules

### Adding a plan

1. Create one semantically named Markdown file for one atomic outcome.
2. Add `type: Plan`, `title`, `description`, `difficulty`, `impact`, a non-empty `package` list, `tags`, and `status` to YAML frontmatter.
3. Executable plans use `execution: phased` and `phase_budget_hours: 5`. Epics and non-executable coordination plans use `execution: coordination` without phases or a phase budget; the browser displays these coordination plans as Epics.
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
