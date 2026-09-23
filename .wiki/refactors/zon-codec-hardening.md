---
type: Plan
title: "Review and harden the ZON codec"
description: "Audit the pinned ZON encoder and decoder, simplify duplicated grammar logic, and strengthen conformance evidence without changing the public wire contract."
difficulty: medium
impact: medium
package: [acorn-schema]
tags: [refactor, serialization, zon, testing, portability]
status: planned
---

# Goal

Review the portable ZON encoder and decoder as one codec, then make the smallest changes that improve correctness, diagnostic consistency, and maintainability. Preserve the implemented strict profile, canonical output, public `encode`/`decode`/`decode_value` API, stable error codes, alloc-only portability, and pinned TypeScript reference compatibility.

# Project code boundaries

- **Update:** `crates/acorn-schema/src/standard/zon/decode.rs`, `crates/acorn-schema/src/standard/zon/encode.rs`, `crates/acorn-schema/src/standard/zon/mod.rs`, `crates/acorn-schema/src/standard/zon/tests/`, and `docs/src/concepts/zon.md` only when the audit clarifies documented behavior.
- **Create:** None; add focused cases to the existing ZON test module and regression corpus.
- **Delete:** None; consolidate duplicate helpers in place only after characterization tests protect their behavior.

# Dependencies

The first-class ZON feature is implemented, so this plan has no implementation prerequisite. It is a leaf in the portable schema-codec lane and owns the exact `crates/acorn-schema/src/standard/zon/` write set while active. It may proceed independently of [semantic transformation laws](../testing/semantic-transformation-laws.md), but edits to shared codec-law tests or ZON fixtures must serialize with that plan's owning slice.

# Current evidence

- `decode.rs` is a 585-line hand-written parser with separate root-line, table/CSV, quoted-string, and inline-compound paths. Root classification sometimes calls a parser to test whether syntax matches and later parses the same input again.
- Quoted strings and escapes are handled by both `parse_quoted`/`unescape` and `InlineParser::quoted`; their error locations and accepted transitions can drift.
- Array, object, and depth limits are checked during inline parsing and again by the post-parse `validate_value` walk. The encoder also threads a depth value through recursive helpers after pre-validation owns the actual limit decision.
- `normalize` allocates a replacement string and a line vector after document-size validation. Encoder helpers construct intermediate strings and vectors at each compound level. These costs are bounded, but there is no measured reason yet to trade clarity for a streaming rewrite.
- Existing golden tests cover canonical numbers, aliases, tables, quoting, malformed inputs, error codes, and bounded limits. The recursive property strategy covers nulls, booleans, `i32` values, strings, arrays, and objects, but it does not generate the wider supported JSON-number space or directly characterize every parser transition and error location.
- Public entry points and typed `OneOrMany` integration already pass the `acorn-schema` suite. The review must preserve that behavior rather than reopen format integration.

# Recommendation

1. Build a compact behavior matrix for root values, grouped objects, nested compounds, tables, quoting/escaping, number forms, aliases, line endings, duplicate keys, limits, and every `ZonErrorCode`. Add characterization tests for uncovered boundaries before restructuring parser code.
2. Give each grammar responsibility one implementation. Reuse one quoted-string/escape routine across root, CSV, and inline paths where their rules are identical; keep CSV doubling separate where the pinned profile differs. Classify root syntax without constructing and discarding full parse errors or parsing the same header twice.
3. Assign each resource limit one authoritative enforcement point and prove that encoding and decoding return the same stable code at the boundary. Retain pre-allocation document and line checks, and avoid a second recursive validation walk when parser or encoder traversal has already proved the same invariant.
4. Remove traversal state that does not affect output, especially encoder depth plumbing after limit ownership is explicit. Consolidate key/string validation and escaping only when the rules and error codes are identical.
5. Improve diagnostics without expanding the public error schema: keep canonical code messages, bounded context, and one-based line/column reporting; add exact-location tests for invalid escapes, malformed numbers, duplicate keys, table headers, and row-width failures.
6. Extend property and regression coverage for signed and fractional numbers, exponent normalization, nested quoted compounds, Unicode, CRLF input, near-limit structures, and encode/decode determinism. Add a regression seed for every discovered failure before applying its fix.
7. Measure representative flat, nested, and tabular fixtures before changing allocation strategy. Optimize only a demonstrated hotspot, and keep the codec dependency-free and alloc-only.

# Validation

- `cargo test --package acorn-schema zon`
- `cargo test --package acorn-schema`
- `cargo clippy --package acorn-schema --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- `make check-portable`
- `make check-wasm`

# Acceptance criteria

1. A checked-in test matrix covers every supported syntax family, every stable error code, and exact source locations for representative failures.
2. Each quoting, escaping, root-classification, and resource-limit rule has one implementation or an explicit format-specific reason for remaining separate.
3. `decode_value(encode(value)) == value` and byte-for-byte deterministic encoding hold across the expanded bounded property domain and pinned reference fixtures.
4. Public functions, canonical output, error-code strings and messages, `.zonf` integration, and strict-profile exclusions remain backward compatible.
5. Malformed or near-limit input does not panic, recurse past the configured limit, or allocate before the applicable size check.
6. Any performance-motivated change includes before/after evidence and does not add a runtime dependency or weaken alloc-only and WASM checks.

# Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Consolidating similar parser paths changes a subtle quoting or alias rule. | High | Land characterization cases first and preserve CSV-specific behavior separately. |
| Moving limit checks changes which stable code wins when several limits fail. | Medium | Record precedence in tests before changing ownership and keep it explicit. |
| Allocation cleanup grows into a streaming-codec rewrite. | Medium | Require measurement and keep streaming or alternate modes outside this plan. |
| Broader schema work edits the same module root or fixtures. | Medium | Hold the ZON codec and fixture lock for one atomic slice and rebase before validation. |

# Related plans

- [Semantic transformation laws](../testing/semantic-transformation-laws.md)
- [Crate boundary architecture](crate-boundary-architecture.md)
