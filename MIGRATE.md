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
| 2 | `acorn-cli/src/commands/export/mod.rs` | Remove Playwright init/launch/context/page boilerplate (~40 lines). Remove `Page` parameter from `pdf::create` call. Keep async call path for PDF export. |
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
- `pdf::create` stays `async` (`async fn create(options: CommandOptions)`), while removing the Playwright `Page` parameter
- `with_progress` closure remains async and awaits PDF creation (for example: `async move { pdf::create(pdf_options).await }`)

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

# Migration: uriparse → fluent-uri

## Goal
Replace `uriparse` (v0.6.4) with `fluent-uri` (v0.4) for all URI parsing, validation, and component access across the workspace.

## Why
- `uriparse` is unmaintained (last release Nov 2021)
- `fluent-uri` is RFC 3986/3987 compliant, actively maintained, and `no_std` compatible
- `fluent-uri` provides separate `Uri` (absolute) and `UriRef` (relative reference) types with better type safety
- Removes `fnv` transitive dependency (~30KB)
- Better IRI support for internationalized resource identifiers

## API Mapping

| uriparse | fluent-uri | Notes |
|---|---|---|
| `URI::try_from(s)` | `Uri::parse(s)` or `UriRef::parse(s)` | `UriRef` accepts relative refs; `Uri` requires scheme |
| `uri.scheme()` (enum) | `uri.scheme().as_str()` | `Scheme` is a struct; use `as_str()` for matching |
| `uriparse::Scheme::HTTPS` | match on `"https"` (lowercased) | Case-insensitive comparison via `as_str()` |
| `uri.host()` | `uri.authority().map(|a| a.host())` | No direct `.host()` on `Uri` |
| `auth.port() -> Option<u16>` | `auth.port_to_u16() -> Result<Option<u16>>` | `.ok().flatten()` to convert |
| `uri.path()` / `.query()` / `.fragment()` | same methods, return `&EStr<E>` | `EStr` implements `Display` / `PartialEq<&str>` |
| `URI<'a>` lifetime | `Uri<T>` generic over `&str` / `String` | `Uri<&str>` is `Copy` |

## Files to Modify

| # | File | Change Type |
|---|---|---|
| 1 | `Cargo.toml` (root) | Replace `uriparse = { version = "0.6.4", default-features = false }` with `fluent-uri = { version = "0.4", default-features = false, features = ["std"] }` |
| 2 | `acorn-lib/Cargo.toml` | `uriparse = { workspace = true }` → `fluent-uri = { workspace = true }` |
| 3 | `acorn-cli/Cargo.toml` | Same as #2 |
| 4 | `acorn-lib/src/lib.rs` | Import swap; rework `Location::scheme()` enum matching → `as_str()` matching; `Location::host()` → `.authority().map(|a| a.host())`; `Location::port()` → `.port_to_u16().ok().flatten()` |
| 5 | `acorn-lib/src/io/api/mod.rs` | Import swap; `From<URI<'a>>` → `From<Uri<&'a str>>`; use `auth.host()`, `.port_to_u16()` |
| 6 | `acorn-lib/src/schema/validate.rs` | Swap import to `UriRef`; `UriRef::parse` in `is_url()` |
| 7 | `acorn-lib/src/util/mod.rs` | Swap import to `UriRef`; `UriRef::parse` in `is_uri_or_path()` |
| 8 | `acorn-lib/src/io/tests/mod.rs` | Swap import alias; `Uri::parse` + `.scheme().as_str()` in test assertions |
| 9 | `Cargo.lock` | Auto-regenerated via `cargo update` |
| 10 | `fuzz/Cargo.lock` | Auto-regenerated |
| 11 | `acorn-py/Cargo.lock` | Auto-regenerated |

## Detailed Changes

### `acorn-lib/src/lib.rs` — 5 usage sites

**Location::scheme()** — Replace `uriparse::Scheme::HTTPS`/`HTTP`/`File` enum matching:
```rust
match uri.scheme().as_str().to_ascii_lowercase().as_str() {
    | "https" => Scheme::HTTPS,
    | "http" => Scheme::HTTP,
    | "file" => Scheme::File,
    | _ => Scheme::Unsupported,
}
```

**Location::uri()** — `parsed.authority().map(|auth| auth.as_str().to_string())` replaces `.map(|auth| auth.to_string())`.

**Location::host()** — `Uri::parse(value.as_str()).ok().and_then(|uri| uri.authority().map(|auth| auth.host().to_string()))`.

**Location::path()** — `Uri::parse(value.as_str()).ok().map(|uri| uri.path().to_string())` (unchanged shape; `EStr::to_string()` via `Display`).

**Location::port()** — `.and_then(|auth| auth.port_to_u16().ok()).flatten()`.

### `acorn-lib/src/io/api/mod.rs` — `From<Uri<&str>>` impl

```rust
impl<'a> From<Uri<&'a str>> for Endpoint {
    fn from(value: Uri<&'a str>) -> Self {
        let domain: String = value.authority()
            .map(|auth| format!("{}://{}", value.scheme().as_str(), auth.host()))
            .unwrap_or_default();
        let port: Option<u16> = value.authority()
            .and_then(|auth| auth.port_to_u16().ok())
            .flatten();
        Endpoint::at(domain).maybe_port(port).build()
    }
}
```

### `acorn-lib/src/schema/validate.rs` — `is_url()`

`URI::try_from(x.as_str())` → `UriRef::parse(x.as_str())` (accepts relative references, matching original behavior).

### `acorn-lib/src/util/mod.rs` — `is_uri_or_path()`

`URI::try_from(value).is_ok()` → `UriRef::parse(value).is_ok()` (same rationale).

### `acorn-lib/src/io/tests/mod.rs` — test assertions

`UriParse::try_from(value.as_str()).unwrap()` → `UriParse::parse(value.as_str()).unwrap()`
`assert_eq!(uri.scheme(), "HTTPS")` → `assert_eq!(uri.scheme().as_str(), "HTTPS")`

## Validation

1. `cargo check --workspace` — compilation succeeds
2. `cargo clippy --workspace --no-deps --all-targets` — no new warnings
3. `make test` — all tests pass (especially `test_location_uri` in `io::tests`)
4. `cargo check --no-default-features` — ensure `no_std` builds still work (fluent-uri supports it)
5. Verify lock files drop `uriparse` and `fnv` entries
