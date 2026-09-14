---
type: Plan
title: "Fuzz OOXML slide-list transformations"
description: "Exercise PowerPoint XML mutation helpers against malformed and adversarial presentation input."
difficulty: low
impact: high
package: [acorn-lib, acorn-fuzz]
tags: [testing, fuzzing, export, powerpoint, ooxml]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/powerpoint/ooxml.rs`, its unit tests, `fuzz/Cargo.toml`, and CI fuzz-smoke configuration.
- **Create:** focused fuzz targets/corpora under `fuzz/` for slide-list and relationship transformations.
- **Delete:** obsolete corpus entries only when minimization preserves the triggering behavior.

# Dependencies

The parser-backed slide-list, relationship, content-type, and ASPECT transforms from [OOXML hardening](ooxml-interpolation-and-slide-copy.md) are ready for fuzzing now. Add interpolation targets after the remaining regex implementation is replaced. Preserve the future `acorn-export` ownership.

# Goal

Add focused fuzz coverage for the pure OOXML transformation boundary used when copying PowerPoint slides.

# Evidence

- `crates/acorn-lib/src/io/powerpoint/ooxml.rs` implements a custom XML event loop in `add_slide_to_presentation_xml`, plus relationship rewriting and well-formedness validation.
- The same module now has parser-backed transforms for PNG content types and ASPECT picture replacement. These helpers process externally supplied PPTX XML and must handle malformed nesting, namespace variants, duplicate slide lists, declarations, entities, truncated input, and large attributes without panicking or silently corrupting output.
- Unit tests reject malformed slide-list XML and missing relationship targets. CLI tests cover slide copying, missing source parts, image relationship rewriting, the embedded draft placeholder, missing authored media, and ASPECT image packaging.
- The focused PowerPoint test suites and end-to-end PPTX export test passed on 2026-09-13. These deterministic examples do not cover adversarial parser state or input growth.
- `fuzz/` currently covers Research Activity, Location, and Repository parsing, but has no OOXML target.

# Recommendation

1. Add an `ooxml_slide_list` cargo-fuzz target centered on `add_slide_to_presentation_xml`.
2. Derive the slide and relationship identifiers from the first eight input bytes and treat the remainder as UTF-8 XML; reject oversized inputs at 64 KiB.
3. Seed the corpus with a valid `presentation.xml` fixture and malformed variants that truncate or duplicate `p:sldIdLst`.
4. Assert that the function never panics, every successful result is well formed, and successful output contains the requested `id` and `r:id` values without losing surrounding XML.
5. Reapply the transform with different identifiers to verify that successful output remains well formed.
6. Add companion paths for `update_relationship_target`, `ensure_png_content_type`, and `replace_aspect_placeholders_with_picture`. Assert that successful mutations change only their intended elements, preserve unrelated XML, and remain well formed.
7. Add the parser-backed interpolation transform after OOXML hardening removes the regex implementation, using split-run and namespace variants in its seed corpus.

# Validation

Build each target, run it against the seeded corpus, and execute a bounded fuzz session. Replay every discovered artifact as a deterministic regression case before closing the plan. The focused unit and export tests are the baseline; they do not satisfy this plan without a cargo-fuzz target and bounded run.

# Acceptance criteria

1. OOXML fuzz targets run without panics or invariant violations across their seed corpora and bounded sessions.
2. Every successful transformation remains well formed, makes the requested change, and preserves unrelated XML content.
3. Every discovered artifact is minimized and retained as a deterministic regression case.

# Related plans

- [OOXML interpolation and slide-copy hardening](ooxml-interpolation-and-slide-copy.md)
