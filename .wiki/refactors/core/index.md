# Core Refactor Plans

## Next

- [Schema I/O validation handoff](schema-io-validation-handoff.md) - Split portable codecs from std path/file adapters and unblock `acorn-schema`.

## In Progress

- [Remote create container targets](remote-create-container-targets.md) - Add preflight, rollback, cleanup, and live validation for remote container creation.
- [OOXML interpolation and slide-copy hardening](ooxml-interpolation-and-slide-copy.md) - Replace remaining fragile OOXML mutations.

## Planned

- [CLI startup separation](cli-startup-separation.md) - Reduce `main` to orchestration.
- [Shared interpolation primitives](interpolation-primitives.md) - Unify string and list placeholder behavior.
- [Python binding deduplication](python-binding-deduplication.md) - Reduce repetitive PID wrappers.
- [OOXML slide-list fuzzing](ooxml-fuzzing.md) - Exercise PowerPoint XML transforms against malformed and adversarial input.
- [Shared export context](export-context.md) - Centralize renderer input extraction.
- [Research activity lifecycle, credit, license, and award fields](research-activity-lifecycle-credit-fields.md) - Add typed schema targets for provider enrichment that does not belong in notes.
- [CLI parse and execution models](cli-parse-execution-models.md) - Separate clap parsing from execution options.
- [Safe path abstraction](safe-path.md) - Centralize normalization and containment.
- [CLI error boundary](cli-error-boundary.md) - Map internal errors to exit codes in one place.
