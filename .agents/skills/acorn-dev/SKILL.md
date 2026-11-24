---
name: acorn-dev
description: ACORN-specific Rust implementation recipes and conventions. Use when adding or changing schema fields, CLI commands, analyzers, or export code; when fixing clippy::unwrap_used warnings; when adding eserde multi-error deserialization; or when unsure where ACORN code, tests, or fixtures live.
---

# ACORN development

Load only the reference the current task needs.

| Task | Read |
| --- | --- |
| Schema field, CLI command, analyzer, export/PowerPoint fix, test commands, debugging, finding code | [tasks.md](references/tasks.md) |
| `clippy::unwrap_used`, `color-eyre` errors, `eserde` multi-error JSON entry points | [errors.md](references/errors.md) |
| Logging and `Label` output, feature flags, `rayon`, `bon`, doc standards, common pitfalls | [conventions.md](references/conventions.md) |

The always-on rules live in the repository's root `AGENTS.md`; these references explain how to satisfy them.
