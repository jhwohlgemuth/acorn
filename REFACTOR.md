# ACORN Refactor Recommendations

This document captures cross-crate refactor and hardening recommendations for acorn-cli and acorn-lib.

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

### 1. Replace crash-oriented control flow with typed errors
- Difficulty: Low
- Why: Runtime `exit()`, `panic!()`, and `unimplemented!()` paths reduce testability and bypass centralized cleanup and diagnostics.
- Evidence:
  - `acorn-cli/src/main.rs`
  - `acorn-cli/src/commands/mod.rs`
  - `acorn-cli/src/commands/export/mod.rs`
  - `acorn-cli/src/commands/export/pdf.rs`
  - `acorn-cli/src/commands/export/powerpoint.rs`
  - `acorn-cli/src/commands/gather/mod.rs`
- Recommendation:
  1. Convert command/helper internals to `Result` return flows.
  2. Keep one top-level exit-code mapping boundary in CLI entrypoint.
  3. Ensure `Drop`-based cleanup remains reachable.

### 2. Remove `unimplemented!()` fallbacks from production export paths
- Difficulty: Low
- Why: Guaranteed panic path under unexpected preconditions.
- Evidence:
  - `acorn-cli/src/commands/export/pdf.rs`
  - `acorn-cli/src/commands/export/powerpoint.rs`
- Recommendation:
  1. Replace with explicit `Err(...)` variants.
  2. Add negative-path tests for missing input/reference and malformed state.

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

### 5. Make path filtering/normalization explicit and safe
- Difficulty: Low
- Why: Current filtering behavior can fail ambiguously and weaken operator confidence in included/excluded files.
- Evidence:
  - `acorn-cli/src/cli/mod.rs`
  - `acorn-lib/src/io/mod.rs`
- Recommendation:
  1. Return error on invalid regex/filter patterns.
  2. Introduce root-containment checks and centralized normalization.
  3. Reuse the same utility in check/export/link/download workflows.

### 6. Eliminate duplicate export field extraction across HTML and PPTX paths
- Difficulty: Medium
- Why: Same `ResearchActivity` fields are mapped in multiple places, increasing drift risk.
- Evidence:
  - `acorn-cli/src/template/mod.rs`
  - `acorn-lib/src/io/powerpoint/mod.rs`
- Recommendation:
  1. Introduce shared `ExportContext` (library-side).
  2. Extract fields once and consume in all format-specific renderers.
  3. Validate required placeholders up front.

### 7. Split CLI startup concerns out of `main`
- Difficulty: Medium
- Why: Logging/env/runtime/database/bootstrap/dispatch concerns are tightly mixed.
- Evidence:
  - `acorn-cli/src/main.rs`
- Recommendation:
  1. Extract setup units: logging, runtime/thread pool, database bootstrap, command dispatch.
  2. Keep main as orchestration glue with uniform error boundary.

### 8. Enforce required API token contracts (avoid silent empty defaults)
- Difficulty: Medium
- Why: Empty-token defaults produce ambiguous auth failures and weaker security posture.
- Evidence:
  - `acorn-lib/src/io/api/raid.rs`
  - `acorn-lib/src/io/api/gitlab.rs`
  - `acorn-lib/src/io/api/mod.rs`
- Recommendation:
  1. Replace permissive defaults with explicit required-secret checks where mandatory.
  2. Redact sensitive values from surfaced error context.

## Medium Impact

### 9. Standardize export handler error/progress contract
- Difficulty: Low
- Why: Handler pattern exists but per-format error paths and progress treatment remain inconsistent.
- Evidence:
  - `acorn-cli/src/commands/export/mod.rs`
- Recommendation:
  1. Keep `ExportHandler` pattern.
  2. Normalize failure mapping and progress completion in one adapter layer.

### 10. Consolidate interpolation primitives under one shared utility
- Difficulty: Low
- Why: Placeholder logic is split between generic interpolation and PowerPoint-specific behavior.
- Evidence:
  - `acorn-lib/src/util/mod.rs`
  - `acorn-lib/src/io/powerpoint/mod.rs`
- Recommendation:
  1. Extend shared interpolation trait(s) for list/bullet use-cases.
  2. Remove parallel interpolation stacks.

### 11. Introduce stronger shared API-client contract
- Difficulty: Medium
- Why: API modules repeat health/auth/endpoint patterns without a fully unified contract.
- Evidence:
  - `acorn-lib/src/io/api/mod.rs`
  - `acorn-lib/src/io/api/raid.rs`
  - `acorn-lib/src/io/api/gitlab.rs`
- Recommendation:
  1. Expand common trait surface for health, auth, endpoint rendering, and error normalization.
  2. Move duplicated logic behind shared helper modules.

### 12. Reduce binding-layer repetition in Python wrapper
- Difficulty: Medium
- Why: Wrapper boilerplate increases maintenance cost and schema drift risk.
- Evidence:
  - `acorn-py/src/lib.rs`
- Recommendation:
  1. Complete data exposure shape first.
  2. Then apply macro-based codegen for repetitive PID wrapper patterns.

## Lower Impact

### 13. Tighten schema I/O and format validation handoff between CLI and library
- Difficulty: High
- Why: CLI and library each carry partial format assumptions and validation behavior.
- Evidence:
  - `acorn-lib/src/io/mod.rs`
  - `acorn-lib/src/schema/research_activity/mod.rs`
  - `acorn-cli/src/commands/export/mod.rs`
- Recommendation:
  1. Extend `InputOutput` with stricter read/validate variants.
  2. Make CLI rely on library-side strict format validation consistently.

### 14. Replace unfinished stubs with explicit unsupported-capability errors
- Difficulty: Low
- Why: Placeholder stubs are less predictable than explicit capability contracts.
- Evidence:
  - `acorn-cli/src/commands/doctor/mod.rs`
  - `acorn-cli/src/cli/arguments.rs`
  - `acorn-lib/src/io/api/gitlab.rs`
- Recommendation:
  1. Return explicit unsupported errors with actionable messages.
  2. Gate partially implemented capabilities behind feature flags/capability checks.

### 15. Separate parsed CLI model from execution options model
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

1. Reliability triage:
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

1. No production `unimplemented!()`/`panic!()` in command paths.
2. Export paths fail with typed errors, never silent corruption.
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

### PR 1: Unblock no-default-features Build

Goal:
- Make `cargo check -p acorn-lib --no-default-features` pass.

Primary edits:
1. `acorn-lib/src/lib.rs`
  - Change unconditional analyzer export to feature-gated analyzer export.
  - From: `pub mod analyzer;`
  - To: `#[cfg(feature = "analysis")] pub mod analyzer;`

2. `acorn-lib/src/util/macros.rs`
  - Gate `check!`, `check_ok!`, `check_err!` macro definitions behind `feature = "analysis"` or add non-analysis-safe variants that do not reference `crate::analyzer::Check*`.

3. `acorn-lib/src/analyzer/mod.rs`
  - Ensure all code requiring `Check`, `CheckCategory`, `CheckSeverity`, `lychee_lib`, `bat`, `rayon`, and std process/runtime APIs is inside `#[cfg(feature = "analysis")]` sections.

Validation commands:
1. `cargo check -p acorn-lib --no-default-features`
2. `cargo check -p acorn-lib --no-default-features --features analysis,std`

Acceptance criteria:
1. no-default-features check succeeds.
2. analysis+std check still succeeds.

### PR 2: Introduce Explicit Capability Features (`core`, `alloc`, `wasm`)

Goal:
- Replace implicit capability assumptions with explicit layering.

Primary edits:
1. `acorn-lib/Cargo.toml` feature block
  - Add:
    - `core = []`
    - `alloc = ["core"]`
    - `wasm = ["alloc"]`
  - Update existing features to depend on capabilities:
    - `std = ["alloc", ...]`
    - `analysis = ["alloc", "std", ...]`
    - `powerpoint = ["alloc"]` or `powerpoint = ["alloc", "std"]` depending on final API placement.

2. `acorn-lib/src/lib.rs`
  - Add compile-time guardrails:
    - `#[cfg(all(feature = "wasm", feature = "std"))]` policy decision comment and allowed/denied behavior.

Validation commands:
1. `cargo check -p acorn-lib --no-default-features --features core`
2. `cargo check -p acorn-lib --no-default-features --features alloc`
3. `cargo check -p acorn-lib --no-default-features --features std`

Acceptance criteria:
1. Each capability layer compiles independently.
2. Existing default behavior remains intact for CLI consumers.

### PR 3: Dependency Hygiene for no-std/WASM

Goal:
- Ensure host-only dependencies are optional and tied to explicit features.

Primary edits:
1. `acorn-lib/Cargo.toml`
  - Make std/host-only crates optional where appropriate:
    - `docx-rs`, `directories`, `indicatif`, `dotenvy`, `rayon`, `rusqlite`, `flate2`, `zip`, `tera`, `percy-dom`, `which`, `reqwest`, `tokio`, `axum`, `tower`, `lychee-lib`, `bat`, `polars`.
  - Keep lightweight alloc/no-std-friendly crates non-optional where required.
  - Replace `chrono` dependency wiring so `chrono/std` is only enabled via `std` feature.

2. `Cargo.toml` (workspace)
  - Adjust workspace-level defaults that force std features (example: chrono std flag).
  - Keep crate-level feature wiring responsible for std activation.

Validation commands:
1. `cargo tree -p acorn-lib --no-default-features -e normal`
2. `cargo check -p acorn-lib --no-default-features --features alloc`

Acceptance criteria:
1. no-default-features tree does not include obvious std-only host stacks (tokio/reqwest/rusqlite/rayon/polars, etc.).

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

## Milestone Definition

M1: no-default-features build passes.
M2: alloc/core feature layering is stable and documented.
M3: wasm feature compiles for `wasm32-unknown-unknown`.
M4: CI matrix enforces no-std + wasm guarantees.

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
