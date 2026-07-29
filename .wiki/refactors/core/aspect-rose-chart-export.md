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
- Custom PowerPoint references do not necessarily declare PNG media in `[Content_Types].xml`.
- PowerPoint slide templates can place an ASPECT chart with a `{{ aspect }}` shape.

# Recommendation

1. Map autonomy, motivity, and portability from their 0-5 scales and maturity from its 0-9 scale into normalized chart values.
2. Preserve missing attributes as unavailable rather than converting them to level zero.
3. Generate one native SVG rose chart per research activity without a web visualization runtime.
4. Embed the SVG in PDF HTML and rasterize the same SVG to PNG with `resvg` for PowerPoint.
5. Defer visualization of the heterogeneous ASPECT `data` collection.

# Validation

- Unit tests cover normalized values, the shared ORNL palette, independent label/score visibility, the 20% frame, distinct zero and missing attributes, empty ASPECT records, placeholder replacement, and missing PNG content types.
- A PDF export containing the vector SVG chart was rendered to PNG and visually inspected.
- The standalone native PNG was visually inspected.
- A PowerPoint export was opened and rendered by Microsoft PowerPoint and visually inspected.
- PowerPoint package tests verify the chart media part, relationship, picture element, and preservation of every placeholder's bounds.
- `cargo clippy --workspace --no-deps --all-targets`, the no-default-features check, and focused export tests pass.

# Acceptance criteria

- A research activity with at least one of autonomy, motivity, portability, or maturity produces an ASPECT rose chart.
- Each present attribute is normalized against its declared ordinal maximum.
- PDF exports contain the rendered chart, and PowerPoint exports place it wherever the slide template contains `{{ aspect }}`.
- Missing attributes are visibly distinguished from level zero.
- Attribute labels and numeric scores can be hidden independently.
- The rose has a solid center and a gray frame extending 20% beyond the maximum data radius.
- PowerPoint exports register generated PNG chart parts in `[Content_Types].xml`.
- ASPECT sectors use the configured ORNL colors for autonomy, motivity, portability, and maturity.
- ASPECT data descriptors do not affect the chart.

# Implementation

- `acorn-lib/src/io/chart.rs` uses Bon-built chart options and attributes, then renders SVG or PNG bytes through one `ChartOptions::render` method.
- `ChartOptions::render_aspect_charts` reads and renders a batch of research activity files.
- `Vec<Attribute>` converts directly from `&AspectFramework`.
- The chart file-type builder accepts string inputs through `Into<ChartFileType>`.
- The existing `resvg` dependency rasterizes that SVG to a 600×600 PNG without adding another graphics stack.
- A dedicated `chart` feature allows PowerPoint chart generation without enabling the Chromium-backed `pdf` feature; `pdf` enables `chart` automatically.
- Shared `ORNL_COLOR_*` constants define the full supplied palette for chart rendering and the ORNL TUI theme.
- PDF HTML places the SVG immediately left of the 125×125-pixel QR code, renders the chart 5% larger at 131.25×131.25 pixels, and separates them by 36 pixels.
- Label-free charts trim the SVG view box to the 240-unit frame so the visible chart fills its dimensions.
- PowerPoint export packages the rasterized PNG and replaces each `{{ aspect }}` shape with a parser-backed OOXML picture at the shape's original bounds.
- PowerPoint picture placement converts from immutable slide events and owns its OOXML serialization method.
- Chart rendering requires neither Chromium nor JavaScript; Chromium remains responsible only for the overall HTML-to-PDF conversion.
- `--show-aspect-labels` and `--show-aspect-scores` enable chart text independently; both default to hidden.
- Colored sectors and white quadrant separators use a 100-unit radius while the uninterrupted gray frame uses 120 units, and every sector begins at the center.
- Present level-zero attributes use a small colored marker so they remain distinct from missing attributes when chart text is hidden.
- PowerPoint export adds an `image/png` content-type default when the reference package does not already provide one.
- The heterogeneous ASPECT `data` collection remains intentionally out of scope.

# Related plans

- [Shared export context](export-context.md)
