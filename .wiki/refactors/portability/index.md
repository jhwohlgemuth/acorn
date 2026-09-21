# Portability Plans

## Coordination

- [Portability roadmap](roadmap.md) - Coordinate the active critical path and feature matrix; schedule its atomic child plans rather than the roadmap itself.

## In Progress

- [Dependency feature defaults](dependency-feature-defaults.md) - Narrow host capability graphs one independently compiling boundary at a time.

## Planned

- [macOS compact-unwind size](macos-compact-unwind-size.md) - Reduce Darwin debug and test unwind metadata without suppressing linker diagnostics.

The implemented portability gate covers minimal, alloc, analyzer-only native and WASM, native analysis, and the current `acorn-wasm` package. Add a new atomic plan only when a concrete browser workflow or alternate host implementation proves a narrow adapter contract.
