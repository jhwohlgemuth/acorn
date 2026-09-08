---
okf_version: "0.1"
type: Plan
title: ACORN MCP server and MCP Bundle lifecycle
description: Harden the existing ACORN RMCP stdio server, package it as target-specific MCP Bundles, and connect the shared tool registry to client sync, Kitfile, OCI, database, inference, documentation, and release workflows.
difficulty: high
impact: high
package:
  - acorn-lib
  - acorn-cli
  - acorn-schema
tags:
  - mcp
  - mcpb
  - bundle
  - agent-tools
  - sync
  - kitfile
  - modelkit
  - oci
  - release
status: planned
priority: next
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/{archive.rs,mcp/mod.rs,mcp/tools.rs,sync/mod.rs,sync/opencode.rs,sync/goose.rs,sync/vscode.rs}`, `crates/acorn-schema/src/agent/opencode.rs`, `crates/acorn-cli/src/{cli/mod.rs,cli/arguments/mod.rs,commands/mod.rs,commands/serve/mcp/mod.rs,commands/create/mcp/mod.rs,commands/sync/mod.rs}`, focused adjacent tests, `Cargo.toml`, crate manifests only if feature wiring changes, `Makefile`, `.gitlab-ci.yml`, `scripts/upload-binaries.sh`, `CommandLineHelp.md`, the CLI/inference/package documentation, and this plan's entries in the feature index, dependency network, and parallel-execution guide.
- **Create:** `crates/acorn-lib/src/io/mcp/bundle.rs`, a client-neutral MCP launch/configuration leaf if sharing it avoids repeated command assembly, dedicated MCPB fixtures under `tests/fixtures/mcpb/`, and `docs/src/packages/mcpb.md`.
- **Delete:** None. Keep the existing `acorn serve mcp` adapter, the empty `commands/create/mcp/mod.rs` path, the Needle catalog export, and the established Kitfile schema; fill or augment them in place.

The implementation owner must narrow these planning boundaries to an exact write set before starting. Module indexes, CLI roots, manifests, generated help, release files, shared sync configuration, and shared snapshots are integration-owned even when a leaf implementation is prepared separately.

# Dependencies and execution conflicts

## Existing contracts to reuse

- `io::mcp::Server`, RMCP's stdio transport, and `ToolRegistry::acorn()` already implement `acorn serve mcp`. The server advertises and invokes the same ordered definitions used by Needle. This plan hardens that path; it does not introduce another server binary or registry.
- `ToolDefinition`, `ToolExposure`, `ToolEffects`, `ToolCallContext`, and `ToolResult` are the provider-neutral tool contract. Availability policy must be expressed as a projection of that contract rather than copied into MCP, MCPB, client sync, or inference adapters.
- `io::archive` already owns ZIP creation/extraction and the workspace already resolves `zip` through the `archive` feature. Add the minimum explicit-entry/mode support needed by MCPB instead of adding a second ZIP dependency or independent archive safety rules.
- `acorn-schema::modelkit::Kitfile` already models `mcpServers`, validates local `.mcpb` paths and unique names, and separates authoring YAML from resolved OCI identity. `KITOPS_MODELKIT_MCPB_RAW` already defines the raw-layer media type. No second Kitfile model is needed.
- The release pipeline already produces `x86_64-unknown-linux-musl` and `x86_64-pc-windows-gnu` binaries and uploads them through `scripts/upload-binaries.sh`. MCPB release work consumes those exact outputs.

## Prerequisites and plan edges

- The portable Kitfile contract is already implemented far enough for bundle references. Any remaining [ModelKit schema](modelkit-schema-model.md) work retains schema ownership; this plan only adds bundle inspection and an explicit authoring-file upsert.
- [Canonical research activity database](research-activity-database.md) owns durable revision and transaction semantics. This plan injects its repository into MCP handlers and does not invent separate MCP persistence.
- [Database-backed artifact generation](database-backed-artifact-generation.md) owns current/revision artifact selection. Its MCP inputs must register through the same tool registry after the typed source contract lands.
- [Agent inference and verified fixes](opencode-acp-inference.md) shares OpenCode agent schema, CLI roots, and MCP modules. Land the current OpenCode MCP configuration contract before either plan writes client configuration. Keep the ACP plan's first-release exclusion of a custom MCP-over-ACP bridge.
- [ModelKit and ModelPack OCI interoperability](oci-model-artifact-downloads.md) owns OCI classification, descriptor verification, and materialization. This plan supplies MCPB validation and the `McpBundle` component behavior; it does not broaden `download model` into a general installer.
- The current MCPB baseline is manifest `0.4`. Pin the schema and validator to a reviewed upstream tag or commit immediately before implementation and record that pin in fixtures. Do not implement against unpinned `main`.

## Integration locks and conflicts

| Lock | Conflicting work | Required sequencing |
| --- | --- | --- |
| Agent tool contract | Needle changes, ACP inference, database artifact tools, service extraction | Land the runtime-policy and dependency-injection contract first. Subsequent plans register handlers through it without changing transport adapters. |
| CLI contract | Parse/execution, startup, error-boundary, remote-create, ACP, CLI coverage | One owner changes `cli/`, `commands/mod.rs`, and generated help. Leaf MCPB and sync code lands before the single CLI integration commit. |
| OpenCode schema/config | ACP inference, sync consolidation | Freeze support for current and legacy MCP shapes first, then let one owner update the schema and JSONC-preserving writer. |
| Archive safety | Safe-path and unified import/download/BagIt work | Add an explicit safe ZIP-entry primitive only after reconciling with the active safe-path changes. Do not modify extraction policy or unrelated archive formats. |
| Database repository | Canonical database and database-backed artifacts | Consume the typed repository after it stabilizes. Do not edit migrations or persistence schema in this slice. |
| ModelKit/OCI | ModelKit schema and OCI download work | Kitfile upsert may land independently; bundle-layer inspection/materialization lands through the OCI owner. |
| Workspace graph | Feature-default, crate extraction, portability, dependency work | No dependency addition is expected. If feature wiring changes, the workspace-graph owner integrates it and reruns the supported feature matrix. |
| Sync configuration | Model workflow sync and duplication consolidation | Introduce MCP as an orthogonal sync capability while one owner controls shared `Config`, `Options`, selection, rendering, and snapshots. |
| Release and docs | CI runner pools, release work, generated help, book changes | Package scripts land before CI wiring. Regenerate help once after all CLI changes; one release owner updates build, upload, and release links. |

Suggested landing order: tool policy and injected services, protocol hardening, archive/MCPB library, `create mcp`, client sync, Kitfile/OCI hooks, then documentation and release integration. A slice that needs another lock stops at its leaf boundary and hands integration to that lock owner.

# Goal and decisions

ACORN will support one MCP implementation across direct execution, local agent configuration, bundled installation, ModelKit packaging, and future tools. The default server remains local, stdio-only, and read-only. Network-capable tools follow the existing global offline policy. Mutation requires an explicit server flag and an available configured database.

The public packaging command is `acorn create mcp`. Do not add `.mcpb` to `acorn export`: that command exports research data and currently requires RAD-oriented inputs, while MCPB packages the application itself. `acorn export --tools` remains the Needle-native `tools.json` export and its help text should say so explicitly.

Release bundles are target-specific. The first release matrix matches the existing binaries:

- Linux `x86_64-unknown-linux-musl`;
- Windows `x86_64-pc-windows-gnu`.

Local macOS users may package the running executable, but release manifests must not claim macOS compatibility until a signed macOS binary lane exists. A multi-platform or fat bundle is outside this plan.

# Recommended implementation

## 1. Make runtime policy and tool availability consistent

Introduce an immutable MCP runtime/service input containing offline state, mutation permission, the structured-output byte limit, and an optional configured research-activity repository. Keep `ToolRegistry::acorn()` as the safe compatibility constructor used by Needle and tests; add a dependency-aware constructor for interface composition roots.

Add one deterministic availability projection, such as `available_definitions(context)`, and use it for:

- MCP `tools/list` and `get_tool`;
- invocation authorization;
- the MCPB manifest's static tool list;
- tests comparing all projections.

The projection omits mutating tools unless mutation is enabled and omits open-world tools in offline mode. A tool that appears in `tools/list` must be callable under the same policy. Preserve `ToolEffects` as MCP annotations.

Inject the selected database/repository into `apply_logbook_graduation` and future persistent handlers. Remove its direct use of `Database::<Table>::default()`. The direct server composition root must honor `--database-path`, the selected backend, and `--no-local-database`.

Extend the CLI contract to:

```text
acorn [GLOBAL OPTIONS] serve mcp [--allow-mutation]
```

- The default is read-only.
- Existing global `--offline` controls open-world tool availability.
- `--allow-mutation` requires local persistence and fails before protocol startup when combined with `--no-local-database`.
- All protocol frames use stdin/stdout; tracing, progress, and diagnostics use stderr only.
- Keep stdio as the only transport. HTTP, SSE, authentication, and remote deployment need a separate plan.

## 2. Add a reusable MCPB plan and builder

Create a std-gated bundle module with a pure planning boundary and a filesystem materializer. The minimum public API is:

```rust
pub struct BundleOptions {
    pub output: PathBuf,
    pub binary: PathBuf,
    pub platform: McpbPlatform,
    pub overwrite: bool,
}

pub enum McpbPlatform {
    Linux,
    Win32,
}

pub struct BundleReport {
    pub output: PathBuf,
    pub manifest: Manifest,
    pub sha256: String,
    pub size: u64,
}

pub fn plan(options: &BundleOptions, registry: &ToolRegistry) -> ApiResult<BundlePlan>;
pub fn build(plan: BundlePlan) -> ApiResult<BundleReport>;
```

Model only the MCPB `0.4` fields ACORN emits: required identity, author, and server data; repository, homepage, documentation, and license metadata; compatibility platform; static `tools`; and `tools_generated: false`. Populate identity from Cargo metadata rather than accepting arbitrary package-name, version, or author overrides.

The bundle contains:

```text
acorn.mcpb
├── manifest.json
└── server/
    └── acorn          # Linux
    └── acorn.exe      # Windows bundle instead
```

Use `server.type: binary`, the matching `entry_point`, and an `mcp_config` command rooted at `${__dirname}` with arguments `serve mcp`. The manifest advertises the safe online MCP tool projection, so mutating tools are excluded. Do not include an icon until a suitable ACORN PNG asset is approved.

Augment `io::archive` with an explicit ZIP-entry writer that accepts a source, a validated relative destination, and a Unix mode. Reuse it from existing ZIP creation where that is a small mechanical change. MCPB generation requires deterministic entry order, fixed ZIP metadata, `0755` for the Linux executable, `0644` for data, no symlinks, and no absolute or parent-traversing names.

Validate the input binary and manifest before opening the destination. Write a sibling temporary file, finish and re-open it for structural validation, calculate SHA-256, then atomically rename. Refuse replacement unless requested and remove only the temporary file on failure.

## 3. Implement `acorn create mcp`

Fill the existing empty `commands/create/mcp/mod.rs` and change `CreateCommands::Mcp` to carry these arguments:

```text
acorn create mcp
    [--output PATH]
    [--binary PATH]
    [--platform linux|win32]
    [--kitfile PATH]
    [--clobber]
    [--dry-run]
```

- Default output: `./acorn.mcpb`.
- Without `--binary`, package `current_exe()` and infer the host platform.
- An external or cross-compiled binary requires `--platform`; validate the Windows `.exe` rule.
- `--dry-run` validates inputs and prints the manifest, inventory, and destination without writing.
- Report output path, platform, size, and SHA-256 with existing `Label` and styling conventions.

`--kitfile` is explicit. After a bundle is built successfully, read the named authoring Kitfile, upsert one `mcpServers` entry named `acorn`, use a path relative to the Kitfile directory, validate the whole document, and replace it atomically. Preserve unrelated content and authoring YAML style as far as the existing codec permits; never emit resolved `digest` or `diffId`. A Kitfile failure leaves the original file unchanged and reports that the bundle itself was already created.

## 4. Add opt-in MCP client synchronization

Add MCP as an orthogonal capability of the existing sync engine:

```text
acorn sync --mcp [--opencode | --vscode | --goose] [--dry-run] [--prune]
```

- Without client selectors, update installed compatible clients discovered through existing command/path conventions.
- With selectors, update only those clients.
- Reject `--mcp --llama-swap`; llama-swap is a model router, not an MCP client.
- MCP-only sync must skip model resolution and work without models or a local database.
- Resolve the running ACORN executable to an absolute path and configure `serve mcp`; do not assume a client inherits the same `PATH`.
- Without `--mcp`, preserve current sync output byte-for-byte.
- `--dry-run` uses existing file diffs. `--prune` removes only the ACORN-owned MCP entry.

Extend the existing adapters:

- **OpenCode:** preserve JSONC comments and unrelated configuration. Read the legacy direct `mcp` map already modeled by ACORN, then write the pinned current layout after the ACP plan refreshes that schema. The current upstream shape nests named local servers below `mcp.servers` and uses a command vector.
- **VS Code:** add a dedicated MCP path, defaulting to workspace `.vscode/mcp.json`; do not mix MCP servers into `chatLanguageModels.json`. Upsert `servers.acorn` with stdio type, the absolute command, and `serve mcp` arguments. Preserve unrelated servers and sandbox configuration.
- **Goose:** extend the existing YAML value transform with `extensions.acorn` using `type: stdio`, `enabled: true`, the absolute command, and `serve mcp` arguments. Preserve providers and other extensions.

Clients that install `.mcpb` directly remain outside sync. `sync --mcp` writes direct local-server configuration and must not create, install, or duplicate a client-managed bundle.

## 5. Connect package, persistence, and inference workflows

- **Kitfile:** reuse `McpServer`, current path/name validation, authoring/resolved codecs, and raw-layer media type. Add bundle inspection at the host boundary; do not put ZIP parsing in `acorn-schema`.
- **OCI:** extend the normalized artifact plan with an `McpBundle` role. `download model` reports and ignores that role because its policy selects model content. The planned general component materializer may verify and write selected MCPB layers but must never execute or auto-install them.
- **Database artifacts:** once typed current/revision selectors land, register them through the same tool registry. Use the injected repository and existing renderer contracts. Every enabled mutation follows the database plan's transaction, revision, actor/interface, and audit rules.
- **Needle:** continue deriving `tools.json` from the canonical registry and its Needle exposure. Do not make Needle speak MCP or consume MCPB.
- **ACP:** rely on the synchronized OpenCode MCP entry when an OpenCode session should see ACORN tools. Keep a custom MCP-over-ACP bridge out of the first release.
- **Future tools:** a definition and handler are registered once. Exposure and runtime policy determine whether it appears in Needle, direct MCP, and generated MCPB manifests.
- **Future service extraction:** keep availability and manifest calculation pure and dependency-injected so they can move to `acorn-service`; keep ZIP/filesystem work in the host adapter and CLI parsing/output in `acorn-cli`.

## 6. Package and publish target-specific bundles

Extend `make build` or add a focused packaging target that consumes the two existing release binaries and invokes `acorn create mcp` explicitly for Linux and Windows. Produce:

```text
acorn-v${VERSION}-x86_64-unknown-linux-musl.mcpb
acorn-v${VERSION}-x86_64-pc-windows-gnu.mcpb
```

Generate SHA-256 files for binaries and bundles. Extend the existing generic-package upload script idempotently and add both MCPB assets to the GitLab release. Pin a reviewed `@anthropic-ai/mcpb` validator version in CI; Node is a build-time validation dependency, not a shipped runtime.

Prepare MCP Registry-compatible release metadata and checksums, but leave publication to a follow-on that owns an accepted namespace and external release policy. Do not claim macOS support or publish unsigned macOS bundles from this workflow.

Update CLI create/serve/sync pages, inference documentation, the package index, the agentic-layer concept, and the embedded ACORN skill. Remove the stale statements and acceptance cases that expect `serve mcp` to fail. Add a dedicated MCPB package page covering direct stdio configuration, `sync --mcp`, bundle creation and installation, trust, offline policy, mutation opt-in, Kitfile use, release targets, and local macOS builds. Regenerate `CommandLineHelp.md` once.

# Validation

## Protocol and policy

- Spawn the compiled CLI and complete MCP initialize, `tools/list`, valid and invalid `tools/call`, and graceful shutdown over stdio.
- Assert stdout contains protocol frames only, with diagnostics on stderr.
- Verify online read-only, offline read-only, and mutation-enabled catalogs. Every advertised tool must be callable under the matching policy.
- Verify the selected database path/backend reaches persistent handlers and startup rejects mutation without persistence.
- Compare direct MCP and manifest tool declarations to the same ordered registry projection; retain independent Needle exposure assertions.

## Bundle and Kitfile

- Validate a golden manifest against the pinned MCPB `0.4` schema and official validator.
- Build twice from the same binary and assert identical bundle bytes and SHA-256.
- Inspect exact entries, normalized paths, modes, fixed metadata, manifest/binary agreement, platform declarations, and archive size.
- Cover missing/non-file binaries, Windows suffix mismatch, unsupported platform, unsafe entry names, existing output, clobber, dry-run, and cleanup after a failed build.
- Extract the Linux bundle and run the protocol smoke test against its binary. Structurally validate the cross-compiled Windows bundle in Linux CI.
- Verify explicit Kitfile upsert, repeat idempotency, relative paths, preservation of unrelated sections, authoring omission of layer identity, and no change after validation/write failure.

## Client and workflow integration

- Cover OpenCode legacy/current fixtures, JSONC comments, Goose providers/extensions, and VS Code servers/sandbox fields.
- Prove MCP-only sync succeeds with zero models, supports dry-run/prune, is idempotent, and leaves llama-swap untouched.
- Run existing sync golden tests without `--mcp` and require identical output.
- Verify `download model` reports but does not materialize MCPB layers; test bundle inspection in the general component boundary.
- Verify a newly registered fixture tool flows to each allowed projection without adapter-specific registration.

## Repository and release gates

```text
cargo fmt --check
make lint
make test
make check-portable
make help
git diff --exit-code -- CommandLineHelp.md
make book
```

Run focused MCP, archive, sync, ModelKit, OCI, CLI parse/execution, and release-script tests before the full gates. Validate both release bundles with the pinned upstream CLI, verify their recorded hashes, and dry-run every package URL before publishing.

# Acceptance criteria

- `acorn serve mcp` completes a real stdio MCP session with the policy-appropriate canonical tool catalog and no stdout contamination.
- Offline and mutation policy changes affect advertisement and invocation consistently; mutating calls use the configured database and revision transaction.
- `acorn create mcp` creates a deterministic, validated MCPB `0.4` archive containing the correct target binary and safe static tool manifest without changing RAD export semantics.
- `--kitfile` performs an explicit, validated, idempotent `mcpServers` upsert and preserves the authoring/resolved identity boundary.
- `acorn sync --mcp` configures detected or selected OpenCode, VS Code, and Goose clients without requiring models; existing sync output is unchanged when the flag is absent.
- ModelKit and planned OCI workflows recognize MCPB as a first-class package component while `download model` retains its GGUF selection policy and no workflow auto-executes a downloaded bundle.
- Linux musl and Windows GNU MCPB files, checksums, package URLs, and release links are produced from the same binaries already shipped by ACORN and pass the pinned upstream validator.
- CLI help, acceptance tests, book documentation, embedded skill text, plan evidence, dependency edges, and execution locks describe the implemented behavior accurately.

# Assumptions and deferred work

- MCPB manifest `0.4` is the compatibility target, pinned immediately before implementation.
- Server defaults are stdio, read-only, and online unless the existing global offline option is set.
- `create mcp` is the only MCPB production command; `export --tools` stays Needle-specific.
- Client sync is opt-in through `--mcp`; MCPB installation remains client-managed.
- The release matrix is Linux x86_64 musl and Windows x86_64 GNU. macOS signing/notarization, ARM releases, fat bundles, remote MCP transports, registry publication, localization, icons, user-configurable MCPB fields, and automatic installation require separate accepted work.

# Evidence

- Current ACORN server and registry: `crates/acorn-lib/src/io/mcp/mod.rs`, `crates/acorn-lib/src/io/mcp/tools.rs`, and adjacent tests.
- Current CLI wiring and packaging stub: `crates/acorn-cli/src/commands/mod.rs`, `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/commands/serve/mcp/mod.rs`, and the empty `crates/acorn-cli/src/commands/create/mcp/mod.rs`.
- Existing archive and ZIP dependency: `crates/acorn-lib/src/io/archive.rs` and `crates/acorn-lib/Cargo.toml`.
- Kitfile MCP layer: `crates/acorn-schema/src/modelkit/kitfile.rs` and `crates/acorn-lib/src/util/constants/oci.rs`.
- Existing client sync: `crates/acorn-lib/src/io/sync/` and `crates/acorn-cli/src/commands/sync/mod.rs`.
- Existing Linux/Windows release flow: `Makefile`, `.gitlab-ci.yml`, and `scripts/upload-binaries.sh`.
- MCPB specification and tooling: <https://github.com/modelcontextprotocol/mcpb/blob/main/MANIFEST.md> and <https://github.com/modelcontextprotocol/mcpb/blob/main/CLI.md>.
- VS Code MCP configuration: <https://code.visualstudio.com/docs/agent-customization/mcp-servers>.
- OpenCode MCP configuration: <https://opencode.ai/v2/docs/mcp-servers>.
- Goose extension configuration: <https://github.com/aaif-goose/goose/blob/main/documentation/docs/getting-started/using-extensions.md>.
- MCP Registry MCPB packages: <https://github.com/modelcontextprotocol/registry/blob/main/docs/modelcontextprotocol-io/package-types.mdx>.
