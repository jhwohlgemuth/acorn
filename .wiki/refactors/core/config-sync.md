---
type: Plan
title: "Synchronize llama-swap and OpenCode configuration"
description: "Generate and update local inference configuration from ACORN model entries."
difficulty: high
impact: high
package: [acorn-cli, acorn-lib]
tags: [feature, cli, config, models, llama-swap, opencode]
status: completed
---

# Goal

Add an idempotent `acorn sync` command that resolves ACORN model entries to downloaded GGUF files and merges the corresponding models into llama-swap and OpenCode project configuration.

# Status

Completed. Configuration validation and deterministic model resolution, state-preserving target merges and opt-in pruning, unified previews, and coordinated multi-target writes are implemented.

# Evidence

- `ApplicationConfiguration.config` now carries optional llama-swap and OpenCode sync settings, and `ApplicationConfiguration.models` provides model selectors.
- `acorn download model` and sync resolution share `~/.models` through `DEFAULT_MODELS_DIRECTORY`.
- `DEFAULT_MODELS_DIRECTORY` and `DEFAULT_LLAMA_SWAP_CONFIG_PATH` centralize the default sync paths.
- GGUF resolution uses shared `files_all` and `FilterSet` utilities, produces `ModelDetails` with absolute local paths, and validates complete, consistently named shard groups through `Shard`/`Shards`.
- Application configuration loading and GGUF model resolution, plus OpenCode provider upsert, default-model selection, CST rendering, and opt-in pruning, live on their owning configuration types.
- llama-swap model rendering, path resolution, state-preserving YAML merge, and managed-model pruning live on its sync configuration implementation.
- CLI sync arguments convert into target overrides through `From<&arguments::Sync>`.
- `Config::merge` applies full field-level runtime overrides, while `Config::merge_cli_overrides` overlays only CLI-supported path and models-directory fields so configured llama-swap model defaults and OpenCode provider values survive path-only CLI overrides.
- Target `path` fields deserialize from application configuration but use `skip_serializing`, preventing application-only paths from leaking into generated target configuration.
- `sync::Config` validates selected target settings and generated model IDs at the synchronization boundary.
- `ModelEntry::requests` and `ModelRequest::resolve` preserve detailed entry filters and ignores, accept direct local GGUF files, and fail on missing, ambiguous, duplicate, filtered, or malformed-shard inputs.
- llama-swap synchronization loads existing YAML, preserves unrelated root, model, and metadata values, writes supported `cmd` entries, and marks generated models with `metadata.acorn` for safe pruning.
- OpenCode synchronization retains modeled root values, unrelated providers, stale managed models by default, and comments outside the managed provider while updating `npm`, `name`, `options.baseURL`, and keyed models through the JSONC CST.
- Dry-run renders and validates every selected target, prints unified diffs, and creates no directories or files.
- Normal synchronization stages every changed target before committing any, restores prior files after a later commit failure, and reports changed paths only after success.
- llama-swap requires a command for each model and supports the minimal `llama-server --port ${PORT} --model <path>` form. [llama-swap configuration](https://github.com/mostlygeek/llama-swap/blob/main/docs/configuration.md)
- OpenCode supports custom OpenAI-compatible providers through `provider.<id>`, `npm: "@ai-sdk/openai-compatible"`, `options.baseURL`, and a keyed model map. [OpenCode providers](https://opencode.ai/docs/providers)

# Recommendation

Retain the implemented `ApplicationConfiguration.config` object with camel-case `llamaSwap` and `opencode` sections:

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

Keep synchronization local and deterministic. CLI path overrides should remain distinct from full configuration merges, selected outputs should be rendered and validated before coordinated writes, and pruning should remain explicit. llama-swap mappings and OpenCode CST updates should continue preserving configuration ACORN does not own.

# Validation

- `cargo nextest run --package acorn-lib io::sync` (34 passed).
- `cargo nextest run --package acorn-cli commands::sync` (4 passed).
- Focused sync tests cover field-level precedence, deterministic resolution, llama-swap and OpenCode preservation/pruning, JSONC comment preservation, stale root-model protection, unified previews, no-write dry runs, repeated-sync stability, pre-write render failure, coordinated rollback, and successful two-target writes.
- `make lint`.
- `make help`.
- `make test` (775 passed, 13 skipped).

# Acceptance criteria

- [x] `acorn sync` selects both targets when neither target flag is supplied and only explicitly selected targets otherwise.
- [x] Resolved GGUF files are represented as `ModelDetails`, and download and sync share the same default models directory.
- [x] Default sync paths are defined once in the shared application constants module.
- [x] Application and OpenCode configuration behavior is implemented by their owning configuration types rather than CLI-only wrappers.
- [x] The CLI command delegates llama-swap rendering, target synchronization, and output writing to library configuration methods.
- [x] CLI path overrides preserve all non-path values from `ApplicationConfiguration.config`.
- [x] Detailed model entries apply their configured filters and invalid or ambiguous models fail with actionable errors.
- [x] Valid configured GGUF models appear in llama-swap and the dedicated OpenCode provider with unique, consistent model IDs.
- [x] Dry-run suppresses writes.
- [x] Existing unrelated configuration survives synchronization for both targets, and repeated sync is stable.
- [x] Dry-run emits unified diffs, and stale managed models for both targets are removed only when requested.
- [x] Every selected output is validated before any target file is changed.
