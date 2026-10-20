---
type: Plan
title: Generic ask executor adapters
description: Bind configured MCP tools, ACP agents, inference endpoints, and local sidecars to the native capability router.
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [feature, inference, routing, mcp, acp, sidecar]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Extend the [native capability router](needle-router-native-parity.md) with explicit configured executor bindings. Keep its capability schemas, authority checks, one-terminal-operation rule, provenance, and bounded cancellation. No external tool or model is enabled by discovery alone.

# Current evidence

The outbound MCP client already provides configured discovery and calls with cancellation. ACP and OpenAI-compatible inference backends exist. Shared sidecar asset and runtime traits exist. RTR-02 must freeze registry and execution contracts before configured adapter work starts; native parity remains a release prerequisite.

# Dependencies and boundaries

ADP-01 follows RTR-02; ADP-09 also follows RTR-03. The four adapter leaves may proceed after ADP-08 while native capability phases continue, but ADP-06 waits for PAR-16 before evaluating heterogeneous selection. MCP, ACP, endpoint, and local sidecar adapters use their existing service contracts and do not change those providers' policy. The [parallel guide](../parallel-execution.md) serializes shared registry, CLI, manifest, and documentation roots.

# Validation and acceptance criteria

Validate each adapter with a fake configured provider, unavailable and denied routes, bounded cancellation, and secret-free provenance. Release the generic router only when the heterogeneous selection matrix passes and documentation describes explicit configuration and failure behavior.

# Execution phases

## ADP-01 — Define executor configuration

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RTR-02
- Exact write set: `crates/acorn-lib/src/io/config/application.rs`, `crates/acorn-lib/src/io/config/tests/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/configuration.rs`, `crates/acorn-lib/src/io/mcp/tools/configuration_tests.rs`
- Primary integration lock: API/config roots
- Outcome: Validate explicit capability bindings to configured MCP aliases, trusted ACP profiles, endpoints, and sidecars without discovery or secrets during parsing.
- Focused validation: Duplicate/invalid bindings, unavailable executor kinds, secret-free descriptors, and configuration round trips.
- Handoff: Remove ADP-01 after preserving the configuration contract; ADP-08 owns feature wiring.

## ADP-08 — Add optional sidecar feature wiring

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-01
- Exact write set: `crates/acorn-lib/Cargo.toml`, `crates/acorn-lib/src/io/mod.rs`, `crates/acorn-lib/src/io/sidecar/mod.rs`, `crates/acorn-lib/src/io/sidecar/slm/mod.rs`, `crates/acorn-cli/Cargo.toml`
- Primary integration lock: Workspace graph
- Outcome: Expose optional sidecar modules independently of Swift artifacts so Julia and generic adapters can compile on their own.
- Focused validation: Minimal and combined feature checks, no new default dependency edge, and an empty sidecar registry build.
- Handoff: Remove ADP-08 and release the workspace graph lock.

## ADP-09 — Compose configured bindings at the CLI

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-08, RTR-03
- Exact write set: `crates/acorn-cli/src/commands/infer/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`
- Primary integration lock: CLI contract
- Outcome: Build the caller-owned registry from validated configuration without changing CLI syntax or provider transports.
- Focused validation: Configured binding composition, unavailable providers, and no discovery during startup.
- Handoff: Remove ADP-09 and release the CLI contract lock.

## ADP-02 — Adapt configured MCP tools

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-08
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/mcp.rs`, `crates/acorn-lib/src/io/mcp/tools/mcp_tests.rs`
- Primary integration lock: Ask capability routing
- Outcome: Use McpClientBackend discovery/calls with collision-safe IDs, local policy, schemas, cancellation, and bounded content. Preserve semantic tool errors; ask eligibility does not imply inbound MCP re-export.
- Focused validation: Mock pagination/schema drift, name collisions, untrusted metadata, offline/allowlist rules, is_error, limits, cancellation cleanup, and no provider fallback.
- Handoff: Remove ADP-02 after preserving contract/evidence in code, tests, and documentation.

## ADP-03 — Adapt trusted ACP agent capabilities

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-08
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/acp.rs`, `crates/acorn-lib/src/io/mcp/tools/acp_tests.rs`
- Primary integration lock: Ask capability routing
- Outcome: Bind named tasks to trusted ACP profiles through InferenceBackend and prompt hydration; preserve deny-mutation policy, configured model/agent, schema support, and provenance. Do not grant arbitrary agent authority.
- Focused validation: Fake sessions for capability mismatch, permission requests, selected model, refusal, timeout/cancellation cleanup, malformed structured output, and final result mapping.
- Handoff: Remove ADP-03 after preserving contract/evidence in code, tests, and documentation.

## ADP-04 — Adapt configured inference endpoints

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-08
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/inference.rs`, `crates/acorn-lib/src/io/mcp/tools/inference_tests.rs`
- Primary integration lock: Ask capability routing
- Outcome: Bind named tasks to existing OpenAI-compatible InferenceBackend with configured endpoint/model, explicit data policy, and validated outputs. No new provider SDK or implicit fallback.
- Focused validation: Mock refusal/failure, malformed output, unsupported schema/attachments, offline rejection, cancellation, and binding preference.
- Handoff: Remove ADP-04 after preserving contract/evidence in code, tests, and documentation.

## ADP-05 — Prove reusable local sidecar registration

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-08
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/sidecar.rs`, `crates/acorn-lib/src/io/mcp/tools/sidecar_tests.rs`
- Primary integration lock: Ask capability routing
- Outcome: Adapt SidecarAssets/Sidecar into registered handlers with verified installed assets, target availability, bounded lifecycle, and typed final output. Use deterministic fixture runners; real SLMs remain model-profile work.
- Focused validation: Readiness, input/output contracts, missing/altered assets, unsupported hosts, timeout/cancellation/process cleanup, exactly one invocation, and no implicit acquisition.
- Handoff: Remove ADP-05 after preserving contract/evidence in code, tests, and documentation.

## ADP-06 — Validate heterogeneous executor selection

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-02, ADP-03, ADP-04, ADP-05, ADP-09, PAR-16
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/capability_tests.rs`, `crates/acorn-lib/src/io/mcp/tools/execution_tests.rs`, `tests/fixtures/routing/executors.json`, `docs/src/inference/routing-evaluation.md`
- Primary integration lock: Router release gate
- Outcome: Prove compatible task bindings across native/SLM/MCP/ACP/endpoint fixtures without router changes. Compare task results and explainable selection; no recursive delegation.
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
