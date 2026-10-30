---
okf_version: "0.1"
type: Plan
title: Harden the shared export and OOXML pipeline
description: Fuzz the fallible OOXML transforms, inspect generated presentations, and close export hardening.
difficulty: high
impact: critical
package: [acorn-lib, acorn-cli, acorn-fuzz]
tags: [refactor, export, powerpoint, ooxml, interpolation, fuzzing]
status: in-progress
execution: phased
phase_budget_hours: 5
---

# Goal

Give HTML and PowerPoint export one immutable `ResearchActivity` projection and one replacement contract, then make every OOXML mutation fallible, parser-backed, staged, and fuzzed. Externally supplied templates must either produce a validated artifact or fail before a partial output is committed.

# Current evidence

- HTML and PowerPoint project once per activity through `ExportContext`, accept sectionless drafts, and use the same embedded 1920 x 1280 placeholder when no image was authored.
- An authored but unreadable image is already a hard PowerPoint error.
- HTML renders projected scalars, ordered lists, contact, identifier, and image choice. PowerPoint interpolation reads `ReplacementPlan`; CLI media copying and chart naming use the same context's image choice and identifier.
- Focused core tests pin contiguous scalar placeholders with whitespace, repetition, absent names, and empty values. PowerPoint tests pin ordered, XML-escaped bullets and reject unreadable authored media.
- `cargo test -p acorn-core --lib test_string_interpolation_replaces_contiguous_scalar_placeholders`, `cargo test -p acorn-lib --features powerpoint --lib test_replace_placeholder_with`, and the alloc-only `acorn-core` check pass.
- `acorn-core::util::interpolation` provides ordered raw `Absent`/`Scalar`/`List` values and literal scalar substitution. `io::export::ExportContext` projects the current presentation fields, raw lists, contact, identifier, and authored/fallback image choice without file access. Four focused tests for each module, the alloc-only core check, and focused library Clippy pass.
- HTML renders each list item as an escaped node and escapes projected scalar, contact, caption, and authored URL text before Percy serialization.
- PowerPoint interpolation now parses XML events, replaces scalar and list tokens across text runs, preserves unrelated run and paragraph events, and reports part-qualified read, parse, and write failures through `create`. The previously stale `technical` fixture token is replaced.
- Slide, relationship, content-type, and ASPECT transforms use `quick_xml`. Slide copies, image replacement, and ASPECT insertion prepare and validate dependent bytes before writing into the private extracted tree. Image lookup uses relationship type, and the shared ZIP writer stages final artifact publication.
- Twenty-one library PowerPoint tests, thirteen CLI PowerPoint helper tests, and the end-to-end `test_export_powerpoint_format` pass. Failure tests cover malformed XML and relationships, read/write errors, unchanged dependent parts, and preservation of an existing output after a late media failure.
- Existing fuzz targets cover schemas and repositories, not OOXML.

# Project code boundaries

- **Update:** `fuzz/Cargo.toml`, fuzz-smoke wiring, validation fixtures, and export documentation.
- **Create:** focused OOXML fuzz targets and seed corpora.

# Dependencies

The replacement contract and context are consumed by both renderers. EXP-08 fuzzing and EXP-09 visual inspection are the next independent phases before `acorn-export` extraction. Shared manifests, export dispatch, snapshots, and CI/build files remain integration locks.

# Decisions

- `ExportContext` projects raw scalar/list values and selects `Authored(String)` or `DraftFallback` from `ResearchActivity::authored_image_url`; it performs no filesystem access. The host renderer resolves authored media and rejects unreadable files before publishing an artifact.
- The projection carries the current presentation fields used by either renderer: title/subtitle, identifier, contact and affiliation, caption, mission/challenge/focus, approach/areas/impact/achievement, and the PowerPoint citation, notes, partners, managers, programs, portability, and technical values. It retains list items as data, never prejoined HTML or OOXML.
- Projection treats missing optional draft sections as empty values and rejects non-drafts without sections. Existing schema validation remains with the schema/read boundary; projection does not repeat unrelated validation rules.
- `ReplacementPlan` owns named `Absent`, `Scalar(String)`, and `List(Vec<String>)` values in deterministic name order. Its plain-text scalar substitution recognizes contiguous `{{ name }}` tokens with optional whitespace, replaces repeated tokens with literal values including `$`, clears explicit absence or empty scalars, and leaves unknown names and list tokens for the adapter. Lists retain source order; HTML/XML escaping and split-run parsing belong to renderers.
- HTML and PowerPoint adapters own their markup but consume the same replacement values and missing-value rules. Each activity is projected once at its renderer boundary; HTML takes branding, QR identity, and image choice from the context, while PowerPoint passes that context through interpolation and media copying.
- OOXML transforms use event parsing, recognize placeholders split across runs, preserve namespaces and unrelated run/paragraph properties, and return part-qualified errors.
- Slide, relationship, image, content-type, and ASPECT changes are staged and validated together before archive publication.
- Fuzz targets cap inputs, assert no panic, require every successful result to remain well formed and contain the requested change, and preserve unrelated XML.

# Implementation sequence

1. Add slide-list, relationship, content-type, ASPECT, and interpolation fuzz targets with bounded seed runs.
2. Inspect generated presentations in PowerPoint or LibreOffice and record the no-repair result.

# Validation

- Each fuzz target builds, runs a bounded session over seeds, and turns every discovered failure into a minimized deterministic regression.
- Run focused schema/core/export tests, the end-to-end PPTX test, no-default CLI tests, Clippy, lint, the full suite, and visual application inspection before closing the plan.

# Acceptance criteria

- Bounded fuzz runs find no panic or invariant violation, and all regression artifacts are retained as deterministic tests.
- Generated presentations open without a repair prompt, and the boundary is ready to move into `acorn-export`.

# Execution phases

## EXP-08 — Add bounded OOXML fuzz targets

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `fuzz/fuzz_targets/powerpoint.rs`, `fuzz/Cargo.toml`
- Primary integration lock: fuzz manifest
- Outcome: Cover interpolation, slide lists, relationships, content types, and ASPECT mutations with bounded invariants.
- Focused validation: Run bounded seed fuzzing and retain every regression input.
- Handoff: Remove EXP-08 and record seeds and commands for EXP-09.

## EXP-09 — Verify generated presentations visually

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
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
