---
type: Plan
title: "Introduce a shared export context"
description: "Complete one Research Activity projection for HTML and PowerPoint renderers."
difficulty: medium
impact: high
package: [acorn-schema, acorn-cli, acorn-lib]
tags: [refactor, export, html, powerpoint]
status: in-progress
---

# Project code boundaries

- **Update:** `crates/acorn-schema/src/research_activity/mod.rs`, `crates/acorn-cli/src/template/`, `crates/acorn-cli/src/commands/export/powerpoint/`, `crates/acorn-lib/src/io/powerpoint/`, renderer tests, and later `acorn-export`.
- **Create:** one immutable export context, typed image-source policy, and pure projection from validated schema values.
- **Delete:** renderer-specific repeated field extraction after all consumers use the context.

# Dependencies

The shared image-selection slice has landed independently. Complete scalar and list projection after [interpolation primitives](interpolation-primitives.md); the finished context precedes `acorn-export` extraction and coordinates with OOXML hardening.

# Goal

Convert validated Research Activity data once into an immutable renderer input with explicit semantics for absent draft sections, authored media, fallback media, and required values.

# Current evidence

- `ResearchActivity::authored_image_url` is now the common non-empty image selector used by HTML and PowerPoint exports.
- Both renderers accept sectionless drafts by mapping absent section fields and lists to empty values.
- Both renderers use the embedded `draft-placeholder-1920x1280.png` asset when no image was authored. HTML embeds it as a data URI and PowerPoint writes the same bytes into `ppt/media/` while updating the slide relationship.
- The CLI-owned image registry and draft-placeholder accessor are compiled independently of the PDF template feature, so PowerPoint fallback rendering remains available with default features disabled.
- An explicitly authored image remains strict: PowerPoint export returns an error if its source file is missing. The fallback applies only when no authored image URL exists.
- Field extraction remains duplicated between `crates/acorn-cli/src/template/mod.rs` and `crates/acorn-lib/src/io/powerpoint/mod.rs::interpolate_values`. The placeholder asset accessor is CLI-owned, and there is no `ExportContext` type or pre-render placeholder validation.

# Remaining work

1. Define a library-owned `ExportContext` with owned scalar/list values and a typed image source that distinguishes authored media from the embedded draft placeholder.
2. Project validated `ResearchActivity` values once. Preserve the current contract: absent sections are valid only for drafts, absent draft section values render empty, absent media selects the placeholder, and an authored but unreadable image is an error.
3. Make HTML and PowerPoint consume the context instead of separately cloning and defaulting schema fields.
4. Validate renderer-required values and template placeholders before any filesystem or archive mutation.
5. Move the embedded placeholder bytes behind the export boundary when that boundary moves to `acorn-export`.
6. Add parity tests that compare the semantic field and image selections used by HTML and PowerPoint.

# Validation

Baseline validation on 2026-09-13:

- `cargo test -p acorn-cli --bin acorn template::tests` passed 5 tests, including a sectionless draft with the embedded 1920 x 1280 image.
- `cargo test -p acorn-cli --bin acorn commands::export::powerpoint::tests` passed 7 tests, including fallback and missing-authored-media cases.
- `cargo test -p acorn-cli --bin acorn commands::export::tests::test_export_powerpoint_format` passed.

Feature-boundary validation on 2026-09-20:

- `cargo test -p acorn-cli --bin acorn --no-default-features commands::export::powerpoint::tests` passed 7 tests.
- `cargo test -p acorn-cli --bin acorn template::tests` passed 5 tests.
- `cargo test --lib --bins --tests --no-default-features` compiled every target and passed the complete 302-test CLI suite; the workspace run later stopped after 990 passing `acorn-lib` tests because the local PowerShell installation crashed in two unrelated shell-selection tests.

Close the plan with context projection tests and renderer parity tests for populated, sectionless, placeholder-image, and missing-authored-image inputs.

# Acceptance criteria

1. HTML and PowerPoint receive every shared field from one immutable export context.
2. The context records missing-value behavior explicitly: sectionless drafts produce empty optional content, while non-drafts without sections fail validation.
3. Both renderers select the same authored image or embedded 1920 x 1280 placeholder, and an authored image that cannot be read fails before an output is committed.
4. Required-value and placeholder validation completes before rendering or package mutation.
5. Renderer parity tests cover scalar, list, sectionless draft, authored-image, and placeholder-image cases.
6. The shared placeholder asset remains available to PowerPoint when CLI default features are disabled.

# Related plans

- [Shared interpolation primitives](interpolation-primitives.md)
- [OOXML interpolation and slide-copy hardening](ooxml-interpolation-and-slide-copy.md)
