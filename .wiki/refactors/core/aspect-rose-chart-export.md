---
type: Plan
title: "Add ASPECT rose charts to document exports"
description: "Render the four scalar ASPECT attributes once and embed the resulting chart image in PDF and PowerPoint exports."
difficulty: medium
impact: high
package: [acorn-cli, acorn-lib]
tags: [export, aspect, visualization, pdf, powerpoint]
status: completed
---

# Goal

Generate a rose chart for the four scalar ASPECT attributes and include the same rendered image in PDF and PowerPoint exports.

# Evidence

- `acorn-lib/src/schema/research_activity/aspect/mod.rs` defines autonomy, motivity, portability, and maturity as ordinal attributes.
- `acorn-cli/src/commands/export/pdf.rs` already owns a Chromium session for HTML-to-PDF rendering.
- `acorn-cli/src/commands/export/powerpoint.rs` currently copies one source image into each generated slide.
- `acorn-cli/src/template/mod.rs` already embeds generated image content as data URIs.
- The PDF feature already includes `resvg` transitively through QR-code generation.

# Recommendation

1. Map autonomy, motivity, and portability from their 0-5 scales and maturity from its 0-9 scale into normalized chart values.
2. Preserve missing attributes as unavailable rather than converting them to level zero.
3. Generate one native SVG rose chart per research activity without a web visualization runtime.
4. Embed the SVG in PDF HTML and rasterize the same SVG to PNG with `resvg` for PowerPoint.
5. Defer visualization of the heterogeneous ASPECT `data` collection.

# Validation

- Unit tests cover normalized values, independent label/score visibility, the 20% frame, missing attributes, and empty ASPECT records.
- A PDF export containing the vector SVG chart was rendered to PNG and visually inspected.
- The standalone native PNG was visually inspected.
- A PowerPoint export was opened and rendered by Microsoft PowerPoint and visually inspected.
- PowerPoint package tests verify the chart media part, relationship, and picture element.
- `cargo clippy --workspace --no-deps --all-targets`, the no-default-features check, and focused export tests pass.

# Acceptance criteria

- A research activity with at least one of autonomy, motivity, portability, or maturity produces an ASPECT rose chart.
- Each present attribute is normalized against its declared ordinal maximum.
- PDF and PowerPoint exports contain the same rendered chart.
- Missing attributes are visibly distinguished from level zero.
- Attribute labels and numeric scores can be hidden independently.
- The rose has a solid center and a gray frame extending 20% beyond the maximum data radius.
- ASPECT data descriptors do not affect the chart.

# Implementation

- `acorn-cli/src/commands/export/chart.rs` maps ASPECT values through a pure native SVG generator.
- The existing `resvg` dependency rasterizes that SVG to a 600×600 PNG without adding another graphics stack.
- A dedicated `chart` feature allows PowerPoint chart generation without enabling the Chromium-backed `pdf` feature; `pdf` enables `chart` automatically.
- PDF HTML embeds the SVG as an inset over the project graphic.
- PowerPoint export packages the rasterized PNG and adds a parser-backed OOXML picture element.
- Chart rendering requires neither Chromium nor JavaScript; Chromium remains responsible only for the overall HTML-to-PDF conversion.
- `--show-aspect-labels` and `--show-aspect-scores` enable chart text independently; both default to hidden.
- Colored sectors and white quadrant separators use a 100-unit radius while the uninterrupted gray frame uses 120 units, and every sector begins at the center.
- The heterogeneous ASPECT `data` collection remains intentionally out of scope.

# Related plans

- [Shared export context](export-context.md)
