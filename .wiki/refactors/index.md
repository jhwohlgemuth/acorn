# Refactor Plans

## Next

- [OOXML interpolation and slide-copy hardening](core/ooxml-interpolation-and-slide-copy.md) - Finish parser-backed and structured OOXML mutations.
- [OOXML slide-list fuzzing](core/ooxml-fuzzing.md) - Exercise PowerPoint XML transforms against malformed and adversarial input.
- [Shared export context](core/export-context.md) - Extract export fields once for every renderer.
- [Isolate std-only modules](portability/isolate-std-only-modules.md) - Complete the portable-domain boundary.

## In Progress

- [Refactor initiative](initiative.md) - Overall goals, phases, risks, scope, and completion criteria.

## Planned

- [Core refactors](core/) - Export, CLI, schema, path, interpolation, and binding boundaries.
- [Portability](portability/) - no-std and WASM capability work.
- [Analyzer](analyzer/) - Composable analysis and validation abstractions.
- [API](api/) - Shared API-client contracts and reusable provider utilities.
- [acorn-cmd extraction](acorn-cmd-extraction.md) - Extract command macros and runtime helpers into a standalone crate.

## Completed

No retained plans are currently complete.
