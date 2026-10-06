# Portability Plans

## Coordination

- [Crate boundary architecture](../crate-boundary-architecture.md) - Coordinates package ownership, feature graphs, direct consumers, WASM composition, and future extraction.

The implemented portability gate covers minimal, alloc, analyzer-only native and WASM, native analysis, explicit host capabilities, the current `acorn-wasm` package, and the Darwin debug/test unwind limit. Add a new atomic plan only when a concrete browser workflow or alternate host implementation proves a narrow adapter contract.
