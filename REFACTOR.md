# ACORN Refactor Recommendations

This document captures cross-crate refactor and hardening recommendations for `acorn-cli` and `acorn-lib`. It is organized as an execution board first, then detailed task backlogs, then reference inventories.

---

## Navigation

1. [Goals](#goals)
2. [Task Board](#task-board)
3. [Risk Assessment](#risk-assessment)
4. [Scope](#scope)
5. [Core Refactor Backlog](#core-refactor-backlog)
6. [Portability Backlog: no-std + WASM](#portability-backlog-no-std--wasm)
7. [Analyzer Backlog](#analyzer-backlog)
8. [io::api Backlog](#ioapi-backlog)
9. [acorn-cmd: Standalone Crate Extraction](#acorn-cmd-standalone-crate-extraction)
10. [GGUF Fallback Discovery](#gguf-fallback-discovery-acorn-download-model)
11. [Interactive GGUF Picker (Follow-up PR)](#interactive-gguf-picker---interactive-tui-follow-up-pr)

## Goals

The refactor initiative is organized around four primary outcomes:

1. **Reliability** — Eliminate crash-prone code paths and improve error handling.
2. **Maintainability** — Modularize the codebase and eliminate duplication.
3. **Extensibility** — Improve architectural boundaries and reusable abstractions.
4. **Compatibility** — Enable no-std and WASM support.

---

## Task Board

Use this section to choose the next work item. Detailed notes for each task are grouped later by subsystem.

### Execution Phases

| Phase | Status | Focus | Primary outcomes |
|---|---|---|---|
| 1. Boundary hardening | Next | OOXML, path handling, export assets | Export failures become typed errors instead of silent corruption |
| 2. Shared abstractions | Planned | `ExportContext`, `SafePath`, interpolation | Shared behavior replaces duplicated mappings and path logic |
| 3. Structural modularization | Planned | CLI startup, API clients, schema I/O | Boundaries are explicit and easier to test |
| 4. Refinement | Planned | Bindings, portability, lower-priority cleanup | Repetition decreases and build targets broaden |

### High-Priority Next Tasks

| Area | Task | Difficulty | Impact | Detail section |
|---|---|---:|---:|---|
| Export/PPTX | Finish hardening OOXML interpolation and slide-copy mutations | Medium | Critical | [Core Refactor Backlog](#core-refactor-backlog) |
| Export/PPTX | Introduce shared `ExportContext` | Medium | High | [Core Refactor Backlog](#core-refactor-backlog) |
| no-std/WASM | Isolate std-only modules from domain modules | Medium | Critical | [Portability Backlog](#portability-backlog-no-std--wasm) |

### Visual Status Legend

- **Next**: best candidate for immediate implementation.
- **Planned**: scoped but depends on earlier work or has lower urgency.
- **Reference**: supporting inventory, not an implementation task by itself.

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Error handling changes break existing behavior | Medium | High | Extensive test suite, gradual rollout |
| Export system modifications affect user output | Medium | High | Feature flags, backward compatibility |
| no-std/WASM architecture changes | High | High | Clear migration docs, CI enforcement |

---

Prioritization rule:
1. Impact on reliability/security/correctness first.
2. Difficulty second within each impact tier.

---

## Scope

Included:
- `acorn-cli` command flow, startup orchestration, export/template behavior
- `acorn-lib` I/O, schema serialization, interpolation, analyzer/API seams
- Cross-crate boundaries where duplicate logic or weak contracts exist

Excluded:
- Generated docs outputs
- Style-only cleanup with no reliability/security/modularity payoff

---

## Core Refactor Backlog

### Critical Impact

#### 3. Finish hardening OOXML interpolation and slide-copy mutations
- Difficulty: Medium
- Status: Slide-copy mutations now fail with typed errors and use structured relationship updates; parser-backed paragraph selection and bullet interpolation remain.
- Evidence: `acorn-cli/src/commands/export/powerpoint.rs`, `acorn-lib/src/io/powerpoint/mod.rs`
- Recommendation:
  1. Replace remaining regex-based paragraph selection and bullet interpolation with parser-backed XML transforms.
  2. Replace slide-copy relationship/path string replacements with structured OOXML relationship and XML updates.
  3. Centralize remaining OOXML utility helpers to avoid split behavior.

### High Impact

#### 5. Eliminate duplicate export field extraction across HTML and PPTX paths
- Difficulty: Medium
- Evidence: `acorn-cli/src/template/mod.rs`, `acorn-lib/src/io/powerpoint/mod.rs`
- Recommendation:
  1. Introduce shared `ExportContext` (library-side).
  2. Extract fields once and consume in all format-specific renderers.
  3. Validate required placeholders up front.

#### 6. Split CLI startup concerns out of `main`
- Difficulty: Medium
- Evidence: `acorn-cli/src/main.rs`
- Recommendation:
  1. Extract setup units: logging, runtime/thread pool, database bootstrap, command dispatch.
  2. Keep main as orchestration glue with uniform error boundary.

### Medium Impact

#### 8. Consolidate interpolation primitives under one shared utility
- Difficulty: Low
- Evidence: `acorn-lib/src/util/mod.rs`, `acorn-lib/src/io/powerpoint/mod.rs`
- Recommendation:
  1. Extend shared interpolation trait(s) for list/bullet use-cases.
  2. Remove parallel interpolation stacks.

#### 9. Introduce stronger shared API-client contract
- Difficulty: Medium
- Evidence: `acorn-lib/src/io/api/mod.rs`, `acorn-lib/src/io/api/raid.rs`, `acorn-lib/src/io/api/gitlab.rs`
- Recommendation:
  1. Expand common trait surface for health, auth, endpoint rendering, and error normalization.
  2. Move duplicated logic behind shared helper modules.

#### 10. Reduce binding-layer repetition in Python wrapper
- Difficulty: Medium
- Evidence: `acorn-py/src/lib.rs`
- Recommendation:
  1. Complete data exposure shape first.
  2. Then apply macro-based codegen for repetitive PID wrapper patterns.

### Lower Impact

#### 11. Tighten schema I/O and format validation handoff between CLI and library
- Difficulty: High
- Evidence: `acorn-lib/src/io/mod.rs`, `acorn-lib/src/schema/research_activity/mod.rs`, `acorn-cli/src/commands/export/mod.rs`
- Recommendation:
  1. Extend `InputOutput` with stricter read/validate variants.
  2. Make CLI rely on library-side strict format validation consistently.

#### 13. Separate parsed CLI model from execution options model
- Difficulty: Medium
- Evidence: `acorn-cli/src/cli/mod.rs`, `acorn-cli/src/main.rs`
- Recommendation:
  1. Keep clap parse structs isolated.
  2. Normalize conversion to execution options in one layer.

### Cross-Crate Abstractions To Build First

1. `ExportContext` in `acorn-lib` used by both CLI template and lib PowerPoint export.
2. `SafePath` utility in `acorn-lib` consumed by CLI path-resolution and export file operations.
3. Unified interpolation extension in `acorn-lib` for string + list placeholders.
4. Single CLI error boundary mapping internal `Result` errors to exit codes.

### Completion Criteria

1. Export paths fail with typed errors, never silent corruption.
2. Path normalization and containment are centralized and reused.
3. Export field mapping exists in one shared context object.
4. Cross-crate interpolation behavior is unified and test-covered.

---

## Portability Backlog: no-std + WASM

### Critical Impact

#### 1. Isolate std-only modules from shared domain modules
- Difficulty: Medium
- Status: Initial boundary repaired. Pure JSON/JSONC detection no longer imports the std-gated `io` module; broader module isolation remains.
- Evidence: `acorn-lib/src/analyzer/mod.rs` imports `crate::io::*`, process APIs, filesystem-like behaviors.
- Recommendation:
  1. Move std-heavy logic into `io/std/*` and keep pure transformations in `io/core/*`.
  2. Keep schema parsing/validation and formatting logic independent from command/process/network concerns.
  3. Define trait boundaries for host operations (FS, HTTP, clock, randomness, process).

### High Impact

#### 2. Replace no-std-hostile error/reporting surface for core builds
- Difficulty: Medium
- Evidence: `acorn-lib/src/io/mod.rs` and other modules use `ApiResult<T> = Result<T, Report>`.
- Recommendation:
  1. Introduce a core error enum (`AcornError`) with `thiserror`/`core::fmt` support.
  2. Keep `color-eyre` conversion in std adapters only.
  3. Make `ApiResult` alias conditional:
     - no-std: `Result<T, AcornError>`
     - std CLI adapters: map to rich reports.

#### 3. Add dedicated WASM feature path
- Difficulty: Medium
- Recommendation:
  1. Add `wasm` feature with wasm-safe dep toggles.
  2. Provide `wasm-bindgen`/`js-sys` adapters in a separate module and avoid pulling browser crates into non-wasm no-std builds.
  3. Define host traits for time/random/http and implement wasm adapters.

### Medium Impact

#### 4. Refactor prelude into strict capability preludes
- Difficulty: Low
- Evidence: `acorn-lib/src/prelude.rs` has dual exports, but call sites in analyzer/util/io often assume std APIs.
- Recommendation:
  1. Create explicit `prelude::core`, `prelude::alloc`, `prelude::std` modules.
  2. Restrict imports by module capability level to prevent accidental std usage.

#### 5. Partition utility module into pure and std submodules
- Difficulty: Medium
- Evidence: `acorn-lib/src/util/mod.rs` imports `comfy_table`, `console`, `owo-colors`, `rust_embed` unconditionally at module scope.
- Recommendation:
  1. Move pure string/identifier/interpolation helpers into `util/core`.
  2. Move console formatting/rendering into `util/std`.
  3. Re-export conditionally.

#### 6. Introduce host-abstraction traits for runtime services
- Difficulty: Medium
- Recommendation:
  1. Define traits: `FileStore`, `HttpClient`, `Clock`, `Entropy`, `ProcessRunner`.
  2. Provide std implementations in adapters.
  3. Provide wasm/no-std implementations where feasible.

### Lower Impact

#### 7. Tighten workspace dependency defaults for no-std friendliness
- Difficulty: Medium
- Evidence: `Cargo.toml` workspace dependency for `chrono` includes `features = ["serde", "std"]`.
- Recommendation:
  1. Move std-dependent feature toggles from workspace defaults into crate-level feature wiring.
  2. Prefer alloc/core-only defaults in workspace where possible.

#### 8. Add no-std and wasm CI matrix as a first-class quality gate
- Difficulty: Low
- Recommendation:
  1. Add CI jobs:
     - `cargo check -p acorn-lib --no-default-features`
     - `cargo check -p acorn-lib --no-default-features --features alloc`
     - `cargo check -p acorn-lib --no-default-features --target wasm32-unknown-unknown --features wasm`
  2. Add `cargo tree` diff checks for feature leakage.

### Recommended Feature Model (Target State)

1. `default = ["std", "analysis", "agentic"]` for existing UX compatibility.
2. `core` feature for minimum compile surface.
3. `alloc` feature for heap-backed core functionality.
4. `std` feature for FS/network/process/runtime adapters.
5. `wasm` feature for wasm32 adapters and constraints.
6. Domain features (`analysis`, `doctor`, `powerpoint`, `agentic`, `database`, `export-html`, `export-pptx`) layered on top of capability features.

### Implementation PR Plan

#### PR 4: Separate Core Domain Logic from std Adapters

Goal: Keep pure domain/schema/transform code portable while isolating host concerns.

Primary edits:
1. Introduce module splits: `io/core/`, `io/std/`, `util/core/`, `util/std/`.
2. Add capability-scoped preludes: `prelude::core`, `prelude::alloc`, `prelude::std`.

Validation: `cargo check -p acorn-lib --no-default-features --features alloc` and `cargo check -p acorn-lib --features std`

Acceptance criteria: Core modules compile without std imports; std-only APIs are contained under feature-gated adapter modules.

#### PR 5: Introduce Host Traits + Implementations

Goal: Replace concrete runtime dependencies with pluggable host interfaces.

Primary edits:
1. Add traits in `io/host.rs`: `FileStore`, `HttpClient`, `Clock`, `Entropy`, `ProcessRunner`.
2. Add std impls in `io/std/host_impl.rs`, wasm stubs in `io/wasm/host_impl.rs`.
3. Refactor selected callsites in analyzer/io/api to use trait abstractions.

Validation: `cargo check -p acorn-lib --features std,analysis` and `cargo check -p acorn-lib --no-default-features --features wasm`

#### PR 6: WASM Target Enablement

Goal: Provide a first-class WASM-capable build mode.

Primary edits:
1. `acorn-lib/Cargo.toml` — Add target-specific dependency blocks for wasm.
2. Add wasm-specific module wiring: `acorn-lib/src/io/wasm/mod.rs`.

Validation: `cargo check -p acorn-lib --no-default-features --features wasm --target wasm32-unknown-unknown`

### Dependency Replacement Candidates

1. `color-eyre` — keep in std adapters; introduce core error type for no-std path.
2. `rayon` — gate strictly behind `analysis` + `std`; avoid in portable core.
3. `docx-rs`, `tera`, `percy-dom`, `indicatif` — isolate under export/template features.
4. `chrono` — ensure `chrono/std` only enabled by std feature.
5. `rusqlite`, `reqwest`, `tokio`, `axum`, `tower`, `which` — keep optional and std-only.

---

## Analyzer Backlog

### Low Difficulty

#### 2. Monoid for Check collection accumulation
- Evidence: `acorn-lib/src/analyzer/mod.rs`, `acorn-lib/src/analyzer/check.rs` — repeated `join_all await flatten collect` patterns.
- Opportunity: Introduce `CheckBatch` newtype with identity as empty list, associative combine as concatenation, `fold_map` style accumulation.

#### 3. Monoid for summary aggregation
- Evidence: `acorn-lib/src/analyzer/check.rs` — summary logic aggregates per-category counts via chained map and collect.
- Opportunity: Represent summary as a monoidal record where fields combine by addition; fold checks directly into one summary value.

#### 4. Endomorphism monoid for ValeConfig normalization pipeline
- Evidence: `acorn-lib/src/analyzer/mod.rs` — ValeConfig package resolution and configuration mutations.
- Opportunity: Model each config transformation as `ValeConfig -> ValeConfig`; compose transformations and apply once.

### Medium Difficulty

#### 5. Catamorphism style fold for validation error trees
- Evidence: `acorn-lib/src/analyzer/error.rs` — process recursively matches Field, Struct, List with duplicated path-context handling.
- Opportunity: Define one algebra over `ValidationErrorsKind` nodes; implement a generic fold carrying path context.

#### 6. Applicative validation for composed checks
- Evidence: `acorn-lib/src/analyzer/mod.rs`, `acorn-lib/src/analyzer/check.rs` — mix success and error paths with repeated match scaffolding.
- Opportunity: Convert independent check computations into applicative composition where possible; accumulate multiple failures in one pass.

#### 7. Reader-like context functor for CheckOptions flow
- Evidence: `acorn-lib/src/analyzer/mod.rs` — repeatedly threads options and derived context through nested async closures.
- Opportunity: Introduce explicit analysis context and composable context-to-result functions.

#### 8. Result functor normalization for conversion boundaries
- Evidence: `acorn-lib/src/analyzer/mod.rs`, `acorn-lib/src/analyzer/vale.rs` — repeated match blocks that map failures to defaults.
- Opportunity: Use `map_err` and recovery combinators through one consistent pipeline.

### High Difficulty

#### 9. Higher-level Traversable abstraction across standards
- Evidence: `acorn-lib/src/analyzer/mod.rs` — near-isomorphic implementations for Cff, ResearchActivity, Docx, and Text check methods.
- Opportunity: Define generic traversal strategy over readable artifacts and check categories; keep per-standard adapters only for reading and URL extraction.

#### 10. Typed semiring-like scoring model for readability and prose checks
- Evidence: `acorn-lib/src/analyzer/readability/mod.rs`, `acorn-lib/src/analyzer/check.rs` — map scoring and severity via ad hoc conditionals.
- Opportunity: Introduce typed score algebra where combine and threshold are explicit; map outputs to `CheckSeverity` through one interpreter layer.

### Recommended Implementation Order

1. Item 2 to reduce duplication in `analyzer/mod.rs`.
2. Item 5 to stabilize validation error flattening.
3. Item 9 if broad standard-level deduplication is desired.

---

## io::api Backlog

### Improvement Ideas

#### Low Difficulty

##### 1. Standardize Auth Header Construction Across API Modules
- Add one shared helper in `acorn-lib/src/io/api/mod.rs` for bearer token headers; replace per-module duplicated header creation in `gitlab`, `raid`, and `openai`.

##### 2. Add Shared Request Builders for Common API Shapes
- Add utility helpers for `{token + template value}` and `{token + JSON body}` parameter sets; remove repeated `param!` vector assembly boilerplate.

##### 3. Normalize Environment Variable Discovery Rules
- Define a consistent fallback sequence for token/domain variables across modules; expose one helper for required vs optional secrets.

#### Medium Difficulty

##### 4. Introduce Shared API Error Envelope Traits
- Define a trait for extracting user-facing error strings from provider-specific error payloads; implement for `gitlab`, `raid`, and `openai` error responses.

##### 5. Add OpenAPI-Aware Endpoint Validation Utilities
- Add utility functions that check endpoint template names and required template keys; catch template drift early when `application.json` changes.

##### 6. Add Lightweight Response DTO Layer for Evolving APIs
- Layer optional typed DTOs for stable subfields (`id`, `object`, usage metadata) over baseline raw `serde_json::Value` support.

#### High Difficulty

##### 7. Generate API Module Skeletons from OpenAPI Specs
- Create an internal codegen path that emits endpoint definitions and baseline DTOs; use generated modules as a starting point.

##### 8. Unified Capability Matrix for io::api Providers
- Define one capability descriptor (`supports_pagination`, `supports_streaming`, `requires_token`, etc.); drive command-layer behavior from capability metadata.

### Reuse Opportunities From Module Scan

#### High Impact

##### 1. Extract endpoint resolution helpers
- Evidence: Repeated `Endpoint::from_template(template).map(|e| e.with_domain(...))` across `openai`, `gitlab`, and `raid`.
- Opportunity: Add root helpers for template+domain endpoint resolution and named endpoint lookup.

##### 2. Extract auth-header parameter builders
- Evidence: Repeated `Authorization: Bearer` in `openai`/`raid`, `PRIVATE-TOKEN` in `gitlab`.
- Opportunity: Add reusable helpers for common header param styles.

##### 3. Extract required-input validators
- Evidence: Repeated required `identifier` and required `body` match branches in `openai`, `raid`, and `gitlab`.
- Opportunity: Add root validators for required non-empty identifier/body values.

##### 4. Extract invoke-and-parse convenience wrappers
- Evidence: Repeated sequence: build params → invoke/invoke_with → handle/handle_or across most provider functions.
- Opportunity: Add wrappers that combine invocation and parse strategy.

#### Medium Impact

##### 5. Extract optional query parameter composition helper
- Evidence: Repeated optional list parameter insertion in `openai` and other modules.
- Opportunity: Add helper to append optional key/value query params when present and non-empty.

##### 6. Normalize environment configuration bootstrap
- Evidence: Repeated `.env` load + env var fallback patterns in `openai`, `raid`, and `gitlab` options.
- Opportunity: Add root helper for consistent env loading and prioritized var resolution.

##### 7. Extract status/health helper pattern
- Evidence: Similar `is_healthy` wrappers in `orcid`, `ror`, and `citeas`.
- Opportunity: Add root utility for status-to-health checks with pluggable predicate.

### Recommended Extraction Order

1. Endpoint resolution + auth/header builders.
2. Required-input validators + optional query param composition helper.
3. Invoke-and-parse wrappers with fallback support.
4. Environment bootstrap and health-check helpers.

---

## acorn-cmd: Standalone Crate Extraction

This section documents the plan to extract `cmd!`, `args!`, and `cmd_sh_words!` into a standalone `acorn-cmd` crate usable without the rest of `acorn-lib`.

### Current Location

- Proc-macro `cmd_sh_words!`: `acorn-macros/src/lib.rs`
- Declarative macros `cmd!` and `args!`: `acorn-lib/src/util/macros.rs` (gated behind `#[cfg(feature = "cmd")]`)
- Runtime helpers: `acorn-lib/src/util/cmd.rs` (gated behind `#[cfg(feature = "std")]`)
- Tests: `acorn-lib/src/util/tests/cmd.rs`

### Extraction Steps

| Step | Action | Dependencies |
|------|--------|-------------|
| 1 | Create `acorn-cmd/Cargo.toml` with proc-macro crate depending on `syn`, `quote`, `proc-macro2` | None |
| 2 | Move `CmdWord` enum, `read_interpolation`, `expand_cmd_sh_words` from `acorn-macros` to `acorn-cmd` | Step 1 |
| 3 | Move `cmd!`, `args!` declarative macros and runtime helpers into `acorn-cmd` as non-proc-macro library code | Step 2 |
| 4 | Add `shell-words` dependency to `acorn-cmd` | Step 3 |
| 5 | Add integration tests matching current coverage | Step 4 |
| 6 | In `acorn-lib`, change `pub use acorn_macros::cmd_sh_words` to `pub use acorn_cmd::cmd_sh_words` | Step 5 |
| 7 | Replace `cmd` feature in `acorn-lib` with direct dep on `acorn-cmd` | Step 6 |

---

## GGUF Fallback Discovery: `acorn download model`

### Overview

When `acorn download model openai/gpt-oss-2b` targets a Hugging Face repository that contains no `.gguf` files, automatically search for the most popular community GGUF quantization repo derived from that base model and download from it instead.

### Problem

`download_from_huggingface` calls `huggingface::repository_tree()` then `RepositoryFiles::filter()` → `select()`, which fails with *"No GGUF model files found"* when zero `.gguf` files exist. Users expect ACORN to find quantization repos (e.g. `bartowski/openai_GPT-OSS-2B-GGUF`) automatically.

### Design

A fallback discovery step activates only when:
- The original repo tree has **zero** `.gguf` files
- The selector looks like `owner/model-name` (no URL scheme, no local path)
- Not in offline mode
- User has not provided `--filter` (which implies intentional non-GGUF format)
- User has not passed `--no-fallback`

Uses the `hf-hub` crate (minimal features) for search; existing ACORN HTTP downloader handles the actual file download and progress.

### Dependency Changes

| Crate | File | Notes |
|-------|------|-------|
| `hf-hub` | `Cargo.toml` (workspace) | `version = "1", default-features = false` |
| `hf-hub` | `acorn-lib/Cargo.toml` | Optional, behind `std` feature |
| `hf-hub` | `acorn-cli/Cargo.toml` | Required (CLI is always `std`) |

### New Library Module: `acorn-lib/src/io/api/gguf.rs`

Behind `#[cfg(feature = "std")]`. Contains:

**Public API:**

```rust
pub async fn find_popular_gguf_repo(base_model: &str, limit: usize) -> ApiResult<String>
```

Returns the matched `repo_id` (e.g. `bartowski/openai_GPT-OSS-2B-GGUF`).

**Internal helpers (not pub):**

- `is_declared_quantization_of(model, base_model)` — checks `base_models` field and `base_model:quantized:*` tags. Strict path only; no weak name fallback.
- `card_declares_base_model(card, base_model)` — checks `card_data.base_model` as string or array, requires exact case-insensitive match.

**Algorithm:**

1. Extract basename: `"openai/gpt-oss-2b"` → `"gpt-oss-2b"`
2. Call `client.list_models().search(basename).filter("gguf").sort("downloads").full(true).card_data(true).limit(limit).send()`
3. Stream results, collect only strong matches (`is_declared_quantization_of`)
4. Sort by `downloads` descending, return first
5. If no match: `Err("no GGUF quantization repo found for {base_model}")`

Register in `acorn-lib/src/io/api/mod.rs`:

```rust
#[cfg(feature = "std")]
pub mod gguf;
```

### Catch "no GGUF" Error in `download_from_huggingface`

**File:** `acorn-cli/src/commands/download/model.rs`

After `files.download(options)` fails, add a fallback branch:

```rust
Err(why) if is_no_gguf_error(&why) && !offline && filter.is_empty() && !no_fallback =>
    attempt_gguf_fallback(&identifier, &self.revision, &options).await,
```

`is_no_gguf_error` checks the error message contains `"No GGUF model files found"` (from `huggingface.rs:150`).

`attempt_gguf_fallback`:

1. Calls `gguf::find_popular_gguf_repo(&identifier, search_limit)`
2. Logs: `"{} GGUF fallback: searching for quantization repos derived from '{identifier}'"`
3. Calls `huggingface::repository_tree(&new_id, "main")`
4. Re-runs `files.download(options)` with updated identifier
5. Returns the new identifier as the model name

### New CLI Arguments

**File:** `acorn-cli/src/cli/arguments.rs`, in `DownloadModel`:

```rust
/// Maximum number of Hugging Face repos to search when discovering GGUF quantizations
///
/// Applies when the requested model repository contains no GGUF files and automatic
/// fallback is enabled. Can also be set via the ACORN_GGUF_SEARCH_LIMIT environment variable.
#[arg(long, value_name = "LIMIT", env = "ACORN_GGUF_SEARCH_LIMIT", default_value_t = 200, help_heading = "OPTIONS")]
pub(crate) gguf_search_limit: usize,

/// Disable automatic GGUF quantization repo discovery when the target repo has no GGUF files
#[arg(long, help_heading = "FLAGS")]
pub(crate) no_fallback: bool,
```

### Signature Changes

`download_from_huggingface` needs `offline: bool`, `no_fallback: bool`, `search_limit: usize` parameters, which should be added to `huggingface::Options` struct.

`run()` in `model.rs` gains `no_fallback: bool` and `search_limit: usize` parameters.

`acorn-cli/src/commands/mod.rs` threads `args.gguf_search_limit` and `args.no_fallback` through to `download::model::run`.

### Tests

**Unit tests** in `acorn-lib/src/io/api/gguf.rs` (`#[cfg(test)]`):

- `is_declared_quantization_of` with `base_models` field match
- `is_declared_quantization_of` with tag match
- `is_declared_quantization_of` with no match
- `card_declares_base_model` with string value
- `card_declares_base_model` with array value
- `card_declares_base_model` with missing field

**Unit tests** in `acorn-cli/src/commands/download/tests/mod.rs`:

- `is_no_gguf_error` matches the expected error string
- `--no-fallback` flag parses correctly
- `--gguf-search-limit` parses correctly with and without env var

### File Change Summary

| # | File | Change |
|---|------|--------|
| 1 | `Cargo.toml` | Add `hf-hub` to `[workspace.dependencies]` |
| 2 | `acorn-lib/Cargo.toml` | Add `hf-hub` (optional, `std`) |
| 3 | `acorn-cli/Cargo.toml` | Add `hf-hub` |
| 4 | `acorn-lib/src/io/api/gguf.rs` | **New** — discovery logic + unit tests |
| 5 | `acorn-lib/src/io/api/mod.rs` | Add `pub mod gguf` behind `cfg(feature = "std")` |
| 6 | `acorn-cli/src/cli/arguments.rs` | Add `--no-fallback` and `--gguf-search-limit` |
| 7 | `acorn-cli/src/commands/download/model.rs` | Catch no-GGUF error, fallback path, updated `run()` signature |
| 8 | `acorn-cli/src/commands/mod.rs` | Thread new args to `download::model::run` |
| 9 | `acorn-cli/src/commands/download/tests/mod.rs` | New tests for fallback behavior |

### Verification

```bash
make lint          # cargo fmt + clippy
make test          # cargo nextest run
make help          # regenerate CommandLineHelp.md after arg changes
```

---

## Interactive GGUF Picker: `--interactive` TUI (Follow-up PR)

### Overview

When `acorn download model openai/gpt-oss-2b --interactive` is used and the GGUF fallback search returns multiple candidates, open a TUI picker screen so the user can choose which quantization repo to download from. Depends on the GGUF Fallback Discovery work above.

### Trigger Conditions

The TUI picker activates when **all** of the following are true:
1. The target repo has no `.gguf` files (triggers fallback search)
2. `--interactive` flag is passed on `DownloadModel`
3. `tui` feature is enabled (default)
4. Stdout is a terminal (`is_terminal::IsTerminal`)
5. The fallback search returns **more than one** candidate

If the search returns exactly **one** candidate, download it automatically (same as non-interactive fallback) without opening the TUI.

If the terminal is not interactive (piped/CI), fall back to the automatic most-downloaded pick with a warning.

### TUI Changes

#### 1. Generalize `run_tui` return type

**File:** `acorn-tui/src/lib.rs`

Current `run_tui` returns `Result<()>`. Add a generic variant:

```rust
pub fn run_tui_with<T>(initial_screen: Screen, initial_state: Option<GgufPickerState>) -> Result<Option<T>>
where
    T: Clone + Send + 'static,
{
    // ...existing setup...
    let mut app = App::new(initial_screen);
    if let Some(state) = initial_state {
        app.set_gguf_picker(state);
    }
    let result = app.run(&mut terminal);
    // ...existing teardown...
    app.take_gguf_picker_result().map(Some).transpose()
}
```

Keep the existing `run_tui` as a convenience wrapper that calls `run_tui_with::<()>(screen, None)`.

#### 2. New screen variant

**File:** `acorn-tui/src/lib.rs`

```rust
pub enum Screen {
    Dashboard,
    Doctor,
    Check,
    GgufPicker,  // NEW
}
```

#### 3. Gguf picker state

**File:** `acorn-tui/src/app.rs`

Add to `App`:

```rust
pub struct GgufPickerState {
    /// Candidate repo IDs sorted by downloads descending
    pub candidates: Vec<GgufCandidate>,
    /// Currently selected index
    pub selected: usize,
    /// Base model that was searched (e.g. "openai/gpt-oss-2b")
    pub base_model: String,
    /// Result: selected candidate ID (set on Enter)
    pub result: Option<String>,
}

pub struct GgufCandidate {
    /// Repository ID (e.g. "bartowski/openai_GPT-OSS-2B-GGUF")
    pub id: String,
    /// Download count
    pub downloads: u64,
    /// Likability score
    pub likes: Option<u64>,
    /// Detected quantizations in repo
    pub quantizations: Vec<String>,
}
```

Add methods:
- `App::set_gguf_picker(state: GgufPickerState)`
- `App::take_gguf_picker_result() -> Option<String>`

#### 4. Gguf picker screen

**New file:** `acorn-tui/src/screens/gguf_picker.rs`

Renders a two-panel layout:
- **Left panel:** scrollable list of candidate repos using `widgets::list::render_list` with `list_item`. Shows repo name, download count, and like count.
- **Right panel:** detail pane using `widgets::detail::render_detail` showing selected repo metadata (quantizations available, download stats).
- **Bottom bar:** status bar with `[↑/↓] Navigate  [Enter] Download  [Esc] Cancel`

Navigation:
- `Up`/`j` and `Down`/`k` move selection
- `Enter` confirms selection (sets `result`, quits)
- `Esc` cancels (returns `None`)

Register in `screens/mod.rs`:
```rust
pub mod gguf_picker;
```

Wire in `App::render` and `App::handle_event`:
```rust
Screen::GgufPicker => screens::gguf_picker::render(f, self),
Screen::GgufPicker => screens::gguf_picker::handle_event(self, evt),
```

#### 5. Export the candidate type

**File:** `acorn-tui/src/lib.rs`

```rust
pub use app::GgufCandidate;
```

So the CLI can construct candidates from the search results.

### CLI Changes

#### 6. `--interactive` flag

**File:** `acorn-cli/src/cli/arguments.rs`

Add to `DownloadModel`:

```rust
/// Open interactive TUI picker when GGUF fallback finds multiple candidates
#[arg(long, help_heading = "FLAGS")]
pub(crate) interactive: bool,
```

#### 7. Interactive fallback path

**File:** `acorn-cli/src/commands/download/model.rs`

Modify `attempt_gguf_fallback` to accept `interactive: bool`:

```rust
async fn attempt_gguf_fallback(
    identifier: &str,
    revision: &str,
    options: CommandOptions,
    search_limit: usize,
    interactive: bool,  // NEW
) -> ApiResult<String>
```

After `gguf::find_popular_gguf_repo` returns multiple candidates (needs refactoring to return `Vec` instead of single result):

1. If `interactive` and stdout is terminal and `tui` feature enabled:
   - Convert candidates to `Vec<acorn_tui::GgufCandidate>`
   - Call `acorn_tui::run_tui_with::<String>(Screen::GgufPicker, Some(state))`
   - If user selected a repo, proceed with download using that repo ID
   - If user cancelled, return error
2. If not interactive or not terminal:
   - Pick the top candidate automatically (existing behavior)
   - Log: `"{} Multiple GGUF repos found; using most downloaded: '{}' (non-interactive mode)"`

#### 8. Refactor search to return candidates list

**File:** `acorn-lib/src/io/api/gguf.rs`

Add a new function alongside the existing one:

```rust
pub async fn search_gguf_repos(base_model: &str, limit: usize) -> ApiResult<Vec<GgufCandidate>>
```

Where `GgufCandidate` is a library-side struct (separate from the TUI one):

```rust
pub struct GgufCandidate {
    pub id: String,
    pub downloads: u64,
    pub likes: Option<u64>,
    pub quantizations: Vec<String>,
}
```

The existing `find_popular_gguf_repo` can be reimplemented as `search_gguf_repos(...).into_iter().next()`.

The CLI converts `acorn::io::api::gguf::GgufCandidate` → `acorn_tui::GgufCandidate`.

### File Change Summary

| # | File | Change |
|---|------|--------|
| 1 | `acorn-tui/src/lib.rs` | Add `GgufPicker` to `Screen`, add `run_tui_with`, export `GgufCandidate` |
| 2 | `acorn-tui/src/app.rs` | Add `GgufPickerState`, `GgufCandidate`, picker state methods |
| 3 | `acorn-tui/src/screens/gguf_picker.rs` | **New** — picker screen render + event handling |
| 4 | `acorn-tui/src/screens/mod.rs` | Register `gguf_picker` module |
| 5 | `acorn-lib/src/io/api/gguf.rs` | Add `GgufCandidate` struct, `search_gguf_repos` returning `Vec` |
| 6 | `acorn-cli/src/cli/arguments.rs` | Add `--interactive` flag to `DownloadModel` |
| 7 | `acorn-cli/src/commands/download/model.rs` | Interactive fallback path, thread `interactive` through |

### Verification

```bash
make lint
make test
cargo build --features tui          # TUI feature compiles
cargo build --no-default-features   # non-TUI fallback compiles
make help                           # regenerated after arg change
```

Manual test:
```bash
cargo run --bin acorn -- download model openai/gpt-oss-2b --interactive
# Should open TUI picker with GGUF repo candidates
```
