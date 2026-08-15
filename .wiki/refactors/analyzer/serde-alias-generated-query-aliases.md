---
type: Plan
title: "Generate query-path aliases from serde metadata"
description: "Eliminate hand-maintained locator alias rewrites by generating alias mappings from schema serde attributes."
difficulty: medium
impact: medium
package: [acorn-lib]
tags: [refactor, analyzer, serde, codegen]
status: planned
---

# Goal

Replace manual locator alias rewrite chains with one generated alias map derived from serde metadata so schema alias additions do not require analyzer-side edits.

# Evidence

- `crates/acorn-lib/src/analyzer/error.rs` and `crates/acorn-lib/src/analyzer/check.rs` both consume path aliases for locator-to-source resolution.
- Recent consolidation reduced duplication, but updates are still manual when new serde aliases are introduced in schema types.

# Recommendation

Add a build-time generation step that extracts serde alias metadata from schema types and emits a deterministic Rust alias map consumed by analyzer query resolution.

Planned shape:

1. Add a generator in the `acorn-lib` build flow (or `xtask`) that emits `OUT_DIR` Rust code for alias mappings.
2. Normalize mappings at segment level (field aliases and known path rewrites).
3. Expose one lookup function for analyzer query builders.
4. Replace ad-hoc and static rewrite tables with generated output.

# Validation

1. Build `acorn-lib` with generated alias output and verify no manual alias table is required for existing behavior.
2. Add tests that assert known aliases resolve to expected canonical/alternate paths.
3. Add a regression test: introduce a new serde alias in fixture schema and verify query path resolution updates without analyzer code edits.

# Acceptance criteria

1. Analyzer query alias resolution uses one generated source of truth.
2. No duplicate manual alias rewrite chains remain in analyzer modules.
3. Adding a serde alias to covered schema fields requires no analyzer code changes.
4. Generated output is deterministic and passes CI without manual post-processing.
