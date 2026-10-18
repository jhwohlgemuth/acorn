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

Provide one `acorn ask` interface that selects an eligible native capability and executes it through the existing bounded ACORN services. Establish measured registry parity before the [generic adapter plan](ask-executor-adapters.md) adds external bindings. Preserve unfinished phase IDs while phases move between the atomic plans.

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

Local evidence checked on 2026-10-14:

- `cli/mod.rs` exposes `ask` as an alias of `infer`; explicit `infer run` already supports configured prompt backends.
- `io/mcp/tools/mod.rs` contains nine built-in definitions with analysis enabled. Check and validate currently use the same schema validator, so names alone overstate behavioral coverage. Graduation apply is not Needle-exposed.
- `ToolRegistry` already supports typed handlers, Needle schema projections, and effect/exposure/output policy. Extend it as the source of truth; its current MCP directory does not require a duplicate registry or crate extraction.
- `io/sidecar/needle/mod.rs` constructs `ToolRegistry::acorn()` during startup, uses a bounded multi-round loop, and caps the catalog at 32 KiB. It already withholds low-confidence, uncalibrated, negated, and ungrounded calls. Caller-owned registry injection, terminal execution, and catalog-size handling are required.
- `io/mcp/client.rs` exposes `McpClientBackend`, configured discovery/calls, allowlists, context, cancellation, and bounded cleanup. A protocol-successful result can still contain `is_error`.
- `agent/mod.rs` supplies `InferenceBackend`, typed requests/results, capabilities, provenance, and `InferencePermissionPolicy::DenyMutation`; ACP and OpenAI-compatible implementations exist. ACP mutation remains unsupported in this plan.
- Analysis has reusable services in `analyzer/host/service/mod.rs`. Other CLI leaves sometimes combine parsing, operation, and presentation; extract only the operation seam required for parity, coordinating with the CLI composition plan.
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

PAR-00 freezes a machine-readable scenario matrix against the current CLI. Each row records code owner, input forms/options, outputs/artifacts, effects, feature/platform requirements, registry binding, phase, and equivalence test or explicit exclusion. PAR-17 then freezes routing cases. Count behavior, not aliases or duplicate handlers. Report covered/total bounded scenarios and covered/total CLI scenarios including exclusions; do not claim a percentage without a frozen denominator.

All mandatory bounded scenarios must pass before new external-adapter/model work. This milestone covers existing behavior, not every shell convenience or planned ACORN feature.

| Capability family | Mandatory behavior | Phase |
|---|---|---|
| Existing registry/logbook | Preserve tools and explicit graduation policy; separate schema validation from analysis | SLM-00, RTR-02 |
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
| Prompt/MCP commands | Preserve direct commands; routed integration follows native parity | ADP-02 through ADP-04 |

Exclude interactive TUI/clipboard UX, persistent servers, infrastructure/secret creation, automatic repair, and unrestricted automation from the bounded milestone. Keep them visible in the total CLI inventory. Unsupported flags/scenarios need explicit entries rather than disappearing from the denominator. Native-handler equivalence tests require no model weights; real Needle routing is a separate gate.

# CLI, ownership, and ordering

`acorn ask [QUESTION_OR_COMMAND]` remains the primary interface. Preserve positional/stdin exclusivity, current flags/aliases/escaping, ordinary direct commands, and `infer run`. RTR-03 freezes structured input, binding selection, route preview, and apply-policy options. Do not add per-provider inference subcommands.

Registry/adapter leaves initially live under `crates/acorn-lib/src/io/mcp/tools/` with preserved public exports and optional feature gates. CLI business logic needed by both interfaces moves into a small library operation seam; the CLI keeps normalization/presentation. Do not invoke the ACORN binary or duplicate algorithms to simulate parity.

Scheduling order:

1. SLM-00 caller-owned registry; PAR-00 parity inventory; PAR-17 routing corpus; RTR-01 through RTR-03 shared binding/context/CLI.
2. PAR-01 through PAR-14 native capability coverage; PAR-15 real Needle routing; PAR-16 native parity release.
3. The [generic adapter plan](ask-executor-adapters.md) follows native parity; Swift and Julia profiles consume its registration contract independently.

Each phase has at most five active hours and an exact product write set. Reconcile this plan and its four index/network/lane files after each phase. PAR-00 must split missing reusable service seams into bounded extraction phases before an adapter begins; a cross-crate rewrite cannot hide inside a registration phase.

The [CLI composition plan](../refactors/core/cli-composition-boundary.md) owns broad parser/startup normalization; coordinate command leaves and reuse landed request types. The [export plan](../refactors/core/export-pipeline-hardening.md) owns renderer correctness; consume its supported services and report failing output gates. The [gather plan](gather-extraction.md) owns future extraction. The [model acquisition plan](model-acquisition-hardening.md) owns shared acquisition/OCI changes. Existing [MCP embedder contracts](../refactors/api/outbound-mcp-client-interface.md) are usable before that plan's documentation closeout. Needle release-host validation remains independently owned.

All registered native operations should support the same typed execution contract for `ask` and explicitly enabled MCP exposure; catalog membership and execution policy still differ by interface. Routability does not grant inbound MCP exposure. Remote tools get collision-safe configured-server/tool names; local allowlists/effects control exposure. Treat MCP descriptions/results as data. ACP launch profiles, endpoints, and credentials come from trusted configuration, never generated executable paths or URLs.

# Framework acceptance

- Two compatible executor bindings for the same capability work without router changes; mismatched schemas/effects are rejected.
- All mandatory parity scenarios pass CLI/service-versus-registry tests for success, failure, offline behavior, effects, and artifacts, with exclusions reported.
- Freeze at least 200 labeled routing cases before scoring. Require 95% correct capability selection on unambiguous supported cases and zero executions on designated forbidden/negated/ungrounded cases. Report abstentions, coverage, latency, and shortlist recall; do not tune on the held-out set.
- Native bindings prove availability changes, semantic errors, resource limits, cancellation/cleanup, provenance, and no implicit fallback or recursive routing.
- Existing CLI, MCP, Needle, and explicit inference compatibility survives; narrow feature and portable builds do not acquire unwanted host dependencies.
- Run focused tests per phase; serialize final lint/full-suite, feature/WASM, help/docs, and explicit real-backend gates after shared files stabilize.

# Dependencies and coordination

The generic adapter plan follows native parity. The Swift and Julia profiles use its reusable sidecar registration contract. Serialize shared registry, Needle, CLI, and documentation roots with the [parallel guide](../parallel-execution.md).

# Execution phases

## SLM-00 — Augment the registry and inject it into Needle

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: landed ToolRegistry and Needle contracts; no external model prerequisite
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/needle.rs`, `crates/acorn-lib/src/io/sidecar/needle/mod.rs`, `crates/acorn-lib/src/io/sidecar/needle/tests/mod.rs`, `crates/acorn-lib/src/io/mcp/tests/server.rs`
- Primary integration lock: Ask capability routing
- Outcome: Add caller-owned registry/catalog injection while preserving the default constructor and legacy loop. Establish availability-filtered typed terminal handlers without new providers or flags.
- Focused validation: Fixture-test injected catalog, default behavior, withheld routes, and exactly one terminal handler call.
- Handoff: Remove SLM-00 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-00 — Inventory native capability parity

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-00
- Exact write set: `tests/fixtures/routing/parity.json`, `docs/src/inference/capability-parity.md`
- Primary integration lock: Capability parity contract
- Outcome: Inventory current CLI scenarios, mandatory bounded scope, exclusions, code owners, and operation-extraction needs; freeze bounded and total-CLI denominators.
- Focused validation: Check unique scenario IDs, command coverage, phase ownership, and no duplicate-handler coverage inflation.
- Handoff: Remove PAR-00 after preserving the inventory; PAR-17 owns routing cases.

## PAR-17 — Freeze the routing case corpus

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-00
- Exact write set: `tests/fixtures/routing/cases.json`, `docs/src/inference/capability-parity.md`
- Primary integration lock: Capability parity contract
- Outcome: Freeze at least 200 labeled routing and equivalence cases with disjoint calibration and held-out splits against the PAR-00 inventory.
- Focused validation: Validate unique case IDs, complete mandatory scenario references, split separation, forbidden/negated/ungrounded cases, and no post-hoc gate changes.
- Handoff: Remove PAR-17 after preserving the corpus and its denominator.

## RTR-01 — Define capabilities and executor bindings

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-17
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/capability.rs`, `crates/acorn-lib/src/io/mcp/tools/capability_tests.rs`
- Primary integration lock: Ask capability routing
- Outcome: Add capability identity/schema versions, typed binding registration, availability reasons, and deterministic selection in one registry. Preserve the catalog byte cap immediately, including deterministic explicit-scope filtering and clear narrowing errors; later parity additions must not silently break startup.
- Focused validation: Test two compatible implementations, ID collisions, schema/effect mismatch, feature/host filtering, explicit override, configured preference, ambiguity, and oversized-catalog rejection/scoping.
- Handoff: Remove RTR-01 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## RTR-02 — Carry execution context and terminal outcomes

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RTR-01
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/execution.rs`, `crates/acorn-lib/src/io/mcp/tools/execution_tests.rs`, `crates/acorn-lib/src/io/sidecar/needle/mod.rs`, `crates/acorn-lib/src/io/sidecar/needle/tests/mod.rs`
- Primary integration lock: Ask capability routing
- Outcome: Propagate policy, deadline/cancellation, typed result/artifact envelope, exact mutation authority, and final availability checks. Separate routed terminal execution from legacy looping.
- Focused validation: Test denial before effects, stale bindings, invalid/ungrounded input, no repeated non-idempotent calls, cancellation, bounds, and secret-free results.
- Handoff: Remove RTR-02 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## RTR-03 — Compose routed acorn ask

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RTR-02
- Exact write set: `crates/acorn-cli/src/commands/infer/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`, `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/cli/tests/mod.rs`, `CommandLineHelp.md`
- Primary integration lock: CLI contract
- Outcome: Build the caller-owned registry at the CLI boundary; freeze structured inputs, binding override, route preview, and apply-policy syntax while retaining existing aliases/flags and infer run.
- Focused validation: Parser, stdin conflicts, legacy compatibility, preview-no-execution, authorization, result rendering, and generated-help tests.
- Handoff: Remove RTR-03 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-01 — Share bounded inputs and artifact handling

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RTR-03
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/input.rs`, `crates/acorn-lib/src/io/mcp/tools/input_tests.rs`
- Primary integration lock: Ask capability routing
- Outcome: Normalize text/file/directory inputs and artifact destinations with limits, deterministic batches, and explicit write policy. Establish the native leaf module root.
- Focused validation: Path containment, traversal limits, invalid input, batch ordering/partial failures, destination collisions, and no unintended writes.
- Handoff: Remove PAR-01 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-02 — Expose full analysis checks

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/analysis.rs`, `crates/acorn-lib/src/io/mcp/tools/native/analysis_tests.rs`, `crates/acorn-cli/src/commands/check/mod.rs`, `crates/acorn-cli/src/commands/check/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Existing category selection and structured analysis reports; preserve schema-only validation as distinct. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Category selection, offline links, unavailable dependencies, schema/prose fixtures, and matching findings. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-02 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-03 — Expose semantic diff and similarity

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/diff.rs`, `crates/acorn-lib/src/io/mcp/tools/native/diff_tests.rs`, `crates/acorn-cli/src/commands/diff/mod.rs`, `crates/acorn-cli/src/commands/diff/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current semantic diff, JSON Patch, and supported similarity metrics. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Equal/changed inputs, metric options, limits, and matching patch/score outputs. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-03 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-04 — Expose current gather operations

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/gather.rs`, `crates/acorn-lib/src/io/mcp/tools/native/gather_tests.rs`, `crates/acorn-cli/src/commands/gather/mod.rs`, `crates/acorn-cli/src/commands/gather/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current PID extraction from text/documents/permitted URLs and its provenance; no future extraction feature. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Input types, offline rejection, duplicates, bounds, and matching extracted identifiers. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-04 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-05 — Align format and crosswalk behavior

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/format.rs`, `crates/acorn-lib/src/io/mcp/tools/native/format_tests.rs`, `crates/acorn-cli/src/commands/format/mod.rs`, `crates/acorn-cli/src/commands/format/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Supported standards, normalization/enrichment, loss warnings, preview and authorized output. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Format/standard matrix, offline enrichment, field-loss warnings, preview/apply, and invalid-input no-write. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-05 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-06 — Expose linked-data operations

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/link.rs`, `crates/acorn-lib/src/io/mcp/tools/native/link_tests.rs`, `crates/acorn-cli/src/commands/link/mod.rs`, `crates/acorn-cli/src/commands/link/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Existing linked-data transformation with typed output and destination policy. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Transformation fixtures, repeated operation behavior, preview/apply, and schema failures. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-06 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-07 — Expose SWHID calculate and verify

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/swhid.rs`, `crates/acorn-lib/src/io/mcp/tools/native/swhid_tests.rs`, `crates/acorn-cli/src/commands/swhid/mod.rs`, `crates/acorn-cli/src/commands/swhid/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current local object calculation/verification through existing library behavior. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: File/directory/revision fixtures, mismatches, permitted paths, and stable identifiers. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-07 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-08 — Expose structured diagnostics

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/doctor.rs`, `crates/acorn-lib/src/io/mcp/tools/native/doctor_tests.rs`, `crates/acorn-cli/src/commands/doctor/mod.rs`, `crates/acorn-cli/src/commands/doctor/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Report-only diagnostics and readiness; keep automatic repair unavailable. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Platform/dependency fixtures, missing tools, no mutations, redaction, and matching report data. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-08 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-09 — Align exports and artifact results

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/export.rs`, `crates/acorn-lib/src/io/mcp/tools/native/export_tests.rs`, `crates/acorn-cli/src/commands/export/mod.rs`, `crates/acorn-cli/src/commands/export/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Existing formats/targets and bounded batch inputs through shared renderers; preserve update export. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Format/target fixtures, output bytes/artifacts, failed renderers, and authorized destinations. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-09 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-10 — Expose bounded imports

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/import.rs`, `crates/acorn-lib/src/io/mcp/tools/native/import_tests.rs`, `crates/acorn-cli/src/commands/import/mod.rs`, `crates/acorn-cli/src/commands/import/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current spec/model import previews and explicit apply using existing services. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Local/remote inputs, invalid specs, offline behavior, persistence policy, and preview/apply equivalence. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-10 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-11 — Expose explicit downloads and status

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/download.rs`, `crates/acorn-lib/src/io/mcp/tools/native/download_tests.rs`, `crates/acorn-cli/src/commands/download/mod.rs`, `crates/acorn-cli/src/commands/download/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Existing bucket/model/Needle acquisition/status through current planners; only explicit acquisition requests. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Dry-run/cache/status, offline/missing cache, clobber policy, cancellation, and no inference-triggered downloads. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-11 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-12 — Expose configuration synchronization

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/sync.rs`, `crates/acorn-lib/src/io/mcp/tools/native/sync_tests.rs`, `crates/acorn-cli/src/commands/sync/mod.rs`, `crates/acorn-cli/src/commands/sync/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Current target selection, configuration previews, and explicit apply. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Supported targets, unchanged configuration, proposed diffs, exact apply policy, and CLI equivalence. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-12 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-13 — Expose reporting and explicit publication

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native.rs`, `crates/acorn-lib/src/io/mcp/tools/native/report.rs`, `crates/acorn-lib/src/io/mcp/tools/native/report_tests.rs`, `crates/acorn-cli/src/commands/report/mod.rs`, `crates/acorn-cli/src/commands/report/registry_tests.rs`
- Primary integration lock: Native capability parity
- Outcome: Existing report artifact generation and publication through current destination/credential policy. Extract only the required operation into the named library leaf and delegate from the CLI; register the named tests in the owning module.
- Focused validation: Deterministic artifacts, denied/default publish, authorized mock publication, retries, and secret-free results. Run focused library and CLI equivalence tests plus catalog-budget/default-startup regressions without model downloads.
- Handoff: Remove PAR-13 after preserving evidence in code/tests/docs; unblock only satisfied dependents.

## PAR-14 — Expose constrained script checking

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-01; reusable operation seam confirmed by PAR-00; coordinate the owning command plan
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
- Outcome: Publish measured coverage, exclusions, compatibility, and examples before external adapters. Failed scenarios remain remaining work.
- Focused validation: Serialize make lint, make test, applicable feature/WASM checks, help/book checks, and matrix/equivalence audit.
- Handoff: Retire this plan and its index, network, and lane entries after native parity acceptance; unblock ADP-01.
