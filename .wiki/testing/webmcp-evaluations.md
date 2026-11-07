---
okf_version: "0.1"
type: Plan
title: Evaluate ACORN WebMCP tools with webmcp-evals
description: Add reproducible deterministic and model-backed evaluations for ACORN's live browser WebMCP tools without replacing the existing adapter and native-browser contract tests.
difficulty: medium
impact: medium
package: [acorn-wasm]
tags: [testing, webmcp, wasm, browser, agent-evaluation]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Use GoogleChromeLabs `webmcp-evals` against ACORN's staged browser demo to verify two distinct outcomes:

1. A deterministic, API-key-free smoke suite discovers and executes every registered ACORN WebMCP tool in a real supported Chrome page.
2. An explicitly invoked model-backed browser suite measures whether an agent selects the correct ACORN tool and exact arguments from natural-language requests, especially where two tools have similar domains.

Keep ACORN's existing Node adapter tests and native browser smoke as the authoritative checks for adapter behavior, strict schemas, annotations, input and output bounds, cancellation, timing, Permissions Policy, rendering safety, and cleanup. The external evaluator adds independent live-page interoperability and agent-selection evidence.

# Current evidence

Evidence checked on 2026-11-07 against `401ab463` and the current clean worktree:

- `crates/acorn-wasm/web/webmcp.js:15-41` names eight tools: FAIR assessment, readability, metadata crosswalk, content diff, persistent-identifier discovery, one-identifier inspection, metadata-standard validation, and Research Activity validation. Lines 579-655 register those tools through the imperative `document.modelContext.registerTool()` API that `webmcp-evals` supports.
- `crates/acorn-wasm/web/webmcp.test.mjs` uses a fake model context to cover adapter validation and registration without launching a browser. `make test-webmcp` runs that contract suite.
- `crates/acorn-wasm/examples/webmcp/smoke.js` and the `?smoke=1` demo route already exercise native registration, discovery, execution, serialization, strict schemas, annotations, bounds, timing, Permissions Policy, manual parity, and disposal in a supported browser. The reference run documented in `crates/acorn-wasm/examples/webmcp/README.md` passed with Chrome 154 on macOS on 2026-10-09.
- `make build-wasm` builds the browser WASM package, stages the demo under `target/acorn-wasm/webmcp/`, runs `make test-webmcp`, verifies staged package contents, and runs the WASM Node tests. The documented local origin is `http://127.0.0.1:8000/webmcp/` after `python3 -m http.server 8000 --directory target/acorn-wasm`.
- The upstream [`webmcp-evals` README](https://github.com/GoogleChromeLabs/webmcp-tools/blob/main/webmcp-evals/README.md) documents `smoke` as a deterministic live-page runner with no model or API key, `browser` as a live tool-calling evaluation, fresh-page isolation for each smoke case, constraint-based argument matching, JSON/HTML reports, and a nonzero exit when a browser case fails. It explicitly labels the framework experimental.
- The published [`webmcp-evals` package](https://github.com/GoogleChromeLabs/webmcp-tools/blob/main/webmcp-evals/package.json) is version `0.1.0`; `npm view webmcp-evals version dist-tags --json` returned `0.1.0` as `latest` on 2026-11-07. No ACORN package manifest, lockfile, eval suite, Make target, script, or CI job currently references it.
- The external evaluator matches requested calls and arguments. It does not replace ACORN assertions over returned envelopes or negative adapter behavior. Oversized, malformed, concurrent, cancelled, and hostile-rendering cases stay in the existing adapter/native smoke suites unless a later upstream evaluator version exposes a deterministic output assertion needed by ACORN.

# Boundaries and decisions

## In scope

- One deterministic `evals.json` suite with valid concrete calls for all eight current tools and natural-language messages reusable by `browser` mode.
- Selection cases that distinguish identifier discovery from one-identifier inspection, Research Activity validation from metadata-standard validation, FAIR assessment from validation, unified diff from JSON Patch, and crosswalk source/target requirements.
- A repository-local npm tool directory with an exact `webmcp-evals` version and committed lockfile. Do not rely on a floating `npx webmcp-evals` install for repeatable repository gates.
- One script that starts an isolated loopback server, waits for readiness, invokes the evaluator, preserves reports, stops only the server process it created, and fails clearly when staged artifacts, Chrome support, or tool discovery are unavailable.
- Named Make targets for deterministic smoke and opt-in model-backed browser evaluation.
- A path-filtered deterministic CI job only after a named runner proves the pinned Chrome/WebMCP configuration. Model-backed runs remain explicit because provider cost, credentials, and nondeterminism are outside ordinary CI.
- Durable developer documentation for prerequisites, commands, expected reports, version updates, and failure diagnosis.

## Out of scope

- Exporting a separate static schema solely to use `webmcp-evals local`; live `smoke` and `browser` modes consume the registered schemas directly.
- Replacing `webmcp.test.mjs`, the native `?smoke=1` suite, or `make build-wasm`.
- Changing ACORN tool names, descriptions, schemas, result envelopes, WASM behavior, or demo UI merely to satisfy one model. A repeatable selection failure may motivate a separate evidence-backed adapter change.
- Adding provider secrets to the repository, printing them in logs, or making paid model calls from normal test targets.
- Treating one model run as a product compatibility guarantee. Reports identify the evaluator version, Chrome version/channel, provider/model, run count, date, and suite revision.
- Adding `simulate` coverage. ACORN's current demo exposes read-only operations whose success is already represented by tool trajectories and tool-tab output; add simulation only under a separate plan if a stateful user workflow appears.

# Evaluation contract

## Deterministic suite

Use concrete, bounded inputs that complete locally and exercise the production WASM bindings. Each case contains a natural-language `messages` entry and one or more exact `expectedCall` entries. `smoke` executes the required calls directly; `browser` uses the same structure to evaluate agent selection. Keep each ordinary case independent because the evaluator opens a fresh page per smoke case.

The minimum matrix is:

| Case | Expected tool | Required evidence |
|---|---|---|
| Assess supplied JSON against FAIR indicators | `acorn_assess_fair_metadata` | Correct `json` and `standard`; optional stable `resource` covered once. |
| Check prose readability | `acorn_check_readability` | Exact prose and one supported metric. |
| Crosswalk metadata | `acorn_crosswalk_metadata` | Supported source/target/format plus required seed fields for the selected pair. |
| Compare text | `acorn_diff_content` | Separate unified and JSON Patch cases with exact `before`, `after`, and `mode`. |
| Find identifiers in prose | `acorn_discover_persistent_identifiers` | Text containing at least two supported identifier kinds. |
| Inspect one identifier | `acorn_inspect_persistent_identifier` | Exact value and one case with an explicit kind. |
| Validate a metadata standard | `acorn_validate_metadata_standard` | Complete content, format, and non-RAD standard. |
| Validate Research Activity JSON | `acorn_validate_research_activity` | Complete valid JSON and an explicit fuzzy policy in one case. |

The deterministic gate passes only when all required calls execute against the live page and the evaluator exits zero. Returned-value semantics remain covered by ACORN's existing tests.

## Agent-selection suite

Add focused prompts rather than a large prose corpus. Include direct cases for every tool plus ambiguity pairs. Do not mention implementation tool names in user messages. Pin the suite contents and require the report to retain exact observed trajectories.

An acceptance baseline uses one explicitly selected provider/model for five runs per case. All cases must match their required trajectory for that recorded baseline. This is release evidence, not a permanent CI threshold: later model drift produces a new dated report and does not justify silently weakening expected calls. The Make target requires `WEBMCP_EVALS_MODEL`; the selected backend's standard environment credential remains outside the command line and repository.

# Associated commands

These commands define the intended operator workflow. Phase WME-02 may refine option placement to match the pinned CLI, but it must preserve the same inputs, isolation, and report paths.

Build and serve ACORN manually:

```sh
make build-wasm
python3 -m http.server 8000 --bind 127.0.0.1 --directory target/acorn-wasm
```

Install the pinned evaluator after its repository-local lockfile lands:

```sh
npm ci --prefix tools/webmcp-evals
```

Run the evaluator directly while the manual server is active:

```sh
tools/webmcp-evals/node_modules/.bin/webmcp-evals \
  --chrome-channel chrome-canary \
  --reporter console json \
  --output-dir target/webmcp-evals/smoke \
  smoke \
  --url http://127.0.0.1:8000/webmcp/ \
  --evals crates/acorn-wasm/examples/webmcp/evals.json \
  --verbose
```

Run the eventual repository wrappers:

```sh
make build-wasm
make test-webmcp-evals-smoke
WEBMCP_EVALS_MODEL=openai:gpt-5 WEBMCP_EVALS_RUNS=5 make test-webmcp-evals-browser
```

The browser target writes JSON and HTML reports under `target/webmcp-evals/browser/`. Provider-specific credentials are supplied through the evaluator's documented environment variables; commands and logs must not echo their values.

Retain the existing focused checks:

```sh
make test-webmcp
make test-webmcp-package
make build-wasm
```

Validate the planning wiki while this plan is active:

```sh
python3 .agents/skills/planning/scripts/validate_wiki.py .
```

# Dependencies and coordination

- WME-01 can begin on the current eight-tool surface. It owns only new evaluation data and does not wait for other plans.
- WME-02 follows WME-01 and claims the CI/build lock for the Makefile change. It must serialize with coverage throughput, report visual regression, runner pools, Mise, Podman, and other phases editing `Makefile`.
- WME-03 follows the deterministic harness so agent failures can be separated from registration, launch, and execution failures. It does not require CI credentials.
- WME-04 requires a named CI runner with a supported Chrome channel and WebMCP enabled, plus accepted WME-02 smoke evidence. Missing runner/browser support blocks CI wiring without blocking local evaluation.
- Big-O BGO-07 may add a ninth WebMCP tool later. That phase must extend the then-current evaluation matrix or explicitly record why its new tool is outside this plan's accepted suite; neither plan edits `webmcp.js` concurrently.
- Upstream `webmcp-evals` is experimental. Version changes are deliberate lockfile updates followed by the deterministic suite, report-schema inspection, and one recorded model baseline. Do not accept a floating update during an unrelated ACORN change.

# Validation

- Parse each committed suite as JSON and verify every expected function name equals a tool registered by the staged page.
- Run `make test-webmcp`, `make test-webmcp-package`, and `make build-wasm` before the external evaluator so an adapter/package defect is reported at its owning layer.
- Run the deterministic evaluator twice from clean temporary Chrome profiles. Confirm all eight tools are discovered, every case executes, reports are written under `target/`, and the wrapper stops its loopback server after success and failure.
- Deliberately change one expected tool name and one required argument in a temporary worktree or disposable copy; verify the smoke command exits nonzero, then restore the accepted suite.
- Run the browser suite for five runs with the recorded provider/model. Inspect the JSON report for exact trajectories and ensure similar-domain prompts select the intended tool.
- On the candidate CI runner, record Chrome version/channel, WebMCP enablement, Node/npm versions, evaluator package/lock digest, job link, and two consecutive successful jobs before making the job required for matching paths.
- Run wiki validation after each phase reconciliation. The final phase runs `make lint`, `make test`, and `make check-wasm` only after implementation and CI integration are stable.

# Acceptance criteria

- A locked `webmcp-evals` installation and committed suite exercise all eight current ACORN WebMCP tools through a live staged page.
- `make test-webmcp-evals-smoke` is API-key-free, starts and cleans up its own loopback server, writes inspectable reports under `target/`, and returns nonzero for missing tools, mismatched calls/arguments, unsupported Chrome, or execution errors.
- The existing adapter, package, WASM, and native-browser smoke checks continue to pass and retain ownership of strict negative/result behavior.
- An explicit model-backed command records five runs per case for a named provider/model and passes direct and ambiguity cases without storing or logging credentials.
- A path-filtered deterministic CI job runs on a proven Chrome/WebMCP runner and retains reports on failure; ordinary `make test` and model-free development remain independent of provider credentials.
- Developer documentation explains prerequisites, exact commands, report locations, failure triage, evaluator upgrades, and the distinction between deterministic interoperability and model selection quality.

# Execution phases

## WME-01 — Define the eight-tool trajectory suite

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-wasm/examples/webmcp/evals.json` (new), `.wiki/testing/webmcp-evaluations.md`
- Primary integration lock: WebMCP evaluation data
- Outcome: Add valid concrete trajectories for every current tool, including diff modes and natural-language prompts reusable by smoke and browser modes, without changing the adapter or demo.
- Focused validation: Parse the suite as JSON; compare every function name and argument shape with the registered schemas; run `make test-webmcp` and `make build-wasm`; execute an exploratory pinned `webmcp-evals@0.1.0 smoke` run when compatible Chrome is available.
- Handoff: Remove WME-01, retain the accepted suite, record evaluator/Chrome evidence, and promote WME-02 when every tool executes.

## WME-02 — Pin the evaluator and add deterministic commands

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: WME-01; supported local Chrome channel with WebMCP enabled.
- Exact write set: `tools/webmcp-evals/package.json` (new), `tools/webmcp-evals/package-lock.json` (new), `scripts/test-webmcp-evals.sh` (new), `Makefile`, `.gitignore`, `crates/acorn-wasm/examples/webmcp/README.md`, `.wiki/testing/webmcp-evaluations.md`
- Primary integration lock: CI/build
- Outcome: Pin the evaluator and add `make test-webmcp-evals-smoke`, with isolated loopback serving, readiness checks, cleanup, clear prerequisite failures, and reports under `target/webmcp-evals/smoke/`.
- Focused validation: Run `npm ci --prefix tools/webmcp-evals`, `make build-wasm`, and the new smoke target twice; force one expected-call mismatch and one server-start failure in disposable state; verify nonzero exits, report retention, and cleanup; rerun `make test-webmcp` and `make test-webmcp-package`.
- Handoff: Remove WME-02 and preserve the pinned install, stable command, observed Chrome version, and failure evidence.

## WME-03 — Add model-backed selection evaluation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: WME-02; one approved provider/model and its credential available in the operator environment.
- Exact write set: `crates/acorn-wasm/examples/webmcp/agent-evals.json` (new), `scripts/test-webmcp-evals.sh`, `Makefile`, `crates/acorn-wasm/examples/webmcp/README.md`, `.wiki/testing/webmcp-evaluations.md`
- Primary integration lock: WebMCP agent evaluation
- Outcome: Add direct and ambiguity prompts plus `make test-webmcp-evals-browser`; require an explicit model and run count, preserve JSON/HTML reports, and keep provider calls outside normal tests.
- Focused validation: Run `WEBMCP_EVALS_MODEL=<provider:model> WEBMCP_EVALS_RUNS=5 make test-webmcp-evals-browser`; inspect exact trajectories and report metadata; verify missing model/credential failures are clear and redact secrets; rerun deterministic smoke to separate agent errors from page errors.
- Handoff: Remove WME-03 and record the dated model/browser/evaluator baseline and any repeatable selection defect without changing adapter descriptions opportunistically.

## WME-04 — Add deterministic CI coverage

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: WME-02; named CI runner with supported pinned Chrome/WebMCP configuration; two consecutive successful local or isolated-runner smoke executions.
- Exact write set: `.gitlab-ci.yml`, `Makefile`, `crates/acorn-wasm/examples/webmcp/README.md`, `.wiki/testing/webmcp-evaluations.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: CI/build
- Outcome: Add a path-filtered deterministic smoke job with locked npm installation, explicit browser/version diagnostics, bounded timeout, and retained JSON/HTML reports; do not add model credentials or browser evaluation to CI.
- Focused validation: Finish two consecutive candidate jobs; confirm a disposable expected-call mismatch fails the job and retains reports; verify unrelated paths skip the job and ordinary `make test` remains independent.
- Handoff: Remove WME-04 and retain job links, runner/browser fingerprints, and failure-artifact evidence outside the active plan.

## WME-05 — Publish the evaluation workflow and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: WME-03 and WME-04; deterministic CI and one five-run model baseline accepted.
- Exact write set: `docs/src/learn/developer.md`, `crates/acorn-wasm/examples/webmcp/README.md`, `.wiki/testing/webmcp-evaluations.md` (delete after acceptance), `.wiki/testing/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: developer test documentation
- Outcome: Document the two evaluation signals, exact commands, prerequisites, report review, upgrade procedure, and failure ownership; run final gates and remove the completed plan from every wiki surface.
- Focused validation: Run `make test-webmcp`, `make test-webmcp-package`, `make build-wasm`, `make test-webmcp-evals-smoke`, `make lint`, `make test`, `make check-wasm`, the book build, and wiki validation; inspect the accepted CI and model reports before deleting the plan.
- Handoff: Preserve commands, evaluator version policy, CI evidence, and model-baseline procedure in durable developer documentation; delete this plan and every index, network, and lane entry.
