# Refactor Plans

## Next

- [Hugging Face request readiness](api/huggingface-request-readiness.md) - Enforce repository and download inputs through operation-specific request types.
- [OOXML interpolation and slide-copy hardening](core/ooxml-interpolation-and-slide-copy.md) - Finish parser-backed and structured OOXML mutations.
- [OOXML slide-list fuzzing](core/ooxml-fuzzing.md) - Exercise PowerPoint XML transforms against malformed and adversarial input.
- [Shared export context](core/export-context.md) - Extract export fields once for every renderer.
- [Isolate std-only modules](portability/isolate-std-only-modules.md) - Complete the portable-domain boundary.

## In Progress

- [Refactor initiative](initiative.md) - Overall goals, phases, risks, scope, and completion criteria.

## Planned

- [Crate boundary architecture](crate-boundary-architecture.md) - Split acorn-lib into portable domain crates, capability adapters, services, and a compatibility facade.
- [GitLab claimed-operation capabilities](api/gitlab-claimed-operation-capabilities.md) - Consume queue claims through guarded single-settlement transitions.
- [Core refactors](core/) - Export, CLI, schema, path, interpolation, and binding boundaries.
- [Portability](portability/) - no-std and WASM capability work.
- [Analyzer](analyzer/) - Composable analysis and validation abstractions.
- [API](api/) - Shared API-client contracts and reusable provider utilities.
- [acorn-cmd extraction](acorn-cmd-extraction.md) - Extract command macros and runtime helpers into a standalone crate.
