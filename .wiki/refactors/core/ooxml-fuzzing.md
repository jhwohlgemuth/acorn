---
type: Plan
title: "Fuzz OOXML slide-list transformations"
description: "Exercise PowerPoint XML mutation helpers against malformed and adversarial presentation input."
difficulty: low
impact: high
package: [acorn-lib, acorn-fuzz]
tags: [testing, fuzzing, export, powerpoint, ooxml]
status: planned
priority: next
---

# Goal

Add focused fuzz coverage for the pure OOXML transformation boundary used when copying PowerPoint slides.

# Evidence

- `crates/acorn-lib/src/io/powerpoint/ooxml.rs` implements a custom XML event loop in `add_slide_to_presentation_xml`, plus relationship rewriting and well-formedness validation.
- These helpers process externally supplied PPTX XML and must handle malformed nesting, namespaces, duplicate slide lists, declarations, entities, truncated input, and large attributes without panicking or silently corrupting output.
- `fuzz/` currently covers Research Activity, Location, and Repository parsing, but has no OOXML target.

# Recommendation

1. Add an `ooxml_slide_list` cargo-fuzz target centered on `add_slide_to_presentation_xml`.
2. Derive the slide and relationship identifiers from the first eight input bytes and treat the remainder as UTF-8 XML; reject oversized inputs at 64 KiB.
3. Seed the corpus with a valid `presentation.xml` fixture and malformed variants that truncate or duplicate `p:sldIdLst`.
4. Assert that the function never panics, every successful result is well formed, and successful output contains the requested `id` and `r:id` values without losing surrounding XML.
5. Reapply the transform with different identifiers to verify that successful output remains well formed.
6. Add `update_relationship_target` as a companion path once the slide-list target is stable, asserting that successful replacement changes only matching relationship targets and preserves valid XML.

# Validation

Build the target, run it against the seeded corpus, and execute a bounded fuzz session. Replay every discovered artifact as a regression case before closing the plan.

# Acceptance criteria

The OOXML fuzz target runs without panics or invariant violations across the seed corpus and a bounded fuzz session; successful transformations remain well formed, contain the requested identifiers, and preserve unrelated XML content.

# Related plans

- [OOXML interpolation and slide-copy hardening](ooxml-interpolation-and-slide-copy.md)
