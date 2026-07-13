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
10. [Reference Inventories](#reference-inventories)
11. [Model Download Implementation](#model-download-implementation)

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
| Model Download | Implement `acorn download model` weight downloads with GGUF-first defaults | Medium | Critical | [Model Download Implementation](#model-download-implementation) — Phase 1 complete, GGUF-first phases planned |
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

---

## Model Download Implementation

This section tracks ACORN-side work to make `acorn download model` the primary backend for downloading model weights used by Gold, `llama.cpp`, `llama-swap`, and local inference environments. The default behavior is GGUF-first: `acorn download model <repo-or-model>` should prefer `*.gguf` files because this command is primarily meant for `llama.cpp`-compatible local inference.

### Current State

- `acorn-cli` exposes `download model` with config-driven resolution and positional-first deduplication.
- `ApplicationConfiguration` supports model entries through `models: Option<Vec<ModelEntry>>`, including bare selectors and detailed entries.
- `--filter` and `--ignore` are parsed and passed into `download::model::run`, but the implementation currently ignores them.
- Remote weight download is not implemented yet; remote selectors currently resolve/validate output directories.
- `acorn-lib/src/schema/agent/mod.rs` exposes `ModelDetails.weights`, `acorn-lib/src/io/database/schema.rs` persists `ModelRow.weights`, and `acorn-lib/src/io/api/models_dev.rs` downloads and caches the models.dev catalog.

### Phase 1: Config + CLI Foundation (Complete)

- Config files can declare model selectors and detailed model entries.
- `--config` / `-c` resolves application config and merges with positional models.
- Positional-first deduplication keeps positional models ahead of config entries.
- Default output directory resolves to `$HOME/.models` when `--output` is omitted.
- Models are validated against the local database when possible, with clear continuation messages when metadata is unavailable.
- Offline mode rejects remote model selectors before any remote operation.
- Existing tests cover config parsing, local model handling, model resolution, offline rejection, and CLI parsing.

### Phase 2: Selector Filtering

Goal: make the existing `--filter` and `--ignore` flags effective before full remote downloads are implemented.

Tasks:
1. Compile `--filter` and `--ignore` as regex patterns using semantics consistent with content downloads.
2. Apply selector-level filtering after config/positional merge and deduplication, before offline checks and model resolution.
3. Match local sources, direct URLs, remote repository IDs, and unsupported selectors against `Source::identifier()`.
4. Treat empty `--filter` as include-all and non-empty `--filter` as include-if-any-pattern-matches.
5. Treat `--ignore` as exclude-if-any-pattern-matches.
6. Return clear errors for invalid regex patterns.
7. Return a specific empty-selection error when all selectors are removed by `--filter`, `--ignore`, or `--whitelist`.

Acceptance criteria:
1. `acorn download model repo/name --filter name` keeps the selector.
2. `acorn download model repo/name --ignore name` removes the selector and reports no matching models.
3. Invalid regex input returns an actionable error.
4. Existing local copy/reference behavior remains unchanged when no filters are provided.

### Phase 3: Download Planning Types

Goal: separate model selection, file planning, and transfer execution into testable units.

Tasks:
1. Define `ModelDownloadPlan` with source repository or URL, selected files, output directory, auth requirement, expected size, checksum, and transfer mode.
2. Define file-level metadata types for repository-relative path, resolved URL, size, checksum, format, and quantization.
3. Define typed errors for unknown models, multiple matches, no matching files, ambiguous quantization, missing auth, failed HTTP, path traversal, and incomplete downloads.
4. Resolve selectors in this order: exact config `models[].name`, exact `ModelRow.model_id`, exact `ModelDetails.id` from models.dev when network is allowed, single case-insensitive alias match, direct URL fallback, then explicit error.
5. Keep direct HTTP(S) file URLs as a fallback path that bypasses repository format selection.

Suggested types: `ModelDownloadEntry`, `ModelSource`, `ModelAuthRequirement`, `ModelFileRule`, `ModelDownloadPlan`, `ModelFileCandidate`, `ModelDownloadError`.

Configuration shape:

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

### Phase 4: Hugging Face Resolution And GGUF Selection

Goal: resolve Hugging Face repository files without requiring Python or the `hf` CLI, while mirroring the useful parts of `hf download --include` and `hf download --exclude`.

Tasks:
1. Implement Hugging Face repository listing through HF APIs with default revision `main`.
2. Construct direct file URLs as `https://huggingface.co/{org}/{repo}/resolve/{revision}/{path}`.
3. Preserve ACORN regex semantics for `--filter` and config `filter`, matching repository-relative paths.
4. Preserve ACORN regex semantics for `--ignore` and config `ignore`, matching repository-relative paths.
5. Convert safe regex patterns to HF-style include/exclude globs when possible, such as `\\.gguf$` to `*.gguf` and `Q4_K_M.*\\.gguf$` to `*Q4_K_M*.gguf`.
6. Fall back to listing repository files and applying regex locally when regex-to-glob conversion is unsafe.
7. Keep full-repo download as an explicit future capability, but do not make it the default for `acorn download model`.

Default selection rules:
1. If a user supplies `--filter` or config `filter`, select only matching files after applying ignores.
2. Without user filters, prefer `*.gguf` files.
3. If multiple GGUF files exist, prefer `Q4_K_M`.
4. If no `Q4_K_M` exists and exactly one GGUF file remains, select it.
5. If multiple GGUF candidates remain, return an ambiguity error listing candidates and suggesting `--filter`.
6. If no GGUF files exist, report no GGUF candidates and suggest a GGUF repo or explicit `--filter` for non-GGUF formats.
7. Direct file URLs bypass GGUF preference because the user already selected a file.

Format fallback policy:
1. GGUF is the default and preferred format for this command.
2. `.safetensors`, `.bin`, `.pt`, and `.pth` are selected only when the user explicitly filters for them or a future explicit mode asks for Transformers/PyTorch behavior.
3. Full Hugging Face repo download is safest for Transformers usage, but it is not the default for this command.

### Phase 5: Transfer Reliability And Auth

Goal: make downloads safe, restart-tolerant, and secure for public, gated, and private model repositories.

Tasks:
1. Resolve Hugging Face auth tokens from `HF_TOKEN`, `HF_API_TOKEN`, and `HUGGINGFACE_HUB_TOKEN` through `acorn::io::first_env_var`.
2. Allow public model downloads without tokens.
3. Return a clear missing-auth error for 401/403 responses from gated or private models.
4. Never print tokens in CLI args, logs, errors, database rows, or debug output.
5. Download to `{target}.part`, flush and close the file, verify expected size/checksum when available, then atomically rename.
6. Remove partial files on failure.
7. Skip existing files when size/checksum matches expected metadata.
8. Prevent path traversal by resolving repository paths under the selected model output directory only.
9. Keep HTTP range resume as a later reliability stage after safe partial downloads and skip-existing behavior are complete.

Offline behavior:
1. Local/config resolution and database lookup are allowed offline.
2. Remote catalog refresh, Hugging Face repository listing, and remote downloads are blocked offline.
3. Direct remote URLs are rejected offline before any network request.

### Phase 6: CLI Help, Documentation, And Validation

Goal: make the behavior understandable and testable from the CLI.

Tasks:
1. Document that `acorn download model` is GGUF-first by default for `llama.cpp`.
2. Document that `--filter` behaves like an include rule and `--ignore` behaves like an exclude rule, while remaining regex-based rather than HF glob-based.
3. Add examples for downloading the default GGUF, a specific quantization, and excluding low-quality quantizations.
4. Add a note that ACORN does not require Python or `hf`, even though behavior maps conceptually to `hf download --include` and `hf download --exclude`.
5. Add a note that Transformers/PyTorch users generally need a full repo or a broader include set containing weights, config, tokenizer, processor, chat template, and optional custom code files.

Example commands:

```bash
acorn download model HuggingFaceTB/SmolLM2-1.7B-Instruct-GGUF
acorn download model HuggingFaceTB/SmolLM2-1.7B-Instruct-GGUF --filter "Q4_K_M.*\\.gguf$"
acorn download model HuggingFaceTB/SmolLM2-1.7B-Instruct-GGUF --ignore "Q2_|Q3_"
```

### Hugging Face Reference Notes

The Hugging Face `hf` CLI does not require `--include` for full repository downloads:

```bash
hf download <org-or-user>/<model-name> --local-dir ./models/<model-name>
```

Use `--include` for selected files or glob patterns:

```bash
hf download <org-or-user>/<model-name> --local-dir ./models/<model-name> --include "*.safetensors" "config.json" "tokenizer.*"
```

Use `--exclude` to omit files:

```bash
hf download <org-or-user>/<model-name> --local-dir ./models/<model-name> --exclude "*.bin"
```

For ACORN, `--filter` maps conceptually to include behavior and `--ignore` maps conceptually to exclude behavior, but ACORN accepts regex patterns and should not shell out to `hf`.

Common Transformers-style model files:

| File | Purpose |
|---|---|
| `README.md` | Model card and usage notes |
| `config.json` | Model architecture and hyperparameters |
| `generation_config.json` | Default generation settings |
| `*.safetensors` | Modern model weight format |
| `pytorch_model.bin` | Older PyTorch weight format |
| `*.index.json` | Index for sharded weight files |
| `tokenizer.json` | Fast tokenizer definition |
| `tokenizer_config.json` | Tokenizer metadata |
| `special_tokens_map.json` | BOS/EOS/PAD/etc token mappings |
| `vocab.json` | Token vocabulary, often used with BPE tokenizers |
| `merges.txt` | BPE merge rules |
| `spiece.model` | SentencePiece tokenizer model |
| `chat_template.jinja` | Chat prompt formatting template |
| `preprocessor_config.json` | Vision/audio preprocessing config |
| `processor_config.json` | Multimodal processor config |
| `adapter_config.json` | LoRA/PEFT adapter config |
| `adapter_model.safetensors` | Adapter weights |

BPE means Byte Pair Encoding. Hugging Face BPE tokenizers commonly use `vocab.json`, `merges.txt`, and `tokenizer.json`.

GGUF is the native model format used by `llama.cpp`. For `llama.cpp`, users usually do not need PyTorch or Safetensors files and should focus on `*.gguf`.

Common GGUF quantization choices:

| Quant | Use case |
|---|---|
| `Q4_K_M` | Good default balance of quality and size |
| `Q5_K_M` | Better quality, larger file |
| `Q6_K` | Higher quality, much larger file |
| `Q8_0` | Near full quality, very large file |
| `F16` / `BF16` | Full-ish precision, huge file |
| `Q2_K`, `Q3_K_*` | Smaller, lower quality |

Example `llama.cpp` run after download:

```bash
llama-cli -m ./models/<model-name>/model.Q4_K_M.gguf -p "Hello"
```

### Test Coverage

1. Selector-level `--filter` and `--ignore` behavior in the current command path.
2. Invalid regex reporting for selector and repository-file filtering.
3. Hugging Face repository resolution and repo-relative path filtering.
4. Safe regex-to-glob conversion and fallback local filtering.
5. GGUF default selection when no user filter is supplied.
6. `Q4_K_M` quantization preference among multiple GGUF files.
7. Ambiguity error when multiple GGUF candidates remain.
8. Explicit non-GGUF filtering for `.safetensors`, `.bin`, `.pt`, and `.pth`.
9. Direct URL downloads bypassing format selection.
10. Output path construction and path traversal prevention.
11. Skip-existing behavior using size/checksum metadata.
12. Token environment resolution and redacted log output.
13. Partial download cleanup.
14. Offline mode blocking remote listing and downloads.
15. Config `outputPath` relative vs absolute resolution.

### Completion Criteria

1. `acorn download model <repo>` downloads the preferred GGUF file by default or reports actionable candidates when ambiguous.
2. Existing `--filter` and `--ignore` flags are honored for model selectors and repository-relative file paths.
3. `Q4_K_M` is selected by default when multiple GGUF quantizations are available.
4. Direct file URLs download exactly the requested file and bypass format preference.
5. Private and gated Hugging Face models work through environment tokens without leaking secrets.
6. Existing files are skipped safely by default when metadata matches.
7. Failed transfers do not leave final-looking corrupt files.
8. No Python tooling or `hf` CLI dependency is required.
9. Default model root remains `$HOME/.models`.
