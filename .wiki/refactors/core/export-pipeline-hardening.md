---
okf_version: "0.1"
type: Plan
title: Harden the shared export and OOXML pipeline
description: Project research activity data once, apply shared interpolation semantics, make PowerPoint mutation atomic and parser-backed, and fuzz the pure transforms.
difficulty: high
impact: critical
package: [acorn-core, acorn-schema, acorn-lib, acorn-cli, acorn-fuzz]
tags: [refactor, export, powerpoint, ooxml, interpolation, fuzzing]
status: in-progress
execution: phased
phase_budget_hours: 5
---

# Goal

Give HTML and PowerPoint export one immutable `ResearchActivity` projection and one replacement contract, then make every OOXML mutation fallible, parser-backed, staged, and fuzzed. Externally supplied templates must either produce a validated artifact or fail before a partial output is committed.

# Current evidence

- HTML and PowerPoint share `ResearchActivity::authored_image_url`, accept sectionless drafts, and use the same embedded 1920 x 1280 placeholder when no image was authored.
- An authored but unreadable image is already a hard PowerPoint error.
- Field extraction remains duplicated between CLI templates and `io::powerpoint::interpolate_values`; the new `ExportContext` has not yet been adopted by either renderer.
- Scalar interpolation uses `acorn_core::util::StringInterpolation`, while bullet interpolation uses regex paragraph discovery, reparses partial XML, and silently preserves stale content after several failures.
- Focused core tests now pin contiguous scalar placeholders with whitespace, repetition, absent names, and empty values. PowerPoint tests pin ordered, XML-escaped bullets and reject an unreadable fixture instead of silently passing.
- `cargo test -p acorn-core --lib test_string_interpolation_replaces_contiguous_scalar_placeholders`, `cargo test -p acorn-lib --features powerpoint --lib test_replace_placeholder_with`, and the alloc-only `acorn-core` check pass.
- Legacy PowerPoint scalar interpolation still treats replacement text as a regex substitution and does not handle placeholders split across OOXML runs; EXP-06 moves that adapter to the portable contract.
- `acorn-core::util::interpolation` provides ordered raw `Absent`/`Scalar`/`List` values and literal scalar substitution. `io::export::ExportContext` projects the current presentation fields, raw lists, contact, identifier, and authored/fallback image choice without file access. Four focused tests for each module, the alloc-only core check, and focused library Clippy pass.
- The HTML template joins list values into markup before rendering; PowerPoint constructs a separate list of scalar and bullet replacements. `ResearchActivity::authored_image_url` already selects the authored image, while PowerPoint resolves it relative to the source file and HTML uses its URL directly.
- Slide-list, relationship, content-type, and ASPECT mutations use `quick_xml` and have deterministic tests. Interpolation still returns `()` and logs read/write failures.
- Existing fuzz targets cover schemas and repositories, not OOXML.

# Project code boundaries

- **Update:** CLI templates, `crates/acorn-lib/src/io/powerpoint/{mod,ooxml}.rs`, PowerPoint command composition, dedicated tests/fixtures, `fuzz/Cargo.toml`, and fuzz-smoke wiring.
- **Create:** parser-backed interpolation transforms and focused OOXML fuzz targets/corpora.
- **Delete:** renderer-specific field extraction, regex-based OOXML selection, silent interpolation failure paths, and ad hoc mutations after parity tests.

# Dependencies

The replacement contract and context are available for renderer integration. Parser-backed transforms still precede `acorn-export` extraction. Fuzzing begins after the pure transform signatures stabilize. Shared manifests, export dispatch, templates, snapshots, and CI/build files are integration locks.

# Decisions

- `ExportContext` projects raw scalar/list values and selects `Authored(String)` or `DraftFallback` from `ResearchActivity::authored_image_url`; it performs no filesystem access. The host renderer resolves authored media and rejects unreadable files before publishing an artifact.
- The projection carries the current presentation fields used by either renderer: title/subtitle, identifier, contact and affiliation, caption, mission/challenge/focus, approach/areas/impact/achievement, and the PowerPoint citation, notes, partners, managers, programs, portability, and technical values. It retains list items as data, never prejoined HTML or OOXML.
- Projection treats missing optional draft sections as empty values and rejects non-drafts without sections. Existing schema validation remains with the schema/read boundary; projection does not repeat unrelated validation rules.
- `ReplacementPlan` owns named `Absent`, `Scalar(String)`, and `List(Vec<String>)` values in deterministic name order. Its plain-text scalar substitution recognizes contiguous `{{ name }}` tokens with optional whitespace, replaces repeated tokens with literal values including `$`, clears explicit absence or empty scalars, and leaves unknown names and list tokens for the adapter. Lists retain source order; HTML/XML escaping and split-run parsing belong to renderers.
- HTML and PowerPoint adapters own their markup but consume the same replacement values and missing-value rules.
- OOXML transforms use event parsing, recognize placeholders split across runs, preserve namespaces and unrelated run/paragraph properties, and return part-qualified errors.
- Slide, relationship, image, content-type, and ASPECT changes are staged and validated together before archive publication.
- Fuzz targets cap inputs, assert no panic, require every successful result to remain well formed and contain the requested change, and preserve unrelated XML.

# Implementation sequence

1. Migrate HTML and PowerPoint to the shared projection, then replace regex interpolation, return errors from all interpolation I/O, select image relationships by relationship type, and stage mutations atomically.
2. Add slide-list, relationship, content-type, ASPECT, and interpolation fuzz targets with bounded seed runs.
3. Inspect generated presentations in PowerPoint or LibreOffice and record the no-repair result.

# Validation

- Renderer parity tests cover populated records, sectionless drafts, authored/fallback images, unreadable authored files, and output escaping.
- OOXML tests cover split runs, namespace prefixes, malformed paragraphs, duplicate slide lists, missing relationships, unsupported layouts, and failures after staging.
- Each fuzz target builds, runs a bounded session over seeds, and turns every discovered failure into a minimized deterministic regression.
- Run focused schema/core/export tests, the end-to-end PPTX test, no-default CLI tests, Clippy, lint, the full suite, and visual application inspection.

# Acceptance criteria

- HTML and PowerPoint obtain every shared field and image decision from one immutable export context.
- Scalar and list replacements use one portable plan with explicit absence, empty, repetition, and literal-value semantics; adapters escape for their output formats.
- Placeholders split across PowerPoint runs are transformed without regex-based XML selection.
- Malformed templates and every read, parse, serialization, relationship, and write failure reach the caller with the affected part identified.
- Mutations are validated before commit and cannot leave a usable partial export.
- Sectionless drafts and authored/fallback media retain their documented behavior.
- Bounded fuzz runs find no panic or invariant violation, and all regression artifacts are retained as deterministic tests.
- Generated presentations open without a repair prompt, and the boundary is ready to move into `acorn-export`.

# Execution phases

## EXP-04 — Migrate HTML rendering to ExportContext

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-cli/src/template/mod.rs`, `crates/acorn-cli/src/template/tests/mod.rs`
- Primary integration lock: HTML export adapter
- Outcome: Render HTML exclusively from the shared context without changing authored/fallback image behavior.
- Focused validation: Run focused template and PDF export tests.
- Handoff: Remove EXP-04 and record renderer parity results for EXP-09.

## EXP-05 — Migrate PowerPoint projection to ExportContext

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-lib/src/io/powerpoint/mod.rs`, `crates/acorn-lib/src/io/powerpoint/tests/mod.rs`
- Primary integration lock: PowerPoint export adapter
- Outcome: Consume the shared context and match HTML missing-value and image semantics.
- Focused validation: Run focused PowerPoint tests against parity fixtures.
- Handoff: Remove EXP-05 and make parser-backed mutation work ready.

## EXP-06 — Replace regex interpolation with event parsing

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: EXP-05
- Exact write set: `crates/acorn-lib/src/io/powerpoint/interpolation.rs`, `crates/acorn-lib/src/io/powerpoint/mod.rs`
- Primary integration lock: OOXML interpolation
- Outcome: Transform scalar and list placeholders split across runs and return part-qualified errors.
- Focused validation: Run interpolation tests for namespaces, properties, malformed XML, and I/O failure.
- Handoff: Remove EXP-06 and pass fallible transforms to EXP-07.

## EXP-07 — Stage OOXML mutations atomically

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: EXP-06
- Exact write set: `crates/acorn-lib/src/io/powerpoint/mod.rs`
- Primary integration lock: OOXML archive mutation
- Outcome: Validate slide, relationship, media, content-type, and ASPECT changes before publication.
- Focused validation: Run failure-injection tests proving no usable partial export.
- Handoff: Remove EXP-07 and expose pure transforms to fuzzing.

## EXP-08 — Add bounded OOXML fuzz targets

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: EXP-06, EXP-07
- Exact write set: `fuzz/fuzz_targets/powerpoint.rs`, `fuzz/Cargo.toml`
- Primary integration lock: fuzz manifest
- Outcome: Cover interpolation, slide lists, relationships, content types, and ASPECT mutations with bounded invariants.
- Focused validation: Run bounded seed fuzzing and retain every regression input.
- Handoff: Remove EXP-08 and record seeds and commands for EXP-09.

## EXP-09 — Verify generated presentations visually

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: EXP-04, EXP-07
- Exact write set: `tests/fixtures/export/reproducible.json`, `.wiki/refactors/core/export-pipeline-hardening.md`
- Primary integration lock: renderer validation
- Outcome: Prove representative generated presentations open without repair and preserve expected layout.
- Focused validation: Inspect outputs in PowerPoint or LibreOffice and compare HTML/PDF/PPTX semantics.
- Handoff: Remove EXP-09 and record application versions and results.

## EXP-10 — Close export hardening

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: EXP-08, EXP-09
- Exact write set: `docs/src/packages/acorn-export.md`, `.wiki/index.md`, `.wiki/refactors/core/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/refactors/core/export-pipeline-hardening.md`
- Primary integration lock: export documentation
- Outcome: Document the shared contract, run final gates, and remove this completed plan.
- Focused validation: Run focused suites, portability checks, `make lint`, `make test`, `make doc`, and wiki validation.
- Handoff: Delete this plan and its index, network, and lane entries.
