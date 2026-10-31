---
type: Plan
title: "Category-based logbook presentations"
description: "Extend report generation to create date-bounded PDF and category-grouped PowerPoint artifacts from accepted RAD logbooks and project index data."
difficulty: medium
impact: critical
package: [acorn-cli, acorn-lib, acorn-schema]
tags: [feature, report, logbook, powerpoint, export, presentation]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Let operators use `acorn report` to generate PDF, PowerPoint, or both from the same selected projects and reporting window. The PowerPoint contains one multi-slide deck per project, with one slide per logbook category represented in that window. Each category slide combines accepted logbook updates with contextual fields and media from that project's `index.json` and uses an optional PowerPoint reference template.

# Current evidence

- `crates/acorn-cli/src/commands/report/mod.rs` resolves configured projects, reads each project's `index.json`, constructs a `ReportingWindow`, and calls `ResearchActivity::updates_within`. The lower bound is inclusive and the upper bound is exclusive. Report date parsing accepts `YYYY-MM-DD`, `DDMMM`, `MMM`, and RFC 3339 timestamps; month bounds resolve to calendar boundaries.
- `Report` in `crates/acorn-cli/src/cli/arguments/mod.rs` already owns `--from`, `--through`, `--project`, `--combine`, output, and template-independent report settings. The current renderer writes per-project PDFs and optionally one combined PDF.
- `LogbookEntry` provides category, timestamp, summary, optional supporting items, project ID, and archive status. `Logbook::entries_within` selects the same half-open reporting window newest first. `LogbookUpdates` and report rendering already support accepted updates.
- `crates/acorn-cli/src/commands/export/powerpoint/` creates a multi-slide presentation from one RAD input per slide, accepts a reference path through export options, adds slides from an OOXML template, and validates the package before publication. It currently does not compose logbook entries by category.
- SharePoint intake emits JSONL receipts; local intake provides stage/inspect/apply and explicit graduation. This plan starts from the accepted local project `index.json` snapshot after those operator-controlled steps. It does not alter PA flows, ingest JSONL, or graduate entries.
- The existing [database-backed artifact generation](database-backed-artifact-generation.md) plan owns direct current/revision database source selection. This capability remains file/index-backed and does not wait for database artifact selection.

# User-facing contract

- `acorn report` accepts repeatable/comma-delimited `--format pdf,powerpoint`; PDF remains the default when omitted. `powerpoint` is the canonical CLI spelling; documentation may mention `.pptx` as the artifact extension.
- A single invocation uses one project selection and the existing `--from`/`--through` interpretation for every selected format.
- `--combine` affects PDF output only. With PDF selected, preserve today's combined PDF behavior. PowerPoint output is always one multi-slide `.pptx` per selected project, whether or not `--combine` is present. `--combine` must not merge projects into a cross-project deck or change PowerPoint file count.
- Add an optional `--reference PATH`, matching `acorn export`, for the PowerPoint template. Resolve a relative template path against the report root, validate it before work begins, and use the existing report default when omitted. A supplied template affects PPTX only.
- Filter entries by project and the report's inclusive-lower/exclusive-upper reporting window before grouping. Include accepted entries regardless of whether they have been graduated; graduation archives source entries and must not make an update disappear. Keep general/global entries on the existing report title-page path; do not assign them to a project category slide without an explicit project.
- Create category slides only for categories that have selected entries. Use deterministic category order and deterministic entry order (newest first within category, with stable identifier tie-breaking). Preserve entry identifiers in speaker notes or a compact traceability footer so a slide's content can be traced to accepted source entries.
- Populate each slide with category-specific update content plus relevant project context from `index.json`. Avoid fabricating facts or silently overflowing a template; define bounded content and overflow behavior in REP-02 after inspecting available template layouts.
- Export into the existing report output tree and include project and date-window identity in artifact names/metadata, following current report directory conventions.

# Boundaries and decisions

- No Power Platform, SharePoint, GitLab, notification, JSONL receipt, or graduation behavior changes.
- No database selector, revision manifest, inbound MCP API, or direct remote artifact publication.
- No cross-project PowerPoint combination. `--combine` is specifically PDF-only, including when both formats are selected.
- Reuse the `ReportingWindow` contract and date parser; do not implement a second interpretation of dates.
- Reuse existing OOXML/reference-template and staged-write protections where practical. Keep presentation composition policy in the report/export layer, not in database persistence.
- Keep the current PDF-only invocation and its output behavior backward compatible.

# Dependencies

- None. The current report command, report window contract, project index inputs, and PowerPoint template exporter are implemented.
- Coordinate shared report dispatch/template edits with grouped report separators and sponsor highlights work; these plans share report/HTML/export integration surfaces but are not behavioral prerequisites.
- Database-backed artifact generation is optional future source selection, not a prerequisite.

# Validation

- Focused tests prove default PDF compatibility; PDF, PowerPoint, and both-format selection; invalid/duplicate formats; and that `--combine` changes only PDF output while PPTX remains one deck per project.
- Date-bound tests cover inclusive lower/exclusive upper timestamps, month shorthand, empty windows, and entries exactly on each boundary, using `ReportingWindow` behavior.
- Composition tests cover category grouping/order, multiple entries in a category, omission of empty categories, graduated and ungraduated entries, project-specific context, safe text interpolation, and source-ID traceability.
- Template tests cover omitted/default and explicit relative/absolute templates, malformed/missing templates, and no partial artifact replacement after failure.
- Generate a representative deck from a retained fixture; inspect slide count, OOXML package validity, rendered slide images, long text, graphics, and visual layout before accepting renderer work.
- Run focused report and PowerPoint tests in each phase. The final integration phase runs the repository's required full validation gates after all feature phases are stable.

# Acceptance criteria

- One `acorn report` invocation can produce PDF, PowerPoint, or both for the same projects and selected date range.
- Date filtering exactly matches the existing report window, including month shorthand and half-open boundary semantics.
- Each selected project produces exactly one PPTX containing category slides for categories present in that project's selected entries.
- Each slide combines source logbook content with relevant `index.json` context and retains traceability to source entry IDs.
- Archived/graduated entries remain eligible when their timestamps fall in the selected window; general entries are not silently assigned to projects.
- An optional reference template is honored for PowerPoint and has no effect on PDF output.
- `--combine` affects only PDF: PDF behavior remains compatible, and PPTX count/content is invariant with the flag.
- Failures are reported with project/template context and do not replace a prior valid deck with a partial artifact.
- User-facing CLI help and report documentation describe formats, template selection, date bounds, output naming, and PDF-only combine semantics.

# Execution phases

## REP-01 — Freeze report format and template CLI contract

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/commands/report/mod.rs`, `crates/acorn-cli/src/commands/report/tests.rs`, `CommandLineHelp.md`
- Primary integration lock: report CLI contract
- Outcome: Add validated PDF/PowerPoint format selection and optional reference-template argument while preserving existing defaults, paths, and PDF-only `--combine` semantics in normalized request dispatch.
- Focused validation: Parser and request tests for default PDF, each format, both formats, comma/repeated values, invalid values, template paths, and combine semantics; regenerate CLI help using the repository command.
- Handoff: Freeze a typed report request for renderer phases; record exact template path resolution and output naming behavior.

## REP-02 — Compose category-grouped project presentation data

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: REP-01
- Exact write set: `crates/acorn-cli/src/commands/report/powerpoint.rs`, `crates/acorn-cli/src/commands/report/tests/powerpoint.rs`, `crates/acorn-cli/src/commands/report/mod.rs`
- Primary integration lock: report presentation composition
- Outcome: Convert a project activity plus selected `LogbookUpdates` into deterministic category slide inputs containing project context, update summaries/items, timestamps, and source entry IDs. Specify bounded text and overflow behavior from the actual reference-template layout.
- Focused validation: Unit tests for half-open window input, category and entry ordering, multiple entries, graduated entries, empty categories, general-entry exclusion, escaping, traceability, and long-content handling.
- Handoff: Provide renderer-ready typed slide content independent of OOXML writing.

## REP-03 — Render one PowerPoint per project from a reference template

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: REP-02
- Exact write set: `crates/acorn-cli/src/commands/report/powerpoint.rs`, `crates/acorn-cli/src/commands/report/tests/powerpoint.rs`, `crates/acorn-cli/src/commands/export/powerpoint/`
- Primary integration lock: PowerPoint OOXML renderer
- Outcome: Render all category inputs for one project into one valid multi-slide PPTX, using the explicit reference or established default. Stage OOXML changes and destination writes so failures preserve an existing valid output.
- Focused validation: Fixture tests for generated slide count/content, malformed/missing template, package relationships/content types, preservation on late failure, and explicit template resolution; render and visually inspect representative output.
- Handoff: Expose a report-facing renderer that accepts typed project data, window identity, template, and destination.

## REP-04 — Integrate multi-format report artifacts and PDF-only combine

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: REP-01, REP-03
- Exact write set: `crates/acorn-cli/src/commands/report/mod.rs`, `crates/acorn-cli/src/commands/report/tests.rs`, `crates/acorn-cli/src/commands/report/powerpoint.rs`
- Primary integration lock: report artifact dispatch
- Outcome: Dispatch selected formats from the same resolved projects and reporting window. Keep combined PDF behavior scoped to PDF and generate one PPTX per selected project independently of `--combine`.
- Focused validation: End-to-end local fixture runs for PDF-only, PowerPoint-only, both, combine with each selection, multiple projects, no matching entries, and a custom template; assert output counts and report-window metadata.
- Handoff: Freeze artifact names and behavior for documentation and release validation.

## REP-05 — Document and close report presentation support

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: REP-04
- Exact write set: `docs/src/cli-reference/commands/report.md`, `docs/src/`, `crates/acorn-cli/src/commands/report/tests.rs`, `.wiki/index.md`, `.wiki/features/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: report release documentation
- Outcome: Document date-range selection, format combinations, custom template behavior, output structure, and the PDF-only meaning of `--combine`; reconcile plan indexes and retire this plan after acceptance.
- Focused validation: Build the relevant book/docs target; run report and PowerPoint focused tests, then serialize and pass `make lint`, `make test`, and required portability gates after shared implementations are stable; inspect final PPTX output visually.
- Handoff: Preserve durable CLI and template guidance in user docs; remove completed plan references and its dependency/lane entries.
