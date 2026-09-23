---
type: Plan
title: "Enforce semantic transformation laws"
description: "Turn category-theory-inspired invariants into focused property and commuting-path tests for RAD normalization, linked-data projection, crosswalks, custom codecs and parsers, and merge behavior."
difficulty: high
impact: high
package: [acorn-core, acorn-schema, acorn-lib, acorn-cli]
tags: [testing, property-testing, research-activity, normalization, json-ld, crosswalk, codecs, identifiers, merge]
status: planned
---

# Goal

Make ACORN's existing transformations trustworthy under composition without adding mathematical vocabulary to the public API. Define the semantic meaning preserved at each boundary, encode the applicable identity, idempotence, round-trip, preservation, and commuting-path laws as tests, and require every intentional loss to remain explicit. Focus generative coverage on custom codecs and parsers rather than mechanically testing derived serialization.

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
- Existing property coverage is narrow but useful: ZON already round-trips generated supported JSON values, RRID tests canonicalization idempotence, core utilities test base32 round trips, and crosswalk tests cover selected reversible field rules. Extend these suites rather than duplicating them.
- `ResearchActivity` adapter tests exercise individual JSON, JSONC, YAML, ZON, and Markdown examples, while `OneOrMany<T>::parse` and `serialize` expose a shared JSON/YAML/ZON boundary without generated shape-preservation coverage.
- `crates/acorn-schema/src/hardware/memory.rs` implements custom string parsing and serialization, but its three codec tests are hidden behind `#[cfg(any())]` and never run. Serialization delegates to `f64` display while parsing accepts only a digit-and-period numeric prefix, so generated scientific-notation values are an important counterexample class.
- `Kitfile` has fixture-based authoring-YAML round trips and separate resolved-JSON semantics. Authoring output intentionally removes layer identity, while resolved JSON must preserve it.
- `OciReference`, `Remote`, SWHID, and Handle expose parse/display or Serde pairs with example tests but no generated canonical round-trip coverage. Their edge domains include ports, IPv6, nested repositories, tag and digest limits, Unicode suffixes, and ordered qualifiers.
- DataCite XML and CFF JSON/YAML have fixture round trips but no generated coverage over their supported typed subsets.
- `proptest` is already a dev dependency of `acorn-schema` and `acorn-lib`. The optional `Location` slice adds it as a dev-only dependency of `acorn-core`.

# Design decisions

## Keep categorical concepts out of the public surface

Do not add public `Category`, `Morphism`, `Functor`, `NaturalTransformation`, generic `Transform`, or `SemanticallyEquivalent` traits. The current traits already describe the operational boundaries contributors use. Add narrowly named production helpers only when a law exposes duplicated behavior that must be shared by two real consumers.

Do not add `RelationshipPath` as part of this plan. ACORN's current graph utility models organizational traversal, while RAD relationships are spread across typed identifiers, outputs, standards adapters, and linked-data projection. A path type needs a concrete query or reasoning consumer and a typed relation vocabulary; otherwise it would imply compositions the domain does not define.

## Define equivalence per boundary

There is no single universal equality relation for every ACORN representation. Use the smallest comparison appropriate to each boundary:

- **Canonical RAD equality:** typed `ResearchActivity` equality after normalization with the same filesystem context.
- **Linked-data equivalence:** canonical RAD fields are preserved after ignoring only derived JSON-LD decoration such as contexts and derived types.
- **Structured codec equivalence:** JSON, JSONC, YAML, and ZON decode to the same normalized typed value while preserving the `OneOrMany` wire shape; Markdown compares only the structural fields its documented format can express.
- **Custom scalar equality:** serialization followed by parsing recovers the same typed scalar for every value accepted by its public constructors and validation rules.
- **Canonical parser equality:** canonical values satisfy `parse(display(x)) = x`; supported aliases need only satisfy `canonicalize(canonicalize(s)) = canonicalize(s)`.
- **Kitfile equivalence:** resolved JSON preserves complete layer identity; authoring YAML compares the authoring projection because dropping derived digest and diff-ID fields is intentional.
- **Standards codec equivalence:** compare typed fields within the supported XML, JSON, or YAML subset and normalize documented wire defaults and omitted empty collections before comparison.
- **Crosswalk equivalence:** compare the exact and reversible fields identified by `.wiki/assets/standards-field-matrix.md`; approximated, simplified, omitted, or unsupported fields must produce the matching structured loss.
- **Merge equivalence:** compare the documented result and provenance/conflict behavior for that specific merge operation. Never substitute set equality for ordered fields unless the schema declares order irrelevant.

Start with crate-private or test-only projection helpers colocated with the law suites. Promote a reusable semantic projection into production only after at least two non-test consumers need the same operation.

Do not claim inverse laws for user-facing or intentionally lossy renderers. `Source::Display` includes labels, `Checksum::Display` omits the algorithm, and PDF, PowerPoint, and archive bytes may contain nondeterministic or unsupported detail. Test documented projections or invariants for those boundaries instead of byte or parse/display equality.

# Law catalog

| Boundary | Law | Required interpretation |
|---|---|---|
| Normalization | `N(N(x, c), c) = N(x, c)` | Formatting is idempotent for the same context `c`. Context-dependent relative media resolution is part of the input, not ignored noise. |
| Linked-data projection | `strip(L(N(x, c))) = N(x, c)` | Removing only derived JSON-LD decoration recovers the normalized canonical fields. |
| Projection determinism | `L(x) = L(x)` | Repeated projection produces byte-stable structured JSON after parsing; object-key ordering is not treated as semantics unless output stability is the explicit contract. |
| Workflow commutation | `link_command(x) ~= workflow(format + link, x)` | The direct link command and workflow route use the same linked-data projection and agree on the emitted document and output path. |
| Structured codecs | `decode_f(encode_f(N(x))) = N(x)` | Apply to JSON/JSONC/YAML/ZON within each format's supported shape, including scalar versus collection shape for `OneOrMany`. Markdown uses its documented structural subset instead of claiming full losslessness. |
| Custom scalar codecs | `decode(encode(x)) = x` | Apply to validated `Memory` values and any other manually implemented Serde pair included by a slice. Generated values include boundary magnitudes and alternate accepted spellings. |
| Canonical parsers | `parse(display(parse(s))) = parse(s)` | Apply to OCI references, SSH remotes, SWHIDs, Handles, and supported `Location` strings. Exact source-text equality is not required for aliases or reordered qualifiers. |
| Kitfile codecs | `decode_resolved(encode_resolved(x)) = x`; `decode_authoring(encode_authoring(x)) = authoring(x)` | Resolved JSON retains layer identity; authoring YAML intentionally strips it. |
| Standards wire codecs | `decode_f(encode_f(x)) ~= x` | Apply to bounded supported DataCite XML and CFF JSON/YAML records, using only documented wire-default normalization. |
| Reversible crosswalk fields | `back(forward(x)) ~= x` | Compare only fields classified exact or reversible in the standards matrix. |
| Loss accounting | `lossy(x) => warnings(x) != []` | Every omitted, approximated, simplified, or unsupported source value has a structured warning; target-required absence remains an error. |
| Output merge | `merge(x, x) = x`; `merge(x, empty) = x` | Preserve the existing left-biased fill-empty policy. Do not require commutativity. |
| Candidate merge | repeated identical evidence is unchanged | Replaying evidence does not duplicate values or provenance; populated conflicts keep the stored value and record the observation. Do not require commutativity because precedence is intentional. |
| PID normalization | supported spelling variants parse to one typed identity | Reuse PID-specific equality and checksum behavior; do not compare raw strings as semantic identity. |

`~=` means the boundary-specific equivalence defined above, not byte equality.

# Project code boundaries

- **Update:** focused tests for `Location`, research activity and `OneOrMany`, `Memory`, ModelKit, selected PID implementations and standards codecs, OCI and remote parsers, adapters, workflows, candidate persistence, and the link command; add `proptest` to `acorn-core` dev-dependencies only when the `Location` slice begins.
- **Create:** dedicated `laws.rs` test modules beside an existing test directory when splitting improves ownership; add a dedicated semantic-transform fixture directory under `tests/fixtures/data/` only for cases not expressible with current fixtures.
- **Delete:** None. Remove duplicated command-local transformation code only if the commuting-path test first proves it differs from the shared workflow projection and the focused implementation slice explicitly owns that repair.

Shared module indexes, crate manifests, `crates/acorn-cli/src/commands/link/mod.rs`, and shared fixtures are integration locks. An implementation slice must name the exact files it will own before editing them.

# Dependencies and coordination

- The normalization, codec, linked-data, and merge laws can start against current contracts.
- Scalar and parser laws should begin with types that already have custom implementations and concrete example tests: `Memory`, `OciReference`, `Remote`, SWHID, and Handle. Extend to other manually implemented codecs only after a failing counterexample or another high-value custom boundary is identified.
- Run cheap scalar and parser properties with the default 256-case budget. Bound structured RAD, Kitfile, and standards generators to 64 cases so the full suite remains practical while preserving minimized regressions.
- Crosswalk preservation and loss-accounting laws consume the implemented [standards field matrix](../assets/standards-field-matrix.md) and existing property tests instead of creating a competing crosswalk model.
- Command/workflow commutation coordinates with [CLI test coverage phase 1](cli-coverage-phase-1.md), which owns the first focused link-command suite. Only one slice edits `crates/acorn-cli/src/commands/link/mod.rs` or its test module at a time.
- This plan does not depend on schema versioning, a generic migration framework, crate extraction, or a new graph model.

# Implementation slices

## 0. Pin the law vocabulary and counterexamples

Write a table-driven test catalog beside the affected tests. For each law, record the operation, comparison projection, generated input domain, expected losses, and one minimal counterexample. Reuse existing valid-project, format, standards, and database fixtures before adding any new fixture.

The first counterexample should cover the separate direct-link and workflow-link paths. Capture current behavior without approving it as correct; the accepted contract is that both routes emit the `linked_content` representation.

## 1. Prove normalization and codec laws

- Add `format_with` idempotence properties for no context, a stable project context, relative media, canonical and aliased organization names, optional collections, and already formatted values.
- Add JSON, JSONC, YAML, and ZON round-trip properties over bounded valid `ResearchActivity` values.
- Add focused `OneOrMany<String>` properties proving that scalar, empty-list, singleton-list, and multiple-value shapes survive JSON, YAML, and ZON serialization through the shared `parse` and `serialize` methods.
- Keep generators validation-aware so failures represent transformation defects rather than arbitrary invalid schemas.
- Add a separate Markdown structural projection and enumerate unsupported/lossy fields; do not force full typed equality where the Markdown contract is intentionally narrower.

## 2. Prove custom scalar and canonical parser laws

- Replace the disabled `Memory` tests with generated JSON and direct parse round trips over finite non-negative amounts and every unit. Verify case-insensitive binary aliases converge on canonical units and invalid, negative, or non-finite inputs remain errors.
- Include scientific-notation magnitudes in the `Memory` generator. If the property exposes the current parser/serializer mismatch, repair parsing by removing a recognized unit suffix before parsing the complete numeric prefix; do not narrow the generated domain to hide a constructible value.
- Generate valid tag- and digest-based `OciReference` values across DNS names, registry ports, IPv6 authorities, nested repositories, and tag boundaries; require exact parse/display equality.
- Generate canonical SSH `Remote` endpoints with optional users, valid ports, and socket paths; require parse/display and JSON Serde equality, while testing bare-host aliases only for canonicalization idempotence.
- Extend the existing RRID property pattern to SWHID and Handle first: generate valid identifiers, qualifiers, ranges, Unicode suffixes, and accepted aliases, then prove canonicalization idempotence and Serde equality where implemented.
- Add `Location` parse/display canonicalization and Serde properties for supported URI schemes and portable paths. Compare revisions only through Serde because display intentionally omits them.

## 3. Prove Kitfile and standards wire-codec laws

- Generate valid authoring Kitfiles with bounded optional components, safe relative paths, and JSON-compatible parameter trees; compare the authoring projection after YAML encode/decode.
- Generate complete resolved Kitfiles with valid digests and diff IDs; require exact JSON encode/decode equality and retain fixture tests for upstream wire examples.
- Generate bounded supported DataCite records for XML and CFF records for JSON/YAML. Compare typed semantic projections after normalizing documented wire defaults and omitted empty collections.
- Keep format-specific fixture tests for exact wire spelling. Do not use snapshots as the sole proof of preservation.

## 4. Prove linked-data preservation and path commutation

- Parse linked JSON and verify that adding contexts/types does not change canonical RAD values.
- Verify projection determinism and that no absent domain assertion is fabricated.
- Add a focused comparison between the direct `link` command and `Workflow<ResearchActivity>` with `format=true`, `link=true`, `enrich=false`, `offline=true`.
- If the comparison exposes the current divergence, route the command through the shared linked-content/workflow seam in a separate, narrow repair commit, then keep the law as the regression test.

## 5. Extend crosswalk preservation and loss laws

- Build generators from `Fields` and typed standard records using the approved field matrix.
- For each exact or reversible mapping, verify forward-then-back preservation of the classified fields.
- For each lossy classification, assert the precise `ConversionWarning` kind and source/target path.
- Test composed routes only when both paths are supported contracts, such as a direct adapter versus the same adapter through the canonical `Fields` representation. Do not invent direct migrations solely to complete a diagram.

## 6. Prove merge-policy laws

- Add idempotence and empty-candidate identity tests for `ResearchOutput::merge`.
- Add replay, unique-array, stable-order, conflict-precedence, and provenance-deduplication properties for candidate persistence merging.
- Include negative tests documenting that left-biased merges are not generally commutative. These protect intentional precedence from a future "algebraic cleanup" that would change user-visible conflict behavior.

# Validation

Run focused tests per slice, then the repository gates appropriate to changed production code:

```sh
cargo nextest run -p acorn-schema --lib research_activity
cargo nextest run -p acorn-schema --lib hardware::memory
cargo nextest run -p acorn-schema --lib modelkit
cargo nextest run -p acorn-schema --lib pid
cargo nextest run -p acorn-schema --lib standard
cargo nextest run -p acorn-schema --lib standard::crosswalk
cargo nextest run -p acorn-lib --lib io::tests
cargo nextest run -p acorn-lib --lib io::adapter
cargo nextest run -p acorn-lib --lib io::workflow
cargo nextest run -p acorn-lib --lib io::database
cargo nextest run -p acorn-cli --lib commands::link
cargo nextest run -p acorn-core --lib location
cargo check -p acorn-core --no-default-features --features alloc
cargo check -p acorn-schema --no-default-features
make check-wasm
make lint
make test
```

All law tests are bounded and offline, persist minimized failing inputs through `proptest`, and reproduce saved regressions before exploring new cases. No snapshot alone is accepted as proof of semantic preservation.

# Acceptance criteria

- The law catalog names the exact comparison relation for every tested boundary; no test relies on an undefined global notion of semantic equality.
- Normalization is idempotent for the same context across generated valid RAD inputs.
- JSON, JSONC, YAML, and ZON round trips preserve normalized canonical RAD, including `OneOrMany` wire shape; Markdown preservation is limited to a documented structural projection.
- Valid `Memory` values round-trip through direct parsing and JSON for all units and generated finite non-negative magnitudes, including scientific-notation cases; aliases converge to canonical units.
- Generated OCI references and SSH remotes round-trip through their canonical displays; remotes also round-trip through JSON.
- Generated SWHIDs and Handles preserve typed identity through canonicalization, including valid qualifiers, ranges, aliases, and Unicode suffixes.
- Authoring Kitfile YAML preserves the authoring projection, while resolved Kitfile JSON preserves complete layer identity.
- Generated supported DataCite XML and CFF JSON/YAML records preserve their documented semantic fields.
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
| A custom serializer emits values its parser cannot read. | High | Generate from the public constructible domain, keep the minimized counterexample, and repair the codec instead of filtering it away. |
| Parse/display is asserted for a lossy renderer. | Medium | Inventory what display omits and use canonicalization or a documented projection rather than exact source-text equality. |
| Structured property suites make routine tests too slow. | Medium | Use 256 cases for cheap scalar laws, 64 cases for structured records, bounded collections, and focused crate filters. |
| Commutativity is asserted for precedence-sensitive merges. | High | Test the actual left-biased laws and include non-commutative counterexamples. |
| Property tests duplicate fixture tests without improving guarantees. | Medium | Keep fixtures for examples and wire shape; reserve properties for laws over bounded input families. |
| The link repair collides with CLI coverage work. | Medium | Serialize ownership of the link module and let both plans share the same focused regression suite. |
| Crosswalk work duplicates metadata alignment. | High | Treat the standards matrix and loss contract as prerequisites and extend their tests in place. |

# Related concepts

- [Standards field matrix](../assets/standards-field-matrix.md)
- [CLI test coverage phase 1](cli-coverage-phase-1.md)
- [Crate boundary architecture](../refactors/crate-boundary-architecture.md)
