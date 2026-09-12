---
type: Plan
title: "AI-enabled scraping and Rhai scripting extensibility"
description: "Extend gather with deterministic CSS scraping and pluggable AI fallback, and expose Rhai scripting that can intercept every workflow step with the full Acorn API available by default."
difficulty: high
impact: high
package: [workspace, acorn-lib, acorn-cli, acorn-schema, acorn-core]
tags: [feature, scraping, ai, rhai, scripting, gather, inference, sidecar, workflow]
status: planned
priority: next
---

# Project code boundaries

- **Update:** `crates/acorn-cli/src/cli/{mod,arguments}.rs` and `crates/acorn-cli/src/commands/mod.rs`, `crates/acorn-cli/src/commands/gather/mod.rs`, `crates/acorn-lib/src/analyzer/host/discovery/mod.rs`, `crates/acorn-lib/src/analyzer/host/service/mod.rs`, `crates/acorn-lib/src/io/{mod,http,cache,source,workflow/repository}.rs`, `crates/acorn-lib/src/io/api/mod.rs`, `crates/acorn-lib/src/io/sidecar/mod.rs`, `crates/acorn-lib/src/lib.rs`, workspace manifests, `deny.toml`, `.wiki/{plan-network,parallel-execution}.md`, and related tests/docs/help.
- **Create:** `crates/acorn-lib/src/script/{mod,resolver,version,hooks,runtime,api/{util,core,schema,io,analyzer,scrape,infer,log,check}}.rs`, `crates/acorn-lib/src/scrape/{mod,fetcher,extractor,pipeline,config}.rs`, `crates/acorn-lib/src/inference/{mod,sidecar,remote,registry}.rs`, sidecar registry extensions, `crates/acorn-cli/src/commands/script/mod.rs`, embedded examples under `crates/acorn-cli/assets/scripts/*.rhai`, fixtures under `tests/fixtures/scrape/` and `tests/fixtures/scripts/`.
- **Delete:** None. Remove `--ai` boolean only from plan text; no existing flag is retired.

# Dependencies

Consumes the completed `io::api::json_rpc` operation registry, webhook lifecycle, and GitLab repository automation as foundations; relies on the existing `Sidecar` trait, `ModelKitManifest`, `DiscoveryExtension`/`Records`, `AnalysisReport`/`CheckBatch`, `ToolRegistry`, `SafePath`, and feature-gate/WASM contracts. Must not fork DTOs or duplicate shared environment/auth handling owned by the API roadmap. This is not part of the highest-priority RPC/automation lane, so it must not take `io/api/json_rpc`, `io/api/webhooks`, or `io/sidecar` locks during an active lane slice.

# Goal

Add automated AI-enabled scraping to `acorn gather` and make Rhai the user-extensible scripting surface for every ACORN workflow step, leveraging existing code to the maximum extent.

# Scope and assumptions

- Extend `gather` rather than adding a new `acorn scrape` command. `gather` already accepts files, directories, URLs, and `--text`; this plan adds URL-list scraping to that existing surface.
- Inference is pluggable: any `Sidecar` implementation (Needle is one) or any OpenAI-compatible remote API (`io::api::openai` pattern via `Configuration`) can satisfy extraction. Needle is not special-cased.
- Script resolution is `(a)` explicit `--script <path>` `(b)` auto-discovery of `.acorn/scripts/**/*.rhai` and `XDG_CONFIG_HOME/acorn/scripts/` `(c)` `rust-embed` curated examples under `crates/acorn-cli/assets/scripts/`.
- Deterministic CSS selectors are tried first; AI fallback runs only when extraction is incomplete and an inference engine was passed (`--inference`). There is no separate `--ai` flag.
- `--selector` is the flag name (not `--scrape-selector`); it is `value_name="SELECTOR"` and may be repeated. `--allow-net` is required for remote fetches and remote inference; `--allow-fs` defaults to the invocation CWD. `offline` continues to override all network paths.
- Rhai is the front-runner and must feel Rust-like. The engine is behind a facade so an alternative VM can replace it without changing hooks or CLI if later required.
- Scripts declare their target API via `// @acorn 0.2` version header; mismatch is a schema check error.
- Every Rhai script receives the full Acorn API by default, without an explicit import. That is `acorn::{util,core,schema,io,analyzer}` as raw mirrors plus convenience aliases (`acorn::http`, `acorn::fs`, `acorn::rad`). The Rhai engine checks `allow_fs`/`allow_net`/`offline` automatically on every IO/network call.
- `Analyzer` inspection is included on day 1.

# Current evidence

- `gather` discovers persistent identifiers via `analyzer::host::discovery::Records` and `Identifier::find_all`, supports `INPUT...`, `--text`, `--filter`/`--ignore`, `--max-depth`, and `--osti` remote search, but has no HTML fetch, CSS extraction, or AI structuring.
- `Commands` and `CommandOptions` plus `preflight!`, `resolve_paths`, `with_progress`, `ProgressType`, and global `offline`/`threads`/`watch` are already composable across `check`/`format`/`link`/`export`/`gather` (`crates/acorn-cli/src/cli/mod.rs:32`, `crates/acorn-cli/src/commands/mod.rs:39,96`, `crates/acorn-lib/src/io/mod.rs:121,978`).
- `Sidecar` is already a trait with one implementation (`crates/acorn-lib/src/io/sidecar/mod.rs:11`, `crates/acorn-lib/src/io/sidecar/needle/mod.rs:43`); `ModelKitManifest` (`crates/acorn-lib/src/io/sidecar/mod.rs:26`) is generic. Remote providers already use the `Configuration` trait (`crates/acorn-lib/src/io/api/mod.rs:79`) with env/keyring-backed tokens.
- `SafePath` (`crates/acorn-lib/src/io/mod.rs:109`), `file_checksum` (`crates/acorn-lib/src/io/mod.rs:1105`), `Label::*()` (`crates/acorn-lib/src/util/mod.rs`), `MimeType`/`Location` (`crates/acorn-core/src/util`), and `ResearchActivity`/`Identifier` (`crates/acorn-schema`) are all reusable without new dependencies.
- `ammonia` and `percy-dom` are already optional dependencies behind the `template` feature and can be reused for HTML sanitization/DOM handling.
- No Rhai dependency or hook registry exists. No `scrape`, `script`, or `ai-scrape` feature gates exist.

# Recommendation

## Feature gates

Add to `crates/acorn-lib/Cargo.toml` and workspace manifests:

```toml
rhai = { version = "1.22", default-features = false, features = ["sync","serde"] }
scraper = { version = "0.24", default-features = false, optional = true }
[features]
scrape = ["std-core","dep:scraper","dep:selectors","dep:ammonia"]
script = ["std-core","dep:rhai"]
ai-scrape = ["scrape","script","agentic"]
```

Gate all new modules with `#[cfg(feature="std")]`; minimal/alloc/WASM graphs expose only `core`+`schema` stubs and keep `make check-wasm` green.

## Library design

### Rhai engine and API surface

- `script::Engine::new()` creates `rhai::Engine` with `set_max_expr_depths(64,64)`, `set_max_operations(200000)`, `set_max_string_size(1<<20)`, and `on_progress` timeout (5 s). Each exposed function re-checks `allow_fs`/`allow_net`/`offline` automatically; no per-function `requires_net` annotation is needed beyond docs.
- Register modules via `rhai::exported_module!` and `register_global_module`. The default injection includes raw `acorn::io::*` (including `files_all`/`files_all_with_max_depth`, `filter_ignored`, `read_file/write_file`, `http::get/download_with_progress`, cache/archive/source), `acorn::analyzer::*` (`Check`, `CheckBatch`, `CheckCategory`, `Standard`, `analyze_paths` DTO surface), `acorn::util::*` (`Label`, constants), `acorn::core::*` (`MimeType`, `Location`), and `acorn::schema::*` (`ResearchActivity`, `Identifier`, `Provider`). Convenience aliases (`acorn::http`, `acorn::fs`, `acorn::rad`, `acorn::scrape`, `acorn::infer`) point at the same functions.
- Types that are not `Copy` become `CustomType` with `serde_json::Value` bridging; scripts work with `Map`/`Value` and helpers like `to_json()`. Unknown raw `Result`/`Option` shapes are not leaked.
- `script::resolver` handles `(a)` explicit path `(b)` `files_all_with_max_depth` discovery of `.acorn/scripts/**/*.rhai` and XDG config `(c)` curated `assets/scripts` via `rust-embed`. `script::version` parses `// @acorn 0.2` and fails closed on mismatch.

### Hooks that cover every workflow step in v1

Dispatch in library, not per command, so `acorn`, `serve rpc`, `workflows/process` (`crates/acorn-lib/src/io/api/json_rpc/mod.rs:322`), and `acorn-tui` all fire the same points:

- `before_check`/`after_check` around `analyzer::host::service::analyze_paths` classification.
- `before_format`/`after_format`, `before_link`/`after_link`, `before_enrich`/`after_enrich`, `before_validate`/`after_validate` around `io::workflow/repository::WorkflowAction`.
- `before_export`/`after_export` around `crates/acorn-cli/src/commands/export/mod.rs:61`.
- `before_gather`/`on_extract`/`after_gather` around `discovery::Records` and `scrape::pipeline`.
- `before_workflow`/`after_workflow` around `RepositoryWorkflowRegistry::process`.

`runtime::execute_hook(hook, ctx: Map)` can skip, replace, or fail the default pipeline by returning a value, `false`, or a thrown `EvalAltResult` mapped to `CheckBatch`.

### Scraping pipeline

- `scrape::fetcher` wraps `io::http` with `offline` guard, cache TTL, `with_progress`/`ProgressType`, and auth header reuse.
- `scrape::extractor` runs `scraper::Html::parse_document` + `selectors` for `--selector` mappings (`"h1.title:title"`), sanitizes via `ammonia`/`percy-dom`, then if required fields are missing and `inference.is_some()` calls `InferenceProvider::extract(schema_json, sanitized_html, prompt)`.
- Deterministic→AI fallback is strictly ordered. No separate `--ai` flag exists.

### Pluggable inference (Needle is one sidecar)

```rust
#[async_trait] pub trait InferenceProvider: Send + Sync {
    async fn extract(&self, schema: &Value, content: String, prompt: Option<String>) -> ApiResult<Value>;
}
```

- `SidecarRegistry` resolves `InferenceBackend::{Needle, OpenAI, Anthropic, Auto, Sidecar(String)}`. `--inference needle` maps to `SidecarKind::Needle` (existing needle adapter); `sidecar:<path|oci://…>` resolves any `ModelKitManifest`.
- `SidecarProvider` wraps any `Sidecar` impl; `RemoteProvider` reuses `Configuration`/env resolution for bearer tokens (existing `io::api::openai` pattern). `Backend::Auto` tries Needle if a cached model is present, otherwise remote if `allow_net`, otherwise errors with actionable help (mirrors `download model` fallback UX).
- Both providers receive the same `schemars`-derived `ResearchActivity` JSON Schema and are tested through the pure `responses_request` mapper pattern.

## CLI design

### Gather

Extend `crates/acorn-cli/src/cli/arguments/mod.rs:748` `Gather`:

- `#[arg(long)] scrape: bool` – enables HTML fetch for `http(s)://` inputs. `INPUT: Vec<String>` already accepts a list of URLs and is now documented and parallelized.
- `#[arg(long, value_name="SELECTOR", requires="scrape")] scrape_selector` renamed to `selector: Vec<String>` – repeatable `"css:field"` values.
- `#[arg(long, value_name="PATH")] script: Option<PathBuf>` – shared via flattened `ScriptArgs`.
- `#[arg(long, value_enum)] inference: Option<InferenceBackend>` – presence enables AI fallback; no `--ai` flag.
- Existing `allow_net`/`allow_fs` are lifted into the shared `ScriptArgs` flattened into `Arguments` (`crates/acorn-cli/src/cli/mod.rs:335`) and every workflow struct so all intercepted commands share the same gates. `allow_fs` defaults to `.` (`current_dir()` canonicalized via `SafePath`).

Example:

```sh
acorn gather https://a.example/one https://b.example/two --scrape --selector "h1:title" --selector ".abstract:description"
acorn gather https://example.com/lab --scrape --selector "h1:title" --inference openai --allow-net
acorn gather https://example.com/lab --scrape --script ./extract.rhai --inference needle --allow-net --allow-fs ./out
```

`crates/acorn-cli/src/commands/gather/mod.rs` threads `ScrapeOptions` into `DiscoveryExtension` (`crates/acorn-lib/src/analyzer/host/discovery/mod.rs:232`), reuses `records.persist`/`candidates`/`enrich`, and fans out URL fetches via `rayon` capped by `--threads`.

### Script command

New `crates/acorn-cli/src/commands/script/mod.rs`:

```
acorn script run  <PATH> [--arg k=v] [--allow-net] [--allow-fs <path>=.] [--inference <engine>]
acorn script lint <PATH>
acorn script doc  [--format json|markdown]
```

`run` checks the version header, compiles/lints to `CheckBatch`, and executes with `Scope` containing `args` and the `acorn` modules. `lint` is compile-only; `doc` prints registered `Module` signatures.

Register `Commands::Script` (`crates/acorn-cli/src/cli/mod.rs:32`) and match at `crates/acorn-cli/src/commands/mod.rs:106` via `with_runtime`.

### MCP / JSON-RPC

Register `gather.scrape` and `script.execute` in `ToolRegistry::acorn()` (`crates/acorn-lib/src/io/mcp/tools.rs:278`) and `OperationRegistry::acorn()` (`crates/acorn-lib/src/io/api/json_rpc/mod.rs:282`); respect `InvocationContext.allow_mutation` and `RpcHttpService` auth.

## Phases (P0/P1 combined)

**P0/P1 – Rhai core, raw API, global flags, scaffolding (single phase):** deps/features, stub `script`/`scrape`/`inference`/sidecar registry modules, `rust-embed` examples, `ARCHITECTURE.md` – plus engine sandbox with default `acorn::{util,core,schema,io,analyzer}` raw mirrors (including `io` raw `files_all`/`http` etc. and full `analyzer` inspection) with automatic `allow_fs`/`allow_net` gates, resolver (a/b/c), `// @acorn 0.2` gate, global `ScriptArgs`, `acorn script` subcommands, and no-op hook wiring at all workflow boundaries.

**P2 – Deterministic gather scrape:** `scrape::fetcher`/`extractor` (`scraper`+`ammonia`), `pipeline::scrape_many`, multi-URL parallel `gather` with `--scrape`/`--selector`.

**P3 – Pluggable AI fallback:** `SidecarRegistry` + `SidecarProvider` (any sidecar) + `RemoteProvider` (`Configuration`) + `--inference` resolving `Backend::Auto`; `on_extract` fallback ordered deterministic→AI.

**P4 – Hardening:** extend `validate_watch_arguments` (`crates/acorn-cli/src/commands/mod.rs:627`), MCP/operation exposure, `make help` regen, `docs/src/packages/scripting.md` hook catalog, focused policy tests, and TUI hook reuse.

# Validation

Run `make lint`, `cargo check -p acorn-lib --no-default-features`, `make check-wasm`, and `make help` after CLI changes. Add unit tests for `script::lifecycle::sandbox_allows_cwd_blocks_parent`, `scrape::fetcher::offline_fails`, `scrape::extractor::deterministic_then_ai`, `inference::registry::auto_selects`; integration fixtures under `tests/fixtures/scrape/*.html` and `tests/fixtures/scripts/*.rhai` with `insta` snapshots; CLI tests for multi-URL scrape, selector parsing, `--allow-fs` default, `// @acorn 0.2` mismatch, and `offline` + `allow_net` gating.

# Acceptance criteria

- `acorn gather` accepts `INPUT...` containing multiple `https://` URLs; with `--scrape` it fetches them in parallel (capped by `--threads`), applies repeatable `--selector` mappings in order deterministic→AI fallback, and produces validated RAD candidates.
- When `--inference` is omitted, no AI call is made. When any `--inference` value is present, AI fallback is enabled automatically; there is no separate `--ai` flag.
- The flag is `--selector` (not `--scrape-selector`); help documents the `css:field` shape and rejects the old name.
- Every Rhai script receives `acorn::{util,core,schema,io,analyzer}` raw mirrors (including `acorn::io::files_all`/`http::*` and `acorn::analyzer::Check`-family) without an explicit import; the full API is available from day 1.
- Network and filesystem calls from Rhai are automatically gated by `--allow-net`/`--allow-fs` (default `.` = invocation CWD) and global `offline`; a script touching a disallowed path or host fails with a typed check error.
- Needle is one `Sidecar` among many; `--inference sidecar:<path>` and `--inference needle/openai/anthropic/auto` all resolve through the same `InferenceProvider` surface with the same JSON-Schema-constrained prompt.
- Rhai can intercept every workflow step (`before/after_check`, `before/after_format`, `before/after_link`, `before/after_export`, `before/on_extract/after_gather`, `before/after_workflow`) in v1, via library hooks, whether invoked from CLI, TUI, or JSON-RPC `workflows/process`.
- Scripts are discovered from explicit path, `.acorn/scripts/**/*.rhai` + XDG, and curated `rust-embed` examples; `--allow-fs` defaults to the invocation directory and `gather` multi-URL fixture coverage passes.
- No new runtime dependency leaks into minimal/WASM graphs, and default/alloc/WASM validation remains green.
