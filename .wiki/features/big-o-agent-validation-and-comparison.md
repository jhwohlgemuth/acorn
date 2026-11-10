---
type: Plan
title: Big-O metadata validation and agent comparison
description: Validate Big-O expressions in complete research-activity metadata and expose deterministic comparison through ACORN MCP, native routing, WASM, and WebMCP interfaces.
difficulty: medium
impact: high
package: [acorn-core, acorn-schema, acorn-lib, acorn-cli, acorn-wasm]
tags: [feature, big-o, validation, mcp, webmcp, wasm]
status: in-progress
execution: phased
phase_budget_hours: 5
---

# Goal

Make Big-O metadata validation a first-class, agent-usable ACORN capability. A complete Research Activity Data (RAD) document must validate every time and space complexity expression, report malformed notation and inconsistent variable declarations at the metadata location, and preserve those findings through `acorn check`, the ACORN MCP server, WASM, and WebMCP. After validation succeeds, an agent may compare two expressions with explicit or deterministic inferred growth assumptions.

The release-critical outcome is document validation. Pairwise comparison is a second operation built on the same parser and proof rules; it must not delay, bypass, or weaken RAD validation.

An MCP-connected agent should use this sequence:

1. Call `acorn.validate_research_activity_data` with a complete RAD object.
2. Correct every Big-O `syntax`, `reference`, `nonempty`, or `duplicate` finding returned for computation complexity metadata.
3. Extract two validated expressions from the document.
4. Call `acorn.compare_big_o` with optional symbol-growth assumptions.
5. Treat `incomparable` and `undetermined` as successful analytical outcomes, not tool failures.

`acorn.check_research_activity_data` remains a compatibility alias for step 1. Documentation and examples use `acorn.validate_research_activity_data` as the canonical name.

# Current evidence

Evidence rechecked on 2026-11-09 against `ad93d5e8` and the current worktree:

- `acorn-core::big_o` has a bounded ASCII parser, located `ParseError`, an `Expression` tree, lexical variable iteration, and limits of 256 input bytes, 16 nesting levels, and 64 parsed nodes. The accepted grammar includes addition, implicit and explicit multiplication, division, positive integer/rational/named-difference powers, `log`, named logarithm bases, `polylog`, and `W_out`.
- `acorn-core::big_o::compare_notation` now provides the serializable, cross-transport request/result contract over the existing proof engine. Pair inference classifies ordinary variables as growing and named logarithm bases as fixed, explicit assumptions override inference, and typed errors distinguish left/right parse failures, assumption overflow, and lexical-first unknown names.
- `AlgorithmComplexity::is_valid` parses every `ComplexityBound`, emits `syntax` for parser failures, checks referenced names against a present `variables` map with `reference`, and reports empty or repeated bounds with `nonempty` and `duplicate`. The committed `tests/fixtures/data/big_o/` suite proves complete valid, invalid-syntax, and invalid-reference RAD documents, exact nested paths, expression round trips, and closed 256-character bound schemas. MCP protocol coverage now exercises those same complete documents.
- `acorn check` preserves nested Big-O validation through its existing interfaces. The complete valid fixture succeeds in human and JSON modes; invalid syntax/reference fixtures retain the `schema` category, `error` severity, unsuccessful JSON status, exact metadata locator, fixture-relative URI, and parser/reference message. Piped human output retains the existing CSV success behavior, while JSON mode exits nonzero for invalid fixtures.
- `ToolRegistry::with_database` registers `acorn.validate_research_activity_data`, compatibility alias `acorn.check_research_activity_data`, and MCP-only `acorn.compare_big_o`. The validation tools accept complete `ResearchActivity` input and return `{ valid, errors }`; comparison accepts `{ left, right, assumptions? }` and returns `{ left, right, dominance, assumptions }`. The comparison definition is read-only and absent from Needle's catalog. Wire-level `tools/list` and `tools/call` tests cover the three complete RAD fixtures, alias equivalence, inferred and explicit comparison, malformed sides, unknown assumptions, and output limits. The MCPB v0.3 manifest fixture matches the registry.
- `acorn-wasm` exports object-based `compareBigO({ left, right, assumptions? })`, delegates to the portable comparison contract, and returns `{ left, right, dominance, assumptions }` with JSON-compatible ordered assumption properties. Bounded `validateBigOList(string[])` and `compareBigOList(BigOComparisonInput[])` APIs inspect the JavaScript array length before deserialization, process up to 64 ordered items, collect every expression-validation result, preserve comparison order, and identify the first malformed pair by index. The crate retains structured `validateResearchActivity` diagnostics for all three complete RAD fixtures.
- WebMCP exposes `acorn_compare_big_o` in a sorted nine-tool registry with matching TypeScript declarations and a ninth fixture-derived demo tab. Its adapter enforces exact fields, 256-byte expression and assumption-name limits, 64 assumptions, growth-value enums, pre/post-call cancellation, bounded success and failure envelopes, request-error mapping, and exact result normalization. The rebuilt package passed 25 adapter tests, 11 WASM binding tests, and a nine-tool native smoke in interactive Chrome 156.0.8078.80 with the WebMCP testing flag enabled. Browser timings and the displayed PASS are recorded in the demo README.
- The computation, schema, check, MCPB, and web package documentation now describes the supported grammar and validated agent workflow. BGO-08 final publication and integration gates remain pending.
- PAR-34's synthetic routing safety matrix has passed and its CLI inference files are released. The former PAR-15 automatic-selection gate is retired after a fresh 44/75 correct calibration and a nonpassing 66/75 rule-assisted probe; unscoped Needle selection remains preview-only. BGO-05 has started: the native `acorn.diff::bigo` v1 capability and `native:acorn.diff::bigo` binding delegate to `compare_notation` with local-read effects and offline support. Focused library tests cover schema/effect equality, scoped discovery, explicit dispatch, invalid input, cancellation, no artifacts, and no fallback; focused CLI routing tests cover the explicit offline route. Routed execution currently returns `ToolFailed` without the left/right parser detail required by the comparison error policy, so BGO-05 remains in progress. Natural-language selection is not scored by BGO.
- Focused implementation gates pass: `cargo test -p acorn-schema computation` (10 passed), `cargo test -p acorn-core --test big_o` (22 passed), `cargo test -p acorn-cli --test check big_o`, `cargo test -p acorn-lib --features agentic io::mcp::tests::server` (17 passed, including MCPB fixture and wire-level protocol coverage), `cargo test -p acorn-wasm` (24 unit tests and 2 doctests), `wasm-pack test --node crates/acorn-wasm` (11 binding tests and 2 doctests), `node --test crates/acorn-wasm/web/webmcp.test.mjs` (25 passed), staged direct-WASM single/list execution, `make build-wasm`, `make check-wasm`, strict all-target `acorn-wasm` Clippy, `cargo check -p acorn-core --no-default-features --features alloc`, focused alloc-only Clippy for the core library plus `big_o` integration test, and strict `acorn-schema` library/test Clippy. The broader alloc-only `--all-targets` Clippy command is not a valid BGO gate because unrelated std-only tests currently fail to compile under that feature set.

# Boundaries

## In scope

- Complete RAD fixtures with valid and invalid computation complexity metadata.
- Runtime validation of Big-O notation and, when a `variables` map is present, every referenced symbol.
- Existing `acorn check` human and JSON output for nested Big-O findings.
- Existing MCP RAD validation/check tools, with descriptions and server-boundary tests that explicitly cover Big-O metadata.
- One new read-only MCP tool, `acorn.compare_big_o`.
- One matching native routed capability, `acorn.diff::bigo`, with binding `native:acorn.diff::bigo`.
- A portable WASM `compareBigO` binding and WebMCP tool `acorn_compare_big_o`.
- One comparison contract, assumption policy, error policy, and result vocabulary across MCP, routed native, WASM, and WebMCP.
- Generated schema and user documentation for RAD bounds and comparison inputs/outputs.

## Out of scope

- A new `acorn compare` or Big-O-specific CLI subcommand. `acorn check` remains the CLI surface for document validation.
- A standalone `acorn.validate_big_o` MCP tool. Document validation belongs to the existing complete-RAD validation tools; comparison validates its two expressions before analysis.
- Inferring whether an author's claimed complexity is scientifically or algorithmically correct.
- Simplifying arbitrary expressions, evaluating constants, benchmarking algorithms, or measuring runtime behavior.
- Changing the accepted grammar except to repair a defect demonstrated by the committed fixtures or existing parser corpus.
- Replacing the current CLI check JSON envelope, adding a Big-O category/rule ID, or removing the MCP validation tool's existing `errors` field.
- Network access, filesystem writes, mutation authority, inference/model execution, or fallback to a remote comparator.
- Reopening the frozen PAR routing corpus during active PAR/ADP release work. BGO native routing initially proves explicit discovery and dispatch; any later natural-language routing evaluation requires a separately versioned corpus revision.

# RAD validation contract

## Metadata shape

Keep the existing serialized shape. A valid stage may contain:

```json
{
  "identifier": "00000000-0000-4000-8000-000000000002",
  "operation": "classical",
  "complexity": {
    "case": "worst",
    "bounds": [
      { "time": "O(n log n)" },
      { "space": "O(n)" }
    ],
    "variables": {
      "n": "Number of records"
    }
  }
}
```

`bounds` retains its `OneOrMany<ComplexityBound>` representation. Each item is exactly one externally tagged `time` or `space` string. A `variables` map is optional for backward compatibility. When it is present, every name returned by `Expression::variables()` must be a key; unused declarations remain permitted.

The generated JSON Schema should describe the two bound alternatives with `additionalProperties: false` and a 256-character maximum:

```json
{
  "oneOf": [
    {
      "type": "object",
      "additionalProperties": false,
      "required": ["time"],
      "properties": {
        "time": {
          "type": "string",
          "maxLength": 256,
          "description": "Big-O time-complexity expression"
        }
      }
    },
    {
      "type": "object",
      "additionalProperties": false,
      "required": ["space"],
      "properties": {
        "space": {
          "type": "string",
          "maxLength": 256,
          "description": "Big-O auxiliary-space expression"
        }
      }
    }
  ]
}
```

JSON Schema advertises structural and length constraints but does not duplicate the grammar as a regular expression or attempt the cross-field variable check. Runtime parsing remains authoritative. The parser accepts ASCII only, so its 256-byte limit and the schema's 256-character bound coincide for accepted expressions.

## Validation behavior

For each bound, preserve this deterministic order:

1. Reject an empty/whitespace-only expression with code `nonempty`.
2. Parse the complete value through `Expression::parse`; report failure with code `syntax` and message `Invalid complexity expression: {parse_error}`.
3. If `variables` is present, iterate parsed names in lexical order and emit one `reference` issue per missing name with message ``Complexity variable `{name}` is not defined``.
4. If an earlier bound has the same resource kind, emit `duplicate` for the later bound.

Do not trim or canonicalize the stored expression. Successful round trips preserve the original notation exactly.

Nested document validation must prefix the local `bounds[index]` path through `aspect.computation[index].stages[index].complexity`. Human and JSON integrations may add document locators, but tests assert the stable metadata-path suffix rather than an OS-dependent absolute filename.

## Fixture design

Create three complete projects by copying the same valid RAD envelope and changing only `aspect.computation`:

| Fixture | Complexity content | Expected result |
|---|---|---|
| `valid/index.json` | One stage with `O(n log n)` time and `O(n)` space; a later dependent stage with `O(n^2)` time; a third stage with `O(P*B+2*log(B)*t*H^2/e)` and declarations for `P`, `B`, `t`, `H`, and `e` | Deserializes, validates, round-trips, and passes `acorn check` |
| `invalid-syntax/index.json` | Replace the first time expression with `O(n - m)` while retaining declarations for both names | One `syntax` issue at the first complexity bound; no secondary reference issue |
| `invalid-reference/index.json` | Retain `O(n log n)` but provide a nonempty variables map that declares `m` instead of `n` | One `reference` issue naming `n` at the first complexity bound |

Use stable UUIDs for the computation and stages. Every `dependsOn` value names an earlier stage. Do not introduce missing infrastructure/profile references or unrelated vocabulary warnings that would obscure the Big-O assertions. The valid fixture's `O(n^2)` and `O(n log n)` strings are the canonical pair reused by core, MCP, native, WASM, and WebMCP integration tests.

# Comparison contract

## Portable Rust API

Extend `acorn_core::big_o` rather than implementing transport-specific parsing or comparison:

```rust
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Dominance {
    Left,
    Right,
    Equivalent,
    Incomparable,
    Undetermined,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SymbolGrowth {
    Growing,
    FixedPositive,
    FixedLogBase,
    ShrinkingPositive,
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonInput {
    #[schemars(length(max = 256))]
    pub left: String,
    #[schemars(length(max = 256))]
    pub right: String,
    #[serde(default)]
    #[schemars(length(max = 64))]
    pub assumptions: BTreeMap<String, SymbolGrowth>,
}

#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonOutput {
    pub left: String,
    pub right: String,
    pub dominance: Dominance,
    pub assumptions: BTreeMap<String, SymbolGrowth>,
}
```

Add a typed `ComparisonError` with separate left/right parse failures, too many assumptions, and unknown assumption names. It implements `Debug`, `Display`, `core::error::Error`, `Eq`, and `PartialEq`; messages name the rejected side or symbol and carry the parser offset where applicable. Public items receive rustdoc and examples. No `color-eyre`, host type, or `std` dependency enters `acorn-core`.

Expose one pure entry point:

```rust
pub fn compare_notation(input: ComparisonInput) -> Result<ComparisonOutput, ComparisonError>;
```

`GrowthAssumptions` gains only the access required by this operation: deterministic pair inference, lookup, and ordered iteration. Keep its map private and return borrowed names from iteration. Do not expose `BTreeMap` mutation as a public invariant bypass.

## Inference and validation rules

`compare_notation` performs these steps in order:

1. Parse `left` and `right` independently; preserve their original strings in successful output.
2. Reject more than 64 explicit assumption entries.
3. Collect the union of symbols referenced by both expressions.
4. Reject every explicit assumption key absent from that union; report lexical-first when more than one is unknown.
5. Infer ordinary symbols as `growing`.
6. Infer symbols used as named logarithm bases as `fixed-log-base`; this wins over ordinary use of the same name.
7. Overlay explicit assumptions after inference. Explicit values may therefore override either inferred class.
8. Call the existing `compare_growth` proof engine with the effective assumptions.
9. Return the effective assumptions in lexical key order.

`fixed-positive` and `shrinking-positive` are never inferred. They are available only as explicit analytical assumptions.

`left` means the left expression grows strictly faster; `right` means the right expression grows strictly faster. `equivalent`, `incomparable`, and `undetermined` are successful outputs. Only malformed inputs or violated request limits are errors.

## JSON Schema

All transport adapters expose the same logical input schema:

```json
{
  "type": "object",
  "additionalProperties": false,
  "required": ["left", "right"],
  "properties": {
    "left": { "type": "string", "maxLength": 256 },
    "right": { "type": "string", "maxLength": 256 },
    "assumptions": {
      "type": "object",
      "default": {},
      "maxProperties": 64,
      "additionalProperties": {
        "enum": [
          "growing",
          "fixed-positive",
          "fixed-log-base",
          "shrinking-positive"
        ]
      }
    }
  }
}
```

The output schema is:

```json
{
  "type": "object",
  "additionalProperties": false,
  "required": ["left", "right", "dominance", "assumptions"],
  "properties": {
    "left": { "type": "string" },
    "right": { "type": "string" },
    "dominance": {
      "enum": ["left", "right", "equivalent", "incomparable", "undetermined"]
    },
    "assumptions": {
      "type": "object",
      "additionalProperties": {
        "enum": [
          "growing",
          "fixed-positive",
          "fixed-log-base",
          "shrinking-positive"
        ]
      }
    }
  }
}
```

# Interface behavior

## CLI check

No new output type is introduced. For invalid syntax, `acorn check --format json` continues to emit its existing check envelope and record shape:

```json
{
  "checks": [
    {
      "record": "check",
      "category": "schema",
      "severity": "error",
      "success": false,
      "locator": "aspect.computation[0].stages[0].complexity.bounds[0]",
      "message": "Invalid complexity expression: ..."
    }
  ],
  "fixes": {}
}
```

Tests should assert existing required fields and a stable locator suffix rather than replacing the complete check schema with this abbreviated example. Do not assign a new `rule_id`.

## ACORN MCP server

The existing canonical validation call remains:

```json
{
  "name": "acorn.validate_research_activity_data",
  "arguments": {
    "activity": {
      "...": "complete RAD object"
    }
  }
}
```

Validation findings are a successful tool result:

```json
{
  "valid": false,
  "errors": [
    "aspect.computation[0].stages[0].complexity.bounds[0]: Invalid complexity expression: ..."
  ]
}
```

Malformed tool arguments or failure to deserialize a complete `ResearchActivity` remain tool errors. Preserve `ValidationOutput` for compatibility; do not add a second Big-O validation response format.

Register `acorn.compare_big_o` as read-only, idempotent, non-destructive, closed-world, and MCP-only:

```json
{
  "name": "acorn.compare_big_o",
  "arguments": {
    "left": "O(n^2)",
    "right": "O(n log n)"
  }
}
```

```json
{
  "left": "O(n^2)",
  "right": "O(n log n)",
  "dominance": "left",
  "assumptions": {
    "n": "growing"
  }
}
```

Set `ToolExposure { needle: false, mcp: true }` on this built-in definition. The separate native binding supplies Ask/Needle routing and avoids a duplicate projection of the same operation.

Update the validation and compatibility-alias descriptions to say that complete RAD validation includes Big-O time/space syntax and declared-variable checks. Keep the existing warning that agents must not synthesize partial activity objects from prose.

## Native routed capability

Register the same typed input/output contract as:

- Capability ID: `acorn.diff::bigo`
- Binding ID: `native:acorn.diff::bigo`
- Version: `1.0.0`
- Effects: read-only, idempotent, non-destructive, closed-world
- Availability: ready and offline-safe

The handler delegates directly to `compare_notation`; it does not spawn a process or call the MCP server recursively. Explicit capability/binding selection, catalog discovery, schema equality, offline selection, and one terminal dispatch receive focused tests. Do not alter the frozen model-routing corpus in this plan.

## WASM and WebMCP

Export the portable operation:

```typescript
export function compareBigO(input: BigOComparisonInput): BigOComparisonOutput;
```

The Rust WASM binding accepts a JS object through `serde-wasm-bindgen`, delegates to `compare_notation`, and returns a JSON-compatible object. It does not accept parallel positional arguments that could drift from the MCP schema.

Add WebMCP tool `acorn_compare_big_o` and typed adapter method:

```typescript
compareBigO(
    input: BigOComparisonInput,
    options?: ExecutionOptions,
): Promise<ToolResult<BigOComparisonOutput>>;
```

WebMCP performs cheap shape/byte/count checks before crossing into WASM, while Rust remains authoritative. Map errors as follows:

| Condition | WebMCP code | MCP/native behavior |
|---|---|---|
| Left/right parse failure | `invalid_arguments` | Tool/capability input error with side and parser message |
| Unknown assumption name | `invalid_arguments` | Tool/capability input error naming the symbol |
| More than 256 UTF-8 bytes per expression or more than 64 assumptions | `input_too_large` | Tool/capability input-limit error |
| Cancellation before or after WASM invocation | `cancelled` | Existing protocol cancellation behavior |
| Malformed WASM result or unexpected internal failure | `execution_failed` | Tool/capability internal error without fallback |
| Serialized WebMCP result over the existing output limit | `result_too_large` | Existing MCP output-size policy |

Adding the tool changes the WebMCP `ToolName` union, registered-tool assertion, and demo tabs from eight to nine.

# Validation matrix

| Layer | Valid fixture | Syntax fixture | Reference fixture | Comparison pair | Limits/errors |
|---|---|---|---|---|---|
| `acorn-schema` | Full RAD validates and round-trips | Exact `syntax` code/path/message | Exact `reference` code/path/message | Not applicable | Bound schema advertises 256; empty/duplicate unit cases remain |
| `acorn check` | No schema failure | Human and JSON finding | Human and JSON finding | Not applicable | Existing JSON envelope unchanged |
| MCP validation | `{ valid: true, errors: [] }` | Successful result with path/message | Successful result with path/message | Not applicable | Incomplete RAD remains a tool error |
| MCP comparison | Not applicable | Invalid left/right is a tool error | Not applicable | `O(n^2)` dominates `O(n log n)` | Unknown/overflow assumptions rejected; incomparable/undetermined succeed |
| Native routing | Not applicable | Typed execution error | Not applicable | Explicit capability/binding dispatch matches core | Offline-safe, no mutation, no fallback, one terminal result |
| WASM | RAD validation remains valid | Structured validation diagnostic | Structured validation diagnostic | Direct `compareBigO` matches core | Typed JS error for invalid request |
| WebMCP | Existing RAD tool remains valid | Structured validation diagnostic | Structured validation diagnostic | Ninth tool matches WASM/core | Error codes, cancellation, result bounds, tool count |

# Acceptance criteria

1. Complete RAD metadata containing every supported fixture expression deserializes, validates, and round-trips without changing the expression strings.
2. Complete RAD metadata containing invalid syntax or an undeclared symbol in a present `variables` map produces the expected validation code, message, and nested metadata path.
3. `acorn check` exposes those findings in human and JSON modes without a new category, rule ID, or envelope change.
4. The ACORN MCP server's `tools/list` includes the canonical RAD validation tool, its compatibility alias, and `acorn.compare_big_o` with accurate schemas, descriptions, and read-only effects.
5. Actual MCP protocol calls validate all three RAD fixtures and compare the canonical fixture pair; tests do not stop at direct handler invocation.
6. MCP invalid RAD findings are successful validation results, while malformed comparison arguments are tool errors with actionable side/symbol information.
7. The native routed capability discovers and explicitly dispatches the same contract offline without mutation authority, process startup, fallback, or recursive MCP use.
8. Direct WASM and WebMCP results exactly match the core dominance and effective-assumption map for the same requests.
9. WebMCP exposes nine tools and the demo contains nine matching tabs/examples.
10. Documentation defines the grammar, validation semantics, inference policy, success/error distinction, and recommended agent workflow without claiming that ACORN proves an author's algorithmic assertion.
11. Focused phase validation and the final lint, test, WASM-portability, and book gates pass.

# Dependencies and coordination

- Complete RAD fixtures, the portable Big-O contract, and CLI integration coverage are complete.
- Direct WASM and WebMCP comparison and validation coverage are complete; router work remains independently locked.
- The MCP registry, protocol tests, schema descriptions, and MCPB manifest are accepted. WME-10 may use the MCP registry and server-test files after this handoff.
- PAR-34 released `crates/acorn-cli/src/commands/infer/mod.rs` and its routing-scope tests for BGO-05. PAR-16 and ADP-07 are not prerequisites for explicit capability discovery and dispatch.
- BGO-08 final validation follows BGO-05 acceptance; its documentation edits can be checked independently.
- No dependency, feature, or lockfile change is planned. If implementation proves one necessary, stop and create a bounded workspace-graph phase rather than expanding an existing phase.

Prepared execution order on 2026-11-09:

- BGO-05: Preserve actionable left/right parser and assumption errors through the routed response without exposing unrelated handler diagnostics. The route and focused tests are otherwise in place; the frozen routing corpus remains unchanged.
- BGO-08: The current documentation builds successfully. After BGO-05 acceptance, add the native-dispatch example, validate all examples and run the exclusive final gates, then retire the plan if all acceptance criteria hold.

# Execution phases

## BGO-05 — Register the native routed Big-O capability

- Status: in-progress
- Budget: 5 active hours maximum
- Prerequisites: Accepted CLI inference-file handoff
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/native/big_o.rs`, `crates/acorn-lib/src/io/mcp/tools/native/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/native/tests/big_o.rs`, `crates/acorn-lib/src/io/mcp/tools/native/tests/mod.rs`, `crates/acorn-cli/src/commands/infer/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/routing_scope.rs`; any routed-error contract extension requires an explicit bounded write-set amendment before implementation.
- Primary integration lock: CLI inference registry
- Current state: Version `1.0.0` local-read, offline-safe discovery and explicit dispatch match `compare_notation`; two focused library tests, eight focused CLI tests, and strict library/CLI Clippy pass. Malformed notation yields `ToolFailed` without the parser's side and message in `RoutedExecution`.
- Outcome: Complete the routed comparison error policy, preserve explicit offline-safe dispatch and one terminal result, and keep the frozen natural-language routing corpus unchanged.
- Focused validation: Assert left/right and unknown-assumption errors at the caller-visible routed boundary, schema/effect equality, discovery, explicit binding selection, cancellation, no mutation requirement, no fallback, and zero artifacts for failures.
- Handoff: Record capability/binding/version/effects and release the routing lock; remove BGO-05 only after the error policy passes. Any request to score natural-language selection becomes a separately versioned routing-evaluation phase.

## BGO-08 — Publish the agent workflow and run release gates

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: BGO-05; stable worktree and exclusive final-validation window
- Exact write set: `docs/src/concepts/aspect/computation.md`, `docs/src/packages/acorn-schema.md`, `docs/src/cli-reference/commands/check.md`, `docs/src/packages/mcpb.md`, `docs/src/packages/acorn-web.md`, `.wiki/features/big-o-agent-validation-and-comparison.md`, `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: Documentation generation
- Current state: Grammar, RAD, MCP, WASM, and WebMCP documentation edits build with `make book`; nine-tool browser evidence and explicit offline native-dispatch tests are recorded. Native-dispatch documentation, final validation, and plan retirement wait for BGO-05's routed error policy.
- Outcome: Add a verified native-dispatch example after BGO-05, validate the published workflow and examples, run final gates, and retire this plan only after every acceptance criterion is durable outside `.wiki`.
- Focused validation: `make lint`; `make test`; `make check-wasm`; `make book`; validate example JSON against generated schemas; run `.agents/skills/planning/scripts/validate_wiki.py`; check internal links and confirm MCP/WebMCP tool counts in documentation match tests.
- Handoff: If all gates pass, delete this concept and remove its feature-index, root-ranking, network, and parallel-lane entries in the same change. A failed product gate creates a bounded repair phase with its own write set; do not broaden BGO-08.
