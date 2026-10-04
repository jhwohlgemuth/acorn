---
type: Plan
title: "Pilot optional CEL metadata validation profiles"
description: "Verify a bounded Rust CEL implementation and add explicitly selected local metadata rules through the existing analyzer pipeline."
difficulty: medium
impact: high
package: [acorn-lib, acorn-cli]
tags: [feature, policy, validation, cel]
status: planned
---

# Goal and recommendation

Allow a team to express three concrete institutional metadata requirements without altering the RAD schema or writing a Rhai workflow. Ship an optional local CEL profile only if the runtime and authoring gates pass. See the [OPA/CEL assessment](../assets/opa-cel-review.md) for technology comparisons, sources, alternatives, and estimates. OPA authorization, remote distribution, browser bindings, and cross-record policy are outside this atomic plan.

# Current evidence

Verified 2026-10-01: ACORN has ordered `ValidationReport` and `CheckBatch` results, analyzer execution states, restricted Rhai hooks, and shared operation effect enforcement. No OPA/CEL plan was found in the wiki. The inspected `cel-rust` `Program::compile` path parses source; full type checking and execution budgets must be verified against a pinned release before adoption. Existing functionality is the comparison baseline, not migration scope.

# Dependencies and scheduling

- Coordinate feature wiring with [dependency capabilities](../refactors/portability/dependency-features-and-macos-size.md). Reuse current in-tree analyzer boundaries; no crate extraction prerequisite.
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
