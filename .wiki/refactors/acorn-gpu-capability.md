---
type: Plan
title: "Add provider-neutral GPU telemetry contract"
description: "Add portable GPU inventory and telemetry types to ACORN while leaving probing and run lifecycle policy to downstream consumers."
difficulty: medium
impact: high
package: [acorn-core, acorn-schema]
tags: [gpu, hardware, portability, schema, telemetry]
status: planned
---

# Goal

Make ACORN the owner of a replay-safe, provider-neutral contract for observed GPU identity, capacity, and telemetry. Keep vendor commands, monitoring threads, journal persistence, aggregation, degradation policy, and reporting outside the portable crates.

# Current evidence

- `acorn_schema::hardware::Resource::GPU` describes a requested resource, not an observed device with a stable provider identifier and local enumeration index.
- `acorn_core::time::Milliseconds` already provides a portable, serialized, non-negative millisecond quantity, but its documentation currently describes only timeout boundaries.
- ACORN has no observed GPU inventory or telemetry types. Xylem therefore defines local `GpuDevice` and `GpuReading` values in its run journal and uses `nvidia-smi` as a downstream adapter.
- Xylem's first NVIDIA monitoring implementation landed in commit [`60273f2`](https://code.ornl.gov/research-enablement/xylem/-/commit/60273f2498aa9d4e5005bb3f7b7be7909c88e19c) and demonstrates the concrete consumer without making NVIDIA behavior part of ACORN.
- The legacy `crates/acorn-schema/src/hardware/tests` module is disabled with `#[cfg(any())]`; new GPU coverage must live in an active module.

# Project code boundaries

- **Update:** `crates/acorn-core/src/time.rs` to document `Milliseconds` as a general serialized millisecond quantity, and `crates/acorn-schema/src/hardware/mod.rs` to expose the GPU module and its public types.
- **Create:** `crates/acorn-schema/src/hardware/gpu.rs` and `crates/acorn-schema/src/hardware/gpu/tests.rs`.
- **Delete:** None.
- **Do not change:** `Resource::GPU`, crate manifests, `Cargo.lock`, `acorn-host`, CLI/configuration surfaces, vendor probes, or Xylem in this slice.

# Dependencies and ordering

This is an independent portable-schema leaf in the schema-features lane. It owns the `hardware/mod.rs` module-index lock and the narrow `acorn-core/src/time.rs` documentation change while active, but it does not take the workspace-graph, CLI, configuration, host-adapter, or CI/build locks. A downstream Xylem migration follows a released or pinned ACORN revision and owns the journal schema transition from NVIDIA-specific `uuid` and required `index` fields.

# Recommendation

1. Add `GpuDevice` with an opaque `identifier`, optional provider-local `index`, non-empty display `name`, and optional `memory_total_mib`.
2. Add transparent `GpuInventory` collection serialization with validated construction, borrowed device access, owned extraction, non-empty validation, and deterministic uniqueness checks for identifiers and present indices. Multiple absent indices remain valid.
3. Add `GpuReading` with an opaque `identifier`, optional `memory_used_mib`, and optional integer `utilization_percent`. Missing metrics remain distinct from zero.
4. Add `GpuSample` as one atomic ordered reading batch with `timestamp: Milliseconds`. Document the timestamp as Unix epoch milliseconds; zero remains the valid epoch.
5. Derive `Clone`, `Debug`, `Deserialize`, `Eq`, `JsonSchema`, `PartialEq`, and `Serialize` for the public contract. Reject unknown record fields and keep snake_case wire names without a compatibility alias for `uuid`.
6. Implement `Validate` for every type. Reject blank identifiers and names, utilization above 100, empty collections, and duplicate identifiers or present indices with stable `nonempty`, `range`, and `unique` diagnostics in input order.
7. Preserve caller ordering and exact case-sensitive identifiers. Do not sort, trim, normalize, compare used memory with inventory capacity, or require a sample to match an inventory; those are adapter and run-policy concerns.
8. Re-export `GpuDevice`, `GpuInventory`, `GpuReading`, and `GpuSample` from `acorn_schema::hardware` without adding a crate-root facade.

# Validation

- Add active unit tests for exact JSON round trips, unknown-field rejection, constructors and accessors, preserved ordering, valid missing optional values, blank identifiers and names, utilization bounds, empty collections, duplicate identifiers, duplicate present indices, and repeated absent indices.
- Run `cargo test -p acorn-core --lib` and `cargo test -p acorn-schema --lib gpu_`.
- Check `acorn-schema` with default features, no default features, `core`, and `alloc`.
- Check the `core` and `alloc` feature paths for `wasm32-unknown-unknown`.
- Run focused Clippy for `acorn-core` and `acorn-schema` with warnings denied, then formatting and scoped `git diff --check` for the declared write set.

# Acceptance criteria

- `acorn_schema::hardware` publicly owns the four provider-neutral GPU contract types with documented MiB, percentage, and millisecond semantics.
- JSON round trips preserve opaque identifiers, ordered readings, null optional metrics, and the numeric `Milliseconds` timestamp representation.
- Validation reports all deterministic local and collection failures while accepting unavailable metrics and absent local indices.
- `Resource::GPU` remains a requested-resource declaration, and ACORN gains no vendor command, SDK, thread, journal, or reporting dependency.
- Native and WebAssembly portable feature checks pass without manifest or lockfile changes.

# Related plans

- [Crate boundary architecture](crate-boundary-architecture.md)
- [Dependency feature defaults](portability/dependency-feature-defaults.md)
- [Portability roadmap](portability/roadmap.md)
