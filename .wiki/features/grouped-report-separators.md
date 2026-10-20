---
type: Plan
title: "Add full-page group separators to a combined activity report"
description: "Generate one ordered, multi-project PDF with full-page group dividers while preserving individual project reports."
difficulty: medium
impact: medium
package: [acorn-lib, acorn-cli]
tags: [feature, reporting, pdf, logbook]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Produce an optional combined activity report with a full-page divider before each nonempty project group. Keep the existing per-project PDFs, immutable packages, and callbacks unchanged.

# Current evidence

- The September 2026 reference uses white separator pages with a centered, large navy serif group title, followed by project pages with edge-to-edge teal banners and a green ORNL footer logo. Its groups are offices or directorates, not logbook statuses.
- `acorn report` currently iterates configured folders and writes one PDF and manifest per project. `LogbookUpdates` has an activity title and optional subtitle but no office or directorate field. The folder registry provides routing identity and path, not a report display group.
- The update PDF renders one project from HTML with Chrome. The test fixture has three projects but no report grouping metadata. Concatenating the published PDFs cannot express group order or a separator before each group without another composition step.

# Boundary and design

- Add optional `options.reportCollection` configuration containing ordered groups. Each group has a display title and ordered project IDs; IDs must resolve to configured folders and occur at most once. Do not infer group names from contacts or folder paths. A selected subset omits groups with no selected projects.
- Render the combined report as one HTML print job so dividers, project starts, footer placement, and page breaks share one layout contract. A divider consumes exactly one letter-size page; its title stays centered and wraps within a safe width. Each following project starts on a new page. Continuing project content may flow to further pages without another divider.
- Expose an explicit local combined-PDF output option. Individual project publication and replay semantics remain as they are; publishing a combined package would be a separate decision.

# Dependencies and acceptance

The first phase freezes ordered grouping in configuration. The renderer consumes that contract; CLI wiring follows the renderer. This plan owns the report composition seam and serializes with the shared export pipeline on `updates.rs` and PDF template code, and with CLI composition on parser/help files.

Acceptance requires deterministic group and project order, one divider per nonempty group, no blank trailing or repeated divider pages, no overlap with the ORNL footer, and clean page starts after long projects. Unknown or repeated project IDs fail before rendering. Existing per-project PDF and manifest behavior must remain stable. Verify with a fixture containing at least two groups, a long divider title, a project that spans pages, and a selected subset that leaves one group empty.

# Validation

Run focused config and report tests, render all fixture PDF pages to images, inspect the divider and continuation transitions, compare individual report manifests with the baseline, and run formatting plus `git diff --check`.

# Execution phases

## SEP-01 — Define ordered report groups

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-lib/src/io/config/application.rs`, `crates/acorn-lib/src/io/config/tests/mod.rs`, `tests/fixtures/report/config.json`
- Primary integration lock: API/config roots
- Outcome: Accept and validate optional `options.reportCollection` groups with display titles and unique, known project IDs; retain their declared order.
- Focused validation: Config deserialization and validation tests for valid order, unknown IDs, duplicates, and empty groups.
- Handoff: Record the frozen grouping type and ordering rules for SEP-02; remove this phase after its tests pass.

## SEP-02 — Render the grouped PDF

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SEP-01
- Exact write set: `crates/acorn-cli/src/commands/export/updates.rs`, `crates/acorn-cli/assets/styles/updates.css`, `crates/acorn-cli/src/commands/export/tests/mod.rs`, `tests/fixtures/report/config.json`
- Primary integration lock: export dispatch and PDF template
- Outcome: Compose one print document with full-page office dividers and project sections using the ordered groups, preserving the current project banner and status hierarchy.
- Focused validation: Render the multi-group fixture and inspect every page, including a wrapped divider title, long project continuation, and empty selected group.
- Handoff: Record the renderer entry point and visual evidence for SEP-03; remove this phase after rendering passes.

## SEP-03 — Add an opt-in local collection output

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SEP-02
- Exact write set: `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/cli/tests/mod.rs`, `crates/acorn-cli/src/commands/report/mod.rs`, `crates/acorn-cli/src/commands/report/tests.rs`, `CommandLineHelp.md`
- Primary integration lock: CLI contract
- Outcome: Add an explicit `--collection-output PATH` option that writes the combined PDF locally while retaining existing per-project packages, manifests, and callbacks.
- Focused validation: CLI parsing, dry-run fixture output, selected-subset behavior, and regression checks for individual project manifests.
- Handoff: Document the local output contract and pass renderer and CLI evidence to SEP-04; remove this phase after its focused checks pass.

## SEP-04 — Verify the combined report and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SEP-03
- Exact write set: `docs/src/cli-reference/commands/report.md`, `.wiki/features/grouped-report-separators.md`, `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: report acceptance
- Outcome: Publish the opt-in local output contract, confirm rendered-page and individual-report acceptance evidence, and retire this plan.
- Focused validation: Recheck the multi-group visual fixture, selected-subset output, individual PDF/manifest regression, links, and wiki validation.
- Handoff: Delete this plan and its index, network, and lane entries after the evidence is durable.
