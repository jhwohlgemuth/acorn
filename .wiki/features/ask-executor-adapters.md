---
type: Plan
title: Generic ask executor adapters
description: Bind configured MCP tools, ACP agents, OpenAI-compatible endpoints, and local sidecars to the native capability router.
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [feature, inference, routing, mcp, acp, sidecar]
status: in-progress
priority: next
execution: phased
phase_budget_hours: 5
---

# Goal

Extend the [native capability router](needle-router-native-parity.md) with explicit configured executor bindings. Keep its capability schemas, authority checks, one-terminal-operation rule, provenance, and bounded cancellation. No external tool or model is enabled by discovery alone.

# Current evidence

Evidence checked on 2026-11-01 against `ee76cad9` and the current worktree:

- `ToolRegistry` has versioned capability and binding definitions, deterministic selection, typed one-terminal execution, and focused registry/execution tests. Routed `ask`/`infer --route` currently registers native `acorn.version` and local `acorn.check`; interactive metadata chat also invokes the native check binding. Native parity still gates the heterogeneous release.
- The outbound MCP client provides configured allowlists, discovery, calls, policy checks, cancellation, and bounded cleanup. Direct `infer run` and the new routed adapters share the existing ACP and OpenAI-compatible backends; Ollama uses local chat defaults and distinct provenance.
- `ApplicationConfiguration.executors` stores explicit, non-secret capability/binding IDs and tagged MCP, trusted ACP, named endpoint, or sidecar selectors. Validation rejects duplicate or malformed IDs, unallowlisted MCP tools, unknown endpoint names, unsupported executor kinds, and untrusted ACP profiles. JSON, JSONC, and YAML parsing and JSON round trips have focused tests. Parsing performs no provider discovery.
- `ToolRegistry` now admits provider-declared runtime JSON Schemas through the same canonical contract checks as typed bindings. It rejects invalid schemas, conflicting capability contracts, duplicate binding IDs, and version/schema/effect mismatches while preserving deterministic selection. Cooperative capability handlers receive caller cancellation and get a four-second bounded cleanup window; native handlers retain drop cancellation.
- `ToolRegistry::register_mcp_bindings` constructs the outbound client from only explicitly referenced MCP server configurations and tool allowlists, discovers each selected server once, and registers only the exact configured tools. Configured IDs and local effect policy remain authoritative; server descriptions and annotations are ignored for routing policy. Calls retain `McpClientBackend` policy and limits, require declared structured output, map `is_error` to tool failure, never fall back, and finish client cleanup on cancellation. Fake-backend adapter tests and existing client pagination/cursor tests pass.
- Routed CLI startup still builds only native `acorn.version` and local `acorn.check`, has no `--config` field on `Infer`, and hard-codes `allow_remote: false`. Existing `ApplicationConfiguration::load(&None)` provides project discovery without new CLI syntax. ADP-09 must load and validate that configuration without contacting or starting providers, then compose the landed adapter registrations.
- The optional `slm` feature now compiles the shared sidecar root and an empty `sidecar::slm` namespace independently of `needle`; the CLI forwards the opt-in feature. Minimal `slm`, existing `needle`, combined `needle,slm`, and CLI `slm` checks pass, and the default library dependency tree remains unchanged.
- ACP bindings register only configured Codex/OpenCode profiles with the shared typed embedded-prompt task contract, selected model, deny-mutation policy, normalized final result, and response-schema validation. Endpoint bindings reuse that contract with an explicitly named ACRE endpoint/model, local-only or allow-remote policy, offline-safe loopback selection, and distinct Ollama provenance when the endpoint is named `ollama`. Registration starts no agent and contacts no endpoint. Unsupported input fields and invalid response schemas are rejected before inference; refusals, incomplete results, and malformed outputs fail without fallback.
- Focused routed tests cover ACP and endpoint cancellation, policy, model selection, binding preference, malformed results, and a real loopback Ollama-compatible HTTP call. Existing ACP fake-agent tests cover permission denial, unavailable models, and timeout. The endpoint adapter also passes with `agentic` without `acp`; ACP registration remains gated on `acp`. Strict library Clippy and formatting checks pass.
- `Sidecar::start` calls `resolve_assets`, which may acquire or replace official assets. Routed registration therefore needs an installed-asset readiness path that does not call `start` or `resolve_official` with downloads enabled. The `slm` namespace currently has no concrete profile; generic fixture registration can prove ADP-05, while ADP-09 must keep unimplemented configured profiles visibly unavailable.
- MCP registration discovers tools on explicitly selected configured servers. CLI configuration loading and validation can remain free of provider calls; selected MCP discovery is the adapter-declared readiness work allowed during registry construction.

# Dependencies and boundaries

Implementation scheduling on 2026-11-01 places ADP-05 next after the ACP and endpoint adapter handoffs. This is a queue order for shared ownership, not a technical dependency.

The common inference contract accepts an embedded prompt asset and object render data, then returns final text, optional validated structured output, usage, and secret-free provenance. Configured binding IDs and capability IDs remain explicit. Provider-advertised structured-output support is not treated as proof of conformance.

ADP-05 may proceed while native capability phases continue. ADP-09 composes the landed MCP, ACP, and endpoint adapters with the remaining sidecar contract through the existing CLI route syntax; it cannot precede ADP-05 because configuration contains a sidecar selector rather than an executable handler. ADP-06 waits for ADP-09 and PAR-16 before evaluating heterogeneous selection. Adapters use their existing service contracts and do not change those providers' policy. The [parallel guide](../parallel-execution.md) serializes shared registry, CLI, manifest, and documentation roots.

# Validation and acceptance criteria

Validate the remaining sidecar adapter with a deterministic fixture provider, unavailable and denied routes, bounded cancellation, and secret-free provenance. Release the generic router only when the heterogeneous selection matrix passes and documentation describes explicit configuration and failure behavior; real Swift and Julia profiles retain their own release gates.

# Execution phases

## ADP-05 — Prove reusable local sidecar registration

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: None
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/sidecar.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/sidecar.rs`
- Primary integration lock: Ask capability routing
- Outcome: Adapt SidecarAssets/Sidecar into registered handlers with verified installed assets, target availability, bounded lifecycle, and typed final output. Expose a reusable named-profile registration contract for CLI composition; use deterministic fixture runners, while real SLMs remain model-profile work. Readiness must not invoke the acquisition-capable `Sidecar::start` path.
- Focused validation: Readiness without network or cache mutation, input/output contracts, missing/altered assets, unsupported hosts, timeout/cancellation/process cleanup, exactly one invocation, and no implicit acquisition. Verify the adapter under the opt-in sidecar feature as well as the default build.
- Handoff: Remove ADP-05 after preserving contract/evidence in code, tests, and documentation.

## ADP-09 — Compose configured bindings at the CLI

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-05 and the landed CLI route syntax
- Exact write set: `crates/acorn-cli/src/commands/infer/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`
- Primary integration lock: CLI contract
- Outcome: Load and validate discovered ACRE configuration, compose the native registry with only its explicitly configured adapter bindings, and derive remote eligibility from the frozen adapter/configuration policy without changing CLI syntax or provider transports. Configuration loading performs no provider contact. Registry construction may discover configured MCP tools but performs no unselected-provider discovery, sidecar acquisition, agent startup, or inference. Until a concrete SLM profile lands, an unimplemented sidecar selector remains an explicit unavailable binding.
- Focused validation: Configured composition across native, MCP, ACP, and endpoint bindings plus explicit unavailable sidecar selectors; missing/default configuration, unavailable providers, offline and remote-policy denial, duplicate/unknown contracts, and proof that startup performs no provider call beyond selected MCP discovery and adapter-declared local readiness work.
- Handoff: Remove ADP-09, preserve the composition contract in focused CLI tests, and release the CLI contract lock.

## ADP-06 — Validate heterogeneous executor selection

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-09, PAR-16
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/tests/capability.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/execution.rs`, `tests/fixtures/routing/executors.json`, `docs/src/inference/routing-evaluation.md`
- Primary integration lock: Router release gate
- Outcome: Prove compatible task bindings across native, deterministic local-sidecar, MCP, ACP, and endpoint fixtures without router changes. Compare task results and explainable selection; no recursive delegation or dependency on real Swift/Julia assets.
- Focused validation: Cross-adapter schema/effect rejection, explicit override, preference/ambiguity, stale/unavailable bindings, data policy, and no repeated side effects.
- Handoff: Remove ADP-06 after preserving contract/evidence in code, tests, and documentation.

## ADP-07 — Document and release the generic router

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-06
- Exact write set: `docs/src/inference/router.md`, `docs/src/inference/capability-parity.md`, `docs/src/SUMMARY.md`, `CommandLineHelp.md`, `.wiki/features/ask-executor-adapters.md`, `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: Router release gate
- Outcome: Publish registration examples, configuration, selection rules, supported policies, and measured coverage. Release independently of Swift and Julia availability.
- Focused validation: Serialized lint/full suite, narrow/combined features, WASM, help/docs, and explicit available-transport smoke tests. Record unsupported/unvalidated profiles; a failed required gate keeps this phase open.
- Handoff: Retire this plan and its index, network, and lane entries after generic-adapter acceptance.
