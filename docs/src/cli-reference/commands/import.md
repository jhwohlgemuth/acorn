# 📥 Import

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Import local or remote research-content buckets and BagIt archives, or populate ACORN's model catalog.</p>
</blockquote>

## Research content and BagIt archives

Use `acorn import` without a subcommand to transfer research content from a local directory, a remote repository, or a 7z, ZIP, TAR, or TAR.GZ BagIt archive:

```sh
acorn import ./local-bucket
acorn import https://github.com/example/research-data
acorn import https://example.org/deposit.tar.gz --output ./content
acorn import https://example.org/deposit.7z --output ./content
```

Archive type is inferred from a bounded content header. `--archive-format 7z|zip|tar|tar.gz` bypasses inference, which is useful for extensionless downloads. Extracted archives must contain valid BagIt metadata at their root or beneath one enclosing directory; their complete SHA-256 or SHA-512 payload manifest is verified before files are imported. Extraction rejects traversal paths, links, duplicate destinations, and writes through pre-existing symlink components.

`--filter`, `--ignore`, `--flatten`, and `--clobber` have the same transfer meaning as bucket download. `--config` loads bucket sources from an ACORN configuration. Offline import is accepted only when every selected source is local. The original archive URI is retained as transfer provenance.

Import and [`download`](download.md#bagit-archives) share one BagIt archive resolution path. The two commands differ only in transfer policy: import accepts local directories, local bucket entries in configuration, and offline local transfers, while top-level download is reserved for remote repositories and verified BagIt archives. Both publish only the contents of the bag's `data/` directory, and neither copies BagIt tag files to the output.

Configuration accepts bare bucket paths and URLs, or a named `source` string when a stable label is useful:

```yaml
buckets:
  - ./local-bucket
  - name: shared-research
    source: https://github.com/example/research-data
```

Use a detailed `codeRepository` object for provider-specific fields. The existing `repository` key remains accepted, and all forms use the same fallible source parser as positional command input.

Top-level `acorn download` remains for remote repository transfers and rejects local sources and offline operation. Model commands retain their own local and offline behavior.

## Model metadata

`acorn import model` can populate the general model catalog or inspect selected Hugging Face repositories for GGUF metadata. Repository inspection records GGUF file URLs, exact quantizations, revisions, and byte sizes without downloading model weights or checksum sidecars.

### Import the model catalog

Run the command without model selectors or a configuration file to populate the local database from the models.dev catalog:

```sh
acorn import model
```

If the model table is already populated, ACORN leaves the existing records in place.

### Import selected Hugging Face models

Supply one or more Hugging Face repository IDs to inspect only those repositories:

```sh
acorn import model openai/gpt-oss-20b

# Comma-separated and space-separated repository IDs are accepted
acorn import model openai/gpt-oss-20b,Qwen/Qwen2.5-0.5B-Instruct-GGUF
```

Use `--model-file` to load repository IDs from a local path, `file://` URI, or HTTP(S) URL:

```sh
acorn import model --model-file ./models.txt
acorn import model --model-file file:///path/to/models.yaml
acorn import model --model-file https://example.org/models.json
```

The document may be a newline-separated plain-text list, a JSON or YAML list of repository IDs, one model-details object, or a list of model-details objects. For minimal model details, `id` is preferred and `name` is used when `id` is absent:

```json
[
    {
        "name": "gpt-oss",
        "id": "openai/gpt-oss-20b"
    },
    {
        "id": "Qwen/Qwen2.5-0.5B-Instruct-GGUF"
    }
]
```

For catalog records that declare `open_weights` or `weights`, ACORN uses explicit Hugging Face repository URLs from `weights`. When an open-weight record has no declared weight source, ACORN uses its `id` to search for a GGUF fallback, including when that ID is not itself a Hugging Face repository. Closed models and records with non-Hugging Face weight sources are skipped. Entries that still cannot resolve produce warnings without aborting the remaining catalog import. This allows catalogs containing both hosted and open-weight models to be used directly:

```sh
acorn import model --model-file https://research.ornl.gov/api/models.json
```

Positional repository IDs, entries from `--model-file`, and supported `--config` entries are combined and deduplicated before import.

### Synchronize imported models

Pass `--sync` to add the unique model identifiers resolved by this invocation to the ACORN configuration's `models` list and to both OpenCode and llama-swap configuration. Existing ACORN model entries are preserved and duplicate identifiers are omitted. Select one inference target with `--sync opencode` or `--sync llama-swap`:

```sh
acorn import model openai/gpt-oss-20b --sync
acorn import model openai/gpt-oss-20b --sync opencode
acorn import model openai/gpt-oss-20b --sync llama-swap
```

Import resolves and persists repository metadata; it does not download weights. Synchronization therefore adds only imported models whose GGUF files are already available in the configured models directory by default. Models without a resolvable local GGUF are reported and skipped.

Add `--force` to skip the local existence check and assume each imported model is located at `<models-dir>/<model-id>`:

```sh
acorn import model openai/gpt-oss-20b --sync --force
```

Use `--dry-run` without `--sync` to resolve and report model metadata without persisting it to the database:

```sh
acorn import model openai/gpt-oss-20b --dry-run
```

Combine `--dry-run` with `--sync` to also print ACORN and inference-configuration diffs without writing files:

```sh
acorn import model openai/gpt-oss-20b --sync --dry-run
```

Inline `--sync` is additive and operates only on models resolved by the current import. Use the standalone `acorn sync` command to reconcile every model in ACORN configuration, override target paths or the models directory, preview changes independently, or remove stale ACORN-managed entries with `--prune`.

When a repository has no GGUF files, ACORN searches for a GGUF repository derived from that model. Control fallback discovery with:

```sh
# Search at most 50 candidate repositories
acorn import model openai/gpt-oss-20b --search-limit 50

# Require GGUF files in the selected repository
acorn import model Qwen/Qwen2.5-0.5B-Instruct-GGUF --no-fallback

# Choose interactively when fallback discovery finds multiple candidates
acorn import model openai/gpt-oss-20b --interactive
```

The default search limit is 20 and can also be set with `ACORN_SEARCH_LIMIT`.

### Import models from configuration

Use `--config` to inspect Hugging Face models listed in an ACORN configuration file:

```sh
acorn import model --config .acorn.json
```

A detailed entry can select a repository revision:

```json
{
    "models": [{
        "name": "qwen-gguf",
        "source": {
            "provider": "huggingface",
            "location": "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF"
        },
        "revision": "main"
    }]
}
```

Individual model selectors must be Hugging Face repository identifiers. `--model-file` identifies a document containing selectors; it does not import a local weight file or a direct model-weight URL.

### Database, authentication, and offline behavior

Use the global `--database-path` option to select the local database:

```sh
acorn --database-path ./acorn.db import model openai/gpt-oss-20b
```

With `--no-local-database`, selected repositories are still resolved and reported, but their metadata is not persisted. A catalog import without selectors is skipped:

```sh
acorn --no-local-database import model openai/gpt-oss-20b
```

For private or gated Hugging Face repositories, set `HF_TOKEN`, `HF_API_TOKEN`, or `HUGGINGFACE_HUB_TOKEN`. Global `--offline` mode rejects HTTP(S) model-list URLs and remote Hugging Face metadata import; local paths and `file://` model lists can still be read.

### Use imported metadata during download

After importing metadata, constrain the download by exact quantization and available GPU memory:

```sh
acorn import model openai/gpt-oss-20b --search-limit 20
acorn download model openai/gpt-oss-20b \
  --quantization Q4_K_M \
  --gpu-memory 24GB
```

See [Download](download.md#models) for download selection, filtering, split-shard sizing, and configuration details.

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Select model weights with <a href="download.md#models">Download</a>.</p>
