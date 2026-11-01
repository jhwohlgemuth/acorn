---
type: Plan
title: Generic ask executor adapters
description: Bind configured MCP tools, ACP agents, OpenAI-compatible endpoints, and local sidecars to the native capability router.
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [feature, inference, routing, mcp, acp, sidecar]
status: planned
priority: next
execution: phased
phase_budget_hours: 5
---

# Goal

Extend the [native capability router](needle-router-native-parity.md) with explicit configured executor bindings. Keep its capability schemas, authority checks, one-terminal-operation rule, provenance, and bounded cancellation. No external tool or model is enabled by discovery alone.

# Current evidence

Evidence checked against the worktree on 2026-10-31:

- `ToolRegistry` has versioned capability and binding definitions, deterministic selection, typed one-terminal execution, and focused registry/execution tests. Routed `ask`/`infer --route` currently registers native `acorn.version` and local `acorn.check`; interactive metadata chat also invokes the native check binding. Native parity still gates the heterogeneous release.
- The outbound MCP client provides configured allowlists, discovery, calls, policy checks, cancellation, and bounded cleanup. Direct `infer run` has ACP and OpenAI-compatible backends; Ollama now uses the existing OpenAI-compatible client with local chat defaults and distinct provenance. These direct paths do not register capability bindings.
- `ApplicationConfiguration.executors` now stores explicit, non-secret capability/binding IDs and tagged MCP, trusted ACP, named endpoint, or sidecar selectors. Validation rejects duplicate or malformed IDs, unallowlisted MCP tools, unknown endpoint names, unsupported executor kinds, and untrusted ACP profiles. JSON, JSONC, and YAML parsing and JSON round trips have focused tests. Parsing performs no provider discovery. The tools directory still has no provider adapter modules; routed CLI startup still hard-codes `allow_remote: false`. The sidecar module exposes shared asset/runtime traits and Needle, with no optional SLM module or generic sidecar binding.

# Dependencies and boundaries

Implementation scheduling on 2026-11-01 places ADP-08 next after the interactive format handoff. This is a queue order for shared ownership, not a technical dependency; ADP-08 remains ready.

ADP-08 can consume the landed configuration contract; ADP-09 can consume the landed CLI route syntax after ADP-08. The four adapter leaves may proceed after ADP-08 while native capability phases continue, but ADP-06 waits for PAR-16 before evaluating heterogeneous selection. MCP, ACP, OpenAI-compatible endpoint, and local sidecar adapters use their existing service contracts and do not change those providers' policy. Direct Ollama inference is an available endpoint transport, not a routed binding. The [parallel guide](../parallel-execution.md) serializes shared registry, CLI, manifest, and documentation roots.

# Validation and acceptance criteria

Validate each adapter with a fake configured provider, unavailable and denied routes, bounded cancellation, and secret-free provenance. Cover local Ollama and remote OpenAI-compatible endpoint policy separately. Release the generic router only when the heterogeneous selection matrix passes and documentation describes explicit configuration and failure behavior; real Swift and Julia profiles retain their own release gates.

# Execution phases

## ADP-08 — Add optional sidecar feature wiring

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: the landed executor configuration contract
- Exact write set: `crates/acorn-lib/Cargo.toml`, `crates/acorn-lib/src/io/mod.rs`, `crates/acorn-lib/src/io/sidecar/mod.rs`, `crates/acorn-lib/src/io/sidecar/slm/mod.rs`, `crates/acorn-cli/Cargo.toml`
- Primary integration lock: Workspace graph
- Outcome: Expose optional sidecar modules independently of Swift artifacts so Julia and generic adapters can compile on their own.
- Focused validation: Minimal and combined feature checks, no new default dependency edge, and an empty sidecar registry build.
- Handoff: Remove ADP-08 and release the workspace graph lock.

## ADP-09 — Compose configured bindings at the CLI

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-08 and the landed CLI route syntax
- Exact write set: `crates/acorn-cli/src/commands/infer/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`
- Primary integration lock: CLI contract
- Outcome: Build the caller-owned registry from validated configuration without changing CLI syntax or provider transports.
- Focused validation: Configured binding composition, unavailable providers, and no discovery during startup.
- Handoff: Remove ADP-09 and release the CLI contract lock.

## ADP-02 — Adapt configured MCP tools

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-08
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/mcp.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/mcp.rs`
- Primary integration lock: Ask capability routing
- Outcome: Use McpClientBackend discovery/calls with collision-safe IDs, local policy, schemas, cancellation, and bounded content. Preserve semantic tool errors; ask eligibility does not imply inbound MCP re-export.
- Focused validation: Mock pagination/schema drift, name collisions, untrusted metadata, offline/allowlist rules, is_error, limits, cancellation cleanup, and no provider fallback.
- Handoff: Remove ADP-02 after preserving contract/evidence in code, tests, and documentation.

## ADP-03 — Adapt trusted ACP agent capabilities

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-08
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/acp.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/acp.rs`
- Primary integration lock: Ask capability routing
- Outcome: Bind named tasks to trusted ACP profiles through InferenceBackend and prompt hydration; preserve deny-mutation policy, configured model/agent, schema support, and provenance. Do not grant arbitrary agent authority.
- Focused validation: Fake sessions for capability mismatch, permission requests, selected model, refusal, timeout/cancellation cleanup, malformed structured output, and final result mapping.
- Handoff: Remove ADP-03 after preserving contract/evidence in code, tests, and documentation.

## ADP-04 — Adapt configured inference endpoints

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-08
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/inference.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/inference.rs`
- Primary integration lock: Ask capability routing
- Outcome: Bind named tasks to the existing OpenAI-compatible `InferenceBackend`, including explicitly configured Ollama endpoints, with a configured endpoint/model, data policy, and validated outputs. Reuse the direct backends without a new provider SDK or implicit fallback.
- Focused validation: Mock refusal/failure, malformed output, unsupported schema/attachments, remote offline rejection versus allowed local Ollama, cancellation, provenance, and binding preference.
- Handoff: Remove ADP-04 after preserving contract/evidence in code, tests, and documentation.

## ADP-05 — Prove reusable local sidecar registration

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-08
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/sidecar.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/sidecar.rs`
- Primary integration lock: Ask capability routing
- Outcome: Adapt SidecarAssets/Sidecar into registered handlers with verified installed assets, target availability, bounded lifecycle, and typed final output. Use deterministic fixture runners; real SLMs remain model-profile work.
- Focused validation: Readiness, input/output contracts, missing/altered assets, unsupported hosts, timeout/cancellation/process cleanup, exactly one invocation, and no implicit acquisition.
- Handoff: Remove ADP-05 after preserving contract/evidence in code, tests, and documentation.

## ADP-06 — Validate heterogeneous executor selection

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-02, ADP-03, ADP-04, ADP-05, ADP-09, PAR-16
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
