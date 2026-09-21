---
type: Plan
title: "Enforce semantic transformation laws"
description: "Turn category-theory-inspired invariants into focused property and commuting-path tests for RAD normalization, linked-data projection, crosswalks, codecs, and merge behavior."
difficulty: medium
impact: high
package: [acorn-schema, acorn-lib, acorn-cli]
tags: [testing, property-testing, research-activity, normalization, json-ld, crosswalk, merge]
status: planned
---

# Goal

Make ACORN's existing transformations trustworthy under composition without adding mathematical vocabulary to the public API. Define the semantic meaning preserved at each boundary, encode the applicable identity, idempotence, preservation, and commuting-path laws as tests, and require every intentional loss to remain explicit.

The design applies category theory as an engineering discipline:

```text
canonical RAD --normalize--> canonical RAD --project--> JSON-LD
      |                                               ^
      +---------------- workflow/link ----------------+
```

The useful result is not a `Category` or `Functor` trait. It is evidence that two supported paths which claim the same meaning produce equivalent results.

# Current evidence

- `crates/acorn-lib/src/io/adapter/mod.rs` implements `InputOutput::format_with` for `ResearchActivity` and creates linked content through `ResearchActivity::with_context`.
- `crates/acorn-lib/src/io/workflow/mod.rs` formats before optional enrichment, formats again after enrichment, and creates the linked artifact through `linked_content`.
- `crates/acorn-cli/src/commands/link/mod.rs` follows a separate path: it formats, calls `serialize_as` using the source MIME type, and writes the result with a `.jsonld` extension. It does not currently call the shared `linked_content` projection used by workflow orchestration.
- `crates/acorn-schema/src/research_activity/mod.rs` already distinguishes canonical RAD from its derived JSON-LD context through `LinkedData::with_context`.
- `crates/acorn-schema/src/standard/crosswalk/` already provides `Crosswalk`, `FieldMapping`, `Fields`, `ConversionWarning`, and property tests for selected reversible mappings.
- `crates/acorn-schema/src/research_activity/output.rs` and `crates/acorn-lib/src/io/database/candidate.rs` implement different, intentional merge policies. Neither merge should be assumed commutative.
- `proptest` is already a dev dependency of `acorn-schema` and `acorn-lib`; this plan adds no property-testing dependency.

# Design decisions

## Keep categorical concepts out of the public surface

Do not add public `Category`, `Morphism`, `Functor`, `NaturalTransformation`, generic `Transform`, or `SemanticallyEquivalent` traits. The current traits already describe the operational boundaries contributors use. Add narrowly named production helpers only when a law exposes duplicated behavior that must be shared by two real consumers.

Do not add `RelationshipPath` as part of this plan. ACORN's current graph utility models organizational traversal, while RAD relationships are spread across typed identifiers, outputs, standards adapters, and linked-data projection. A path type needs a concrete query or reasoning consumer and a typed relation vocabulary; otherwise it would imply compositions the domain does not define.

## Define equivalence per boundary

There is no single universal equality relation for every ACORN representation. Use the smallest comparison appropriate to each boundary:

- **Canonical RAD equality:** typed `ResearchActivity` equality after normalization with the same filesystem context.
- **Linked-data equivalence:** canonical RAD fields are preserved after ignoring only derived JSON-LD decoration such as contexts and derived types.
- **Codec equivalence:** JSON, JSONC, and YAML decode to the same normalized typed value; Markdown compares only the structural fields its documented format can express.
- **Crosswalk equivalence:** compare the exact and reversible fields identified by `.wiki/assets/standards-field-matrix.md`; approximated, simplified, omitted, or unsupported fields must produce the matching structured loss.
- **Merge equivalence:** compare the documented result and provenance/conflict behavior for that specific merge operation. Never substitute set equality for ordered fields unless the schema declares order irrelevant.

Start with crate-private or test-only projection helpers colocated with the law suites. Promote a reusable semantic projection into production only after at least two non-test consumers need the same operation.

# Law catalog

| Boundary | Law | Required interpretation |
|---|---|---|
| Normalization | `N(N(x, c), c) = N(x, c)` | Formatting is idempotent for the same context `c`. Context-dependent relative media resolution is part of the input, not ignored noise. |
| Linked-data projection | `strip(L(N(x, c))) = N(x, c)` | Removing only derived JSON-LD decoration recovers the normalized canonical fields. |
| Projection determinism | `L(x) = L(x)` | Repeated projection produces byte-stable structured JSON after parsing; object-key ordering is not treated as semantics unless output stability is the explicit contract. |
| Workflow commutation | `link_command(x) ~= workflow(format + link, x)` | The direct link command and workflow route use the same linked-data projection and agree on the emitted document and output path. |
| Structured codecs | `decode_f(encode_f(N(x))) = N(x)` | Apply to JSON/JSONC/YAML within each format's supported shape. Markdown uses its documented structural subset instead of claiming full losslessness. |
| Reversible crosswalk fields | `back(forward(x)) ~= x` | Compare only fields classified exact or reversible in the standards matrix. |
| Loss accounting | `lossy(x) => warnings(x) != []` | Every omitted, approximated, simplified, or unsupported source value has a structured warning; target-required absence remains an error. |
| Output merge | `merge(x, x) = x`; `merge(x, empty) = x` | Preserve the existing left-biased fill-empty policy. Do not require commutativity. |
| Candidate merge | repeated identical evidence is unchanged | Replaying evidence does not duplicate values or provenance; populated conflicts keep the stored value and record the observation. Do not require commutativity because precedence is intentional. |
| PID normalization | supported spelling variants parse to one typed identity | Reuse PID-specific equality and checksum behavior; do not compare raw strings as semantic identity. |

`~=` means the boundary-specific equivalence defined above, not byte equality.

# Project code boundaries

- **Update:** `crates/acorn-schema/src/research_activity/tests/`, `crates/acorn-schema/src/standard/crosswalk/` tests, `crates/acorn-lib/src/io/adapter/tests/`, `crates/acorn-lib/src/io/workflow/` tests, `crates/acorn-lib/src/io/database/` tests, and the focused link tests owned by `crates/acorn-cli/src/commands/link/`.
- **Create:** dedicated `laws.rs` test modules under the existing research-activity, crosswalk, adapter, and workflow test directories when splitting the existing modules improves ownership; add a dedicated semantic-transform fixture directory under `tests/fixtures/data/` only for cases not expressible with current fixtures.
- **Delete:** None. Remove duplicated command-local transformation code only if the commuting-path test first proves it differs from the shared workflow projection and the focused implementation slice explicitly owns that repair.

Shared module indexes, `crates/acorn-cli/src/commands/link/mod.rs`, and shared fixtures are integration locks. An implementation slice must name the exact files it will own before editing them.

# Dependencies and coordination

- The normalization, codec, linked-data, and merge laws can start against current contracts.
- Crosswalk preservation and loss-accounting laws consume the implemented [standards field matrix](../assets/standards-field-matrix.md) and existing property tests instead of creating a competing crosswalk model.
- Command/workflow commutation coordinates with [CLI test coverage phase 1](cli-coverage-phase-1.md), which owns the first focused link-command suite. Only one slice edits `crates/acorn-cli/src/commands/link/mod.rs` or its test module at a time.
- This plan does not depend on schema versioning, a generic migration framework, crate extraction, or a new graph model.

# Implementation slices

## 0. Pin the law vocabulary and counterexamples

Write a table-driven test catalog beside the affected tests. For each law, record the operation, comparison projection, generated input domain, expected losses, and one minimal counterexample. Reuse existing valid-project, format, standards, and database fixtures before adding any new fixture.

The first counterexample should cover the separate direct-link and workflow-link paths. Capture current behavior without approving it as correct; the accepted contract is that both routes emit the `linked_content` representation.

## 1. Prove normalization and codec laws

- Add `format_with` idempotence properties for no context, a stable project context, relative media, canonical and aliased organization names, optional collections, and already formatted values.
- Add JSON, JSONC, and YAML round-trip properties over bounded valid `ResearchActivity` values.
- Keep generators validation-aware so failures represent transformation defects rather than arbitrary invalid schemas.
- Add a separate Markdown structural projection and enumerate unsupported/lossy fields; do not force full typed equality where the Markdown contract is intentionally narrower.

## 2. Prove linked-data preservation and path commutation

- Parse linked JSON and verify that adding contexts/types does not change canonical RAD values.
- Verify projection determinism and that no absent domain assertion is fabricated.
- Add a focused comparison between the direct `link` command and `Workflow<ResearchActivity>` with `format=true`, `link=true`, `enrich=false`, `offline=true`.
- If the comparison exposes the current divergence, route the command through the shared linked-content/workflow seam in a separate, narrow repair commit, then keep the law as the regression test.

## 3. Extend crosswalk preservation and loss laws

- Build generators from `Fields` and typed standard records using the approved field matrix.
- For each exact or reversible mapping, verify forward-then-back preservation of the classified fields.
- For each lossy classification, assert the precise `ConversionWarning` kind and source/target path.
- Test composed routes only when both paths are supported contracts, such as a direct adapter versus the same adapter through the canonical `Fields` representation. Do not invent direct migrations solely to complete a diagram.

## 4. Prove merge-policy laws

- Add idempotence and empty-candidate identity tests for `ResearchOutput::merge`.
- Add replay, unique-array, stable-order, conflict-precedence, and provenance-deduplication properties for candidate persistence merging.
- Include negative tests documenting that left-biased merges are not generally commutative. These protect intentional precedence from a future "algebraic cleanup" that would change user-visible conflict behavior.

# Validation

Run focused tests per slice, then the repository gates appropriate to changed production code:

```sh
cargo nextest run -p acorn-schema --lib research_activity
cargo nextest run -p acorn-schema --lib standard::crosswalk
cargo nextest run -p acorn-lib --lib io::adapter
cargo nextest run -p acorn-lib --lib io::workflow
cargo nextest run -p acorn-lib --lib io::database
cargo nextest run -p acorn-cli --lib commands::link
cargo check -p acorn-schema --no-default-features
make check-wasm
make lint
make test
```

All law tests are deterministic, bounded, offline, and report the smallest failing input through `proptest`. No snapshot alone is accepted as proof of semantic preservation.

# Acceptance criteria

- The law catalog names the exact comparison relation for every tested boundary; no test relies on an undefined global notion of semantic equality.
- Normalization is idempotent for the same context across generated valid RAD inputs.
- JSON, JSONC, and YAML round trips preserve normalized canonical RAD; Markdown preservation is limited to a documented structural projection.
- Linked-data projection preserves canonical fields, adds only documented derived decoration, and fabricates no research assertion.
- The direct link command and workflow link path emit equivalent JSON-LD through one shared projection seam.
- Exact/reversible crosswalk fields survive supported round trips, and every intentional loss produces the classified structured warning.
- `ResearchOutput` and candidate merges satisfy their documented idempotence, identity, replay, ordering, conflict, and provenance laws without being mislabeled commutative.
- PID spelling variants already accepted by ACORN resolve to the same typed identity.
- No public category-theory trait, speculative relationship path, universal transform abstraction, or schema migration framework is introduced.
- Default, minimal, alloc, and WASM-supported paths remain green where the touched crate exposes them; all tests run without network access.

# Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| A universal equivalence helper hides meaningful loss. | High | Define comparison projections per boundary and keep helpers test-only until real production reuse exists. |
| Generated values mostly fail validation. | Medium | Compose generators from valid typed primitives and retain explicit invalid-input tests separately. |
| Commutativity is asserted for precedence-sensitive merges. | High | Test the actual left-biased laws and include non-commutative counterexamples. |
| Property tests duplicate fixture tests without improving guarantees. | Medium | Keep fixtures for examples and wire shape; reserve properties for laws over bounded input families. |
| The link repair collides with CLI coverage work. | Medium | Serialize ownership of the link module and let both plans share the same focused regression suite. |
| Crosswalk work duplicates metadata alignment. | High | Treat the standards matrix and loss contract as prerequisites and extend their tests in place. |

# Related concepts

- [Standards field matrix](../assets/standards-field-matrix.md)
- [CLI test coverage phase 1](cli-coverage-phase-1.md)
- [Crate boundary architecture](../refactors/crate-boundary-architecture.md)
