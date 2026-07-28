# Core Refactor Plans

## Next

- [OOXML interpolation and slide-copy hardening](ooxml-interpolation-and-slide-copy.md) - Replace remaining fragile OOXML mutations.
- [OOXML slide-list fuzzing](ooxml-fuzzing.md) - Exercise PowerPoint XML transforms against malformed and adversarial input.
- [Shared export context](export-context.md) - Centralize renderer input extraction.

## In Progress

- [Remote create container targets](remote-create-container-targets.md) - Automated implementation is complete; live SSH Docker smoke testing remains.

## Planned

- [CLI startup separation](cli-startup-separation.md) - Reduce `main` to orchestration.
- [Shared interpolation primitives](interpolation-primitives.md) - Unify string and list placeholder behavior.
- [Python binding deduplication](python-binding-deduplication.md) - Reduce repetitive PID wrappers.
- [Schema I/O validation handoff](schema-io-validation-handoff.md) - Centralize strict format validation.
- [CLI parse and execution models](cli-parse-execution-models.md) - Separate clap parsing from execution options.
- [Safe path abstraction](safe-path.md) - Centralize normalization and containment.
- [CLI error boundary](cli-error-boundary.md) - Map internal errors to exit codes in one place.

## Completed

- [ASPECT rose chart export](aspect-rose-chart-export.md) - Render the four scalar ASPECT attributes once for PDF and PowerPoint.
