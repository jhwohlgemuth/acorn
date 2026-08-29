---
type: Plan
title: "Tighten schema I/O validation handoff"
description: "Make CLI entry points rely consistently on library-owned strict format validation."
difficulty: high
impact: critical
package: [acorn-cli, acorn-lib, acorn-schema, acorn-host]
tags: [refactor, schema, validation, io]
status: planned
priority: next
---

# Current evidence

- `crates/acorn-lib/src/io/mod.rs`
- `crates/acorn-lib/src/schema/research_activity/mod.rs`
- `crates/acorn-cli/src/commands/export/mod.rs`
- Phase 1.1 portable validation is complete; this handoff is crate-boundary architecture Phase 1.2.
- The 2026-08-28 feature-matrix check fails before path I/O is reached: `schema/research_activity/output.rs` imports the schema-facing `License` enum from std-gated `io/mod.rs`. Minimal, alloc, and WASM builds all fail with unresolved `crate::io`.
- Path/file implementations currently live inside `schema/research_activity/mod.rs`, `schema/standard/{cff,datacite,dcat,huwise,invenio,text}.rs`, and `schema/pid/raid/mod.rs`, all coupled to `io::InputOutput`, `read_file`, `write_file`, or `PathBuf`.
- String/byte parsing and serialization are mixed into those implementations, so moving whole modules would incorrectly move portable codecs into the host layer.

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/mod.rs`; `src/schema/mod.rs`; `src/schema/research_activity/{mod,output}.rs`; `src/schema/standard/{cff,datacite,dcat,huwise,invenio,text}.rs`; `src/schema/pid/raid/mod.rs`; CLI consumers under `crates/acorn-cli/src/commands/{check,export,format,import,link}/`; adjacent tests and fixtures.
- **Create:** portable string/byte codec traits or free functions under the schema boundary; std-only path/file adapters under `io/` during Phase 1, later owned by `acorn-host`; compatibility re-exports for `acorn::io::InputOutput`.
- **Delete:** schema-to-`io` imports and schema-owned path/file method implementations after adapters and compatibility tests land. Do not delete string/byte codecs or the legacy public trait path.

# Dependencies

- **Prerequisite:** Phase 1.1 portable validation and current minimal/alloc/WASM checks, already present.
- **Blocks:** creation of `acorn-schema`, the clean analyzer boundary, and meaningful host traits.
- **Coordinate with:** [std-only isolation](../portability/isolate-std-only-modules.md) and [crate boundary architecture](../crate-boundary-architecture.md).

# Parallel execution

This is the sole Phase 1.2 integration owner. Its first atomic slice claims `schema/research_activity/output.rs`, the current `License` definition and compatibility re-export, focused tests, and any required schema/I/O module indexes. It must not simultaneously run with another schema-shape, schema-codec, `io/mod.rs`, library-root, or manifest change. Typed scoring, remote-create leaf work, Python binding deduplication, OOXML leaf work, and a provider-specific API leaf are orthogonal under the [parallel execution guide](../../parallel-execution.md).

# Recommendation

First restore the portable matrix by moving the data-only `License` value into schema/core ownership and re-exporting it through `acorn::io` for compatibility. Then split serialization from materialization: schema-owned codecs parse/serialize strings or bytes and return portable errors; std adapters read/write paths and invoke those codecs plus validation. Make CLI consumers use the adapter boundary consistently while retaining the existing `InputOutput` API as a compatibility shim.

# Validation

Run focused schema and CLI tests, then `cargo check -p acorn-lib --no-default-features`, `cargo check -p acorn-lib --no-default-features --features alloc`, `make wasm`, `make lint`, and `make test`. Add compatibility tests proving representative ResearchActivity, CFF, DataCite, DCAT, HuWISE, Invenio, text/document, and RAiD reads preserve behavior.

# Acceptance criteria

Minimal, alloc, and WASM builds pass; schema no longer imports `io`; and user-facing reads apply the same format and schema validation regardless of the invoking command.

# Related plans

- [Plan dependency network](../../plan-network.md)
