# Portability Plans

## Next

- [Isolate std-only modules](isolate-std-only-modules.md) - Complete the separation between portable domain logic and host services.
- [Utility module partitioning](utility-module-partitioning.md) - Move portable implementations out of the mixed root module.
- [Dependency feature defaults](dependency-feature-defaults.md) - Finish classifying the portable dependency tree.

## Planned

- [WASM feature path](wasm-feature-path.md) - Complete the existing WASM capability path.
- [Capability preludes](capability-preludes.md) - Make core, alloc, and std imports explicit.
- [Host abstraction traits](host-abstraction-traits.md) - Abstract files, HTTP, time, entropy, and processes.
- [macOS compact-unwind size](macos-compact-unwind-size.md) - Reduce Darwin debug and test unwind metadata without suppressing linker diagnostics.
- [no-std and WASM CI](no-std-wasm-ci.md) - Make portable builds a quality gate.
- [Portability roadmap](roadmap.md) - Coordinate features, implementation stages, and dependency isolation.
