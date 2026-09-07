# 🔄 Sync

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Route downloaded GGUF models into supported applications with <code>acorn sync --dry-run</code>.</p>
</blockquote>

`acorn sync` resolves model entries to GGUF files in the local models directory. With no target flags, it includes every application whose command is available: `llama-swap`, `opencode`, `code`, and `goose`. App-specific flags form an inclusion set, so `--opencode --vscode` includes both OpenCode and VS Code. Every included target must have its command available unless it was explicitly included alongside `--force`; `--force` without app-specific flags does not bypass command detection. Use `--assume-models` separately to skip model filesystem verification.

<div style="height: 3rem" aria-hidden="true"></div>

```pikchr
scale = 0.72
Start: circle radius 0.07in fill 0x00662C thickness 0
arrow right 0.45in thick color 0x00662C
Sources: box "ACORN config +" "model file" wid 1.2in ht 0.52in rad 0.08in fill 0xF3F0FF thickness 0
arrow right 0.45in thick color 0x00662C
Resolve: box "Resolve local" "GGUF files" wid 1.05in ht 0.52in rad 0.08in fill 0xE8F1FB thickness 0
arrow right 0.45in thick color 0x00662C
Select: box wid 0.9in ht 0.52in rad 0.08in fill 0x00662C color 0x00662C
text "Select targets" color white at Select
arrow right 0.75in thick color 0x00662C
Llama: box "llama-swap" "YAML" wid 1.2in ht 0.52in rad 0.08in fill 0xE7F3E9 thickness 0
arrow right 0.55in thick color 0x00662C
End: circle radius 0.1in fill white thickness 0
circle radius 0.05in fill 0x00662C thickness 0 at End
OpenCode: box "OpenCode" "JSON / JSONC" wid 1.2in ht 0.52in rad 0.08in fill 0xFCE8E6 thickness 0 at Llama+(0,-0.72in)
VSCode: box "VS Code" "model JSON" wid 1.2in ht 0.52in rad 0.08in fill 0xE8F1FB thickness 0 at Llama+(0,-1.44in)
Goose: box "Goose" "YAML" wid 1.2in ht 0.52in rad 0.08in fill 0xFFF4D6 thickness 0 at Llama+(0,-2.16in)
Skip: box "Report and skip" wid 1.15in ht 0.52in rad 0.08in fill 0xF2F2F2 thickness 0 at Resolve+(0,-1.08in)
arrow from Select.e right 0.375in then down until even with OpenCode then to OpenCode.w thick color 0x00662C
arrow from OpenCode.e right 0.275in then up until even with End then to End.w thick color 0x00662C
arrow from Select.e right 0.375in then down until even with VSCode then to VSCode.w thick color 0x00662C
arrow from VSCode.e right 0.275in then up until even with End then to End.w thick color 0x00662C
arrow from Select.e right 0.375in then down until even with Goose then to Goose.w thick color 0x00662C
arrow from Goose.e right 0.275in then up until even with End then to End.w thick color 0x00662C
arrow dashed from Resolve.s to Skip.n thick color 0x00662C
text "unresolved" color 0x00662C at Resolve+(0.38in,-0.56in)
```

<div id="sync-workflow" data-pikchr-fit hidden></div>

<div style="height: 3rem" aria-hidden="true"></div>

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

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;See how every model stage connects in <a href="../model-workflows.md">Model workflows</a>.</p>
