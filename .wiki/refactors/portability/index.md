# Portability Plans

## Next

- [Isolate std-only modules](isolate-std-only-modules.md) - Complete the separation between portable domain logic and host services.

## Planned

- [Core error surface](core-error-surface.md) - Replace rich host reports in portable builds.
- [WASM feature path](wasm-feature-path.md) - Complete the existing WASM capability path.
- [Capability preludes](capability-preludes.md) - Make core, alloc, and std imports explicit.
- [Utility module partitioning](utility-module-partitioning.md) - Separate pure helpers from host presentation.
- [Host abstraction traits](host-abstraction-traits.md) - Abstract files, HTTP, time, entropy, and processes.
- [Dependency feature defaults](dependency-feature-defaults.md) - Stop workspace defaults from leaking std.
- [macOS compact-unwind size](macos-compact-unwind-size.md) - Reduce Darwin debug and test unwind metadata without suppressing linker diagnostics.
- [no-std and WASM CI](no-std-wasm-ci.md) - Make portable builds a quality gate.
- [Portability roadmap](roadmap.md) - Coordinate features, implementation stages, and dependency isolation.
