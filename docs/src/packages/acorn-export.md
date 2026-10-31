# Export pipeline

ACORN's HTML and PowerPoint export renderers share one immutable `ResearchActivity` projection and one replacement contract, so both artifact families stay semantically identical. Every OOXML mutation is fallible, parser-backed, staged, and fuzzed: an externally supplied template either produces a validated artifact or fails before a partial output is committed. Command usage lives in the [`acorn export` CLI reference](../cli-reference/commands/export.md); this page covers the contract renderers implement.

## Projection

`ExportContext::project(&ResearchActivity)` (in `acorn-lib/src/io/export`) builds the renderer-independent projection without touching media or rendering markup. It carries the `contact`, the `identifier`, the `image_source`, and the `replacements` plan. A non-draft activity without sections is a projection error, while sectionless drafts are accepted. `ImageSource` is either `Authored(url)`, resolved by the host renderer, or `DraftFallback`, the embedded 1920 x 1280 placeholder used when no image was authored.

## Replacement plan

`acorn_core::util::interpolation` defines the values renderers consume:

| Type | Meaning |
|---|---|
| `ReplacementValue::Scalar(String)` | One raw text value |
| `ReplacementValue::List(Vec<String>)` | Raw values in presentation order |
| `ReplacementValue::Absent` | A known placeholder without content |

`ReplacementPlan` stores named values in a deterministic `BTreeMap`. Registered names cover titles, sections, contact, citation, aspect portability, notes, managers, programs, and partners; renderers own escaping so the plan stays markup-agnostic.

```rust
let rendered = plan.replace_scalars("<h1>{{ title }}</h1>");
```

Scalar substitution replaces contiguous `{{ name }}` placeholders with raw values, removes `Absent` placeholders, and leaves unknown names and list placeholders intact for the renderer to handle.

## Fallible OOXML transforms

The PowerPoint layer (`acorn-lib/src/io/powerpoint`) rewrites template parts as `quick_xml` events instead of string interpolation:

- `interpolate_xml` is paragraph-scoped, joins placeholders split across multiple `a:t` runs, fans a list placeholder out one paragraph per item, and rejects paragraphs that name two different lists.
- `add_slide_to_presentation_xml` appends an `<p:sldId>` entry and validates the `p:sldIdLst` end event, so unmatched tags error instead of returning malformed slides.
- `update_relationship_target` rewrites targets through the parsed relationship model and rejects an absent current target.
- `ensure_png_content_type` declares PNG media in `[Content_Types].xml` and is idempotent.
- `replace_aspect_placeholders_with_picture` swaps each `{{ aspect }}` shape for an ASPECT rose-chart picture at the same bounds and preserves unrelated shapes.
- `validate_xml_well_formed` revalidates every successful transform output.

Each function is a total `Result` with part-qualified errors citing the failed file.

## Staged publication

The `acorn export --format powerpoint` handler in `acorn-cli` extracts the reference template (a sibling `reference.pptx` by default, `-r` to override) into a private staging tree, then applies slide, media, relationship, content-type, and ASPECT chart mutations. New media is materialized under `ppt/media/` from the same context's image choice, and an authored but unreadable image is a hard error before publication. Every staged part is revalidated (`validate_ooxml_files`) before the finished package replaces the destination, leaving the previous output intact when a late media failure occurs.

## Fuzzing

`fuzz/fuzz_targets/powerpoint.rs` drives every transform with structured and raw input under `cargo-fuzz`, capped at 64 KiB per input. Run `make fuzz FUZZ_TARGET=powerpoint`, `make fuzz-repro FUZZ_TARGET=powerpoint FUZZ_CRASH=path`, or `make fuzz-smoke` for a bounded smoke pass over all targets. Between runs it asserts the invariants renderers rely on: successful mutations stay well formed, unrelated events and attributes are preserved, requested replacements are applied and removed, and the content-type transform is idempotent. `tests/fixtures/export/reproducible.json` is the deterministic end-to-end fixture whose generated presentations import without repair in LibreOffice, Chrome, and macOS QuickLook.
