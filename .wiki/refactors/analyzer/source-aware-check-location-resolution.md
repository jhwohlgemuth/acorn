---
type: Plan
title: "Resolve check locations from source structure"
description: "Replace hand-maintained serde alias rewrites with conservative source-aware path matching, while reducing ambiguous schema aliases."
difficulty: high
impact: medium
package: [acorn-lib]
tags: [refactor, analyzer, serde, validation, migration]
status: planned
---

# Goal

Resolve validator and link check locations against the field names actually present in JSON, JSONC, and YAML documents without generating or maintaining a general serde alias lookup. Preserve exact-path behavior, support mechanical serde renames and retained aliases when the source provides enough evidence, and return an unlocated or ambiguous result rather than guessing when evidence is insufficient.

# Decision summary

1. Do not generate alias maps from serde attributes. Serde aliases are deserialization-only and are not available as stable runtime reflection metadata; a build-time source scanner would duplicate parts of `serde_derive` and would still need type-aware handling for nested structs, enums, `rename`, `rename_all`, and aliases.
2. Move alternate-name tolerance into `DocumentIndex`, where canonical validator paths, actual source paths, values, array indices, and spans can be considered together.
3. Keep exact path matching as the fast path. Use normalized and structural matching only after an exact lookup is missing.
4. Accept only a unique, sufficiently anchored best candidate. Preserve `DocumentMatch::Ambiguous` when duplicate values cannot be distinguished safely.
5. Reduce the three convenience alias groups most likely to create ambiguous validated paths:
   - Remove `url` and `href` aliases from `ImageObject::content_url` and `VideoObject::content_url`; require the canonical serialized key `contentUrl`.
   - Remove the `outcomes` alias from `Sections::impact`; require `impact`.
   - Remove the `profile` alias from `ContactPoint::url`; require `url`.
6. Retain external-format and established compatibility aliases for now, including `graphics`, `id`, `phone`, `orcid`, RAiD `organisation`, and DataCite XML-oriented aliases.

# Evidence

- `crates/acorn-lib/src/analyzer/error.rs` exposes one shared `locator_alias` function; `crates/acorn-lib/src/analyzer/check.rs` consumes it for validator and link queries. The duplication is in query construction, not in separately maintained alias tables.
- The current replacements cover `meta.identifier` to `meta.id`, `contact.telephone` to `contact.phone`, `media` to `graphics`, and `content_url` to `href`. Substring replacement is context-insensitive and offers only one alternate complete path.
- `ImageObject` and `VideoObject` use `rename_all = "camelCase"`, so `contentUrl` is the canonical serialized key, while the current lookup tries `content_url` and `href` but not `contentUrl`, `url`, or mixed canonical/alias paths.
- `tests/fixtures/data/invalid_project_b/index.yaml` uses the double-alias path `meta.graphics[].href` and the contact alias `profile`, so the planned reductions require fixture and migration updates.
- `Sections::impact` accepts `outcomes`, but the current manual lookup does not cover it. A repeated prose value at the same index in `approach` and `outcomes` is structurally ambiguous.
- RAiD fixtures use `organisation`. RAiD also contains repeated `schemaUri` fields and placeholder values, so this external compatibility alias must be retained while ambiguous matches remain conservative.
- DataCite aliases such as `$text`, `@schemeURI`, and `@rightsURI` represent external JSON/XML forms. Most validated fields remain distinguishable through normalized leaf names, exact ancestors, and collection indices.
- Enum variant aliases and aliases on fields that cannot emit validator errors do not affect check-location paths and are outside this refactor.

# Recommendation

Implement a conservative, source-aware fallback in `DocumentIndex` behind the existing exact-path fast path. Isolate aligned scoring and unique-best selection as private generic utilities, keep document-path policy in a thin adapter, reduce the three high-risk convenience alias groups, and return `Ambiguous` instead of guessing when duplicate values remain symmetric. Do not add serde metadata generation or canonical/source tree reconciliation unless measured results demonstrate that the simpler resolver is insufficient.

# Scope

## Included

- Validator and standard-link `DocumentQuery` construction.
- Exact, normalized, and structurally compatible path resolution in `DocumentIndex`.
- JSON and JSONC source entries and YAML semantic paths.
- Conservative handling of duplicate values.
- The three schema alias reductions listed in the decision summary.
- Fixture, documentation, migration-note, unit-test, integration-test, and benchmark updates required by those changes.

## Excluded

- Runtime serde reflection or a custom serde derive.
- A build script or `xtask` that parses Rust source attributes.
- Removal of external-standard aliases, enum aliases, or all legacy aliases.
- Fuzzy edit-distance or domain-specific synonym matching.
- Public exposure of the generic matching utilities.
- Canonical/source tree reconciliation in the initial implementation. It remains a possible escalation if measured ambiguity after the alias reductions is unacceptable.

# Resolution model

## Generic matching boundary

Add a private `crates/acorn-lib/src/io/document/matching.rs` module containing pure, allocation-free mechanisms that know nothing about documents, paths, serde, or aliases:

- `aligned_score(expected, actual, segment_score)` compares equal-length slices and returns no score when any paired segment is incompatible.
- `unique_max(candidates)` returns `Missing`, `Unique`, or `Ambiguous` and rejects tied highest-scoring candidates.
- Keep the types and functions `pub(super)`. Promote them to a general utility module only after a second production use exists.

The document adapter owns all policy:

- Key-to-key pairs are compatible.
- Equal keys are stronger than mechanically normalized keys.
- Differing keys may represent aliases but add no confidence.
- Index-to-index pairs are compatible only when the indices are equal.
- Key-to-index and index-to-key pairs are incompatible.
- Key normalization compares naming conventions such as `content_url`, `contentUrl`, and `content-url` without allocating normalized strings in the candidate loop.

Use a lexicographically ordered confidence value rather than an opaque summed score. It should prioritize exact indices, exact keys, normalized keys, and matching ancestry in a documented order. Array indices are hard constraints, not fuzzy evidence.

## Candidate acceptance

A structurally compatible candidate is eligible only when all of these conditions hold:

1. Exact lookup returned `Missing`.
2. Expected and actual paths have the same segment kinds and depth.
3. Every array index is equal.
4. The decoded scalar value matches, or the candidate scalar span contains the query needle for link checks.
5. At least one key segment is exact or mechanically normalized, unless the matching scalar occurs only once in the document.
6. The candidate has a unique highest confidence. A tie returns `DocumentMatch::Ambiguous`.

These conditions deliberately prefer a missing source position over a plausible but unproven position.

## Resolution order

`DocumentIndex::resolve` should apply this order:

1. Exact structured path with the existing value or needle constraint.
2. Mechanically normalized path with exact indices.
3. Structurally compatible source paths ranked by confidence and constrained by value or needle.
4. Existing conservative text fallback.
5. `Missing` or `Ambiguous`; never select the first tied candidate.

Analyzer query builders should submit only canonical validator or locator paths after the source-aware resolver is proven. Remove `locator_alias` and its call sites only after parity tests pass.

# Format handling

## JSON and JSONC

Reuse `DocumentEntry`, which already stores the actual source path, decoded scalar value, and physical span. Exact and structural resolution can return the entry span directly.

## YAML

Recursively flatten the existing semantic `serde_json::Value` into semantic path/value entries. Use the same generic matching and document-specific scoring policy to select the actual YAML path. Continue using the existing text search to obtain a physical span because the current YAML semantic representation does not contain spans. Repeated identical text may therefore remain ambiguous even after its semantic path is identified.

# Duplicate-value policy

Structural context mitigates duplicate values through exact ancestors, normalized leaf names, path depth, and exact array indices. It cannot solve symmetric inputs such as two aliased sibling fields with identical values and no remaining exact or normalized key evidence.

Do not use source order, validator error order, edit distance, or the first candidate as a tie-breaker. Return `Ambiguous` for symmetric cases. The three alias reductions remove the most likely RAD symmetry, especially the double-alias `media.content_url` to `graphics.href` path.

If corpus testing shows an unacceptable ambiguity rate after these changes, evaluate canonical/source tree reconciliation as a separate follow-up phase:

1. Serialize the validated value once into a canonical `serde_json::Value`.
2. Align canonical and source objects by exact keys, normalized keys, unique subtree fingerprints, and one-to-one elimination.
3. Cache the resulting correspondence once per document.
4. Preserve `Ambiguous` when multiple mappings remain possible.

Do not add this phase speculatively because it requires carrying canonical document context from schema validation to source attachment and increases implementation and memory cost.

# Schema alias reductions and migration

## Media URL aliases

Remove `alias = "url"` and `alias = "href"` from both media `content_url` fields. Keep `rename_all = "camelCase"`, making `contentUrl` the accepted serialized field name. Update RAD fixtures, examples, and documentation from `href` or media-local `url` to `contentUrl`. Add rejection tests for removed spellings and acceptance tests for `contentUrl`.

## Sections outcome alias

Remove `alias = "outcomes"` from `Sections::impact`. Update fixtures, examples, and documentation to `impact`. Add rejection and acceptance tests.

## Contact profile alias

Remove `alias = "profile"` from `ContactPoint::url`. Update fixtures, examples, and documentation to `url`. Add rejection and acceptance tests.

Treat these as intentional schema compatibility changes. Document them in the appropriate release or migration notes. Do not add a hidden preprocessing rewrite, because that would recreate the alias problem outside serde.

# Retained aliases

Retain and cover these categories with resolver tests:

- `ResearchActivityMetadata::identifier` accepting `id`.
- `ResearchActivityMetadata::media` accepting `graphics`.
- `ContactPoint::identifier` accepting `orcid`.
- `ContactPoint::telephone` accepting `phone`.
- RAiD `organization` accepting `organisation`.
- DataCite external JSON/XML aliases.

Aliases on unvalidated fields may remain because they do not affect validator check locations. Reassess them only as a separate schema simplification effort.

# Performance

- Preserve exact-path lookup as the common fast path.
- Run normalization and structural matching only after exact lookup is missing.
- Initially scan existing entries without adding another index; expected work is `O(entries * path depth)` only on fallback.
- Avoid temporary normalized `String` allocation in candidate comparisons.
- Avoid canonical-tree serialization and reconciliation in the initial implementation.
- If profiling identifies fallback scans as material, add a value-to-entry index so structural work considers only entries sharing the query value. Do not add it without benchmark evidence.
- If canonical/source reconciliation is later approved, compute fingerprints and correspondence once per document and cache them; never repeat an `O(canonical nodes * source nodes)` comparison for each issue.

# Implementation sequence

1. Add baseline tests for all four current manual rewrites, canonical camel-case paths, mixed canonical/retained-alias paths, duplicate values, and YAML behavior.
2. Apply the three schema alias reductions and update fixtures, examples, schema expectations, and migration documentation.
3. Add the private generic aligned-scoring and unique-best utilities with focused unit tests.
4. Add the document-specific normalization, confidence, and compatibility policy.
5. Add normalized and structural fallback resolution for JSON and JSONC.
6. Add semantic path enumeration and compatible resolution for YAML.
7. Route validator and link queries through canonical paths only, run parity tests, then remove `locator_alias` and all analyzer call sites.
8. Benchmark the resolver and representative offline check workloads. Add a value index only if the fallback scan produces a material regression.
9. Measure remaining `Missing` and `Ambiguous` cases across fixtures and an approved representative corpus. Create a separate canonical/source reconciliation plan only if the results justify its complexity.

# Validation

## Generic utility tests

Keep test function names prefixed with `test_` and alphabetically sorted. Cover:

- `test_aligned_score_rejects_different_lengths`
- `test_aligned_score_rejects_incompatible_segments`
- `test_aligned_score_sums_compatible_segments`
- `test_unique_max_recovers_after_a_lower_tie`
- `test_unique_max_rejects_tied_highest_scores`
- `test_unique_max_selects_the_highest_score`

## Document resolver tests

Keep test function names prefixed with `test_` and alphabetically sorted. Cover:

- Canonical exact paths remain preferred.
- `content_url` resolves to canonical `contentUrl` through mechanical normalization.
- `media` resolves against retained `graphics` when the leaf remains canonical `contentUrl`.
- `meta.identifier` resolves against retained `meta.id`.
- `contact.telephone` resolves against retained `contact.phone`.
- RAiD `organization[i].schema_uri` resolves against `organisation[i].schemaUri` when uniquely anchored.
- DataCite attribute and text aliases resolve when uniquely anchored.
- Duplicate values with a uniquely stronger structural candidate resolve correctly.
- Duplicate values with tied confidence return `Ambiguous`.
- Differing array indices are never treated as compatible.
- A candidate without a matching value or needle is rejected.
- YAML semantic alias paths gate text matching correctly.
- Repeated YAML text remains ambiguous when no physical span can be proven.

## Alias migration tests

- Confirm `contentUrl`, `impact`, and contact `url` deserialize and validate.
- Confirm removed media `href` and `url`, section `outcomes`, and contact `profile` are rejected under `deny_unknown_fields`.
- Update `tests/fixtures/data/invalid_project_b/index.yaml` and its bag copy to canonical spellings while preserving their intended validation failures.
- Confirm retained aliases continue to deserialize.

## Commands

1. Run focused `acorn-lib` document, analyzer, schema, DataCite, and RAiD tests during implementation.
2. Run `make lint`.
3. Run `make test`.
4. Record before-and-after resolver microbenchmarks and an offline check benchmark over representative fixtures. Target no more than a 5% median end-to-end regression; investigate and document any larger change before acceptance.

# Risks and mitigations

- **False-positive unique candidate:** Require path shape, exact indices, value or needle agreement, a confidence anchor, and a unique maximum.
- **Duplicate scalar values:** Use structural confidence and return `Ambiguous` on ties; do not guess.
- **Breaking alias removals:** Update fixtures and documentation together and publish a migration note in the release containing the change.
- **YAML lacks semantic spans:** Reuse semantic path validation plus conservative unique text matching.
- **Normalization collisions:** Treat normalization only as evidence, not proof; retain value constraints and unique-best selection.
- **Fallback performance:** Keep it miss-only, allocation-conscious, and benchmarked; add indexing only when measurements justify it.
- **Over-generalized utility API:** Keep matching utilities private to the document subsystem until a second use exists.

# Acceptance criteria

1. Analyzer query builders no longer contain or call a hand-maintained serde alias rewrite table.
2. Exact canonical source paths retain current location behavior.
3. Canonical camel-case names and retained aliases resolve without analyzer-side alias knowledge when source evidence is unique.
4. Tied duplicate-value candidates return `DocumentMatch::Ambiguous` and never select an arbitrary span.
5. JSON, JSONC, and YAML behavior is covered by focused tests.
6. Media `url` and `href`, section `outcomes`, and contact `profile` aliases are removed; canonical replacements, fixtures, documentation, and migration notes are updated.
7. `graphics`, `id`, `phone`, `orcid`, RAiD `organisation`, and DataCite external aliases remain supported.
8. Generic matching utilities are pure, private, independently tested, and contain no document- or serde-specific policy.
9. All new test function names use the `test_` prefix and remain alphabetically sorted within their modules.
10. `make lint` and `make test` pass.
11. Benchmark results meet the stated target or any larger regression is investigated, justified, and recorded before acceptance.
