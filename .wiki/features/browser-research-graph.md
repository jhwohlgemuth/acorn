---
type: Plan
title: Research activity graph in the ACORN browser
description: Add an accessible, interactive Graph page over validated RAD connections with a portable linked-data projection and Turtle export.
difficulty: high
impact: high
package: [acorn-web, acorn-schema]
tags: [feature, browser, research-activity, graph, rdf, turtle, accessibility]
status: planned
execution: phased
phase_budget_hours: 5
---

# Outcome

People can open `/graph` beside Portfolio, explore how loaded research activities connect to projects, organizations, contributors, identifiers, and explicitly related activities, and inspect the RAD source behind every connection. The graph is a view of the same validated, bounded workspace as Portfolio. A downloadable Turtle representation expresses the supported relations using documented RDF terms; the visualization and RDF output consume one deterministic graph projection.

The [OKF Plan Browser Graph](https://www.jasonwohlgemuth.com/okf-plan-browser/graph) is the interaction reference: a searchable network, typed/color-coded nodes and edges, filters, focus/neighbor exploration, selected-node details, fit/zoom/reset, and an equivalent navigable list. ACORN uses research relationships and its existing shell, theme, and data permissions. The public route's HTML and bundled GraphPage source were inspected on 2026-11-25; its live rendered page could not be observed in the available browser host. Recheck responsive visual details before UI acceptance, without making them a semantic dependency.

# Current evidence and decisions

- At `041577d0` plus the 2026-11-25 worktree, `crates/acorn-web/src/research_index.rs:16-61,72-150,186-268` derives activities and projects from `Workspace::activities()` and already normalizes identifiers and explicit `related`/`relatedResources` references. Its `related()` resolves only loaded activities. `session.rs:34-49` holds supplied files, validation results, selection, and source provenance. The working tree has concurrent browser and wiki edits; phase owners must recheck the baseline.
- `crates/acorn-web/src/ui.rs:717-728,895-975` routes generic page slugs, and the sidebar already lists Portfolio and Geospatial. There is no Graph page. `crates/acorn-schema/src/standard/rdf/mod.rs:1-110` contains RDF syntax tooling for Dublin Core; that is not a RAD graph export contract. [Browser format parity](browser-format-parity.md#fmt-07b--add-proven-rdf-dublin-core-formatting) separately owns Dublin Core RDF formatting.
- The graph projection uses validated RAD only. Stable IDs are source-qualified for activity records and normalized explicit identifiers for shared entities. An untyped same-name string is not enough to merge people or organizations across records; when no explicit identifier exists, scope the entity to its source activity. Preserve record paths and the fields that justify each edge. Missing related targets remain inspectable references, visibly distinguished from loaded activities; do not invent a loaded RAD or fetch a remote target.
- Canonical edge kinds are `project contains activity`, `activity involves organization`, `activity credits contributor`, `activity has identifier`, and `activity explicitly relates to activity/reference`. Direction and labels remain visible. Keywords can filter and appear in details, but do not become entity nodes in the first release. This keeps the initial graph legible and avoids treating shared terms as evidence of collaboration.
- The portable graph model and identity/relationship projection are implemented in `acorn-schema` from typed RAD records. The browser must adapt its existing validated workspace/index to that model without a second mutable graph store. Turtle serializes the same model through the [RAD graph profile](browser-research-graph-profile.md), using stable IRIs, RDF-safe escaping, and explicit predicates; source/file and edge-field provenance remain in the graph model and are omitted from Turtle v1. It never serializes layout coordinates, search state, or inferred collaboration. Do not reuse the Dublin Core document codec as if it defined RAD semantics. No remote context lookup, SPARQL endpoint, graph database, or network write is needed.
- `crates/acorn-schema/src/research_activity/graph.rs` projects activity, project, contributor, organization, identifier, and unresolved reference nodes with authored, source-backed edges; `graph_rdf.rs` emits deterministic Turtle from those nodes and edges. The browser `/graph` route now derives that projection from validated `Workspace::activities()`, displays a bounded SVG and semantic lists, and exports the full graph as Turtle. Schema graph tests cover identity, relations, parsing, and escaping. Browser Graph tests and root plus `/acorn/` smoke cover sample, empty, invalid, oversized, and deleted workspaces, URL restoration, keyboard selection, source navigation, filters, export scope, reduced motion, and narrow layout. `make web-check`, final focused host/WASM Clippy, and both final release builds passed on 2026-11-25; the oversized graph response measured 69 ms at root and 87 ms at `/acorn/` in the final local browser smoke. Combined-source acceptance still waits for ABW-63–65.
- Initial rendering must be bounded by an explicit visible node/edge budget, with a count and a prompt to narrow filters when exceeded. Retain the full bounded projection for search and list navigation. The diagram must work without WebGL, honor reduced motion, and offer keyboard selection plus a semantic relationship list. Persist only URL-safe filters/selection; avoid storing duplicate RAD content or sensitive labels in a shareable URL.

# Dependencies and boundaries

- The initial local/single-bucket graph can follow the accepted `ResearchIndex` projection; final multi-bucket identity and source behavior consume [ABW-63–65](acorn-browser-workspace/index.md#abw-63--compose-multiple-repository-buckets-in-one-bounded-workspace). Do not make the first local graph wait for hosted private-provider acceptance [ABW-19](acorn-browser-workspace/index.md#abw-19--accept-the-initial-research-activity-index-capability). The final acceptance phase waits for the combined-source contract so duplicate paths across buckets cannot collide.
- Coordinate shared `acorn-web` UI/styles/assets with ABW, SCU, USW, FMT, BWP, and ACK owners; schema roots with schema/standards owners. Use a bounded, dependency-free SVG diagram driven by the same accessible list model; inspect the reference GraphPage's interaction details without importing its D3/React bundle.
- The Graph page is read-only. Turtle is an export of **supported graph relations**, not a lossless serialization of every RAD field or a claim that arbitrary Turtle can be imported as RAD. Import, round-trip fidelity, named graphs, and external RDF federation require separate evidence and plans.

# Acceptance criteria

1. With bundled sample RADs, `/graph` shows typed nodes and directed, labeled connections; selecting a node shows its direct relations, source activity/file, and an in-app route to Portfolio details. Explicit cross-activity links are visually distinct from shared entity membership.
2. Search, entity/edge filters, focus radius, fit/zoom/reset, and URL-restorable selection work on desktop and narrow screens. Empty, oversized, malformed/invalid, missing-target, and deleted-workspace states explain what happened without stale nodes.
3. A keyboard and screen-reader user can reach every result through a semantic node/relationship list and selected-node panel; colors are supplemented by text, focus is visible, reduced motion is respected, and no canvas interaction is required to open details.
4. A stable Turtle download contains only documented, source-backed relations, parses under an independent RDF parser, and has the same entity/edge counts as the graph projection after applying its documented omission rules. Local data stays local; navigation or export causes no remote fetch.
5. Focused Rust/WASM and browser checks pass at root and `/acorn/` base paths, with a bounded large-fixture performance check and no new workspace Clippy warnings.

# Execution phases

## GPH-07 — Accept the combined-source graph

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: accepted ABW-63–65 combined-source contract
- Exact write set: `crates/acorn-web/scripts/graph-smoke.mjs`, `crates/acorn-web/src/tests/graph.rs`, `README.md`, `ARCHITECTURE.md`
- Primary integration lock: browser acceptance and documentation
- Outcome: Verify local plus multiple source buckets, same relative path across sources, replacement/removal, large graph limits, accessibility, reduced motion, and static-host deployment; document the graph and linked-data scope.
- Focused validation: Focused Rust tests, graph browser smoke at root and `/acorn/`, `make web-check`, `cargo clippy --workspace --no-deps --all-targets`, `cargo fmt --check`, and `make check-wasm` when its target is installed.
- Handoff: Durable acceptance evidence in tests/docs; retire this plan and its index/network/lane entries once all criteria hold.
