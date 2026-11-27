---
type: Plan
title: Claude Code ACP agent profile
description: Add Claude Code as a third trusted ACP agent beside Codex and OpenCode for `infer run`, routed executor bindings, diagnostics, and opt-in interoperability tests.
difficulty: medium
impact: medium
package: [acorn-schema, acorn-lib, acorn-cli]
tags: [feature, inference, acp, agent, claude]
status: planned
execution: phased
phase_budget_hours: 5
created: "2026-11-24"
updated: "2026-11-26"
---

# Goal

Let users run ACORN inference through Claude Code wherever Codex and OpenCode are accepted today: `acorn infer run --agent claude`, explicitly configured `{"kind": "acp", "profile": "claude"}` executor bindings, `acorn doctor` installation checks, and the ignored live ACP interoperability test. Claude runs through the external `claude-agent-acp` adapter under the same deny-mutation permission policy, typed prompt contract, response-schema validation, cancellation, and secret-free provenance as the existing profiles. No agent is enabled by discovery alone.

# Current evidence — November 24, 2026

Checked against `2369335b` plus the current worktree:

- `AgentProfile` in `crates/acorn-lib/src/io/acp/mod.rs` defines launch arguments, environment, executable, version probe, and missing-executable help. `Codex` (executable `codex-acp`, environment `INITIAL_AGENT_MODE=read-only`) and `OpenCode` (`opencode acp`) are the only implementations. `AgentProcessConfig::codex` and `::opencode` construct them.
- `Software` in `crates/acorn-schema/src/software.rs` has `Codex`, `CodexAcp`, and `OpenCode` identities and a 13-entry `SOFTWARE` diagnostic list; there is no Claude identity.
- Trust is enforced in two places that must agree: `ExecutorConfiguration::Acp` validation in `crates/acorn-lib/src/io/mcp/configuration.rs` hard-codes `"codex" | "opencode"`, and `AcpProfile` in `crates/acorn-lib/src/io/mcp/tools/acp.rs` repeats the same identifiers for routed registration. `crates/acorn-lib/src/io/config/tests/mod.rs` proves an untrusted profile is rejected.
- `AcpAgentKind` in `crates/acorn-cli/src/cli/arguments/mod.rs` offers `opencode` (default) and `codex`; `crates/acorn-cli/src/commands/infer/mod.rs` maps them to launch profiles.
- `crates/acorn-lib/tests/acp_interoperability.rs` runs a generic `run_agent_interoperability::<A: AgentProfile>()` with one ignored test per agent. `make test-acp` accepts only `ACP_AGENT=codex|opencode` and preflights `codex-acp`.
- Unrelated existing Claude identifiers: `Harness::ClaudeCode` (benchmark harness) and the `Anthropic` provider in `crates/acorn-schema/src/agent/mod.rs`. They describe catalog metadata, not launchable agents, and stay unchanged.
- Upstream adapter: npm `@agentclientprotocol/claude-agent-acp` 0.111.0 publishes the `claude-agent-acp` binary. The earlier `@zed-industries/claude-code-acp` package is deprecated in its favor. Codex's adapter moved to the same `@agentclientprotocol` scope.

Not yet verified (CLA-01): the adapter's version flag, advertised authentication methods, session modes, whether it routes every edit and command through ACP permission requests when user Claude settings contain allow rules, model selection, and usage reporting.

# Boundaries

In scope: one ACP launch profile, its software identities, trusted-profile validation, CLI agent selection, diagnostics, the opt-in live test, and user documentation.

Out of scope, each needing its own plan if wanted:

- `acorn sync` targets for Claude Code configuration (MCP servers, models). Codex has no sync target either; parity here is ACP execution.
- Installing ACORN skills into `.claude/skills`.
- A direct Anthropic Messages API endpoint binding. Endpoint bindings remain OpenAI-compatible.
- Bundling or installing the adapter. Users install it like `codex-acp`.

# Decisions

- Agent identifier `claude`; `Software::Claude` (Claude Code CLI) and `Software::ClaudeAgentAcp` (adapter executable `claude-agent-acp`), mirroring `Codex`/`CodexAcp`.
- Replace the two hard-coded trusted lists with one source owned by the routed ACP module, so adding a profile changes one place. Configuration validation and error text derive from it.
- Read-only enforcement must not depend on the user's Claude settings. If CLA-01 shows client permission denial is not authoritative, the profile sets the adapter's most restrictive session mode or isolating environment, and the interoperability test must prove the input is unmodified.
- Claude usage counts against the user's Claude subscription or API key. ACORN never reads, stores, or forwards Anthropic credentials; authentication stays with the adapter.

# Dependencies and ownership

- CLA-01 needs a host with Node, `claude-agent-acp`, and an authenticated Claude account. It changes no product code.
- CLA-02 starts after CLA-01. CLA-03, CLA-04, and CLA-05 each need only CLA-02 and may run in parallel because their write sets and locks differ.
- CLA-03 takes the Ask capability routing lock. Serialize with [generic ask executor adapters](ask-executor-adapters.md) ADP-06/07 if either edits `io/mcp/tools/acp.rs` or `io/mcp/configuration.rs` at the same time; CLA does not wait for PAR-16.
- CLA-04 takes the CLI contract lock; CLA-05 takes the CI/build lock for `Makefile`; CLA-06 takes Documentation generation.

# Acceptance criteria

- `acorn infer run <prompt> <input> --agent claude` returns validated output through `claude-agent-acp` without modifying the input file.
- An executor binding with `"profile": "claude"` validates and registers; unknown profiles are still rejected with the updated message.
- `acorn doctor` reports Claude Code and the adapter like the Codex pair, including actionable install help when missing.
- `make test-acp ACP_AGENT=claude` passes on an authenticated host; default `make test` makes no network or agent call.
- Documentation lists all three agents with install, authentication, cost, and read-only notes.

# Execution phases

## CLA-01 — Record Claude ACP adapter behavior

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: Host with Node, `npm install -g @agentclientprotocol/claude-agent-acp`, and an authenticated Claude account
- Exact write set: `.wiki/features/claude-acp-agent.md`
- Primary integration lock: none
- Outcome: Current evidence records the adapter version and `--version` behavior; the ACP `initialize` result (agent name/version, protocol version, authentication methods); available session modes and models; and whether file writes and terminal commands always raise ACP permission requests that a denying client blocks, including with permissive allow rules in user Claude settings. Decisions name the exact launch arguments, environment, and version arguments for CLA-02.
- Focused validation: Run ACORN's existing readiness handshake against the adapter by constructing `AgentProcessConfig::new("claude", "claude-agent-acp", <dir>)` in a scratch test or example, plus one denied-write prompt on a disposable copy of a fixture; record commands and observed results.
- Handoff: Update this plan's evidence and decisions, set CLA-02 to `ready`, or record the blocker if read-only behavior cannot be enforced.

## CLA-02 — Add Claude software identities and launch profile

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CLA-01
- Exact write set: `crates/acorn-schema/src/software.rs`, `crates/acorn-lib/src/io/acp/mod.rs`, `crates/acorn-lib/src/io/acp/tests.rs`
- Primary integration lock: none (leaf ACP profile)
- Outcome: `Software::Claude` and `Software::ClaudeAgentAcp` with display names, parsing, executable names, and `SOFTWARE` entries; `AgentProfile for Claude` with the CLA-01 arguments, environment, and npm install help; `AgentProcessConfig::claude`. Fake-agent tests cover launch data, missing-executable help, permission denial, and timeout for the new profile.
- Focused validation: `cargo nextest run -p acorn-schema software`, `cargo nextest run -p acorn-lib --features acp io::acp`, and the doctor tests affected by the larger `SOFTWARE` list; `cargo clippy -p acorn-lib --features acp --no-deps`.
- Handoff: Set CLA-03, CLA-04, and CLA-05 to `ready`.

## CLA-03 — Trust the Claude profile for routed executors

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CLA-02
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/acp.rs`, `crates/acorn-lib/src/io/mcp/configuration.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/acp.rs`, `crates/acorn-lib/src/io/config/tests/mod.rs`
- Primary integration lock: Ask capability routing
- Outcome: `AcpProfile::Claude` registers through the existing typed prompt contract, deny-mutation policy, model selection, and provenance. Configuration validation reads the same trusted-profile source instead of a second literal list, and its message names all trusted profiles.
- Focused validation: `cargo nextest run -p acorn-lib --features acp,agentic mcp::tools` and the executor configuration tests in `io::config`.
- Handoff: Record test names and results; no CLI change is required for configured bindings.

## CLA-04 — Select Claude from the CLI

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CLA-02
- Exact write set: `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/commands/infer/mod.rs`, `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`, `CommandLineHelp.md`
- Primary integration lock: CLI contract
- Outcome: `AcpAgentKind::Claude` (`--agent claude`) maps to the Claude profile and honors `--agent-executable`; help text and examples list the third agent; the default stays `opencode`.
- Focused validation: `cargo nextest run -p acorn-cli commands::infer cli::tests`, then `make help-cli` and review the `CommandLineHelp.md` diff.
- Handoff: Note any argument docs that still say “codex or opencode”.

## CLA-05 — Add the opt-in live interoperability check

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CLA-02; an authenticated Claude account for the live run
- Exact write set: `crates/acorn-lib/tests/acp_interoperability.rs`, `Makefile`
- Primary integration lock: CI/build
- Outcome: An ignored `test_claude_acp_returns_text_and_validated_fix_suggestions_without_mutating_input` runs the generic interoperability check for `Claude`. `make test-acp` accepts `ACP_AGENT=claude`, preflights `claude-agent-acp` with install guidance, and still rejects other values.
- Focused validation: `make test-acp ACP_AGENT=claude` on the authenticated host; `make test` confirms the test stays ignored by default.
- Handoff: Record adapter version, model, duration, and result; never record credentials or account identifiers.

## CLA-06 — Document and accept three-agent support

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CLA-03, CLA-04, CLA-05
- Exact write set: `docs/src/inference/README.md`, `docs/src/cli-reference/commands/infer.md`, `docs/src/cli-reference/commands/doctor.md`
- Primary integration lock: Documentation generation
- Outcome: Docs describe Claude installation, authentication through Claude Code, subscription or API usage cost, read-only behavior, and executor configuration beside Codex and OpenCode. Every acceptance criterion above has recorded evidence.
- Focused validation: `make book`; rerun `make test-acp` for all three agents where authenticated hosts are available, recording any agent not rerun.
- Handoff: Retire this plan and remove its index, network, and lane entries.
