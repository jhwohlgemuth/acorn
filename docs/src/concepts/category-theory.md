# Rooted in Category Theory

🌱 **Small laws can support large systems.** ACORN borrows a few practical ideas from category theory and abstract algebra to combine diagnostics, transform configuration, and keep effects at clear boundaries.

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a Nutshell</p>
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

The current analyzer summary scans the checks for each category and immediately produces table strings. The planned summary monoid separates the count from its presentation:

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

The all-zero summary is the identity, and field-wise addition is associative. Partial reports can therefore be summarized independently and reduced to one value:

```rust,ignore
let total = reports
    .iter()
    .map(|report| CheckSummary::from_checks(report.checks().as_slice()))
    .fold(CheckSummary::default(), CheckSummary::combine);
```

The CLI can render `total` as a table while WebAssembly serializes it for JavaScript. The domain value defines what the counts mean; each interface owns how they look. This plan does not merge the analyzer summary with the separate discovery summary, which counts different things.

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

Schema validation already follows this distinction. Applying it across analyzer orchestration remains a planned audit and refactor: the analyzer currently collects every category but does so through explicit sequential loops. The goal is to make independence and ordering clear, not merely to replace a loop or add concurrency. CLI `--exit-on-first-error` remains an outer reporting policy rather than changing the underlying collection contract.

## Normalize, Then Materialize

An endomorphism transforms a value back into the same type. Vale configuration normalization uses that shape:

```text
ValeConfig → normalize packages → merge policy → deduplicate → ValeConfig
```

Each pure step is tested independently and composed in a declared order. Package-source transformation trims input, preserves paths and URIs, and expands named Vale packages. The normalized value is then passed to deterministic INI construction before host code performs release discovery and writes the configuration to disk. Offline materialization selects the pinned package fallback without release discovery.

Host operations discover remote releases, select fallbacks, create directories, write the INI file, and launch Vale only after normalization. Network and filesystem operations are effects, not configuration normalization. Keeping that boundary visible supports ACORN's [portability](aspect/portability.md) and offline guarantees.

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

| Pattern | Status | Intended benefit |
| --- | --- | --- |
| Ordered `CheckBatch` combination | Implemented | Flatten partial analyzer results without losing order. |
| Ordered `ValidationReport` accumulation | Implemented | Report independent schema failures together while preserving dependent skips. |
| Additive analyzer summary | Planned | Calculate once and render consistently across interfaces. |
| Applicative analyzer orchestration | Planned audit/refactor | Make collect-all, ordering, and prerequisite boundaries explicit. |
| Vale configuration endomorphisms | Implemented | Separate deterministic normalization from host effects. |

These patterns help ACORN grow higher-level workflows from dependable parts. Composition explains how the branches fit together; the next question is [...but is it agentic?](agentic.md)
