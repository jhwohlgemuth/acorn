# Model workflows

ACORN separates model metadata, model files, and inference configuration into three operations. Choose the command based on the state you want to change:

| Command | Changes | Use it when |
| --- | --- | --- |
| `acorn import model` | Resolves and stores Hugging Face GGUF metadata | You want to inspect repositories, discover GGUF fallbacks, or populate ACORN's model catalog without downloading weights |
| `acorn download model` | Resolves metadata and downloads selected GGUF files | You need model weights on the local machine |
| `acorn sync` | Updates OpenCode and llama-swap configuration | Model files already exist and you want to reconcile inference configuration |

Import and download accept positional model entries. All three commands can use configured entries and accept `--model-file` for a local path, `file://` URI, or HTTP(S) document containing model IDs or model details.

## Import metadata without downloading

Use `import model` to learn what GGUF files and quantizations are available, record that metadata in ACORN's database, or find a GGUF fallback for a repository that does not provide one directly:

```sh
acorn import model openai/gpt-oss-20b
acorn import model --model-file ./models.json
```

Import does not transfer model weights. This makes it useful for catalog maintenance and for checking model availability before deciding what to download.

Add `--dry-run` to resolve and report metadata without writing to the local database. This does not require `--sync`.

## Download model files

Use `download model` when the desired result is a local GGUF file. Download can constrain selection by quantization, GPU memory, filters, or whitelists:

```sh
acorn download model openai/gpt-oss-20b --quantization Q4_K_M
acorn download model --model-file ./models.json --gpu-memory 24GB
```

The command resolves metadata as part of the download, so a separate import is optional. Import first when you want to inspect or persist metadata independently.

## Synchronize inference configuration

Use `sync` when model files are already present and OpenCode or llama-swap configuration needs to reflect ACORN's model list:

```sh
acorn sync
acorn sync --model-file ./models.json --opencode
acorn sync --model-file ./models.json --llama-swap
```

Standalone sync performs full reconciliation and supports target-path and models-directory overrides. Add `--prune` when stale ACORN-managed entries should be removed. Use `--dry-run` to preview configuration diffs without writing files:

```sh
acorn sync --dry-run
acorn sync --prune --dry-run
```

By default, synchronization includes only models whose local GGUF files can be resolved. `--force` skips that check and assumes each model exists at `<models-dir>/<model-id>`. This is useful when preparing configuration before files are mounted or copied, but the generated paths are not verified.

## Download or import and sync in one step

Both model commands offer inline synchronization for the models resolved by that invocation. Inline sync also adds their identifiers to the ACORN configuration's `models` list, preserving existing entries and omitting duplicate identifiers:

```sh
# Download weights, then add the downloaded models to both configurations
acorn download model openai/gpt-oss-20b --sync

# Import metadata, then configure a model that will exist at the assumed path
acorn import model openai/gpt-oss-20b --sync --force

# Preview the configuration update without downloading or writing configuration
acorn download model openai/gpt-oss-20b --sync --dry-run --force
```

Inline `--sync` is additive and limited to the current import or download. `--dry-run` previews both the ACORN configuration update and inference-configuration updates without writing files. Use standalone `acorn sync` for repeated reconciliation of the complete configured model list, custom target paths, or pruning.

## Which command should I use?

- To inspect or catalog model metadata without downloading weights, use `import model`.
- To place GGUF weights on this machine, use `download model`.
- To update inference configuration for models that are already present, use `sync`.
- To download and configure selected models in one invocation, use `download model --sync`.
- To prepare configuration before model paths exist, add `--force`, preferably with `--dry-run` first.

See [Import](import.md#model-metadata), [Download](download.md#models), and [Sync](sync.md) for all command-specific options.
