---
okf_version: "0.1"
type: Plan
title: Evaluate ACORN WebMCP and MCP interoperability
description: Add reproducible browser WebMCP evaluations and remote MCP client interoperability checks without replacing ACORN's existing protocol contract tests.
difficulty: medium
impact: medium
package: [acorn-wasm, acorn-lib, acorn-cli]
tags: [testing, webmcp, mcp, wasm, browser, agent-evaluation, interoperability]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Use GoogleChromeLabs `webmcp-evals`, ACORN's outbound MCP client, and an independent public MCP control to verify four distinct outcomes:

1. A deterministic, API-key-free smoke suite discovers and executes every registered ACORN WebMCP tool in a real supported Chrome page.
2. An explicitly invoked model-backed browser suite measures whether an agent selects the correct ACORN tool and exact arguments from natural-language requests, especially where two tools have similar domains.
3. ACORN's Streamable HTTP MCP client discovers and invokes an explicitly allowlisted read-only tool on a live, unauthenticated remote server.
4. The same public server works through the official OpenAI Plugins quickstart in ChatGPT Work, providing a control that separates endpoint availability and tool metadata from ACORN client behavior.

Keep ACORN's existing Node adapter tests, native browser smoke, synthetic Streamable HTTP client tests, and stdio interoperability tests as the authoritative checks for protocol behavior, strict schemas, policy, input and output bounds, cancellation, timing, Permissions Policy, rendering safety, and cleanup. The external checks add independent live-page, live-server, and agent-selection evidence.

# Current evidence

Evidence rechecked on 2026-11-09 against `10b17f22` and the current worktree:

- `crates/acorn-wasm/web/webmcp.js` names nine tools: FAIR assessment, readability, Big-O comparison, metadata crosswalk, content diff, persistent-identifier discovery, one-identifier inspection, metadata-standard validation, and Research Activity validation. Its sorted registry exposes those tools through the imperative `document.modelContext.registerTool()` API that `webmcp-evals` supports.
- `crates/acorn-wasm/web/webmcp.test.mjs` uses a fake model context to cover adapter validation and registration without launching a browser. `make test-webmcp` runs that contract suite.
- `crates/acorn-wasm/examples/webmcp/smoke.js` and the `?smoke=1` demo route already exercise native registration, discovery, execution, serialization, strict schemas, annotations, bounds, timing, Permissions Policy, manual parity, and disposal in a supported browser. The reference run documented in `crates/acorn-wasm/examples/webmcp/README.md` passed with Chrome 154 on macOS on 2026-10-09.
- `make build-wasm` builds the browser WASM package, stages the demo under `target/acorn-wasm/webmcp/`, runs `make test-webmcp`, verifies staged package contents, and runs the WASM Node tests. The documented local origin is `http://127.0.0.1:8000/webmcp/` after `python3 -m http.server 8000 --directory target/acorn-wasm`.
- The upstream [`webmcp-evals` README](https://github.com/GoogleChromeLabs/webmcp-tools/blob/main/webmcp-evals/README.md) documents `smoke` as a deterministic live-page runner with no model or API key, `browser` as a live tool-calling evaluation, fresh-page isolation for each smoke case, constraint-based argument matching, JSON/HTML reports, and a nonzero exit when a browser case fails. It explicitly labels the framework experimental.
- The published [`webmcp-evals` package](https://github.com/GoogleChromeLabs/webmcp-tools/blob/main/webmcp-evals/package.json) is version `0.1.0`; `npm view webmcp-evals version dist-tags --json` returned `0.1.0` as `latest` on 2026-11-07. No ACORN package manifest, lockfile, eval suite, Make target, script, or CI job currently references it.
- The external evaluator matches requested calls and arguments. It does not replace ACORN assertions over returned envelopes or negative adapter behavior. Oversized, malformed, concurrent, cancelled, and hostile-rendering cases stay in the existing adapter/native smoke suites unless a later upstream evaluator version exposes a deterministic output assertion needed by ACORN.
- `crates/acorn-lib/src/io/mcp/client.rs` implements allowlisted Streamable HTTP tool discovery and invocation with initialization, capability checks, catalog bounds, per-tool timeouts, output limits, cancellation, cleanup, offline rejection, and optional bearer authentication. `crates/acorn-lib/src/io/mcp/tests/client.rs` exercises those behaviors against a local synthetic HTTP server, including list/call, authentication, pagination, malformed responses, cancellation, and timeouts.
- `crates/acorn-cli/tests/mcp_client.rs` proves compiled-binary interoperability between ACORN's outbound client and ACORN's stdio MCP server. `acorn mcp --config <path> list <server>` and `acorn mcp --config <path> call <server> <tool> --arguments <json>` expose the same policy-checked client for a live remote smoke test. No committed test currently calls a public remote MCP server.
- The official [OpenAI Plugins quickstart](https://developers.openai.com/plugins/quickstart) was checked on 2026-11-07. It states: “This quickstart uses a public example MCP server at `https://tinymcp.dev/api/moldy-aloof-zettabyte/mcp`. It exposes a read-only `roll_dice` tool and does not require authentication.” The documented control connects that URL as a no-auth personal plugin, calls `roll_dice` once with `sides` set to `20`, and confirms one returned value from 1 through 20. This is official documentation for a live external example, not evidence that ACORN has successfully connected to it.
- The [MCP Inspector](https://github.com/modelcontextprotocol/inspector) provides a scriptable CLI for initialization, tool listing, tool calls, strict schema checks, machine-readable output, and CI exit codes. Its [server-testing documentation](https://github.com/modelcontextprotocol/inspector/blob/main/docs/test-servers.md) distinguishes real in-process and subprocess transports from mocks. `npm view @modelcontextprotocol/inspector version dist-tags --json` returned `2.14.0` as `latest` on 2026-11-07.
- The [MCP conformance suite](https://github.com/modelcontextprotocol/conformance) runs scenario servers around a client command or sends scenarios to a running HTTP server, validates captured wire messages against the negotiated specification schema, and writes detailed results. `npm view @modelcontextprotocol/conformance version dist-tags --json` returned `0.1.16` as `latest` on 2026-11-07. ACORN needs a small scenario-aware client adapter before its outbound client can use that harness. The current server-mode CLI accepts an HTTP URL; ACORN's released MCP server surface is stdio, so server conformance needs a feasibility decision rather than an assumed pass.
- The [MCP Python SDK testing guide](https://py.sdk.modelcontextprotocol.io/get-started/testing/) documents using the same SDK client against an in-memory server to test tool behavior without a subprocess or port. ACORN uses Rust and already applies the equivalent pattern through `rmcp`, synthetic Streamable HTTP fixtures, and compiled stdio integration tests; this plan does not add Python to the workspace.

# Layered test strategy

The strongest approach is a layered test suite: protocol checks for compatibility, deterministic tests for your server’s behavior, and a small set of end-to-end tests for real client workflows.

| Layer | Good option | What it checks |
|---|---|---|
| Protocol smoke tests | [MCP Inspector](https://github.com/modelcontextprotocol/inspector) CLI | Can a client connect, list tools, and invoke them? Useful locally and in CI. |
| Spec conformance | [MCP conformance suite](https://github.com/modelcontextprotocol/conformance) | Does the server meet the protocol requirements covered by the suite? Best when compatibility across clients matters. |
| Unit and integration tests | Your SDK’s client with an in-memory server or transport | Tool logic, input validation, errors, resources, prompts, and edge cases. The [Python SDK](https://py.sdk.modelcontextprotocol.io/get-started/testing/) documents this pattern. |
| Contract regression tests | Checked-in cases for tool inputs and expected outputs | Do tools still behave as your product expects? Add cases for invalid inputs and failure paths, not just happy paths. |
| Agent workflow evaluations | A small fixed set of tasks run with a real model and client | Can an agent discover the right tool, choose it, and use its result correctly? These tests are useful, but less deterministic and more expensive. |

**Practical CI setup:** run fast in-memory tests on every change, then start the server and run Inspector smoke checks. Add conformance checks if protocol compatibility is a release requirement. Run model-driven evaluations less often, such as before releases, and review failures because model behavior can vary.

Inspector is a hands-on debugging tool as well as an automation option; the conformance suite is more focused on protocol requirements. Neither replaces tests of your server’s actual business behavior. [Inspector documentation](https://github.com/modelcontextprotocol/inspector/blob/main/docs/test-servers.md) · [Conformance suite](https://github.com/modelcontextprotocol/conformance)

If you share your SDK and transport (stdio or HTTP), I can suggest a concrete CI setup. ACORN's known answer is Rust with `rmcp`, a stdio server, and a Streamable HTTP outbound client, so the phases below make that setup concrete.

For ACORN, this maps to fast Rust unit/integration and checked-in contract tests on every change, pinned Inspector smoke checks against the built stdio server, applicable client conformance scenarios on release paths, opt-in live public-server checks, and infrequent model-backed WebMCP/Plugins evaluations.

# Boundaries and decisions

## In scope

- One deterministic `evals.json` suite with valid concrete calls for all nine current tools and natural-language messages reusable by `browser` mode.
- Selection cases that distinguish identifier discovery from one-identifier inspection, Research Activity validation from metadata-standard validation, FAIR assessment from validation, unified diff from JSON Patch, and crosswalk source/target requirements.
- A repository-local npm tool directory with an exact `webmcp-evals` version and committed lockfile. Do not rely on a floating `npx webmcp-evals` install for repeatable repository gates.
- One script that starts an isolated loopback server, waits for readiness, invokes the evaluator, preserves reports, stops only the server process it created, and fails clearly when staged artifacts, Chrome support, or tool discovery are unavailable.
- Named Make targets for deterministic smoke and opt-in model-backed browser evaluation.
- A path-filtered deterministic CI job only after a named runner proves the pinned Chrome/WebMCP configuration. Model-backed runs remain explicit because provider cost, credentials, and nondeterminism are outside ordinary CI.
- Durable developer documentation for prerequisites, commands, expected reports, version updates, and failure diagnosis.
- A committed no-auth Streamable HTTP configuration fixture that allowlists only `roll_dice` with `network-read`, a bounded timeout, and a bounded output size.
- An ignored, explicitly invoked live MCP test and Make target that list the public server's allowlisted catalog, call `roll_dice` with `sides: 20`, and accept exactly one returned integer from 1 through 20.
- A manual ChatGPT Work control following the official OpenAI Plugins quickstart, performed only in an authorized personal Plugins directory and recorded separately from ACORN results.
- A pinned MCP Inspector CLI smoke that initializes the built ACORN stdio server, strictly lists its tools, calls one representative read-only tool, and emits machine-readable diagnostics.
- A bounded conformance feasibility and implementation slice for ACORN's outbound MCP client, with the target specification revision and applicable scenarios recorded. Server conformance is included only if the harness can test ACORN's supported transport without adding production HTTP behavior solely for the test.
- Checked-in transport-level contract cases for representative success, invalid-input, server-error, allowlist-denial, and output-bound behavior.

## Out of scope

- Exporting a separate static schema solely to use `webmcp-evals local`; live `smoke` and `browser` modes consume the registered schemas directly.
- Replacing `webmcp.test.mjs`, the native `?smoke=1` suite, or `make build-wasm`.
- Changing ACORN tool names, descriptions, schemas, result envelopes, WASM behavior, or demo UI merely to satisfy one model. A repeatable selection failure may motivate a separate evidence-backed adapter change.
- Adding provider secrets to the repository, printing them in logs, or making paid model calls from normal test targets.
- Adding authentication, mutation, UI resources, plugin publication, or a permanent dependency on the public dice server. The reference server is a read-only, no-auth interoperability control.
- Making the public remote MCP smoke part of `make test` or a required CI gate. External availability, DNS, TLS, and service changes must not break offline or ordinary repository validation.
- Treating one model run as a product compatibility guarantee. Reports identify the evaluator version, Chrome version/channel, provider/model, run count, date, and suite revision.
- Adding `simulate` coverage. ACORN's current demo exposes read-only operations whose success is already represented by tool trajectories and tool-tab output; add simulation only under a separate plan if a stateful user workflow appears.
- Claiming full MCP conformance from Inspector smoke tests, local SDK tests, or a partial scenario run. Record the exact conformance package, requirements revision, role, scenarios, expected-failure baseline, and result directory.
- Adding an HTTP MCP server surface only to satisfy the conformance harness. If stdio server conformance remains unavailable, preserve the limitation and open a separate product-backed transport plan only when ACORN needs an HTTP server.

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
npm ci --prefix tools/mcp-testing
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

Run existing deterministic MCP coverage:

```sh
cargo test -p acorn-lib io::mcp::tests::client
cargo test -p acorn-cli --test mcp_client
```

After the public MCP fixture and wrapper land, inspect and invoke the reference server directly through ACORN:

```sh
cargo run --bin acorn -- mcp \
  --config tests/fixtures/mcp/tinymcp-public.json \
  list tinymcp
cargo run --bin acorn -- mcp \
  --config tests/fixtures/mcp/tinymcp-public.json \
  call tinymcp roll_dice \
  --arguments '{"sides":20}'
make test-mcp-remote-smoke
```

The live target must fail clearly when the server is unavailable, `roll_dice` is absent, the schema is incompatible, the call reports an MCP error, or the result cannot be reduced to one integer from 1 through 20. It is never invoked by `make test`.

After WME-08 lands, run the pinned Inspector against ACORN's stdio server:

```sh
cargo build -p acorn-cli --bin acorn
tools/mcp-testing/node_modules/.bin/mcp-inspector \
  --cli target/debug/acorn --no-local-database serve mcp -- \
  --method tools/list --strict --format json
tools/mcp-testing/node_modules/.bin/mcp-inspector \
  --cli target/debug/acorn --no-local-database serve mcp -- \
  --method tools/call --tool-name acorn.version \
  --tool-args-json '{}' --format json
make test-mcp-inspector
```

After WME-09 lands, run the applicable pinned client conformance suite through the scenario-aware ACORN adapter:

```sh
tools/mcp-testing/node_modules/.bin/mcp-conformance \
  client \
  --command "cargo run -p acorn-cli --example mcp_conformance_client --" \
  --suite core \
  --verbose
make test-mcp-conformance
```

The implementation phase must replace `--suite core` with a frozen `--requirements <revision>` invocation when the selected conformance release publishes an applicable requirements set for ACORN's client role. Results stay under `target/mcp-conformance/`; any expected-failure file names individual unsupported checks and may not baseline a whole scenario merely to hide one failure.

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

- WME-01 can begin on the current nine-tool surface. It owns only new evaluation data and does not wait for other plans.
- WME-02 follows WME-01 and claims the CI/build lock for the Makefile change. It must serialize with coverage throughput, report visual regression, runner pools, Mise, Podman, and other phases editing `Makefile`.
- WME-03 follows the deterministic harness so agent failures can be separated from registration, launch, and execution failures. It does not require CI credentials.
- WME-04 requires a named CI runner with a supported Chrome channel and WebMCP enabled, plus accepted WME-02 smoke evidence. Missing runner/browser support blocks CI wiring without blocking local evaluation.
- Big-O comparison is now the ninth WebMCP tool. WME-01 must include its canonical fixture-derived request in the accepted evaluation matrix; this plan does not edit `webmcp.js`.
- Upstream `webmcp-evals` is experimental. Version changes are deliberate lockfile updates followed by the deterministic suite, report-schema inspection, and one recorded model baseline. Do not accept a floating update during an unrelated ACORN change.
- WME-06 can proceed independently of the WebMCP phases. It consumes the current outbound MCP client and CLI without changing their production behavior. Coordinate its new compiled-binary test file with CLI coverage work, but do not take the Ask capability-routing lock.
- WME-07 follows WME-06 and requires explicit authorization to create or use a personal plugin in ChatGPT Work. The OpenAI Plugins quickstart is an independent control, not a release dependency for ACORN's offline tests or WebMCP suite.
- The public MCP endpoint is external mutable state. Record its observed catalog and result shape at execution time; a service change or outage blocks only the live acceptance phase and does not justify patching ACORN policy or weakening local tests.
- WME-08 can proceed against the current built stdio server and owns a separate pinned MCP test-tool package. It serializes its Makefile change with WME-02, WME-06, coverage throughput, report visual regression, Mise, Podman, and other CI/build owners.
- WME-09 follows WME-08 and owns only the scenario adapter, conformance wrapper, and narrowly baselined results. It must select a dated MCP requirements revision supported by ACORN rather than following a moving draft. A failed server-transport feasibility check records the limitation and does not authorize a production HTTP surface.
- WME-10 is independent test-data work until its final registration in the compiled-binary test target. It serializes edits to existing MCP test files with other MCP registry/server-test owners, including BGO-04 and sidecar registrations.

# Validation

- Parse each committed suite as JSON and verify every expected function name equals a tool registered by the staged page.
- Run `make test-webmcp`, `make test-webmcp-package`, and `make build-wasm` before the external evaluator so an adapter/package defect is reported at its owning layer.
- Run the deterministic evaluator twice from clean temporary Chrome profiles. Confirm all nine tools are discovered, every case executes, reports are written under `target/`, and the wrapper stops its loopback server after success and failure.
- Deliberately change one expected tool name and one required argument in a temporary worktree or disposable copy; verify the smoke command exits nonzero, then restore the accepted suite.
- Run the browser suite for five runs with the recorded provider/model. Inspect the JSON report for exact trajectories and ensure similar-domain prompts select the intended tool.
- Run the existing synthetic Streamable HTTP and compiled-binary stdio client tests before the public MCP smoke. Confirm the public fixture contains only the HTTPS URL, no bearer-token environment name, and one allowlisted `roll_dice` policy with `network-read` and explicit bounds.
- Invoke the public MCP target twice. Confirm the advertised allowlisted catalog contains `roll_dice`, each call returns exactly one integer in the requested range, and invalid arguments surface as a tool error or a documented server rejection without mutation.
- In an authorized ChatGPT Work test account, follow the [official OpenAI Plugins quickstart](https://developers.openai.com/plugins/quickstart): create the no-auth personal plugin, install it, invoke it explicitly in Work, ask for one 20-sided die, and record that `roll_dice` was called once with `sides: 20` and returned one value from 1 through 20. Test another die size, an invalid value, and one request that should not call the tool. Do not treat this control as an ACORN execution result.
- Run pinned Inspector initialization, strict `tools/list`, and `acorn.version` invocation against the built stdio server. Confirm stdout remains machine-readable, schemas pass the Inspector's strict gate, failures produce nonzero exits, and no network or database is required.
- Run the applicable client conformance requirements at their dated wire version. Inspect `checks.json`, stdout, and stderr for every scenario; accept only narrowly explained unsupported checks and keep server-role results separate from client-role results.
- Drive the checked-in MCP contract cases through the same compiled server/client boundaries used in production. Assert exact stable fields for success plus invalid arguments, tool errors, policy denial, timeout/output limits, and cleanup; avoid snapshots of volatile timestamps, ordering not promised by the contract, or platform paths.
- On the candidate CI runner, record Chrome version/channel, WebMCP enablement, Node/npm versions, evaluator package/lock digest, job link, and two consecutive successful jobs before making the job required for matching paths.
- Run wiki validation after each phase reconciliation. The final phase runs `make lint`, `make test`, and `make check-wasm` only after implementation and CI integration are stable.

# Acceptance criteria

- A locked `webmcp-evals` installation and committed suite exercise all nine current ACORN WebMCP tools through a live staged page.
- `make test-webmcp-evals-smoke` is API-key-free, starts and cleans up its own loopback server, writes inspectable reports under `target/`, and returns nonzero for missing tools, mismatched calls/arguments, unsupported Chrome, or execution errors.
- The existing adapter, package, WASM, and native-browser smoke checks continue to pass and retain ownership of strict negative/result behavior.
- An explicit model-backed command records five runs per case for a named provider/model and passes direct and ambiguity cases without storing or logging credentials.
- The existing local MCP protocol tests pass, and an opt-in `make test-mcp-remote-smoke` proves ACORN can list and call the public no-auth `roll_dice` server through its configured Streamable HTTP client while enforcing its allowlist and bounds.
- The official OpenAI Plugins quickstart is linked and its public-server statement is retained verbatim. A separately labeled ChatGPT Work control records the expected single `roll_dice({"sides":20})` call and a value from 1 through 20 without presenting that result as ACORN evidence.
- Ordinary `make test`, offline runs, and required CI do not contact `tinymcp.dev`; external MCP failures remain isolated to the explicit live target.
- A pinned `make test-mcp-inspector` command initializes ACORN's built stdio server, strictly lists tools, calls `acorn.version`, returns machine-readable output, and fails on connection, schema, listing, or invocation errors.
- Applicable ACORN client conformance requirements pass for one recorded dated MCP revision, or narrowly scoped expected failures identify unsupported requirements without claiming full conformance. The plan records that server conformance is unresolved when the harness cannot exercise ACORN's stdio-only server surface.
- Checked-in MCP contract cases cover representative success, invalid input, tool-reported error, allowlist denial, timeout/output bounds, and cleanup through production client/server seams.
- CI follows the layered cadence: fast local Rust and contract tests on changes, Inspector smoke after the server build, applicable conformance on release paths, and model-driven evaluations outside ordinary change validation.
- A path-filtered deterministic CI job runs on a proven Chrome/WebMCP runner and retains reports on failure; ordinary `make test` and model-free development remain independent of provider credentials.
- Developer documentation explains prerequisites, exact commands, report locations, failure triage, evaluator upgrades, and the distinction between deterministic interoperability and model selection quality.

# Execution phases

## WME-01 — Define the nine-tool trajectory suite

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
- Prerequisites: WME-02, WME-08, WME-09, and WME-10; named CI runner with supported pinned Chrome/WebMCP configuration; two consecutive successful local or isolated-runner smoke executions.
- Exact write set: `.gitlab-ci.yml`, `Makefile`, `crates/acorn-wasm/examples/webmcp/README.md`, `.wiki/testing/webmcp-evaluations.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: CI/build
- Outcome: Add fast Rust/contract tests and pinned Inspector smoke to relevant change validation, path-filtered deterministic WebMCP smoke on its compatible runner, and applicable conformance requirements on release paths. Retain machine-readable diagnostics; do not add the public server, model credentials, or model-backed evaluation to required CI.
- Focused validation: Finish two consecutive candidate jobs; confirm disposable Inspector, contract, conformance, and WebMCP mismatches fail their owning jobs with retained reports; verify unrelated paths skip expensive jobs and ordinary `make test` remains offline.
- Handoff: Remove WME-04 and retain job links, runner/browser fingerprints, and failure-artifact evidence outside the active plan.

## WME-08 — Pin MCP Inspector protocol smoke checks

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: Node version supported by the pinned Inspector; a built ACORN binary that can run `serve mcp` over stdio.
- Exact write set: `tools/mcp-testing/package.json` (new), `tools/mcp-testing/package-lock.json` (new), `scripts/test-mcp-inspector.sh` (new), `Makefile`, `.gitignore`, `.wiki/testing/webmcp-evaluations.md`
- Primary integration lock: CI/build
- Outcome: Pin MCP Inspector and add `make test-mcp-inspector` for initialization, strict tool listing, and representative `acorn.version` invocation against the real stdio server with machine-readable output and no network dependency.
- Focused validation: Run `npm ci --prefix tools/mcp-testing`, build ACORN, execute the direct Inspector commands and Make target, corrupt one expected tool/schema in disposable state, and verify clear nonzero exits while ordinary Rust MCP tests still pass.
- Handoff: Remove WME-08 and preserve the pinned version, lock digest, commands, inspected protocol version/capabilities, and failure evidence.

## WME-09 — Establish applicable MCP conformance coverage

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: WME-08; one selected dated MCP requirements revision supported by ACORN's `rmcp` version; pinned conformance package installed in `tools/mcp-testing`.
- Exact write set: `crates/acorn-cli/examples/mcp_conformance_client.rs` (new), `tools/mcp-testing/package.json`, `tools/mcp-testing/package-lock.json`, `scripts/test-mcp-conformance.sh` (new), `Makefile`, `tests/fixtures/mcp/conformance-expected-failures.yml` (new only if required), `.wiki/testing/webmcp-evaluations.md`
- Primary integration lock: outbound MCP client conformance
- Outcome: Add a scenario-aware adapter and `make test-mcp-conformance` that run the applicable client requirements at a frozen dated wire version, retain detailed results under `target/`, and use only narrowly justified expected failures. Record whether server-role conformance can exercise ACORN's stdio server without adding product behavior.
- Focused validation: List the pinned requirements, run the client adapter through every applicable scenario, inspect each `checks.json`, force one wire-schema failure to prove the gate, and verify any expected-failure entry names one check or genuinely unsupported scenario. Attempt server-role feasibility once and record `verified`, `incorrect`, or `unresolved` without introducing HTTP solely for testing.
- Handoff: Remove WME-09 and preserve the requirements revision, role/scenario matrix, package version, result summary, expected-failure rationale, and server-transport decision.

## WME-10 — Add checked-in MCP contract regressions

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `tests/fixtures/mcp/contracts.json` (new), `crates/acorn-cli/tests/mcp.rs`, `crates/acorn-cli/tests/mcp_client.rs`, `.wiki/testing/webmcp-evaluations.md`
- Primary integration lock: compiled MCP contract tests
- Outcome: Drive representative checked-in inputs through ACORN's real stdio server and outbound client, asserting stable success outputs plus invalid input, tool error, allowlist denial, output bound, cancellation/cleanup, and protocol-only stdout behavior.
- Focused validation: Run `cargo test -p acorn-cli --test mcp` and `cargo test -p acorn-cli --test mcp_client`; alter one expected output and one invalid-input expectation in disposable state to prove both gates; confirm fixtures contain no secrets, network URLs, volatile paths, or timestamps.
- Handoff: Remove WME-10 and preserve the minimal contract corpus and its ownership boundaries.

## WME-06 — Add the public remote MCP client smoke

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: the public endpoint remains available with a read-only, no-auth `roll_dice` tool; network access is explicitly enabled for the live run.
- Exact write set: `tests/fixtures/mcp/tinymcp-public.json` (new), `crates/acorn-cli/tests/mcp_remote.rs` (new), `Makefile`, `.wiki/testing/webmcp-evaluations.md`
- Primary integration lock: outbound MCP client integration tests
- Outcome: Add an ignored compiled-binary live test and `make test-mcp-remote-smoke` that use ACORN's configured Streamable HTTP client to list the allowlisted public tool, call `roll_dice` with 20 sides, and validate one integer from 1 through 20 without credentials or mutation authority.
- Focused validation: Run `cargo test -p acorn-lib io::mcp::tests::client`, `cargo test -p acorn-cli --test mcp_client`, and `make test-mcp-remote-smoke`; inspect the emitted catalog and result; verify offline mode rejects the remote endpoint before connection and ordinary `make test` does not run the ignored live test.
- Handoff: Remove WME-06, preserve the fixture, explicit command, observed server contract/date, and live result evidence, and leave outages or upstream contract changes isolated to the opt-in target.

## WME-07 — Run the official Plugins MCP control

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: WME-06; explicit authorization to use an available ChatGPT Work personal Plugins directory; the official quickstart and public server remain available.
- Exact write set: `docs/src/learn/developer.md`, `.wiki/testing/webmcp-evaluations.md`; named external system: authorized ChatGPT Work personal Plugins directory.
- Primary integration lock: MCP reference-control acceptance
- Outcome: Follow the official OpenAI Plugins quickstart with the same public endpoint, record the exact `roll_dice({"sides":20})` control and returned range, exercise another size, invalid input, and a request that should not call the tool, and document how this control distinguishes endpoint/plugin behavior from ACORN client evidence.
- Focused validation: Compare the observed ChatGPT Work tool call and result with the official quickstart; rerun `make test-mcp-remote-smoke` against the same observed server contract; label each result by client, date, endpoint, tool, arguments, and outcome without recording account details or credentials.
- Handoff: Remove WME-07 and retain the official link, control procedure, sanitized evidence, and separation of ACORN versus ChatGPT Work results in durable developer documentation.

## WME-05 — Publish the WebMCP and MCP evaluation workflow and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: WME-03, WME-04, WME-06, WME-07, WME-08, WME-09, and WME-10; layered deterministic CI, one five-run model baseline, public MCP smoke, applicable conformance evidence, and official Plugins control accepted.
- Exact write set: `docs/src/learn/developer.md`, `crates/acorn-wasm/examples/webmcp/README.md`, `.wiki/testing/webmcp-evaluations.md` (delete after acceptance), `.wiki/testing/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: developer test documentation
- Outcome: Document deterministic WebMCP, model-backed WebMCP, local MCP protocol, public MCP client, and official Plugins control signals; publish exact commands, prerequisites, report review, upgrade procedure, and failure ownership; run final gates and remove the completed plan from every wiki surface.
- Focused validation: Run `make test-webmcp`, `make test-webmcp-package`, `make build-wasm`, `make test-webmcp-evals-smoke`, `cargo test -p acorn-lib io::mcp::tests::client`, `cargo test -p acorn-cli --test mcp`, `cargo test -p acorn-cli --test mcp_client`, `make test-mcp-inspector`, `make test-mcp-conformance`, `make test-mcp-remote-smoke`, `make lint`, `make test`, `make check-wasm`, the book build, and wiki validation; inspect the accepted WebMCP reports and separately labeled MCP evidence before deleting the plan.
- Handoff: Preserve commands, pinned Inspector/conformance/evaluator policies, CI evidence, requirements matrix, model-baseline procedure, public MCP fixture policy, official Plugins quickstart link, and evidence-labeling rules in durable developer documentation; delete this plan and every index, network, and lane entry.
