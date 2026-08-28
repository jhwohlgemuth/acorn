---
type: Plan
title: "Add asynchronous validation rules"
description: "Extend ACORN's portable Validate contract and derive macro with ordered asynchronous custom rules."
difficulty: medium
impact: medium
package: [acorn-cli, acorn-lib, acorn-macros]
tags: [refactor, validation, async, macros, portability]
status: planned
---

# Goal

Support I/O-bound or otherwise asynchronous custom validation without replacing the synchronous validation API, requiring a second trait, or coupling validation to Tokio. Existing synchronous implementations and callers remain valid, while async-capable callers can request one complete synchronous-plus-asynchronous validation pass.

# Current evidence

- `acorn-lib::validation::Validate` exposes only `validate(&self) -> Result<(), ValidationReport>` and has portable implementations for `Vec`, arrays, and `Box`.
- `acorn-macros::Validate` generates one ordered synchronous accumulator. It supports field rules, optional wrappers, `each(...)`, `nested`, and schema-level rules with `skip_on_field_errors`.
- `collect_validation_checks` is synchronous even though every analyzer `check_schema` entry point is async. Those entry points currently combine synchronous reads and validation inside Rayon iterators.
- No ACORN code uses `dyn Validate`. Adding a return-position future to the trait therefore has no internal trait-object migration, but it intentionally makes the public trait non-object-safe.
- The workspace minimum Rust version is 1.95, so a portable return-position `impl Future` contract is available without `async-trait` or boxed futures.

# Recommendation

Keep one `Validate` trait and add a default `validate_async` method:

```rust
fn validate_async(
    &self,
) -> impl core::future::Future<Output = Result<(), ValidationReport>> + '_ {
    core::future::ready(self.validate())
}
```

`validate()` continues to run only synchronous rules. `validate_async()` is the complete pass and evaluates synchronous and asynchronous rules in declaration order. Manual implementations inherit the ready future unless they need to override it; users do not implement a separate async-validation trait.

Add `#[validate(async(function = "path"))]` in both supported attribute contexts:

- On a field, the function receives the same borrowed value shape as `custom(...)`, is awaited, and returns `Result<(), ValidationError>`. Optional fields skip `None`; `code` and `message` override returned error metadata exactly as synchronous custom rules do.
- On a struct, the function receives `&Self`, is awaited, and returns `Result<(), ValidationError>`. It supports `skip_on_field_errors`, defaulting to `true`, and runs after field rules like the existing `schema(...)` rule.
- Inside a collection, `each(async(function = "path"))` validates every item sequentially and reports every failing index. It accepts the same `Vec`, array, slice, `OneOrMany`, `BTreeMap`, reference, and optional-wrapper shapes as the existing `Each` path. Recursive `each` and nested-object validation inside `each` remain unsupported.

# Derive and propagation design

- Generate separate sync and async statement streams from the same parsed rule model. The async stream must not call `validate()` first because that would move all synchronous failures ahead of interleaved async rules.
- Preserve field order, rule order, schema-rule order, structured paths, codes, messages, parameters, and `skip_on_field_errors` behavior in both streams.
- Generate an async override for a derived type when it has a direct async rule or a `nested` path that can contain async validation. Nested fields call `Validate::validate_async(...).await`; sync-only children use the default ready future.
- Override `validate_async` for `Vec`, arrays, and `Box` so async validation propagates through existing blanket implementations. Sequence items are awaited in index order and accumulated with the existing path-prefix logic.
- Keep the contract in `core`/`alloc`. The validation API and generated code must not require Tokio, `async-trait`, `std`, `Send`, or heap-allocated futures. Host-backed rule implementations may be feature-gated separately.
- Treat future cancellation as ordinary Rust cancellation by drop. Retries, timeouts, caching, rate limiting, and side-effect policy belong to individual async rule implementations rather than the validation engine.

# Analyzer integration

Make `collect_validation_checks` async and await `validate_async()`. Adapt each async schema-analysis entry point by separating its existing parallel document reads from async validation; collect document futures in input order so final checks retain stable path and diagnostic ordering.

Rules within one document and items within one collection remain sequential. Different documents may be awaited concurrently with the existing `futures` dependency, provided the collector preserves input order. Strictly synchronous library entry points continue to call `validate()` and therefore do not perform external validation implicitly.

# Validation

- Unit-test the default ready future for existing manual implementations and explicit manual overrides.
- Test field and schema async rules, optional values, custom codes/messages, schema skipping, multiple failures, and mixed sync/async declaration order.
- Test `each(async(...))` across every existing `Each` shape, all failing indices, nested async propagation, nested collections through blanket implementations, and stable prefixed paths.
- Add macro compile-fail cases for missing or unknown arguments, async rules on unsupported types, invalid future output, recursive `each`, and unsupported nested rules inside `each`.
- Keep existing synchronous validation tests unchanged and add an assertion that generated validation uses portable paths and introduces no executor dependency.
- Update analyzer tests to await async collection and add `acorn check` compatibility snapshots proving documents without async rules retain exit status, stdout, stderr, diagnostic content, and ordering.
- Run `cargo check -p acorn-lib --no-default-features`, the `core` and `alloc` feature checks, `make wasm`, `make lint`, and `make test`.

# Acceptance criteria

- Existing manual `Validate` implementations compile without adding a method, and existing synchronous callers retain their behavior.
- A derived value can mix synchronous, async, nested, collection, and schema rules while producing one deterministic `ValidationReport` from `validate_async().await`.
- Async rules are never executed by `validate()` and are always included by `validate_async()`, including through nested values, `Vec`, arrays, and `Box`.
- Validation itself remains runtime-independent and portable; no Tokio, `async-trait`, boxed-future, or `std` requirement is added to the validation contract or generated code.
- `acorn check` awaits complete validation without changing output for schemas that declare no async rules.

# Scope and estimate

The first delivery adds async custom rules and propagation only. Concurrent rules within one value, async built-in network rules, retry/caching policy, and converting `validate()` into a breaking async-only API are out of scope. Estimated implementation effort is three to five development days.

Related work: [schema I/O validation handoff](schema-io-validation-handoff.md) and [std-only module isolation](../portability/isolate-std-only-modules.md).
