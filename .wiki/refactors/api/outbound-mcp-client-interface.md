---
okf_version: "0.1"
type: Plan
title: Refine the outbound MCP client interface for embedders
description: Document the landed embedder interface and close its remaining portability evidence.
difficulty: medium
impact: high
package: [acorn-lib]
tags: [refactor, mcp, api, cancellation, embedding, interoperability]
status: in-progress
execution: phased
phase_budget_hours: 5
---

# Goal

Finish the embedder documentation and portability evidence for the landed outbound MCP client interface, then retire this plan. The public API and focused implementation tests are already in code.

# Current evidence

- `McpClientBackend` is implemented for `McpClient`. Direct `BTreeMap<String, McpServerConfiguration>` construction, cancellation-aware call/list operations, bounded child and HTTP cleanup, and reusable client state are present in `crates/acorn-lib/src/io/mcp/client.rs`.
- The focused client suite covers ready, initialization, pagination, pending-request, reuse, and child-reaping cases. The narrow `mcp-client` feature check and native portability checks were recorded as passing in the implementation change.
- `docs/src/packages/mcp-client.md` still shows application-root construction and does not document direct construction, the backend trait, or caller-driven cancellation. The `wasm32-unknown-unknown` target is installed locally; the remaining portability check has not been recorded for this interface.

# Dependencies and ownership

MCP-01 owns the book page and navigation. MCP-02 records the portability result and removes this plan from its directory index, portfolio, dependency network, and parallel lane. No client code or configuration change is part of these phases. The narrow feature graph and allowlist, effect, offline, timeout, output-limit, authentication, transport, discovery, and `CallToolResult.is_error` semantics are the accepted baseline.

# Acceptance criteria

- The package page shows direct map construction, `McpClientBackend` dispatch, optional caller cancellation, bounded cleanup, and semantic tool errors with compiling examples.
- The narrow client feature and portable/WASM checks have dated results. Any failure that reveals a product defect becomes a separately bounded repair phase with an exact write set.
- The plan is removed after those gates pass and durable documentation is linked from the book.

# Execution phases

## MCP-01 — Finish embedder documentation

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `docs/src/packages/mcp-client.md`, `docs/src/SUMMARY.md`
- Primary integration lock: book documentation
- Outcome: Document direct construction, backend dispatch, cancellation, cleanup, and preserved semantic-error behavior.
- Focused validation: Run the narrow `mcp-client` check/tests and `make doc`.
- Handoff: Remove MCP-01 and record the accepted documentation and narrow graph.

## MCP-02 — Complete portability evidence and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: MCP-01
- Exact write set: `.wiki/refactors/api/outbound-mcp-client-interface.md`, `.wiki/index.md`, `.wiki/refactors/api/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: portability validation
- Outcome: Run the remaining WASM/portable gate, confirm the downstream handoff, and remove the plan.
- Focused validation: Run `make check-portable`, the WASM check, focused clippy, and wiki validation.
- Handoff: Delete this plan and all index, network, and lane entries.
