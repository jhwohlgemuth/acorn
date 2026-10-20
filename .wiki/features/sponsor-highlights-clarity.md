---
type: Plan
title: "Make monthly sponsor highlights easier to interpret and reuse"
description: "Add sponsor-facing project synthesis and reviewed, evidence-backed ASPECT graduation to the monthly highlights workflow."
difficulty: mixed
impact: high
package: [acorn-cli, acorn-lib, acorn-schema]
tags: [feature, reporting, pdf, logbook, sponsor-highlights]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Help a sponsor reader answer what changed, why it matters to the mission, what can be reused by another project, what remains difficult, and what happens next. Preserve the time-coded logbook as the detailed evidence behind the monthly synthesis, and allow reviewed events to update canonical ASPECT capability attributes when their evidence supports the change.

# Current evidence

- Stakeholder feedback on the monthly sponsor highlights, supplied on 2026-10-18, values operational transitions, new data and tools, engagement, infrastructure planning, and evidence of partnership health. It requests quantified mission value, a consistent five-part project update, and broader approved access.
- The current report PDF has a general-updates title page, project title/subtitle and Principal Investigator/sponsor credits, then logbook entries grouped by completed, in-progress, and upcoming status. Each entry has a `summary` and optional supporting `items`; the report has no dedicated project-level mission-value, challenge, or cross-project-opportunity fields. See `crates/acorn-schema/src/research_activity/logbook.rs` and `crates/acorn-cli/src/commands/export/updates.rs`.
- The fictional `tests/fixtures/report/` projects exercise layout and logbook diversity, but do not test a repeatable sponsor-facing editorial structure. The existing [grouped separator plan](grouped-report-separators.md) owns office divider pages and is independent of this content change.
- `GraduationCandidates` currently proposes only classification and contributors. `ResearchActivity::graduate` accepts entry IDs, applies every candidate attached to each accepted entry, and archives the source. Database, MCP, and GitLab paths consume that contract. See `crates/acorn-schema/src/research_activity/logbook.rs`, `crates/acorn-lib/src/io/database/logbook.rs`, and `crates/acorn-lib/src/io/api/gitlab/automation.rs`.
- Canonical `aspect.maturity` is a `TechnologyReadinessLevel` (0–9); the six-level ASPECT `Maturity` label is derived for display. `aspect` also contains portability, motivity, autonomy, and data. An event category or completed state alone does not establish any of those values. See `crates/acorn-schema/src/research_activity/aspect/mod.rs`, `crates/acorn-schema/src/types.rs`, and `docs/src/concepts/aspect/maturity.md`.

# Recommendations, ordered by difficulty then impact

| Difficulty | Impact | Remaining improvement |
|---|---|---|
| Low | High | Require a concise, evidence-backed “Why it matters” statement and one relevant measure when available. Distinguish a demonstration, deployment, adoption, and measured mission outcome. |
| Low | High | Pilot the same five-part editorial structure for each project: description, summary update, key accomplishments, challenges, and next steps. Record “None reported” when a challenge is explicitly checked and absent. |
| Low | Medium | Put a portfolio synopsis on the first page, highlighting transitions, newly usable data or tools, common infrastructure needs, and opportunities for reuse. |
| Medium | High | Make cross-project opportunities explicit by naming the shared asset or approach, relevant projects, contact, and proposed next action. |
| Medium | Medium | Add a validated report-specific editorial input only after the template is accepted; keep it separate from individual logbook entries. |
| Medium | Medium | Check generated PDF headings, reading order, text alternatives, and contrast with assistive-technology-oriented validation. |
| High | High | Add structured ASPECT evidence to qualifying logbook events and make canonical capability changes individually reviewable during graduation. Start with maturity transitions; extend the same contract to other ASPECT attributes only with field-specific evidence rules. |
| High | High | Pilot wider distribution through an approved sponsor SharePoint location and editorial workflow, with audience, content, and preview text reviewed by the report owner first. |

# Boundary and decisions

- Keep `LogbookEntry` as an event/development record. Do not force every entry to repeat mission value, challenges, and next steps. A separate optional editorial input, bound to one reporting window and keyed by project ID, supplies the monthly synthesis. Runs without it retain the present PDF behavior.
- For projects with editorial input, require the five-part structure. Project description may reuse the existing subtitle, while the monthly summary and lists are editorial content. A measure must carry its meaning and evidence; omit unsupported numbers. Challenges may explicitly say “None reported,” but silence is not treated as that assertion.
- The first-page synopsis names cross-project opportunities without claiming a connection was acted on. Keep the detailed status-grouped entries beneath each project's concise synthesis. Do not duplicate full logbook narratives in the synthesis.
- ASPECT graduation uses an explicit, typed proposal on a source entry: target attribute, observed prior canonical value, proposed canonical value, and a concise outcome plus a reviewable evidence reference. Completed validation, prototype demonstration in a stated environment, operational integration, and successful mission operation may support different TRLs; a meeting, planned activity, or `Completed` label alone cannot promote maturity. Preserve the source entry and prior revision. Proposal generation is deterministic; application requires a human to accept a specific entry-and-target candidate. Preserve the existing entry-ID command for entries with one candidate, while requiring an unambiguous target selection when an entry has more than one.
- Compare a maturity proposal with the current canonical TRL before application. Reject unsupported, equal, lower, stale, or conflicting transitions; do not infer intermediate readiness from a numerical jump. Store the accepted `TechnologyReadinessLevel` in `aspect.maturity` and derive the ASPECT display label. For portability, motivity, autonomy, and data, define separate evidence and replacement/merge rules before enabling each target; these attributes are not interchangeable ordinal scores.
- Wider publication is an external decision. This plan does not authorize uploading the report or posting a news item. Report-owner approval, classification review, audience access, and editorial sign-off are prerequisites to the publication pilot.

# Dependencies and acceptance

HIL-01 establishes the authoring contract. HIL-02 defines the optional editorial data contract; HIL-03 wires it into report generation. HIL-04 and HIL-05 render project and portfolio views from the same data. HIL-06 audits the resulting PDF. HIL-07 requires a named publication owner and approved route, but does not block local authoring or rendering. HIL-08 establishes the ASPECT evidence contract; HIL-09 makes graduation target-specific; HIL-10 adds reviewed maturity transitions. HIL-11 through HIL-14 add portability, motivity, autonomy, and data targets one at a time after their field-specific evidence rules are approved. HIL-15 verifies database and MCP persistence; HIL-16 verifies the GitLab review path; HIL-17 surfaces only accepted capability changes in sponsor output. HIL-08 through HIL-16 can proceed independently of editorial-input work, but HIL-17 follows HIL-04 and HIL-16. HIL-03 through HIL-06 and HIL-17 serialize with [grouped report separators](grouped-report-separators.md) and the shared export pipeline on report renderer files. HIL-02 and HIL-09 through HIL-14 claim the research-activity schema lock; HIL-03 claims the CLI contract lock. HIL-16 serializes with GitLab intake work on shared automation files, without waiting for its live publication pilot.

Acceptance requires a complete synthetic monthly report whose project pages present the five parts in a stable order, identify mission relevance without inventing impact, show supported measures with context, and retain the underlying logbook entries and their statuses. Its first page must surface at least one concrete cross-project opportunity with a proposed next action. Missing or stale editorial input must fail clearly when explicitly supplied; absent optional input must preserve current output. A qualifying completed event must produce a reviewable ASPECT maturity candidate, and accepting only that target must update the canonical TRL once while retaining the event and prior revision. Nonqualifying, unsupported, stale, duplicate, and conflicting proposals must not change ASPECT; accepting one target must not implicitly accept another. The other ASPECT attributes require tested, field-specific evidence and update rules before they can graduate. Sponsor output must distinguish accepted capability changes from proposed changes. The PDF accessibility audit must pass or produce bounded repair phases before this plan retires. Wider distribution is complete only after approved audience and publication checks on the named sponsor systems; a denied route must be recorded as an explicit decision, not silently bypassed.

# Validation

Use focused schema and report-command tests, a dry-run over `tests/fixtures/report/`, rendered-page inspection, a comparison of existing per-project package/manifest behavior, and PDF structure checks. Include qualifying and nonqualifying event fixtures, mixed-target selection, current/proposed TRL conflicts, replay, and a successful mission-use case before allowing TRL 9. Confirm accepted ASPECT values survive database persistence and GitLab review, and that source entries remain available as evidence. Have a sponsor editor review one real monthly draft for scanability and defensible claims before treating the editorial template as accepted. Record publication approvals and access-test evidence separately from local PDF tests.

# Execution phases

## HIL-01 — Publish the editorial authoring contract

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `docs/src/cli-reference/commands/report.md`, `docs/src/SUMMARY.md`, `tests/fixtures/report/README.md`
- Primary integration lock: documentation generation
- Outcome: Document the five-part project template, mission-value sentence, measure/evidence rule, explicit challenge handling, and portfolio/cross-project summary pattern with fictional examples.
- Focused validation: Review examples against the current logbook schema and one sample PDF; prepare the guide for sponsor-editor review outside active phase time.
- Handoff: Record the proposed field labels and review request; remove this phase after the guide is complete.

## HIL-02 — Define validated monthly editorial input

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HIL-01 and sponsor-editor acceptance of its labels
- Exact write set: `crates/acorn-schema/src/research_activity/report_highlights.rs`, `crates/acorn-schema/src/research_activity/mod.rs`, `crates/acorn-schema/src/research_activity/tests/mod.rs`, `tests/fixtures/report/highlights.json`
- Primary integration lock: research-activity schema roots
- Outcome: Add an optional, reporting-window-bound input type keyed by project ID, with project synthesis, contextual measures, and explicit cross-project opportunities; validate missing, duplicate, stale-window, and malformed records.
- Focused validation: Focused `acorn-schema` tests for valid and invalid editorial input plus fixture deserialization.
- Handoff: Freeze the JSON shape and validation errors for HIL-03; remove this phase after its contract tests pass.

## HIL-03 — Load editorial input for a report run

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HIL-02
- Exact write set: `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/cli/tests/mod.rs`, `crates/acorn-cli/src/commands/report/mod.rs`, `crates/acorn-cli/src/commands/report/tests.rs`, `crates/acorn-cli/src/commands/export/updates.rs`, `CommandLineHelp.md`
- Primary integration lock: CLI contract
- Outcome: Add an explicit optional editorial-input path, validate its reporting window and selected project IDs before rendering, and pass the typed synthesis to the PDF renderer without changing runs that omit it.
- Focused validation: CLI parsing, valid/invalid dry runs, missing-project and stale-window errors, and unchanged legacy per-project manifest behavior.
- Handoff: Record the renderer input contract for HIL-04 and HIL-05; remove this phase after command tests pass.

## HIL-04 — Render the consistent project synopsis

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HIL-03
- Exact write set: `crates/acorn-cli/src/commands/export/updates.rs`, `crates/acorn-cli/assets/styles/updates.css`, `crates/acorn-cli/src/commands/export/tests/mod.rs`, `tests/fixtures/report/highlights.json`
- Primary integration lock: export dispatch and PDF template
- Outcome: Show the five project parts and “Why it matters” above status-grouped logbook detail, including contextual measures and explicit challenge state.
- Focused validation: Focused renderer tests and visual review of short, long, missing-optional, and multipage project cases.
- Handoff: Record the project layout contract and page evidence for HIL-05; remove this phase after project pages pass.

## HIL-05 — Render portfolio takeaways and connections

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HIL-04
- Exact write set: `crates/acorn-cli/src/commands/export/updates.rs`, `crates/acorn-cli/assets/styles/updates.css`, `crates/acorn-cli/src/commands/export/tests/mod.rs`, `tests/fixtures/report/highlights.json`
- Primary integration lock: export dispatch and PDF template
- Outcome: Add a scannable first-page synopsis and named cross-project opportunities with shared asset, contact, and next action; retain general updates below it.
- Focused validation: Focused title-page tests plus visual review with zero, one, and several opportunities and a long synopsis.
- Handoff: Record first-page visual evidence and release the renderer lock for HIL-06; remove this phase after checks pass.

## HIL-06 — Audit sponsor PDF accessibility

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HIL-04 and HIL-05
- Exact write set: `docs/src/cli-reference/commands/report.md`, `tests/fixtures/report/README.md`
- Primary integration lock: report validation evidence
- Outcome: Record repeatable checks for tagged headings, reading order, text alternatives, and contrast on the generated sample PDF. If a check fails, create a bounded repair phase with its own exact write set before closing the plan.
- Focused validation: Inspect generated PDF structure and reading sequence with an accessibility checker and a screen reader or equivalent accessibility-tree inspection; keep the findings reproducible.
- Handoff: Preserve pass evidence or the specific repair phase and update acceptance status.

## HIL-07 — Pilot approved wider distribution

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted editorial draft, report-owner authorization, classification and audience review, named sponsor SharePoint site owner, and sponsor editorial approval
- Exact write set: approved sponsor SharePoint report library and sponsor draft/publication workflow (external systems)
- Primary integration lock: sponsor publication ownership
- Outcome: Publish one approved issue and short discovery item to the agreed audience, or record the owner's decision to defer either route.
- Focused validation: Confirm intended-reader access, unintended-reader denial, accurate preview/headline text, and the live link; collect initial readership and cross-project inquiry feedback after the issue has circulated.
- Handoff: Record approval, access checks, and publication links or the explicit deferral decision; remove this phase when the external outcome is verified.

## HIL-08 — Define ASPECT event evidence and review rules

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `docs/src/concepts/aspect/README.md`, `docs/src/concepts/aspect/maturity.md`, `tests/fixtures/report/README.md`
- Primary integration lock: ASPECT authoring contract
- Outcome: Specify the typed proposal shape, including prior value, entry-and-target selection syntax, evidence/reference requirements, and field-specific acceptance rules. Distinguish TRL 6 relevant-environment demonstration, TRL 7 operational-environment demonstration, TRL 8 qualified system, and TRL 9 successful mission operation. Define portability evidence as an observed packaging/runtime capability; motivity and autonomy as observed behavior; and data graduation as an append-only, identified Real/Synthetic dataset record with no silent replacement. Each rule names cases that must not propose a change.
- Focused validation: Review the rules against current ASPECT types and representative completed, in-progress, unsupported, and conflicting fictional events; ensure each field has a decision the reviewer can verify.
- Handoff: Freeze the evidence and selection examples for HIL-09 through HIL-14; remove this phase after the contract is documented.

## HIL-09 — Select graduation targets independently

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HIL-08
- Exact write set: `crates/acorn-schema/src/research_activity/logbook.rs`, `crates/acorn-schema/src/research_activity/tests/mod.rs`
- Primary integration lock: research-activity schema roots
- Outcome: Give each candidate a stable entry-and-target selection key and record which targets on an entry have graduated. Keep entry-ID-only acceptance for an entry with exactly one candidate; reject ambiguous entry-ID-only acceptance and allow explicit selection of one target without archiving an entry that still has unapplied candidates. Previously accepted targets must not reappear as candidates.
- Focused validation: Focused schema tests for one target, two targets on one entry, duplicate selections, unknown targets, deterministic notes, and archived-entry behavior.
- Handoff: Record the accepted selection syntax and compatibility behavior for HIL-10 and adapter phases; remove this phase after tests pass.

## HIL-10 — Graduate evidence-backed maturity transitions

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HIL-09
- Exact write set: `crates/acorn-schema/src/research_activity/logbook.rs`, `crates/acorn-schema/src/research_activity/tests/mod.rs`, `tests/fixtures/report/aspect-graduation.json`
- Primary integration lock: research-activity schema roots
- Outcome: Add optional typed maturity evidence on a logbook event, generate a candidate only from a qualifying completed outcome, and apply an explicitly accepted transition to canonical `aspect.maturity` as a TRL. Keep unaccepted and previously archived source entries inspectable.
- Focused validation: Focused schema and fixture tests for TRL 6–9 evidence, missing ASPECT, unchanged/equal/lower/stale values, unsupported outcomes, repeated acceptance, and one accepted target among several.
- Handoff: Record the canonical serialized shape and a reviewed example for HIL-11 through HIL-17; remove this phase after tests pass.

## HIL-11 — Graduate observed portability capability

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HIL-08 and HIL-10
- Exact write set: `crates/acorn-schema/src/research_activity/logbook.rs`, `crates/acorn-schema/src/research_activity/tests/mod.rs`, `tests/fixtures/report/aspect-graduation.json`
- Primary integration lock: research-activity schema roots
- Outcome: Add a separate portability target that proposes the canonical `SoftwarePortability` value only from reviewed packaging or runtime evidence; do not infer portability from a software release alone.
- Focused validation: Focused tests for container/build, installer, and WebAssembly evidence, unsupported claims, current-value conflicts, and independent target acceptance.
- Handoff: Record tested proposal and rejection examples; remove this phase after tests pass.

## HIL-12 — Graduate observed motivity classification

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HIL-08 and HIL-11
- Exact write set: `crates/acorn-schema/src/research_activity/logbook.rs`, `crates/acorn-schema/src/research_activity/tests/mod.rs`, `tests/fixtures/report/aspect-graduation.json`
- Primary integration lock: research-activity schema roots
- Outcome: Add a separate motivity target based on observed input, output, and state behavior. Treat it as a reviewed classification change, not an automatic numerical promotion.
- Focused validation: Focused tests for reactive versus adaptive evidence, absent behavioral evidence, conflicting current values, and isolated acceptance.
- Handoff: Record tested proposal and rejection examples; remove this phase after tests pass.

## HIL-13 — Graduate observed autonomy classification

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HIL-08 and HIL-12
- Exact write set: `crates/acorn-schema/src/research_activity/logbook.rs`, `crates/acorn-schema/src/research_activity/tests/mod.rs`, `tests/fixtures/report/aspect-graduation.json`
- Primary integration lock: research-activity schema roots
- Outcome: Add a separate autonomy target based on demonstrated human and machine roles, including the actual approval/oversight boundary. Do not infer autonomy from the use of AI or automation terms.
- Focused validation: Focused tests for observed oversight roles, unsupported labels, conflicting current values, and isolated acceptance.
- Handoff: Record tested proposal and rejection examples; remove this phase after tests pass.

## HIL-14 — Graduate identified data evidence

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HIL-08 and HIL-13
- Exact write set: `crates/acorn-schema/src/research_activity/aspect/data.rs`, `crates/acorn-schema/src/research_activity/logbook.rs`, `crates/acorn-schema/src/research_activity/tests/mod.rs`, `tests/fixtures/report/aspect-graduation.json`
- Primary integration lock: research-activity schema roots
- Outcome: Add optional stable identity to `DataDescription` and a data target for an explicitly identified Real/Synthetic dataset, appending it to `aspect.data` only after review. Reject duplicate identities and conflicting descriptions; leave existing data untouched. Model artifacts remain outside this first data target.
- Focused validation: Focused tests for new dataset, duplicate/replay, same identity with conflicting description, missing evidence, and preservation of existing data.
- Handoff: Record the supported data subset and compatibility checks for HIL-15; remove this phase after tests pass.

## HIL-15 — Preserve ASPECT graduation in database and MCP paths

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HIL-10 and HIL-14
- Exact write set: `crates/acorn-lib/src/io/database/logbook.rs`, `crates/acorn-lib/src/io/database/tests/logbook.rs`, `crates/acorn-lib/src/io/mcp/tools/mod.rs`
- Primary integration lock: logbook persistence and MCP contract
- Outcome: Carry target-specific reviewed selections and ASPECT changes through durable database graduation and MCP apply/list responses while retaining the pre-graduation revision and source entry.
- Focused validation: Focused persistence and MCP tests for a maturity change, one non-maturity target, unchanged unselected targets, revision history, and replay.
- Handoff: Record the stable adapter contract and persistence evidence for HIL-16; remove this phase after tests pass.

## HIL-16 — Preserve ASPECT graduation in GitLab review

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HIL-15
- Exact write set: `crates/acorn-lib/src/io/api/gitlab/automation.rs`, `crates/acorn-lib/src/io/api/gitlab/tests/bot.rs`, `crates/acorn-lib/src/io/api/gitlab/tests/jsonl.rs`
- Primary integration lock: GitLab logbook automation
- Outcome: Show entry, target, proposed canonical value, and evidence in review notes; apply only selected targets through the authorized `/acorn graduate` path and publish a diff that exposes the ASPECT change and retained source with its per-target graduation state.
- Focused validation: Focused bot/automation tests for one accepted maturity target on a multi-target entry, rejected ambiguity, unsupported or stale proposal, and replay without duplicate effects.
- Handoff: Record the reviewed note/diff contract for HIL-17; remove this phase after tests pass.

## HIL-17 — Show verified capability transitions in sponsor highlights

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HIL-04 and HIL-16
- Exact write set: `crates/acorn-cli/src/commands/export/updates.rs`, `crates/acorn-cli/assets/styles/updates.css`, `crates/acorn-cli/src/commands/export/tests/mod.rs`, `tests/fixtures/report/aspect-graduation.json`
- Primary integration lock: export dispatch and PDF template
- Outcome: Add a concise capability-transition line to the relevant project synopsis when an accepted ASPECT graduation has evidence in the reporting window. Name the target and prior/new value with a source-entry reference; do not present a pending proposal as achieved impact.
- Focused validation: Focused renderer tests and visual review for accepted maturity, another accepted ASPECT target, pending/rejected evidence, and a project with no capability transition.
- Handoff: Record rendered-page evidence and release the renderer lock; pass acceptance evidence to HIL-18 and remove this phase after checks pass.

## HIL-18 — Reconcile sponsor evidence and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HIL-06, HIL-07, HIL-17
- Exact write set: `docs/src/cli-reference/commands/report.md`, `.wiki/features/sponsor-highlights-clarity.md`, `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: sponsor report acceptance
- Outcome: Preserve editorial, ASPECT graduation, accessibility, and approved publication or deferral evidence in durable guidance; retire this plan.
- Focused validation: Review the synthetic monthly report, accepted target-specific graduation evidence, PDF accessibility result, publication decision, links, and wiki validation.
- Handoff: Delete this plan and its index, network, and lane entries after acceptance evidence is durable.
