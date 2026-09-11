---
type: Plan
title: "Integrate With derive across workspace"
description: "Adopt acorn_macros::With in schema/lib/cli where immutable with_* pipelines replace manual clone/copy patterns."
difficulty: low
impact: low
package: [acorn-schema, acorn-core, acorn-lib]
tags: [refactor, macros, ergonomics, follow-up]
status: planned
---

# Goal

With the `acorn_macros::With` derive complete, incrementally adopt `With` on high-value structs where immutable `with_*` pipelines improve over manual `clone`/`copy()`/`format(self)` at `crates/acorn-schema/src/research_activity/mod.rs:811,836`. This follow-up uses schema locks and remains separate from the completed macro implementation.

# Current evidence

* `acorn_macros::With` is verified with `cargo test -p acorn-macros` (10 pass tests + 5 trybuild fail cases) and provides `with_*`/`with_0` consuming `self -> Self` with `impl Into<T>`.
* Candidate structs live under `crates/acorn-schema/src/research_activity/mod.rs:114` (`ResearchActivity`), `:192` (`ResearchActivityMetadata`), `:489` (`Sections`), `:93` (`Research`), and `crates/acorn-core/src/types.rs:814` (`OneOrMany` not applicable — enum), plus selected `acorn-lib` row/config structs.
* `crates/acorn-schema/src/` is a shared schema ownership lock with metadata alignment, dPID, ISNI, OCP, ZON per `plan-network.md:116`.

# Project code boundaries

* **Update:** `crates/acorn-schema/src/research_activity/mod.rs` (add `#[derive(With)]` to `ResearchActivity`, `ResearchActivityMetadata`, `Sections`, `Research` where approved), `crates/acorn-schema/src/lib.rs` or `crates/acorn-core/src/prelude.rs` if a `pub use acorn_macros::With` re-export is desired, `crates/acorn-lib/src/io/database/` row structs (optional), docs/examples (`docs/src/**`, `README.md` snippets).
* **Create:** None (no new crates). Add `tests/fixtures/` examples for `with_*` chaining if illustrative.
* **Delete:** None (retain existing `bon::Builder` construction; `With` is additive, not a replacement).

# Dependencies

* The completed `acorn_macros::With` derive is the implemented prerequisite.
* Serializes on `crates/acorn-schema/src/` shared lock; must not run concurrently with `rad-metadata-standards-alignment`, `dPID`, `ISNI`, `OCP` slices that touch `research_activity/**`.
* `crate-boundary-architecture` coordination only.

# Recommendation

| Step | Action | Verification |
|---|---|---|
| 1 | Pilot PR: add `With` to `Sections` + `Research` (smallest structs), keep `Builder` — demonstrate `Sections { mission: ... }.with_mission("...")` chaining | `cargo test -p acorn-schema` + `cargo check --no-default-features` |
| 2 | Expand to `ResearchActivity` + `ResearchActivityMetadata` if pilot approved; deprecate manual `copy()` where `with_*` chain is clearer | `cargo nextest run -p acorn-schema` |
| 3 | Optional re-export `pub use acorn_macros::With` via `acorn-schema` prelude (only if ergonomics warrant) | `make check-wasm` |
| 4 | Update book/docs: show `with_*` alongside `bon` builder pattern (construction vs. post-construction update) | `make book` |

Preserve `bon::Builder` for construction (`ResearchActivity::init().title(...).build()`); `With` is for post-construction functional updates (`activity.with_title("...").with_subtitle(...)`). Do not bulk-migrate all structs in one PR.

# Validation

* `cargo check --workspace --all-targets` + `cargo check --no-default-features`
* `make lint` + `make test` (schema + macros)
* `make book` if docs changed; snapshot tests unaffected

# Risks

| Risk | Mitigation |
|---|---|
| Bulk adoption touches many schema files and conflicts with metadata-alignment lane | Pilot small structs first, serialize on schema lock |
| Duplicate `with_*` names if struct already has manual `with_*` | Uses `#[with(skip)]` or renames; compiler catches duplicate |

# Acceptance criteria

1. Pilot structs derive `With` and pass new unit tests demonstrating `with_*`/`into` chaining.
2. No public API break beyond additive `with_*` methods; existing `Builder` still works.
3. Follow-up plan deleted after pilot lands; docs updated if re-export added.

# Related plans

* [RAD metadata standards alignment](../features/rad-metadata-standards-alignment.md) — shares schema lock, serialize.
* [Crate boundary architecture](crate-boundary-architecture.md)
