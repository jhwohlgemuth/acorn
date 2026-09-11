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

# Current evidence

- API `Options` duplicated 7×: `crates/acorn-lib/src/io/api/{citeas,ror,orcid,openai,raid,openalex,geonames}/mod.rs` + `crates/acorn-lib/src/io/api/gitlab/mod.rs:662` each repeat `token: Secret`, `body`, `domain`, `identifier`, `custom_params` plus identical `impl Configuration` (`from_env`/`with_body`/`with_domain`/`with_identifier`/`with_params`). Only env var names and default domains differ.
- `crates/acorn-lib/src/io/api/huggingface.rs:219` is intentionally excluded — divergent shape (`token: Option<Secret>`, `revision`, `search_limit`, `selector`, `filter/ignore`) justifies bespoke struct.
- `crates/acorn-lib/src/io/api/harbor.rs:72` `Options<'a>` is lifetime-generic and registry-profile-bound; excluded for same reason.
- Runtime flags repeated 5×: `crates/acorn-lib/src/io/config/bucket.rs:52` `BucketOptions`, `crates/acorn-lib/src/io/workflow/mod.rs:77` `Options`, `crates/acorn-lib/src/analyzer/host/discovery/mod.rs:230` `Options`, `crates/acorn-lib/src/analyzer/host/context/mod.rs:27/43/71` `AnalysisOptions/ExecutionOptions/ReportOptions`, `crates/acorn-lib/src/io/sync/mod.rs:130` shares `offline/quiet/dry_run`.
- Sync `Config` duplicated 4×: `crates/acorn-lib/src/io/sync/{goose,opencode,vscode,llama_swap}.rs` each repeat `path: Option<String>` plus manual `SyncConfig::merge` at `crates/acorn-lib/src/io/sync/mod.rs:192` enumerating 4 fields.
- `crates/acorn-lib/src/io/api/mod.rs:78` `Configuration` trait is the integration lock for all API options.
- `crates/acorn-lib/src/io/config/application.rs:63` `ApplicationConfiguration` holds `config: Option<sync::Config>` and owns file I/O.

# Project code boundaries

- **Update:** `crates/acorn-core/Cargo.toml`, `crates/acorn-core/src/lib.rs`, `crates/acorn-lib/src/io/api/mod.rs`, `crates/acorn-lib/src/io/api/{citeas,ror,orcid,openai,raid,openalex,geonames}.rs`, `crates/acorn-lib/src/io/api/gitlab/mod.rs`, `crates/acorn-lib/src/io/config/bucket.rs`, `crates/acorn-lib/src/io/workflow/mod.rs`, `crates/acorn-lib/src/analyzer/host/{context,discovery}/mod.rs`, `crates/acorn-lib/src/io/sync/{mod,goose,opencode,vscode,llama_swap}.rs`, `crates/acorn-lib/src/io/config/application.rs`, `crates/acorn-lib/src/prelude.rs`, `crates/acorn-lib/src/lib.rs`, all API/runtime/sync call sites (`cargo fix` + manual `common.*` rewrites), `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/refactors/index.md`.
- **Create:** `crates/acorn-core/src/options.rs` (generic `ApiOptions<E>`, `ApiExtension`, `CommonRuntime`, `RuntimeOptions<E>`, `TargetConfig<I>`, `SyncInner`, `SyncConfig` + `schemars` shims); this plan file.
- **Delete:** None (old `Options`/`Config` structs are replaced in place; no compatibility aliases per immediate breaking change).

# Dependencies and ordering

- Requires `secrecy` in `acorn-core` (retained — already transitively via `acorn-schema`).
- Precedes `acorn-api` extraction: generic `ApiOptions<E>` establishes the stable shared contract.
- Serializes with `API shared contract` lane (`io/api/mod.rs`, `io/config/mod.rs` locks) — single integrator owns those roots per [parallel-execution](parallel-execution.md).
- HuggingFace (`huggingface-request-readiness.md`) and Harbor remain independent leaves.

# Recommendation

## Shared core (`crates/acorn-core/src/options.rs`)

```rust
#[derive(Builder, Clone, Debug, Default, Deserialize, Serialize)]
pub struct CommonRuntime { offline, quiet, dry_run, no_color, threads, filter, ignore }

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
pub struct RuntimeOptions<E: Default+Clone> {
    #[builder(default)] pub common: CommonRuntime,
    pub output: Option<PathBuf>,
    #[serde(default, flatten)] #[builder(default)] pub extension: E,
}

#[derive(Builder, Clone, Redact)]
#[builder(start_fn = with_token, on(String, into), on(SecretString, into))]
pub struct ApiOptions<E: ApiExtension+Clone> {
    #[builder(start_fn)] #[redact] pub token: SecretString,
    pub body: Option<String>,
    #[builder(default = E::default_domain())] pub domain: String,
    pub identifier: Option<String>,
    #[builder(default)] pub custom_params: Vec<Param>,
    #[serde(default, flatten)] #[builder(default)] pub extension: E,
}
pub trait ApiExtension: Default+Clone { fn default_domain()->String; fn env_token_var()->&'static str; fn env_domain_var()->&'static str; }
impl<E: ApiExtension> Configuration for ApiOptions<E> { /* single blanket impl */ }

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
pub struct TargetConfig<I: SyncInner> {
    #[serde(skip_serializing)] #[validate(length(min=1))] pub path: Option<String>,
    #[serde(default, flatten)] #[validate(nested)] pub inner: I,
}
```

- Composition over flatten-inline: callers use `options.common.offline` (explicit, greppable).
- `#[serde(flatten)]` only on `extension`/`inner` with `#[serde(default)]`.
- `schemars` shim: `#[schemars(skip)]` on flatten plus manual `JsonSchema` impl delegating to `E::json_schema`.
- Runtime flags live only in `CommonRuntime`/`RuntimeOptions`, not in `ApiOptions`.

## Per-file migration

- Each API leaf (`citeas`, `ror`, `orcid`, `openai`, `raid`, `openalex`, `geonames`, optionally `gitlab`) becomes `struct RaidExt { metadata, prefix, suffix }` + `impl ApiExtension for RaidExt`.
- `BucketOptions` → `RuntimeOptions<BucketExt>` (`database_path`, `no_local_database`, `strict`, `canonicalize`, `flatten`, `clobber` in `BucketExt`).
- `workflow::Options` → `RuntimeOptions<WorkflowExt>` (`check`, `format`, `enrich`, `link`, `show_changes`).
- Analyzer `Options`/`AnalysisOptions` → `RuntimeOptions<DiscoveryExt>`.
- Sync: each target's `Config` splits into `Inner` (domain fields) + `TargetConfig<Inner>`; `SyncConfig::merge` becomes generic over `TargetConfig<I>`.

# Validation

1. `cargo check --workspace --all-features`
2. `cargo check -p acorn-lib --no-default-features` and `make check-wasm` (wasm32-unknown-unknown)
3. `cargo clippy --workspace --no-deps --all-targets` (no `unwrap_used`, `missing-docs`)
4. `cargo nextest run --run-ignored all` (covers `io::config::tests`, `io::sync`, `analyzer`, API)
5. Snapshot of `ApplicationConfiguration::parse` error paths (unknown field via `flatten` still reported)
6. `schemars` snapshot diff (`cargo run -p acorn-cli -- schema`) — only `extension` ordering change

# Acceptance criteria

- `ApiOptions<E>` + single `Configuration` impl replaces 6+ duplicate `Options` structs/impls; HuggingFace/Harbor remain bespoke.
- `RuntimeOptions<E>` replaces `BucketOptions`, workflow and analyzer option duplication; runtime flags exist in one place.
- `TargetConfig<I>` + `SyncConfig` replaces 4× `path` duplication and manual `merge` enumeration; adding a 5th target requires only one `Inner` + one field.
- All `#[serde(flatten)]` uses are limited to `extension`/`inner` with `#[serde(default)]`.
- `acorn-core` owns `common` types with `secrecy` retained; no `acorn-lib` facade re-export.
- Portable feature matrix (`--no-default-features`, `check-wasm`) passes; `make lint` + `make test` green.
- Wiki `plan-network.md` and `parallel-execution.md` updated; this plan removed after landing.

# Related plans

- [Crate boundary architecture](crate-boundary-architecture.md)
- [API roadmap](api/roadmap.md)
- [Workspace duplication consolidation](workspace-duplication-consolidation.md)
- [Host abstraction traits](portability/host-abstraction-traits.md)
- [Hugging Face request readiness](api/huggingface-request-readiness.md)
