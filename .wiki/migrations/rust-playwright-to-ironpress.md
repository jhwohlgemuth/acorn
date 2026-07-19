---
type: Plan
title: "Migrate PDF export from rust-playwright to Ironpress"
description: "Replace headless Chromium PDF generation with an in-process Rust HTML-to-PDF converter."
difficulty: not-assessed
impact: not-assessed
package: [acorn-cli]
tags: [migration, pdf, export, dependencies]
status: in-progress
---

# Goal

Replace `rust-playwright` and headless Chromium with `ironpress` for Research Activity Data fact-sheet PDF export.

# Rationale

- Remove Chromium and its system-library installation from the PDF pipeline.
- Remove Node.js/npm setup used only to install Playwright dependencies.
- Replace browser-session orchestration with an in-process conversion call.
- Make PDF tests runnable without a separately installed browser.

# Current status

The font blocker is resolved. Legacy PostScript-flavored OTF assets were removed, and the template uses the TTF-only Aptos assets `aptos.ttf`, `aptos_bold.ttf`, and `aptos_italic.ttf`.

The repository still depends on Playwright, still installs its dependencies in both Dockerfiles, and still ignores the PDF export test. The stylesheet also retains three `position: fixed` declarations that must be evaluated and changed for the migration.

# Planned changes

| File | Change |
|---|---|
| `acorn-cli/Cargo.toml` | Replace the optional Playwright dependency with `ironpress = { version = "1.4", optional = true }` and update the `pdf` feature. |
| `acorn-cli/src/commands/export/mod.rs` | Remove browser initialization and the `Page` argument passed to PDF creation while keeping the async outer workflow. |
| `acorn-cli/src/commands/export/pdf.rs` | Convert generated HTML with `HtmlConverter`, select Letter page size, and write returned PDF bytes through the ACORN prelude. |
| `acorn-cli/src/template/mod.rs` | Retain the Aptos TTF-only font-face generation. |
| `acorn-cli/assets/styles/project.css` | Replace unsupported fixed positioning with absolute positioning after visual verification. |
| `acorn-cli/src/commands/export/tests/mod.rs` | Re-enable the PDF test and assert both file existence and a `%PDF` header. |
| `Dockerfile.rust` | Remove Node.js and Playwright installation steps used for PDF export. |
| `Dockerfile.bucket` | Remove Node.js and Playwright installation steps used for PDF export. |
| `Cargo.lock` | Regenerate dependency resolution. |

# Execution model

`export::run` and `pdf::create` remain async to fit the existing progress machinery, even though the core Ironpress conversion is synchronous. The progress closure continues to await PDF creation.

```text
Before: HTML -> Playwright -> Chromium -> browser context -> page -> PDF
After:  HTML -> HtmlConverter -> PDF bytes -> ACORN file write
```

# Validation

1. Re-enable `test_export_pdf_format` and assert the artifact begins with `%PDF`.
2. Generate a real fact sheet and compare layout, colors, logos, fonts, and positioning with the current output.
3. Confirm PDF-enabled builds and Docker images no longer install or reference Playwright, Chromium, Node.js, or npm solely for export.

# Acceptance criteria

PDF export produces a valid artifact without browser dependencies, and visual parity is explicitly accepted.

# Citations

[1] [Ironpress crate documentation](https://docs.rs/ironpress/latest/ironpress/) - Pure Rust conversion, builder API, Letter page size, PDF byte output, and TrueType font support.
