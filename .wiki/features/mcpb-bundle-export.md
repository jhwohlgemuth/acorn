---
okf_version: "0.1"
type: Plan
title: MCP Bundle (.mcpb) export for the ACORN MCP server
description: Package the existing ACORN RMCP stdio server as a portable MCP Bundle ZIP (.mcpb) with a spec-conformant manifest.json and wire the artifact to Kitfile mcpServers referenced by ModelKit OCI layers.
difficulty: medium
impact: medium
package:
  - acorn-lib
  - acorn-cli
  - acorn-schema
tags:
  - mcp
  - mcpb
  - bundle
  - kitfile
  - modelkit
  - oci
  - export
status: planned
priority: next
---

# Project code boundaries

- **Update:** `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/commands/mod.rs`, `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-lib/src/io/mcp/mod.rs`, `crates/acorn-lib/src/io/mcp/tools.rs`, `crates/acorn-schema/src/schema/modelkit/kitfile.rs`, `crates/acorn-lib/src/util/constants/oci.rs`, `Cargo.toml` (workspace dependencies), crate `Cargo.toml` files for `acorn-lib` and `acorn-cli`, `CommandLineHelp.md`, and `Makefile`/`docs` help generation.
- **Create:** `crates/acorn-lib/src/io/mcp/bundle.rs` (or `crates/acorn-lib/src/mcpb/mod.rs` with `manifest.rs` + `bundle.rs`), `crates/acorn-cli/src/commands/create/mcp/mod.rs`, optional `crates/acorn-cli/src/commands/export/mcpb.rs` alias, fixtures under `tests/fixtures/mcpb/` (golden `manifest.json` + sample `.mcpb` inventories), and `docs/src/packages/mcpb.md` or expanded CLI docs.
- **Delete:** None. The `create mcp` stub error in `crates/acorn-cli/src/commands/mod.rs:162` is replaced; no existing download or OCI path is removed.

# Dependencies

- Reuse the existing RMCP stdio server contract in `crates/acorn-lib/src/io/mcp/mod.rs:22` (`Server`, `ServerHandler`, `run(offline)`) and the allowlisted `ToolRegistry::acorn()` in `crates/acorn-lib/src/io/mcp/tools.rs:256` plus `Server::tools()` filtered by `exposure.mcp`. Do not change tool names, schemas, or exposure without updating `acorn-schema::schema::agent::tools`.
- Consume the Kitfile schema already landed in `crates/acorn-schema/src/schema/modelkit/kitfile.rs:133` (`Kitfile.mcp_servers: Vec<McpServer>`) and `McpServer` (`kitfile.rs:160` with `name`, `path` ending `.mcpb`, `description`, `LayerInfo`) plus its validation (`kitfile.rs:702-731`, `validate_wire_shape:935` requiring `.mcpb` suffix and unique `name`) and `KITOPS_MODELKIT_MCPB_RAW` (`crates/acorn-lib/src/util/constants/oci.rs:11` `application/vnd.kitops.modelkit.mcpb.v1.raw`).
- Pin the MCP Bundle contract to the MCP project spec `MANIFEST.md` version `0.3` (2025-12-02) — required fields `manifest_version`, `name`, `version`, `description`, `author:{name}`, `server:{type, entry_point, mcp_config}` with variable substitution `${__dirname}`/`${HOME}`/`${user_config.*}` and optional `tools`, `tools_generated`, `compatibility` (`platforms: darwin|win32|linux`, `runtimes`).
- Keep the implementation `#[cfg(feature = "std")]` so `make check-wasm` remains green; `acorn-lib` ZIP generation must not leak into the `alloc`-only or WASM path.
- This plan occupies the **Agent tools and sidecars** and **CLI composition** lanes in the [parallel execution guide](../parallel-execution.md) and touches shared **CLI contract** and **Workspace graph** locks. It follows [portable ModelKit and Kitfile schema model](modelkit-schema-model.md) and is consumed by [ModelKit and ModelPack OCI interoperability](oci-model-artifact-downloads.md) for layer materialization, but it does not change the GGUF selection policy owned by that plan. Serialize edits to `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/commands/mod.rs`, `Cargo.toml`, and `CommandLineHelp.md`.

# MCP Bundle (.mcpb) export for the ACORN MCP server

## Goal

Make `acorn` distributable as a one-click local MCP server bundle: `acorn create mcp --output acorn.mcpb` (and optionally `acorn export --format mcpb`) produces a ZIP `acorn.mcpb` containing a spec-conformant `manifest.json` plus the compiled `acorn` binary executed via `serve mcp` over stdio, so any MCPB-compatible client (Claude Desktop, Claude Code, MCP for Windows, et al.) can install it and so a Kitfile can reference it declaratively:

```yaml
manifestVersion: 1.0.0
package: { name: acorn-research, version: 0.2.0 }
mcpServers:
  - name: acorn
    path: ./acorn.mcpb
    description: Allowlisted ACORN research-activity-data tools
```

The bundle must reuse the same `ToolRegistry`/`ServerHandler` surface already exercised by `acorn serve mcp`, declare its capabilities statically from `Server::tools()`, and integrate with the ModelKit OCI layer `application/vnd.kitops.modelkit.mcpb.v1.raw` without creating a second Kitfile model or changing archive safety invariants.

## Current evidence and gaps

| Concern | Current ACORN behavior | Target behavior |
|---|---|---|
| MCP server | `acorn-lib::io::mcp::Server` exposes allowlisted tools via RMCP stdio (`crates/acorn-lib/src/io/mcp/mod.rs:22-152`, `tools.rs:256` `ToolRegistry::acorn()`); `acorn serve mcp` runs it (`crates/acorn-cli/src/commands/mod.rs:540`, `cli/mod.rs:484`). | Same runtime is packaged; no tool rename or second server binary. |
| Bundle creation | `CreateCommands::Mcp` is stubbed `Err("MCP server creation is not yet implemented")` (`crates/acorn-cli/src/commands/mod.rs:162`); `Export` has handlers `Bag/Cff/Pdf/PowerPoint/Structured/Updates` but no `Mcpb` (`crates/acorn-cli/src/commands/export/mod.rs:140,183`) and `FileFormat`/`Target` have no `mcpb` (`cli/arguments/mod.rs`). | `acorn create mcp --output acorn.mcpb` creates a valid `.mcpb`; optional `export --format mcpb` delegates to the same core. |
| Kitfile reference | `Kitfile.mcp_servers` and `McpServer` already model bundles (`kitfile.rs:133,160`), enforce `.mcpb` suffix and unique `name` (`kitfile.rs:702-731`), participate in `local_layers()` (`kitfile.rs:550`), and define OCI media type `KITOPS_MODELKIT_MCPB_RAW` (`oci.rs:11`). | Bundles produced by this plan validate against that contract and can be inspected via the resolved-config JSON path (`from_resolved_json`/`to_resolved_json`). |
| MCPB spec | No `manifest.json` generation or ZIP assembly exists. | ZIP with `manifest.json` (0.3), `server/acorn` binary (0755), optional `icon.png`, no `yaml:"-"` leakage, `${__dirname}`-relative `mcp_config`. |
| Distribution | Manual `cargo run --bin acorn -- serve mcp` only. | One-file `.mcpb` installable in any MCPB-compatible client; Kitfile-aware `--kitfile` upsert. |

## Recommended implementation

### 1. Model the MCPB manifest in `acorn-lib` (no CLI dependency)

Create `crates/acorn-lib/src/io/mcp/bundle.rs` (or `crates/acorn-lib/src/mcpb/{mod,manifest,bundle}.rs`) behind `#[cfg(feature = "std")]`:

- Define `McpbManifest`, `McpbAuthor`, `McpbServer { type: McpbServerType::Binary, entry_point, mcp_config: McpbMcpConfig { command, args, env, platform_overrides } }`, `McpbCompatibility`, `McpbToolDecl { name, description }` matching `MANIFEST.md` 0.3. Derive `Serialize/Deserialize/JsonSchema` with `deny_unknown_fields` where the spec does; `manifest_version` is const `"0.3"`. Validate `name` lowercase/kebab, `version` semver (reuse `is_license`/`version` validator style in `kitfile.rs`), `author.name` required.

- Derive `tools: Vec<McpbToolDecl>` from `ToolRegistry::acorn().definitions().filter(|d| d.exposure.mcp)` sorted by `name`, matching `Server::tools()` (`mod.rs:39`). Set `tools_generated = false` (ACORN's tool set is static; only `format_and_enrich` is `open_world` but still enumerated). Preserve `description` from `ToolDefinition`.

- Fill workspace metadata from `env!("CARGO_PKG_VERSION")`, `env!("CARGO_PKG_DESCRIPTION")`, `CARGO_PKG_AUTHORS`, `CARGO_PKG_REPOSITORY`, `license = "MIT"`, `homepage`, `icon` default `None`. Allow caller overrides via `BuildOptions { name, display_name, version, description, author_name }`.

- Server config: `type = "binary"`, `entry_point = "server/acorn"`, `mcp_config.command = "${__dirname}/server/acorn"`, `args = ["serve","mcp"]` (append `"--offline"` only when `BuildOptions.offline` true, matching `Server::with_offline` in `mod.rs:35`). Set `platform_overrides.win32.command = "${__dirname}/server/acorn.exe"` when bundling a Windows binary. Keep `env = {}` unless `user_config` is introduced later.

- `BuildOptions { output: PathBuf, binary_path: Option<PathBuf>, icon: Option<PathBuf>, offline: bool, clobber: bool }`. No `user_config` in v1; leave field absent when `None`.

Exit condition: `McpbManifest::from_registry(&ToolRegistry::acorn()?)` round-trips through `serde_json` and `schemars` snapshot, and `manifest.json` contains exactly `Server::tools().len()` entries in deterministic order.

### 2. ZIP assembly with executable preservation

- Add `zip = { version = "2", features = ["deflate"] }` to `[workspace.dependencies]` and `acorn-lib/Cargo.toml` optional `std` dependency. Do not add `zip` to `acorn-schema` or `acorn-core`.

- In `bundle::build(options)`:
  1. Validate `output` ends with `.mcpb`, parent exists, refuse overwrite without `clobber`.
  2. Resolve binary: `binary_path` if supplied and exists, else `std::env::current_exe()` when it ends with `acorn`/`acorn.exe` and is a file, else `target/release/acorn` (and `target/release/acorn.exe` on Windows). Error with guidance `cargo build --release` if not found.
  3. Write to `output.tmp` then atomically rename. Use `zip::ZipWriter` with `FileOptions::default().compression_method(Deflated).unix_permissions(0o644)` for `manifest.json` and `0o755` for `server/acorn`/`server/acorn.exe`. Structure:
     ```
     acorn.mcpb
     ├── manifest.json
     ├── server/acorn      (or acorn.exe)
     └── icon.png          (optional, manifest.icon set when present)
     ```
  4. Serialize `manifest.json` pretty-printed with sorted keys; validate `serde_json::from_str::<McpbManifest>(&json).is_ok()` before writing.
  5. Preserve no extra top-level directory; ZIP entries are relative and safe (no absolute paths, no `..`).

Exit condition: `zipinfo -l acorn.mcpb` lists expected entries, `unzip -p acorn.mcpb manifest.json | jq .server.type` is `"binary"`, and extracted `server/acorn --help` succeeds.

### 3. CLI: `acorn create mcp` (primary) + optional `export --format mcpb` alias

`crates/acorn-cli/src/cli/arguments/mod.rs` — add:

```rust
#[derive(Clone, Debug, Args)]
pub struct CreateMcp {
  #[arg(short='o', long, value_name="PATH", default_value="acorn.mcpb")]
  pub output: PathBuf,
  #[arg(long, value_name="PATH")] pub binary: Option<PathBuf>,
  #[arg(long, value_name="PATH")] pub icon: Option<PathBuf>,
  #[arg(long)] pub name: Option<String>,
  #[arg(long)] pub description: Option<String>,
  #[arg(long, value_name="PATH")] pub kitfile: Option<PathBuf>,
  #[arg(long)] pub clobber: bool,
  #[arg(long)] pub offline: bool,
  #[arg(long)] pub dry_run: bool,
  #[command(flatten)] pub verbose: Verbosity,
}
```

Update `CreateCommands::Mcp(Box<CreateMcp>)` in `crates/acorn-cli/src/cli/mod.rs:451` and replace the stub in `crates/acorn-cli/src/commands/mod.rs:162` with `create::mcp::run()`:

- Build `BuildOptions` from args, call `acorn_lib::mcpb::build()`, emit `info!("{} Created {}", Label::pass(), output.display().cyan())` via `acorn::util::Label` and `owo_colors`, or on `--dry-run` print manifest to stdout via `write_stdout`.
- If `--kitfile` is `Some` or a `Kitfile`/`Kitfile.yaml` exists in `output.parent()` or cwd, read via `Kitfile::from_yaml` (`kitfile.rs:421`), upsert `mcpServers` entry `{ name, path: rel_output, description }`, validate (`kitfile.validate()`), write via `to_yaml()` (`kitfile.rs:443`), and log `Label::using()` for the mutated path. Reuse portable `Location::normalize()` logic; do not write `digest`/`diffId` to YAML (authoring projection).

Add `crates/acorn-cli/src/commands/create/mcp/mod.rs` and re-export in `commands/mod.rs:2`.

Optionally, add `FileFormat::Mcpb` and `McpbExportHandler` in `crates/acorn-cli/src/commands/export/mod.rs` delegating to the same `acorn_lib::mcpb::build()` (ignoring `path` input). This keeps `acorn export` parity for scripting while `create mcp` remains the documented entry point.

Exit condition: `cargo run -- create mcp --help` shows the new flags, `cargo run -- create mcp --dry-run` prints JSON without writing, and `cargo run -- create mcp --output /tmp/acorn.mcpb --kitfile ./Kitfile.yaml` upserts one `mcpServers` entry and passes `Kitfile::from_yaml(...).validate()`.

### 4. Kitfile and OCI wiring

- No schema change required: `McpServer.path` already requires `.mcpb` and `name` uniqueness is enforced. Ensure `validate_wire_shape:935` (`mcpServers: ["name","path","description", "digest","diffId"]`) and `local_layers:550` continue to include `mcpServers` layers.

- OCI: document that a Kitfile-referenced `.mcpb` becomes an OCI layer with media type `KITOPS_MODELKIT_MCPB_RAW` (`oci.rs:11`) when the Kitfile is packed via `kit` or future `acorn` packing. This plan does not implement packing; it prepares the bundle for consumption by the existing OCI classification in `oci-model-artifact-downloads.md`. Follow-on work may add `KitfileExt::mcpb_content()` analogous to `modelkit_content` in `crates/acorn-lib/src/io/oci.rs:366` for pull-time validation.

Exit condition: A Kitfile with `mcpServers: [{name: acorn, path: ./acorn.mcpb}]` validates, round-trips `to_yaml`/`from_yaml` without emitting layer identity, and its resolved JSON form (`to_resolved_json`) preserves `digest`/`diffId` when present.

### 5. Cross-platform and size considerations

- v1 ships one binary per `.mcpb` for the host that built it. Document `cargo build --target x86_64-unknown-linux-musl --release` + `--binary target/.../acorn` for alternate platforms, and `platform_overrides` for Windows `.exe` auto-append per `MANIFEST.md`.
- Future multi-arch bundle may embed `server/darwin/acorn`, `server/linux/acorn`, `server/win32/acorn.exe` with `platform_overrides` mapping — deferred.
- `Cargo.toml` release profile already sets `opt-level="z"`, `strip=true`, `lto=true` (`Cargo.toml:117`), so the binary is size-optimized; ZIP deflate further reduces it.

### 6. Documentation and housekeeping

- `make help` regenerates `CommandLineHelp.md`; update `docs/src/packages/acorn-web.md` or `docs/src/cli-reference/commands/create.md` (if exists) with the new command.
- Add `docs/src/packages/mcpb.md` summarizing install in Claude Desktop/Code/MCP for Windows and the Kitfile snippet.
- Update `.wiki/refactors/initiative.md` or `plan-network.md` cross-edges if needed; do not move `acorn-schema` ownership.

## Validation

### Unit and fixture

- `McpbManifest` golden: `tests/fixtures/mcpb/manifest.json` snapshot of `manifest_version=0.3`, `server.type=binary`, `mcp_config.command=${__dirname}/server/acorn`, `mcp_config.args=["serve","mcp"]`, `tools` count/order matches `Server::tools()`, `compatibility.platforms` when provided.
- `Bundle` fixture: build to temp, open with `zip::ZipArchive`, assert entries `manifest.json`, `server/acorn`, optional `icon.png`, no `..` or absolute entries, `server/acorn` has `0o755`, `manifest.schema_version` parses.
- Kitfile fixture: `tests/fixtures/modelkit/kitfile-mcpb.yaml` with one `mcpServers` entry; `from_yaml`/`to_yaml` round-trip preserves authoring shape, `from_resolved_json` with `digest`/`diffId` preserves identity, validation rejects missing `name`, duplicate `name`, and non-`.mcpb` suffix.
- Schema: no new `deny_unknown_fields` breakage; `McpbManifest` `schemars` snapshot committed.

### CLI

- `cargo nextest run -p acorn-cli -- create::mcp` (new tests) plus `cargo run -- create mcp --dry-run --output /tmp/acorn.mcpb` prints JSON.
- `cargo run -- create mcp --output /tmp/acorn.mcpb --kitfile tests/fixtures/modelkit/Kitfile.yaml --clobber` updates exactly one entry.

### Repository checks

```
cargo fmt
make lint          # cargo clippy --no-deps, deny missing-docs, no unwrap_used
cargo nextest run --run-ignored all
cargo check --no-default-features
cargo check --all-features
rustup target add wasm32-unknown-unknown && make check-wasm  # bundle is std-gated
make help && make book
make audit
```

Differential: validate with `npx @modelcontextprotocol/mcpb validate acorn.mcpb` (when available) or `unzip -l` + `jq` against the 0.3 spec; install the bundle in Claude Desktop/Code to observe ACORN tools listed.

## Risks and mitigations

- **Bundle size (10-30 MB):** Rust binary + ZIP deflate is expected; `opt-level=z` + `strip` already minimize it. Advise excluding `debug` symbols and sharing one bundle per platform rather than fat binary.
- **Platform coupling:** Client will fail if the binary arch/OS mismatches. Mitigate with `compatibility.platforms`, clear error when `--binary` is absent, and `platform_overrides.win32`.
- **Code-signing/notarization (macOS):** ZIP loses quarantine/signature context — document `xattr -d com.apple.quarantine` or signing outside ACORN.
- **Workspace graph lock:** Adding `zip` and new CLI args touches `Cargo.toml` and `cli/mod.rs` — serialize via single integration commit after leaf module is reviewed.
- **Kitfile mutation surprise:** Do not mutate a Kitfile without `--kitfile` or explicit discovery; default is to create the bundle only.
- **WASM leakage:** Gate `mcpb` behind `std`; unit tests for manifest are `alloc`-compatible.

## Acceptance criteria

- `cargo run -- create mcp --output /tmp/acorn.mcpb` (after `cargo build --release`) creates a ZIP where `unzip -l` lists `manifest.json` and `server/acorn`, `manifest.json` satisfies MCPB 0.3 required fields and `server.type="binary"` with `${__dirname}`-relative command, and `tools.len()` equals live `ToolRegistry::acorn().definitions().filter(|d| d.exposure.mcp).count()`.
- Installing the bundle in an MCPB-compatible client installs the server and lists the 8 allowlisted ACORN tools (`acorn.version`, `acorn.validate_*`, `acorn.export_*`, `acorn.propose/apply_logbook_graduation`, `acorn.format_and_enrich_research_activity_data`, `acorn.crosswalk_*` when `analysis` enabled) identical to `acorn serve mcp` output.
- `Kitfile.yaml` containing `mcpServers: [{name: acorn, path: ./acorn.mcpb}]` passes `Kitfile::from_yaml(...).validate()` and `to_yaml` omits `digest`/`diffId`, while resolved JSON preserves them.
- No change to GGUF inference policy or to the existing Hugging Face/ORAS download paths; `make test` (skips respected), `make lint`, and `make check-wasm` pass.
- `CommandLineHelp.md` regenerated; docs describe both standalone install and Kitfile reference.
- Evidence notes the MCPB spec commit`main` `MANIFEST.md` 0.3 (2025-12-02) baseline and KitOps `kitfile.md` `application/vnd.kitops.modelkit.mcpb.v1.raw` OCI media type used by `Kitfile`.

## Evidence

- ACORN RMCP server: `crates/acorn-lib/src/io/mcp/mod.rs:22` (`Server`), `crates/acorn-lib/src/io/mcp/mod.rs:141` (`run(offline)` via `rmcp::transport::stdio`), `crates/acorn-lib/src/io/mcp/tools.rs:256` (`ToolRegistry::acorn()`), `crates/acorn-lib/src/io/mcp/mod.rs:39` (`tools()`).
- Kitfile MCP bundle layer: `crates/acorn-schema/src/schema/modelkit/kitfile.rs:133` (`Kitfile.mcp_servers`), `kitfile.rs:160` (`McpServer` with `name`/`path`/`description`/`LayerInfo`), `kitfile.rs:702-731` (`.mcpb` suffix + `name` required/unique), `kitfile.rs:518,550` (`local_layers`/`layer_info_completeness` include `mcpServers`), `kitfile.rs:935` (`validate_wire_shape` allows `digest`/`diffId` only in resolved JSON).
- OCI media type: `crates/acorn-lib/src/util/constants/oci.rs:11` (`KITOPS_MODELKIT_MCPB_RAW = "application/vnd.kitops.modelkit.mcpb.v1.raw"`).
- CLI stubs and export dispatch: `crates/acorn-cli/src/cli/mod.rs:451` (`CreateCommands::Mcp`), `crates/acorn-cli/src/commands/mod.rs:162` (stub), `crates/acorn-cli/src/commands/export/mod.rs:140,183` (`ExportHandler`/`ExportRequest`).
- KitOps format: `pkg/artifact/kitfile.md:48` (MCP `.mcpb` verbatim raw layer) and `pkg/artifact/kitfile.go:108-118` (`MCPServer` type) at KitOps v1.15.0 commit `6b8162ae5da4d46f1d2af2beb43e7fb077f052f4` (same pin as [modelkit-schema-model](modelkit-schema-model.md)).
- MCP Bundle spec and CLI: MCPB announced as portable ZIP + `manifest.json` with `server.type` `node|python|binary|uv` and `mcp_config` variable substitution: <https://blog.modelcontextprotocol.io/posts/2025-11-20-adopting-mcpb/>; manifest 0.3 schema, `mcp_config` `${__dirname}`/`${user_config.*}`/`${HOME}`, `tools`/`tools_generated`, `compatibility.platforms`/`runtimes`, `server/lib` vs `server/venv` bundling: <https://github.com/modelcontextprotocol/mcpb/blob/main/MANIFEST.md> (raw `https://github.com/modelcontextprotocol/mcpb/raw/refs/heads/main/MANIFEST.md`); tooling and examples: <https://github.com/modelcontextprotocol/mcpb> (`CLI.md`, `examples/`).
