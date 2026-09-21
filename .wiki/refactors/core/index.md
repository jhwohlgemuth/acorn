# Core Refactor Plans

## In Progress

- [Remote create container targets](remote-create-container-targets.md) - Add preflight, rollback, cleanup, and live validation for remote container creation.
- [OOXML interpolation and slide-copy hardening](ooxml-interpolation-and-slide-copy.md) - Replace remaining fragile OOXML mutations.
- [Shared export context](export-context.md) - Complete the shared renderer projection started by common image and draft-fallback selection.

## Planned

- [CLI startup separation](cli-startup-separation.md) - Reduce `main` to orchestration.
- [Shared interpolation primitives](interpolation-primitives.md) - Unify string and list placeholder behavior.
- [OOXML slide-list fuzzing](ooxml-fuzzing.md) - Exercise PowerPoint XML transforms against malformed and adversarial input.
- [Structured research-activity enrichment](research-activity-enrichment-integration.md) - Map provider and logbook evidence into the existing typed activity fields with deterministic merge behavior.
- [CLI parse and execution models](cli-parse-execution-models.md) - Separate clap parsing from execution options.
- [CLI error boundary](cli-error-boundary.md) - Map internal errors to exit codes in one place.
