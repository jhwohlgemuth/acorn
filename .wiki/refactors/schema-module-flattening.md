---
type: Plan
title: "Flatten the acorn-schema module hierarchy"
description: "Remove the redundant public schema namespace, flatten the source tree, and migrate first-party and Python binding consumers."
difficulty: medium
impact: high
package: [workspace, acorn-schema, acorn-lib, acorn-cli, acorn-tui, acorn-wasm, acorn-py]
tags: [refactor, schema, api, rust, python]
status: in-progress
---

# Goal

Expose schema modules and shared types directly from `acorn_schema`, removing the redundant `acorn_schema::schema` namespace and the matching `src/schema/` source layer.

# Current evidence

- `acorn-schema` previously declared one public `schema` module containing its domain modules and shared types.
- First-party Rust consumers and `acorn-py` imported those types through `acorn_schema::schema`.
- `acorn-py` pins `acorn-lib` and `acorn-schema` to one immutable public ACORN Git revision while exposing a separate, stable Python hierarchy under `acorn.schema`.

# Project code boundaries

- **Update:** `crates/acorn-schema/src/lib.rs`, schema modules moved directly under `crates/acorn-schema/src/`, first-party Rust consumers, documentation and planning paths, and `/Users/o9w/dev/re/acorn-py/{Cargo.toml,Cargo.lock,src/lib.rs}` plus dependency documentation.
- **Create:** `crates/acorn-schema/src/types.rs` for private implementation of shared types re-exported at the crate root.
- **Delete:** `crates/acorn-schema/src/schema/` and the public `acorn_schema::schema` namespace.

# Recommendation

Move every schema leaf one level upward, declare public modules from the crate root, and re-export shared types from a private `types` module. Migrate all Rust consumers without a compatibility alias. Preserve `acorn-py`'s Python-visible `acorn.schema`, `acorn.schema.pid`, and `acorn.schema.validate` modules; only its Rust imports and immutable dependency revision change.

# Dependencies and landing

The ACORN change lands first. After its commit is publicly reachable, `acorn-py` pins both ACORN crates to that exact revision, regenerates its lockfile, and validates both supported Python versions and its ABI3 wheel. No local path override is committed.

# Validation

- **Passed:** `make check-portable`, `make lint`, `cargo test -p acorn-schema`, and the full workspace suite with `cargo nextest run --test-threads 1` (1,521 passed, 14 skipped).
- **Passed:** `make cover` (67.51%, 21,247/31,473 lines, +0.61%) and stale-path/snapshot-artifact scans.
- **Blocked on landing order:** run `make check`, `make test`, and `make wheel-smoke` in `acorn-py` after this ACORN change has a publicly reachable immutable Git revision.
- `cargo package -p acorn-schema --allow-dirty` remains blocked by the existing unpublished `acorn-core` 0.2.0 registry dependency.

# Acceptance criteria

- Root paths such as `acorn_schema::pid::DOI`, `acorn_schema::research_activity::ResearchActivity`, and `acorn_schema::OneOrMany` compile.
- `acorn_schema::schema` no longer exists and no first-party Rust consumer references it.
- Serialization, validation, generated schemas, and supported feature combinations remain unchanged.
- `acorn-py` builds from its pinned public dependencies while its existing Python import paths and tests remain unchanged.
