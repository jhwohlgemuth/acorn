---
type: Plan
title: "Consolidate interpolation primitives"
description: "Use one shared abstraction for string, list, and bullet placeholders."
difficulty: low
impact: medium
package: [acorn-core, acorn-lib, acorn-cli]
tags: [refactor, interpolation, export]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-core/src/util/`, interpolation helpers in `crates/acorn-lib/src/io/powerpoint/`, CLI templates, and focused tests.
- **Create:** pure scalar/list interpolation functions with explicit missing-value semantics.
- **Delete:** duplicate regex/string interpolation implementations after compatibility tests.

# Dependencies

This prepares completion of [export context](export-context.md), parser-backed OOXML interpolation, and the portable `acorn-core` boundary. The export context's shared image-selection slice does not depend on this plan.

# Current evidence

- Scalar PowerPoint placeholders use `acorn_core::util::StringInterpolation` and escape replacement text before insertion.
- Bullet placeholders use the PowerPoint-only `PowerpointInterpolation` trait, regex paragraph selection, partial `TextParagraph` deserialization, and string replacement.
- A serialization or paragraph-parse failure in bullet interpolation silently preserves the original XML. `interpolate_values` also logs file failures and returns `()`.
- The sectionless-draft change establishes the intended missing-value behavior for PowerPoint and HTML: absent section scalars and lists render as empty values.
- `crates/acorn-cli/src/template/mod.rs` still builds HTML strings and list fragments separately from PowerPoint interpolation.

# Remaining work

1. Specify one pure replacement input for scalar and ordered-list values, including explicit behavior for absent, empty, repeated, and escaped values.
2. Extend the portable `acorn-core` abstraction for format-neutral replacement planning without importing XML, HTML, filesystem, or renderer dependencies.
3. Apply that plan through parser-backed PowerPoint and structured HTML adapters. Keep format-specific paragraph, run, and list markup inside each adapter.
4. Return structured failures for malformed templates, unsupported placeholder layouts, and I/O errors instead of preserving stale placeholders silently.
5. Add shared contract tests and adapter tests for escaping, empty lists, absent section fields, repeated placeholders, and PowerPoint placeholders split across runs.

# Validation

The sectionless baseline passed `cargo test -p acorn-cli --bin acorn template::tests` and `cargo test -p acorn-cli --bin acorn commands::export::powerpoint::tests` on 2026-09-13. These tests establish current missing-value behavior but do not prove a shared interpolation implementation.

# Acceptance criteria

1. Scalar and list replacements share one pure, portable replacement-plan abstraction.
2. HTML and PowerPoint adapters use the same value and missing-value semantics while retaining format-specific markup handling.
3. Malformed or unsupported placeholder structures return errors; they do not silently leave stale content.
4. Contract and adapter tests cover escaping, absent and empty values, repeated placeholders, and split PowerPoint runs.

# Related plans

- [Shared export context](export-context.md)
- [OOXML interpolation and slide-copy hardening](ooxml-interpolation-and-slide-copy.md)
