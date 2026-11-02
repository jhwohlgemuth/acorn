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

Evidence checked on 2026-11-01 against `da1d04be` and the current worktree:

- `ToolRegistry` has versioned capability and binding definitions, deterministic selection, typed one-terminal execution, and focused registry/execution tests. Routed `ask`/`infer --route` currently registers native `acorn.version` and local `acorn.check`; interactive metadata chat also invokes the native check binding. Native parity still gates the heterogeneous release.
- The outbound MCP client provides configured allowlists, discovery, calls, policy checks, cancellation, and bounded cleanup. Direct `infer run` has ACP and OpenAI-compatible backends; Ollama now uses the existing OpenAI-compatible client with local chat defaults and distinct provenance. These direct paths do not register capability bindings.
- `ApplicationConfiguration.executors` stores explicit, non-secret capability/binding IDs and tagged MCP, trusted ACP, named endpoint, or sidecar selectors. Validation rejects duplicate or malformed IDs, unallowlisted MCP tools, unknown endpoint names, unsupported executor kinds, and untrusted ACP profiles. JSON, JSONC, and YAML parsing and JSON round trips have focused tests. Parsing performs no provider discovery.
- `ToolRegistry` now admits provider-declared runtime JSON Schemas through the same canonical contract checks as typed bindings. It rejects invalid schemas, conflicting capability contracts, duplicate binding IDs, and version/schema/effect mismatches while preserving deterministic selection. Cooperative capability handlers receive caller cancellation and get a four-second bounded cleanup window; native handlers retain drop cancellation.
- `ToolRegistry::register_mcp_bindings` constructs the outbound client from only explicitly referenced MCP server configurations and tool allowlists, discovers each selected server once, and registers only the exact configured tools. Configured IDs and local effect policy remain authoritative; server descriptions and annotations are ignored for routing policy. Calls retain `McpClientBackend` policy and limits, require declared structured output, map `is_error` to tool failure, never fall back, and finish client cleanup on cancellation. Fake-backend adapter tests and existing client pagination/cursor tests pass.
- Routed CLI startup still builds only native `acorn.version` and local `acorn.check`, has no `--config` field on `Infer`, and hard-codes `allow_remote: false`. Existing `ApplicationConfiguration::load(&None)` provides project discovery without new CLI syntax. ADP-09 must load and validate that configuration without contacting or starting providers, then compose the landed adapter registrations.
- The optional `slm` feature now compiles the shared sidecar root and an empty `sidecar::slm` namespace independently of `needle`; the CLI forwards the opt-in feature. Minimal `slm`, existing `needle`, combined `needle,slm`, and CLI `slm` checks pass, and the default library dependency tree remains unchanged.

# Dependencies and boundaries

Implementation scheduling on 2026-11-01 places ADP-03 next after the optional sidecar feature and configured MCP adapter handoffs. This is a queue order for shared ownership, not a technical dependency.

ADP-03 through ADP-05 may proceed while native capability phases continue. ADP-09 composes the landed MCP adapter and those remaining adapter contracts through the existing CLI route syntax; it cannot precede them because configuration contains selectors rather than executable handlers or capability schemas. ADP-06 waits for ADP-09 and PAR-16 before evaluating heterogeneous selection. MCP, ACP, OpenAI-compatible endpoint, and local sidecar adapters use their existing service contracts and do not change those providers' policy. Direct Ollama inference is an available endpoint transport, not a routed binding. The [parallel guide](../parallel-execution.md) serializes shared registry, CLI, manifest, and documentation roots.

# Validation and acceptance criteria

Validate each adapter with a fake configured provider, unavailable and denied routes, bounded cancellation, and secret-free provenance. Cover local Ollama and remote OpenAI-compatible endpoint policy separately. Release the generic router only when the heterogeneous selection matrix passes and documentation describes explicit configuration and failure behavior; real Swift and Julia profiles retain their own release gates.

# Execution phases

## ADP-03 — Adapt trusted ACP agent capabilities

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: None
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/acp.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/acp.rs`
- Primary integration lock: Ask capability routing
- Outcome: Bind named tasks to trusted ACP profiles through InferenceBackend and prompt hydration; preserve deny-mutation policy, configured model/agent, schema support, and provenance. Do not grant arbitrary agent authority.
- Focused validation: Fake sessions for capability mismatch, permission requests, selected model, refusal, timeout/cancellation cleanup, malformed structured output, and final result mapping.
- Handoff: Remove ADP-03 after preserving contract/evidence in code, tests, and documentation.

## ADP-04 — Adapt configured inference endpoints

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: None
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/inference.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/inference.rs`
- Primary integration lock: Ask capability routing
- Outcome: Bind named tasks to the existing OpenAI-compatible `InferenceBackend`, including explicitly configured Ollama endpoints, with a configured endpoint/model, data policy, and validated outputs. Reuse the direct backends without a new provider SDK or implicit fallback.
- Focused validation: Mock refusal/failure, malformed output, unsupported schema/attachments, remote offline rejection versus allowed local Ollama, cancellation, provenance, and binding preference.
- Handoff: Remove ADP-04 after preserving contract/evidence in code, tests, and documentation.

## ADP-05 — Prove reusable local sidecar registration

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: None
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/sidecar.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/sidecar.rs`
- Primary integration lock: Ask capability routing
- Outcome: Adapt SidecarAssets/Sidecar into registered handlers with verified installed assets, target availability, bounded lifecycle, and typed final output. Use deterministic fixture runners; real SLMs remain model-profile work.
- Focused validation: Readiness, input/output contracts, missing/altered assets, unsupported hosts, timeout/cancellation/process cleanup, exactly one invocation, and no implicit acquisition.
- Handoff: Remove ADP-05 after preserving contract/evidence in code, tests, and documentation.

## ADP-09 — Compose configured bindings at the CLI

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-03, ADP-04, ADP-05, and the landed CLI route syntax
- Exact write set: `crates/acorn-cli/src/commands/infer/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`
- Primary integration lock: CLI contract
- Outcome: Load and validate discovered ACRE configuration, compose the native registry with only its explicitly configured adapter bindings, and derive remote eligibility from the frozen adapter/configuration policy without changing CLI syntax or provider transports. Registry construction performs no unselected-provider discovery, startup, or inference.
- Focused validation: Configured composition across all four adapter kinds, missing/default configuration, unavailable providers, offline and remote-policy denial, duplicate/unknown contracts, and proof that startup performs no provider call beyond adapter-declared readiness work.
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
