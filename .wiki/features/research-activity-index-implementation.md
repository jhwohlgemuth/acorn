---
type: Plan
title: Research Activity Index projection and navigation
description: Build a deterministic index of validated loaded activities and expose accessible catalog navigation.
difficulty: medium
impact: high
package: [acorn-web]
tags: [feature, research-activity, index, browser, accessibility]
status: planned
execution: phased
phase_budget_hours: 5
---

# Current evidence and boundary

`crates/acorn-web/src/session.rs` bounds files and derives validated activities, while `workspace.rs` renders a searchable table and `details.rs` displays record details. Accepted ABW-38 restores the full bounded workspace snapshot, including validations and source provenance; accepted ABW-39 supplies secure-context session credentials and accepted ABW-41 supplies authenticated GitHub loading. ABW-40 resolves the GitLab scope boundary before index work starts. The live ORNL Research Index was inspected on November 16 and supplies the catalog interaction target: search, result count, alphabetical/relevance sorting, list/card layouts, project links, and project detail sections for research focus, media, related links, challenge, approach, impact, and contact when the loaded RAD supplies them. The public [OKF Plan Browser source](https://github.com/jhwohlgemuth/okf-plan-browser) supplies implementation patterns without importing its planning-domain state or application shell.

# Acceptance criteria

- Stable project/activity identity and ordering across mixed validated activities, duplicate organizations and identifiers, authenticated replacement, and localStorage restoration.
- Search, result counts, alphabetical/relevance sorting, list/card layouts, project/activity navigation, source provenance, and detail sections follow the inspected Research Index interaction model where the RAD has corresponding data.
- Catalog and detail navigation expose equivalent text and controls for every visual relationship, including empty, missing-field, keyboard, and narrow states.
- Focused tests and browser evidence cover selection, filtering, relation links, restoration, and maximum bounded input.

# Execution phases

## ABW-42 — Define the Research Activity Index projection

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-40; accepted validated RAD, authenticated GitLab/GitHub loading, and bounded restored-workspace contracts frozen
- Exact write set: `crates/acorn-web/src/session.rs`, `crates/acorn-web/src/presentation.rs`, `crates/acorn-web/src/research_index.rs` (new, including focused unit tests), `crates/acorn-web/src/lib.rs`, `crates/acorn-web/README.md`, `.wiki/features/research-activity-index-implementation.md`
- Primary integration lock: validated activity projection
- Outcome: Derive deterministic project/activity rows, search text, sort keys, catalog summaries, optional detail sections, source provenance, and normalized organization/contributor/identifier relationships from accepted validated RAD without copying loaded files into another mutable state model. Invalid files remain visible only through existing validation state and never enter the index.
- Focused validation: `cargo test -p acorn-web --lib research_index` with mixed, duplicate, invalid, restored, replaced, missing-field, and maximum-bound fixtures; prove stable identity and alphabetical/relevance ordering.
- Handoff: Record projection identity and ordering contract; make ABW-43 eligible.

## ABW-43 — Expose index catalog and relationship navigation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ABW-42 projection accepted
- Exact write set: `crates/acorn-web/src/workspace.rs`, `crates/acorn-web/src/details.rs`, `crates/acorn-web/src/ui.rs`, `crates/acorn-web/assets/app.css`, `crates/acorn-web/scripts/parity-smoke.mjs`, `crates/acorn-web/README.md`, `.wiki/features/research-activity-index-implementation.md`, `.wiki/features/acorn-browser-workspace/_assets/acorn-browser-portfolio-preview.svg`
- Primary integration lock: Portfolio route and details drawer
- Outcome: Present the Research Index-inspired catalog with quick search, visible result count, alphabetical/relevance sorting, list/card switch, project/activity links, source provenance, and keyboard-accessible detail navigation through available research focus, media, related links, challenge, approach, impact, contact, organization, contributor, and identifier information. Preserve semantic equivalents, missing-data states, and list-first narrow-screen behavior.
- Focused validation: `make web-test` plus focused Chromium checks for search, counts, both sorts, both layouts, project/activity/detail links, keyboard focus, empty/missing-field states, restored data, and narrow screens; compare the updated Portfolio preview with the inspected reference behavior.
- Handoff: Preserve the accepted catalog/navigation and accessibility contract in tests/README; make ABW-19 eligible. Retire this child plan and the epic only after ABW-19 passes integrated acceptance.
