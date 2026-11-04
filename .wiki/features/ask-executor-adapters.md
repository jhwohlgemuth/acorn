---
type: Plan
title: Generic ask executor adapters
description: Bind configured MCP tools, ACP agents, OpenAI-compatible endpoints, and local sidecars to the native capability router.
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [feature, inference, routing, mcp, acp, sidecar]
status: in-progress
execution: phased
phase_budget_hours: 5
---

# Goal

Extend the [native capability router](needle-router-native-parity.md) with explicit configured executor bindings. Keep its capability schemas, authority checks, one-terminal-operation rule, provenance, and bounded cancellation. No external tool or model is enabled by discovery alone.

# Current evidence

Evidence rechecked on 2026-11-02 against `95ec0e09` and the current worktree:

- `ToolRegistry` has versioned capability and binding definitions, deterministic selection, typed one-terminal execution, and focused registry/execution tests. Routed `ask`/`infer --route` currently registers native `acorn.version` and local `acorn.check`; interactive metadata chat also invokes the native check binding. Native parity still gates the heterogeneous release.
- The outbound MCP client provides configured allowlists, discovery, calls, policy checks, cancellation, and bounded cleanup. Direct `infer run` and the new routed adapters share the existing ACP and OpenAI-compatible backends; Ollama uses local chat defaults and distinct provenance.
- `ApplicationConfiguration.executors` stores explicit, non-secret capability/binding IDs and tagged MCP, trusted ACP, named endpoint, or sidecar selectors. Validation rejects duplicate or malformed IDs, unallowlisted MCP tools, unknown endpoint names, unsupported executor kinds, and untrusted ACP profiles. JSON, JSONC, and YAML parsing and JSON round trips have focused tests. Parsing performs no provider discovery.
- `ToolRegistry` now admits provider-declared runtime JSON Schemas through the same canonical contract checks as typed bindings. It rejects invalid schemas, conflicting capability contracts, duplicate binding IDs, and version/schema/effect mismatches while preserving deterministic selection. Cooperative capability handlers receive caller cancellation and get a four-second bounded cleanup window; native handlers retain drop cancellation.
- `ToolRegistry::register_mcp_bindings` constructs the outbound client from only explicitly referenced MCP server configurations and tool allowlists, discovers each selected server once, and registers only the exact configured tools. Configured IDs and local effect policy remain authoritative; server descriptions and annotations are ignored for routing policy. Calls retain `McpClientBackend` policy and limits, require declared structured output, map `is_error` to tool failure, never fall back, and finish client cleanup on cancellation. Fake-backend adapter tests and existing client pagination/cursor tests pass.
- Routed CLI startup loads and validates discovered ACRE configuration through `ApplicationConfiguration::load(&None)` without new CLI syntax. It composes native, configured ACP, endpoint, and selected MCP bindings; offline mode skips MCP discovery. Remote eligibility follows explicit MCP or endpoint policy and is denied offline. Unimplemented sidecar selectors remain visible as unavailable bindings.
- The optional `slm` feature now compiles the shared sidecar root and an empty `sidecar::slm` namespace independently of `needle`; the CLI forwards the opt-in feature. Minimal `slm`, existing `needle`, combined `needle,slm`, and CLI `slm` checks pass, and the default library dependency tree remains unchanged.
- ACP bindings register only configured Codex/OpenCode profiles with the shared typed embedded-prompt task contract, selected model, deny-mutation policy, normalized final result, and response-schema validation. Endpoint bindings reuse that contract with an explicitly named ACRE endpoint/model, local-only or allow-remote policy, offline-safe loopback selection, and distinct Ollama provenance when the endpoint is named `ollama`. Registration starts no agent and contacts no endpoint. Unsupported input fields and invalid response schemas are rejected before inference; refusals, incomplete results, and malformed outputs fail without fallback.
- Focused routed tests cover ACP and endpoint cancellation, policy, model selection, binding preference, malformed results, and a real loopback Ollama-compatible HTTP call. Existing ACP fake-agent tests cover permission denial, unavailable models, and timeout. The endpoint adapter also passes with `agentic` without `acp`; ACP registration remains gated on `acp`. Strict library Clippy and formatting checks pass.
- Routed `register_sidecar_binding` checks the host, executable permission, and ModelKit assets without acquisition or process startup. It rechecks assets immediately before the consuming sidecar invocation and applies a 120-second task bound; the deterministic fixture proves one typed result, stale/altered asset rejection, and cancellation. No concrete SLM profile has landed, so configured profile selectors remain unavailable until their profile plans register handlers.
- MCP registration discovers tools on explicitly selected configured servers. CLI configuration loading and validation make no provider call; selected MCP discovery is the adapter-declared readiness work allowed during online registry construction. Focused sidecar tests and 44 CLI inference tests pass.
- Native parity is at 46/46 bounded rows; PAR-15–16 routing evaluation and release validation remain. Existing capability/execution tests prove selection and single-dispatch policy with typed fixtures, while separate MCP, ACP, endpoint, and sidecar test modules prove their provider boundaries. No heterogeneous same-task matrix or `executors.json` fixture exists yet. `docs/src/inference/routing-evaluation.md` is first owned by PAR-15; PAR-16 owns the shared release documentation before ADP-07.

# Dependencies and boundaries

ADP-05 and ADP-09 have landed. ADP-06 starts after PAR-16 accepts native parity and hands off its measured routing evaluation. ADP-07 starts after ADP-06 accepts the heterogeneous matrix. The shared routing evaluation and release documentation must be handed off in that order under the Router release gate.

The common inference contract accepts an embedded prompt asset and object render data, then returns final text, optional validated structured output, usage, and secret-free provenance. Configured binding IDs and capability IDs remain explicit. Provider-advertised structured-output support is not treated as proof of conformance.

Adapters use their existing service contracts and do not change those providers' policy. A same-task comparison requires the same capability ID, version, input/output schemas, and effects; provider-specific tests alone are supporting evidence, not proof that arbitrary MCP tools, native operations, and inference profiles are interchangeable. The [parallel guide](../parallel-execution.md) serializes shared registry, CLI, manifest, and documentation roots.

# Validation and acceptance criteria

Release the generic router only when the heterogeneous selection matrix passes and documentation describes explicit configuration and failure behavior; real Swift and Julia profiles retain their own release gates.

# Execution phases

## ADP-06 — Validate heterogeneous executor selection

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: PAR-16 native parity acceptance and handoff of PAR-15's `routing-evaluation.md`; the 46 bounded rows and held-out routing gate must have measured results. Do not start this phase until the held-out gate and PAR-16 release checks pass.
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/tests/capability.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/execution.rs`, `tests/fixtures/routing/executors.json`, `docs/src/inference/routing-evaluation.md`
- Primary integration lock: Router release gate
- Outcome: Add a versioned, secret-free same-task fixture matrix and exercise compatible native, deterministic local-sidecar, configured MCP, ACP, and endpoint bindings without router changes or real Swift/Julia assets. Compare normalized results and selected binding IDs while preserving provider provenance. Treat intentionally incompatible schemas/effects as registration failures, never as equivalent task results.
- Focused validation: Matrix rows cover canonical contract equality, schema/effect/version rejection, explicit override, configured preference and remote ambiguity, stale/unavailable selections, offline and remote data policy, exact mutation authority, cancellation, and one terminal operation without retry. Reuse the existing provider-boundary fixtures in `tests/{mcp,acp,inference,sidecar}.rs` as supporting evidence; record which rows exercise actual adapter registration and which use synthetic same-contract bindings. Run focused `acorn-lib` capability/execution and adapter tests with the required feature combinations, then record row-level outcomes in `routing-evaluation.md`.
- Handoff: Remove ADP-06 after preserving contract/evidence in code, tests, and documentation.

## ADP-07 — Document and release the generic router

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: ADP-06 matrix accepted, PAR-16 native release documentation landed, and the shared Router release gate available.
- Exact write set: `docs/src/inference/router.md`, `docs/src/inference/capability-parity.md`, `docs/src/SUMMARY.md`, `CommandLineHelp.md`, `.wiki/features/ask-executor-adapters.md`, `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: Router release gate
- Outcome: Publish registration examples, configuration, selection rules, supported policies, and measured coverage. Release independently of Swift and Julia availability.
- Focused validation: Serialize `make lint` and `make test`, run narrow and combined `needle`/`slm`/`acp`/`mcp-client` feature checks plus `make check-wasm`, regenerate and check CLI help and the book, and smoke test only configured transports available on the validation host. Publish the ADP-06 matrix beside the covered/46 bounded and covered/72 total native-parity accounting; identify unavailable or unvalidated profiles without counting them as passing. A failed required gate keeps this phase open.
- Handoff: Retire this plan and its index, network, and lane entries after generic-adapter acceptance.
