# 🔄 Sync

> Synchronize downloaded GGUF models with llama-swap, OpenCode, VS Code, and Goose configuration

`acorn sync` resolves model entries to GGUF files in the local models directory. With no target flags, it includes every application whose command is available: `llama-swap`, `opencode`, `code`, and `goose`. App-specific flags form an inclusion set, so `--opencode --vscode` includes both OpenCode and VS Code. Every included target must have its command available unless it was explicitly included alongside `--force`; `--force` without app-specific flags does not bypass command detection. Use `--assume-models` separately to skip model filesystem verification.

Unless overridden, llama-swap configuration is written to `~/.config/llama-swap/config.yaml`. OpenCode uses an existing project or user `opencode.jsonc`/`opencode.json` configuration when available and otherwise writes `~/.config/opencode/opencode.jsonc`. VS Code writes `Code/User/chatLanguageModels.json` beneath the platform configuration directory. Goose writes `~/.config/goose/config.yaml` on macOS/Linux or `%APPDATA%\Block\goose\config\config.yaml` on Windows. Override those paths with `--vscode-config` and `--goose-config`.

VS Code receives a `customendpoint` provider using the Chat Completions API. Goose receives an active `openai` provider and the `OPENAI_HOST` and `OPENAI_BASE_PATH` settings; because Goose selects one active model, `config.goose.defaultModel` chooses it and otherwise the first synchronized model is used. Existing unrelated providers and settings are preserved. See the [VS Code custom endpoint reference](https://code.visualstudio.com/docs/agent-customization/language-models#_add-a-custom-endpoint-model) and [Goose configuration guide](https://goose-docs.ai/docs/guides/config-files/).

```yaml
config:
  vscode:
    path: ./chatLanguageModels.json
    url: http://localhost:8080/v1/chat/completions
    providerName: Local (llama-swap)
    maxInputTokens: 28672
    maxOutputTokens: 4096
  goose:
    path: ./goose.yaml
    host: http://localhost:8080
    basePath: v1/chat/completions
    defaultModel: acme/model
```

Generated llama-swap configuration follows the [gold base configuration](https://github.com/jhwohlgemuth/gold/blob/main/config/llama-swap/config.yaml): health-check, proxy, and macro defaults are added when absent, and model commands enable its offline/tool-use launch profile. Existing root settings and per-model proxy values are preserved. Explicit `extraArgs` replace matching launch-profile options. Generated filesystem paths use the platform's native form.

## Model lists

Use `--model-file` to load model IDs from a local path, `file://` URI, or HTTP(S) URL. Plain-text lists and JSON/YAML lists of IDs or model details are accepted:

```sh
acorn sync --model-file ./models.txt
acorn sync --model-file file:///path/to/models.yaml
acorn sync --model-file https://example.org/models.json
```

Entries from `--model-file` are combined with models from `--config`. Remote model-list documents cannot be loaded with global `--offline` mode.

Synchronization only includes models whose downloaded GGUF files can be resolved in `--models-dir` or the configured models directory by default. Unresolved entries are reported and skipped.

Use `--assume-models` to skip filesystem verification and assume each model is located at `<models-dir>/<model-id>`. For example, `acme/model` resolves to `<models-dir>/acme/model`:

```sh
acorn sync --model-file ./models.txt --assume-models
```

## Preview changes

Use `--dry-run` to print configuration diffs without writing files:

```sh
acorn sync --model-file ./models.txt --dry-run
```

Use `--prune` to remove stale ACORN-managed models from the selected target configurations.
