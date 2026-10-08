---
okf_version: "0.1"
type: Plan
title: WebMCP browser validation and readability tools
description: Expose ACORN's existing WASM validation and readability bindings through an optional browser adapter with bounded inputs, structured results, and a demonstrable browser workflow.
difficulty: medium
impact: medium
package:
  - acorn-wasm
tags:
  - agents
  - browser
  - validation
  - wasm
  - webmcp
status: planned
---

# WebMCP browser validation and readability tools

## Goal and first consumer

Let a user open an ACORN demonstration page, supply Research Activity JSON or prose, and ask a compatible browser agent to validate the metadata or assess readability. The page and agent must receive equivalent structured results from the existing Rust implementation. The page remains usable when WebMCP is unavailable.

The first deliverable is an opt-in JavaScript adapter distributed beside the generated WASM web package, plus a locally served demonstration page. An institutional research portal can subsequently embed the adapter. A production portal, hosted deployment, and metadata editing workflow are outside this plan.

## Evidence and status

Reviewed on 2026-10-08. This is planned work; no WebMCP implementation or browser verification is claimed.

| Evidence | Implication |
|---|---|
| [WASM bindings](../../crates/acorn-wasm/src/lib.rs) export `validateResearchActivity(json)` and `checkReadability(text, metric, uri?)`. | Both initial tools can reuse existing operations without new domain logic. |
| Validation uses `eserde::json::from_str` and returns `valid`, `value`, and ordered diagnostics with kind, path, message, code, and params. | Preserve multiple errors and field paths; metadata errors are ordinary tool results. |
| Readability supports `ari`, `cli`, `fkgl`, `fres`, `gfi`, `lix`, and `smog`, returning checks and summary rows. | Publish a bounded metric enum; avoid a second scoring implementation. |
| The bindings use `serde_wasm_bindgen::to_value`. | Verify actual JS values before returning them through WebMCP; maps, undefined values, and non-finite numbers need explicit treatment. |
| [WASM manifest](../../crates/acorn-wasm/Cargo.toml) selects portable analyzer/schema features. | Keep browser integration outside portable Rust crates and retain the host-dependency exclusion gate. |
| [Makefile](../../Makefile) has `build-wasm`, `check-wasm`, and `check-portable`; `build-wasm` builds the web package and runs Node WASM tests. | Extend the current target; `wasm-package` in older documentation is stale. Node tests alone do not verify browser registration. |
| [WASM guide](../../docs/src/packages/acorn-web.md) documents an in-memory API and no supported browser application or browser service adapters. | Add a narrowly scoped example and adapter documentation without claiming a full browser application. |
| The former `wasm-feature-path.md` planning file is absent; the [portability index](../refactors/portability/index.md) calls for atomic plans with concrete consumers. | This page owns the browser workflow and does not revive the completed portability umbrella. |

External baseline: the [WebMCP Community Group draft](https://webmachinelearning.github.io/webmcp/) dated 2026-10-08 exposes `document.modelContext` and JavaScript tool registration. It is not a W3C Standard. [Chrome documentation](https://developer.chrome.com/docs/ai/webmcp), updated 2026-10-07, describes imperative and declarative APIs, an origin trial from Chrome 149, local testing flags, and the `tools` Permissions Policy. Recheck the draft and the actual test browser together before implementation; a draft API and a browser implementation can differ.

## Scope and architecture

The execution path is browser agent -> page WebMCP adapter -> generated WASM bindings -> portable Rust validation/analysis. The page's manual controls call the same adapter operations. Tool invocation displays an operation status and result in the page without changing the supplied document.

Use the imperative API because ACORN already has callable functions and structured results. Register only after WASM initialization succeeds. Keep module import free of registration and network side effects. The host explicitly initializes the package and installs the adapter against a supplied model context; this also supports deterministic test doubles.

The first release owns two tools, input validation, output normalization, registration cleanup, a demonstration, packaging, tests, and documentation. It excludes native command execution, remote MCP transport, filesystem access, URL fetching, persistent storage, credentials, model inference, document conversion, publication, and automatic metadata repair. Identifier checks and schema crosswalk bindings require separate consumer-driven plans after this outcome ships.

## Proposed tool contracts

Freeze these contracts in phase 1 after checking current browser behavior. Tool schemas describe wrapper arguments; Rust remains authoritative for Research Activity schema semantics.

| Tool | Input | Successful execution result |
|---|---|---|
| `acorn_validate_research_activity` | Required `json` string containing raw Research Activity JSON. | `valid`, ordered `diagnostics`, `diagnosticCount`, and `truncated`; omit the validated `value` to avoid echoing the whole document. |
| `acorn_check_readability` | Required `text` string and required `metric` from the seven supported values. | Existing ordered `checks` and `summary`, normalized to JSON-compatible values. |

Raw JSON is intentional: malformed JSON and incorrect field types must reach ACORN's multi-error parser. Do not pre-parse the document in JavaScript. The readability wrapper omits the optional URI because the first consumer does not need one and does not fetch resources.

Both schemas use object inputs, required fields, descriptions, and `additionalProperties: false`. Enforce the same constraints at runtime; schema publication alone is insufficient. Use a common application envelope: `{ ok: true, result: ... }` or `{ ok: false, error: { code, message } }`. Invalid research metadata yields `ok: true` with `valid: false`. Adapter errors use stable codes such as `invalid_arguments`, `input_too_large`, `cancelled`, `result_too_large`, and `execution_failed`. Messages must not include full input, stack traces, or local paths. Do not assume a remote MCP content-block envelope is required by WebMCP; verify the browser's accepted result contract.

Proposed initial limits are 256 KiB of UTF-8 input per call, 100 returned diagnostics, 256 KiB of serialized output, and one active execution per adapter instance. Treat these as implementation defaults to confirm against committed fixtures in phase 1. Schema string-length limits supplement the authoritative UTF-8 byte check. Preserve diagnostic ordering, report total count and truncation, and return a bounded error if the remaining result still exceeds the output budget. Never silently truncate input before validating it.

Convert supported Map values to plain objects recursively, normalize optional missing values consistently, and explicitly handle non-finite numeric values. Prefer a documented unavailable value over invented numeric scores; if the existing binding cannot represent that safely, stop the contract phase and resolve it under the library-root lock. Add a JSON serialization round-trip test that preserves diagnostic params and nested data. Keep normalization in the adapter unless a separately justified binding correction is required.

## Lifecycle, cancellation, and trust boundaries

- Feature-detect the supported `document.modelContext` registration surface. Unsupported browsers retain manual validation and display capability status. Do not load an automatic third-party polyfill or silently promise compatibility with the old navigator API.
- Return an explicit installation result and disposer. Track registrations owned by the adapter, reject duplicate installation, and clean up partial registration failures. Use the cleanup mechanism supported by the verified browser/draft contract; never unregister another component's tools.
- Account for asynchronous registration and disposal while initialization is pending. Repeated mounting and unmounting must not leave duplicate registrations or callbacks attached to stale UI state.
- Check cancellation before executing and before publishing results. Current WASM calls are synchronous, so an AbortSignal cannot interrupt active computation on the same thread. Measure bounded-input latency; if it exceeds the phase 1 responsiveness gate, add an explicitly scoped worker slice with termination and restart tests before release. Do not advertise hard execution deadlines until that exists.
- Use read-only and untrusted-output annotations when supported by the verified implementation. Annotations are hints; the adapter itself must enforce its lack of write and network operations.
- Keep registration same-origin by default and document the `tools` Permissions Policy. Cross-origin embedding is outside the initial support matrix.
- Treat document fields and diagnostic params as untrusted data. Render results as text, keep tool descriptions static, and avoid including input in logs. Tool outputs can be sent to the browser agent's model provider even though ACORN computes locally; explain this in the example and guide.
- Readiness means WASM initialization completed. Failures must leave manual controls with a useful error and no partially advertised tool set.

## Dependencies and integration locks

The existing portable WASM package is the prerequisite. This plan is independent of the [outbound MCP client](mcp-client.md), SLM sidecars, and shared API client extraction. It adds no Rust dependency or workspace manifest change by default. Any required manifest change follows the [dependency capabilities plan](../refactors/portability/dependency-features-and-macos-size.md) and the workspace-graph lock.

Assign the work to the Browser tools lane in the [parallel execution guide](../parallel-execution.md). Claim the CI/build lock for Makefile and CI wiring, the library-roots lock for any justified binding correction, and documentation-generation lock if book indexes change. Final `make lint`, `make test`, and `make check-portable` run serially after implementation owners are stable.

## Proposed exact write set

Confirm these paths and record the baseline commit before implementation. New paths below are proposals, not existing artifacts. No generated package files under `target/` are tracked.

| Slice | Create or update | Ownership |
|---|---|---|
| Adapter | Create `crates/acorn-wasm/web/webmcp.js` and `crates/acorn-wasm/web/webmcp.d.ts`. | Registration, runtime argument checks, bounded result normalization, public adapter types. |
| Contract tests | Create `crates/acorn-wasm/web/webmcp.test.mjs`. | Node built-in test runner with injected bindings/model context; no new runtime dependency. |
| Demonstration | Create `crates/acorn-wasm/examples/webmcp/index.html` and `crates/acorn-wasm/examples/webmcp/main.js`. | Accessible manual controls, capability status, invocation/result display, text-safe rendering. |
| Browser verification | Create `crates/acorn-wasm/examples/webmcp/smoke.js` and `crates/acorn-wasm/examples/webmcp/README.md`. | Browser assertions and reproducible setup; exact automated browser runner choice is a phase 1 gate. |
| Packaging | Update `Makefile`. | Copy adapter/types alongside generated JS/WASM after wasm-pack; add focused test target under CI/build lock. |
| CI | Update `.gitlab-ci.yml`. | Run adapter contracts in portable job; wire native browser smoke only after the runner contract is proven. |
| Durable docs | Update `docs/src/packages/acorn-web.md`. | Installation, supported matrix, sample workflow, limits, privacy, troubleshooting, current build target and removal of obsolete plan references. |
| Conditional binding repair | Update `crates/acorn-wasm/src/lib.rs` and `crates/acorn-wasm/src/tests.rs` only if phase 1 demonstrates a binding defect. | Separate declared slice; preserve existing JS behavior or document/version any compatibility change. |

Do not add a browser dependency, package manifest, lockfile, worker, fixture, or CI template outside this set without first naming its exact path and lock in the active slice. The initial example can embed small synthetic examples directly; any shared fixture reuse must be read-only.

## Implementation phases

### 1. Verify browser contract and freeze the adapter boundary

1. Record browser version/channel, feature flags or origin-trial setup, spec URL/date, Node version, wasm-pack version, and baseline commit.
2. Verify registration, input schema, JSON result serialization, annotations, execution cancellation signal, and cleanup in a native WebMCP browser. Confirm policy behavior in top-level same-origin and unsupported environments.
3. Inspect generated package layout and actual binding values, including diagnostic params, optional values, empty prose, and non-finite score cases. Compare manual and tool-call result shapes.
4. Confirm the proposed input/output limits using synthetic valid, malformed, Unicode, and near-limit inputs. Record cold initialization and warm execution timings. Proposed responsiveness gate: each warm call under 100 ms at the maximum accepted input on the documented reference browser/machine, across 20 calls per operation; retain individual timings and maximum. If this fails, schedule a worker slice or reduce documented limits based on fixture needs.
5. Select the smallest existing browser-test facility that can run the required native implementation. If none exists, record the proposed runner's exact dependency/files and CI feasibility before introducing it. Keep deterministic adapter tests independent of experimental browser availability.

Exit: documented supported contract, frozen tool schemas and limits, reproducible native smoke, and an exact file/lock assignment. If native registration or lossless serialization cannot be demonstrated, keep status `planned` and record the blocker; a mock is insufficient evidence.

### 2. Implement and test the adapter

Implement pure argument/result transformations separately from registration side effects. Initialize WASM once at the host boundary, inject the two bindings, register the fixed tool set, and return cleanup ownership. Add TypeScript declarations matching runtime behavior. Verify error codes, readiness, duplicate install, disposal races, partial failure, bounded results, and cancellation behavior with the Node test runner.

Exit: both tools produce the agreed result shape; all supported fixture outputs survive JSON serialization; argument checks run before WASM; unsupported contexts do not disrupt manual calls.

### 3. Demonstrate a complete user workflow

Build a small page with a JSON textarea, prose textarea, metric selector, manual buttons, and visible diagnostics. Serve it through a documented local HTTP origin that satisfies browser security requirements. Use generated WASM assets from the build output and document their URL mapping. Never require opening the page directly with `file://`.

Run the same supplied input manually and through a native WebMCP tool invocation. Display operation state and results without auto-editing the source. Verify keyboard operation, readable status updates, safe text rendering, unsupported-browser behavior, and initialization failure. Exercise discovery and execution with a tool inspector; separately record one compatible agent workflow if available, without making a cloud model credential necessary for deterministic tests.

Exit: reproducible browser smoke with equivalent results and a fully usable manual fallback.

### 4. Package, integrate, and document

Extend `build-wasm` to include adapter JS and declarations after package generation, with paths resolved relative to the generated package. Verify a clean rebuild replaces stale assets and that imports work from the documented example. Add a focused adapter test target and connect it to portable CI under the CI/build lock. Treat missing native WebMCP support as an explicit unsupported test environment, never as a passing native smoke.

Document the tested browser matrix, opt-in installation, lifecycle, tool contracts, resource limits, synchronous cancellation limitation, data disclosure, and manual fallback. Run serialized repository gates and save browser evidence in durable documentation before closeout. A hosted public origin trial is optional follow-on deployment work.

## Validation matrix

| Area | Required checks |
|---|---|
| Validation fidelity | Valid document; malformed JSON; multiple type errors; schema validation errors; nested diagnostic paths and params; stable ordering; no returned document echo. |
| Readability fidelity | Every supported metric; empty and short text; Unicode; invalid metric; numerical edge cases; direct-binding comparison. |
| Argument bounds | Missing/wrong fields; unknown keys; exact limit and one byte over; multibyte UTF-8; no binding invocation on rejected input. |
| Output bounds | Diagnostic cap/count; explicit truncation; nested Map conversion; null/undefined policy; non-finite values; full JSON round trip; oversized result error. |
| Lifecycle | Unsupported API; initialization failure; registration rejection; partial rollback; duplicate installation; dispose/reinstall; pending initialization disposal; cancellation before call and before result delivery. |
| Browser behavior | Actual registration/discovery/execution; tested cleanup; tool inspection; manual parity; reload; permissions-policy denial; unsupported browser fallback. |
| Security and presentation | No tool-triggered network/write operations; static metadata; hostile HTML rendered as text; no payload logging; model-provider disclosure visible. |
| Packaging | Clean `make build-wasm`; copied adapter and type declarations; example imports resolve; source and packaged adapter contracts agree. |
| Repository gates | `node --test crates/acorn-wasm/web/webmcp.test.mjs`; `make lint`; `make test`; `make check-portable`, serialized under the final integration owner. |

Native browser evidence must record version, configuration, commands or inspector steps, expected/actual outcomes, and date. Keep deterministic mock tests and native compatibility results distinct. Re-run browser compatibility when the pinned browser baseline or adapter API changes.

## Acceptance criteria

- [ ] The two named tools are discoverable and callable in a documented native WebMCP browser.
- [ ] Both tools reuse existing WASM operations and preserve the defined diagnostic/scoring semantics.
- [ ] Runtime input checks and byte/output limits are enforced, including Unicode and oversized results.
- [ ] Outputs are JSON-compatible and preserve nested diagnostic information; no full metadata echo occurs.
- [ ] Readiness, partial failure, repeated installation/disposal, and supported cancellation behavior are tested.
- [ ] Manual controls work without WebMCP and produce equivalent results on the same inputs.
- [ ] Tool execution leaves source content unchanged and introduces no network, filesystem, storage, or process capability.
- [ ] The responsiveness gate passes or an explicitly scoped worker implementation passes replacement responsiveness/cancellation gates.
- [ ] The generated package includes the adapter and types, and the example works using documented build/serve instructions.
- [ ] Adapter tests, native browser smoke, and serialized repository gates pass with recorded evidence.
- [ ] Durable WASM documentation states experimental support, browser matrix, limits, lifecycle, and data disclosure accurately.

## Risks and closeout

API churn is the main external risk. Keep registration isolated, test the supported implementation, and update the compatibility matrix before changing the contract. Synchronous WASM calls and serialization differences are the main technical risks; resolve them before broadening the tool set. A demonstration validates feasibility but does not establish institutional adoption demand.

During implementation update status, evidence, validation, and acceptance criteria together. Use `waiting-on-validation` only when implementation is complete and remaining work is verification. On completion, move durable results into the public guide and tests, delete this plan, and remove its feature-index, portfolio, dependency-network, and lane entries. Track any identifier, crosswalk, editing, or backend expansion as its own atomic outcome.
