---
type: Plan
title: Portable Big-O notation parser and literal macro
description: Parse bounded Big-O expressions without std, validate computation bounds, and expose a compile-time checked bigo! literal.
difficulty: medium
impact: medium
package: [acorn-core, acorn-schema, acorn-macros]
tags: [feature, computation, complexity, parsing, portability, macro]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Give ACORN one grammar for Big-O notation in research-activity metadata and Rust literals. Parse expressions under `no_std` with `alloc`, validate `ComplexityBound` strings through that parser, and provide `bigo!("O(n log n)")` for compile-time checked literals. Parsing establishes syntax and named variables; it does not prove an algorithmic bound or infer complexity from timings.

# Current evidence

- The schema has `AlgorithmComplexity { case: Option<ComplexityCase>, bounds: OneOrMany<ComplexityBound>, variables: Option<BTreeMap<String, String>> }`. `ComplexityCase` records `worst`, `average`, `amortized`, or `best` for all bounds in an entry; `ComplexityBound::{Space, Time}` each carry a `String`. Validation currently rejects empty and duplicate resource bounds, but does not parse notation or inspect variable references. Phase COM-02 must recheck the accepted schema baseline before editing it.
- The computation guide shows `O(n log n)` and `O(n)` as strings. Execution profiles record concrete workload sizes and durations independently of the declared expressions.
- `acorn-core` has `core`, `alloc`, and `std` feature paths. `acorn-schema` already uses `alloc` in its minimal build and depends on both `acorn-core` and the separate `acorn-macros` proc-macro crate. Its direct `acorn-core` dependency currently has no features enabled, so schema integration must explicitly enable `acorn-core/alloc`. `acorn-macros` does not currently depend on `acorn-core`, so an alloc-only dependency can share the parser without a cycle.
- `make check-portable` checks the minimal and alloc-only crate paths; `make check-wasm` checks `wasm32-unknown-unknown`. Existing macro tests use `trybuild` for compile failures.

# Decisions and boundaries

1. Put the parser and its typed expression tree in `acorn-core` behind its `alloc` feature. The API accepts `&str` and returns a typed expression or an error with a byte offset and stable error category. It does no I/O, timing, fitting, or evaluation. Do not introduce a `std` dependency or a new crate.
2. The first grammar accepts uppercase `O(...)`; positive integer constants; ASCII identifiers; `+`, explicit or adjacent multiplication, grouping, positive integer or identifier exponents, `log(identifier)`, `log(grouped expression)`, and the shorthand `log n` for one identifier. It accepts the existing `O(n log n)` example. Operator precedence is exponent, then multiplication, then addition; `log n^2` is rejected as ambiguous, so callers write `log(n^2)` or `(log n)^2`. `Theta`, `Omega`, division, fractional exponents, arbitrary functions, and semantic simplification remain outside this plan.
3. Bound input to 256 bytes, nesting to 16 levels, and parsed nodes to 64. Errors identify the first offending byte or limit. Reject empty expressions, malformed operators, zero or negative exponents, trailing tokens, and invalid UTF-8 boundaries without panicking. Preserve the supplied notation rather than rewriting mathematically equivalent expressions.
4. Keep the serialized `ComplexityBound` values as strings. Enable `acorn-core/alloc` on `acorn-schema`'s direct dependency so parsing is available in every schema build, including `--no-default-features`; the standalone `acorn-core` minimal build remains independent. Schema validation parses each string and, when `variables` is supplied, rejects expression identifiers absent from that map. An omitted `variables` map remains valid; unused entries are allowed. Existing empty and duplicate-bound checks continue to apply.
5. `bigo!` accepts one string literal, invokes the same `acorn-core` parser during macro expansion, and expands to the original `&'static str` literal. It reports a compile error on the literal for invalid notation. Callers can write `ComplexityBound::Time(bigo!("O(n log n)").into())`. The macro does not require a runtime parser call and does not couple the parser to `big_o` or any measurement crate.

# Dependencies and coordination

- COM-01 has no implementation prerequisite. COM-02 and COM-03 consume its parser API. COM-04 follows both integrations.
- COM-02 requires the pending `ComplexityBound` shape to be accepted in its checkout. If it changes before implementation, update this plan's exact contract and write set rather than imposing the current working-tree shape.
- COM-02 and COM-03 serialize their manifest edits through the workspace-graph lock. COM-01 and COM-04 serialize crate-root registration and public re-exports through the library-roots lock. COM-02 also coordinates computation schema files with other schema owners. Other schema, macro, and portable-codec work may proceed on disjoint files.
- This plan is independent of empirical complexity inference, execution-profile fitting, and the `big_o` dependency decision.

# Validation

- Focused parser cases cover every accepted operator and precedence rule, nested groups, variable extraction, malformed forms, Unicode rejection, and all three resource limits.
- Macro compile-pass and compile-fail cases prove valid literals expand to `&'static str`, invalid literals fail with useful diagnostics, and use from a `no_std` consumer does not pull in a runtime parser.
- Computation schema cases prove JSON round trips, nested error paths, optional variable-map behavior, and retained duplicate and empty-bound behavior.
- Check `acorn-core` and `acorn-schema` with minimal and alloc-only features; check the relevant WASM targets; run focused macro and schema checks before final repository gates.

# Acceptance criteria

1. Rust callers and JSON schema validation use the same documented grammar and reject the same malformed expression.
2. Parser errors expose a stable category and byte offset; over-limit input returns an error without excessive allocation or recursion.
3. `ComplexityBound` JSON remains a string-valued `time` or `space` enum entry, and an omitted variables map remains accepted.
4. `bigo!` accepts exactly one literal, checks it at compile time, and expands to a usable `&'static str` without runtime parsing.
5. Minimal, alloc-only, and WASM portability gates remain green. No `std` or `big_o` dependency enters the portable parser or schema path.
6. The computation guide and API docs distinguish notation validation from mathematical proof and measured inference.

# Execution phases

## COM-01 — Land the portable parser

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-core/src/big_o.rs` (create), `crates/acorn-core/src/lib.rs`, `crates/acorn-core/tests/big_o.rs` (create)
- Primary integration lock: library roots
- Outcome: Public alloc-only typed expression parser, bounded grammar, variable extraction, and located errors.
- Focused validation: `cargo test -p acorn-core --no-default-features --features alloc --test big_o`; `cargo check -p acorn-core --no-default-features`; `cargo check -p acorn-core --no-default-features --features alloc --target wasm32-unknown-unknown`.
- Handoff: Remove COM-01; record the accepted parser API and grammar in rustdoc for schema and macro consumers.

## COM-02 — Validate computation bounds with the parser

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COM-01 and an accepted checkout containing the `ComplexityBound` enum
- Exact write set: `crates/acorn-schema/Cargo.toml`, `crates/acorn-schema/src/computation.rs`, `crates/acorn-schema/src/computation/tests/mod.rs`, `docs/src/concepts/aspect/computation.md`
- Primary integration lock: workspace graph
- Outcome: Parse each bound during validation and check identifiers against an optional variables map while preserving the JSON shape and existing bound rules.
- Focused validation: Focused computation schema cases; `cargo check -p acorn-schema --no-default-features`; `cargo check -p acorn-schema --no-default-features --features alloc`; and the schema WASM check.
- Handoff: Remove COM-02; document accepted and rejected JSON examples and exact validation paths.

## COM-03 — Add the compile-time literal macro

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COM-01
- Exact write set: `crates/acorn-macros/Cargo.toml`, `crates/acorn-macros/src/lib.rs`, `crates/acorn-macros/src/big_o.rs` (create), `crates/acorn-macros/tests/big_o_compile.rs` (create), `crates/acorn-macros/tests/ui/big-o-invalid-syntax.rs` (create), `crates/acorn-macros/tests/ui/big-o-invalid-syntax.stderr` (create), `crates/acorn-macros/tests/ui/big-o-nonliteral.rs` (create), `crates/acorn-macros/tests/ui/big-o-nonliteral.stderr` (create), `Cargo.lock`
- Primary integration lock: workspace graph
- Outcome: Public `acorn_macros::bigo!` uses the core parser at compile time and emits the original string literal.
- Focused validation: Focused macro compile-pass and `trybuild` compile-fail cases plus `cargo check -p acorn-macros`.
- Handoff: Remove COM-03; retain accepted diagnostics and macro examples in rustdoc.

## COM-04 — Expose and verify the complete capability

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: COM-02, COM-03
- Exact write set: `crates/acorn-schema/src/lib.rs`, `docs/src/packages/acorn-schema.md`, `.wiki/features/portable-big-o-notation.md`, `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: library roots
- Outcome: Re-export `bigo!` from `acorn-schema`, document Rust and JSON use together, run integration and portability gates, and retire this plan when acceptance passes.
- Focused validation: Focused core/schema/macro suites; `make check-portable`; `make check-wasm`; relevant rustdoc and lint checks; wiki validation.
- Handoff: Delete this plan and its index, network, and lane references after durable docs and all acceptance criteria pass.
