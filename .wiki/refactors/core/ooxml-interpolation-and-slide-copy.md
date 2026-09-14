---
type: Plan
title: "Harden OOXML interpolation and slide-copy mutations"
description: "Finish parser-backed PowerPoint interpolation and make slide and media mutations fail cleanly."
difficulty: medium
impact: critical
package: [acorn-cli, acorn-lib]
tags: [refactor, export, powerpoint, ooxml]
status: in-progress
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/powerpoint/{mod,ooxml}.rs`, `crates/acorn-lib/src/io/powerpoint/tests/mod.rs`, `crates/acorn-cli/src/commands/export/powerpoint.rs`, `crates/acorn-cli/src/commands/export/powerpoint/tests/mod.rs`, dedicated fixtures, and later `acorn-export`.
- **Create:** parser-backed interpolation helpers and focused malformed, namespace, and split-run regression fixtures.
- **Delete:** remaining regex-based paragraph/bullet selection and ad hoc slide-copy mutations after parity tests.

# Dependencies

Consume [interpolation primitives](interpolation-primitives.md) for the remaining text and list work. The structured slide-list and relationship transforms are ready for [OOXML fuzzing](ooxml-fuzzing.md), while completion of this plan still precedes extraction of pure export transforms.

# Parallel execution

Parser-backed transformations and dedicated PowerPoint tests can run with portability work when confined to `io/powerpoint/` and the PowerPoint command subtree. Shared export dispatch, templates, snapshots, interpolation utilities, manifests, and module indexes remain integration locks.

# Goal

Finish the PowerPoint mutation boundary so externally supplied template XML is changed through fallible, parser-backed transforms and every failure reaches the export caller with the affected OOXML part identified.

# Current evidence

- `crates/acorn-lib/src/io/powerpoint/ooxml.rs` uses `quick_xml` event transforms for slide-list insertion, relationship target replacement, PNG content-type insertion, ASPECT placeholder replacement, and well-formedness checks. Its unit tests cover successful mutations and malformed or missing targets.
- `crates/acorn-cli/src/commands/export/powerpoint.rs::add_slide` validates copied slide and notes XML, updates both relationship directions, and propagates read, parse, serialization, and write failures.
- `copy_image` now rewrites the slide relationship and propagates a missing authored image as an export error. When no image was authored, it writes the embedded `draft-placeholder-1920x1280.png` asset instead. Dedicated command tests cover both paths.
- `create` now propagates `copy_image` failures instead of logging and continuing. The generated-export integration test covers the complete PPTX packaging path.
- `crates/acorn-lib/src/io/powerpoint/mod.rs::PowerpointInterpolation::replace_placeholder_with_bullets` still discovers paragraphs and placeholders with `fancy_regex`, silently preserves input after parse or serialization failures, and cannot reliably recognize placeholders split across arbitrary text runs.
- `interpolate_values` now maps absent draft sections to empty scalar and bullet values, but it still returns `()` and logs file read/write failures instead of returning them to the export caller.

# Remaining work

1. Replace regex paragraph discovery and placeholder matching with an event-based transform that recognizes placeholders split across text runs and preserves namespaces, run properties, paragraph properties, and unrelated content.
2. Make scalar and bullet interpolation use the shared missing-value semantics from [interpolation primitives](interpolation-primitives.md). Return `ApiResult` from `interpolate_values` and propagate read, parse, serialization, and write failures through `create`.
3. Select image relationships by OOXML relationship type instead of filename suffix. Stage and validate all changed XML before committing slide-copy or media mutations so a failed transform cannot leave a partially updated presentation tree.
4. Add focused tests for split-run placeholders, namespace prefixes, absent placeholders, malformed paragraphs, duplicate slide lists, unsupported relationship layouts, and failure after mutation staging.
5. Inspect a generated presentation in PowerPoint or LibreOffice and confirm that the application opens it without a repair prompt.

# Validation

Baseline validation on 2026-09-13:

- `cargo test -p acorn-cli --bin acorn commands::export::powerpoint::tests` passed 7 tests.
- `cargo test -p acorn-cli --bin acorn commands::export::tests::test_export_powerpoint_format` passed the end-to-end PPTX export test.
- `cargo clippy --workspace --no-deps --all-features --all-targets -- -D warnings` passed.

Close the plan only after the parser-specific cases pass, the generated archive has root-relative ZIP entries and valid relationship targets, and an office application opens the result without repair.

# Acceptance criteria

1. Scalar and bullet placeholders, including placeholders split across text runs, are transformed without regex-based XML selection.
2. Malformed or unsupported OOXML and every interpolation I/O failure reach the export caller with the affected part identified.
3. Slide, relationship, image, content-type, and ASPECT mutations are validated before they are committed and do not leave a usable partial export after failure.
4. Sectionless drafts render empty section placeholders and the embedded 1920 x 1280 image when media is absent; an explicitly authored but missing image remains an error.
5. Focused tests pass and generated presentations open without repair.

# Related plans

- [Shared interpolation primitives](interpolation-primitives.md)
- [OOXML slide-list fuzzing](ooxml-fuzzing.md)
- [Shared export context](export-context.md)
