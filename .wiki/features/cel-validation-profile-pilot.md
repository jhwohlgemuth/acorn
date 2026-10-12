---
type: Plan
title: "Pilot optional CEL metadata validation profiles"
description: "Verify a bounded Rust CEL implementation and add explicitly selected local metadata rules through the existing analyzer pipeline."
difficulty: medium
impact: high
package: [acorn-lib, acorn-cli]
tags: [feature, policy, validation, cel]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal and recommendation

Allow a team to express three concrete institutional metadata requirements without altering the RAD schema or writing a Rhai workflow. Ship an optional local CEL profile only if the runtime and authoring gates pass. See the [OPA/CEL assessment](../assets/opa-cel-review.md) for technology comparisons, sources, alternatives, and estimates. OPA authorization, remote distribution, browser bindings, and cross-record policy are outside this atomic plan.

# Current evidence

Verified 2026-10-01: ACORN has ordered `ValidationReport` and `CheckBatch` results, analyzer execution states, restricted Rhai hooks, and shared operation effect enforcement. No OPA/CEL plan was found in the wiki. The inspected `cel-rust` `Program::compile` path parses source; full type checking and execution budgets must be verified against a pinned release before adoption. Existing functionality is the comparison baseline, not migration scope.

# Dependencies and scheduling

- Preserve the landed dependency-capability boundary when wiring the optional evaluator. Reuse current in-tree analyzer boundaries; no crate extraction prerequisite.
- Use the policy-validation lane in the [parallel execution guide](../parallel-execution.md). Claim exact files before implementation; the broad planning boundaries below are not concurrent ownership grants.
- Read-only corpus and runtime evaluation may precede integration availability. Evaluator integration precedes CLI wiring, which precedes pilot acceptance. Any OPA work needs its own future plan and deployment consumer.

# Project code boundaries

- **Update:** `Cargo.toml`, `Cargo.lock`, `crates/acorn-lib/{Cargo.toml,src/lib.rs,src/analyzer/mod.rs,src/analyzer/check.rs}`, focused category/summary definitions if required, `crates/acorn-lib/src/analyzer/host/{service/mod.rs,runtime/mod.rs,render.rs}`, `crates/acorn-cli/{Cargo.toml,src/cli/arguments/,src/commands/check/,tests/check.rs}`, `docs/src/cli-reference/commands/check.md`, and generated `CommandLineHelp.md`. Resolve category/summary and CLI files to exact paths before taking locks.
- **Create:** `crates/acorn-lib/src/analyzer/policy/{mod.rs,cel.rs,tests.rs}` and `tests/fixtures/policy/` for the narrow input contract, profile validation, evaluator, and corpus. Runtime dependency remains optional; first release is native unless portable execution is demonstrated.
- **Delete:** None.
- **Locks:** Workspace graph, module indexes/library roots, analyzer result contract, CLI contract, documentation generation, and any shared test snapshots. No changes to schema validators, Rhai execution, operation authorization, MCP, database, or network configuration.

# Implementation slices

1. **Runtime and value gate (3–5 engineer-days).** Gather three owner-backed rules and positive/negative fixtures. Compare authoring with existing Rust/Rhai. Pin the CEL crate and reference implementation; verify license, MSRV, checker capability, macros, numeric/null semantics, error locations, thread isolation, dependency graph, and enforceable CPU/memory bounds. Record any unsupported CEL subset explicitly. Stop if enforcing bounds requires inventing a language implementation or if the user benefit is weak.
2. **Evaluator and integration (5–10 engineer-days).** Specify versioned allowlisted inputs and a manifest with stable IDs, static messages/paths, severity, and boolean expressions. Validate using error-collecting deserialization at the user-facing boundary. Compile once, evaluate with fresh bindings, and attach bounded provenance. Add explicit local profile selection to check; advisory reporting is distinct from explicit enforcement. Integrate findings and skipped/error states with existing analyzer summaries. Generate help after CLI changes.
3. **Acceptance pilot (3–5 engineer-days).** Document supported syntax and contracts, publish representative profiles, run regression/conformance/resource tests, and measure cold/warm performance and binary size. Have the profile owner edit a requirement and its tests without Rust changes. Keep the capability opt-in until evidence justifies broader availability.

# Concrete implementation examples

Use the [overview's three-rule manifest and projected input](../assets/opa-cel-review.md#concrete-cel-profile-example) as the initial design fixture. These examples specify proposed behavior; neither the manifest nor the CLI flags below exist yet. Confirm that the three requirements match the pilot owner's needs before implementing them.

## Rust evaluator boundary

This small integration sketch uses the inspected upstream `cel` APIs: [variable binding example](https://raw.githubusercontent.com/cel-rust/cel-rust/master/example/src/variables.rs) and [Program/Value API](https://raw.githubusercontent.com/cel-rust/cel-rust/master/cel/src/lib.rs). It is not a compiled ACORN implementation. Compile each expression when loading a profile, then call `evaluate` with fresh bindings for each input. The `input` argument is the overview's envelope; the program is one manifest expression.

```rust
use cel::{Context, Program, Value as CelValue};
use color_eyre::eyre::{eyre, Result};
use serde_json::Value as JsonValue;

fn compile(expression: &str) -> Result<Program> {
    Program::compile(expression).map_err(|_| eyre!("Policy expression could not be parsed"))
}
fn evaluate(program: &Program, input: &JsonValue) -> Result<bool> {
    let mut context = Context::default();
    ["context", "params", "record"]
        .into_iter()
        .try_for_each(|name| {
            input
                .get(name)
                .ok_or_else(|| eyre!("Missing policy binding: {name}"))
                .and_then(|value| {
                    context
                        .add_variable(name, value.clone())
                        .map_err(|_| eyre!("Invalid policy binding: {name}"))
                })
        })
        .and_then(|()| {
            program.execute(&context).map_err(|_| eyre!("Policy evaluation failed"))
        })
        .and_then(|value| match value {
            CelValue::Bool(result) => Ok(result),
            _ => Err(eyre!("Policy expression must return a boolean")),
        })
}
```

`Ok(true)` maps to pass, `Ok(false)` to violation, and `Err` to evaluation-error; skipped is assigned by orchestration when valid input is unavailable. This boundary intentionally omits raw engine errors that may contain input values. The production adapter must add stable error codes, bounded safe locations, rule/profile provenance, and CheckBatch mapping. Its local mutable context is isolated to one invocation.

The sketch does **not** implement type checking, profile validation, input-size limits, evaluator fuel, or cancellation. It must not be used as the production boundary for untrusted expressions until slice 1 proves those protections. A host timeout around `evaluate` alone is insufficient. Verify these APIs against the selected published release before copying the sketch.

## CLI behavior to implement

Proposed flags make selection and enforcement explicit:

```sh
acorn --offline check activity.json --policy-profile lab-metadata.yaml
acorn --offline check activity.json --policy-profile lab-metadata.yaml --policy-enforce
```

The first command reports policy findings without making policy violations change the exit status; existing built-in checks retain their exit behavior. The second also fails for error-severity policy violations or evaluator errors. Reject `--policy-enforce` without a profile and reject its combination with `--no-fail`, so an explicit enforcement request cannot be silently neutralized. An unreadable or invalid profile is a command configuration error in either mode. Offline mode does not skip local CEL evaluation.

## Fixture cases and expected results

The following proposed JSON fixture tests the `keywords.allowed` expression independently of RAD decoding. Each case supplies bindings using the same contract as the overview; empty objects provide the unused `context` binding. The fixture runner compiles the expression once and evaluates every case with a fresh context.

```json
{
  "expression": "size(record.keywords) > 0 && record.keywords.all(keyword, keyword in params.allowed_keywords)",
  "cases": [
    {
      "expected": "pass",
      "input": {
        "context": {},
        "params": { "allowed_keywords": ["energy"] },
        "record": { "keywords": ["energy"] }
      },
      "name": "allowed keyword"
    },
    {
      "expected": "violation",
      "input": {
        "context": {},
        "params": { "allowed_keywords": ["energy"] },
        "record": { "keywords": [] }
      },
      "name": "empty keywords"
    },
    {
      "expected": "violation",
      "input": {
        "context": {},
        "params": { "allowed_keywords": ["energy"] },
        "record": { "keywords": ["unlisted"] }
      },
      "name": "unlisted keyword"
    }
  ]
}
```

Add separate fixtures for `record.title` returning a string (evaluation-error), malformed expressions (profile-load error), missing `record` (binding error), and invalid RAD (policy stage skipped with the original decode failure retained). Mutate a valid input between two evaluations to confirm bindings do not leak across records. These fixtures are proposed acceptance inputs, not tests executed during this documentation change.

# Validation

- Positive/negative corpus plus absent/null/empty, invalid types, numeric boundaries, unsupported functions, parse errors, duplicate IDs, incompatible versions, non-bool results, oversized expressions/inputs/results, nested comprehensions, and per-record/aggregate budget exhaustion.
- Assert deterministic finding order, correct field locations and issue counts, fresh bindings under concurrency, redacted errors, profile digest/version attribution, and no host effects.
- CLI snapshots for disabled/advisory/enforced modes, malformed input, invalid profile, skipped evaluation, and existing `--no-fail` semantics. Policy-off output/exit behavior must remain unchanged.
- Differential checks against a pinned reference CEL implementation for every supported language feature. Measure bounded termination rather than merely cancelling a waiting future.
- Focused analyzer and CLI tests first, then serialize `make lint`, `make test`, `make check-portable`, and `make check-wasm` after integration is stable. Confirm actual Makefile targets when implementing. An optional native engine must not enter default portable dependency graphs.
- Measure the workload and proposed budgets in the assessment; agree thresholds before implementation. Current assessment has no measured performance or compatibility result.

# Acceptance criteria

- [ ] A named profile owner supplies three requirements and can maintain their rules/tests.
- [ ] Pinned engine and supported CEL subset pass conformance and resource-limit gates; unsupported type checking is disclosed and the contract narrowed or adoption stopped.
- [ ] Built-in schema/FAIR results remain authoritative; policy-off behavior is unchanged.
- [ ] Every rule produces a distinguishable pass, violation, evaluation-error, or skipped state; explicit enforcement never treats evaluation failure as success.
- [ ] Offline evaluation uses only explicit inputs, with bounded outputs and reproducible provenance.
- [ ] Portable feature gates and native regression checks pass; performance/size budgets are documented and met or explicitly revised from measurements.
- [ ] User documentation and generated help explain selection, enforcement, compatibility, and testing.

When completed, remove this plan and its index/network/lane entries; retain implemented contracts and verified evidence in code, tests, and public documentation.

# Execution phases

## CEL-01 — Obtain owner-backed rules and fixtures

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: named validation-profile owner
- Exact write set: `tests/fixtures/cel/pilot.json`, this plan
- Primary integration lock: CEL pilot contract
- Outcome: Freeze three useful rules with positive, negative, missing, null, and type-mismatch cases.
- Focused validation: Review each expected finding and compare it with current Rust/Rhai behavior.
- Handoff: Remove CEL-01 and make the runtime gates ready.

## CEL-02 — Pin the dependency and license gate

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CEL-01
- Exact write set: `Cargo.toml`, `Cargo.lock`, `deny.toml`, this plan
- Primary integration lock: workspace manifests
- Outcome: Select a pinned CEL crate only if license, MSRV, dependency graph, and portability are acceptable.
- Focused validation: Run dependency/license checks and minimal native/portable builds.
- Handoff: Remove CEL-02 or record the stop decision without adding runtime code.

## CEL-03 — Characterize language and checker semantics

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CEL-02
- Exact write set: `tests/fixtures/cel/conformance.json`, this plan
- Primary integration lock: CEL pilot contract
- Outcome: Freeze the supported syntax, checking behavior, macros, numbers, nulls, errors, and reference differences.
- Focused validation: Run the fixture corpus through the selected crate and reference implementation.
- Handoff: Remove CEL-03 and document the supported subset.

## CEL-04 — Prove enforceable runtime budgets

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CEL-02
- Exact write set: CEL resource harness, `tests/fixtures/cel/resources.json`, this plan
- Primary integration lock: CEL runtime feasibility
- Outcome: Demonstrate bounded CPU, memory, recursion, input, output, and thread isolation or stop adoption.
- Focused validation: Exercise adversarial expressions at every proposed boundary.
- Handoff: Remove CEL-04 or record the stop gate and retire the plan.

## CEL-05 — Add the versioned profile manifest

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CEL-03, CEL-04
- Exact write set: `crates/acorn-lib/src/analyzer/cel/manifest.rs`, manifest tests
- Primary integration lock: analyzer profile schema
- Outcome: Decode strict rule IDs, messages, paths, severities, and boolean expressions with collected errors.
- Focused validation: Run valid, multi-error, unknown-field, and duplicate-ID tests.
- Handoff: Remove CEL-05 and freeze the serialized profile contract.

## CEL-06 — Implement compile-once evaluation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CEL-05
- Exact write set: `crates/acorn-lib/src/analyzer/cel/runtime.rs`, runtime tests
- Primary integration lock: CEL analyzer module
- Outcome: Compile profiles once and evaluate fresh allowlisted bindings within the proven limits.
- Focused validation: Test deterministic reuse, isolation, timeout/resource failure, and offline operation.
- Handoff: Remove CEL-06 and expose typed evaluation results.

## CEL-07 — Map findings and provenance

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CEL-06
- Exact write set: `crates/acorn-lib/src/analyzer/cel/mod.rs`, finding tests
- Primary integration lock: analyzer result model
- Outcome: Emit ordered findings, skipped/error states, profile identity, and bounded provenance.
- Focused validation: Compare results with the frozen expected fixture cases.
- Handoff: Remove CEL-07 and provide the analyzer boundary to CLI integration.

## CEL-08 — Add explicit CLI profile selection

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CEL-07
- Exact write set: `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/commands/check/mod.rs`, `CommandLineHelp.md`
- Primary integration lock: CLI syntax and generated help
- Outcome: Add explicit local profile selection with distinct advisory and enforcement behavior.
- Focused validation: Run parser, absent-profile, advisory, enforcement, offline, and generated-help tests.
- Handoff: Remove CEL-08 and freeze user-visible behavior.

## CEL-09 — Integrate analyzer summaries

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CEL-08
- Exact write set: `crates/acorn-analysis/src/check.rs`, analyzer summary tests
- Primary integration lock: check analyzer pipeline
- Outcome: Include CEL findings and skipped/error states without changing existing check ordering.
- Focused validation: Run mixed native/CEL batch and summary tests.
- Handoff: Remove CEL-09 and record compatibility evidence.

## CEL-10 — Measure conformance and resource cost

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CEL-09
- Exact write set: CEL benchmark harness, this plan
- Primary integration lock: CEL acceptance evidence
- Outcome: Record cold/warm performance, binary size, memory, and conformance against frozen budgets.
- Focused validation: Run the predeclared benchmark and resource corpus on the supported profile.
- Handoff: Remove CEL-10 and preserve dated measurements.

## CEL-11 — Prove owner-editable policy

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CEL-10
- Exact write set: representative CEL profiles, CEL user documentation
- Primary integration lock: CEL profile documentation
- Outcome: Have the named owner change a requirement and tests without Rust changes.
- Focused validation: Run the edited profile and review its resulting findings.
- Handoff: Remove CEL-11 and record owner acceptance.

## CEL-12 — Decide adoption and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: CEL-11
- Exact write set: CEL user documentation, wiki indexes, this plan
- Primary integration lock: CEL release decision
- Outcome: Keep the capability opt-in or remove it according to the frozen gates, then retire the plan.
- Focused validation: Run focused tests, portability, `make lint`, `make test`, docs/help checks, and wiki validation.
- Handoff: Delete this plan and all index, network, and lane entries after the decision is durable.
