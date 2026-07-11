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
10. [Model Download Implementation](#model-download-implementation)
11. [Reference Inventories](#reference-inventories)

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
| 1. Reliability triage | Complete | Command flow and internal error contracts | Panic/unimplemented/exit-heavy command paths removed |
| 2. Boundary hardening | Next | OOXML, path handling, export assets | Export failures become typed errors instead of silent corruption |
| 3. Shared abstractions | Planned | `ExportContext`, `SafePath`, interpolation | Shared behavior replaces duplicated mappings and path logic |
| 4. Structural modularization | Planned | CLI startup, API clients, schema I/O | Boundaries are explicit and easier to test |
| 5. Refinement | Planned | Bindings, portability, lower-priority cleanup | Repetition decreases and build targets broaden |

### High-Priority Next Tasks

| Area | Task | Difficulty | Impact | Detail section |
|---|---|---:|---:|---|
| Export/PPTX | Finish hardening OOXML interpolation and slide-copy mutations | Medium | Critical | [Core Refactor Backlog](#core-refactor-backlog) |
| Export/Templates | Fail fast on missing embedded assets | Low | High | [Core Refactor Backlog](#core-refactor-backlog) |
| Export/PPTX | Introduce shared `ExportContext` | Medium | High | [Core Refactor Backlog](#core-refactor-backlog) |
| io::api | Complete HTTP method dispatch in root invoke path | Low | Critical | [io::api Backlog](#ioapi-backlog) |
| no-std/WASM | Isolate std-only modules from domain modules | Medium | Critical | [Portability Backlog](#portability-backlog-no-std--wasm) |
| Model Download | Implement `acorn download model` weight downloads with `.gguf` defaults | Medium | Critical | [Model Download Implementation](#model-download-implementation) — Phase 1 ✅, Phase 2 planned |
| Analyzer | Extract link-check traversal helpers | Low | High | [Analyzer Backlog](#analyzer-backlog) |

### Visual Status Legend

- **Complete**: implementation exists and the task should move to verification or cleanup only.
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
- Status: Partially complete — parser-backed presentation and relationship updates exist; interpolation and slide-copy mutations still need cleanup.
- Evidence: `acorn-cli/src/commands/export/powerpoint.rs`, `acorn-lib/src/io/powerpoint/mod.rs`
- Recommendation:
  1. Replace remaining regex-based paragraph selection and bullet interpolation with parser-backed XML transforms.
  2. Replace slide-copy relationship/path string replacements with structured OOXML relationship and XML updates.
  3. Centralize remaining OOXML utility helpers to avoid split behavior.

### High Impact

#### 4. Fail fast on missing export/template assets
- Difficulty: Low
- Evidence: `acorn-cli/src/template/mod.rs`
- Recommendation:
  1. Return `Result` from asset lookup/embedding paths.
  2. Bubble failures into export command errors.
  3. Add one integration check for required embedded assets.

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

#### 1. Functor plus Traversable extraction for link-check futures
- Evidence: `acorn-lib/src/analyzer/mod.rs` — Cff website checks build multiple optional futures through repeated map, flatten, collect chains.
- Opportunity: Introduce a small helper based on functor and traversable style; replace repeated URL-to-Check transformation with one generic mapper.

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

1. Items 1 and 2 to reduce duplication in `analyzer/mod.rs`.
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

#### Critical

##### 1. Complete HTTP method dispatch in root invoke path
- Difficulty: Low
- Impact: Critical
- Evidence: `acorn-lib/src/io/api/mod.rs` currently dispatches only `GET`, `POST`, and `PUT` and falls through to `unimplemented!` for other methods; `openai::api` now includes `DELETE` resources.
- Opportunity: Add full `HttpMethod` dispatch support in `RemoteResource::invoke_with`; remove `unimplemented!` fallback; add tests for method routing parity.

#### High Impact

##### 2. Extract endpoint resolution helpers
- Evidence: Repeated `Endpoint::from_template(template).map(|e| e.with_domain(...))` across `openai`, `gitlab`, and `raid`.
- Opportunity: Add root helpers for template+domain endpoint resolution and named endpoint lookup.

##### 3. Extract auth-header parameter builders
- Evidence: Repeated `Authorization: Bearer` in `openai`/`raid`, `PRIVATE-TOKEN` in `gitlab`.
- Opportunity: Add reusable helpers for common header param styles.

##### 4. Extract required-input validators
- Evidence: Repeated required `identifier` and required `body` match branches in `openai`, `raid`, and `gitlab`.
- Opportunity: Add root validators for required non-empty identifier/body values.

##### 5. Extract invoke-and-parse convenience wrappers
- Evidence: Repeated sequence: build params → invoke/invoke_with → handle/handle_or across most provider functions.
- Opportunity: Add wrappers that combine invocation and parse strategy.

#### Medium Impact

##### 6. Extract optional query parameter composition helper
- Evidence: Repeated optional list parameter insertion in `openai` and other modules.
- Opportunity: Add helper to append optional key/value query params when present and non-empty.

##### 7. Normalize environment configuration bootstrap
- Evidence: Repeated `.env` load + env var fallback patterns in `openai`, `raid`, and `gitlab` options.
- Opportunity: Add root helper for consistent env loading and prioritized var resolution.

##### 8. Extract status/health helper pattern
- Evidence: Similar `is_healthy` wrappers in `orcid`, `ror`, and `citeas`.
- Opportunity: Add root utility for status-to-health checks with pluggable predicate.

### Recommended Extraction Order

1. Critical method dispatch support for all declared `HttpMethod` variants.
2. Endpoint resolution + auth/header builders.
3. Required-input validators + optional query param composition helper.
4. Invoke-and-parse wrappers with fallback support.
5. Environment bootstrap and health-check helpers.

---

## acorn-cmd: Standalone Crate Extraction

This section documents the plan to extract `cmd!`, `args!`, and `cmd_sh_words!` into a standalone `acorn-cmd` crate usable without the rest of `acorn-lib`.

### Current Location

- Proc-macro `cmd_sh_words!`: `acorn-macros/src/lib.rs`
- Declarative macros `cmd!` and `args!`: `acorn-lib/src/util/macros.rs` (gated behind `#[cfg(feature = "cmd")]`)
- Runtime helpers: `acorn-lib/src/util/cmd.rs` (gated behind `#[cfg(feature = "std")]`)
- Tests: `acorn-lib/src/util/tests/cmd.rs`

### Status

- Feature gate (`#[cfg(feature = "cmd")]`): ✅ Complete
- Extraction to new crate: 🔜 Future work

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

## Model Download Implementation

This section tracks ACORN-side work to make `acorn download model` the primary backend for downloading model weights used by Gold and local inference environments.

### Phase 1: Config + CLI Foundation (✅ Complete)

- `ApplicationConfiguration` supports `models: Option<Vec<String>>` field.
- `--config` / `-c` flag resolves application config and merges with positional models.
- Positional-first deduplication with positional models always winning.
- Default output dir resolves to `$HOME/.models` when `--output` is omitted.
- Models validated against local database with clear error messages.
- Offline mode rejects download immediately with error.
- 17 tests passing covering config parsing, model resolution, and CLI integration.

### Phase 2: GGUF Resolver + Weight Downloads (Planned)

#### Goal

Make ACORN responsible for model download orchestration so downstream projects do not need Python-specific Hugging Face tooling.

#### Current State

- `acorn-cli` exposes `download model` with config-driven resolution and positional-first deduplication.
- Actual weight download is deferred to Phase 2.
- `acorn-lib/src/schema/agent/mod.rs` exposes `ModelDetails.weights`, `acorn-lib/src/io/database/schema.rs` persists in `ModelRow.weights`.
- `acorn-lib/src/io/api/models_dev.rs` downloads and caches the models.dev catalog.

#### Tasks

1. Define `ModelDownloadPlan` type with source URL/repository, selected files, output directory, auth requirements, expected size/checksum.
2. Resolve selectors from config → local database (`ModelRow.weights`) → models.dev catalog (`ModelDetails.weights`).
3. Implement Hugging Face resolver: list/resolve repository files, preserve ACORN `--filter` regex semantics, convert safe regex to HF include-style globs.
4. Support direct HTTP(S) model weight URLs as fallback.
5. Prefer `.gguf` weights by default for `llama.cpp`, `llama-swap`, Gold.
6. Resolve auth tokens from `HF_TOKEN`, `HF_API_TOKEN`, `HUGGINGFACE_HUB_TOKEN` via `acorn::io::first_env_var`.
7. Download through `.part` files, verify size/checksum, atomic rename.
8. Skip existing files matching expected size/checksum.
9. Typed errors for unknown models, ambiguous selections, missing auth, failed HTTP, incomplete downloads.

#### Configuration Shape

```json
{
  "models": [
    {
      "name": "smollm2",
      "source": {
        "type": "huggingFace",
        "repository": "HuggingFaceTB/SmolLM2-1.7B-Instruct-GGUF",
        "revision": "main"
      },
      "outputPath": "smollm2",
      "auth": "optional",
      "filter": ["\\.gguf$"],
      "ignore": ["Q2_", "Q3_"]
    }
  ]
}
```

Suggested types: `ModelDownloadEntry`, `ModelSource`, `ModelAuthRequirement`, `ModelFileRule`.

#### Resolver Order

1. Exact config `models[].name`.
2. Exact `ModelRow.model_id` from local database.
3. Exact `ModelDetails.id` from models.dev (network allowed).
4. Case-insensitive alias (single result only).
5. Direct URL fallback.
6. Error on zero or multiple matches.

#### Default Weight Selection

Precedence: User filter > `.gguf` > `.safetensors` > `.bin` > `.pt`/`.pth`.

Quantization for multiple `.gguf`: explicit filter > `Q4_K_M` default > smallest if exactly one candidate > ambiguity error.

#### Filter Semantics

- `--filter` accepts regex patterns; matches against repository-relative paths.
- Config `filter` and CLI `--filter` combined as additional regex constraints.
- Safe regex-to-glob conversion: `\\.gguf$` → `*.gguf`, `Q4_K_M.*\\.gguf$` → `*Q4_K_M*.gguf`.
- Unsupported regex falls back to listing repository files and applying regex locally.

#### Hugging Face Support

- No Python or `hf` CLI dependency.
- Direct file URLs: `https://huggingface.co/{org}/{repo}/resolve/{revision}/{path}`.
- Repository listing via HF APIs; default revision `main`.

#### Auth

```rust
first_env_var(&["HF_TOKEN", "HF_API_TOKEN", "HUGGINGFACE_HUB_TOKEN"])
```

- Public models work without tokens.
- Gated/private models return clear missing-auth error on 401/403.
- Tokens never appear in CLI args, logs, errors, or database rows.

#### Transfer Reliability

1. Download to `{target}.part`, flush/close, verify size/checksum, atomic rename.
2. Remove partial files on failure.
3. Stage 1: safe partial + skip-existing. Stage 2: HTTP range resume. Stage 3: checksum from manifests.

#### Offline Behavior

- Local/config resolution and database lookup allowed offline.
- Remote catalog refresh and remote downloads blocked offline.

#### Test Coverage (Phase 2)

1. Hugging Face repository resolution.
2. Direct URL downloads.
3. `--filter` regex behavior matching other ACORN commands.
4. Default `.gguf` selection and quantization ambiguity handling.
5. Output path construction and path traversal prevention.
6. Skip-existing behavior.
7. Token environment resolution and redacted log output.
8. Partial download cleanup.
9. Offline mode blocking remote operations.
10. Config `outputPath` relative vs absolute resolution.

#### Completion Criteria (Phase 2)

1. `acorn download model smollm2` downloads preferred `.gguf` or reports candidates when ambiguous.
2. Existing files skipped safely by default or with explicit flag.
3. Private/gated HF models downloadable via environment tokens.
4. No Python tooling dependency.
5. Direct URL downloads bypass format selection.
6. Default model root `$HOME/.models`.

---

## Reference Inventories

These sections support planning and fixture maintenance. They are intentionally separated from the actionable refactor backlogs above.

### OpenAI OpenAPI Remaining Endpoint Templates

Snapshot details:
1. Source spec: https://raw.githubusercontent.com/openai/openai-openapi/refs/heads/master/openapi.yaml
2. Total path templates in spec: 162
3. Path templates currently covered: 9
4. Remaining: 153

<details>
<summary>Expand full list (153 templates)</summary>

#### assistants (2)
1. /assistants
2. /assistants/{assistant_id}

#### audio (6)
1. /audio/speech
2. /audio/transcriptions
3. /audio/translations
4. /audio/voice_consents
5. /audio/voice_consents/{consent_id}
6. /audio/voices

#### batches (3)
1. /batches
2. /batches/{batch_id}
3. /batches/{batch_id}/cancel

#### chatkit (5)
1. /chatkit/sessions
2. /chatkit/sessions/{session_id}/cancel
3. /chatkit/threads
4. /chatkit/threads/{thread_id}
5. /chatkit/threads/{thread_id}/items

#### completions (1)
1. /completions

#### containers (5)
1. /containers
2. /containers/{container_id}
3. /containers/{container_id}/files
4. /containers/{container_id}/files/{file_id}
5. /containers/{container_id}/files/{file_id}/content

#### conversations (4)
1. /conversations
2. /conversations/{conversation_id}
3. /conversations/{conversation_id}/items
4. /conversations/{conversation_id}/items/{item_id}

#### embeddings (1)
1. /embeddings

#### evals (6)
1. /evals
2. /evals/{eval_id}
3. /evals/{eval_id}/runs
4. /evals/{eval_id}/runs/{run_id}
5. /evals/{eval_id}/runs/{run_id}/output_items
6. /evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}

#### files (3)
1. /files
2. /files/{file_id}
3. /files/{file_id}/content

#### fine_tuning (11)
1. /fine_tuning/alpha/graders/run
2. /fine_tuning/alpha/graders/validate
3. /fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions
4. /fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions/{permission_id}
5. /fine_tuning/jobs
6. /fine_tuning/jobs/{fine_tuning_job_id}
7. /fine_tuning/jobs/{fine_tuning_job_id}/cancel
8. /fine_tuning/jobs/{fine_tuning_job_id}/checkpoints
9. /fine_tuning/jobs/{fine_tuning_job_id}/events
10. /fine_tuning/jobs/{fine_tuning_job_id}/pause
11. /fine_tuning/jobs/{fine_tuning_job_id}/resume

#### images (3)
1. /images/edits
2. /images/generations
3. /images/variations

#### moderations (1)
1. /moderations

#### organization (46)
1-46: /organization/... (admin_api_keys, audit_logs, certificates, costs, groups, invites, projects, roles, usage, users)

#### projects (6)
1-6: /projects/{project_id}/... (groups, roles, users)

#### realtime (9)
1-9: /realtime/... (calls, client_secrets, sessions, transcription_sessions, translations)

#### responses (2)
1. /responses/compact
2. /responses/input_tokens

#### skills (6)
1-6: /skills/{skill_id}/... (content, versions)

#### threads (11)
1-11: /threads/{thread_id}/... (messages, runs, steps, submit_tool_outputs)

#### uploads (4)
1-4: /uploads/... (cancel, complete, parts)

#### vector_stores (10)
1-10: /vector_stores/{vector_store_id}/... (file_batches, files, search)

#### videos (8)
1-8: /videos/... (characters, edits, extensions)

</details>

### CKAN Fixture Conversion to DCAT v3

This section documents the rules for converting CKAN API format test fixtures (in `tests/fixtures/schema/ckan/`) to DCAT v3 compliant JSON (in `tests/fixtures/schema/dcat/`).

#### Field Mapping CKAN → DCAT v3

| CKAN field path | DCAT v3 path | Transformation |
|---|---|---|
| `title` | `title` | Wrap in array |
| `notes` (HTML) | `description` | Strip HTML tags, wrap in array |
| `url` | `landingPage` | Wrap in array |
| `identifier`, `guid`, `id` | `identifier` | Prefer custom `identifier`, then `guid`, then `id` |
| `issued`, `metadata_created` | `issued` | Prefer custom/top-level `issued` |
| `modified`, `metadata_modified` | `modified` | Prefer custom/top-level `modified` |
| `tags[].name` | `keywords` | Extract `.name` into array |
| `publisher[0]`, `organization` | `publisher` | Use first explicit publisher; fall back to `organization` |
| `creator[]` | `creator[]` | Each → `Agent { name }`; normalize ORCID identifiers |
| `contact[].email`, etc. | `contactPoint` | Format as `mailto:` URI |
| `qualified_relation[]` | `qualifiedRelation[]` | Use `uri` as `relation`; `role` as `hadRole` |
| `resources[]` | `distribution[]` | Each → `Distribution` |

#### Fixture Files

| Source (ckan/) | Target (dcat/) | Content |
|---|---|---|
| `0sbdv-gh024.json` | `0sbdv-gh024.json` | Full metadata, no resources |
| `6rpmk-n3x82.json` | `6rpmk-n3x82.json` | Minimal, fallback behavior |
| `j5xbm-d7v43.json` | `j5xbm-d7v43.json` | Medium, multiple creators |

#### Verification

```bash
cargo run --bin acorn -- check ./tests/fixtures/schema/dcat/ --standard dcat
```
