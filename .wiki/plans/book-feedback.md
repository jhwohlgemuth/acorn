---
type: Plan
title: "Address book feedback: overview, Why ACORN, Getting Started, and Concepts/ASPECT"
description: "Resolve ~60 reviewer comments across overview/motivation, navigation/order, Concepts motivation, ASPECT editorial, motivity/data/geospatial, PID lifecycle, agentic/IID, and citations. Consolidates v1 (15 shipped) + v2 (26) + v3 (14 PID/agentic/citations)."
difficulty: medium
impact: medium
package: [acorn-cli]
tags: [docs, book, mdbook, content, aspect, geospatial, pid, agentic]
status: planned
priority: next
---

# Project code boundaries

- **Update:**
  - `docs/book.toml:5` - title `🌱 Plant an acorn, grow your science` — **WONTFIX: cannot be centered** (mdbook `book.title` renders only in sidebar `menu-title` and `<title>` tag, not as centered cover H1; see user update 2026-09-06; no code change — document as accepted constraint)
  - `docs/src/SUMMARY.md:3-26` - reorder Concepts before Getting Started; resolve Choose your path vs Quick Quest order; resolve merge conflict markers `<<<<<<<` at 42-48; fix ASPECT subpage order (portability/maturity/autonomy/motivity/data) to match display order; move `Citations` to very end (section 9) per new feedback
  - `docs/src/overview.md:38,40,47,63,69,106` - prior fixes retained; verify no regression
  - `docs/src/motivation.md:6,13,19-28,30-36,43` - prior fixes retained
  - `docs/src/learn/README.md:1-13` - heading/nav and Next-stop link; Quick Quest ordering
  - `docs/src/learn/developer.md:13` - broken `[command pages](../cli-reference/commands/index.html)` -> `../cli-reference/commands/README.md`
  - `docs/src/concepts/README.md:1-42` - motivation 1-3 tone, citations 2018/2016, REI acronym, science-all-the-way-down punctuation, research.ornl.gov coming-soon, link audit
  - `docs/src/concepts/aspect/README.md:1-99` - remove 4 AI/ML-attack sentences, fix TIP link, fix figure numbering/caption, add white text boxes for dark mode, reorder attribute list vs page order, reduce cross-link manic density, fix geospatial passthrough link
  - `docs/src/concepts/aspect/maturity.md:1-23` - move table below Discussion, de-duplicate Additionally, fix TIP link
  - `docs/src/concepts/aspect/autonomy.md:1-92` - add consistent descriptions for HMT 2-5 (currently only 0-1 have prose), verify TIP link
  - `docs/src/concepts/aspect/motivity.md:1-94` - reorder table columns to perception→projection→comprehension, define stateless/stateful, reconcile level order vs text, add missing level 4/5 prose parity
  - `docs/src/concepts/aspect/data.md:1-98` - add citations or drop "multiple other data quality frameworks", fix geospatial passthrough link `(#geospatial-data)`, edit electro-optical/NASA scope, (e.g. satellites) wording, Level 2B vs 1B quality claim, related cross-link reduction
  - `docs/src/concepts/persistent-identifiers.md:1-182` - enlarge mermaid figure text, clarify table Roles/Validation/Normalized, add purpose/context for steps 1-7, reduce/convert code snippets to links, emphasize steps 5-7 vs 1-4, convert flags in 6 to bulleted list, ensure table after referencing text
  - `docs/src/concepts/agentic.md:1-127` - JORS/IID publication timing, define MCP/ACP, fix "in" vs "In"/IiD capitalization, move table below text + add numbers, consider splitting dense page, reduce "it's this, not this" phrasing, explain OpenAI-compatible scope
  - `docs/src/concepts/citations.md:1-45` - move to end of Concepts (or book) per feedback
  - `docs/src/concepts/aspect/portability.md` - verify order consistency (no change expected)
  - `docs/src/_assets/aspect_example_annotated.png` - regenerate or overlay white text boxes for dark mode legibility (or add CSS filter)
- **Create:** `None`
- **Delete:** `None` (if `SUMMARY.md` conflict markers removed, no file deletion)

# Dependencies

- `make book` / `mdbook build docs` anchor verification for `### 🪣 Buckets` (`id="-buckets"`), `### 🌎 Geospatial Data` (`id="-geospatial-data"`), maturity/REI anchors, new PID/agentic anchors.
- No code dependency. Docs-only, orthogonal to analyzer/portability/feature lanes per `.wiki/parallel-execution.md`. One mdbook integration owner for `SUMMARY.md` (single-writer lock for index generation).
- Requires editorial sign-off on tone softening (citations 2018/2016), AI/ML sentence removal, and JORS/IID timing before merge.

# Evidence

## Already shipped (v1, retained)

- `docs/src/overview.md:40` period, `:47` footnote quote, `:63` `learn/README.md` link, `:69` Title Case heading - verified `mdbook build` zero warnings.
- `docs/src/motivation.md:13` softened to `few widely adopted systems use the project itself as the primitive`; `:21-23` conjunction on `budgets, and`; `:27-28` security-first with `SECURITY.md` defer; `:32-33` TIP support; `:36` lifecycle `acorn format`/`acorn check`; `:43` `concepts/README.md#-buckets`.
- `docs/src/getting-started.md:3-10` IMPORTANT reframe, bridge, per-method `> Use this if...` callouts.

## New feedback (v2) - mapped to file:line

| # | Comment | File:line (current) | Evidence |
|---|---------|---------------------|----------|
| N1 | Choose your path before Quick Quest? Next stop points to Quest | `SUMMARY.md:6-10` + `learn/README.md:13` | SUMMARY orders `Choose your path` (6) then `Quick Quest` (10); `learn/README.md:13` `Next stop: Quick Quest` creates loop. Reviewer asks if order should swap. |
| N2 | choose your path: developer, link to command pages broken | `learn/developer.md:13` | `../cli-reference/commands/index.html` - file is `commands/README.md` (`SUMMARY.md:26` indexes `README.md`), build generates `commands/index.html` but source link should be `README.md` for consistency. |
| N3 | "Plant an acorn..." not centered | `docs/book.toml:5` `title = "🌱 Plant an acorn, grow your science"` | Title appears only in sidebar `menu-title` and `<title>` tag (`docs/book/*/index.html:143`), not centered on `overview.md` cover; per user update 2026-09-06 this is mdbook-generated behavior and **cannot be centered** — accepted as WONTFIX. |
| N4 | Concepts should be before Getting Started (why section 6?) | `SUMMARY.md:5-11` | Order is Overview(3) Why(4) Getting Started(5) Choose(6) Quest(10) Concepts(11). Reviewer questions pedagogy. |
| N5 | concepts motivation citation 1 (2018) outdated, attacks ego | `concepts/README.md:7,38` | `Publish or perish` with Grimes 2018; text `Scientists...often care more about publishing than sharing`. |
| N6 | citation 2 (2016) also old, attacks publications | `concepts/README.md:8,40` | `Reproducibility crisis` Baker 2016; `Many publications do not provide data/code`. |
| N7 | point 3 does ACORN also include people/pubs/budgets? clarify | `concepts/README.md:9` | `No clear way to demonstrate science: [Publications, people, budgets do not tell...]` then `Science needs cross-domain standard` - ambiguous if ACORN subsumes all. |
| N8 | "science all the way down" comma inside quotes | `concepts/README.md:11` | `Predicated on the idea of applying "science all the way down",` comma outside US-style `,"`. |
| N9 | include (REI) after first mention | `concepts/README.md:14` | First mention `Research Enablement Initiative at ORNL. REI aims...` second sentence uses REI without parenthetical. |
| N10 | research.ornl.gov still coming soon? | `concepts/README.md:23` + `cli-reference/commands/export.md:51` | Link to `https://research.ornl.gov` catalog; site may be stub. |
| N11 | ASPECT remove 4 sentences attacking AI/ML | `concepts/aspect/README.md:10-12` | `We focus on "technology" instead...Focusing on AI/ML as end goal is not fruitful or correct. In fact, doing so is backward. AI/ML software is not novel...Even if it was...100% predicated...In the context of technology, AI/ML and automation are the same.` |
| N12 | aspect link in tip broken | `concepts/aspect/README.md:15` | `[Read docs.rs documentation for ASPECT](https://docs.rs/acorn-lib/latest/acorn/schema/research_activity/aspect/index.html)` - path may 404 (crate is `acorn-schema` not `acorn-lib` for schema docs). |
| N13 | aspect maturity link in tip broken | `concepts/aspect/maturity.md:14` | `[Read docs.rs documentation for maturity levels](https://docs.rs/acorn-lib/latest/acorn/schema/enum.TechnologyReadinessLevel.html)` - enum path likely `acorn/...TechnologyReadinessLevel` not under `acorn-lib`. |
| N14 | maturity table should be below discussion (figures after mention, TRL defined later) | `concepts/aspect/maturity.md:3-17` | Table at `1:3` before `## Discussion` at `1:16` which defines TRL; violates figure-after-mention. |
| N15 | maturity discussion has Additionally twice | `concepts/aspect/maturity.md:17` | `Additionally, ASPECT incorporates work... Additionally, ASPECT maturity levels work well...` |
| N16 | autonomy descriptions only for HMT 0,1 not all | `concepts/aspect/autonomy.md:14-88` | `Manual (HMT 0)` has prose at `:15`; `Machine-assisted (1)` at `:26`; `Human as primary (2)` `:39` empty, `Machine as primary(3)` `:53` empty, `Human supervisor(4)` `:67` empty, `Machine only(5)` `:80` empty - inconsistent. |
| N17 | listed attributes p>mat>aut>mot doesn't match page order (switch aut/mat) | `concepts/aspect/README.md:6` `portability \| maturity \| autonomy \| motivity` vs `SUMMARY.md:14-17` same order `portability, autonomy, maturity, motivity` mismatch |
| N18 | motivity table should be perception>projection>comprehension, define stateless/stateful | `concepts/aspect/motivity.md:19` | Header `Perception \| Comprehension \| Projection \| Stateless/Stateful` - reviewer suggests `Perception \| Projection \| Comprehension`. Stateless/stateful undefined. |
| N19 | motivity levels not in order, levels 4,5 not defined like 1-3 | `concepts/aspect/motivity.md:12-26,68-82` | Table order `Inert 0, Computational 1, Perceptive 2, Projective 3, Reactive 4, Adaptive 5` vs text primitives `perception, projection, comprehension` then note, but levels 4 Reactive and 5 Adaptive sparse vs 1-3 detail imbalance; reviewer says messy. |
| N20 | data: citations for multiple other data quality frameworks or drop | `concepts/aspect/data.md:74` | `...inspiration from data readiness levels [^1], analysis-ready data[^2], and multiple other data quality frameworks.` only 2 cited. |
| N21 | aspect: do not need link to geospatial section below (doesn't move) | `concepts/aspect/data.md:74` | `See [the geospatial data section below](#geospatial-data)` - anchor `id="-geospatial-data"` should work but reviewer reports no scroll. |
| N22 | a lot of cross-talking links feels manic, cut down | Global - `concepts/aspect/README.md`, `data.md`, `concepts/README.md` | Many `See ...`, `Read docs.rs`, inline `ASPECT`, `acorn-py` links per page. |
| N23 | geospatial levels are for electro-optical imagery, SAR different, NASA vs commercial | `concepts/aspect/data.md:79-89` | `Based on NASA's Data Processing Levels` list 0-4 without scoping. |
| N24 | (e.g. satellites) can be removed, say that may include processing level | `concepts/aspect/data.md:91` | `Most remote sensing sources (e.g. satellites) provide metadata that includes the data processing level.` |
| N25 | not claim L2B higher quality than L1B (1B closer to raw, more accurate) | `concepts/aspect/data.md:93` | `For instance, Level 2B data...would generally be considered higher quality than Level 1B data...` - reviewer disputes quality framing. |
| N26 | figure 1 labeled, earlier figures none; fig1 detailed caption fig2 not; fig2 annotations unreadable in dark mode | `concepts/aspect/README.md:91-99` | `Figure 1: ASPECT badge` at `:91`, `Figure 2: Annotated...` at `:99`; earlier pikchr figures unlabeled; `aspect_example_annotated.png` dark text invisible on dark theme. |

## New feedback (v3) - PID, Agentic, Citations

| # | Comment | File:line (current) | Evidence |
|---|---------|---------------------|----------|
| P1 | PID: text in figure too small to read (giant screen) | `concepts/persistent-identifiers.md:5-18` | Mermaid `flowchart LR` with 8 nodes `input→find→parse→validate→normalize→classify→report/persist→resolve` rendered small. |
| P2 | PID: need clarifications/definitions surrounding table (Roles, Validation vs type checking, Normalized, links vs non-links) + table should be after referencing text + unstructured text after table | `concepts/persistent-identifiers.md:20-44` | Table at `:24` `Kind \| Validation \| Normalized \| Role` with terse values `NAAN and assigned-name structure`, `Bare DOI`, `Project identity` without glossary; `Validation is local...` at `:39` after table (should be before); `Role` column mixes `Project identity` vs `Supporting evidence` vs `Person evidence`. |
| P3 | PID: include more purpose/context for inclusion of steps 1-7 what are they for together? | `concepts/persistent-identifiers.md:3-7` + `:45-182` | Steps `1. Find candidates` through `7. Build exact identities` listed as lifecycle at `:3` `PID as evidence...` but no umbrella purpose paragraph linking them as evidence pipeline. |
| P4 | PID: are code snippets necessary? Suggest only if user-managed, else links | `concepts/persistent-identifiers.md:48-121` | Code blocks at `:49-57` (find_all), `:65-73` (from_string), `:81-87` (is_valid), `:95-118` (format/normalized) — reviewer asks if these are user-managed. |
| P5 | PID: maybe emphasize 5,6,7 as primary purposes, 1-4 as single entity/sub-components, not equal weight | `concepts/persistent-identifiers.md:5-18` mermaid + `:126-168` sections 5-7 | Mermaid shows 7 equal nodes; sections 5 `Classify`, 6 `Resolve and enrich`, 7 `Build exact identities` are persistence-critical vs 1-4 prep. Reviewer suggests weighting. |
| P6 | PID: flags in 6 would be easier as bulleted list | `concepts/persistent-identifiers.md:137-152` | Paragraph at `:137` `arXiv and DOI evidence is sent to CiteAs, Handle...` dense; `All enrichment follows same merge rules: 1-5` at `:143` plus RAiD roles at `:151` could be bulleted. |
| A1 | agentic: should we publish IID in JORS first before public? | `concepts/agentic.md:3` | `**It is growing in that direction.** ACORN approaches agentic development through **Intelligence in Depth (IID)**: ...` — term IID appears publicly without formal prior art. |
| A2 | agentic: define MCP and ACP | `concepts/agentic.md:10,68-69` | `Model Context Protocol (MCP)[^mcp] can connect...` and `Agent Client Protocol (ACP)[^acp] lets clients...` at `:10` with footnote only; main text at `:60-69` defines but sparse. |
| A3 | agentic: "in" should be capitalized or acronym "IiD" | `concepts/agentic.md:3,33-34` | Title `Intelligence in Depth (IID)` vs inline `Intelligence in Depth`; reviewer suggests `Intelligence In Depth` or `IiD` consistently. |
| A4 | agentic: table before text — switch; add numbers to table if keeping numbers in text | `concepts/agentic.md:44-82` + `:72-82` | Table at `:74` `IID layer | Local-first...` before `## How IID Shapes ACORN` explanatory text; IID Layers numbered 1-6 at `:46-71` but tablerows not numbered. |
| A5 | agentic: break into multiple pages (longest/densest, most important, needs emphasis) | `concepts/agentic.md:1-127` (127 lines) | Longest concepts page (vs PID 182 but dense, aspect 99); reviewer suggests split. |
| A6 | agentic: lots of "it's this, not this" chatgpt content | `concepts/agentic.md:28-31,42-43,85-97` | `Arrows indicate... not increasing maturity`, `IID is an ordered architecture, not an ASPECT maturity scale`, `not a synonym for maximum autonomy`, `Local-first ... not prohibiting ... without...`, `protocols are adapters, not dependencies`, `not yet implemented`, `does not score... or imply...`, etc. |
| A7 | agentic: describe why only OpenAI compatible (researcher preferences, support multiple or state why) | `concepts/agentic.md:48-50,99-103,109-111` | `OpenAI-compatible` at `:50` defined as format not provider, but no rationale why only that API; table `:103` `OpenAI-compatible API`. |
| A8 | agentic: also N11-like table-before-text for IID layers? (duplicate of A4) | `concepts/agentic.md:14-27` + `:46-82` | Pikchr at `:14` then layers 1-6 text; reviewer notes same table-before-text antipattern as maturity. |
| C1 | citations: move to very end as section 9, feels weird here | `concepts/citations.md:1-45` + `SUMMARY.md:23` | `SUMMARY.md:23` lists `Citations` inside `Concepts` before `Command Line Reference`; reviewer suggests move to book end (section 9). |

# Recommendation

## Phase S - Structure & Navigation (single-writer: SUMMARY.md)

- **S1 `SUMMARY.md:6-11` Choose vs Quick Quest order:** Propose keeping `Choose your path` before `Quick Quest` (current) - rationale: path choice is decision hub, Quest is execution. Alternative if reviewer insists: swap to `Quick Quest` before `Choose your path` and change `learn/README.md:13` Next-stop from `Quick Quest` to most-visited trail (e.g., Researcher). **Decision needed:** Default recommendation is **keep order**, clarify `learn/README.md:13` copy to `If you have chosen a trail, begin... else Quick Quest` to resolve "next stop" circularity without reordering. If swap, then `SUMMARY.md:6` and `:10` lines transpose and `learn/README.md:13` already correct.
- **S2 `SUMMARY.md:11-18` Concepts before Getting Started:** Recommend moving `🧠 Concepts` before `🚀 Getting Started` only if book is theory-first; currently Getting Started is intentionally early for hands-on. Propose **compromise**: keep Getting Started at 5, but add forward-link in `getting-started.md:8` `If you want background, see [Concepts](../concepts/README.md) first.` Alternatively move full Concepts block to line 5 and push Getting Started to after Quick Quest (line ~12). **Decision needed** - default recommendation: **keep current**, add cross-link, document rationale.
- **S3 `SUMMARY.md:23,42-48` Merge conflict + Citations move:** Remove `<<<<<<<`, `|||||||`, `=======`, `>>>>>>>` and decide inclusion: keep `🧰 Local inference and tools` (upstream) plus `Human acceptance test plan` if both valid, else choose one. Move `- [Citations](./concepts/citations.md)` from line 23 to end of book (e.g., after Packages as section 9): `- [Citations](./concepts/citations.md)` or `docs/src/citations.md`. Update `SUMMARY.md` accordingly; ensure `concepts/citations.md` remains or is moved physically.
- **S4 `SUMMARY.md:14-18` ASPECT order:** Align attribute list and page order. Either change `concepts/aspect/README.md:6` from `portability | maturity | autonomy | motivity` to `portability | autonomy | maturity | motivity` to match `SUMMARY.md`, OR reorder `SUMMARY.md:15-16` to swap autonomy/maturity. Recommend **fix README.md** to match SUMMARY (less churn). Also reorder pikchr badge attributes if needed.
- **S5 `learn/developer.md:13` broken command pages:** Change `../cli-reference/commands/index.html` -> `../cli-reference/commands/README.md` (mdbook canonical source). Verify `docs/book/cli-reference/commands/index.html` still resolves.
- **S6 `docs/book.toml:5` title centering — WONTFIX:** Per user update 2026-09-06, `book.title` is mdbook-generated (sidebar `menu-title` + `<title>` tag) and cannot be centered on-page. No CSS or `overview.md` H1 injection `acorn-docs.css` change required. Document as accepted constraint; no validation step.

## Phase C - Concepts Motivation Tone & Accuracy

- **C1 `concepts/README.md:7` Pub-or-perish reword (N5):** Replace
  ```md
  1. **Publish or perish**[^publish]: Scientists and researchers often care more about *publishing* than about sharing knowledge. This is not without reason...
  ```
  with
  ```md
  1. **Publication pressure**[^publish]: Unfortunately, researchers are often burdened by the pressure to publish often, which, while sometimes beneficial to the community, hinders researchers from allotting more time and energy to conducting science. Publications remain a key metric for career and funding, which can tilt incentives toward quantity and a reluctance to share negative results or unsupported hypotheses.
  ```
  Keep Grimes 2018 citation but soften to system/institution blame, not ego.
- **C2 `concepts/README.md:8` Reproducibility reword (N6):** Replace attack on publications with
  ```md
  2. **Reproducibility gaps**[^reproducibility]: Many valuable results are difficult to reproduce. This is not for lack of intent — researchers often want to share — but sharing underlying data and code is not always required, so reproducibility is unfortunately sometimes left behind, raising questions about validity.
  ```
  Keep Baker 2016 but hedge, praise intent.
- **C3 `concepts/README.md:9` Clarify ACORN scope (N7):** Append to point 3: `ACORN does not replace these systems; its research activity data record links and contextualizes people, publications, budgets, and approvals together.` Add explicit sentence after list before `ACORN addresses...`
- **C4 `concepts/README.md:11` Science all the way down (N8):** Change `applying "science all the way down",` -> `applying "science all the way down,"` (comma inside).
- **C5 `concepts/README.md:13-14` (REI) (N9):** Change `Research Enablement Initiative at ORNL. REI aims...` -> `Research Enablement Initiative (REI) at Oak Ridge National Laboratory (ORNL). REI aims...`
- **C6 `concepts/README.md:23` research.ornl.gov (N10):** Audit site; if still `coming soon`, replace link with `research.ornl.gov (catalog in preview)` or hide link behind `> [!NOTE] Catalog is in preview - contact REI for access.` Alternatively add footnote `Catalog is being populated; check back or see buckets at ...`.
- **C7 `concepts/README.md:17-20` link audit (N22):** Reduce from 5 links to 2: keep ACORN + ASPECT, move crate/python links to footer or Packages page. Remove redundant `[/concepts/acorn.html]` absolute vs relative.

## Phase A - ASPECT Editorial & Structure

- **A1 `concepts/aspect/README.md:10-12` Remove 4 attacking sentences (N11):** Delete
  `We focus on "technology" instead of "***AI/ML*** technology" because...Focusing on AI/ML as the end goal is not fruitful...In fact, doing so is backward. AI/ML software is not novel...Even if it was, it would still be 100% predicated...In the context of technology, AI/ML and automation are the same.`
  Replace with neutral: `ASPECT treats automation, AI/ML, and classical software as subsets of technology, providing one shared classification space.`
- **A2 `concepts/aspect/README.md:15` TIP link (N12):** Verify `https://docs.rs/acorn-lib/latest/acorn/schema/research_activity/aspect/index.html` vs actual crate. Check `crates/acorn-schema` docs path; likely `https://docs.rs/acorn-schema/latest/acorn_schema/schema/research_activity/aspect/index.html` or `acorn-lib`. Fix or replace with `https://docs.rs/acorn-lib/latest/acorn/` generic. Add `mdbook` link check.
- **A3 `concepts/aspect/maturity.md:14` TIP link (N13):** Same - enum `TechnologyReadinessLevel` lives in `crates/acorn-schema/src/schema/research_activity/aspect/maturity.rs`; fix docs.rs URL or point to `https://docs.rs/acorn-schema/...`.
- **A4 `concepts/aspect/maturity.md:3-17` Table below discussion (N14):** Move table `| Maturity Level | TRL |` from top `:3` to after `## Discussion` paragraph (after `:17` or after TRL image). Keep TIP just below table. Ensure figure-after-mention satisfied and TRL defined before table appears.
- **A5 `concepts/aspect/maturity.md:17` Additionally x2 (N15):** Change first `Additionally, ASPECT incorporates work[^lavin]...` -> `ASPECT also incorporates work[^lavin]...` and keep second `Additionally, ASPECT maturity levels work well...` or rephrase second to `Moreover,`.
- **A6 `concepts/aspect/autonomy.md:39-88` HMT 2-5 descriptions (N16):** Add 1-2 sentence prose for each empty level, matching style of HMT 0/1. Draft:
  - HMT2 Human as primary: `Human directs execution; machine proposes next steps and executes subtasks under continuous human approval.`
  - HMT3 Machine as primary: `Machine directs execution; human intervenes on exceptions or policy checks.`
  - HMT4 Human supervisor: `Machine operates independently; human monitors and sets bounds.`
  - HMT5 Machine only: `Machine executes end-to-end with no human in the loop.`
- **A7 `concepts/aspect/README.md:6` vs SUMMARY order (N17):** As S4, ensure `portability | maturity | autonomy | motivity` matches final SUMMARY order. Add note to keep consistent.
- **A8 `concepts/aspect/motivity.md:19` Column order + define (N18):** Change header to `| Level | Name | Perception | Projection | Comprehension | Stateless/Stateful | Examples |` (swap projection before comprehension per suggestion? Actually N18 says perception > projection > comprehension, which matches primitives order described at `:14`. Implement that order. Add footnote or sentence before table: `Stateless: no memory between interactions; Stateful: retains internal state.` )
- **A9 `concepts/aspect/motivity.md:12-26,68-82` Level order & completeness (N19):** Current table order 0 Inert,1 Computational,2 Perceptive,3 Projective,4 Reactive,5 Adaptive does not follow perception→projection→comprehension primitive order but is intentional ordinal complexity. Either add introductory sentence `Levels are ordered by increasing interaction complexity, not by primitive order.` or reorder to `Perceptive/Projective` adjacency if preferred. Add expanded prose for levels 4 Reactive and 5 Adaptive to match length of 1-3 (currently present but short - expand to 2 paragraphs each with examples and stateless/stateful note). Mark as editorial completeness.
- **A10 `concepts/aspect/data.md:74` Citations (N20):** Either add 1-2 citations for other data quality frameworks (e.g., `https://doi.org/...` for DAMA, or remove phrase. Recommend: change `...and multiple other data quality frameworks.` -> `...and related data quality frameworks[^dama].` and add `[^dama]` entry, or delete clause to `...from data readiness levels [^1] and analysis-ready data[^2].`
- **A11 `concepts/aspect/data.md:74` Geospatial passthrough link (N21):** `(#geospatial-data)` currently fails because mdbook slug is `#-geospatial-data` (from `### 🌎 Geospatial Data`). Fix to `(#-geospatial-data)` or remove link entirely per N21 (reviewer says do not need link). Recommend **remove link**, keep plain `See the geospatial data section below` or delete sentence clause.
- **A12 `concepts/aspect/data.md:74-94` Cross-link manic reduction (N22):** Audit all pages for `See ...` / `Read docs.rs` density. Reduce to at most one forward-link per section. Remove at least 2 of 3: pikchr caption links, docs.rs TIPs, geospatial passthrough.

## Phase G - Geospatial (N23-N25)

- **G1 `concepts/aspect/data.md:79` NASA scope (N23):** Prepend `For many electro-optical imagery collections, NASA's Data Processing Levels[^nasa-dpl] define:` and add note `SAR and other modalities use different level schemes; not all providers adhere to NASA levels. NASA and several commercial providers publish similar level metadata.`
- **G2 `concepts/aspect/data.md:91` (e.g. satellites) wording (N24):** Change `Most remote sensing sources (e.g. satellites) provide metadata that includes the data processing level.` -> `Many remote sensing sources provide metadata that may include the data processing level, depending on the provider.`
- **G3 `concepts/aspect/data.md:93` Quality claim (N25):** Change `For instance, Level 2B data ... would generally be considered higher quality than Level 1B data, which is only annotated` -> `For instance, Level 1B data (closer to raw) preserves measured values more faithfully, while Level 2B data trades some fidelity for usability through additional processing. Which is "higher quality" depends on use case - analysis-ready workflows may prefer 2B, calibration work may prefer 1B. For example, orthorectification might be the difference between silver and gold readiness for AI/ML applications.`

## Phase F - Figures (N26)

- **F1 `concepts/aspect/README.md:91-99` Figure labeling:** Remove `Figure 1`/`Figure 2` labels OR add consistent labeling to all earlier pikchr figures (portability/maturity/autonomy/motivity). Recommend **remove Figure 1/2 labels** and use `Figure: ASPECT badge...` without numbering, or number all figures sequentially. Also make captions consistent: both detailed or both minimal. Ensure `Figure 1 is an example... Figure 2 below shows...` text updated to match.
- **F2 Dark mode legibility:** `aspect_example_annotated.png` annotations use dark text invisible on dark themes. Fix by adding white text boxes behind annotations, or CSS `filter: drop-shadow + background`, or regenerate with `fill white` callouts. Test in `ayu/coalm/navy` vs `light/rust`.

## Phase P - PID Lifecycle

- **P1 `concepts/persistent-identifiers.md:5-18` Mermaid text size (P1):** Increase `flowchart LR` node font via CSS `mermaid` theme or convert to `flowchart TD` with larger `fontSize`/`nodeSpacing`, or replace mermaid with `pikchr` to match other diagrams. Add `%%{init: {'themeVariables':{'fontSize':'16px'}}}%%` and test on 4k. Ensure `text` wrapping not clipped.
- **P2 `concepts/persistent-identifiers.md:20-44` Table definitions + placement (P2):** Move table at `:24` to after explanatory paragraphs `:39-44`. Before table, add glossary paragraph defining each column: `Role: Project identity = can create/match a research activity; Supporting evidence = retained in report/history but not candidate-forming; Person/Organization evidence = entity-bound.` `Validation: local syntax/checksum/type checking, no network.` `Normalized: display form suitable for RAD fields vs `identity_key` for matching.` Add sentence explaining why some cells contain bare links (`Bare DOI`, `https://ror.org/...`) vs formatting. Structure formally: H2 Definitions → H2 Supported identifiers (table) → H2 Lifecycle narrative.
- **P3 `concepts/persistent-identifiers.md:3,45-182` Purpose for steps 1-7 (P3):** Add umbrella paragraph after `:3` `ACORN treats a PID as evidence...`:
  ```md
  Together steps 1-7 form the evidence pipeline that turns unstructured text into candidate research activities: steps 1-4 prepare and clean identifiers locally; steps 5-7 decide their role, optionally enrich from authorities, and persist exact identities. The pipeline is local-first and additive.
  ```
  Reference purpose before diving into steps.
- **P4 `concepts/persistent-identifiers.md:48-121` Code snippets necessity (P4):** Audit each snippet: `find_all`, `from_string`, `is_valid`, `format`, `Identifier::normalized` are trait APIs users may call in Rust/Python/WASM. Convert to **collapsible** `<details>` blocks labeled `Implementation detail` or replace with `> [!NOTE] Full API at docs.rs` plus link `https://docs.rs/acorn-schema/.../PersistentIdentifierParse`. Keep at most one snippet if editorial insists, else move to `packages/acorn-lib.md` and link. Ensure `acorn gather` CLI path remains documented for non-Rust users.
- **P5 `concepts/persistent-identifiers.md:126-168` Emphasis weighting 5-7 vs 1-4 (P5):** Re-structure: group 1-4 under `## A. Prepare evidence locally` (with subheads 1-4), and 5-7 under `## B. Decide, enrich, persist (primary purposes)` with heavier headings, callout `> [!IMPORTANT] Classification, resolution, and identity building are the primary purposes; finding/parsing/validating/normalizing are preparatory sub-components.` If reviewer wrong, instead add sentence clarifying equal weight and why 1-4 are not subordinate.
- **P6 `concepts/persistent-identifiers.md:137-152` Flags as bulleted list (P6):** Convert dense paragraph `:137` `arXiv and DOI -> CiteAs, Handle -> Handle proxy...` into bullet list:
  ```md
  - `arXiv`/`DOI` -> CiteAs
  - `Handle` -> public Handle proxy REST API
  - `ORCID` -> ORCID API
  - `RAiD` -> RAiD resolver
  - `SWHID` -> Software Heritage resolver
  - `ISNI` -> local-only (no resolver)
  ```
  Similarly bullets for enrichment merge rules 1-5 and RAiD role mapping.

## Phase Ag - Agentic / IID

- **Ag1 `concepts/agentic.md:3` JORS timing (A1):** Editorial decision: if IID term prior art is intended, add footnote `IID terminology is being prepared for JORS submission; cite preprint or contact authors for prior claim.` Alternatively gate public docs behind `> [!NOTE] IID is ACORN-internal terminology pending publication.` Requires PI decision before merge.
- **Ag2 `concepts/agentic.md:10,60-69,98-104` Define MCP/ACP (A2):** In first mention at `:10`, expand: `Model Context Protocol (MCP) — a standard for agents to access tools/resources — and Agent Client Protocol (ACP) — a standard for clients to supervise agents —`. Ensure footnote `[^mcp][^acp]` remain but main text self-contained.
- **Ag3 `concepts/agentic.md:3,33` Capitalization (A3):** Choose `Intelligence in Depth (IID)` (title case, `in` lower) vs `Intelligence In Depth (IID)` or `IiD`. Recommend **keep `Intelligence in Depth (IID)`** (Chicago/AP title case keeps `in` lower for 2-letter preposition) and enforce globally via `grep -rn "Intelligence [Ii]n"`; update any `IiD` to `IID`. Document decision as house style.
- **Ag4 `concepts/agentic.md:46-82` Table before text + numbers (A4/A8):** Move table `| IID layer | Local-first...` at `:74` to after `## How IID Shapes ACORN` explanatory paragraphs or below `### Guiding Principles`. Add numbering column to table: `| # | IID layer | Application |` aligning 1-6 with layer list. Ensure numbers in text (`1. One-time prompting`) correspond to table rows.
- **Ag5 `concepts/agentic.md:1-127` Split page (A5):** Page is 127 lines (dense). Recommend split into two pages: `concepts/agentic.md` (overview + guiding principles + layers 1-4) and `concepts/agentic-harness.md` or `concepts/iid-harness.md` (layers 5-6, How IID Shapes ACORN, Current Support, MCP/ACP table). Alternatively keep single page but add `> [!TIP] This is ACORN's most important conceptual page` plus collapsible TOC. **Decision needed.**
- **Ag6 `concepts/agentic.md:28-43,72-97` Reduce "it's this, not this" phrasing (A6):** Audit for `not`, `rather than`, `instead of expecting`, `not a synonym`, `not more mature`, `not requiring`, `not yet implemented`. Rephrase to affirmative: e.g., `IID distributes intelligence across...` instead of `Instead of expecting one prompt...`. Limit to at most one hedged contrast per section.
- **Ag7 `concepts/agentic.md:48-50,99-103` OpenAI-compatible rationale (A7):** Expand `OpenAI-compatible describes the connection format, not who provides the model.` with rationale:
  ```md
  ACORN chose the OpenAI-compatible format because it is widely emulated by local and hosted servers (llama-swap, vLLM, LiteLLM) and minimizes custom client code. Support for additional formats (e.g., Anthropic, Google) can be added; contributions for a new `ModelProvider` are welcome.
  ```
  Or explicitly state **WONTFIX: only OpenAI-compatible for now** with justification.
- **Ag8 `concepts/agentic.md:14-27,44-82` Table/text order (duplicate of Ag4):** Consolidated with Ag4; ensure pikchr at `:14` remains but is introduced by preceding sentence `The six layers compose as follows:` rather than appearing before explanatory text.

## Phase Cit - Citations

- **Ct1 `SUMMARY.md:23` + `concepts/citations.md:1-45` Move to end (C1):** Move `- [Citations](./concepts/citations.md)` from `Concepts` subsection to book end as section 9 (e.g., `- [Citations](./citations.md)` or `- [References](./references.md)`). Either move file to `docs/src/citations.md` or keep path but list under separate top-level heading `Citations` after `Packages`. Update `concepts/citations.md` header to `Citations` without numbering. Ensure `mdbook` index regenerates.

# Validation

- [ ] `mdbook build docs` zero warnings; merge conflict markers removed; citations moved and still indexed.
- [ ] Click-through every changed link: `learn/developer.md` command pages, ASPECT/docs.rs TIPs, geospatial anchor, REI/research.ornl.gov, buckets, PID docs.rs trait links, agentic MCP/ACP footnotes.
- [ ] `grep -c "Figure [12]" docs/src/concepts/aspect/README.md` == 0 or consistent numbering.
- [ ] `grep -n "Additionally" docs/src/concepts/aspect/maturity.md` == 1 max.
- [ ] `grep -n "science all the way down,"` verifies comma inside quotes.
- [ ] `grep -n "(REI)" docs/src/concepts/README.md` present.
- [ ] PID mermaid renders legibly on 1440p/4k; table follows definitions and is not first element under H2.
- [ ] `grep -n "Intelligence in Depth"` consistent capitalization; no `IiD` stray.
- [ ] Agentic table numbered 1-6 and placed after Guiding Principles/layers text.
- [ ] `mdbook` HTML in `ayu`, `coal`, `navy`, `light`, `rust` shows legible annotated figure (title centering exempt per WONTFIX N3).
- [ ] `vale` or prose lint no new warnings; citations still resolve; citations at book end indexed as section 9.

# Acceptance criteria

- Navigation order decisions documented and implemented (or rationale for keeping), merge conflict cleared, ASPECT order consistent, developer command-pages link fixed, title centering acknowledged as mdbook WONTFIX, citations moved to end.
- Motivation tone softened (N5/N6), scope clarified (N7), punctuation/REI fixed (N8/N9), research.ornl.gov stub handled (N10), and cross-link density reduced (N22).
- ASPECT 4 attacking sentences removed (N11), both TIP links verified/fixed (N12/N13), maturity table moved below discussion and Additionally de-duplicated (N14/N15), autonomy HMT 2-5 prose added (N16), attribute order aligned (N17), motivity table columns defined and order reconciled (N18/N19), data citations handled (N20), geospatial link removed/fixed (N21), geospatial scope clarified (N23-N25).
- Figures un-numbered or consistently numbered and dark-mode legible (N26).
- PID figure legible, table clarified and correctly placed after definitions, purpose for steps 1-7 added, code snippets collapsed/linked, emphasis on 5-7 vs 1-4 clarified, flags bulleted (P1-P6).
- Agentic IID publication timing documented, MCP/ACP defined, capitalization consistent, IID table reordered below text and numbered, page split or emphasis decided, "it's this, not this" reduced, OpenAI-compatible rationale added (Ag1-Ag8).
- Citations relocated to book end (Ct1).
