# Rooted in Category Theory

🌱 **Small laws can support large systems.** ACORN borrows a few practical ideas from category theory and abstract algebra to combine diagnostics, transform configuration, and keep effects at clear boundaries.

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>ACORN uses a few ideas from category theory to combine results predictably, keep transformations simple, and separate pure logic from side effects.</p>
</blockquote>

## Why Composition?

ACORN reads research records, validates them, runs several kinds of analysis, and presents the resulting diagnostics through multiple interfaces. Those pieces need to combine without losing failures, changing report order, or coupling portable calculations to files, processes, and network access.

Category theory supplies a language for composition. Abstract algebra supplies structures such as monoids. The subjects are related, but the terms are not interchangeable. ACORN draws on both while keeping their use deliberately small:

| Idea | Practical meaning in ACORN |
| --- | --- |
| **Composition** | Connect transformations or combine partial results. |
| **Identity** | An empty value that changes nothing when combined. |
| **Associativity** | Regrouping combinations does not change the result. |
| **Monoid** | One value type with an associative combination and an identity. |
| **Endomorphism** | A transformation from a type back to the same type, such as `ValeConfig -> ValeConfig`. |
| **Applicative style** | Accumulate results from independent computations without making one result the input to the next. |

These names describe useful laws and boundaries. ACORN does not expose generic `Category`, `Monoid`, or `Applicative` traits. A concrete operation earns its place when callers use it and its laws make changes safer.

## Ordered Checks

Analyzer operations already return an ordered `CheckBatch`. An empty batch contributes nothing, while `combine` preserves every check from left to right:

```rust,ignore
let checks = schema_checks
    .combine(link_checks)
    .combine(prose_checks);
```

`CheckBatch::empty()` acts as the identity:

```text
empty + checks = checks = checks + empty
```

Combination is associative:

```text
(schema + link) + prose = schema + (link + prose)
```

ACORN has tests for both laws and for insertion order. Combination is not commutative: `schema + link` and `link + schema` contain the same checks in a different report order. That intentional distinction lets callers regroup partial work without sacrificing deterministic output.

The portable schema `ValidationReport` has a similar collect-all shape. Independent field rules contribute ordered issues to one flat report. Dependent whole-value and asynchronous rules can still be skipped after field errors when they cannot produce meaningful diagnostics from an invalid prerequisite.

## Additive Summaries

Analyzer diagnostics are summarized by an additive `CheckSummary` monoid that separates the domain count from its presentation:

```rust,ignore
#[derive(Clone, Copy, Default)]
struct CheckSummary {
    schema: usize,
    link: usize,
    prose: usize,
    quality: usize,
    readability: usize,
    crosswalk: usize,
}

impl CheckSummary {
    fn combine(self, other: Self) -> Self {
        Self {
            schema: self.schema + other.schema,
            link: self.link + other.link,
            prose: self.prose + other.prose,
            quality: self.quality + other.quality,
            readability: self.readability + other.readability,
            crosswalk: self.crosswalk + other.crosswalk,
        }
    }
}
```

`CheckSummary::zero()` (`crates/acorn-lib/src/analyzer/check.rs:313`) is the identity and `combine` (`check.rs:326`) is associative with `counts()`/`rows()` for rendering (`check.rs:337,348`). Laws are tested in `crates/acorn-lib/src/analyzer/tests/mod.rs:94,107,123`. Partial reports can therefore be summarized independently and reduced to one value:

```rust,ignore
let total = reports
    .iter()
    .map(|report| CheckSummary::from(report.checks().as_slice()))
    .fold(CheckSummary::zero(), CheckSummary::combine);
```

The CLI renders `total` as a table while WebAssembly serializes it for JavaScript. The domain value defines what the counts mean; each interface owns how they look. The remaining `analyzer-composition` slice moves any lingering table-string scanning off the domain and onto the hosts. This monoid does not merge with the separate discovery summary, which counts different things.

## Independent Checks, Dependent Steps

Applicative-style validation applies when several checks receive the same input and none needs another check's output:

```text
                 ┌─ schema check ─┐
research input ──├─ link check ───┼─ ordered report
                 └─ prose check ──┘
```

Every branch can contribute diagnostics even when another branch reports a failure. The checks may execute sequentially or concurrently; independence does not require parallelism. Their declared order can still determine the order of the combined report.

Dependent work grows differently:

```text
bytes → deserialize → validate value → resolve identifiers
```

Identifier resolution cannot consume a value that failed to deserialize. Hiding that dependency behind collect-all machinery would make the control flow less accurate. ACORN keeps prerequisite chains in ordinary `Result` control flow and reserves applicative-style accumulation for genuinely independent work.

Schema validation already follows this distinction via ordered `ValidationReport` accumulation independent branches, with whole-value and async rules skipped after prerequisite field errors. The same distinction is being applied across analyzer orchestration in the consolidated [analyzer composition plan](https://code.ornl.gov/research-enablement/acorn/-/blob/main/.wiki/refactors/analyzer/analyzer-composition.md): the service currently collects every category through explicit sequential loops (`host/service/mod.rs:30` `Analysis::check`, `154` `analyze<T>`, `177` `analyze_paths`), and the plan makes independence (`schema | link | prose | quality | readability | crosswalk` on the same input) and ordering (stable report order) explicit in a shared traversal/fold. CLI `--exit-on-first-error` remains an outer reporting policy rather than changing the underlying collection contract.

## Context Through Composition

ACORN becomes [context-aware](acorn.md#context-awareness) by composing partial views of the same research activity. Schema, link, prose, quality, readability, and crosswalk checks each answer a different question about one input. Their results retain their category and order when combined, so the final report carries more context than a single pass or fail result. A caller can see what ACORN observed, which rule produced the observation, and where it belongs in the record.

Prerequisite chains determine when ACORN has enough context for the next operation. Deserialization must produce a typed value before validation can interpret its fields, and identifier resolution needs a valid identifier before it can add external evidence. This ordering prevents a later step from treating malformed input as meaningful context.

Pure transformations and explicit effect boundaries keep that context consistent across interfaces. The CLI and WebAssembly can render the same ordered checks differently without changing what the checks mean. Network and filesystem operations may add evidence or materialize a result, but they do so after the portable transformations have established the value being carried forward.

## Normalize, Then Materialize

An endomorphism transforms a value back into the same type. Vale configuration normalization already uses that shape (`crates/acorn-lib/src/analyzer/host/vale.rs:89`):

```text
ValeConfig → normalize packages → merge policy → deduplicate → ValeConfig
```

Each pure step (`normalize_packages` `host/vale.rs:107`, `merge_policy` `115`, `deduplicate` `124`) is tested independently and composed in declared order as `normalize()` (`host/vale.rs:89` fold; `tests/mod.rs:787` ordering/idempotence). Package-source transformation `resolve_package` (`host/vale.rs:94`) trims input, preserves paths and URIs, and expands named Vale packages. The normalized value is then passed to deterministic `ini` construction before host code performs release discovery and writes the configuration to disk. Offline materialization selects the pinned fallback (`DEFAULT_VALE_PACKAGE_URL` / `vale-package` at `v0.0.1`) without release discovery.

Host operations discover remote releases, select fallbacks, create directories, write the INI file, and launch Vale only after normalization (`host/runtime/mod.rs:217` `check_prose_for`, `728` `StaticAnalyzer<ValeConfig>`, `1005` `sync`). Network and filesystem operations are effects, not configuration normalization. Keeping that boundary visible supports ACORN's [portability](aspect/portability.md) and offline guarantees.

A consuming method implemented with local `mut` is not automatically impure from its caller's perspective. A builder that consumes and returns a value may still compose cleanly. The meaningful question is whether the transformation's result depends on hidden external state.

## Avoid Unnecessary Abstraction

Category-inspired design is a constraint, not a destination. ACORN does not need:

- a generic category, monoid, or applicative framework;
- an abstraction used by only one call site;
- concurrency merely because operations are independent;
- algebraic terminology in user-facing diagnostics;
- effectful work disguised as a pure transformation; or
- a rewrite of clear `Result`, iterator, or builder code.

The practical test is simple: does an operation have an identity, an associative combination, or a pure transformation boundary that real callers use? If so, naming and testing that law can provide strong roots for later changes. If not, a direct function is healthier.

## Current Direction

| Pattern | Status | Intended benefit | Evidence |
| --- | --- | --- | --- |
| Ordered `CheckBatch` combination | Implemented | Flatten partial analyzer results without losing order. | `check.rs:279` `empty` / `289` `combine`; `tests/mod.rs:21,26,36` |
| Ordered `ValidationReport` accumulation | Implemented | Report independent schema failures together while preserving dependent skips. | `acorn_schema::validation::ValidationReport` + `acorn_core::validation` derive |
| Additive analyzer summary | Implemented | Calculate once and render consistently across interfaces. | `check.rs:313` `CheckSummary::zero` / `326` `combine` / `337` `counts` / `348` `rows`; `tests/mod.rs:94,107,123,57` |
| Applicative vs dependent orchestration | In progress | Make collect-all (`schema \| link \| prose \| quality \| readability \| crosswalk`) vs prerequisite chains (`bytes → deserialize → validate → resolve`) explicit. | Consolidated in `.wiki/refactors/analyzer/analyzer-composition.md` slices 3-4; current `host/service/mod.rs:30,154,177` loops pending fold |
| Vale configuration endomorphisms | Implemented | Separate deterministic normalization from host effects. | `host/vale.rs:89` `normalize` pipeline; `tests/mod.rs:787,854` |

These patterns give ACORN a stable way to accumulate context without losing its source, category, or order. The remaining orchestration and link work is tracked as a single consolidated plan: [analyzer composition and traversal](https://code.ornl.gov/research-enablement/acorn/-/blob/main/.wiki/refactors/analyzer/analyzer-composition.md). Composition explains how the branches fit together; the next question is [...But Is It Agentic?](agentic.md).
