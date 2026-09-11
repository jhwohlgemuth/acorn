---
type: Plan
title: "Generic extensible Options and Config"
description: "Replace duplicated *Options and *Config structs with generic composition types shared via acorn-core::options."
difficulty: high
impact: high
package: [acorn-core, acorn-lib, acorn-cli]
tags: [refactor, api, config, generic, composition]
status: in-progress
priority: next
---

# Goal

Reduce redundancy across ~65 `*Options`/`*Config` structs by introducing fully generic composition types owned by `acorn-core::options`. Each domain keeps a narrow extension struct (`*Ext`/`Inner`) while common fields (`token/domain/identifier/body/custom_params` for API; `offline/quiet/threads/filter/ignore/dry_run/no_color/output` for runtime; `path` for sync targets) live once.

# Implementation progress

- **Phase 1 complete:** `acorn-core::options` owns the generic API, runtime, and sync foundations.
- **Phase 2 complete:** CiteAs, OpenAI, OpenAlex, ORCiD, RAiD, and ROR use `ApiOptions<Extension, Param>` plus one blanket `Configuration` implementation. Provider modules now own only their environment/default-domain extension, with RAiD retaining its domain-specific metadata fields in `Extension`.
- **Phase 3 complete:** bucket transfer uses `RuntimeOptions<BucketExtension, PathBuf>` and workflows use `RuntimeOptions<WorkflowExtension>`. Shared runtime fields are accessed through `common`, domain fields through `extension`, and CLI/library call sites construct both explicitly.
- **Phase 4 next:** migrate analyzer runtime options, preserving the existing analysis/execution/report split.
- **Phase 5:** migrate sync targets to `TargetConfig<I>` + `SyncConfig`, then evaluate GitLab against its additional pagination and token-header fields, re-export `With` alongside `Validate` as `acorn::With`, and finish schema/documentation validation.

# Current evidence

- `crates/acorn-core/src/options.rs` now provides a redacting generic `ApiOptions<E, P = ApiParam>` builder. The parameter type remains generic so `acorn-core` does not depend on `acorn-lib::io::api::Param`; provider aliases fix `P = Param` at the host boundary.
- `crates/acorn-lib/src/io/api/mod.rs` now has the single `Configuration for ApiOptions<E, Param>` implementation used by six provider aliases in `citeas.rs`, `openai.rs`, `openalex.rs`, `orcid.rs`, `raid.rs`, and `ror.rs`.
- `crates/acorn-lib/src/io/api/gitlab/mod.rs:663` remains bespoke for a later phase because it additionally owns pagination, token-header, and project-selection behavior.
- `crates/acorn-lib/src/io/api/huggingface.rs:219` is intentionally excluded — divergent shape (`token: Option<Secret>`, `revision`, `search_limit`, `selector`, `filter/ignore`) justifies bespoke struct.
- `crates/acorn-lib/src/io/api/harbor.rs:72` `Options<'a>` is lifetime-generic and registry-profile-bound; excluded for same reason.
- `crates/acorn-lib/src/io/config/bucket.rs` aliases `BucketOptions` to `RuntimeOptions<BucketExtension, PathBuf>` so native output paths retain non-UTF-8 fidelity while common runtime fields stay portable.
- `crates/acorn-lib/src/io/workflow/mod.rs` aliases `Options` to `RuntimeOptions<WorkflowExtension>`; validation remains enabled by default through `WorkflowExtension::default`.
- Runtime flags remain repeated across `crates/acorn-lib/src/analyzer/host/discovery/mod.rs:230` `Options`, `crates/acorn-lib/src/analyzer/host/context/mod.rs:27/43/71` `AnalysisOptions/ExecutionOptions/ReportOptions`, and `crates/acorn-lib/src/io/sync/mod.rs:130`.
- Sync `Config` duplicated 4×: `crates/acorn-lib/src/io/sync/{goose,opencode,vscode,llama_swap}.rs` each repeat `path: Option<String>` plus manual `SyncConfig::merge` at `crates/acorn-lib/src/io/sync/mod.rs:192` enumerating 4 fields.
- `crates/acorn-lib/src/io/api/mod.rs:79` `Configuration` and its generic implementation remain the integration lock for all shared API options.
- `crates/acorn-lib/src/io/config/application.rs:63` `ApplicationConfiguration` holds `config: Option<sync::Config>` and owns file I/O.

# Project code boundaries

- **Updated:** `crates/acorn-core/Cargo.toml`, `crates/acorn-core/src/lib.rs`, `crates/acorn-core/src/options.rs`, `crates/acorn-lib/src/io/api/mod.rs`, `crates/acorn-lib/src/io/api/{citeas,ror,orcid,openai,raid,openalex}.rs`, `crates/acorn-lib/src/io/config/{bucket,mod,transfer}.rs`, `crates/acorn-lib/src/io/workflow/mod.rs`, their call sites and tests, `.wiki/plan-network.md`, and `.wiki/refactors/index.md`.
- **Remaining updates:** `crates/acorn-lib/src/io/api/gitlab/mod.rs`, `crates/acorn-lib/src/analyzer/host/{context,discovery}/mod.rs`, `crates/acorn-lib/src/io/sync/{mod,goose,opencode,vscode,llama_swap}.rs`, `crates/acorn-lib/src/io/config/application.rs`, `crates/acorn-lib/src/lib.rs`, all analyzer/sync call sites, and `.wiki/parallel-execution.md`.
- **Create:** None; the shared module and this plan were created in Phase 1.
- **Delete:** None (old `Options`/`Config` structs are replaced in place; no compatibility aliases per immediate breaking change).

# Dependencies and ordering

- Requires `secrecy` in `acorn-core` (retained — already transitively via `acorn-schema`).
- Precedes `acorn-api` extraction: generic `ApiOptions<E>` establishes the stable shared contract.
- Serializes with `API shared contract` lane (`io/api/mod.rs`, `io/config/mod.rs` locks) — single integrator owns those roots per [parallel-execution](parallel-execution.md).
- HuggingFace (`huggingface-request-readiness.md`) and Harbor remain independent leaves.

# Recommendation

## Shared core (`crates/acorn-core/src/options.rs`)

```rust
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct CommonRuntime { offline, quiet, dry_run, no_color, threads, filter, ignore }

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
pub struct RuntimeOptions<E: Default+Clone, O = String> {
    #[builder(default)] pub common: CommonRuntime,
    pub output: Option<O>,
    #[serde(default, flatten)] #[builder(default)] pub extension: E,
}

#[derive(Builder, Clone)]
#[builder(start_fn = with_token, on(String, into), on(SecretString, into))]
pub struct ApiOptions<E: ApiExtension, P: Clone+Debug = ApiParam> {
    #[builder(start_fn)] pub token: SecretString,
    pub body: Option<String>,
    #[builder(default = E::default_domain())] pub domain: String,
    pub identifier: Option<String>,
    #[builder(default)] pub custom_params: Vec<P>,
    #[builder(default)] pub extension: E,
}
pub trait ApiExtension: Default+Clone+Debug { fn default_domain()->String; fn env_token_var()->&'static str; fn env_domain_var()->&'static str; }
impl<E: ApiExtension> Configuration for ApiOptions<E, Param> { /* single blanket impl */ }

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
pub struct TargetConfig<I: SyncInner> {
    #[serde(skip_serializing)] #[validate(length(min=1))] pub path: Option<String>,
    #[serde(default, flatten)] #[validate(nested)] pub inner: I,
}
```

`ApiOptions` implements `core::fmt::Debug` manually so tokens remain redacted on portable builds without relying on a `std`-expanding derive.

- Composition over flatten-inline: callers use `options.common.offline` (explicit, greppable).
- `#[serde(flatten)]` only on `extension`/`inner` with `#[serde(default)]`.
- `schemars` shim: `#[schemars(skip)]` on flatten plus manual `JsonSchema` impl delegating to `E::json_schema`.
- Runtime flags live only in `CommonRuntime`/`RuntimeOptions`, not in `ApiOptions`.

## Per-file migration

- CiteAs, ROR, ORCiD, OpenAI, OpenAlex, and RAiD now expose `type Options = ApiOptions<Extension, Param>` plus `impl ApiExtension for Extension`; RAiD keeps `metadata`, `prefix`, and `suffix` in its extension.
- `BucketOptions` is now `RuntimeOptions<BucketExtension, PathBuf>` (`database_path`, `no_local_database`, `strict`, `canonicalize`, `flatten`, `clobber` in `BucketExtension`).
- `workflow::Options` is now `RuntimeOptions<WorkflowExtension>` (`check`, `format`, `enrich`, `link`, `show_changes`).
- Analyzer `Options`/`AnalysisOptions` → `RuntimeOptions<DiscoveryExt>`.
- Sync: each target's `Config` splits into `Inner` (domain fields) + `TargetConfig<Inner>`; `SyncConfig::merge` becomes generic over `TargetConfig<I>`.

# Validation

Phase 2 validation on 2026-09-10:

- `cargo test -p acorn-core --lib options::tests` — 4 passed.
- `cargo test -p acorn-lib --lib io::api::tests` — 115 passed, 1 network test ignored.
- `cargo check --workspace --all-features` — passed.
- `cargo check -p acorn-lib --no-default-features` — passed.
- `cargo clippy -p acorn-core -p acorn-lib --no-deps --all-targets` — passed; macOS SDK discovery emitted sandbox cache warnings only.
- `make check-wasm` — passed; macOS SDK discovery emitted sandbox cache warnings only.
- `make lint` — passed.
- `make test` — 1,640 passed, 14 skipped; one existing leaky-process diagnostic and one slow test were reported.

Phase 3 validation on 2026-09-11:

- `cargo test -p acorn-core --lib options::tests` — 5 passed.
- `cargo test -p acorn-lib --lib io::config::tests` — 42 passed.
- `cargo test -p acorn-lib --lib io::workflow::tests` — 10 passed.
- `cargo check -p acorn-core -p acorn-lib -p acorn-cli --all-features` — passed.
- Native `--no-default-features` checks for `acorn-core` and `acorn-lib` — passed.
- The complete `check-wasm` cargo matrix — passed through the pinned rustup toolchain; sandboxed macOS SDK cache warnings were emitted only.
- `make lint` — passed.
- `make test` — 1,641 passed, 14 skipped; one existing slow test was reported.

Remaining final validation:

1. `cargo check --workspace --all-features`
2. `cargo check -p acorn-lib --no-default-features` and `make check-wasm` (wasm32-unknown-unknown)
3. `cargo clippy --workspace --no-deps --all-targets` (no `unwrap_used`, `missing-docs`)
4. `cargo nextest run --run-ignored all` (covers `io::config::tests`, `io::sync`, `analyzer`, API)
5. Snapshot of `ApplicationConfiguration::parse` error paths (unknown field via `flatten` still reported)
6. `schemars` snapshot diff (`cargo run -p acorn-cli -- schema`) — only `extension` ordering change
7. Compile an external-style derive importing `use acorn::With;` to verify the facade re-export

# Acceptance criteria

- [x] `ApiOptions<E, Param>` + single `Configuration` impl replaces six duplicate `Options` structs/impls; Hugging Face and Harbor remain bespoke.
- [x] `RuntimeOptions<E, O>` replaces bucket and workflow option duplication while preserving native output path types.
- [ ] `RuntimeOptions<E, O>` replaces analyzer option duplication while preserving the analysis/execution/report split.
- [ ] `TargetConfig<I>` + `SyncConfig` replaces 4× `path` duplication and manual `merge` enumeration; adding a fifth target requires only one `Inner` + one field.
- [ ] All planned `#[serde(flatten)]` uses are limited to `extension`/`inner` with `#[serde(default)]`.
- [x] `acorn-core` owns the shared API type with `secrecy` retained and no `acorn-lib` facade re-export.
- [ ] `With` is re-exported alphabetically alongside `Validate` as `acorn::With` when the final phase lands.
- [ ] The final portable feature matrix, `make lint`, and `make test` are green after all runtime/sync migrations.
- [ ] Wiki network, execution lanes, and indexes reflect completion; remove this plan after the final phase lands.

# Related plans

- [Crate boundary architecture](crate-boundary-architecture.md)
- [API roadmap](api/roadmap.md)
- [Workspace duplication consolidation](workspace-duplication-consolidation.md)
- [Host abstraction traits](portability/host-abstraction-traits.md)
- [Hugging Face request readiness](api/huggingface-request-readiness.md)
