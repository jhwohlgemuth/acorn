# Portability Plans

## Coordination

- [Crate boundary architecture](../crate-boundary-architecture.md) - Coordinates package ownership, feature graphs, direct consumers, WASM composition, and future extraction.

## In Progress

- [Dependency capabilities and macOS unwind size](dependency-features-and-macos-size.md) - Narrow host graphs, preserve independent portability checks, and use measured profile/dependency work to remove the Darwin compact-unwind warning.

The implemented portability gate covers minimal, alloc, analyzer-only native and WASM, native analysis, and the current `acorn-wasm` package. Add a new atomic plan only when a concrete browser workflow or alternate host implementation proves a narrow adapter contract.
