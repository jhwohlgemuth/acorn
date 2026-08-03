# 🔄 Sync

> Synchronize downloaded GGUF models with OpenCode and llama-swap configuration

`acorn sync` resolves model entries to GGUF files in the local models directory and updates both application configurations by default. Use `--opencode` or `--llama-swap` to select one target.

## Model lists

Use `--model-file` to load model IDs from a local path, `file://` URI, or HTTP(S) URL. Plain-text lists and JSON/YAML lists of IDs or model details are accepted:

```sh
acorn sync --model-file ./models.txt
acorn sync --model-file file:///path/to/models.yaml
acorn sync --model-file https://example.org/models.json
```

Entries from `--model-file` are combined with models from `--config`. Remote model-list documents cannot be loaded with global `--offline` mode.

Synchronization only includes models whose downloaded GGUF files can be resolved in `--models-dir` or the configured models directory by default. Unresolved entries are reported and skipped.

Use `--force` to skip filesystem verification and assume each model is located at `<models-dir>/<model-id>`. For example, `acme/model` resolves to `<models-dir>/acme/model`:

```sh
acorn sync --model-file ./models.txt --force
```

## Preview changes

Use `--dry-run` to print configuration diffs without writing files:

```sh
acorn sync --model-file ./models.txt --dry-run
```

Use `--prune` to remove stale ACORN-managed models from the selected target configurations.
