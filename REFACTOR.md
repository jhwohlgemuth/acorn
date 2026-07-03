# ACORN Refactor Recommendations

This document captures cross-crate refactor and hardening recommendations for acorn-cli and acorn-lib, organized for both strategic overview and detailed implementation guidance.

---

## Table of Contents

1. [Overview & Goals](#overview--goals)
2. [Suggested Execution Order](#suggested-execution-order)
3. [Risk Assessment](#risk-assessment)
4. [Original Detailed Recommendations](#original-detailed-recommendations)

## Overview & Goals

The ACORN refactor initiative is organized around four primary goals:

1. **Improve Reliability** — Eliminate crash-prone code paths and improve error handling
2. **Enhance Maintainability** — Modularize the codebase and eliminate duplication
3. **Increase Extensibility** — Improve architectural boundaries and abstractions
4. **Expand Compatibility** — Enable no-std and WASM support

## Suggested Execution Order

The refactor work follows a phased approach with clear dependencies between stages:

### Phase 1 — Reliability triage (weeks 1-4) ✅ COMPLETE
Remove panic/unimplemented/exit-heavy internal command flow and normalize command/helper `Result` contracts.

### Phase 2 — Boundary hardening (weeks 5-8)
Harden OOXML and path handling, fail fast on asset lookup and invalid filters.

### Phase 3 — Shared abstractions (weeks 9-12)
Introduce `ExportContext`, `SafePath`, unified interpolation utility.

### Phase 4 — Structural modularization (weeks 13-20)
Split `main` setup concerns, tighten API client and schema I/O contracts.

### Phase 5 — Refinement (weeks 21+)
Binding-layer macroization, no-std/WASM enablement, lower-priority cleanup.

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

## Scope

Included:
- `acorn-cli` command flow, startup orchestration, export/template behavior
- `acorn-lib` I/O, schema serialization, interpolation, analyzer/API seams
- Cross-crate boundaries where duplicate logic or weak contracts exist

Excluded:
- Generated docs outputs
- Style-only cleanup with no reliability/security/modularity payoff

## Critical Impact

### 3. Harden OOXML mutation boundary in PowerPoint export
- Difficulty: Medium
- Why: Regex/string-based OOXML manipulation is brittle and can produce malformed PPTX content.
- Evidence:
  - `acorn-cli/src/commands/export/powerpoint.rs`
  - `acorn-lib/src/io/powerpoint/mod.rs`
- Recommendation:
  1. Shift critical edits to parser-backed XML transforms.
  2. Validate well-formed XML before archive write.
  3. Centralize OOXML utility helpers to avoid split behavior.

## High Impact

### 4. Fail fast on missing export/template assets
- Difficulty: Low
- Why: Silent empty-string fallbacks hide output corruption.
- Evidence:
  - `acorn-cli/src/template/mod.rs`
- Recommendation:
  1. Return `Result` from asset lookup/embedding paths.
  2. Bubble failures into export command errors.
  3. Add one integration check for required embedded assets.

### 5. Eliminate duplicate export field extraction across HTML and PPTX paths
- Difficulty: Medium
- Why: Same `ResearchActivity` fields are mapped in multiple places, increasing drift risk.
- Evidence:
  - `acorn-cli/src/template/mod.rs`
  - `acorn-lib/src/io/powerpoint/mod.rs`
- Recommendation:
  1. Introduce shared `ExportContext` (library-side).
  2. Extract fields once and consume in all format-specific renderers.
  3. Validate required placeholders up front.

### 6. Split CLI startup concerns out of `main`
- Difficulty: Medium
- Why: Logging/env/runtime/database/bootstrap/dispatch concerns are tightly mixed.
- Evidence:
  - `acorn-cli/src/main.rs`
- Recommendation:
  1. Extract setup units: logging, runtime/thread pool, database bootstrap, command dispatch.
  2. Keep main as orchestration glue with uniform error boundary.

## Medium Impact

### 8. Consolidate interpolation primitives under one shared utility
- Difficulty: Low
- Why: Placeholder logic is split between generic interpolation and PowerPoint-specific behavior.
- Evidence:
  - `acorn-lib/src/util/mod.rs`
  - `acorn-lib/src/io/powerpoint/mod.rs`
- Recommendation:
  1. Extend shared interpolation trait(s) for list/bullet use-cases.
  2. Remove parallel interpolation stacks.

### 9. Introduce stronger shared API-client contract
- Difficulty: Medium
- Why: API modules repeat health/auth/endpoint patterns without a fully unified contract.
- Evidence:
  - `acorn-lib/src/io/api/mod.rs`
  - `acorn-lib/src/io/api/raid.rs`
  - `acorn-lib/src/io/api/gitlab.rs`
- Recommendation:
  1. Expand common trait surface for health, auth, endpoint rendering, and error normalization.
  2. Move duplicated logic behind shared helper modules.

### 10. Reduce binding-layer repetition in Python wrapper
- Difficulty: Medium
- Why: Wrapper boilerplate increases maintenance cost and schema drift risk.
- Evidence:
  - `acorn-py/src/lib.rs`
- Recommendation:
  1. Complete data exposure shape first.
  2. Then apply macro-based codegen for repetitive PID wrapper patterns.

## Lower Impact

### 11. Tighten schema I/O and format validation handoff between CLI and library
- Difficulty: High
- Why: CLI and library each carry partial format assumptions and validation behavior.
- Evidence:
  - `acorn-lib/src/io/mod.rs`
  - `acorn-lib/src/schema/research_activity/mod.rs`
  - `acorn-cli/src/commands/export/mod.rs`
- Recommendation:
  1. Extend `InputOutput` with stricter read/validate variants.
  2. Make CLI rely on library-side strict format validation consistently.

### 13. Separate parsed CLI model from execution options model
- Difficulty: Medium
- Why: Routing and options conversion remain repetitive and tightly coupled.
- Evidence:
  - `acorn-cli/src/cli/mod.rs`
  - `acorn-cli/src/main.rs`
- Recommendation:
  1. Keep clap parse structs isolated.
  2. Normalize conversion to execution options in one layer.

## Cross-Crate Abstractions To Build First

1. `ExportContext` in `acorn-lib` used by both CLI template and lib PowerPoint export.
2. `SafePath` utility in `acorn-lib` consumed by CLI path-resolution and export file operations.
3. Unified interpolation extension in `acorn-lib` for string + list placeholders.
4. Single CLI error boundary mapping internal `Result` errors to exit codes.

## Suggested Migration Order

1. Reliability triage: ✅ COMPLETE
   - Remove panic/unimplemented/exit-heavy internal command flow.
   - Normalize command/helper `Result` contracts.
2. Boundary hardening:
   - Harden OOXML and path handling.
   - Fail fast on asset lookup and invalid filters.
3. Shared abstractions:
   - Introduce `ExportContext`, `SafePath`, unified interpolation utility.
4. Structural modularization:
   - Split `main` setup concerns.
   - Tighten API client and schema I/O contracts.
5. Refinement:
   - Binding-layer macroization and lower-priority cleanup.

## Completion Criteria For Refactor Work

1. No production `unimplemented!()`/`panic!()` in command paths. ✅
2. Export paths fail with typed errors, never silent corruption. ✅
3. Path normalization and containment are centralized and reused.
4. Export field mapping exists in one shared context object.
5. Cross-crate interpolation behavior is unified and test-covered.

## acorn-lib no-std + WASM Maximization Plan

This section focuses on positioning `acorn-lib` as a high-capability no-std-first library with strong WASM support.

Prioritization rule:
1. Impact on no-std correctness and portability first.
2. Difficulty second within each impact tier.

### Current Validation Snapshot

1. `cargo check -p acorn-lib --no-default-features` currently fails.
2. Observed blocker: analyzer paths compile while analyzer exports are feature-gated, producing unresolved `Check`/`CheckSeverity` references.
3. `cargo check -p acorn-lib --no-default-features --target wasm32-unknown-unknown` cannot be validated yet in this environment because the target is not installed.

### Critical Impact

#### 1. Fix feature topology: gate analyzer module at crate boundary
- Difficulty: Low
- Why: no-default-features build currently fails due to inconsistent analyzer gating.
- Evidence:
  - `acorn-lib/src/lib.rs` exports `pub mod analyzer;` unconditionally.
  - `acorn-lib/src/analyzer/mod.rs` uses types exported only under `#[cfg(feature = "analysis")]`.
- Recommendation:
  1. Gate analyzer module in `lib.rs` behind `feature = "analysis"`.
  2. Keep non-analysis link-check and analysis-only codepaths separated.
  3. Ensure `no-default-features` compiles before any dependency work.

#### 2. Split acorn-lib into capability layers (`core`, `alloc`, `std`)
- Difficulty: Medium
- Why: current feature model mixes environment concerns (`std`) with domain concerns (`analysis`, `powerpoint`, `doctor`).
- Evidence:
  - `acorn-lib/Cargo.toml` uses `default = ["analysis", "agentic", "std"]`.
  - `analyzer`, `schema`, and `util` pull mixed dependency classes.
- Recommendation:
  1. Introduce explicit layering features:
     - `core` (no_std + no_alloc feasible subset)
     - `alloc` (string/vec/hashmap support)
     - `std` (filesystem, process, network, threads)
  2. Make domain features depend on capability features explicitly (example: `analysis = ["alloc", "std", ...]`).
  3. Add compile matrix checks for each layer.

#### 3. Isolate std-only modules from shared domain modules
- Difficulty: Medium
- Why: std-bound imports leak into major surfaces, limiting no-std portability.
- Evidence:
  - `acorn-lib/src/analyzer/mod.rs` imports `crate::io::*`, process APIs, filesystem-like behaviors.
  - `acorn-lib/src/io/mod.rs` is std-heavy and currently broad.
- Recommendation:
  1. Move std-heavy logic into `io/std/*` and keep pure transformations in `io/core/*`.
  2. Keep schema parsing/validation and formatting logic independent from command/process/network concerns.
  3. Define trait boundaries for host operations (FS, HTTP, clock, randomness, process).

### High Impact

#### 4. Make heavy std dependencies optional and feature-scoped
- Difficulty: Low
- Why: several crates are currently unconditional despite being host-only in practice.
- Evidence:
  - In `acorn-lib/Cargo.toml`, dependencies such as `docx-rs`, `directories`, `indicatif`, `color-eyre`, `rayon`, `percy-dom`, `tera`, `rusqlite`, `dotenvy`, `flate2`, `zip` are not uniformly tied to capability features.
- Recommendation:
  1. Mark host-only crates optional and gate by `std`/format-specific features.
  2. Keep only core no-std-compatible crates mandatory (serde, schemars where needed, hashbrown, alloc-safe helpers).
  3. Add a policy: every non-alloc dependency must declare its enabling feature.

#### 5. Replace no-std-hostile error/reporting surface for core builds
- Difficulty: Medium
- Why: `color-eyre` is excellent for CLI UX but not ideal as core-library baseline for no-std/WASM.
- Evidence:
  - `acorn-lib/src/io/mod.rs` and other modules use `ApiResult<T> = Result<T, Report>`.
- Recommendation:
  1. Introduce a core error enum (`AcornError`) with `thiserror`/`core::fmt` support.
  2. Keep `color-eyre` conversion in std adapters only.
  3. Make `ApiResult` alias conditional:
     - no-std: `Result<T, AcornError>`
     - std CLI adapters: map to rich reports.

#### 6. Add dedicated WASM feature path (not just no-std)
- Difficulty: Medium
- Why: WASM has different constraints than bare-metal no-std (target-specific panic, allocator, networking, fs absence).
- Recommendation:
  1. Add `wasm` feature with wasm-safe dep toggles.
  2. Provide `wasm-bindgen`/`js-sys` adapters in a separate module and avoid pulling browser crates into non-wasm no-std builds.
  3. Define host traits for time/random/http and implement wasm adapters.

### Medium Impact

#### 7. Refactor prelude into strict capability preludes
- Difficulty: Low
- Why: prelude currently provides both std and no-std exports, but module callers still mix concerns.
- Evidence:
  - `acorn-lib/src/prelude.rs` has dual exports, but call sites in analyzer/util/io often assume std APIs.
- Recommendation:
  1. Create explicit `prelude::core`, `prelude::alloc`, `prelude::std` modules.
  2. Restrict imports by module capability level to prevent accidental std usage.

#### 8. Partition utility module into pure and std submodules
- Difficulty: Medium
- Why: `util/mod.rs` includes table formatting, console emoji, diff rendering, embedding support, and generic helpers together.
- Evidence:
  - `acorn-lib/src/util/mod.rs` imports `comfy_table`, `console`, `owo-colors`, `rust_embed` unconditionally at module scope.
- Recommendation:
  1. Move pure string/identifier/interpolation helpers into `util/core`.
  2. Move console formatting/rendering into `util/std`.
  3. Re-export conditionally.

#### 9. Introduce host-abstraction traits for runtime services
- Difficulty: Medium
- Why: file/network/process/runtime dependencies are currently concrete, reducing portability.
- Recommendation:
  1. Define traits: `FileStore`, `HttpClient`, `Clock`, `Entropy`, `ProcessRunner`.
  2. Provide std implementations in adapters.
  3. Provide wasm/no-std implementations where feasible.

### Lower Impact

#### 10. Tighten workspace dependency defaults for no-std friendliness
- Difficulty: Medium
- Why: workspace-level defaults currently include `chrono/std` and other std-oriented defaults that can leak into no-std planning.
- Evidence:
  - `Cargo.toml` workspace dependency for `chrono` includes `features = ["serde", "std"]`.
- Recommendation:
  1. Move std-dependent feature toggles from workspace defaults into crate-level feature wiring.
  2. Prefer alloc/core-only defaults in workspace where possible.

#### 11. Add no-std and wasm CI matrix as a first-class quality gate
- Difficulty: Low
- Why: regressions are likely without continuous compile checks.
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

### Practical Execution Steps

1. Stage 1 (stability): fix analyzer feature-gate break so `--no-default-features` compiles.
2. Stage 2 (dependency hygiene): make host-heavy deps optional and gate them explicitly.
3. Stage 3 (architecture): split pure/domain code from std adapters, add host traits.
4. Stage 4 (WASM): add dedicated `wasm` feature and adapter implementations.
5. Stage 5 (enforcement): add CI matrix and feature-leak checks.

### Completion Criteria for no-std/WASM Positioning

1. `acorn-lib` builds with `--no-default-features`.
2. `acorn-lib` builds for `wasm32-unknown-unknown` with wasm feature path.
3. Core schema and validation APIs remain usable without std.
4. std-only concerns live behind adapter modules and explicit features.
5. CI enforces no-std + wasm compile guarantees.

## no-std/WASM Implementation PR Plan (Concrete)

This is an implementation-oriented sequence designed to be executed as small, reviewable PRs.

### PR 4: Separate Core Domain Logic from std Adapters

Goal:
- Keep pure domain/schema/transform code portable while isolating host concerns.

Primary edits:
1. Introduce module splits:
  - `acorn-lib/src/io/core/` for pure data transforms
  - `acorn-lib/src/io/std/` for filesystem/network/runtime/process operations
  - `acorn-lib/src/util/core/` and `acorn-lib/src/util/std/` split where needed

2. `acorn-lib/src/prelude.rs`
  - Add capability-scoped preludes:
    - `prelude::core`
    - `prelude::alloc`
    - `prelude::std`
  - Restrict imports in each module to avoid accidental std leakage.

Validation commands:
1. `cargo check -p acorn-lib --no-default-features --features alloc`
2. `cargo check -p acorn-lib --features std`

Acceptance criteria:
1. Core modules compile without std imports.
2. std-only APIs are contained under feature-gated adapter modules.

### PR 5: Introduce Host Traits + Implementations

Goal:
- Replace concrete runtime dependencies with pluggable host interfaces.

Primary edits:
1. Add new traits (suggested location: `acorn-lib/src/io/host.rs`):
  - `FileStore`
  - `HttpClient`
  - `Clock`
  - `Entropy`
  - `ProcessRunner`

2. Add implementations:
  - std impls in `io/std/host_impl.rs`
  - wasm impl stubs/adapters in `io/wasm/host_impl.rs`

3. Refactor selected callsites in analyzer/io/api to use trait abstractions.

Validation commands:
1. `cargo check -p acorn-lib --features std,analysis`
2. `cargo check -p acorn-lib --no-default-features --features wasm`

Acceptance criteria:
1. Host-sensitive logic no longer hardcodes std APIs in core paths.

### PR 6: WASM Target Enablement

Goal:
- Provide a first-class WASM-capable build mode.

Primary edits:
1. `acorn-lib/Cargo.toml`
  - Add target-specific dependency blocks for wasm where required.
  - Ensure wasm feature enables only wasm-compatible crates.

2. Add wasm-specific module wiring:
  - `acorn-lib/src/io/wasm/mod.rs`
  - optional wrappers for JS interop if required by public API.

Environment requirement:
1. Install target: `rustup target add wasm32-unknown-unknown`

Validation commands:
1. `cargo check -p acorn-lib --no-default-features --features wasm --target wasm32-unknown-unknown`
2. `cargo check -p acorn-lib --no-default-features --features alloc --target wasm32-unknown-unknown`

Acceptance criteria:
1. wasm checks pass for at least one useful feature set.

### PR 7: CI Enforcement + Leak Detection

Goal:
- Prevent future regressions in no-std and wasm support.

Primary edits:
1. `.github/workflows/` add dedicated jobs:
  - no-default-features core/alloc checks
  - wasm target checks
  - dependency leak checks (`cargo tree`) for no-default-features

2. Add feature matrix script under `scripts/` for local reproducibility.

Suggested CI command matrix:
1. `cargo check -p acorn-lib --no-default-features --features core`
2. `cargo check -p acorn-lib --no-default-features --features alloc`
3. `cargo check -p acorn-lib --no-default-features`
4. `cargo check -p acorn-lib --no-default-features --features wasm --target wasm32-unknown-unknown`
5. `cargo check -p acorn-lib --features std,analysis`

Acceptance criteria:
1. CI fails on feature leakage or no-std/wasm breakage.

## Dependency Replacement Candidates (Targeted)

Use replacement only when needed to achieve capability goals:

1. `color-eyre` for core library path
  - Impact: High
  - Difficulty: Medium
  - Action: keep `color-eyre` in std adapters; introduce core error type for no-std path.

2. `rayon` in analyzer/domain pathways
  - Impact: Medium
  - Difficulty: Medium
  - Action: gate strictly behind `analysis` + `std`; avoid in portable core.

3. `docx-rs`, `tera`, `percy-dom`, `indicatif`
  - Impact: Medium
  - Difficulty: Low
  - Action: isolate under export/template features; do not load in base alloc/no-std builds.

4. `chrono`
  - Impact: Medium
  - Difficulty: Low
  - Action: ensure `chrono/std` only enabled by std feature.

5. `rusqlite`, `reqwest`, `tokio`, `axum`, `tower`, `which`
  - Impact: High
  - Difficulty: Low
  - Action: keep optional and std-only.

## Analyzer Category-Theoretic Refactor Opportunities

This section focuses only on code in acorn-lib/src/analyzer and identifies improvements using constructs such as functor, traversable, monoid, and algebraic folds.

Sort order:
1. Difficulty first.
2. Impact second within each difficulty level.

### Low Difficulty

#### 1. Functor plus Traversable extraction for link-check futures
- Difficulty: Low
- Impact: High
- Evidence:
  - acorn-lib/src/analyzer/mod.rs
  - Cff website checks build multiple optional futures through repeated map, flatten, collect chains.
- Opportunity:
  1. Introduce a small helper based on functor and traversable style.
  2. Transform optional URL fields with map into optional futures.
  3. Traverse into one list of futures with a reusable combinator.
  4. Replace repeated root, identifier, and reference URL builders with one generic mapper.
- Why this helps:
  - Reduces repeated branching and keeps URL-to-Check transformation compositional.

#### 2. Monoid for Check collection accumulation
- Difficulty: Low
- Impact: High
- Evidence:
  - acorn-lib/src/analyzer/mod.rs
  - acorn-lib/src/analyzer/check.rs
  - Repeated join_all await flatten collect patterns.
- Opportunity:
  1. Introduce CheckBatch newtype with identity as empty list.
  2. Define associative combine as concatenation.
  3. Use fold_map style accumulation from paths and nested structures.
- Why this helps:
  - Makes accumulation strategy explicit and reusable across Cff, ResearchActivity, Docx, and Text analyzers.

#### 3. Monoid for summary aggregation
- Difficulty: Low
- Impact: Medium
- Evidence:
  - acorn-lib/src/analyzer/check.rs summary logic aggregates per-category counts via chained map and collect.
- Opportunity:
  1. Represent summary as a monoidal record where fields combine by addition.
  2. Fold checks directly into one summary value instead of building intermediates.
- Why this helps:
  - Cleaner associative aggregation and easier extension for new categories.

#### 4. Endomorphism monoid for ValeConfig normalization pipeline
- Difficulty: Low
- Impact: Medium
- Evidence:
  - acorn-lib/src/analyzer/mod.rs ValeConfig package resolution and configuration mutations.
- Opportunity:
  1. Model each config transformation as function from ValeConfig to ValeConfig.
  2. Compose transformations and apply once.
- Why this helps:
  - Avoids ad hoc mutation sequencing and makes configuration transformation declarative.

### Medium Difficulty

#### 5. Catamorphism style fold for validation error trees
- Difficulty: Medium
- Impact: High
- Evidence:
  - acorn-lib/src/analyzer/error.rs process recursively matches Field, Struct, List with duplicated path-context handling.
- Opportunity:
  1. Define one algebra over ValidationErrorsKind nodes.
  2. Implement a generic fold carrying path context and returning flattened ValidatorIssue values.
- Why this helps:
  - Centralizes recursion semantics and reduces drift in error flattening behavior.

#### 6. Applicative validation for composed checks
- Difficulty: Medium
- Impact: High
- Evidence:
  - acorn-lib/src/analyzer/mod.rs and acorn-lib/src/analyzer/check.rs mix success and error paths with repeated match scaffolding.
- Opportunity:
  1. Convert independent check computations into applicative composition where possible.
  2. Accumulate multiple failures in one pass rather than short-circuit branching.
- Why this helps:
  - Better fit for analyzer workflows that aim to collect all issues.

#### 7. Reader-like context functor for CheckOptions flow
- Difficulty: Medium
- Impact: Medium
- Evidence:
  - acorn-lib/src/analyzer/mod.rs repeatedly threads options and derived context through nested async closures.
- Opportunity:
  1. Introduce explicit analysis context and composable context-to-result functions.
  2. Use map and and_then style composition to remove repeated option plumbing.
- Why this helps:
  - Reduces ceremony and improves testability of pure transformations.

#### 8. Result functor normalization for conversion boundaries
- Difficulty: Medium
- Impact: Medium
- Evidence:
  - acorn-lib/src/analyzer/mod.rs and acorn-lib/src/analyzer/vale.rs contain repeated match blocks that map failures to defaults or fallback checks.
- Opportunity:
  1. Use map_err and recovery combinators through one consistent pipeline.
  2. Separate error policy mapping from business transformation logic.
- Why this helps:
  - Improves predictability and composability of error handling.

### High Difficulty

#### 9. Higher-level Traversable abstraction across standards
- Difficulty: High
- Impact: High
- Evidence:
  - acorn-lib/src/analyzer/mod.rs has near-isomorphic implementations for Cff, ResearchActivity, Docx, and Text check methods.
- Opportunity:
  1. Define generic traversal strategy over readable artifacts and check categories.
  2. Keep per-standard adapters only for reading and URL extraction.
- Why this helps:
  - Significant duplication reduction and lower cost to add new standards.

#### 10. Typed semiring-like scoring model for readability and prose checks
- Difficulty: High
- Impact: Medium
- Evidence:
  - acorn-lib/src/analyzer/readability/mod.rs and acorn-lib/src/analyzer/check.rs map scoring and severity via ad hoc conditionals.
- Opportunity:
  1. Introduce typed score algebra where combine and threshold are explicit.
  2. Map algebra outputs to CheckSeverity through one interpreter layer.
- Why this helps:
  - Provides a principled basis for combining heterogeneous quality signals.

### Recommended Implementation Order for This Section

1. Start with item 1 and item 2 to reduce duplication quickly in analyzer/mod.rs.
2. Follow with item 5 to simplify and stabilize validation error flattening.
3. Then apply item 9 if broad standard-level deduplication is desired.

## io::api Improvement Ideas (Sorted by Difficulty and Impact)

### Low Difficulty

#### 1. Standardize Auth Header Construction Across API Modules
- Difficulty: Low
- Impact: High
- Opportunity:
  1. Add one shared helper in `acorn-lib/src/io/api/mod.rs` for bearer token headers.
  2. Replace per-module duplicated header creation in `gitlab`, `raid`, and `openai`.
  3. Add focused tests for empty-token behavior and header map output.

#### 2. Add Shared Request Builders for Common API Shapes
- Difficulty: Low
- Impact: High
- Opportunity:
  1. Add utility helpers for `{token + template value}` and `{token + JSON body}` parameter sets.
  2. Remove repeated `param!` vector assembly boilerplate from individual modules.
  3. Keep module-specific logic focused on action names and response types.

#### 3. Normalize Environment Variable Discovery Rules
- Difficulty: Low
- Impact: Medium
- Opportunity:
  1. Define a consistent fallback sequence for token/domain variables across modules.
  2. Expose one helper for required vs optional secrets.
  3. Keep `.env` loading behavior uniform and testable.

### Medium Difficulty

#### 4. Introduce Shared API Error Envelope Traits
- Difficulty: Medium
- Impact: High
- Opportunity:
  1. Define a trait for extracting user-facing error strings from provider-specific error payloads.
  2. Implement for `gitlab`, `raid`, and `openai` error responses.
  3. Centralize fallback error rendering in `handle_or` helpers.

#### 5. Add OpenAPI-Aware Endpoint Validation Utilities
- Difficulty: Medium
- Impact: High
- Opportunity:
  1. Add utility functions that check endpoint template names and required template keys.
  2. Validate key mappings for action names at startup or in unit tests.
  3. Catch template drift early when `application.json` changes.

#### 6. Add Lightweight Response DTO Layer for Evolving APIs
- Difficulty: Medium
- Impact: Medium
- Opportunity:
  1. Keep baseline raw `serde_json::Value` support for broad schemas.
  2. Layer optional typed DTOs for stable subfields (`id`, `object`, usage metadata).
  3. Improve compile-time safety without losing forward compatibility.

### High Difficulty

#### 7. Generate API Module Skeletons from OpenAPI Specs
- Difficulty: High
- Impact: High
- Opportunity:
  1. Create an internal codegen path that emits endpoint definitions and baseline DTOs.
  2. Use generated modules as a starting point, then keep hand-written ergonomic wrappers.
  3. Reduce maintenance drift for rapidly evolving APIs.

#### 8. Unified Capability Matrix for io::api Providers
- Difficulty: High
- Impact: Medium
- Opportunity:
  1. Define one capability descriptor (`supports_pagination`, `supports_streaming`, `requires_token`, etc.).
  2. Drive command-layer behavior from capability metadata instead of per-provider conditionals.
  3. Enable cleaner extension as additional providers are introduced.

## io::api Reuse Opportunities (Module Scan)

This section captures concrete extraction opportunities identified by scanning the existing API modules (`openai`, `gitlab`, `raid`, `orcid`, `ror`, `citeas`, `geonames`, `spdx`) with a focus on code that can move into `acorn-lib/src/io/api/mod.rs` and be reused.

### Critical

#### 1. Complete HTTP method dispatch in root invoke path
- Difficulty: Low
- Impact: Critical
- Evidence:
  - `acorn-lib/src/io/api/mod.rs` currently dispatches only `GET`, `POST`, and `PUT` and falls through to `unimplemented!` for other methods.
  - `openai::api` now includes `DELETE` resources.
- Opportunity:
  1. Add full `HttpMethod` dispatch support in `RemoteResource::invoke_with`.
  2. Remove `unimplemented!` fallback for declared resource methods.
  3. Add tests for method routing parity between template methods and request dispatch.

### High Impact

#### 2. Extract endpoint resolution helpers
- Difficulty: Low
- Impact: High
- Evidence:
  - Repeated `Endpoint::from_template(template).map(|e| e.with_domain(...))` across `openai`, `gitlab`, and `raid`.
  - Repeated `INCLUDED_ENDPOINTS.find_by_name(name)` across `orcid`, `ror`, `citeas`, `geonames`, and `spdx`.
- Opportunity:
  1. Add root helpers for template+domain endpoint resolution and named endpoint lookup.
  2. Centralize endpoint-not-found error message formatting.

#### 3. Extract auth-header parameter builders
- Difficulty: Low
- Impact: High
- Evidence:
  - Repeated `Authorization: Bearer` parameter construction in `openai` and `raid`.
  - Repeated `PRIVATE-TOKEN` parameter construction in `gitlab`.
- Opportunity:
  1. Add reusable helpers for common header param styles.
  2. Standardize token handling and reduce per-module header string literals.

#### 4. Extract required-input validators
- Difficulty: Low
- Impact: High
- Evidence:
  - Repeated required `identifier` and required `body` match branches in `openai`.
  - Similar identifier branching in `raid` and parts of `gitlab`.
- Opportunity:
  1. Add root validators for required non-empty identifier/body values.
  2. Return uniform, provider-specific error messages via helper formatting.

#### 5. Extract invoke-and-parse convenience wrappers
- Difficulty: Medium
- Impact: High
- Evidence:
  - Repeated sequence: build params → invoke/invoke_with → handle/handle_or across most provider functions.
- Opportunity:
  1. Add wrappers that combine invocation and parse strategy.
  2. Allow optional fallback parser type where error envelopes differ by provider.

### Medium Impact

#### 6. Extract optional query parameter composition helper
- Difficulty: Low
- Impact: Medium
- Evidence:
  - Repeated optional list parameter insertion (`after`, `limit`, `order`, `model`) in `openai`.
  - Similar optional query assembly in other modules.
- Opportunity:
  1. Add helper to append optional key/value query params when present and non-empty.
  2. Keep module logic focused on endpoint actions rather than param plumbing.

#### 7. Normalize environment configuration bootstrap
- Difficulty: Medium
- Impact: Medium
- Evidence:
  - Repeated `.env` load + env var fallback patterns in `openai`, `raid`, and `gitlab` options.
- Opportunity:
  1. Add root helper for consistent env loading and prioritized var resolution.
  2. Reduce duplicated configuration boilerplate in provider `Configuration::from_env` implementations.

#### 8. Extract status/health helper pattern
- Difficulty: Low
- Impact: Medium
- Evidence:
  - Similar `is_healthy` wrappers in `orcid`, `ror`, and `citeas` that map status result to bool.
- Opportunity:
  1. Add root utility for status-to-health checks with pluggable predicate.
  2. Reduce repetitive status wrapper code in providers.

### Recommended Extraction Order

1. Implement critical method dispatch support for all declared `HttpMethod` variants.
2. Add endpoint resolution + auth/header builders.
3. Add required-input validators + optional query param composition helper.
4. Introduce invoke-and-parse wrappers with fallback support.
5. Normalize environment bootstrap and health-check helpers.

## OpenAI OpenAPI Remaining Endpoint Templates (Grouped by Feature Area)

Snapshot details:
1. Source spec: https://raw.githubusercontent.com/openai/openai-openapi/refs/heads/master/openapi.yaml
2. Total path templates in spec: 162
3. Path templates currently covered in acorn openai module: 9
4. Remaining path templates: 153

### assistants (2)

1. /assistants
2. /assistants/{assistant_id}

### audio (6)

1. /audio/speech
2. /audio/transcriptions
3. /audio/translations
4. /audio/voice_consents
5. /audio/voice_consents/{consent_id}
6. /audio/voices

### batches (3)

1. /batches
2. /batches/{batch_id}
3. /batches/{batch_id}/cancel

### chatkit (5)

1. /chatkit/sessions
2. /chatkit/sessions/{session_id}/cancel
3. /chatkit/threads
4. /chatkit/threads/{thread_id}
5. /chatkit/threads/{thread_id}/items

### completions (1)

1. /completions

### containers (5)

1. /containers
2. /containers/{container_id}
3. /containers/{container_id}/files
4. /containers/{container_id}/files/{file_id}
5. /containers/{container_id}/files/{file_id}/content

### conversations (4)

1. /conversations
2. /conversations/{conversation_id}
3. /conversations/{conversation_id}/items
4. /conversations/{conversation_id}/items/{item_id}

### embeddings (1)

1. /embeddings

### evals (6)

1. /evals
2. /evals/{eval_id}
3. /evals/{eval_id}/runs
4. /evals/{eval_id}/runs/{run_id}
5. /evals/{eval_id}/runs/{run_id}/output_items
6. /evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}

### files (3)

1. /files
2. /files/{file_id}
3. /files/{file_id}/content

### fine_tuning (11)

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

### images (3)

1. /images/edits
2. /images/generations
3. /images/variations

### moderations (1)

1. /moderations

### organization (46)

1. /organization/admin_api_keys
2. /organization/admin_api_keys/{key_id}
3. /organization/audit_logs
4. /organization/certificates
5. /organization/certificates/{certificate_id}
6. /organization/certificates/activate
7. /organization/certificates/deactivate
8. /organization/costs
9. /organization/groups
10. /organization/groups/{group_id}
11. /organization/groups/{group_id}/roles
12. /organization/groups/{group_id}/roles/{role_id}
13. /organization/groups/{group_id}/users
14. /organization/groups/{group_id}/users/{user_id}
15. /organization/invites
16. /organization/invites/{invite_id}
17. /organization/projects
18. /organization/projects/{project_id}
19. /organization/projects/{project_id}/api_keys
20. /organization/projects/{project_id}/api_keys/{api_key_id}
21. /organization/projects/{project_id}/archive
22. /organization/projects/{project_id}/certificates
23. /organization/projects/{project_id}/certificates/activate
24. /organization/projects/{project_id}/certificates/deactivate
25. /organization/projects/{project_id}/groups
26. /organization/projects/{project_id}/groups/{group_id}
27. /organization/projects/{project_id}/rate_limits
28. /organization/projects/{project_id}/rate_limits/{rate_limit_id}
29. /organization/projects/{project_id}/service_accounts
30. /organization/projects/{project_id}/service_accounts/{service_account_id}
31. /organization/projects/{project_id}/users
32. /organization/projects/{project_id}/users/{user_id}
33. /organization/roles
34. /organization/roles/{role_id}
35. /organization/usage/audio_speeches
36. /organization/usage/audio_transcriptions
37. /organization/usage/code_interpreter_sessions
38. /organization/usage/completions
39. /organization/usage/embeddings
40. /organization/usage/images
41. /organization/usage/moderations
42. /organization/usage/vector_stores
43. /organization/users
44. /organization/users/{user_id}
45. /organization/users/{user_id}/roles
46. /organization/users/{user_id}/roles/{role_id}

### projects (6)

1. /projects/{project_id}/groups/{group_id}/roles
2. /projects/{project_id}/groups/{group_id}/roles/{role_id}
3. /projects/{project_id}/roles
4. /projects/{project_id}/roles/{role_id}
5. /projects/{project_id}/users/{user_id}/roles
6. /projects/{project_id}/users/{user_id}/roles/{role_id}

### realtime (9)

1. /realtime/calls
2. /realtime/calls/{call_id}/accept
3. /realtime/calls/{call_id}/hangup
4. /realtime/calls/{call_id}/refer
5. /realtime/calls/{call_id}/reject
6. /realtime/client_secrets
7. /realtime/sessions
8. /realtime/transcription_sessions
9. /realtime/translations/client_secrets

### responses (2)

1. /responses/compact
2. /responses/input_tokens

### skills (6)

1. /skills
2. /skills/{skill_id}
3. /skills/{skill_id}/content
4. /skills/{skill_id}/versions
5. /skills/{skill_id}/versions/{version}
6. /skills/{skill_id}/versions/{version}/content

### threads (11)

1. /threads
2. /threads/{thread_id}
3. /threads/{thread_id}/messages
4. /threads/{thread_id}/messages/{message_id}
5. /threads/{thread_id}/runs
6. /threads/{thread_id}/runs/{run_id}
7. /threads/{thread_id}/runs/{run_id}/cancel
8. /threads/{thread_id}/runs/{run_id}/steps
9. /threads/{thread_id}/runs/{run_id}/steps/{step_id}
10. /threads/{thread_id}/runs/{run_id}/submit_tool_outputs
11. /threads/runs

### uploads (4)

1. /uploads
2. /uploads/{upload_id}/cancel
3. /uploads/{upload_id}/complete
4. /uploads/{upload_id}/parts

### vector_stores (10)

1. /vector_stores
2. /vector_stores/{vector_store_id}
3. /vector_stores/{vector_store_id}/file_batches
4. /vector_stores/{vector_store_id}/file_batches/{batch_id}
5. /vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel
6. /vector_stores/{vector_store_id}/file_batches/{batch_id}/files
7. /vector_stores/{vector_store_id}/files
8. /vector_stores/{vector_store_id}/files/{file_id}
9. /vector_stores/{vector_store_id}/files/{file_id}/content
10. /vector_stores/{vector_store_id}/search

### videos (8)

1. /videos
2. /videos/{video_id}
3. /videos/{video_id}/content
4. /videos/{video_id}/remix
5. /videos/characters
6. /videos/characters/{character_id}
7. /videos/edits
8. /videos/extensions

## CKAN Fixture Conversion to DCAT v3

This section documents the rules for converting CKAN API format test fixtures (in `tests/fixtures/schema/ckan/`) to DCAT v3 compliant JSON (in `tests/fixtures/schema/dcat/`). The rules follow the ckanext-dcat mapping guide: <https://docs.ckan.org/projects/ckanext-dcat/en/latest/mapping/>.

### Source Shape

Fixtures are CKAN dataset objects from `package_show.result`, not full `{ "success": true, "result": ... }` API envelopes. Custom fields may appear as first-level fields when CKAN uses scheming, or as legacy `extras[]` entries in older profiles. Current fixtures use first-level fields.

### Field Mapping CKAN → DCAT v3

| CKAN field path | DCAT v3 path | Transformation |
|---|---|---|
| `title` | `title` | Wrap in array: `["CKAN value"]` |
| `notes` (HTML) | `description` | Strip HTML tags, wrap in array |
| `url` | `landingPage` | Wrap single string in array |
| `identifier`, `guid`, `id` | `identifier` | Prefer custom `identifier`, then `guid`, then `id`; wrap in array |
| `issued`, `metadata_created` | `issued` | Prefer custom/top-level `issued`, then `metadata_created` |
| `modified`, `metadata_modified` | `modified` | Prefer custom/top-level `modified`, then `metadata_modified` |
| `tags[].name` | `keywords` | Extract `.name` into array of strings |
| `publisher[0]`, `organization` | `publisher` | Use first explicit publisher object; fall back to `organization.title` or `organization.name` |
| `creator[]` | `creator[]` | Each → `Agent { name }`; normalize ORCID-like `identifier` values to `https://orcid.org/{id}` |
| `contact[].email`, `maintainer_email`, `author_email` | `contactPoint` | Prefer contact entries; fall back to maintainer/author email; format as `mailto:` URI |
| `qualified_relation[]` | `qualifiedRelation[]` | Use `uri` as `relation`; use `role` when present, otherwise `relation`, as `hadRole` |
| `resources[]` | `distribution[]` | Each → `Distribution`; set `accessURL` from `access_url` then `url`; set `downloadURL` from `download_url`; map resource `name`, `description`, `mimetype`, `format`, `size`, `created`, `issued`, `modified`, and `metadata_modified` when present |

### Profile Notes From ckanext-dcat

- Mapping is profile-dependent and intentionally loose; do not assume every CKAN instance uses the same custom fields.
- `custom:` fields in ckanext-dcat may come from legacy `extras[]` or from first-level scheming fields.
- CKAN dataset-level license values do not map cleanly to ACORN's current DCAT fixture shape because ckanext-dcat primarily maps license data at Distribution level.
- Spatial, temporal, language, theme, `conformsTo`, and related profile fields are future fixture expansion candidates, not requirements for the current three fixtures.

### Fixture Files

| Source (ckan/) | Target (dcat/) | Content |
|---|---|---|
| `0sbdv-gh024.json` | `0sbdv-gh024.json` | Full metadata: HTML notes, creators, publisher, contact, keywords, qualified relations, no resources |
| `6rpmk-n3x82.json` | `6rpmk-n3x82.json` | Minimal dataset metadata and fallback behavior |
| `j5xbm-d7v43.json` | `j5xbm-d7v43.json` | Medium fixture with multiple creators, contact, and relation coverage |

### Verification

```bash
cargo run --bin acorn -- check ./tests/fixtures/schema/dcat/ --standard dcat
```

Expected result: 0 schema validation errors and no URL validation failures for generated DCAT fixtures.

Manual review checklist:

- Date precedence uses `issued` / `modified` when present.
- HTML notes are plain text in DCAT.
- ORCID identifiers are normalized.
- `qualifiedRelation` uses stable URI/role mapping.

---

## Critical Impact Implementation: Step 3 — Harden OOXML Mutation Boundary

This section provides the detailed implementation plan for Critical Impact item 3 (REFACTOR.md §94-103).

### Design Constraints

All implementation in this section must avoid:
- **`?` operator** — use combinator chaining (`and_then`, `map`, `map_err`, `or_else`) instead
- **Early returns** — function bodies are single expressions; use `match`/`if-else` arms to express branching
- **Mutation** — no `let mut`, no mutable references; use functional transforms with `let` bindings

### Problem Summary

Three sites in `acorn-cli/src/commands/export/powerpoint.rs` use regex/string-based XML manipulation:

1. **`add_slide_element`** (line 20-28): regex injection of `<p:sldId>` into `presentation.xml`
2. **`copy_image` relationship target** (lines 121-123): regex replacement of image target path in `.rels` file
3. **Slide path string replace** (lines 46-47): `String::replace()` for `slides/slide1` → `slides/slideN`

The library (`acorn-lib/src/io/powerpoint/ooxml.rs`) already has full `quick_xml` typed data structures for `Relationships`, `Relationship`, `TextParagraph`, etc. The fix moves the three mutation sites onto these types.

### Step 3a: Add Presentation XML types to `ooxml.rs`

Add typed structs for `presentation.xml` root element:

```rust
// In acorn-lib/src/io/powerpoint/ooxml.rs

/// Root element of presentation.xml
#[derive(Deserialize, Serialize)]
#[serde(rename = "p:presentation")]
pub struct Presentation {
    #[serde(rename = "p:sldIdLst")]
    pub slide_id_list: Option<SlideIdList>,
    // Other fields can be added as needed; unknown fields use #[serde(default)] via deny_unknown_fields
}

/// List of slide IDs
#[derive(Deserialize, Serialize)]
pub struct SlideIdList {
    #[serde(rename = "p:sldId")]
    pub slide_ids: Vec<SlideId>,
}

/// Single slide ID entry
#[derive(Deserialize, Serialize)]
pub struct SlideId {
    #[serde(rename = "@id")]
    pub id: u32,
    #[serde(rename = "@r:id")]
    pub relationship_id: String,
}
```

### Step 3b: Replace regex `add_slide_element` with typed round-trip

**File:** `acorn-cli/src/commands/export/powerpoint.rs`

**Before (lines 20-28):**
```rust
fn add_slide_element(xml: &str, revision_identifier: u32) -> String {
    let pattern = r#"<p:sldIdLst>(?<first_slide><p:sldId id="(?<identifier>\d+)" r:id="rId(?<revision>\d+)"/>)(?<new_slides>.*)</p:sldIdLst>"#;
    let re = Regex::new(pattern).unwrap();
    let new_slide_element_identifier: u32 = nanoid!(4, &['1', '2', '3', '4', '5', '6', '7', '8', '9']).parse().unwrap();
    let new_slide_element = format!(r#"<p:sldId id="{}" r:id="rId{revision_identifier}" />"#, new_slide_element_identifier);
    re.replace(xml, format!(r#"<p:sldIdLst>$1$4{new_slide_element}</p:sldIdLst>"#)).to_string()
}
```

**After:**
```rust
fn add_slide_element(xml: &str, revision_identifier: u32) -> ApiResult<String> {
    use acorn::io::powerpoint::ooxml::{Presentation, SlideId, SlideIdList};
    quick_xml::de::from_str::<Presentation>(xml)
        .map_err(|why| eyre!("Failed to parse presentation.xml — {why}"))
        .and_then(|mut pres| {
            let mut list = pres.slide_id_list.unwrap_or(SlideIdList { slide_ids: vec![] });
            let new_id = nanoid!(4, &['1', '2', '3', '4', '5', '6', '7', '8', '9'])
                .parse::<u32>()
                .unwrap_or(list.slide_ids.iter().max().unwrap_or(&0).saturating_add(1));
            list.slide_ids.push(SlideId {
                id: new_id,
                relationship_id: format!("rId{revision_identifier}"),
            });
            pres.slide_id_list = Some(list);
            quick_xml::se::to_string(&pres)
                .map(|s| format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>{s}"#))
                .map_err(|why| eyre!("Failed to serialize presentation.xml — {why}"))
        })
}
```

**Note:** The `mut pres` above is internal to the closure — it's a local binding, not module-level mutation. If strict immutability is required, construct a new `Presentation` with the additional `SlideId` via builder pattern.

### Step 3c: Replace regex image target manipulation with typed Relationships round-trip

**File:** `acorn-cli/src/commands/export/powerpoint.rs`

**Before (lines 120-123):**
```rust
let re = Regex::new(&target).unwrap();
let content = read_file(xml_rels_path.clone()).unwrap_or_default();
let updated = re.replace(&content, format!("../media/{name}.{extension}")).to_string();
```

**After:**
```rust
let updated = read_file(xml_rels_path.clone())
    .map_err(|why| eyre!("Failed to read slide relationships — {why}"))
    .and_then(|content| {
        quick_xml::de::from_str::<Relationships>(&content)
            .map_err(|why| eyre!("Failed to parse slide relationships — {why}"))
            .and_then(|rels| {
                let new_rels: Vec<Relationship> = rels.relationship.into_iter().map(|rel| {
                    if rel.target.ends_with("png") {
                        Relationship::init()
                            .id(rel.id)
                            .target(format!("../media/{name}.{extension}"))
                            .relationship_type(rel.relationship_type)
                            .maybe_target_mode(rel.target_mode)
                            .build()
                    } else {
                        rel
                    }
                }).collect();
                let updated_rels = Relationships::init().relationship(new_rels).build();
                updated_rels.to_string()
                    .map_err(|why| eyre!("Failed to serialize relationships — {why}"))
            })
    })?;
```

But wait — this uses `?` at the end. We need to avoid that. Restructure as:
```rust
let updated: ApiResult<String> = read_file(xml_rels_path.clone())
    .map_err(|why| eyre!("Failed to read slide relationships — {why}"))
    .and_then(|content| {
        quick_xml::de::from_str::<Relationships>(&content)
            .map_err(|why| eyre!("Failed to parse slide relationships — {why}"))
            .and_then(|rels| {
                let new_rels: Vec<Relationship> = rels.relationship.into_iter().map(|rel| {
                    if rel.target.ends_with("png") {
                        Relationship::init()
                            .id(rel.id)
                            .target(format!("../media/{name}.{extension}"))
                            .relationship_type(rel.relationship_type)
                            .maybe_target_mode(rel.target_mode)
                            .build()
                    } else {
                        rel
                    }
                }).collect();
                Relationships::init().relationship(new_rels).build()
                    .to_string()
                    .map_err(|why| eyre!("Failed to serialize relationships — {why}"))
            })
    });
```

### Step 3d: Replace slide path string replacements with structured path construction

**File:** `acorn-cli/src/commands/export/powerpoint.rs`, `add_slide` inner `slide_paths` + file copying (lines 42-56)

The slide copy loop creates new slide files by reading slide 1 and doing string replacements:

```rust
let content = result
    .replace("slides/slide1", &format!("slides/slide{}", slide_number))
    .replace("notesSlides/notesSlide1", &format!("notesSlides/notesSlide{}", slide_number));
```

This is the least risky of the three sites since the XML structure itself doesn't change — only internal path references. However, to harden it:

1. Validate the resulting XML with quick_xml before writing
2. Use a helper function `with_well_formed_xml` that round-trips through `quick_xml::de::from_str` + `quick_xml::se::to_string`

```rust
fn validate_xml_well_formed(content: &str) -> bool {
    quick_xml::Reader::from_str(content)
        .read_event()
        .is_ok()
}
```

### Step 3e: Add XML validation before archive write

**File:** `acorn-cli/src/commands/export/powerpoint.rs` — before `archive()` call (line 226)

```rust
// Validate modified OOXML files before archiving
let validation_targets = vec!["ppt/presentation.xml", "ppt/_rels/presentation.xml.rels"];
let all_valid = validation_targets.iter().all(|rel_path| {
    let path = reference_extract_path.join(rel_path);
    read_file(path)
        .map(|content| validate_xml_well_formed(&content))
        .unwrap_or(false)
});
if !all_valid {
    return Err(eyre!("Malformed OOXML content detected before archive write"));
}
// proceed with archive...
```

### Step 3f: Centralize OOXML utility helpers in `ooxml.rs`

Move the following into `acorn-lib/src/io/powerpoint/ooxml.rs`:
- `validate_xml_well_formed` function
- `read_xml_rel` (currently in `mod.rs`)
- `prettify_xml` (currently in `mod.rs`)
- The `add_slide_element` / `update_relationship_target` typed helpers

This ensures OOXML logic lives in one place and the CLI export module only orchestrates.

### Validation Commands

```bash
# Build tests
cargo test -p acorn-lib -- io::powerpoint
cargo test -p acorn-cli -- commands::export::powerpoint

# No-regression check
cargo check -p acorn-cli --features pdf
cargo clippy --workspace --no-deps --all-targets

# OOXML round-trip test
cargo test -p acorn-lib -- io::powerpoint::tests::test_ooxml_roundtrip
```

### Acceptance Criteria

1. All PowerPoint export tests pass with no `unwrap()` on regex ops
2. Modified `presentation.xml` and `.rels` files can be round-tripped through quick_xml
3. Malformed XML after mutation is caught before archive write (returns `Err`, not malformed PPTX)
4. `unreachable!()` at `powerpoint/mod.rs:39` is eliminated


