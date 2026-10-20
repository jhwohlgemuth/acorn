---
okf_version: "0.1"
---

# Plan dependency network

This map records implementation dependencies among the active plans. Portfolio rank is maintained in the [planning index](index.md); dependency order can override rank when a prerequisite contract is not ready.

Executable detail lives in each plan's `# Execution phases` section. Cross-plan edges name stable phase IDs; completed phases are pruned without renumbering unfinished phases. A plan-level edge is not permission to start every phase in that plan.

## Boundary coordination

The [crate boundary architecture](refactors/crate-boundary-architecture.md) coordinates remaining package ownership. The shared API and export plans establish contracts before separate package-extraction plans can be scheduled; no active plan owns package creation or `acorn-store` extraction. CLI composition consumes the landed service/server contracts. The architecture record is not an implementation job.

## Direct edges

### Portable and extraction boundaries

- [Outbound MCP client embedder interface](refactors/api/outbound-mcp-client-interface.md) has landed direct server-map construction and bounded caller-driven cancellation. Its remaining phases document the public interface and record portable/WASM validation.

- [Shared API client contract](refactors/api/shared-client-contract.md) prepares `acorn-api`; endpoint, request, resolved configuration, invocation, error, and provider policy move together only after its acceptance gates pass. API-14 migrates provider environment discovery after API-06 freezes the resolved configuration contract.
- [Shared export and OOXML pipeline](refactors/core/export-pipeline-hardening.md) prepares `acorn-export` by producing one pure projection and fallible in-memory transformations.
- [CLI composition boundary](refactors/core/cli-composition-boundary.md) follows the stable service/server request and error contracts. Command leaves may normalize earlier only when they avoid shared CLI roots; CLI-11 closes the plan after accepted startup and exit behavior.
- [Portable diff hardening](refactors/acorn-diff-resource-and-api-hardening.md) and [ZON codec hardening](refactors/zon-codec-hardening.md) are independent portable leaves.

### Provider and acquisition work

- [Beam inference integration](features/beam-inference-integration.md) requires published release/deployment evidence and a verified serving contract before promotion. It consumes landed shared API mechanics; gather adoption follows the gather contract. It is independent of local SLM sidecars.

- [Model acquisition and metadata hardening](features/model-acquisition-hardening.md) consumes shared endpoint/authentication mechanics but owns Hugging Face identity, GGUF metadata, OCI package discrimination, and safe materialization.
- [Azure Cosmos bucket downloads](features/azure-cosmos-buckets.md) consumes the shared API policy and landed bucket-source/transfer contracts. It does not extend the repository enum or local database backend.
- [dPID research-object support](features/dpid-research-object-support.md) lands portable parser and RO-Crate slices before resolver, gather, persistence, and CLI integration.
- [Structured research-activity enrichment](refactors/core/research-activity-enrichment-integration.md) and dPID serialize on research-activity schema and standards roots.
- [Database-backed artifact generation](features/database-backed-artifact-generation.md) follows the export-context portion of the export plan and uses the existing canonical database/revision contract.

### Gather, inference, and agent tools

- [Deterministic and assisted gather extraction](features/gather-extraction.md) lands deterministic selector and transport policy before its optional inference adapter. Its TUI cutover follows the shared source contract.
- [Needle routing and native capability parity](features/needle-router-native-parity.md) starts SLM-00 registry injection and PAR-00 inventory independently. PAR-18 through PAR-21 build bounded routing-case batches, PAR-17 freezes the complete corpus, and RTR-01 requires SLM-00 and PAR-17. RTR-02/03 then freeze execution and CLI contracts; PAR-01 through PAR-16 deliver measured native parity. PAR-00 schedules narrower extraction phases when a family lacks a reusable operation seam.
- [Interactive metadata chat and terminal prompts](features/interactive-metadata-chat.md) starts IMC-00 inline viewport, IMC-01 UI-neutral state, and IMC-08 enrichment decisions independently; transcript storage and prompt controls follow their respective foundations. IMC-04 consumes PAR-02's full check capability; IMC-05/06 also consume RTR-03's `ask` contract. IMC-07 applies only reviewed fixes. IMC-10/11 add `format --interactive` on the existing workflow independently of generic executor adapters; IMC-12 freezes a future TUI consumer seam without adding a TUI screen. Serialize its parser, inference command, analysis, enrichment, workflow, and generated-help write sets with their current owners.
- [Generic ask executor adapters](features/ask-executor-adapters.md) starts ADP-01 after RTR-02. ADP-08 wires optional features; ADP-09 also waits for RTR-03. ADP-02 through ADP-05 implement explicit MCP, ACP, endpoint, and local-sidecar bindings after ADP-08; ADP-06 waits for PAR-16 before heterogeneous validation and ADP-07 release. This release does not depend on model licensing or macOS hosts.
- Router native adapters consume existing operation services and coordinate command seams with CLI composition; export/gather/model-acquisition plans retain their domain behavior ownership. MCP and ACP adapters reuse landed client/inference contracts; outbound MCP documentation closeout is not a code prerequisite. No automatic inbound MCP re-export is implied.
- The [Swift SLM profile](features/swift-slm-profile.md) and [Julia decision profile](features/julia-decision-sidecar.md) consume ADP-05 for registration. Their legal, host, and feasibility work may begin earlier; SLM-22 and JUL-15 release only after ADP-07. SLM-17 and JUL-11 register library handlers; SLM-23 and JUL-16 own their separate CLI syntax. Each retains its own external, artifact, and quality gates; JUL-15 does not depend on SLM-22.
- [Needle 3 host validation](migrations/needle-3-upgrade.md) and the router plan serialize Needle roots, tool-index/artifact, and CI/build changes. Linux x86_64 and ARM64 validation can run directly on Oklahoma and Dante, independent of Windows job preparation; Windows validation uses its prepared CI jobs. Router implementation uses existing contracts; PAR-15 still needs an installed pinned runner on a supported host for real routing evaluation. The [coordination record](features/slm-sidecars.md) does not own implementation.

### Optional policy validation

- [CEL validation-profile pilot](features/cel-validation-profile-pilot.md) builds on the implemented analyzer and validation contracts. Runtime conformance and resource-bound evidence precede evaluator integration, followed by CLI wiring and team acceptance.
- CEL feature wiring must preserve the landed dependency-capability boundaries. Its runtime/corpus gate can proceed without shared locks, but analyzer results, manifests, and CLI integration serialize with their current owners.
- OPA integration remains conditional future work described in the [assessment](assets/opa-cel-review.md), not a scheduled implementation.

### Reliability and operations

- [Sponsor highlights clarity](features/sponsor-highlights-clarity.md) starts with an editorial template, then validated monthly input, project and portfolio rendering, and PDF accessibility evidence. In parallel, HIL-08 defines event-evidence rules; HIL-09 makes graduation target-specific; HIL-10 through HIL-14 add reviewed ASPECT targets; HIL-15 and HIL-16 prove database/MCP and GitLab review paths; HIL-17 surfaces accepted changes in the report after HIL-04. HIL-18 retires the plan after accessibility, external publication or deferral, and renderer acceptance. Its distribution pilot waits for report-owner, classification, audience, and editorial approvals. HIL-02 and HIL-09 through HIL-14 serialize on research-activity schema roots; HIL-03 through HIL-05 and HIL-17 serialize with grouped separators and shared export work on their exact CLI and renderer files. HIL-16 consumes the implemented GitLab admission and review contracts without waiting for the Power Automate live pilot.
- [Grouped report separators](features/grouped-report-separators.md) starts with explicit group order in report configuration, then composes a single PDF, wires an opt-in local output, and closes with SEP-04 validation and retirement. It is independent of live Power Automate deployment and serializes with the shared export plan on PDF template files.
- [GitLab JSONL logbook reporting](features/powerapps-gitlab-logbook-reporting.md) now uses shared canonical form admission in both the offline manual batch and GitLab JSONL adapter. PA-02 waits on owner/access decisions, PA-04 provisions credentials and the callback flow, PA-03 proves authenticated Vergil polling, and PA-05 connects the intake flow. PA-06 through PA-09 prove human review, live publication, callbacks, and routing. A GitLab-sourced manual fallback requires an explicit repository/inbox handoff before bot intake resumes.
- [Runner-pool isolation](refactors/ci-runner-pools.md) precedes claims that Rust/fuzz/reporting jobs are concurrently isolated. Its reporting pool supports the scheduled reporting workflow.
- [Remote container creation](refactors/core/remote-create-container-targets.md) consumes normalized CLI requests when that boundary lands; leaf rollback/preflight work can proceed first.
- [Podman Makefile migration](migrations/podman-makefile.md) changes development/release tooling only and must preserve product-level Docker and Podman support.
- [CLI coverage](testing/cli-coverage.md) may add command-leaf tests at any time; startup/exit tests follow the CLI composition boundary.
- [PID verification](testing/pid-verification.md) serializes PID production helpers with dPID, then adds Miri and Creusot without blocking ordinary builds.
- [Coverage throughput](testing/coverage-throughput.md) is developer-workflow only and owns no product contract. It serializes on the `Makefile` and root-manifest locks, so it must not overlap Podman migration, runner-pool, PID verification, or export fuzzing phases. Its bounded-test-time phase must observe the exact write sets of concurrent CLI and MCP test phases; COVT-05 preserves measurements in developer documentation before retirement.

## Shared integration surfaces

| Surface | Plans that serialize |
|---|---|
| Workspace manifests and lockfile | model acquisition, Cosmos, gather extraction, CEL, ask adapters, SLM, Miri/Creusot wiring |
| Analyzer results, check orchestration, and module indexes | CEL, gather inference, structured enrichment, diff, interactive metadata chat, CLI coverage |
| `io/api/mod.rs`, configuration roots, endpoint snapshots | shared API contract, dPID, Cosmos, model acquisition |
| `io/mcp/client.rs`, outbound client tests and documentation | outbound MCP client interface; the landed feature graph and manifest boundary remain fixed |
| Research-activity/PID schema roots | dPID, structured enrichment, PID verification, sponsor highlights clarity |
| Logbook graduation persistence, MCP apply/list, and GitLab automation | sponsor highlights clarity; the GitLab JSONL reporting plan's remaining phases own external deployment and pilot systems |
| `io/{archive,oci}.rs`, `io/model/gguf/`, and model planner | model acquisition, SLM/Needle artifact work |
| Tool registry, Needle startup/catalog, inference dispatch | router/parity, Needle validation, CLI composition, interactive metadata chat; adapter leaves consume the frozen binding contract |
| CLI parser, `main.rs`, generated help | CLI composition, dPID, gather, CEL, remote create, ask adapters, interactive metadata chat, SLM, sponsor highlights clarity |
| Enrichment candidates and format workflow | structured enrichment, interactive metadata chat; IMC-08/09 freeze candidate decisions before IMC-11 consumes the workflow |
| Export dispatch/templates/PowerPoint | export pipeline, database-backed artifacts, grouped report separators, sponsor highlights clarity |
| `Makefile` and `.gitlab-ci.yml` | runner pools, Podman, SLM, Needle, PID verification, export fuzzing, coverage throughput |
| Developer test and coverage workflow (`Makefile` test targets, `.config/nextest.toml`, `[profile.coverage]`) | Coverage throughput and CLI coverage serialize; slow-test gating and coverage scoping must preserve other plans' acceptance tests. |

Exact ownership and safe concurrent slices are maintained in the [parallel execution guide](parallel-execution.md).

## Completion discipline

- A plan closes only after its acceptance criteria and durable documentation are complete.
- Remove each completed section, phase, and task after reconciling durable evidence. Do not retain checked lists or completed phase records.
- Preserve the IDs of unfinished phases when earlier phases are pruned, and remove dangling phase dependencies in the same change.
- Delete completed plan files and remove their index, network, and lane entries immediately.
- If a plan discovers a materially different outcome, create a new atomic concept instead of widening the active write set silently.
