# Portability Plans

## Coordination

- [Portability roadmap](roadmap.md) - Coordinate the active critical path and feature matrix; schedule its atomic child plans rather than the roadmap itself.

## In Progress

- [Dependency feature defaults](dependency-feature-defaults.md) - Narrow host capability graphs one independently compiling boundary at a time.
- [no-std and WASM CI](no-std-wasm-ci.md) - Enforce current portable graphs and expand the gate as packages appear.

## Planned

- [Downstream host support](downstream-host-support.md) - Extract narrow `acorn-host` capabilities proven by Xylem without enabling `acorn-lib` or analysis.
- [WASM browser services](wasm-feature-path.md) - Extend the minimal in-memory package with concrete browser adapters after service ports are proven.
- [Host abstraction traits](host-abstraction-traits.md) - Abstract files, HTTP, time, entropy, and processes.
- [macOS compact-unwind size](macos-compact-unwind-size.md) - Reduce Darwin debug and test unwind metadata without suppressing linker diagnostics.
