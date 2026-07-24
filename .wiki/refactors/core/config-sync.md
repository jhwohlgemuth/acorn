---
type: Plan
title: "Synchronize llama-swap and OpenCode configuration"
description: "Generate and update local inference configuration from ACORN model entries."
difficulty: high
impact: high
package: [acorn-cli, acorn-lib]
tags: [feature, cli, config, models, llama-swap, opencode]
status: in-progress
---

# Goal

Add an idempotent `acorn sync` command that resolves ACORN model entries to downloaded GGUF files and merges the corresponding models into llama-swap and OpenCode project configuration.

# Status

In progress. The command, model/provider configuration schema, GGUF discovery, llama-swap generation, OpenCode upsert/pruning, target selection, field-level CLI/config merging, dry-run write suppression, and atomic per-file writes are implemented. Remaining work includes configured target-path deserialization, fail-fast model resolution, llama-swap merge/pruning, unified dry-run diffs, validation of configured overrides, and validation of every selected output before any write.

# Evidence

- `ApplicationConfiguration.config` now carries optional llama-swap and OpenCode sync settings, and `ApplicationConfiguration.models` provides model selectors.
- `acorn download model` and sync resolution share `~/.models` through `DEFAULT_MODELS_DIRECTORY`.
- `DEFAULT_MODELS_DIRECTORY` and `DEFAULT_LLAMA_SWAP_CONFIG_PATH` centralize the default sync paths.
- GGUF resolution uses shared `files_all` and `FilterSet` utilities, produces `ModelDetails` with resolved local paths, and encapsulates shard grouping in `Shard`/`Shards`.
- Application configuration loading and GGUF model resolution, plus OpenCode provider upsert, default-model selection, and pruning, live on their owning configuration types.
- llama-swap model rendering and path resolution, `Options`-based target synchronization, and atomic per-file output writing live on sync configuration implementations.
- CLI sync arguments convert into target overrides through `From<&arguments::Sync>`.
- `Config::merge` applies full field-level runtime overrides, while `Config::merge_cli_overrides` overlays only CLI-supported path and models-directory fields so configured llama-swap model defaults and OpenCode provider values survive path-only CLI overrides.
- Target `path` fields currently use `#[serde(skip)]`, so paths shown in the proposed `ApplicationConfiguration.config` schema are not yet deserialized; output paths currently come from CLI overrides or target defaults.
- `ApplicationConfiguration::resolve_models` currently reduces detailed entries to names, does not forward entry filters, and logs and skips GGUF resolution errors.
- llama-swap synchronization currently serializes a new root containing `modelsDir` and `models`; it does not load or preserve an existing YAML configuration.
- OpenCode synchronization loads the existing configuration and preserves unrelated root values and providers while replacing ACORN's managed provider.
- Dry-run suppresses writes and prints complete generated configurations, not unified diffs.
- Selected targets synchronize sequentially, so a later target failure can occur after an earlier target was written.
- llama-swap requires a command for each model and supports the minimal `llama-server --port ${PORT} --model <path>` form. [llama-swap configuration](https://github.com/mostlygeek/llama-swap/blob/main/docs/configuration.md)
- OpenCode supports custom OpenAI-compatible providers through `provider.<id>`, `@ai-sdk/openai-compatible`, `options.baseURL`, and a keyed model map. [OpenCode providers](https://opencode.ai/docs/providers)

# Recommendation

## Configuration schema

Add an optional `ApplicationConfiguration.config` object with camel-case `llamaSwap` and `opencode` sections:

```yaml
config:
  llamaSwap:
    path: ./llama-swap.yaml
    modelsDirectory: ~/.models
    executable: llama-server
    contextSize: 8192
    ttl: 300
    extraArgs: ["--jinja"]
    environment: ["CUDA_VISIBLE_DEVICES=0"]
    models:
      qwen-gguf:
        contextSize: 32768
        ttl: 0
        aliases: ["coding"]
        extraArgs: ["--flash-attn", "on"]
  opencode:
    path: ./opencode.jsonc
    baseUrl: http://localhost:8080/v1
    providerId: llama-swap
    providerName: Llama Swap
    defaultModel: qwen-gguf
```

The llama-swap section supplies shared defaults. Entries in `llamaSwap.models`, keyed by ACORN model name, override executable, context size, TTL, extra arguments, environment, and aliases. Validate non-empty executables, positive context sizes, supported TTLs, unique aliases, `KEY=VALUE` environment entries, known override keys, and default models that exist.

Add typed llama-swap root and model structures with flattened unknown fields. Replace OpenCode's catalog-oriented provider field with OpenCode-specific provider and keyed model types.

## CLI

Add a top-level command:

```text
acorn sync
  --config <ACORN_PATH>
  --opencode
  --llama-swap
  --models-dir <PATH>
  --opencode-config <PATH>
  --llama-swap-config <PATH>
  --dry-run
  --prune
```

- With neither target flag, synchronize both configurations.
- With `--opencode` or `--llama-swap`, synchronize only the selected target; allow both flags together.
- Accept `--what-if` as a visible alias for `--dry-run`, consistent with other dry-run CLI arguments.
- Resolve `--config` through existing ACORN JSON, JSONC, and YAML loading. Path flags override paths in `ApplicationConfiguration.config`.
- Default to `~/.models`, `~/.config/llama-swap/config.yaml`, and the first existing project OpenCode file (`opencode.jsonc`, then `opencode.json`), creating `opencode.jsonc` when neither exists.
- Default the OpenCode provider to `llama-swap`, display name to `Llama Swap`, base URL to `http://localhost:8080/v1`, and executable to `llama-server`. Omit context size and TTL unless configured.
- Make `--dry-run` print unified diffs without writing. Make `--prune` remove stale managed models; otherwise only add or update entries.
- Keep sync local and deterministic: do not initialize the database, download weights, contact the network, or launch either application.

## Resolution and merge behavior

1. Reuse the model-entry normalization from `download model`, extracting pure shared functions where necessary.
2. Resolve direct local GGUF sources as-is. Otherwise search the model's expected directory beneath `modelsDirectory`, honoring its filter and ignore expressions. Use the first shard of one consistently named sharded set.
3. Fail before writing when a model has no GGUF candidate, multiple independent candidates, a duplicate generated ID, or an override for an unknown model.
4. Build shell-safe absolute commands as `llama-server --port ${PORT} --model <path>`, followed by configured context and extra arguments. Reject `--port`, `--model`, and `-m` in extra arguments because ACORN owns those options.
5. Merge llama-swap models by ID. Replace ACORN-owned fields, retain unrelated root and model values, and mark generated models in supported metadata for safe pruning. Preserve YAML semantics while allowing comments and formatting to normalize.
6. Upsert the configured OpenCode provider, `@ai-sdk/openai-compatible`, `options.baseURL`, and keyed models. Preserve unrelated root configuration and providers. Change the root default model only when `defaultModel` is configured.
7. Treat models inside ACORN's dedicated OpenCode provider as managed. During pruning, fail rather than leave the root model pointing to a removed entry without a configured replacement.
8. Update JSONC through its CST so mutations are written and comments outside managed nodes survive.
9. Validate every selected output before writing, write sibling temporary files, and rename them only after successful serialization.

# Validation

- Completed: `cargo test --package acorn-lib io::sync --lib` (17 passed).
- Completed: `cargo test --package acorn-cli commands::sync` (4 passed).
- Completed: field-level merge regression tests cover llama-swap path/models-directory precedence, OpenCode path precedence, preservation of configured target values, full runtime overrides, and addition of an otherwise unconfigured target.
- Completed: `cargo clippy --package acorn-lib --package acorn-cli --all-targets --no-deps -- -D warnings`.
- Completed: `cargo fmt --all -- --check`.
- Reconfirmed after implementing field-level CLI/config merging: `make test` (758 passed, 13 skipped).
- Current focused tests cover command construction, default paths and provider values, GGUF discovery/filtering/shard selection, field-level CLI/config precedence, and parent-directory creation.
- Remaining validation must cover configured target-path deserialization, detailed model filters, duplicate and ambiguous models, reserved arguments, llama-swap preservation/pruning, unified dry-run output, repeated-sync stability, and failure atomicity across both targets.
- After the remaining implementation, regenerate CLI help and run focused tests, `make lint`, `make test`, and `make help`.

# Acceptance criteria

- [x] `acorn sync` selects both targets when neither target flag is supplied and only explicitly selected targets otherwise.
- [x] Resolved GGUF files are represented as `ModelDetails`, and download and sync share the same default models directory.
- [x] Default sync paths are defined once in the shared application constants module.
- [x] Application and OpenCode configuration behavior is implemented by their owning configuration types rather than CLI-only wrappers.
- [x] The CLI command delegates llama-swap rendering, target synchronization, and output writing to library configuration methods.
- [x] CLI path overrides preserve all non-path values from `ApplicationConfiguration.config`.
- [ ] Detailed model entries apply their configured filters and invalid or ambiguous models fail with actionable errors.
- [ ] Valid configured GGUF models appear in llama-swap and the dedicated OpenCode provider with unique, consistent model IDs.
- [ ] Existing unrelated configuration survives synchronization for both targets, and repeated sync is stable.
- [ ] Dry-run emits unified diffs, pruning follows the documented non-destructive default, and stale managed models are removed only when requested.
- [ ] Every selected output is validated before any target file is changed.
