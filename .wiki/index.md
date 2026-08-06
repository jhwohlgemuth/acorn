---
okf_version: "0.1"
---

# ACORN Planning Wiki

This directory is the authoritative home for ACORN implementation plans. It is an [Open Knowledge Format](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md) bundle.

## Contents

- [Refactor plans](refactors/) - Reliability, maintainability, extensibility, and portability work.
- [Migration plans](migrations/) - Changes that replace an existing dependency, architecture, or workflow.
- [Testing plans](testing/) - CLI test coverage, integration testing, and quality assurance work.

## Adding a plan

1. Create one semantically named Markdown file for one atomic topic.
2. Add `type: Plan`, `title`, `description`, `difficulty`, `impact`, a non-empty `package` list, `tags`, and `status` to its YAML frontmatter.
3. Use `low`, `medium`, `high`, `mixed`, or `not-assessed` for difficulty; use `low`, `medium`, `high`, `critical`, `mixed`, or `not-assessed` for impact.
4. Use `planned` or `in-progress` for status. Add `priority: next` when the plan is an immediate candidate.
5. Document the goal, current evidence, recommendation, validation, and acceptance criteria. Add citations for externally sourced claims.
6. Link related concepts with relative Markdown links and add the new concept to the nearest directory index.

## Maintaining plans

1. Verify evidence and status against the current code before editing a plan.
2. Update frontmatter and body together when scope, package, priority, or status changes.
3. Delete finished plans and remove their index entries.
4. Consolidate overlapping proposals into one canonical concept instead of duplicating them.
5. Keep external claims cited and verify that internal links resolve.
6. Preserve OKF frontmatter and the reserved `index.md` and `log.md` filename conventions.
7. After changes, check frontmatter, required fields, links, legacy references, and whitespace.
