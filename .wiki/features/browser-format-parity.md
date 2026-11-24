---
type: Plan
title: Browser and WebMCP metadata formatting parity
description: Ship in-memory Research Activity formatting quickly, then extend portable standards and reconcile the shared page with reviewed desktop format capabilities.
difficulty: high
impact: high
package: [acorn-wasm, acorn-web, acorn-schema, acorn-lib, acorn-desktop]
tags: [feature, format, browser, webmcp, wasm, desktop, metadata]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Add a `/format` page and `acorn_format_metadata` WebMCP tool over one in-memory formatting operation. Deliver useful Research Activity Data (RAD) formatting first. Expand toward every deterministic, content-only behavior of `acorn format` while keeping native file context, enrichment, directory traversal, and writes behind the future desktop host boundary. The browser page remains usable without WebMCP; WebMCP never receives a local path or desktop write authority.

This is one format operation, not a second crosswalk or generic pretty printer. It parses a declared standard and input syntax, applies that standard's normalization rules, and returns formatted content in the same syntax. Conversion between standards belongs to Crosswalk; rendering PDF/PPTX/DOCX/BagIt belongs to Export.

# Current evidence — November 24, 2026

- At baseline `2369335b` with concurrent uncommitted browser and wiki work, `crates/acorn-wasm/src/lib.rs` exports validation, crosswalk, diff, identifiers, FAIR, readability, and Big-O operations, but no format binding. `crates/acorn-wasm/web/webmcp.js` registers nine read-only tools and enforces a 256 KiB serialized result limit (`resultEnvelope`, lines 255–260). Its TypeScript contract and Node tests cover that adapter.
- `crates/acorn-web/src/operations.rs` declares the nine browser `Tool` variants, bounded `Request`, and worker dispatch. `crates/acorn-web/src/ui.rs` routes tool slugs through `ToolWorkspace`, with a loaded-file editor, and `crates/acorn-web/src/results.rs` plus `presentation.rs` provide result rendering and content-only export. Existing dirty edits in these files are owned by concurrent work and must be coordinated before implementation.
- `acorn_schema::codec::DocumentCodec` decodes and encodes supplied text without host I/O (`crates/acorn-schema/src/codec.rs`, lines 30–36). RAD supports JSON, JSONC, Markdown, YAML, and ZON (`codec.rs`, lines 120–180), and `ResearchActivity::format_with_vocabulary_report` provides portable vocabulary normalization (`research_activity/mod.rs`, lines 1032–1046).
- The direct native format route currently supports ACRE, CFF, DataCite, Dublin Core, Invenio, and RAD (`crates/acorn-lib/src/io/mcp/tools/native/format.rs`, lines 55–69 and 340–363). Its `normalize_metadata_file` reads a path and calls the host `InputOutput` trait (`format.rs`, lines 681–700). Native RAD additionally scans neighboring image files (`crates/acorn-lib/src/io/adapter/mod.rs`, lines 581–600 and 809–846). CLI `format` defaults to strict vocabulary matching (`crates/acorn-cli/src/cli/arguments/mod.rs`, lines 254–264).
- The [desktop workspace plan](acorn-browser-workspace/index.md#abw-50--review-local-creation-and-mutation-workflows) assigns host-only `format` preview, confirmation, freshness checks, and bounded writes to ABW-50 after the companion boundary in ABW-45. `crates/acorn-desktop/` is absent at this baseline. ABW-47 owns export formats; its CFF/JSON/YAML output list is not a promise that the new formatter supports those standards.

These are inspected source and plan observations, not passing test results for the proposed format operation.

# Tiered delivery and contract

| Tier | Outcome | Earliest useful handoff |
|---|---|---|
| 0 — fast path | RAD JSON formatting, then the read-only WebMCP tool and `/format` page | FMT-01 → FMT-02 makes the agent tool usable; FMT-03 makes the browser page usable. Neither waits for desktop. |
| 1 — browser quality | One result contract, safe review/export, explicit limits and live smoke | FMT-04 accepts the initial browser operation without claiming CLI parity. |
| 2 — portable breadth | Additional RAD syntaxes, then CFF/DataCite/Invenio, Dublin Core, and ACRE where portable behavior is proven | FMT-05–08 expand the exact supported matrix without exposing all validation standards as format standards. |
| 3 — maximum parity | Compare portable results with CLI/native fixtures; integrate the same page with ABW-50 desktop preview/apply; accept cross-surface behavior | FMT-09–10 close evidenced gaps. Host-only behavior remains desktop-only. |

Initial public request: `{ content: string, standard: "research-activity-data", format: "json", fuzzy?: "off" | "relaxed" | "strict" }`. Default `fuzzy` to `strict`, matching `acorn format`. Initial success: `{ content: string, changed: boolean, format: "json", standard: "research-activity-data", warnings: [] }`. A later version may add bounded normalization reports as `warnings`; changing the shape requires synchronized Rust, JavaScript, TypeScript, browser, and test edits. `changed` compares original supplied bytes with returned bytes. Parse or unsupported-combination failures must be structured and must leave supplied text intact. No write occurs on invocation.

The WebMCP result is the replacement content once, not `{original, content, diff}`. Keep the existing 256 KiB envelope bound and choose a conservative input bound plus post-format output check; formatting can expand text and JSON escaping can expand the envelope. The browser may calculate a separate unified diff for display, but the user can always inspect/copy/download the exact returned text. Browser save-to-workspace requires a separately clicked action and source-freshness check. Native disk apply belongs only to ABW-50. Cancellation is checked before and after synchronous WASM work; it cannot interrupt a running synchronous formatter without a separate worker/cancellable-core change.

## Intended support matrix

| Standard | Tier-0 syntax | Candidate portable syntaxes | Native-only difference to record |
|---|---|---|---|
| RAD | JSON | JSONC, Markdown, YAML, ZON after fixture parity | Neighboring-image context; optional provider enrichment; path/directory input. |
| CFF | — | CFF/YAML and JSON if the existing codec and CLI accept the exact pair | Provider enrichment and local-file naming. |
| DataCite | — | JSON and YAML only after codec/CLI fixture comparison | Provider enrichment and local-file selection. |
| Invenio | — | JSON and YAML only after wrapper-shape fixture comparison | Provider enrichment and local-file selection. |
| Dublin Core | — | Only representations proven by its portable codec and native format route | Host parser/serializer differences and RDF representation support must be measured, not assumed. |
| ACRE | — | JSON, JSONC, YAML, RON only if a portable configuration formatter can be extracted without changing comments or semantics | Configuration discovery, file names, and native formatting context. |

Unsupported pairs remain absent from selector choices and are rejected by the binding. `dcat`, `document`, `huwise`, `kitfile`, `modelkit`, `raid`, and `text` are validation/export standards, not currently direct native format-route promises. Adding one requires its own source-backed matrix row and bounded phase. Do not claim byte identity with the CLI where the CLI uses file context, dprint/pretty_yaml, or a different serializer until a fixture comparison passes.

# Dependencies and ownership

- FMT-01–04 use existing portable and browser boundaries and do not wait for ABW-11/45/50, hosted deployment, external WebMCP evaluators, or a native OS. Coordinate the exact `acorn-web`/`acorn-wasm` write sets with active ABW, BWP, USW, frontend-quality, and WME owners. No workspace-wide formatting while these edits overlap.
- [WebMCP interoperability](../testing/webmcp-evaluations.md) owns independent evaluation tooling and model-selection evidence. This plan owns new tool contract tests, native adapter smoke, and page acceptance. After FMT-02, WME's checked-in evaluation matrix should add the tenth tool in its next eligible phase; it does not gate initial delivery.
- FMT-05–08 may edit shared schema, codec, or native formatting leaves only after confirming current owners and freezing fixture parity. Their sequence is deliberate: land portable normalization semantics before extending the public selector. Do not add `acorn-lib` or `acorn-host` to `acorn-wasm` or grant WebMCP Tauri IPC.
- ABW-45 and ABW-50 own the desktop companion and reviewed local format capability. FMT-09 consumes their accepted contract; it may adapt the shared page to expose desktop-only actions, but it does not implement native path routing or write commands. If ABW-50 has not landed, FMT-09 remains planned while the browser tiers can still complete.
- The separate browser-to-React handoff in ABW-69 must carry the `/format` route, request/result schema, and browser-versus-desktop effect boundary if FMT-03 has landed. It does not gate the current Dioxus page.

# Acceptance criteria

1. After FMT-02, a supported WebMCP context registers one additional read-only `acorn_format_metadata` tool; unsupported contexts leave the manual app available. Strict schema, UTF-8 byte bounds, output envelope, busy, cancellation, disposal, invalid syntax, unsupported pair, and no-side-effect tests pass.
2. After FMT-03, `/format` runs the same in-memory WASM function as WebMCP. Load/paste/example, settings, unchanged result, parse error, copy/download, and explicit workspace-save behavior are visible and keyboard operable. Navigation retains independent drafts and preserves user content after failure.
3. The initial claim is RAD JSON with strict-default normalization. For each later pair, record a valid fixture, invalid fixture, idempotence check, exact output syntax, and CLI/native comparison with any documented contextual difference. Never broaden a selector from the validation matrix alone.
4. Browser WebMCP cannot select a local path, read a repository or neighbor file, invoke a desktop command, enrich through providers, or apply writes. Desktop local format is available only after ABW-50's explicit source/destination and confirmation contract passes; a desktop without that capability keeps the in-memory operation.
5. Focused Rust, Node adapter, real-WASM, and built-browser checks pass on a stable tree. Root and prefixed static-host routes, output-size edges, and at least one supported real-Chrome WebMCP invocation have recorded evidence. Record measured bundle and latency changes before claiming maximum portable parity.

# Execution phases

## FMT-01 — Export bounded RAD JSON normalization from WASM

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: no concurrent `acorn-wasm` binding or schema-root owner
- Exact write set: `crates/acorn-wasm/src/lib.rs`, `crates/acorn-wasm/src/tests.rs`
- Primary integration lock: WASM format binding
- Outcome: Add a pure `formatMetadata` binding for RAD JSON using `DocumentCodec` and `format_with_vocabulary_report`; default to strict, return the agreed result, and reject invalid/oversized input without mutation or network use. Keep initial DTO small and explicit.
- Focused validation: `cargo test -p acorn-wasm`; `cargo check -p acorn-wasm --target wasm32-unknown-unknown`; valid/invalid fixture, strict/off policy, idempotence, and boundary-size tests.
- Handoff: Record the exact Rust/JS DTO and supported pair; make FMT-02 eligible.

## FMT-02 — Register the first format WebMCP tool

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: FMT-01; adapter owner available
- Exact write set: `crates/acorn-wasm/web/webmcp.js`, `crates/acorn-wasm/web/webmcp.d.ts`, `crates/acorn-wasm/web/webmcp.test.mjs`, `crates/acorn-wasm/examples/webmcp/smoke.js`
- Primary integration lock: WebMCP adapter contract
- Outcome: Register `acorn_format_metadata` with a strict RAD/JSON-only schema, read-only annotations, structured error mapping, result-shape validation, and no original/diff duplication. Update the demo smoke for the new tool count and a real invocation.
- Focused validation: `make test-webmcp`; fake-context tests for registration/unsupported host, invalid keys/pair, byte and envelope bounds, busy/cancel/dispose, and result shape; native demo smoke when a supported Chrome is available.
- Handoff: Publish the tenth-tool contract to FMT-03 and WME; no page or desktop dependency.

## FMT-03 — Add the browser Format page

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: FMT-01; exclusive shared browser UI/operation write set
- Exact write set: `crates/acorn-web/src/operations.rs`, `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/results.rs`, `crates/acorn-web/src/presentation.rs`, `crates/acorn-web/src/window_sync.rs`, `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/src/tests/mod.rs`
- Primary integration lock: browser tool route and result contract
- Outcome: Add `Tool::Format`, `/format`, navigation, independent form state, RAD/JSON and fuzzy controls, example, exact formatted-code result, explicit copy/download, and optional user-clicked save to an unchanged selected workspace source. Reuse the operation worker and existing content-only effect boundary.
- Focused validation: `cargo test -p acorn-web`; `make web-check`; focused browser smoke for direct route, unchanged/changed/error, loaded file, stale-source save denial, reload, keyboard and narrow layout.
- Handoff: Record operation/route shape and any shared UI conflicts; make FMT-04 eligible.

## FMT-04 — Accept and document the initial browser delivery

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: FMT-02 and FMT-03 on a stable browser tree; no concurrent CI/build owner
- Exact write set: `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/README.md`, `crates/acorn-wasm/examples/webmcp/README.md`, `crates/acorn-cli/assets/skills/acorn/SKILL.md`, `Makefile`, `.wiki/features/browser-format-parity.md`
- Primary integration lock: format release and build acceptance
- Outcome: Add the tenth operation to browser and package smoke, update documented support and the release-matched ACORN skill, and record the exact initial browser/WebMCP limits. Keep independent WME evaluator ownership.
- Focused validation: `make build-wasm`, `make test-webmcp`, `make web-test`, root and `/acorn/` `make web-test-pages`; real supported-Chrome tool invocation and no-WebMCP manual route. Serialize broad gates after concurrent browser edits settle.
- Handoff: Preserve dated test/size evidence; remove accepted Tier-0/1 phases and make FMT-05 eligible.

## FMT-05 — Extend RAD to JSONC and YAML

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: FMT-04; no schema codec or WebMCP adapter owner
- Exact write set: `crates/acorn-schema/src/codec.rs`, `crates/acorn-schema/src/research_activity/mod.rs`, `crates/acorn-wasm/src/lib.rs`, `crates/acorn-wasm/src/tests.rs`, `crates/acorn-wasm/web/webmcp.js`, `crates/acorn-wasm/web/webmcp.d.ts`, `crates/acorn-wasm/web/webmcp.test.mjs`, `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/operations.rs`
- Primary integration lock: RAD portable syntax matrix
- Outcome: Admit RAD JSONC and YAML only where decode/normalize/encode round trips preserve documented semantics; retain exact source syntax and surface comment-loss caveats for JSONC.
- Focused validation: per-format valid/invalid/idempotent fixtures, CLI output comparison, Rust/WASM tests, Node adapter schema tests, and browser selection smoke.
- Handoff: Record accepted pairs and unresolved discrepancies; do not mark an unproven pair supported.

## FMT-05B — Extend RAD to Markdown and ZON

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: FMT-05; no schema codec or WebMCP adapter owner
- Exact write set: `crates/acorn-schema/src/codec.rs`, `crates/acorn-schema/src/research_activity/mod.rs`, `crates/acorn-wasm/src/lib.rs`, `crates/acorn-wasm/src/tests.rs`, `crates/acorn-wasm/web/webmcp.js`, `crates/acorn-wasm/web/webmcp.d.ts`, `crates/acorn-wasm/web/webmcp.test.mjs`, `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/operations.rs`
- Primary integration lock: RAD Markdown/ZON format contract
- Outcome: Add Markdown and ZON only after the existing portable codecs prove useful idempotence and preserve metadata meaning; report any unavoidable source-layout differences from CLI output.
- Focused validation: valid/invalid/idempotent fixtures for each pair, CLI comparison, Rust/WASM tests, Node adapter tests, and browser selection smoke.
- Handoff: Record accepted pairs and caveats; keep unsupported cases out of the selectors.

## FMT-06 — Share deterministic CFF formatting

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: FMT-04; accepted per-standard fixture/serialization matrix and exclusive native-format/schema owner
- Exact write set: `crates/acorn-schema/src/codec.rs`, `crates/acorn-lib/src/io/mcp/tools/native/format.rs`, `crates/acorn-wasm/src/lib.rs`, `crates/acorn-wasm/src/tests.rs`, `crates/acorn-wasm/web/webmcp.js`, `crates/acorn-wasm/web/webmcp.d.ts`, `crates/acorn-wasm/web/webmcp.test.mjs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/src/ui.rs`
- Primary integration lock: typed standard formatter semantics
- Outcome: Extract or reuse the same pure decode/normalize/encode behavior for CFF; enable only tested CFF/YAML and JSON syntax pairs. Keep enrichment outside WASM.
- Focused validation: native CLI/MCP versus WASM fixture bytes or documented semantic equivalence; malformed/wrapper-shape, idempotence, size, and adapter/browser selection tests.
- Handoff: Record the authoritative implementation and accepted matrix; leave unsupported pairs disabled.

## FMT-06B — Share deterministic DataCite formatting

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: FMT-06; accepted DataCite fixture matrix and exclusive native-format/schema owner
- Exact write set: `crates/acorn-schema/src/codec.rs`, `crates/acorn-lib/src/io/mcp/tools/native/format.rs`, `crates/acorn-wasm/src/lib.rs`, `crates/acorn-wasm/src/tests.rs`, `crates/acorn-wasm/web/webmcp.js`, `crates/acorn-wasm/web/webmcp.d.ts`, `crates/acorn-wasm/web/webmcp.test.mjs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/src/ui.rs`
- Primary integration lock: DataCite format contract
- Outcome: Add only native-comparable DataCite JSON/YAML pairs; preserve typed parsing and output shape without enrichment.
- Focused validation: native/WASM fixture comparison, invalid and idempotent cases, size bound, adapter schema and browser selector.
- Handoff: Record accepted pairs and any serialization difference.

## FMT-06C — Share deterministic Invenio formatting

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: FMT-06B; accepted Invenio fixture matrix and exclusive native-format/schema owner
- Exact write set: `crates/acorn-schema/src/codec.rs`, `crates/acorn-lib/src/io/mcp/tools/native/format.rs`, `crates/acorn-wasm/src/lib.rs`, `crates/acorn-wasm/src/tests.rs`, `crates/acorn-wasm/web/webmcp.js`, `crates/acorn-wasm/web/webmcp.d.ts`, `crates/acorn-wasm/web/webmcp.test.mjs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/src/ui.rs`
- Primary integration lock: Invenio format contract
- Outcome: Add only native-comparable Invenio JSON/YAML pairs; preserve its native wrapper shape and keep enrichment outside WASM.
- Focused validation: wrapper and native/WASM fixture comparison, invalid and idempotent cases, size bound, adapter schema and browser selector.
- Handoff: Record accepted pairs and any serialization difference.

## FMT-07 — Add proven structured Dublin Core formatting

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: FMT-04; Dublin Core codec/native fixture matrix inspected; exclusive standards owner
- Exact write set: `crates/acorn-schema/src/standard/dublin_core/mod.rs`, `crates/acorn-schema/src/codec.rs`, `crates/acorn-lib/src/io/mcp/tools/native/format.rs`, `crates/acorn-wasm/src/lib.rs`, `crates/acorn-wasm/src/tests.rs`, `crates/acorn-wasm/web/webmcp.js`, `crates/acorn-wasm/web/webmcp.d.ts`, `crates/acorn-wasm/web/webmcp.test.mjs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/src/ui.rs`
- Primary integration lock: Dublin Core representation contract
- Outcome: Compare native normalization and portable codec for JSON, JSON-LD, and YAML candidates; expose each pair only after exact or explicitly documented semantic parity. Keep crosswalk conversion separate.
- Focused validation: structured representation round trips, invalid JSON/YAML, idempotence, native comparison, result bound and browser selector tests.
- Handoff: Record supported structured representations and discrepancies before FMT-07B.

## FMT-07B — Add proven RDF Dublin Core formatting

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: FMT-07; exclusive Dublin Core codec/adapter owner
- Exact write set: `crates/acorn-schema/src/standard/dublin_core/mod.rs`, `crates/acorn-schema/src/codec.rs`, `crates/acorn-lib/src/io/mcp/tools/native/format.rs`, `crates/acorn-wasm/src/lib.rs`, `crates/acorn-wasm/src/tests.rs`, `crates/acorn-wasm/web/webmcp.js`, `crates/acorn-wasm/web/webmcp.d.ts`, `crates/acorn-wasm/web/webmcp.test.mjs`, `crates/acorn-web/src/operations.rs`, `crates/acorn-web/src/ui.rs`
- Primary integration lock: Dublin Core RDF representation contract
- Outcome: Compare RDF/XML and Turtle separately against native formatting; expose only pairs that preserve meaning and have stable portable encoding. Do not imply that a crosswalkable RDF representation is necessarily a format-route representation.
- Focused validation: RDF/XML and Turtle valid/invalid/idempotent fixtures, native comparison, output bound and browser selector checks.
- Handoff: Record supported RDF representations or a precise native-only limitation.

## FMT-08 — Decide ACRE formatting portability

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: FMT-04; ACRE native formatter and configuration semantics inspected; exclusive ACRE/config owner
- Exact write set: `.wiki/features/browser-format-parity.md`, `crates/acorn-lib/src/io/mcp/tools/tests/format.rs`
- Primary integration lock: ACRE format portability decision
- Outcome: Determine which ACRE JSON/JSONC/YAML/RON pairs can move to a portable leaf without changing comments, configuration meaning, or native output. Record the exact destination crate, function contract, required dependencies, and pair-level fixtures. If no pair is portable, retain ACRE as a documented native-only row. If a pair is portable, add one bounded implementation phase with its proven exact write set before starting it.
- Focused validation: native ACRE fixtures, comment and RON round trips, malformed configuration, output-size and semantics comparison; WASM target check for any extracted dependency.
- Handoff: Record a verified portable matrix or a specific native-only reason; add and link a bounded implementation phase for each supported pair before the final parity gate.

## FMT-09 — Reconcile the shared page with desktop local format

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-45 and ABW-50 accepted; FMT-03; exclusive browser/desktop UI owner
- Exact write set: `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/results.rs`, `crates/acorn-web/src/presentation.rs`, `crates/acorn-web/src/host.rs`, `crates/acorn-web/src/host/local.rs`, `crates/acorn-desktop/acceptance/capability-matrix.md`, `crates/acorn-web/README.md`
- Primary integration lock: shared browser/desktop format presentation
- Outcome: Show in-memory preview in both deliveries and, only in desktop, the ABW-50 selected-file preview/apply affordance with visible source, destination, freshness, confirmation, conflicts and provider prerequisites. Use the already accepted typed host API; do not add a Tauri command here. Keep browser download/workspace save separate from native disk apply.
- Focused validation: browser unsupported-state regression; desktop fixture preview/apply/cancel/stale-source denial; WebMCP attempt to invoke native effects denied; same portable result in browser and desktop.
- Handoff: Record desktop-only rows, accepted native artifacts, and remaining format gaps.

## FMT-10 — Accept the maximum evidenced format matrix

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: FMT-05, FMT-05B, FMT-06, FMT-06B, FMT-06C, FMT-07, FMT-07B, FMT-08 and any ACRE implementation phases it adds, and FMT-09 resolved; stable shared frontend and native acceptance environment
- Exact write set: `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/README.md`, `crates/acorn-wasm/examples/webmcp/README.md`, `crates/acorn-cli/assets/skills/acorn/SKILL.md`, `crates/acorn-desktop/acceptance/capability-matrix.md`, `.wiki/features/browser-format-parity.md`, `.wiki/features/acorn-browser-workspace/index.md`
- Primary integration lock: cross-surface format parity acceptance
- Outcome: Publish a versioned matrix of accepted standard/syntax/policy pairs and explicit browser-versus-native differences. Measure format latency and WASM size; review user-facing names and limits. Close only gaps that fit this write set; any new product defect opens a bounded repair phase.
- Focused validation: fixed fixture corpus across CLI/native MCP, WASM, WebMCP and browser page; supported Chrome tool invocation; desktop format preview/apply on an available native host; root/prefixed browser builds, output boundary, keyboard and narrow-screen checks. Run final broad gates through one integration owner.
- Handoff: Move durable matrix/limits to product docs and tests, retire this plan, and remove its wiki index/network/lane entries.
