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
---

# Goal

Give HTML and PowerPoint export one immutable `ResearchActivity` projection and one replacement contract, then make every OOXML mutation fallible, parser-backed, staged, and fuzzed. Externally supplied templates must either produce a validated artifact or fail before a partial output is committed.

# Current evidence

- HTML and PowerPoint share `ResearchActivity::authored_image_url`, accept sectionless drafts, and use the same embedded 1920 x 1280 placeholder when no image was authored.
- An authored but unreadable image is already a hard PowerPoint error.
- Field extraction remains duplicated between CLI templates and `io::powerpoint::interpolate_values`; there is no `ExportContext`.
- Scalar interpolation uses `acorn_core::util::StringInterpolation`, while bullet interpolation uses regex paragraph discovery, reparses partial XML, and silently preserves stale content after several failures.
- Slide-list, relationship, content-type, and ASPECT mutations use `quick_xml` and have deterministic tests. Interpolation still returns `()` and logs read/write failures.
- Existing fuzz targets cover schemas and repositories, not OOXML.

# Project code boundaries

- **Update:** portable interpolation utilities, research-activity export projection, CLI templates, `crates/acorn-lib/src/io/powerpoint/{mod,ooxml}.rs`, PowerPoint command composition, dedicated tests/fixtures, `fuzz/Cargo.toml`, and fuzz-smoke wiring.
- **Create:** one immutable export context, typed image-source policy, pure scalar/list replacement plan, parser-backed interpolation transforms, and focused OOXML fuzz targets/corpora.
- **Delete:** renderer-specific field extraction, regex-based OOXML selection, silent interpolation failure paths, and ad hoc mutations after parity tests.

# Dependencies

The replacement contract lands before renderer integration. The complete context and parser-backed transforms precede `acorn-export` extraction. Fuzzing begins after the pure transform signatures stabilize. Shared manifests, export dispatch, templates, snapshots, and CI/build files are integration locks.

# Decisions

- `ExportContext` owns scalar/list values and a typed image source distinguishing authored media from the embedded fallback. It contains no filesystem mutation.
- Projection enforces schema semantics once: sectionless drafts map optional section content to empty values; non-drafts missing required sections fail; unreadable authored media fails before rendering.
- A portable replacement plan represents scalar and ordered-list values, absence, emptiness, repetition, and escaping without importing HTML or XML.
- HTML and PowerPoint adapters own their markup but consume the same replacement values and missing-value rules.
- OOXML transforms use event parsing, recognize placeholders split across runs, preserve namespaces and unrelated run/paragraph properties, and return part-qualified errors.
- Slide, relationship, image, content-type, and ASPECT changes are staged and validated together before archive publication.
- Fuzz targets cap inputs, assert no panic, require every successful result to remain well formed and contain the requested change, and preserve unrelated XML.

# Implementation sequence

1. Land the pure scalar/list replacement plan and contract tests in `acorn-core`.
2. Add `ExportContext`, projection validation, typed image selection, and HTML/PowerPoint parity tests.
3. Replace regex interpolation, return errors from all interpolation I/O, select image relationships by relationship type, and stage mutations atomically.
4. Add slide-list, relationship, content-type, ASPECT, and interpolation fuzz targets with bounded seed runs.
5. Inspect generated presentations in PowerPoint or LibreOffice and record the no-repair result.

# Validation

- Contract tests cover escaping, absent and empty values, repeated placeholders, lists, and deterministic projection.
- Renderer parity tests cover populated records, sectionless drafts, authored images, fallback images, and missing authored files.
- OOXML tests cover split runs, namespace prefixes, malformed paragraphs, duplicate slide lists, missing relationships, unsupported layouts, and failures after staging.
- Each fuzz target builds, runs a bounded session over seeds, and turns every discovered failure into a minimized deterministic regression.
- Run focused schema/core/export tests, the end-to-end PPTX test, no-default CLI tests, Clippy, lint, the full suite, and visual application inspection.

# Acceptance criteria

- HTML and PowerPoint obtain every shared field and image decision from one immutable export context.
- Scalar and list replacements use one portable plan with explicit absence, empty, repetition, and escaping semantics.
- Placeholders split across PowerPoint runs are transformed without regex-based XML selection.
- Malformed templates and every read, parse, serialization, relationship, and write failure reach the caller with the affected part identified.
- Mutations are validated before commit and cannot leave a usable partial export.
- Sectionless drafts and authored/fallback media retain their documented behavior.
- Bounded fuzz runs find no panic or invariant violation, and all regression artifacts are retained as deterministic tests.
- Generated presentations open without a repair prompt, and the boundary is ready to move into `acorn-export`.
