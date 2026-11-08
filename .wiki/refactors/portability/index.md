# Portability Plans

## Coordination

- [Crate boundary architecture](../crate-boundary-architecture.md) - Coordinates package ownership, feature graphs, direct consumers, WASM composition, and future extraction.
- [Rust browser and Tauri desktop workspace](../../features/acorn-browser-workspace/index.md) - Supplies the concrete static-browser and alternate-host consumer required to extend the portability matrix: `acorn-web` composes portable owners with `acorn-wasm`, while `acorn-desktop` selects only bounded `acorn-host` filesystem capability behind Tauri dialogs.

The implemented portability gate covers minimal, alloc, analyzer-only native and WASM, native analysis, explicit host capabilities, the current `acorn-wasm` package, and the Darwin debug/test unwind limit. The browser/Tauri workspace is the active atomic plan that proves the next concrete browser workflow and alternate host composition; its ABW-01 and ABW-12 phases must extend this record with accepted dependency and feature evidence rather than bypassing the established gates.
