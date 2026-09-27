---
type: Plan
title: "ACP inference and verified fixes"
description: "Add trusted deterministic remediation, opt-in interoperability, and CLI/doctor/TUI integration on the existing inference and fix-suggestion foundation."
difficulty: high
impact: high
package: [workspace, acorn-schema, acorn-lib, acorn-cli, acorn-tui]
tags: [feature, inference, acp, opencode, openai, fixes, doctor]
status: in-progress
priority: next
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/analyzer/{fix,host/discovery}/`, `crates/acorn-lib/src/io/workflow/`, `crates/acorn-cli/src/commands/{check,doctor,serve}/`, CLI parsing/help, `crates/acorn-tui/src/{app.rs,export.rs,screens/doctor.rs}`, manifests, command documentation, and focused tests.
- **Create:** trusted planner, executor, verifier, and recipe leaves inside `crates/acorn-lib/src/analyzer/fix/`; ignored real-adapter smoke tests under the existing integration-test structure; `crates/acorn-cli/src/commands/prompt/` when the CLI slice begins.
- **Delete:** `crates/acorn-lib/src/doctor/mod.rs` after discovery-backed CLI/TUI callers land; remove the `doctor --fix` not-implemented path when the shared planner is functional.

Shared integration locks are `crates/acorn-lib/src/{lib.rs,io/mod.rs,io/api/json_rpc/mod.rs}`, `crates/acorn-lib/src/analyzer/host/discovery/mod.rs`, workspace and crate manifests, `crates/acorn-cli/src/cli/{mod.rs,arguments/mod.rs}`, `crates/acorn-cli/src/commands/mod.rs`, `CommandLineHelp.md`, and the doctor/TUI report views. Keep one integration owner across phases 6–8.

# Dependencies

- Preserve the implemented prompt hydration, transport-neutral inference, ACP, OpenAI-compatible, operation-registry, and proposal-only `analyzer::fix` contracts.
- Reuse `io::workflow::transaction` for staged document writes and rollback, adding only the minimum crate-visible seam required by the executor.
- Reuse `io::SafePath`, `acorn-cmd::CommandSpec`, the current RAD/Vale/Chromium operations, and existing command discovery/version probes instead of creating parallel abstractions.
- Follow the ownership, portability, API, CLI composition, and feature constraints in the [related plans](#related-plans).
- The [MCP client](mcp-client.md) follows this plan’s exit gate because both plans share registry, manifest, module-index, and CLI roots.

# Goal

Finish the verified-fix lifecycle on the existing inference foundation: transform trusted deterministic proposals into policy-checked plans, execute only approved fixed operations, verify outcomes, represent doctor diagnostics as discovery findings, validate real adapters behind explicit gates, and expose the workflow through CLI and TUI views without granting model output execution authority.

# Current baseline

- `acorn::agent` owns strict prompt hydration and the transport-neutral inference request/result/backend contracts.
- `acorn::io::acp` uses the official ACP SDK with immutable agent launch data, installation and protocol readiness, bounded sessions, denied-by-default mutation permissions, and deterministic fake-agent coverage.
- `acorn::io::api::openai` implements explicitly selected Responses and Chat Completions modes with local schema validation and loopback-only offline access.
- The shared operation registry exposes explicitly authorized `acorn.inference.run`, `acorn.inference.readiness`, and `acorn.fixes.suggest` operations.
- `acorn::analyzer::fix` owns bounded check/discovery references, non-executable proposals, grouping, validation, and suggestion orchestration. Inference output cannot deserialize executable commands.
- `analyzer::host::discovery::Record` does not yet represent doctor-related findings.
- `acorn doctor` still has a separate display-oriented module; `doctor --fix` is unimplemented, and the TUI duplicates doctor report data.
- RAD formatting, Vale replacement data and installation, Chromium installation, and command execution already exist behind ACORN-owned paths that trusted providers can adapt.

# Locked decisions

- Keep proposal, planning, recipe, execution, and verification code under `acorn::analyzer::fix`; do not create another fix root or a doctor-specific fix model.
- Add doctor-related `Record::Finding` values to analyzer discovery. Keep `acorn::doctor` only as a temporary compatibility facade and delete it after CLI/TUI migration.
- Planning is a pure transformation over trusted deterministic proposals and locally validated provider output. Executable types stay private and do not implement `Deserialize`.
- Inference-generated fixes remain suggestions. Model output may reference a registered recipe but cannot define or alter executable paths, repositories, package names, or arguments.
- Native execution rechecks source fingerprints and preconditions, uses fixed argument vectors, records redacted output, and reports resolution only after verification succeeds.
- `doctor --fix` may apply only safe reversible deterministic fixes. Package-manager, network, elevation, restart, or non-reversible effects require interactive selection and confirmation. `doctor --interactive` remains report-only.
- The only new package-manager recipes in this plan are Node through WinGet `OpenJS.NodeJS.LTS` on Windows ([WinGet manifest](https://github.com/microsoft/winget-pkgs/blob/master/manifests/o/OpenJS/NodeJS/LTS/24.13.1/OpenJS.NodeJS.LTS.installer.yaml)) and Homebrew `node` on macOS ([Homebrew formula](https://formulae.brew.sh/formula/node)). Linux and other package managers, Git, Pandoc, and OpenCode remain manual. Vale and Chromium reuse their existing installers.
- ACP inference remains unavailable offline because ACORN cannot prove the configured OpenCode provider is local. OpenAI-compatible inference is permitted offline only for an explicitly configured loopback endpoint.
- Real-adapter tests remain ignored and require explicit environment gates. Ordinary tests use synthetic content, fake processes, and fake HTTP services.
- CLI parsing, generated help, and TUI locks begin only in phase 8.

# Active implementation slices

| Slice | Atomic write set | Exit evidence |
|---|---|---|
| 6A. Trusted planning and discovery findings | Extend `crates/acorn-lib/src/analyzer/fix/` with private executable plans, policy, outcomes, and verifier contracts; add doctor-related findings to `crates/acorn-lib/src/analyzer/host/discovery/mod.rs`; retain current table and JSON output through a temporary facade. | Pure planner tests cover dependencies, conflicts, Node/npm/npx coalescing, risk/effect ceilings, stale preconditions, preview output, and conversion of system, network, configuration, and software diagnostics into discovery findings. |
| 6B. Native executors and curated recipes | Extend `analyzer::fix` with document, download, package, and registry leaves; adapt `io::workflow::transaction`, `SafePath`, fixed `CommandSpec` vectors, existing RAD/Vale/Chromium operations, and isolated fake runners. | Temporary-directory tests prove apply, verification, and rollback; fake WinGet, Homebrew, Vale, and Chromium runners prove approval, offline, elevation, timeout, redaction, and post-fix verification without changing the host. |
| 7. Opt-in interoperability | Add only ignored smoke tests and required test-target manifest entries. | Tests run solely through the named environment gates; both real adapters return a non-empty schema-valid fix result, reap processes, and leave synthetic inputs unchanged. |
| 8A. Prompt and suggestion CLI | Add `acorn prompt run` and extend `acorn check --suggest-fixes`; preserve the Needle-backed `acorn infer` command. | Parsing and integration tests cover dry-run, stdin/file data, ZON/JSON prompt data, backend selection, readiness failures, clean structured output, and suggestion-only behavior. |
| 8B. Doctor/TUI and release surfaces | Migrate doctor/TUI views and exports to discovery records and `analyzer::fix`; delete `crates/acorn-lib/src/doctor/mod.rs`; update manifests, help, and documentation. | All three doctor modes use one discovery/planner lifecycle, capability-scoped tests pass, and help/documentation are regenerated. |

Landing order is strict: 6A planner and discovery findings, 6B native executors and recipes, 7 ignored interoperability tests, then 8A/8B CLI and TUI integration. Each slice passes its focused tests and relevant feature checks before the next slice takes a shared lock.

# Target design

## Fix planning and execution

Trusted planning resolves declarative fixes into private executable actions:

```text
Fix proposals
  -> reject stale or unsupported proposals
  -> resolve known recipe IDs
  -> enforce policy and allowed effects
  -> coalesce dependencies and remove duplicates
  -> detect conflicts
  -> topologically order actions
  -> return immutable FixPlan and exact preview
```

The implementation keeps these responsibilities separate inside `analyzer::fix`:

- `FixProvider` maps supported analysis or discovery findings to declarative fixes without host mutation.
- `FixPlanner` resolves only trusted deterministic actions or known recipe IDs and produces an immutable plan.
- `FixExecutor` handles one action family, such as document edits, verified downloads, or package installation.
- `FixVerifier` reruns the originating check or discovery probe.
- `FixPolicy` enforces offline mode, approval, effects, risk, elevation, and reversibility.
- `ExecutableFix`, `FixPlan`, and `FixOutcome` remain non-deserializable trusted types with private executable fields.

Document changes use expected original values or fingerprints, confined paths, atomic writes, and recoverable backups where practical. Execution is the only mutating stage. A successful process exit records execution; only a successful verifier records resolution.

## Discovery-backed doctor integration

Extend the existing discovery stream with a finding record containing category, subject, status, observed and expected values, and structured secret-free details. Do not add `DoctorReference`, `DoctorFinding`, or `DoctorReport`. `analyzer::fix` copies only stable scalar fields into its source-fingerprinted `DiscoveryReference`.

```text
run diagnostics
  -> append doctor-related discovery findings
  -> collect deterministic Fix proposals
  -> plan under FixPolicy
  -> print exact preview
  -> obtain required confirmation
  -> execute trusted actions
  -> rerun source diagnostics
  -> render FixOutcome values
```

When Node, npm, and npx are absent, one Node recipe satisfies the three findings while retaining three verification probes. Package installation is never selected by safe-only non-interactive policy.

## Initial deterministic fixes

| Finding | Candidate fix | Automation | Verification |
|---|---|---|---|
| RAD differs from `ResearchActivity::format_with` | Atomically write the existing format transform | Safe automatic after a matching fingerprint | Re-read, validate, and compare normalized output |
| Supported Vale replacement with exact span and expected text | Apply the declared replacement | Safe automatic after stale-span check | Rerun the same Vale rule |
| Missing deterministic linked-data context | Apply the existing link augmentation | Confirmation required | Re-read and rerun schema/link checks |
| Vale missing for requested prose analysis | Reuse the verified Vale resolver | Confirmation required; network-sensitive | Resolve and probe Vale, then rerun preflight |
| Chromium missing for PDF export | Reuse the existing Chromium installer | Confirmation required; network and disk effects | Run the existing browser preflight |
| OpenCode missing for ACP | Emit manual instructions unless a separately accepted recipe exists | Manual | Run installation/version preflight |
| Node.js missing | Resolve only WinGet `OpenJS.NodeJS.LTS` or Homebrew `node` | Interactive | Probe Node, npm, and npx independently |
| Git or Pandoc missing | Emit manual instructions | Manual | Rerun the corresponding discovery probe |

Dead URLs, missing scientific content, ambiguous schema values, readability findings, crosswalk loss, and inferred prose rewrites remain suggestions because ACORN cannot prove the intended replacement.

## CLI contract

The remaining CLI slice adds:

```text
acorn prompt run summarize --data input.json
acorn prompt run translate --data input.yaml --set language=French
acorn prompt run summarize --data input.json --dry-run
acorn prompt run summarize --data input.json --backend acp --agent opencode
acorn prompt run summarize --data input.json --prompt-data-format json --dry-run
acorn check activity.json --suggest-fixes --backend acp --agent opencode
acorn check activity.json --suggest-fixes --backend openai --model local-model
```

- Select only embedded prompt assets and explicitly select `acp` or `openai`; do not fall back between backends or endpoint modes.
- Load JSON, YAML, or ZON object data from `--data <PATH|->`, otherwise use piped stdin or return a usage error. Apply repeated `--set KEY=VALUE` values after parsing while rejecting reserved keys and type conflicts.
- `--prompt-data-format zon|json` controls only the fenced model-facing data block.
- `--dry-run` performs no readiness, process, credential, or network work.
- Keep provider credentials out of arguments, logs, results, and provenance.
- Render suggestions separately from checks and never apply them from `check --suggest-fixes`.
- Preserve the existing positional-or-stdin Needle `acorn infer` workflow.

# Plan conflicts

| Plan or lane | Coordination rule |
|---|---|
| JSON-RPC and gRPC | Reuse registered service handlers. ACP retains its official envelopes, initialization, sessions, streaming, permissions, cancellation, and errors. |
| MCP server/client | Serialize agent schema, manifests, module indexes, and CLI roots; do not translate MCP and ACP envelopes or automatically re-export tools. |
| GitLab/webhooks | Permit only bounded suggestion operations; webhook authorization cannot grant ACP tool permission or fix-application authority. |
| Analyzer discovery | ACP owns the doctor-finding addition while dPID and enrichment work avoid the same discovery leaf. |
| Portability and crate extraction | Keep planning pure and native executors feature-gated. Record future owners without starting speculative extraction. |

# Delivery sequence

## 6. Implement deterministic planning and execution

1. Add pure planner, policy, preview, outcome, and private executable contracts inside `analyzer::fix`.
2. Add doctor-related discovery findings and retain current doctor renderers as temporary projections.
3. Adapt one deterministic document fix end to end with stale-input rejection, transaction-backed rollback, and verification.
4. Adapt RAD, Vale, and Chromium behavior through their existing ACORN-owned operations.
5. Add fixed WinGet and Homebrew Node recipes, coalesce Node/npm/npx, and make unsupported combinations manual.
6. Prove effect, approval, elevation, offline, timeout, conflict, redaction, rollback, and verification policy with fake runners and temporary directories.

Exit gate: deterministic fixtures can preview, apply, verify, and where supported roll back document fixes; fake WinGet, Homebrew, Vale, and Chromium tests invoke only fixed argument vectors and never change the test host.

## 7. Validate real interoperability

1. Add an ignored OpenCode smoke test gated by `ACORN_TEST_OPENCODE_ACP=1`.
2. Verify negotiated capabilities before asserting optional model, mode, or metadata behavior.
3. Use synthetic content and a local or sandbox model where practical.
4. Add an ignored OpenAI-compatible smoke test requiring `ACORN_TEST_OPENAI_COMPATIBLE=1`, `ACORN_TEST_OPENAI_BASE_URL`, and `ACORN_TEST_OPENAI_MODEL`; read a bearer credential only from `OPENAI_API_KEY`.

Exit gate: a hydrated prompt and one check-fix request return non-empty schema-valid results through both configured adapters, the ACP process is reaped, and neither route mutates its input.

## 8. Expose CLI and reusable task hooks

1. Add `acorn prompt run` with file/stdin data, `--set`, explicit backend configuration, ZON/JSON prompt data, dry-run, timeout, working-directory, and structured output controls.
2. Add suggestion-only `check --suggest-fixes` using the existing batch/context ceilings and explicit backend/model/agent selection.
3. Complete report-only interactive, safe non-interactive fix, and interactive confirmed-fix doctor modes over shared discovery records and `analyzer::fix`.
4. Migrate TUI and JSON views, delete the doctor compatibility module, and remove the not-implemented fix path.
5. Expose reusable event and permission hooks without adding arbitrary commands, automatic tool approval, inference-fix application, or long-lived sessions.
6. Regenerate help and update user documentation.

Exit gate: users can preview and run embedded prompts, request structured suggestions through either backend, and preview or explicitly apply registered deterministic remediations with verified outcomes.

# Validation

1. Test discovery correlation, stale inputs, effect declarations, dependency/conflict planning, Node/npm/npx coalescing, exact previews, and inability to deserialize executable actions.
2. Test atomic document edits and rollback, fixed command invocation, offline/network policy, approval/elevation, timeouts, non-zero exits, redaction, and mandatory verification with isolated fake runners.
3. Test CLI parsing and execution for file/stdin data, dry-run without OpenCode, missing readiness, unavailable API endpoints, capability-scoped doctor output, JSON output, safe-only fixes, interactive selection, declined approval, shared exports, invalid data, timeout, backend selection, and clean stdout.
4. Run ignored real-adapter smoke tests only through their explicit environment gates.
5. Run focused `analyzer::fix`, analyzer discovery, ACP, OpenAI, JSON-RPC, CLI doctor/check/prompt, and TUI tests, then `make lint` and `make test`.
6. Run minimal, alloc, analysis-only, ACP-only, combined agentic/ACP, default, and WASM checks. Confirm native executor dependencies stay outside portable builds.
7. Regenerate CLI help and build documentation when phase 8 lands; record accepted pre-existing audit or documentation failures without attributing them to this plan.

# Acceptance criteria

- Doctor-related system, network, configuration, and software observations use analyzer discovery records, and `analyzer::fix` retains their provenance and source fingerprints.
- Trusted deterministic fixes can be previewed, policy-checked, explicitly applied, rolled back where supported, and verified through one planner/executor lifecycle.
- Safe non-interactive runs execute only reversible actions; network, elevation, package-manager, restart, or non-reversible effects require interactive confirmation.
- One Node remediation satisfies Node, npm, and npx without duplicate installation. Windows resolves only WinGet `OpenJS.NodeJS.LTS`, macOS resolves only Homebrew `node`, and other combinations remain manual.
- Executors accept only trusted `ExecutableFix` plans. Every action rechecks preconditions and reruns its originating check or discovery probe before reporting resolution.
- CLI and TUI doctor views and exports consume the same discovery records, and no separate `acorn::doctor` module remains.
- Real smoke tests run only through `ACORN_TEST_OPENCODE_ACP=1` or `ACORN_TEST_OPENAI_COMPATIBLE=1` with explicit endpoint/model configuration and synthetic inputs.
- Native executor code remains outside portable schema/analysis builds, and tests never install packages or modify the real host environment.
- `acorn prompt run` supports preview and one-shot inference without changing Needle-backed `acorn infer`; `check --suggest-fixes` remains suggestion-only.
- `doctor --fix`, `doctor --fix --interactive`, and `doctor --interactive` follow the safe-auto, interactive-fix, and report-only policies above.
- Focused tests, supported feature checks, `make lint`, `make test`, generated help, and user documentation pass or record only accepted unrelated baselines.

# Completion handling

Keep this plan, its index entry, dependency edge, evidence, and acceptance criteria synchronized while phases 6–8 remain active. After every acceptance criterion passes, delete the plan and remove its entries from the feature index, plan network, and parallel-execution guide. Durable behavior belongs in code, tests, user documentation, and release history.

# Next plan

After this plan passes its exit gate, continue with the [MCP client](mcp-client.md), reusing the established operation policy and inference service boundary while retaining RMCP protocol ownership.

# Deferred decisions

- Whether a later release adds `fix suggest` as an alias or a broader fix-lifecycle command.
- Whether non-fix prompt outputs opt into schema validation.
- Which generated document-action subset, if any, may move beyond review-only suggestions.
- Whether later releases add Linux or other package-manager recipes.
- Whether fixes need durable identifiers across runs instead of request-local correlation and source fingerprints.
- Whether long-lived, resumed, or forked ACP sessions belong in CLI, TUI, or a later service API.
- Which additional agents provide direct ACP commands or require maintained bridges; each needs verified primary documentation and an opt-in interoperability test.

# Related plans

- [Crate boundary architecture](../refactors/crate-boundary-architecture.md)
- [Portability roadmap](../refactors/portability/roadmap.md)
- [Dependency feature defaults](../refactors/portability/dependency-feature-defaults.md)
- [API client roadmap](../refactors/api/roadmap.md)
- [CLI parse and execution models](../refactors/core/cli-parse-execution-models.md)
- [CLI startup separation](../refactors/core/cli-startup-separation.md)
- [CLI error boundary](../refactors/core/cli-error-boundary.md)
- [MCP client](mcp-client.md)
