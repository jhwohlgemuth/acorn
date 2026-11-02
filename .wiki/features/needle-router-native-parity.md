---
type: Plan
title: Needle routing and native capability parity
description: Route acorn ask through one bounded native capability and measure parity with existing operations.
difficulty: high
impact: high
package: [acorn-lib, acorn-cli, acorn-schema]
tags: [feature, inference, routing, tools, parity]
status: in-progress
execution: phased
phase_budget_hours: 5
---

# Goal

Provide one `acorn ask` interface that selects an eligible native capability and executes it through the existing bounded ACORN services. Establish measured registry parity before the [generic adapter plan](ask-executor-adapters.md) releases heterogeneous routing. Adapter leaves may consume the frozen registry and execution contracts earlier. Preserve unfinished phase IDs while phases move between the atomic plans.

```text
question/command + explicit inputs + application policy
    -> available capability catalog
    -> Needle selects capability and grounded arguments
    -> validate route, arguments, authority, and availability
    -> deterministic executor binding
       native Rust | local SLM | configured MCP tool | ACP agent | inference endpoint
    -> execute once -> typed result / artifacts / provenance
```

Native operations execute existing Rust services; models and agents implement capabilities that require them. The framework is extensible through registered descriptors and adapters, with concrete contracts at operation boundaries. It does not generate shell commands or autonomously orchestrate multiple agents.

# Framework evidence

Local evidence checked on 2026-10-22:

- `cli/mod.rs` exposes `ask` as an alias of `infer`; explicit `infer run` already supports configured prompt backends.
- `io/mcp/tools/mod.rs` contains nine built-in definitions with analysis enabled. Check and validate currently use the same schema validator, so names alone overstate behavioral coverage. Graduation apply is not Needle-exposed.
- `ToolRegistry` already supports typed handlers, Needle schema projections, and effect/exposure/output policy. Extend it as the source of truth; its current MCP directory does not require a duplicate registry or crate extraction.
- `io/sidecar/needle/mod.rs` constructs `ToolRegistry::acorn()` during startup, uses a bounded multi-round loop, and caps the catalog at 32 KiB. It already withholds low-confidence, uncalibrated, negated, and ungrounded calls. The caller-owned registry, terminal execution and catalog-size contracts are now implemented, as detailed below; remaining phases concern native behavioral parity and release evidence.
- `io/mcp/client.rs` exposes `McpClientBackend`, configured discovery/calls, allowlists, context, cancellation, and bounded cleanup. A protocol-successful result can still contain `is_error`.
- `agent/mod.rs` supplies `InferenceBackend`, typed requests/results, capabilities, provenance, and `InferencePermissionPolicy::DenyMutation`; ACP and OpenAI-compatible implementations exist. ACP mutation remains unsupported in this plan.
- Analysis has reusable services in `analyzer/host/service/mod.rs`. Other CLI leaves sometimes combine parsing, operation, and presentation; extract only the operation seam required for parity, coordinating with the CLI composition plan.
- The PAR-00 [inventory](../../tests/fixtures/routing/parity.json) freezes 72 CLI behavior rows: 46 bounded and 26 explicit exclusions. No row has passing registry equivalence evidence yet. The [capability parity guide](../../docs/src/inference/capability-parity.md) records the denominator and extraction seams; `analyze_paths` already supplies the reusable PAR-02 analysis operation.
- Needle now accepts a caller-owned `ToolRegistry`, filters its catalog by offline and exposure policy, and keeps the default startup path. Focused `needle` tests cover injected handlers, MCP isolation, compact deterministic catalogs, and exact custom catalog validation.
- The routing corpus now has 50 labeled nonexecution cases across forbidden, negated, ungrounded, missing-input, and ambiguous requests. Each class has five calibration and five held-out prompts, all linked to inventory rows with zero expected effects.
- The corpus also has 50 grounded read-only cases covering check, diff, gather, doctor, SWHID, and bounded script checks. Expected capability IDs and existing fixture paths are recorded with an even calibration/held-out split.
- The corpus now includes 50 effect and artifact cases across format, link, export, import, download, sync, report, and script checks. Each captures preview or exact caller apply policy, expected effects, and artifact references; denied apply requests expect no execution. PAR-21 completes selection boundaries before the freeze.
- The corpus has 200 unique prompts with 100 calibration and 100 held-out labels. The final 50 cover explicit binding, availability, offline policy, equivalent binding, and remaining mandatory input forms; every bounded inventory row is referenced. PAR-17 audited the split, safety nonexecution, references, input paths, and expected authority, then recorded file hashes in the parity guide. The corpus is frozen before router scoring.
- `ToolRegistry` now owns versioned capability definitions and typed executor bindings. It rejects duplicate IDs and schema/effect/version mismatches, records availability reasons, resolves explicit/preferred/native/local/remote bindings deterministically, and serializes only eligible capabilities under the 32-KiB catalog cap. Five focused library tests pass with the `needle` feature.
- Routed execution now rechecks a selected binding, grounding, schema, offline policy, exact operation/argument/destination authority, limits, deadline, and cancellation before one handler call. It returns typed outcomes, bounded artifact references, and credential-free provenance; cancellation drops the handler future and marks effects as possibly started. Needle has a separate one-completion terminal route while its legacy inference loop remains available. Six focused executor tests and the terminal Needle response test pass.
- `ask`/`infer --route` now compose a caller-owned registry with a first native version capability. Exact JSON input, capability scope, binding override, route preview, and destination-bound apply flags are frozen. The default one-shot path and `infer run` remain available while native leaves are added. Focused route and inference regression tests, catalog checks, and regenerated command help pass.
- Shared native input policy now resolves bounded text, file, and directory references under a canonical root, rejects traversal and escaping symlinks, sorts batches, and reports per-entry failures. Artifact policy validates exact destinations, collision and overwrite authority without writing. Four focused input tests pass; `native.rs` is the leaf module root for subsequent operations.
- The local `acorn.check` v1 native binding now calls `analyze_paths` and returns standard-specific category statuses and structured findings. It is available offline with website checks skipped, and routed `ask` can select it. Four focused library tests and the routed CLI registry tests pass. PAR-02 still owes CLI-equivalent remote, Git, OCI, and fix-preview forms, plus the recorded parity evidence; no inventory row is counted as covered yet.
- Shared sidecar asset/distribution contracts exist. The four planned SLM adapters remain unimplemented; their external gates do not block native parity or generic adapters.

# Shared framework contract

## Capabilities and executor bindings

A capability declares a stable namespaced ID, versioned input/output schemas, task description/examples, effects, and resource limits. An executor binding declares implementation kind, configured model/server/agent identity, supported contract version, availability, and provenance. Multiple implementations may bind to one capability only when they satisfy the same contract. Arbitrary MCP tools or general-purpose agents are not automatically equivalent to native operations.

Extend the existing `ToolRegistry` with narrow registration/adapter interfaces; reuse `Sidecar`, `McpClientBackend`, and `InferenceBackend`. Keep Rust types canonical for built-ins and validate dynamic MCP JSON at the protocol boundary. Do not force tool calls into a chat API. Registration and policy evaluation are pure; discovery/readiness/execution are explicit effects.

## Routing, selection, and execution

1. Build an immutable request-scoped catalog filtered by features, host, verified assets, allowlists, permissions, network/data policy, and caller constraints.
2. Needle selects a capability and grounded arguments. Retain confidence/negation/grounding gates. Missing inputs, uncertainty, or multiple incompatible actions return clarification/withheld with no execution. Preserve source data through typed references or verbatim projection.
3. Bind deterministically: valid explicit caller selection, then configured preference, then an equivalent native implementation, then a compatible local implementation. Equally preferred external bindings require clarification. Do not invent cost/quality rankings or use routing confidence as model-quality evidence.
4. Revalidate arguments, policy, and availability before dispatch. Stale catalogs or schemas fail explicitly. Configured remote services are eligible only under network/data policy; no implicit remote fallback, provider substitution, or model download. Credentials remain outside prompts/catalogs.
5. Execute one terminal capability and return its final result. No recursive `ask` or registry changes from provider output. Preserve the legacy Needle loop through its existing API while giving routed capability execution a distinct single-execution contract.

Honor the 32-KiB catalog limit: deduplicate executor alternatives behind capabilities, bound descriptions/schema projections, and measure serialized size. If needed, shortlist deterministically using explicit scope and declared task metadata, with retained-candidate recall measured against the routing corpus. Never silently truncate tools; request narrower scope when a valid catalog cannot fit. Catalog/schema/binding changes invalidate session tool-index caches.

Use a normalized context for offline/data policy, permitted paths, mutation authority, deadline, cancellation, and input/output budgets. Adapters may narrow authority, never broaden it. Existing CLI writes remain available through explicit typed policy; model-generated arguments are not authorization. Default writes yield a proposal or permission-required outcome, and apply authority binds the exact operation, arguments, and destination. Never automatically retry non-idempotent effects.

Distinguish completed, needs-input/withheld, unavailable, permission-required, invalid-input, cancelled, timed-out, transport-failed, and tool-failed outcomes. Preserve structured/text output, bounded artifact references, and secret-free provenance. Transport success does not mean task success. Verify cancellation cleanup per adapter and report already-completed effects accurately.

# Early near-parity milestone

The PAR-00 machine-readable scenario matrix is frozen against the current CLI. Each row records code owner, input forms/options, outputs/artifacts, effects, feature/platform requirements, registry binding, operation seam, phase, and equivalence test or explicit exclusion. PAR-17 then freezes routing cases. Count behavior, not aliases or duplicate handlers. Report covered/46 bounded scenarios and covered/72 CLI scenarios including exclusions; do not claim a percentage until equivalence evidence exists.

All mandatory bounded scenarios must pass before heterogeneous router release. Adapter and model-profile leaves may use the frozen contracts earlier under the shared integration locks. This milestone covers existing behavior, not every shell convenience or planned ACORN feature.

| Capability family | Mandatory behavior | Phase |
|---|---|---|
| Existing registry/logbook | Preserve tools and explicit graduation policy; separate schema validation from analysis | Landed registry contract |
| Check | Existing analysis categories, findings, offline and unavailable-category behavior | PAR-02 |
| Diff/similarity | Existing semantic diff/patch and pairwise metrics | PAR-03 |
| Gather | Existing PID extraction from text/documents/permitted URLs | PAR-04 |
| Format/crosswalk | Supported standards, normalization/enrichment, loss warnings, preview/apply | PAR-05 |
| Linked data | Existing transformation and authorized output | PAR-06 |
| SWHID | Current calculate/verify over permitted inputs | PAR-07 |
| Doctor | Structured diagnostics/readiness; no automatic repair | PAR-08 |
| Export | Existing formats/targets, bounded batches, artifact output | PAR-09 |
| Import | Current spec/model import, preview and explicit apply | PAR-10 |
| Download | Existing bucket/model/Needle acquisition/status, explicitly requested | PAR-11 |
| Sync | Current target configuration preview/apply | PAR-12 |
| Report | Existing artifact generation and explicit publication | PAR-13 |
| Script checks | Existing check/check-safe; no unrestricted automation | PAR-14 |
| Prompt/MCP commands | Preserve direct commands; the configured MCP adapter is landed and heterogeneous release follows native parity | ADP-03 through ADP-04 |

Exclude interactive TUI/clipboard UX, persistent servers, infrastructure/secret creation, automatic repair, and unrestricted automation from the bounded milestone. Keep them visible in the total CLI inventory. Unsupported flags/scenarios need explicit entries rather than disappearing from the denominator. Native-handler equivalence tests require no model weights; real Needle routing is a separate gate.

# CLI, ownership, and ordering

`acorn ask [QUESTION_OR_COMMAND]` remains the primary interface. Preserve positional/stdin exclusivity, current flags/aliases/escaping, ordinary direct commands, and `infer run`. The landed `--route` mode freezes structured input, binding selection, route preview, and apply-policy options while native bindings are added. Do not add per-provider inference subcommands.

Registry/adapter leaves initially live under `crates/acorn-lib/src/io/mcp/tools/` with preserved public exports and optional feature gates. CLI business logic needed by both interfaces moves into a small library operation seam; the CLI keeps normalization/presentation. Do not invoke the ACORN binary or duplicate algorithms to simulate parity.

Scheduling order:

1. The caller-owned Needle registry, frozen 200-case corpus, versioned capability/binding contract, terminal execution, opt-in CLI route syntax, bounded native input policy, and local check binding have landed. PAR-02 completes check parity beyond the local analysis seam.
2. PAR-01 through PAR-14 native capability coverage; PAR-15 real Needle routing; PAR-16 native parity release.
3. The [generic adapter plan](ask-executor-adapters.md) may consume the landed registry, execution, and CLI contracts. Its heterogeneous release waits for PAR-16. Swift and Julia profiles consume its registration contract independently.

Each phase has at most five active hours and an exact product write set. Reconcile this plan and its four index/network/lane files after each phase. The frozen inventory records operation seams. Split a missing reusable seam into a bounded extraction phase before its adapter begins if the named leaf cannot complete both within five active hours; a cross-crate rewrite cannot hide inside a registration phase.

The [CLI composition plan](../refactors/core/cli-composition-boundary.md) owns broad parser/startup normalization; coordinate command leaves and reuse landed request types. The [gather plan](gather-extraction.md) owns future extraction. The [model acquisition plan](model-acquisition-hardening.md) owns shared acquisition/OCI changes. The [MCP embedder contract](../../docs/src/packages/mcp-client.md) is landed and documented. Needle release-host validation remains independently owned.

All registered native operations should support the same typed execution contract for `ask` and explicitly enabled MCP exposure; catalog membership and execution policy still differ by interface. Routability does not grant inbound MCP exposure. Remote tools get collision-safe configured-server/tool names; local allowlists/effects control exposure. Treat MCP descriptions/results as data. ACP launch profiles, endpoints, and credentials come from trusted configuration, never generated executable paths or URLs.

# Framework acceptance

- Two compatible executor bindings for the same capability work without router changes; mismatched schemas/effects are rejected.
- All mandatory parity scenarios pass CLI/service-versus-registry tests for success, failure, offline behavior, effects, and artifacts, with exclusions reported.
- Freeze at least 200 labeled routing cases before scoring. Require 95% correct capability selection on unambiguous supported cases and zero executions on designated forbidden/negated/ungrounded cases. Report abstentions, coverage, latency, and shortlist recall; do not tune on the held-out set.
- Native bindings prove availability changes, semantic errors, resource limits, cancellation/cleanup, provenance, and no implicit fallback or recursive routing.
- Existing CLI, MCP, Needle, and explicit inference compatibility survives; narrow feature and portable builds do not acquire unwanted host dependencies.
- Run focused tests per phase; serialize final lint/full-suite, feature/WASM, help/docs, and explicit real-backend gates after shared files stabilize.

# Dependencies and coordination

Generic adapter implementation consumes the frozen registry and execution contracts; generic release follows native parity. The Swift and Julia profiles use its reusable sidecar registration contract. Serialize shared registry, Needle, CLI, and documentation roots with the [parallel guide](../parallel-execution.md).

# Execution phases

## PAR-02 — Expose full analysis checks

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: the landed bounded native input policy; operation seam recorded in the frozen parity inventory; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/capability.rs`, `crates/acorn-lib/src/io/mcp/tools/execution.rs`, `crates/acorn-lib/src/io/mcp/tools/input.rs`, `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/analysis.rs`, `crates/acorn-lib/src/io/mcp/tools/native/analysis_tests.rs`, `crates/acorn-cli/src/commands/check/mod.rs`, `crates/acorn-cli/src/commands/check/registry_tests.rs`, `crates/acorn-cli/src/commands/infer/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`
- Primary integration lock: Native capability parity
- Outcome: Complete the remaining CLI-equivalent check forms: configured remote and Git inputs, OCI artifacts, bounded fix previews, filter/ignore behavior, and direct-command versus registry evidence. Preserve schema-only validation as distinct. The local category and structured-finding binding is already implemented.
- Focused validation: Remote/Git/OCI/offline and fix-preview fixtures, filter/ignore selection, unavailable dependencies, and matching findings. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-02 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-03 — Expose semantic diff and similarity

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: the landed bounded native input policy; operation seam recorded in the frozen parity inventory; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/diff.rs`, `crates/acorn-lib/src/io/mcp/tools/native/diff_tests.rs`, `crates/acorn-cli/src/commands/diff/mod.rs`, `crates/acorn-cli/src/commands/diff/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current semantic diff, JSON Patch, and supported similarity metrics. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Equal/changed inputs, metric options, limits, and matching patch/score outputs. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-03 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-04 — Expose current gather operations

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: the landed bounded native input policy; operation seam recorded in the frozen parity inventory; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/gather.rs`, `crates/acorn-lib/src/io/mcp/tools/native/gather_tests.rs`, `crates/acorn-cli/src/commands/gather/mod.rs`, `crates/acorn-cli/src/commands/gather/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current PID extraction from text/documents/permitted URLs and its provenance; no future extraction feature. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Input types, offline rejection, duplicates, bounds, and matching extracted identifiers. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-04 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-05 — Align format and crosswalk behavior

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: the landed bounded native input policy; operation seam recorded in the frozen parity inventory; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/format.rs`, `crates/acorn-lib/src/io/mcp/tools/native/format_tests.rs`, `crates/acorn-cli/src/commands/format/mod.rs`, `crates/acorn-cli/src/commands/format/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Supported standards, normalization/enrichment, loss warnings, preview and authorized output. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Format/standard matrix, offline enrichment, field-loss warnings, preview/apply, and invalid-input no-write. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-05 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-06 — Expose linked-data operations

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: the landed bounded native input policy; operation seam recorded in the frozen parity inventory; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/link.rs`, `crates/acorn-lib/src/io/mcp/tools/native/link_tests.rs`, `crates/acorn-cli/src/commands/link/mod.rs`, `crates/acorn-cli/src/commands/link/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Existing linked-data transformation with typed output and destination policy. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Transformation fixtures, repeated operation behavior, preview/apply, and schema failures. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-06 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-07 — Expose SWHID calculate and verify

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: the landed bounded native input policy; operation seam recorded in the frozen parity inventory; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/swhid.rs`, `crates/acorn-lib/src/io/mcp/tools/native/swhid_tests.rs`, `crates/acorn-cli/src/commands/swhid/mod.rs`, `crates/acorn-cli/src/commands/swhid/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current local object calculation/verification through existing library behavior. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: File/directory/revision fixtures, mismatches, permitted paths, and stable identifiers. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-07 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-08 — Expose structured diagnostics

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: the landed bounded native input policy; operation seam recorded in the frozen parity inventory; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/doctor.rs`, `crates/acorn-lib/src/io/mcp/tools/native/doctor_tests.rs`, `crates/acorn-cli/src/commands/doctor/mod.rs`, `crates/acorn-cli/src/commands/doctor/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Report-only diagnostics and readiness; keep automatic repair unavailable. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Platform/dependency fixtures, missing tools, no mutations, redaction, and matching report data. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-08 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-09 — Align exports and artifact results

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: the landed bounded native input policy; operation seam recorded in the frozen parity inventory; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/export.rs`, `crates/acorn-lib/src/io/mcp/tools/native/export_tests.rs`, `crates/acorn-cli/src/commands/export/mod.rs`, `crates/acorn-cli/src/commands/export/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Existing formats/targets and bounded batch inputs through shared renderers; preserve update export. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Format/target fixtures, output bytes/artifacts, failed renderers, and authorized destinations. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-09 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-10 — Expose bounded imports

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: the landed bounded native input policy; operation seam recorded in the frozen parity inventory; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/import.rs`, `crates/acorn-lib/src/io/mcp/tools/native/import_tests.rs`, `crates/acorn-cli/src/commands/import/mod.rs`, `crates/acorn-cli/src/commands/import/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current spec/model import previews and explicit apply using existing services. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Local/remote inputs, invalid specs, offline behavior, persistence policy, and preview/apply equivalence. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-10 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-11 — Expose explicit downloads and status

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: the landed bounded native input policy; operation seam recorded in the frozen parity inventory; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/download.rs`, `crates/acorn-lib/src/io/mcp/tools/native/download_tests.rs`, `crates/acorn-cli/src/commands/download/mod.rs`, `crates/acorn-cli/src/commands/download/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Existing bucket/model/Needle acquisition/status through current planners; only explicit acquisition requests. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Dry-run/cache/status, offline/missing cache, clobber policy, cancellation, and no inference-triggered downloads. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-11 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-12 — Expose configuration synchronization

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: the landed bounded native input policy; operation seam recorded in the frozen parity inventory; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/sync.rs`, `crates/acorn-lib/src/io/mcp/tools/native/sync_tests.rs`, `crates/acorn-cli/src/commands/sync/mod.rs`, `crates/acorn-cli/src/commands/sync/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current target selection, configuration previews, and explicit apply. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Supported targets, unchanged configuration, proposed diffs, exact apply policy, and CLI equivalence. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-12 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-13 — Expose reporting and explicit publication

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: the landed bounded native input policy; operation seam recorded in the frozen parity inventory; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/report.rs`, `crates/acorn-lib/src/io/mcp/tools/native/report_tests.rs`, `crates/acorn-cli/src/commands/report/mod.rs`, `crates/acorn-cli/src/commands/report/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Existing report artifact generation and publication through current destination/credential policy. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Deterministic artifacts, denied/default publish, authorized mock publication, retries, and secret-free results. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-13 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-14 — Expose constrained script checking

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: the landed bounded native input policy; operation seam recorded in the frozen parity inventory; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/script.rs`, `crates/acorn-lib/src/io/mcp/tools/native/script_tests.rs`, `crates/acorn-cli/src/commands/exec/mod.rs`, `crates/acorn-cli/src/commands/exec/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current check/check-safe behavior; do not register unrestricted automation. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Static/evaluation fixtures, forbidden operations, fuel/size bounds, and matching findings. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-14 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-15 — Prove routing at near-parity catalog size

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-02 through PAR-14; pinned Needle runner installed on a supported host
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/needle.rs`, `crates/acorn-lib/src/io/sidecar/needle/mod.rs`, `crates/acorn-lib/src/io/sidecar/needle/assets.rs`, `crates/acorn-lib/src/io/sidecar/needle/tests/mod.rs`, `tests/fixtures/routing/results.json`, `docs/src/inference/routing-evaluation.md`
- Primary integration lock: Needle routing evaluation
- Outcome: Enforce catalog budgets/cache invalidation and score the frozen corpus against the actual native catalog. Add measured deterministic scoping only if needed.
- Focused validation: Declared routing gates, no silent truncation, retained-candidate recall, negative-case nonexecution, abstention, latency, and index/schema cache isolation.
- Handoff: Remove PAR-15 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-16 — Release native near parity

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-15
- Exact write set: `tests/fixtures/routing/parity.json`, `docs/src/inference/capability-parity.md`, `docs/src/inference/router.md`, `docs/src/SUMMARY.md`, `CommandLineHelp.md`, `.wiki/features/needle-router-native-parity.md`, `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: Router release gate
- Outcome: Publish measured coverage, exclusions, compatibility, and examples before heterogeneous router release. Failed scenarios remain remaining work.
- Focused validation: Serialize make lint, make test, applicable feature/WASM checks, help/book checks, and matrix/equivalence audit.
- Handoff: Retire this plan and its index, network, and lane entries after native parity acceptance; unblock ADP-06's heterogeneous release gate.
