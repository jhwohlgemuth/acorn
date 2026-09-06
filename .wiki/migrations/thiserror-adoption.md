---
type: Plan
title: "Adopt thiserror for typed error contracts"
description: "Standardize owned Rust error types on thiserror while retaining color-eyre at application and host boundaries."
difficulty: medium
impact: medium
package: [workspace, acorn-lib, acorn-cli]
tags: [rust, errors, thiserror, diagnostics, portability]
status: planned
---

# Project code boundaries

- **Update:** workspace and crate manifests; owned error definitions beginning at `crates/acorn-lib/src/error.rs` and boundary-specific errors in analyzer, API, schema, store, export, host, service, and interfaces.
- **Create:** one focused error module per extracted crate when needed; conversion tests at facade and application boundaries.
- **Delete:** handwritten `Display`/`Error` implementations replaced mechanically; never delete `color-eyre` from CLI/host reporting boundaries in this migration.

# Dependencies

This migration is incremental and must not block portable extraction. Every new crate in the [crate boundary architecture](../refactors/crate-boundary-architecture.md) starts with owned portable errors, while existing modules convert only when touched for an accepted boundary.

# Goal

Use `thiserror` to define ACORN-owned error types with consistent `Display`, `Error`, `source`, and conversion behavior. Preserve `color-eyre` reports for executable, I/O, API, and orchestration boundaries where callers need rich diagnostic context rather than a closed error enum.

This is a targeted error-contract migration, not a workspace-wide replacement of every `Result` error type.

# Assessment

The migration is viable and worthwhile if it remains bounded to errors ACORN owns.

## Benefits

- Removes repetitive handwritten `Display` and `core::error::Error` implementations while keeping the same public Rust traits.
- Makes source chains and infallible `From` conversions explicit through `#[source]` and `#[from]`.
- Keeps variant-specific data available to library callers instead of flattening failures into strings or opaque reports too early.
- Supports the existing portable error boundary: `thiserror` 2 can derive errors without its `std` feature, and ACORN's Rust 1.95 minimum is above thiserror 2.0.18's Rust 1.68 minimum.
- Adds no new runtime error representation. The derive generates ordinary trait implementations, and thiserror is already resolved transitively in the current lockfile.
- Aligns with the crate-boundary plan's rule that pure functions return typed errors while composition roots and host adapters own presentation.

## Costs and risks

- The derive macro adds another direct workspace dependency and can modestly affect clean compile time even though its macro dependencies are already present.
- Blind use of `#[from]` can erase operation context, create overly broad implicit conversions, or make public enums harder to evolve.
- Blind use of `#[source]` can change observable error chains even when the displayed message is unchanged.
- Public enum changes remain semver-sensitive. Thiserror itself is implementation-only, but adding, removing, or restructuring public variants is not.
- Converting the 544 current `ApiResult` signatures or 703 `eyre!` construction sites would be high-churn work with little benefit and would weaken rich boundary diagnostics.
- Some `Result<_, String>` uses are intentional: proc-macro parsers need span-attached text, private transformation callbacks expose simple reasons, and application-only tools may not benefit from a closed taxonomy.

# Current evidence

- `crates/acorn-lib/src/error.rs` already defines the intended split: portable `AcornError`/`AcornResult` for owned domain failures and `ApiResult<T> = Result<T, color_eyre::eyre::Report>` for host boundaries.
- ACORN currently handwrites `Display` plus `Error` implementations for `AcornError`, `HuggingFaceError`, `OutputFailure`, `WebhookError`, `SyncError`, `ShardError`, and `ModelDownloadError`. `CrosswalkError` has a handwritten `Display` implementation but does not currently implement `Error`.
- The default and no-default dependency graphs already resolve `thiserror` 2.0.18 transitively. Making it direct therefore does not introduce a new crate version into the current lockfile.
- `acorn-cli` and `acorn-tui` use `color-eyre` as application error reporters. `acorn-macros` primarily returns `syn::Error`, and `xtask` prints internal `String` failures.
- Several DTOs are named `Error`, `ErrorResponse`, or `ErrorResponseFailure`; these represent remote response payloads and are not candidates for deriving `std::error::Error` merely because of their names.
- Thiserror documents that its derive is equivalent to handwritten `Error` implementations and does not become part of a library's public API.[^1]

# Recommendation

Adopt one workspace-managed `thiserror` 2 dependency with default features disabled, then add it directly only to packages that define typed ACORN errors. Initially those packages are `acorn-lib` and `acorn-cli`.

Keep the following taxonomy:

| Layer | Error policy |
|---|---|
| Portable domain and schema code | Small owned error structs or enums derived with `thiserror::Error` |
| Reusable adapters and services | Operation-specific typed errors when callers can recover or classify; otherwise convert to `ApiResult` at the host boundary |
| CLI and TUI composition roots | `color_eyre::Report` with contextual diagnostics |
| Proc macros | Continue using `syn::Error` and span-aware string messages |
| Remote error payload DTOs | Continue as serialization types; do not derive `Error` unless they become actual Rust failure contracts |
| Maintenance binaries such as `xtask` | Retain simple application errors until a concrete need for classification or source inspection appears |

Do not create one workspace-wide `AcornError` enum. Error types should remain capability-owned so that new crate extraction does not recreate coupling through a central catch-all type.

# Migration plan

## 1. Establish the dependency and invariants

1. Add `thiserror = { version = "2.0.18", default-features = false }` to `[workspace.dependencies]`.
2. Add `thiserror.workspace = true` only to `acorn-lib` and `acorn-cli` initially.
3. Record three invariants near `AcornResult` and `ApiResult`: typed errors belong to reusable operations, reports belong to host/application boundaries, and conversion must preserve actionable operation context.
4. Capture the current default, no-default, alloc, and WASM checks before changing error implementations.

## 2. Convert existing handwritten error implementations mechanically

Convert one capability at a time, preserving visibility, variants, fields, messages, and conversion behavior:

1. `ModelDownloadError` in `acorn-cli`.
2. Private leaf errors `OutputFailure`, `WebhookError`, `SyncError`, and `ShardError` in `acorn-lib`.
3. Public capability errors `HuggingFaceError` and `CrosswalkError`.
4. Convert `AcornError` last, after proving the derive remains portable in every supported feature configuration. Retain its constructors, accessors, `Clone`, equality traits, and `From<&str>`/`From<String>` behavior.

For every conversion:

- Add exact-message tests before removing the handwritten `Display` implementation.
- Preserve the current absence of a source unless the operation already semantically wraps an underlying error and callers benefit from traversing it.
- Add `#[from]` only when conversion is unambiguous and cannot discard operation-specific context.
- Treat any public variant or message change as separate work; do not hide API redesign inside the mechanical migration.

## 3. Audit weak error contracts without converting them wholesale

Classify each `Result<_, String>` occurrence as one of:

1. Intentional diagnostic text for a proc macro, parser callback, or private helper: retain it.
2. A public or cross-module operation whose caller needs to distinguish failures: introduce a local typed error.
3. An application boundary that only renders and exits: retain or convert directly to `color-eyre::Report` with context.

Prioritize public crosswalk and document-parsing paths where stringification currently prevents classification. Defer `xtask`, TUI, and proc-macro changes unless the audit finds a concrete recovery or source-chain requirement.

## 4. Preserve rich boundary diagnostics

1. Keep `ApiResult` and `color-eyre` in native I/O, network, CLI, TUI, service, and orchestration paths.
2. Convert typed errors to reports only at those boundaries, adding operation and resource context there rather than embedding terminal-oriented prose into domain variants.
3. Keep existing exit codes, top-level messages, verbose diagnostics, and webhook HTTP status mapping stable.
4. Do not add terminal styling, filesystem paths, HTTP status policy, or backtrace capture to portable domain errors unless that data is intrinsic to the domain failure.

## 5. Carry the policy into crate extraction

As `acorn-core`, `acorn-schema`, `acorn-analysis`, `acorn-api`, and adapter crates are extracted, each package should depend on `thiserror` only if it owns a typed error. Consumers import those errors directly from their owner; `acorn-lib` must neither re-export them nor introduce a catch-all enum. Native composition crates continue to own `color-eyre` reporting.

# Validation

1. Add focused unit tests for every converted type's exact `Display` output and intended `Error::source()` behavior.
2. Add compile-time or ordinary tests demonstrating that public typed errors convert into `color_eyre::Report` at application boundaries without changing top-level diagnostics.
3. Run focused tests for model download, Hugging Face selection, crosswalks, synchronization, gather output failure, and GitLab webhooks after their respective conversions.
4. Run the supported portability matrix:

```sh
cargo check -p acorn-lib --no-default-features
cargo check -p acorn-lib --no-default-features --features alloc
cargo check -p acorn-lib --no-default-features --target wasm32-unknown-unknown --features wasm
```

5. Inspect `cargo tree` for the minimal and WASM configurations and confirm no host-reporting dependency is introduced through thiserror.
6. Run `make lint` and `make test` after the mechanical conversions and again after any approved string-error replacements.
7. Smoke-test representative CLI failures and confirm exit codes, user-visible messages, and rich report context remain stable.

# Acceptance criteria

1. ACORN-owned reusable error types use `thiserror::Error` unless a documented portability or API constraint justifies a handwritten implementation.
2. `ApiResult` remains the host/application result type; the migration does not mechanically replace report-returning APIs with a workspace-wide enum.
3. Public error variants, fields, and existing display messages remain compatible during the mechanical phase.
4. Source chains are tested and contain only intentionally wrapped underlying errors.
5. Proc-macro diagnostics, remote error DTOs, and private string callbacks are not converted without a caller-driven reason.
6. Default, no-default, alloc, and WASM builds pass without resolving `color-eyre` in portable configurations.
7. CLI, TUI, and webhook boundary behavior remains unchanged unless separately approved.
8. Extracted crates own capability-specific error types and do not depend on a central catch-all error enum.

# Related plans

- [Crate boundary architecture](../refactors/crate-boundary-architecture.md)
- [Portability roadmap](../refactors/portability/roadmap.md)
- [Refactor initiative](../refactors/initiative.md)

[^1]: [thiserror documentation](https://github.com/dtolnay/thiserror)
