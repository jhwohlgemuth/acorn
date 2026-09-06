---
type: Plan
title: "Address book feedback: overview, Why ACORN, Getting Started"
description: "Resolve 15 reviewer comments on docs/src/overview.md, docs/src/motivation.md and docs/src/getting-started.md - capitalization, punctuation, broken links, risk ordering, claim softening, single-source-of-truth lifecycle, and install UX."
difficulty: low
impact: medium
package: [acorn-cli]
tags: [docs, book, mdbook, content, security]
status: planned
priority: next
---

# Project code boundaries

- **Update:**
  - `docs/src/overview.md:38` sentence body - keep lower-case `research activity data (RAD)` (prose rule)
  - `docs/src/overview.md:40` - add missing period after `Linux machine`
  - `docs/src/overview.md:47` - fix footnote `[^acronym]` missing closing `"`; keep footnote per decision
  - `docs/src/overview.md:63` - broken `[role guide](learn/index.html)` -> `[role guide](learn/README.md)`
  - `docs/src/overview.md:69` - heading `### Research Activity Data` stays Title Case (prose lower / heading Title)
  - `docs/src/motivation.md:6` - align prose to lower-case `research activity data` (already correct)
  - `docs/src/motivation.md:13` - soften novelty claim
  - `docs/src/motivation.md:19-23` - move `, and` from `people` bullet to `budgets` bullet
  - `docs/src/motivation.md:27-28` - reorder risks: security first, then funding/HR; defer detail to SECURITY.md
  - `docs/src/motivation.md:30-33` - add one supporting sentence after `> [!TIP]` claim
  - `docs/src/motivation.md:36` - append 2-3 lines on single-source-of-truth evolution (schema versioning + `acorn format` migration)
  - `docs/src/motivation.md:43` - broken `[^bucket]: ... [Buckets](concepts/index.html#-buckets)` -> `[Buckets](concepts/README.md#buckets)`
  - `docs/src/getting-started.md:3-4` - reframe `> [!IMPORTANT]` from "You do not need to install" to "Local install is optional" + add bridge before `## Installation`
  - `docs/src/getting-started.md:10-19` table - keep as TL;DR; add 1-line `> Use this if...` selector under each `### Install with...` / `### Run Inside...` / `### Download...` / `### Install from source`
- **Create:** `None`
- **Delete:** `None`

# Dependencies

- Prerequisite: `make book` anchor verification for `### 🪣 Buckets` -> `#buckets`. Gates `motivation.md:43`.
- No code dependency. Orthogonal to analyzer/portability/feature lanes per `.wiki/parallel-execution.md`. Docs lane isolated.

# Evidence

- `docs/src/overview.md:40` `... Linux machine` missing terminal period.
- `docs/src/overview.md:47` `[^acronym]: ACORN stands for "Accessible Content Optimization for Research Needs` missing closing `"`.
- `docs/src/overview.md:63` `[role guide](learn/index.html)` -> source is `docs/src/learn/README.md` (`SUMMARY.md:6`), mdbook maps `README.md` <-> `index.html`.
- `docs/src/motivation.md:43` `[Buckets](concepts/index.html#-buckets)` -> real file is `concepts/README.md:25` `### 🪣 Buckets`.
- `docs/src/motivation.md:13` absolute claim `there is no system we know of that employs projects as the primitive`.
- `docs/src/motivation.md:21` conjunction on second bullet; `docs/src/motivation.md:27-28` risks ordered funding/HR before security.
- `SECURITY.md:32-40` holds canonical security considerations; book defers there.

# Recommendation

Phase A (mechanical): add period, fix footnote quote, fix two links, move conjunction.
Phase B (editorial): soften claim to `few widely adopted systems use the project itself as the primitive`, reorder risks with `see SECURITY.md`, add TIP justification (flat files, open schema), add lifecycle paragraph (`acorn format`, `acorn check`, pinned CI version, `CHANGELOG.md`).
Phase C (UX): reframe IMPORTANT to `Local install is optional` + `If you are running ACORN locally...`, add per-method `> Use this if...` callouts.

# Validation

- [ ] `make book` / `mdbook build docs` zero warnings; click-through role guide, buckets footnote, `learn/index.html` and `concepts/index.html#buckets`.
- [ ] `grep -n "Research Activity Data" docs/src/overview.md docs/src/motivation.md` shows Title Case only in headings.
- [ ] No new vale errors.

# Acceptance criteria

- Five reviewer decisions applied: prose lower-case / heading Title Case, footnote retained+fixed, security deferred via `SECURITY.md` link, claim softened, plan filed.
- Four broken/punctuation fixes closed; risk section reordered; TIP and single-source-of-truth each have >=1 supporting sentence; Getting Started inconsistency and choice overload addressed.
