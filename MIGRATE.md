# Migration: rust-playwright → ironpress

## Goal
Replace `rust-playwright` (headless Chromium) with `ironpress` (pure Rust HTML→PDF converter) for PDF export of Research Activity Data fact sheets.

## Why
- Eliminates ~300MB Chromium binary + system library dependencies
- Removes Node.js/npm from build pipeline
- Drops two `npx playwright install-deps` steps from Dockerfiles
- Simplifies PDF pipeline: 60 lines of browser orchestration → ~5 lines of library call
- Enables PDF tests in CI (ironpress has zero external dependencies)

## Status: In Progress (Font Blocker Resolved)

### Font Compatibility: Resolved
Legacy PostScript-flavored OTF fonts (`proxima_nova_*`, `myriad_pro_*`) were removed from the project.

The template now uses **TTF-only** Aptos assets:
- `aptos.ttf`
- `aptos_bold.ttf`
- `aptos_italic.ttf`

This aligns with Ironpress's TrueType-only custom font pipeline.

### Minor Issue: `position: fixed` not supported
Ironpress supports `static/relative/absolute` but not `fixed`. Since the fact sheet is single-page, `absolute` renders identically. Change in `project.css`:
- `header { position: fixed; ... }` → `position: absolute`
- `footer { position: fixed; ... }` → `position: absolute`
- `#contact { position: fixed; ... }` → `position: absolute`

### Open Question
1. Validate visual parity of Aptos-based PDF output against prior branded output.

## Files to Modify

| # | File | Change |
|---|------|--------|
| 1 | `acorn-cli/Cargo.toml` | Replace `playwright` dep with `ironpress = { version = "1.4", optional = true }`; change `pdf = ["dep:ironpress"]` |
| 2 | `acorn-cli/src/commands/export/mod.rs` | Remove Playwright init/launch/context/page boilerplate (~40 lines). Remove `Page` parameter from `pdf::create` call. Closure becomes sync. |
| 3 | `acorn-cli/src/commands/export/pdf.rs` | Rewrite: remove `Page` param, `goto_builder`, `set_content_builder`, temp file workaround. Use `HtmlConverter::new().page_size(PageSize::LETTER).convert(&content)`. Write PDF bytes via `acorn::prelude::write`. |
| 4 | `acorn-cli/src/template/mod.rs` | Keep `generate_font_faces()` and define Aptos TTF `@font-face` entries only (remove Myriad/Proxima OTF references). |
| 5 | `acorn-cli/assets/styles/project.css` | Use Aptos `font-family` values throughout. Change all `position: fixed` to `position: absolute`. |
| 6 | `acorn-cli/src/commands/export/tests/mod.rs` | Remove `#[ignore]` from PDF test. Add `%PDF` header assertion. |
| 7 | `Dockerfile.rust` | Remove Node.js installation and `npx playwright install-deps`. |
| 8 | `Dockerfile.bucket` | Remove Node.js installation and `npx playwright install-deps`. |
| 9 | `Cargo.lock` | Auto-updated by `cargo update`. |

## Prelude Imports

New `pdf.rs` uses `acorn::prelude::{create_dir_all, exit, write}` instead of `std::process::exit` and raw `std::fs`.

## Async/Sync Transition

- `export::run` stays `async` (outer progress machinery)
- `pdf::create` becomes sync (`fn create(options: CommandOptions)` instead of `async fn create(page: &Page, ...)`)
- `with_progress` closure changes from `async move { pdf::create(page, pdf_options).await }` to `{ pdf::create(pdf_options); Ok(()) }`

## Architecture Before/After

```
BEFORE: HTML → [Playwright init → install Chromium → launch browser
                → create context → new page → goto file:// → set content
                → pdf_builder() → close page → close browser] → PDF

AFTER:  HTML → [HtmlConverter::new().convert()] → PDF bytes → fs::write
```

## Testing

- `test_export_pdf_format` re-enabled in CI (no longer requires Playwright/Chromium)
- Verify PDF output visually matches current output (layout, colors, logos, positioning)
