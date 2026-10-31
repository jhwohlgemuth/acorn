---
type: Plan
title: "Category-based logbook presentations"
description: "Extend report generation to create date-bounded PDF and category-grouped PowerPoint artifacts from accepted RAD logbooks and project index data."
difficulty: medium
impact: critical
package: [acorn-cli, acorn-schema]
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
- `prepare_project` already filters `global` entries from per-project updates, while keeping them on the combined PDF title page. `updates_within` does not remove archived entries, so graduated entries remain eligible.
- The report command is also a publication workflow: per-project reporting publishes a PDF plus `ReportManifest` through GitLab and can enqueue callbacks. The current manifest has a single PDF checksum and the package API accepts an optional PDF payload. The `--combine` branch generates one local PDF and bypasses per-project publication/callback processing.
- With `--from` omitted, normal per-project PDF reporting can derive its lower bound from the previous GitLab manifest's `through` value, and can replay that manifest when its upper bound is unchanged. The manifest also records its original `from` value. Combined PDF does not read previous manifests.
- `crates/acorn-cli/src/commands/export/powerpoint/` creates a multi-slide presentation from one RAD input per slide, accepts a reference path through export options, adds slides from an OOXML template, and validates the package before publication. It currently does not compose logbook entries by category.
- The retained PowerPoint fixture has one slide template with named fields including `title`, `subtitle`, `achievement`, and a `notes` field on its notes slide. The category renderer maps project/window content to those fields, expands the update area, and rejects missing fields or content beyond its verified capacity.
- Latest baseline commit `678846941e63a3c06f62e240a4167f0b08e4d721` hardens `add_slide_to_presentation_xml` to reject malformed transformed XML, adds a regression test, and adds a bounded PowerPoint OOXML fuzz target covering slide-list mutation and other transforms. The shared renderer contract is now a usable dependency; category rendering should reuse these APIs and avoid rebuilding slide-list mutation logic.
- Report format parsing and dispatch now support PDF, PowerPoint, and both. PowerPoint-only runs remain local without GitLab publication; combined mode keeps one local PDF and project decks. Normal PDF publication and its callbacks remain unchanged, and a deck failure after PDF success does not enqueue a PDF failure event.
- `report/powerpoint.rs` composes category-ordered, newest-first slide inputs with project context, source IDs, and available authored image URL. The shared 1,200-character logbook summary limit applies per entry. The renderer reuses hardened OOXML slide insertion, image copying, interpolation, validation, and staged archive publication; it rejects overfull category content and preserves an earlier deck on failure. A six-slide fixture deck was opened and inspected in LibreOffice, including its dense Software slide.
- Focused report tests, malformed-template and preservation tests, `cargo clippy -p acorn-cli --all-targets --all-features -- -D warnings`, and a Chrome-backed PDF plus PPTX run in both normal and combined modes pass. REP-05 owns user documentation and final validation gates.
- SharePoint intake emits JSONL receipts; local intake provides stage/inspect/apply and explicit graduation. This plan starts from the accepted local project `index.json` snapshot after those operator-controlled steps. It does not alter PA flows, ingest JSONL, or graduate entries.
- The existing [database-backed artifact generation](database-backed-artifact-generation.md) plan owns direct current/revision database source selection. This capability remains file/index-backed and does not wait for database artifact selection.

# User-facing contract

- `acorn report` accepts repeatable/comma-delimited `--format pdf,powerpoint`; PDF remains the default when omitted. `powerpoint` is the canonical CLI spelling; documentation may mention `.pptx` as the artifact extension.
- A single invocation uses one project selection and the existing `--from`/`--through` parser. With an explicit `--from`, every selected format uses that same window. Without `--from`, normal PDF plus PPTX uses each project's previous manifest `through` as the lower bound for new work, or full history when no manifest exists. If the normal PDF path replays an unchanged-through manifest, use that manifest's recorded `from` and `through` for the deck. PowerPoint-only and combined runs use a full-history lower bound through the selected upper bound. The PDF path keeps its existing replay behavior.
- `--combine` affects PDF output only. With PDF selected, preserve today's combined PDF behavior. PowerPoint output is always one multi-slide `.pptx` per selected project, whether or not `--combine` is present. `--combine` must not merge projects into a cross-project deck or change PowerPoint file count.
- Preserve the existing GitLab publication contract: selected PDFs continue to use the current PDF manifest/package and callback flow. PPTX is a local artifact beside the per-project report output and is not added to GitLab packages or the PDF-only manifest in this plan. A PowerPoint-only run generates local decks without publishing PDF manifests or delivering report callbacks. If both are selected, PDF publication proceeds as today and PPTX remains local. Combined mode retains today's local combined-PDF behavior and additionally creates one local deck per project when PowerPoint is selected.
- Add an optional `--reference PATH`, matching `acorn export`, for the PowerPoint template. Resolve a relative path against each project's directory containing its `index.json`; an absolute path is used as given. When omitted, use `reference.pptx` beside each project's `index.json`, matching the export command's default. Validate the template before rendering; it affects PPTX only.
- Filter entries by project and the report's inclusive-lower/exclusive-upper reporting window before grouping. Include accepted entries regardless of whether they have been graduated; graduation archives source entries and must not make an update disappear. Keep general/global entries on the existing report title-page path; do not assign them to a project category slide without an explicit project.
- Create category slides only for categories that have selected entries. Use the schema's declared `LogbookCategory` order and the selected entries' existing newest-first order within each category. Preserve entry identifiers in speaker notes or a compact traceability footer so a slide's content can be traced to accepted source entries.
- If a project has no non-global entries in the selected window, do not create an empty deck for it; preserve the report's no-change result for that project.
- Populate each slide with category-specific update content plus relevant project context and available media from `index.json`. Use the project title, category/window label, formatted updates, and source IDs in the reference's title, subtitle, achievement, and notes fields respectively. Preserve other applicable project-context fields. REP-03 must verify rendered capacity and reject content that does not fit; do not truncate, split into extra category slides, or replace an earlier valid deck.
- Write `report.pptx` in the existing per-project `{through}-{commit}` output directory beside `report.pdf` when present. That path carries project, upper-bound, and commit identity; include the full selected window inside the deck. Do not put project decks in the combined PDF's root output location.

# Boundaries and decisions

- No Power Platform, SharePoint, GitLab, notification, JSONL receipt, or graduation behavior changes.
- No database selector, revision manifest, inbound MCP API, or direct remote artifact publication.
- No cross-project PowerPoint combination. `--combine` is specifically PDF-only, including when both formats are selected.
- Do not extend GitLab package contents, `ReportManifest`, or callback payloads to include PPTX. That publication contract can be planned separately if requested.
- A local PPTX failure must be reported for that project without emitting a PDF publication-failure callback after its PDF succeeded. Preserve independent project processing and the existing PDF callback result.
- Reuse the `ReportingWindow` contract and date parser; do not implement a second interpretation of dates.
- Reuse existing OOXML/reference-template and staged-write protections where practical. Keep presentation composition policy in the report/export layer, not in database persistence.
- Keep the current PDF-only invocation and its output behavior backward compatible.
- PowerPoint-only and combined local deck generation must not require GitLab pipeline/job identifiers or an online publication connection. Continue requiring the configured project registry and a commit identity for the per-project/date/commit output path; `--combine` without PowerPoint keeps its present requirements.

# Dependencies

- None. The current report command, report window contract, project index inputs, and PowerPoint template exporter are implemented.
- Coordinate shared report dispatch/template edits with grouped report separators and sponsor highlights work; these plans share report/HTML/export integration surfaces but are not behavioral prerequisites.
- Database-backed artifact generation is optional future source selection, not a prerequisite.

# Validation

- Document format selection, date bounds, project deck paths, template requirements, and PDF-only `--combine` and publication behavior.
- Run the repository's final lint, test, documentation, and portability gates after shared implementations are stable; recheck representative PDF and PPTX output.

# Acceptance criteria

- One `acorn report` invocation can produce PDF, PowerPoint, or both for the same projects and selected date range.
- Date filtering exactly matches the existing report window, including month shorthand, half-open boundary semantics, and the mode-specific omitted-`--from` rule above.
- Each selected project produces exactly one PPTX containing category slides for categories present in that project's selected entries.
- Projects with no selected non-global entries produce no PPTX.
- PPTX artifacts remain local; PDF package publication and manifests remain backward compatible and PDF-only.
- Each slide combines source logbook content with relevant `index.json` context and retains traceability to source entry IDs.
- Archived/graduated entries remain eligible when their timestamps fall in the selected window; general entries are not silently assigned to projects.
- An optional reference template is honored for PowerPoint and has no effect on PDF output.
- `--combine` affects only PDF: PDF behavior remains compatible, and PPTX count/content is invariant with the flag.
- Failures are reported with project/template context and do not replace a prior valid deck with a partial artifact.
- User-facing CLI help and report documentation describe formats, template selection, date bounds, output naming, and PDF-only combine semantics.

# Execution phases

## REP-05 — Document and close report presentation support

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `docs/src/cli-reference/commands/report.md` (new), `docs/src/cli-reference/commands/README.md`, `docs/src/SUMMARY.md`, `crates/acorn-cli/src/commands/report/tests/mod.rs`, `.wiki/features/logbook-category-presentations.md`, `.wiki/index.md`, `.wiki/features/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: report release documentation
- Outcome: Document date-range selection, format combinations, custom template behavior, output structure, and the PDF-only meaning of `--combine`; reconcile plan indexes and retire this plan after acceptance.
- Focused validation: Build the relevant book/docs target; run report and PowerPoint focused tests, then serialize and pass `make lint`, `make test`, and required portability gates after shared implementations are stable; inspect final PPTX output visually.
- Handoff: Preserve durable CLI and template guidance in user docs; remove completed plan references and its dependency/lane entries.
