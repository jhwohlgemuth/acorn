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

Local evidence checked through 2026-11-02:

- `cli/mod.rs` exposes `ask` as an alias of `infer`; explicit `infer run` already supports configured prompt backends.
- `io/mcp/tools/mod.rs` contains nine built-in definitions with analysis enabled. Check and validate currently use the same schema validator, so names alone overstate behavioral coverage. Graduation apply is not Needle-exposed.
- `ToolRegistry` already supports typed handlers, Needle schema projections, and effect/exposure/output policy. Extend it as the source of truth; its current MCP directory does not require a duplicate registry or crate extraction.
- `io/sidecar/needle/mod.rs` constructs `ToolRegistry::acorn()` during startup, uses a bounded multi-round loop, and caps the catalog at 32 KiB. It already withholds low-confidence, uncalibrated, negated, and ungrounded calls. The caller-owned registry, terminal execution and catalog-size contracts are now implemented, as detailed below; remaining phases concern native behavioral parity and release evidence.
- `io/mcp/client.rs` exposes `McpClientBackend`, configured discovery/calls, allowlists, context, cancellation, and bounded cleanup. A protocol-successful result can still contain `is_error`.
- `agent/mod.rs` supplies `InferenceBackend`, typed requests/results, capabilities, provenance, and `InferencePermissionPolicy::DenyMutation`; ACP and OpenAI-compatible implementations exist. ACP mutation remains unsupported in this plan.
- Analysis has reusable services in `analyzer/host/service/mod.rs`. Other CLI leaves sometimes combine parsing, operation, and presentation; extract only the operation seam required for parity, coordinating with the CLI composition plan.
- The PAR-00 [inventory](../../tests/fixtures/routing/parity.json) freezes 72 CLI behavior rows: 46 bounded and 26 explicit exclusions. The [capability parity guide](../../docs/src/inference/capability-parity.md) records the current 46/46 bounded and 46/72 total evidence and extraction seams, including the versioned export baseline corrections.
- Needle now accepts a caller-owned `ToolRegistry`, filters its catalog by offline and exposure policy, and keeps the default startup path. Focused `needle` tests cover injected handlers, MCP isolation, compact deterministic catalogs, and exact custom catalog validation.
- The routing corpus now has 50 labeled nonexecution cases across forbidden, negated, ungrounded, missing-input, and ambiguous requests. Each class has five calibration and five held-out prompts, all linked to inventory rows with zero expected effects.
- The corpus also has 50 grounded read-only cases covering check, diff, gather, doctor, SWHID, and bounded script checks. Expected capability IDs and existing fixture paths are recorded with an even calibration/held-out split.
- The corpus now includes 50 effect and artifact cases across format, link, export, import, download, sync, report, and script checks. Each captures preview or exact caller apply policy, expected effects, and artifact references; denied apply requests expect no execution. PAR-21 completes selection boundaries before the freeze.
- The corpus has 200 unique prompts with 100 calibration and 100 held-out labels. The final 50 cover explicit binding, availability, offline policy, equivalent binding, and remaining mandatory input forms; every bounded inventory row is referenced. PAR-17 audited the split, safety nonexecution, references, input paths, and expected authority, then recorded file hashes in the parity guide. The corpus is frozen before router scoring.
- Corpus revision 3 corrects generic capability, caller-policy, and native binding IDs to registered IDs before real Needle scoring. Prompts, case IDs, splits, and effect expectations remain fixed; the 200 cases retain a 100/100 split. Its SHA-256 is `d87c85d47df150fdb3aeaf18a90e5f8a403778eeac5d6f0ba62883cdb8045596`.
- `ToolRegistry` now owns versioned capability definitions and typed executor bindings. It rejects duplicate IDs and schema/effect/version mismatches, records availability reasons, resolves explicit/preferred/native/local/remote bindings deterministically, and serializes only eligible capabilities under the 32-KiB catalog cap. Five focused library tests pass with the `needle` feature.
- Routed execution now rechecks a selected binding, grounding, schema, offline policy, exact operation/argument/destination authority, limits, deadline, and cancellation before one handler call. It returns typed outcomes, bounded artifact references, and credential-free provenance; cancellation drops the handler future and marks effects as possibly started. Needle has a separate one-completion terminal route while its legacy inference loop remains available. Six focused executor tests and the terminal Needle response test pass.
- `ask`/`infer --route` now compose a caller-owned registry with a first native version capability. Exact JSON input, capability scope, binding override, route preview, and destination-bound apply flags are frozen. The default one-shot path and `infer run` remain available while native leaves are added. Focused route and inference regression tests, catalog checks, and regenerated command help pass.
- Shared native input policy now resolves bounded text, file, and directory references under a canonical root, rejects traversal and escaping symlinks, sorts batches, and reports per-entry failures. Artifact policy validates exact destinations, collision and overwrite authority without writing. Four focused input tests pass; `native/mod.rs` is the leaf module root for subsequent operations.
- The `acorn.check` v1 native binding calls `analyze_paths` and returns standard-specific category statuses and structured findings. It accepts bounded local, HTTP(S), Git branch/commit, GitLab merge-request, and OCI selections; filters and offline policy are checked. The configured `acorn.check_fixes` binding uses the shared `AnalysisReport::suggest_fixes` service and produces nonexecuting unified previews. Eleven focused library tests and eight direct CLI equivalence tests pass, including a positive configured fix proposal and preview with an unchanged source file. The eight PAR-02 inventory rows have passing evidence.
- The `acorn.diff` v1 native binding shares semantic diff and similarity operations with the direct CLI. Three focused library tests and four direct CLI equivalence tests pass across unified output, JSON Patch, file similarity, and literal similarity. The four PAR-03 inventory rows have passing evidence.
- Bounded native gather routes now extract the same discovery records as the direct command from text, files, directories, and permitted HTTP(S) documents. Separate local/remote read and authorized report-write routes preserve source provenance; report artifacts use the CLI JSON serializer. Focused library and direct CLI tests cover duplicate PIDs, bounds, offline rejection, remote selection, report shape, and exact write authority. The four PAR-04 inventory rows have passing evidence.
- Native format preview/apply and crosswalk preview/apply routes use the shared input and artifact policies. The CLI and native plain format path share one normalization function, and ACRE shares one syntax-preserving formatter; enrichment uses the existing provider pipeline. Focused tests cover six supported standards, provider/offline policy, loss warnings, preview/no-write behavior, authorized writes, and direct CLI equivalence. Catalog projection retains canonical execution schemas while omitting redundant schema prose from Needle input; the default catalog stays below 32 KiB. The four PAR-05 inventory rows have passing evidence.
- Native linked-data preview and exact companion-artifact apply reuse the `ResearchActivity` projection already called by the direct link workflow. Focused library tests cover bounded input, invalid schema, denied/default writes, exact authorization, and unchanged source content; direct CLI equivalence covers dry run and output bytes. The `link.transform` row now has passing evidence without bypassing configured link hooks.
- Native SWHID calculation and verification use the direct command's `io::swhid` operation with bounded local paths. Focused tests cover local file/directory identifiers, expected identifier matching and mismatch, and traversal rejection; direct CLI equivalence covers content, directory, revision, and successful verification. The four PAR-07 inventory rows have passing evidence.
- Native bucket download uses the direct command's shared source resolution and transfer operation. Focused tests cover exact mutation authority, verified local BagIt transfer, offline configured remote denial, path containment, and byte-identical direct CLI output; `download.bucket` has passing evidence.
- Native model preview and explicit acquisition use the direct command's model selector planner and transfer operation. Focused tests compare local copy and mocked HTTP bytes with the direct CLI, reject offline remote selection, and prove no-write preview, exact output authority, cancellation, overwrite policy, and containment. The `download.model` and `download.preview` rows have passing evidence.
- Feature-gated native static script and hook-manifest checks reuse `Resolved::direct` and `ScriptManifest::read`. Focused library and direct CLI tests cover valid and invalid script sources, exact JSON status, manifest summary, local path containment, read-only effects, and zero artifacts. `exec.check_safe` and `exec.manifest` have passing evidence; evaluated checks remain separate.
- The feature-gated `acorn.exec_check` and `acorn.exec_hook_check` routes reuse shared restricted evaluators with bounded local script and input files. Focused tests compare direct command findings and failure counts, confirm unchanged inputs, require digest authorization, reject offline network capabilities, and return zero applied artifacts. Both rows have passing evidence.
- The read-only `acorn.doctor` native binding uses `Records::diagnose_host` and returns the same structured JSON array as `doctor --report`. Focused library and direct CLI tests cover category selection, report fields, unknown categories, rejected repair input, and zero artifacts. The `doctor.report` row has passing evidence; automatic repair remains excluded.
- Schema export now uses one `io::export::schema_text` operation from the direct CLI and the read-only native binding. Focused tests cover every modeled standard in JSON and YAML, unsupported formats, exact CLI output, and no artifact. The `export.schema` row has passing evidence after correcting its baseline to stdout text; the original and revised inventory/corpus hashes remain in the parity guide.
- PAR-09 has a bounded, exact-authority JSONL batch binding over `ResearchActivity::batch_read`, `Batch::batch_write`, and the shared `StandardCrosswalk` conversion. Library tests prove denied/default writes and source preservation; direct CLI tests prove byte-identical plain and selected crosswalk JSONL output. The `export.batch` row has passing evidence.
- The BagIt binding delegates to the existing `Bag::save_as` and verification service. A focused test proves default write denial, source preservation, staging outside the source, and ZIP/TAR/TAR.GZ/7z outputs; a direct CLI test proves byte-identical ZIP archives. The `export.bag` row has passing evidence.
- PAR-25 closes `export.preview` and `export.render` with read-only archive, document, graph, PowerPoint, and updates previews and exact-authority render/crosswalk bindings. Focused tests compare direct CLI text and graph bytes, PowerPoint parts, valid PDFs, denied writes, and unchanged source files. Direct PowerPoint and updates `--dry-run` now leave no artifacts.
- The unscoped native catalog is over Needle's 32-KiB input limit after export registrations. PAR-15 now selects a deterministic prompt shortlist under 22 KiB. Its first 200-case real Needle preview failed the selection gate: 163 tool failures, 0/75 accepted correct previews in each split, and 73/75 calibration versus 71/75 held-out shortlist retention under the initial online-built registry. The [evaluation](../../docs/src/inference/routing-evaluation.md) records the limitations and diagnostic replays. Complex tool schemas can exhaust Needle's call budget even with a single capability; a bounded repair precedes a valid rerun.
- OpenAPI specification import now has read-only preview and exact-authority apply bindings over a shared endpoint projection. Focused tests cover local and mocked HTTP input, invalid specifications, offline rejection, bounded paths, denied writes, matching direct CLI JSON, and unchanged source files. The `import.spec` row has passing evidence.
- Hugging Face metadata import now shares one GGUF resolver between the direct CLI and native preview/apply routes. A loopback fixture verifies matching metadata count and persisted database weights, offline denial, exact database authority, and no model-weight requests. The six import corpus prompts now use real endpoint and metadata semantics; corpus revision 3 retains 200 unique cases and a 100/100 split.
- Configuration synchronization now exposes the shared target renderer as a structured preview and uses the same coordinated commit as the direct CLI. Focused tests compare OpenCode direct and native configuration text, preview Goose, llama-swap, and VS Code, require exact destination authority, and preserve model files. The two sync rows have passing evidence.
- The legacy one-shot `ask` row has focused direct-command and shared Needle service equivalence on the pinned offline runner, including structured version results, missing prompt rejection, and no working-directory artifacts.
- PDF rendering has a feature-gated shared Chromium session and HTML-to-PDF operation in `acorn-lib`; direct and native exports delegate to it. Focused tests render valid direct and native PDFs with the configured Chrome executable, verify source preservation, and reject an invalid executable.
- PowerPoint reference-deck rendering now lives in `acorn-lib` with explicit source, output, chart, and fallback-image inputs. The direct command delegates to it; report slides reuse its staged OOXML helpers. A focused test compares direct and shared slide parts, verifies a two-input deck, and preserves the source. The existing 13 PowerPoint helper tests and the 10-test export registry suite pass.
- Since-filtered updates projection, artifact naming, DOCX, and Markdown generation now live in `acorn-lib`; the direct command delegates to them. Focused direct/shared tests compare exact artifact bytes for both formats, cover a missing logbook and invalid timestamps, and preserve source bytes. The shared HTML/PDF seam consumes this projection.
- Updates HTML page composition now lives in `acorn-lib` with explicit embedded assets supplied by the CLI adapter. Shared and CLI project, title, and legend HTML match; the direct command and shared Chromium renderer produce valid updates PDFs. The 12-test export registry suite, eight updates HTML tests, strict library Clippy, and no-default CLI check pass.
- Pinned Needle cache status and explicit acquisition use the same verified asset resolver as `download needle`. Focused tests prove read-only offline status, exact cache destination authority, alternate destination rejection, and direct CLI runner equivalence. The local macOS ARM64 cache contains the pinned runner for eventual PAR-15 scoring.
- Native report generation and publication use the direct command's shared project projection and manifest writer, Chromium PDF renderer, immutable GitLab package service, and callback queue. Focused tests compare direct/native project and legend PDFs, manifest fields, exact local and publication authority, and offline rejection; mocked package tests cover ordered publication, checksum checks, no-change ledgers, and identical-content retries. The `report.local` and `report.publish` rows have passing evidence.
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
| Gather | Existing PID extraction from text/documents/permitted URLs | Landed |
| Format/crosswalk | Supported standards, normalization/enrichment, loss warnings, preview/apply | Landed |
| Linked data | Existing transformation and authorized output | Landed |
| SWHID | Current calculate/verify over permitted inputs | Landed |
| Doctor | Structured diagnostics/readiness; no automatic repair | Landed |
| Export | Existing formats/targets, bounded batches, artifact output | Landed |
| Import | OpenAPI and Hugging Face metadata preview and explicit apply landed | Landed |
| Download | Bucket, model, and pinned Needle acquisition/status landed | Landed |
| Sync | Local target configuration preview and exact apply landed | Landed |
| Report | Existing artifact generation and explicit publication landed | Landed |
| Script checks | Static, direct, and manifest-authorized hook checks landed | Landed |
| Prompt/MCP commands | Preserve direct commands; configured MCP, ACP, and endpoint adapters are landed, and heterogeneous release follows native parity | ADP-05 and ADP-09 |

Exclude interactive TUI/clipboard UX, persistent servers, infrastructure/secret creation, automatic repair, and unrestricted automation from the bounded milestone. Keep them visible in the total CLI inventory. Unsupported flags/scenarios need explicit entries rather than disappearing from the denominator. Native-handler equivalence tests require no model weights; real Needle routing is a separate gate.

# CLI, ownership, and ordering

`acorn ask [QUESTION_OR_COMMAND]` remains the primary interface. Preserve positional/stdin exclusivity, current flags/aliases/escaping, ordinary direct commands, and `infer run`. The landed `--route` mode freezes structured input, binding selection, route preview, and apply-policy options while native bindings are added. Do not add per-provider inference subcommands.

Registry/adapter leaves initially live under `crates/acorn-lib/src/io/mcp/tools/` with preserved public exports and optional feature gates. CLI business logic needed by both interfaces moves into a small library operation seam; the CLI keeps normalization/presentation. Do not invoke the ACORN binary or duplicate algorithms to simulate parity.

Scheduling order:

1. The caller-owned Needle registry, frozen 200-case corpus, versioned capability/binding contract, terminal execution, opt-in CLI route syntax, bounded native input policy, and check/fix/diff bindings have landed.
2. PAR-34 repairs the failed model-facing routing and harness; PAR-15 reruns real Needle routing on corpus revision 3; PAR-16 validates the native parity release.
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

## PAR-34 — Repair model-facing routing and corpus harness

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: First PAR-15 measurement and diagnostic evidence
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/capability.rs`, `crates/acorn-lib/src/io/mcp/tools/needle.rs`, `crates/acorn-lib/src/io/sidecar/needle/mod.rs`, `crates/acorn-lib/src/io/sidecar/needle/process.rs`, `crates/acorn-cli/src/commands/infer/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/routing_scope.rs`, `tests/fixtures/routing/results.json`, `docs/src/inference/routing-evaluation.md`
- Primary integration lock: Needle routing evaluation
- Outcome: Preserve canonical validation while making complex native tool calls fit the pinned runner; apply each case's offline, caller-policy, and binding context in the measurement harness.
- Focused validation: Single-capability check and doctor replays, calibration selection and shortlist recall, response-error capture, negative-case execution safety, no artifact writes, and strict Clippy.
- Handoff: Remove PAR-34 after the repaired harness and model-facing catalog pass focused calibration; keep PAR-15's held-out gate separate.

## PAR-15 — Prove routing at near-parity catalog size

- Status: waiting-on-validation
- Budget: 5 active hours maximum
- Prerequisites: PAR-34 repair accepted; pinned Needle runner installed on a supported host
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/capability.rs`, `crates/acorn-lib/src/io/mcp/tools/needle.rs`, `crates/acorn-lib/src/io/sidecar/needle/mod.rs`, `crates/acorn-lib/src/io/sidecar/needle/assets.rs`, `crates/acorn-lib/src/io/sidecar/needle/tests/mod.rs`, `crates/acorn-cli/src/commands/infer/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`, `tests/fixtures/routing/cases.json`, `tests/fixtures/routing/results.json`, `docs/src/inference/capability-parity.md`, `docs/src/inference/routing-evaluation.md`
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
