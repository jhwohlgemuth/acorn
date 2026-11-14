---
type: Plan
title: Conversational RAD assistance and reviewed fixes
description: Add opt-in Ask ACORN, grounded explanations, and reviewed fix proposals to the shared browser and desktop workspace after the desktop shell exists.
difficulty: high
impact: high
package: [acorn-web, acorn-desktop, acorn-wasm, acorn-service, acorn-lib, acorn-cli]
tags: [feature, assistant, copilotkit, dioxus, tauri, inference, fixes, privacy, accessibility]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Add an optional Ask ACORN experience to the **same Dioxus workspace** in browser and desktop delivery. Users can ask about a selected Research Activity or diagnostic, request a grounded explanation, obtain a proposed correction, inspect its semantic diff, accept or decline each change, revalidate the result, and explicitly save or download it. All existing manual tools and offline desktop behavior remain usable without an assistant account or runtime.

**Global prerequisite:** [ABW-11](acorn-browser-workspace/index.md#abw-11--embed-the-shared-dioxus-workspace-in-a-minimal-tauri-shell) must create `acorn-desktop` and prove that it embeds the shared frontend before ACK-01 or any other phase begins. ABW-12 and ABW-17 are additional prerequisites for native save and folder-specific acceptance. This plan does not build a second desktop UI or expand ABW-11's original three native file/folder capabilities.

# Current evidence and boundaries

- On November 13, 2026, `crates/acorn-desktop/` does not exist. The [browser/desktop plan](acorn-browser-workspace/index.md) still lists ABW-11 as planned. The present `acorn-web` UI is Dioxus with one lazy React/Resium map island; the browser operations call `acorn-wasm` and its session holds bounded, in-memory supplied files. See `crates/acorn-web/README.md`, `src/operations.rs`, and `src/session.rs`.
- The browser installs nine read-only WebMCP tools through `crates/acorn-wasm/web/webmcp.js`. CopilotKit's WebMCP registration would duplicate this surface; do not replace it or turn fix application into a WebMCP tool.
- `AnalysisReport::suggest_fixes` and `FixReport::build_preview` exist in `acorn-lib`. The native `acorn.check::fixes` capability returns analysis, validated proposals, unified differences, and preview diagnostics without applying them. `acorn ask --interactive` already reviews one document fix at a time, checks source freshness, asks for confirmation, and executes a restricted plan. Reuse these semantics and contracts; do not treat model text as an approved edit.
- CopilotKit's current v2 React SDK supplies chat, frontend tools, tool rendering and human-in-the-loop UI. A static SPA needs a separately hosted TypeScript Copilot Runtime. Its open-source SDK/runtime are MIT; optional Intelligence services are outside this plan. Sources: [repository](https://github.com/CopilotKit/CopilotKit), [React SPA](https://docs.copilotkit.ai/react-spa), [frontend tools](https://docs.copilotkit.ai/frontend-tools), [human review](https://docs.copilotkit.ai/human-in-the-loop), [open-source scope](https://docs.copilotkit.ai/ag2/concepts/oss-vs-enterprise).
- The present About copy promises no telemetry and local files stay on-device unless the user requests a remote load. Assistant use must therefore be explicitly enabled, show exactly which content will be transmitted, and never ship a model key to the client. Disable CopilotKit OSS runtime telemetry and verify its documented opt-out; document provider-side data handling separately. Source: [telemetry](https://docs.copilotkit.ai/a2a/telemetry).

# Product and architecture contract

1. **One shared view:** Dioxus owns route, selection, form state, review state, accessible focus, and user-facing save/download controls. A lazy React island owns only CopilotKit's chat and registered tool/render components. A narrow, versioned JSON/event bridge carries selected file ID, content fingerprint, bounded diagnostic excerpts, and user-initiated requests. It never grants an agent a Tauri command handle or a reusable filesystem path.
2. **One opt-in runtime:** The browser and desktop panel connect to a separately configured, authenticated TypeScript Copilot Runtime over HTTPS or an explicitly configured loopback development endpoint. The runtime hosts an agent and server-side model credentials. No assistant code loads and no research content leaves the workspace before the user enables the feature and confirms the selected content. Unavailable runtime produces a clear disabled state and preserves manual tools. Desktop offline launch must remain network silent.
3. **Grounded answers:** Read-only calls use the current bounded WASM operations for selected content; every answer about a finding references its diagnostic path, code, or category. General knowledge answers are identified as such. The agent gets a minimal tool list and may not claim a validation result it did not receive. Existing WebMCP registration remains independent.
4. **Fix contract:** The model may ask for a fix proposal, but ACORN validates its shape, provenance, scope, and source fingerprint and produces a preview. Where the host `check::fixes` capability is available, a server-side adapter accepts only explicitly transmitted text and returns nonexecuting proposals/diffs. The UI must not expose a general path or command argument to that adapter. For a selected browser document, proposal application first changes only a review draft; the user confirms each change, ACORN revalidates it, and the user explicitly downloads it. On desktop, the same reviewed draft uses ABW-12's user-mediated save path, with stale-source rejection before overwrite. Unsupported fix kinds remain explanation-only.
5. **Security and privacy:** Authenticate runtime requests and scope threads to the current user/session; bound request, response, history, tool steps, rate, and timeout. Require an explicit content-transmission choice for each newly selected source. Redact credentials and remote URLs from prompts and logs. No automatic apply, remote tile/asset load, background upload, persistent transcript, Intelligence, memory, or analytics. Escape untrusted document text and tool output in the chat/review UI. Browser CSP/CORS and desktop Tauri origin/allowlist changes must be scoped to the configured assistant endpoint. A local model provider can be configured at the runtime; offline availability is claimed only after an observed end-to-end test.

# Dependencies and acceptance

The implementation order is ABW-11 → ACK-01–07. ACK-08 additionally waits for ABW-12 and ABW-17; ACK-09/10 accept the browser and desktop independently; ACK-11 waits for both. Shared `acorn-web` files, npm lockfile, workspace manifests, MCP registration and CI/Make changes serialize with the corresponding ABW, frontend-quality, interactive-chat and WebMCP owners. The desktop release plan does not wait for this optional feature; this plan does not claim signed installers.

Acceptance requires: a real selected-file conversation in both browser and desktop; grounded diagnostic explanations; a declined fix with zero source mutation; an accepted fix with a visible diff, current-source check, revalidation and explicit download/save; blocked stale, malformed, oversized and out-of-scope proposals; offline desktop launch with no assistant requests; a runtime outage that leaves all nine manual tools working; verified no duplicate WebMCP registrations; keyboard and screen-reader review flow; model keys absent from distributable assets; and an opt-out telemetry check. Tests must use fixture model responses by default; opt-in live provider checks must not enter required CI.

# Execution phases

## ACK-01 — Freeze the shared assistant contract after desktop creation

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: accepted ABW-11 desktop shell and shared-frontend digest; no concurrent shared-UI or workspace-manifest owner
- Exact write set: `crates/acorn-web/assistant/contract.md` (new), `crates/acorn-web/assistant/bridge.schema.json` (new), `crates/acorn-web/README.md`, `crates/acorn-desktop/README.md`, `.wiki/features/assisted-rad-workspace.md`
- Primary integration lock: shared browser/desktop assistant contract
- Outcome: Pin a compatible CopilotKit v2 package set, prove the lazy React-island build against the shipped frontend, freeze bridge DTOs and content-transmission/approval states, and record asset/network/bundle limits. If the SDK cannot fit the shared bundle, record the measured incompatibility and create a bounded repair phase before continuing.
- Focused validation: `make web-check`, `make web-build`, `cargo xtask verify-web`, and an ABW-11 desktop smoke build; inspect output for one WASM payload and no eager assistant request.
- Handoff: Commit the versioned bridge and measured bundle/runtime assumptions; make ACK-02 eligible.

## ACK-02 — Add Dioxus selection and consent state

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACK-01
- Exact write set: `crates/acorn-web/src/assistant.rs` (new), `crates/acorn-web/src/lib.rs`, `crates/acorn-web/src/session.rs`, `crates/acorn-web/src/ui.rs`, `crates/acorn-web/src/tests/assistant.rs` (new), `.wiki/features/assisted-rad-workspace.md`
- Primary integration lock: shared Dioxus state and route
- Outcome: Add an Ask entry for a selected file or diagnostic, bounded context projection, explicit per-source transmission consent, and deterministic reset on file replacement/clear. No network call or model UI in this phase.
- Focused validation: targeted `acorn-web` session/assistant tests and `make web-check`; verify selection changes cannot reuse prior consent.
- Handoff: Preserve the bridge state contract in tests; make ACK-03 eligible.

## ACK-03 — Mount a lazy, accessible CopilotKit chat island

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACK-02
- Exact write set: `crates/acorn-web/assistant/package.json` (new), `crates/acorn-web/assistant/package-lock.json` (new), `crates/acorn-web/assistant/src/` (new), `crates/acorn-web/scripts/build-assistant.mjs` (new), `crates/acorn-web/src/assistant.rs`, `crates/acorn-web/assets/app.css`, `.wiki/features/assisted-rad-workspace.md`
- Primary integration lock: frontend assistant asset and chat island
- Outcome: Mount/unmount CopilotKit's React provider/chat only after the user opens Ask, relay bounded selection context, and render loading, unavailable, cancellation, empty and error states with Dioxus-owned focus restoration. Use the frozen bridge, without exposing raw Tauri APIs.
- Focused validation: assistant component/bridge tests, `make web-build`, and browser smoke at root and `/acorn/` base paths; check keyboard focus and zero requests before opt-in.
- Handoff: Record bundle delta and lazy-load evidence; make ACK-04 eligible.

## ACK-04 — Provide an authenticated opt-in agent runtime

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACK-03 and an approved deployment origin or explicitly local development endpoint
- Exact write set: `crates/acorn-web/assistant/server/` (new), `crates/acorn-web/assistant/package.json`, `crates/acorn-web/assistant/package-lock.json`, `crates/acorn-web/assistant/README.md` (new), `.wiki/features/assisted-rad-workspace.md`
- Primary integration lock: assistant runtime and authentication boundary
- Outcome: Run a pinned TypeScript Copilot Runtime with server-held model credentials, authenticated thread ownership, bounded turns/bytes/steps/timeouts, exact CORS origins, redacted logs and telemetry disabled. Support configured model endpoints without shipping a provider key to clients; no ACORN write tool.
- Focused validation: fixture-agent HTTP tests for unauthenticated access, cross-session thread access, CORS, limits, cancellation and telemetry configuration; inspect a packaged frontend for secrets.
- Handoff: Record deployment settings, provider boundary and observed request routing; make ACK-05 eligible.

## ACK-05 — Ground Ask ACORN in read-only ACORN tools

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACK-04
- Exact write set: `crates/acorn-web/assistant/src/tools.ts` (new), `crates/acorn-web/assistant/src/finding-card.tsx` (new), `crates/acorn-web/assistant/server/agent.ts` (new), `crates/acorn-web/assistant/src/tests/` (new), `.wiki/features/assisted-rad-workspace.md`
- Primary integration lock: assistant read-only tool registry
- Outcome: Register selected-file validation, metadata-standard checks, identifier inspection, FAIR and diff as bounded frontend tools over the existing WASM bridge. Render finding references and actual diagnostic data; make unsupported questions explicit. Keep the existing nine WebMCP registrations unchanged.
- Focused validation: deterministic fixture conversations for each read-only tool, unsupported input, oversized output and prompt-injection text; verify WebMCP names/count and manual routes remain unchanged.
- Handoff: Record grounded-answer examples and tool bounds; make ACK-06 eligible.

## ACK-06 — Expose nonexecuting ACORN fix proposals

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACK-05; confirm the existing `acorn.check::fixes` capability accepts explicitly supplied text on the selected host transport
- Exact write set: `crates/acorn-web/assistant/server/fixes.ts` (new), `crates/acorn-web/assistant/server/agent.ts`, `crates/acorn-web/assistant/src/fix-card.tsx` (new), `crates/acorn-web/assistant/src/tests/` (new), `.wiki/features/assisted-rad-workspace.md`
- Primary integration lock: check/fix proposal adapter
- Outcome: Send only user-confirmed bounded source text to the host fix capability, return ACORN-validated proposals and unified diffs, and present per-fix provenance, risk, finding and preview failure. Reject path-bearing requests and never register an apply tool with the agent. When a full host check is unavailable, show the precise capability gap rather than fabricating a fix.
- Focused validation: fixture MCP/backend tests for allowed text, refused inference, malformed proposal, wrong target, no diff and bounded result; browser review-card rendering tests.
- Handoff: Record adapter transport and supported fix classes; make ACK-07 eligible.

## ACK-07 — Apply approved edits to a browser review draft

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACK-06
- Exact write set: `crates/acorn-web/src/assistant.rs`, `crates/acorn-web/src/session.rs`, `crates/acorn-web/src/ui.rs`, `crates/acorn-web/assistant/src/fix-card.tsx`, `crates/acorn-web/src/tests/assistant.rs`, `crates/acorn-web/assistant/src/tests/`, `.wiki/features/assisted-rad-workspace.md`
- Primary integration lock: browser reviewed-draft state
- Outcome: Make Accept and Decline Dioxus-owned user actions. Compare original fingerprint, materialize only the approved document change in an unsaved draft, revalidate with WASM, and offer explicit browser download. Recipes, external effects and multi-file writes stay unavailable.
- Focused validation: accepted/declined/stale/invalid/multiple-proposal tests and browser download byte comparison.
- Handoff: Preserve the no-agent-write invariant in tests and document the exact supported edit class; make ACK-08 and ACK-09 eligible.

## ACK-08 — Save a reviewed draft through desktop effects

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACK-07; accepted ABW-12 native save effect and ABW-17 selected-root boundary
- Exact write set: `crates/acorn-web/src/assistant.rs`, `crates/acorn-web/src/tests/assistant.rs`, `crates/acorn-desktop/src/commands.rs`, `crates/acorn-desktop/src/tests.rs`, `crates/acorn-desktop/README.md`, `.wiki/features/assisted-rad-workspace.md`
- Primary integration lock: Tauri selected-source save boundary
- Outcome: Route the Dioxus-approved draft through ABW's existing user-mediated save effect. Recheck the selected native source before overwrite; preserve the original and report conflict when stale. Add no agent-callable Tauri command or new filesystem authority.
- Focused validation: desktop Tauri command tests against a temporary selected file, stale content, and an unauthorized path; offline manual save regression.
- Handoff: Record the native stale-source contract and make ACK-10 eligible.

## ACK-09 — Accept the browser assistant flow

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACK-07
- Exact write set: `crates/acorn-web/scripts/assistant-smoke.mjs` (new), `crates/acorn-web/assistant/README.md`, `crates/acorn-web/README.md`, `.wiki/features/assisted-rad-workspace.md`
- Primary integration lock: browser assistant acceptance
- Outcome: Record root and `/acorn/` browser evidence for consent, selected-file chat, grounded findings, proposal/decline/accept/download, runtime failure, privacy disclosure, keyboard and screen-reader operation, and unchanged nine-tool WebMCP/manual behavior.
- Focused validation: fixture-backed browser smoke at both base paths, browser network trace, accessibility inspection, and opt-in live model smoke outside required CI.
- Handoff: Record reproducible evidence and any exact browser defect; make ACK-11 eligible after desktop acceptance.

## ACK-10 — Accept the desktop assistant flow

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACK-08; ABW-13/14/15 native host acceptance for each claimed platform
- Exact write set: `crates/acorn-desktop/acceptance/assistant.md` (new), `crates/acorn-desktop/README.md`, `.wiki/features/assisted-rad-workspace.md`; external systems: only the native hosts already used by ABW-13/14/15
- Primary integration lock: desktop assistant acceptance
- Outcome: Record observed native-host evidence for opt-in chat, review/save, offline launch, runtime outage, no background network or agent filesystem access, and the declared platform matrix. Document unobserved platforms honestly.
- Focused validation: fixture runtime on each claimed native host, offline/network trace, selected-file conflict case, keyboard/screen-reader review, and opt-in live model smoke outside required CI.
- Handoff: Record reproducible native evidence and exact remaining host blockers; make ACK-11 eligible after browser acceptance.

## ACK-11 — Publish assistant documentation and retire the plan

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACK-09 and ACK-10 accepted; no outstanding security, privacy or accessibility failure
- Exact write set: `docs/src/packages/acorn-web.md`, `docs/src/packages/acorn-desktop.md`, `docs/src/SUMMARY.md`, `crates/acorn-web/README.md`, `crates/acorn-desktop/README.md`, `.wiki/features/assisted-rad-workspace.md` (delete), `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: assistant product documentation and wiki retirement
- Outcome: Publish setup, supported providers, data-sharing semantics, offline/unavailable behavior, limits, fix-review guarantees and tested platform matrix. Remove this plan and all index/network/lane references after its acceptance is verified.
- Focused validation: documentation build, `make web-check`, focused assistant tests, wiki validator and `git diff --check`; serialize final `make lint` and `make test` after concurrent implementations settle.
- Handoff: Retain durable behavior and test evidence in product documentation and release history, with no completed plan stub.
