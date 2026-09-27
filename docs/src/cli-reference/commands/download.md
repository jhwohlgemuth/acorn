# 📥 Download
<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Fetch bucket content, verified BagIt archives, or local-model weights with <code>acorn download &lt;SOURCE&gt;</code>.</p>
</blockquote>

- See the [configuration](../configuration.md) documentation for details on configuring ACORN commands.
- By default, `download` will save files to `./content` in the working directory unless an output path is specified via the `--output` flag.

## Example usage
```sh
# Download research activity data from a single bucket repository URL
acorn download https://code.ornl.gov/research-enablement/buckets/nssd

# Download the verified payload of a BagIt archive
acorn download https://example.org/bags/nssd.zip --output ./content

# Download research activity data from a list of buckets
acorn download --config /path/to/.acorn.json

# Download research activity data to a specific output directory
acorn download --config /path/to/.acorn.yml --output /path/to/output

# Download selected files directly beneath the output directory
acorn download https://github.com/example/project --filter '^examples/quest/' --flatten --output ./quest

# Replace selected paths that already exist in the output directory
acorn download https://github.com/example/project --output ./content --clobber

# Download using JSONC configuration (supports comments and trailing commas)
acorn download --config /path/to/.acorn.jsonc
```

## Buckets

After a bucket transfer, ACORN ingests transferred JSON, JSONC, YAML, YML, ZONF, and recognized Markdown research activity files into the local `research_activities` table. Unrelated Markdown documentation is ignored. Each row has a portable NanoID and retains the typed RAD JSON plus provenance for the bucket, repository, relative path, and observation time. Exact DOI, RAiD, ISBN, and patent metadata identities are used when present; a repository-qualified RAD identifier always identifies the transferred activity.

Repeated transfers are idempotent. A matching candidate gains missing fields and unique array values, while conflicting populated fields remain unchanged and are recorded as provenance. Filters and ignore rules apply before transfer, so only transferred RAD files are ingested.

Use `--flatten` to discard repository directory components and save every selected file as a direct child of the output directory. Filters and ignore rules continue to match the original repository-relative paths. If multiple selected paths have the same filename, the command fails.

By default, a local bucket copy fails rather than overwrite an existing destination, while a complete existing remote file is skipped. Add `--clobber` to replace files, symlinks, or directories that conflict with selected output paths. Local imports obtain the replacement content before removing the old destination; remote downloads remove the selected destination before streaming its replacement. Unrelated contents beneath the output directory are preserved. This flag applies only to bucket transfers; the model subcommand's existing `--force` flag retains its synchronization-specific meaning.

If a transferred RAD file cannot be parsed or persisted, the files already written to the output directory are preserved and the command fails. Deprecated RAD fields are accepted with warnings and serialized under their canonical names in the database. Pass `--canonicalize` to rewrite deprecated fields in transferred files before ingestion, or `--strict` to reject them. Combining both flags canonicalizes first and then verifies the rewritten files with strict ingestion. Global `--no-local-database` performs the transfer but skips RAD parsing and candidate persistence; `--canonicalize` still rewrites deprecated fields when database access is disabled.

```sh
# Rewrite deprecated RAD aliases in downloaded files, then require canonical ingestion
acorn download --config /path/to/.acorn.json --canonicalize --strict
```

```sh
# Transfer bucket files without creating or updating local candidates
acorn --no-local-database download --config /path/to/.acorn.json
```

### Local and remote sources
Top-level `download` accepts remote bucket repositories and BagIt archives. Remote repositories require online mode, and local directory or local repository sources are still rejected. Use [`acorn import`](import.md#research-content-and-bagit-archives) for local directories and local bucket entries in configuration. This policy does not apply to `download model`, which retains its documented local-file and offline behavior.

```json
"buckets": [
    {
        "name": "nssd (remote)",
        "repository": {
            "provider": "gitlab",
            "id": 17411,
            "uri": "https://code.ornl.gov/research-enablement/buckets/nssd"
        }
    },
    {
        "name": "test (local)",
        "repository": {
            "provider": "git",
            "location": "file:./tests/fixtures/data/bucket/"
        }
    }
]
```

Bare paths and URLs are the concise form. Use a detailed `repository` object when a provider-specific field such as a GitLab project ID is required; the `codeRepository` key is also accepted.

### BagIt archives

A local path, `file:` URI, or HTTP(S) URI that names a 7z, ZIP, TAR, or TAR.GZ BagIt archive is verified and downloaded to a folder of project folders. ACORN completes BagIt verification before publishing anything, then writes only the contents of the bag's `data/` directory, preserving the project-directory structure beneath it. A `bagit.zip` containing `data/project-a/index.json` becomes `<output>/project-a/index.json`. BagIt tag files such as `bagit.txt`, `bag-info.txt`, and payload manifests are never copied to the output.

The bag is resolved at the archive root or beneath one enclosing directory. Filters, ignore rules, `--flatten`, `--clobber`, `--canonicalize`, `--strict`, provenance, ingestion, counts, and activity logging all apply to the verified payload paths, exactly as they do for bucket repositories.

Local BagIt archives work offline. A remote BagIt archive requires online mode and is rejected before any network access under `--offline`. An incomplete or incorrect bag fails without publishing payload, and downloaded archives, extraction staging, and unpublished output are cleaned up on both success and failure.

The archive format is inferred from the source name and, for local files, from the content header. Pass `--archive-format` to force 7z, `tar`, `tar.gz`, or `zip` for extensionless or misleading names. The flag applies only to bucket downloads and is rejected by `download model` and `download needle`.

```sh
# Download a verified local BagIt archive
acorn download ./bag.zip --output ./content

# Download a BagIt archive over HTTPS
acorn download https://example.org/bags/nssd.zip --output ./content

# Download a local BagIt archive without network access
acorn download ./bag.7z --offline --output ./content

# Force the archive format for a source name inference cannot classify
acorn download ./release-bundle --archive-format zip --output ./content
```

### GitLab and GitHub
The `download` command supports both GitLab and GitHub remote repositories for ACORN buckets. The configuration for each is similar, with the main difference being the `provider` field in the `repository` object.
```json
"buckets": [
    {
        "name": "ccsd (gitlab)",
        "repository": {
            "provider": "gitlab",
            "id": 17410,
            "uri": "https://code.ornl.gov/research-enablement/buckets/ccsd"
        }
    },
    {
        "name": "test (github)",
        "repository": {
            "provider": "github",
            "uri": "https://github.com/jhwohlgemuth/bucket"
        }
    }
]
```

## Models

Download model weights for use with ACORN research harnesses and local inference. By default, ACORN selects GGUF model files from Hugging Face repositories and prefers the Q4_K_M quantization. Python, pip, or the `hf` tool are not required.

For complete metadata-import options, configuration examples, authentication, and database behavior, see [Import model](import.md#model-metadata).

```sh
# Import GGUF file metadata without downloading weights
acorn import model openai/gpt-oss-20b --search-limit 20

# Import metadata for repositories listed in a local or remote document
acorn import model --model-file https://example.org/models.json

# Download models listed in a local or remote document
acorn download model --model-file https://example.org/models.json

# Select an exact quantization that fits in available GPU memory
acorn download model openai/gpt-oss-20b --quantization Q4_K_M --gpu-memory 24GB

# Download default GGUF (Q4_K_M) from a Hugging Face repo
acorn download model meta-llama/Llama-3.1-8B

# Download model weights into a specific local directory
acorn download model meta-llama/Llama-3.1-8B --local-dir ./models

# Download a specific quantization
acorn download model meta-llama/Llama-3.1-8B --quantization Q8_0

# Exclude low-quality quantizations
acorn download model meta-llama/Llama-3.1-8B --ignore "Q2_|Q3_|imatrix"

# Download models from a config file
acorn download model --config .acorn.json
```

Use `--model-file` to load model IDs from a local path, `file://` URI, or HTTP(S) URL. Plain-text lists and JSON/YAML lists of IDs or model details are accepted. Positional models, entries from `--model-file`, and supported `--config` entries are combined and deduplicated before download. Remote model-list documents cannot be loaded with `--offline`.

Pass `--sync` to add the unique model identifiers resolved by this invocation to the ACORN configuration's `models` list and to both OpenCode and llama-swap configuration. Existing ACORN model entries are preserved and duplicate identifiers are omitted. Use `--sync opencode` or `--sync llama-swap` to select one inference target:

```sh
acorn download model openai/gpt-oss-20b --sync
acorn download model openai/gpt-oss-20b --sync opencode
acorn download model openai/gpt-oss-20b --sync llama-swap
```

`--dry-run` continues to avoid model downloads and prints ACORN and target-configuration diffs without writing them when combined with `--sync`. Only models already available in the selected output/models directory can appear in the inference-configuration diffs by default. Add `--force` to skip this existence check and assume each model is located at `<models-dir>/<model-id>`:

```sh
acorn download model openai/gpt-oss-20b --sync --dry-run --force
```

Inline `--sync` is additive and operates only on models resolved by the current download. The standalone `acorn sync` command remains the full reconciliation operation for every model in ACORN configuration, including target-path and models-directory overrides, independent dry runs, and removal of stale ACORN-managed entries with `--prune`.

Metadata import resolves the configured revision, records each GGUF file URL, quantization, and byte size, and inspects a selected fallback GGUF repository when the base repository is unavailable or has no GGUF files. It does not transfer model weights or checksum sidecars. `--quantization` is an ordered, comma-separated exact allowlist. When only `--gpu-memory` is supplied, ACORN evaluates Q4_K_M. Split shards are treated as one variant and their sizes are summed.

Detailed configuration entries can set the same constraints:

```json
{
    "models": [{
        "name": "gpt-oss",
        "source": {
            "provider": "huggingface",
            "location": "https://huggingface.co/openai/gpt-oss-20b"
        },
        "quantization": ["Q5_K_M", "Q4_K_M"],
        "gpuMemory": "24GB"
    }]
}
```

### OCI model artifacts

Explicit `oci://` references download model artifacts from any OCI Distribution-compatible registry, including Harbor. [ORAS 1.3.0 or newer](https://oras.land/docs/installation) must be installed and available on `PATH`; `acorn doctor --check software` reports its availability. Unqualified `owner/repository` values remain Hugging Face selectors.

```sh
# Resolve a tag and preview its immutable digest and selected files
acorn download model oci://savannah.ornl.gov/models/gpt-oss:20b --dry-run

# Pull an immutable artifact digest
acorn download model oci://registry.example.org/ai/models/qwen@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
```

Tags are resolved during planning. Dry-run and raw output include the requested reference, resolved SHA-256 manifest digest, detected package format, selected and ignored transfer layers, known logical files, transfer size, and inventory confidence. Archive inventories remain explicitly deferred when they cannot be known without downloading the archive. Downloads pin the resolved digest, verify the config and every selected blob against its descriptor, stage into a unique sibling directory, validate every materialized path, and publish with one rename. A non-empty existing destination is rejected. `--skip-verify-checksum` does not disable OCI descriptor verification.

OCI model downloads have a GGUF-only inference boundary. ACORN accepts annotated artifact layers with safe `.gguf` `org.opencontainers.image.title` paths, KitOps ModelKits whose declared model is GGUF, and current CNCF ModelPacks. Native ModelKit configuration is decoded through ACORN's portable KitOps v1.15.0 Kitfile schema, including package, model parts, datasets, code, documentation, prompts, MCP Bundles, parameters, and resolved layer identity. The model command selects the primary GGUF and GGUF model parts by their resolved digests; valid unrelated package components remain metadata and are not materialized implicitly. Older configs without layer identity are accepted only when each selected path maps unambiguously to one supported manifest layer.

ModelPack recognizes weight, weight-configuration, documentation, code, and dataset layers in raw, TAR, TAR+gzip, and TAR+Zstandard forms. The model command installs selected GGUF weights and their weight-configuration layers while reporting documentation, code, and datasets as ignored package components. Raw selected layers require a safe `org.cncf.model.filepath` or an unambiguous file-metadata name; archive inventories are filtered and validated after bounded extraction.

Every downloaded GGUF is checked for its magic, supported container version, bounded metadata, tensor declarations, data bounds, and consistent split metadata. A selection must contain exactly one complete primary model group; related projector GGUFs are allowed, while projector- or adapter-only artifacts fail. SafeTensors, ONNX, PyTorch, extensionless application-store blobs, ordinary runnable container images, unsafe paths, symbolic links, malformed digests, and unknown ModelPack media types are rejected. Valid non-runtime ModelPack roles are ignored explicitly rather than misclassified. A one-candidate OCI index resolves to its child manifest digest; ambiguous, nested, or mixed runnable/model indexes fail closed. Digest integrity proves content identity, not publisher authenticity.

#### Custom Harbor endpoints

Use an explicit `oci://` reference for a model in any Harbor registry. A named registry profile is optional for public artifacts, but enables Harbor metadata and registry-specific authentication or TLS settings. ORNL's Savannah Harbor instance is one example:

```sh
# Inspect the tag, resolved digest, GGUF files, and transfer layers
acorn download model oci://savannah.ornl.gov/models/qwen:Q4_K_M --dry-run --raw

# Download into ACORN's digest-addressed model directory
acorn download model oci://savannah.ornl.gov/models/qwen:Q4_K_M --local-dir ./models
```

ACORN keeps the verified artifact in a registry/repository/digest-qualified directory so mutable tags cannot overwrite another resolution; the directory names are portable to Windows. Application-specific model stores are outside this command's scope.

ORAS uses the normal Docker-compatible credential store by default. A named profile may instead identify a credential environment variable; ACORN passes that value through standard input and never places it in configuration, process arguments, or diagnostics. TLS verification stays enabled. Plain HTTP is allowed only by an explicit development profile.

```json
{
    "registries": {
        "savannah": {
            "kind": "harbor",
            "endpoint": "https://savannah.ornl.gov",
            "credentialEnv": "ACORN_SAVANNAH_TOKEN"
        }
    },
    "models": [
        {
            "name": "gpt-oss-20b",
            "source": {
                "provider": "oci",
                "location": "oci://savannah.ornl.gov/models/gpt-oss:20b",
                "registry": "savannah"
            },
            "auth": "required"
        },
        {
            "name": "qwen-huggingface",
            "source": {
                "provider": "huggingface",
                "location": "Qwen/Qwen2.5-Coder-7B-Instruct-GGUF"
            }
        },
        "oci://ghcr.io/example/models/qwen:Q4_K_M"
    ]
}
```

The second entry continues to use the pre-OCI Hugging Face API. The final entry shows the simplified selector syntax for an OCI artifact that does not need a named registry profile.

Profiles also support `username`, `registryConfig`, `caFile`, paired `clientCert`/`clientKey`, and `plainHttp`. They intentionally have no password or token field. Harbor profiles enable its v2 artifact metadata API; content transfer always remains portable through ORAS. Global `--offline` rejects OCI access before ORAS or Harbor is contacted.

Command-line constraints override configuration. If neither constraint is present, download selection remains unchanged. With global `--no-local-database`, import reports metadata without persisting it and constrained downloads proceed with an eligibility-unknown warning.

### Filter and ignore semantics

The `--filter` flag behaves as an **include** rule: only sources matching at least one pattern will be downloaded. The `--ignore` flag behaves as an **exclude** rule: sources matching any ignore pattern are excluded even if they match a filter pattern. Both flags accept regular expression patterns, not Hugging Face glob patterns. Simple patterns are automatically optimized to glob matching for Hugging Face repositories.

### Note for Transformers and PyTorch users

Users of Transformers or PyTorch typically need the full repository or a broader filter that includes `config.json`, tokenizer files, `tokenizer_config.json`, and optionally custom code files. The default GGUF-only filter is designed for llama.cpp inference.

## Whitelists

A model whitelist restricts downloads to matching user-facing model names. ACORN uses the first non-empty whitelist source in this order: `--whitelist` or `--whitelist-file`, application configuration, then `ACORN_MODEL_WHITELIST`. The environment variable accepts comma-separated inline entries or one remote HTTP or HTTPS whitelist URI. An empty whitelist does not restrict downloads.

Use `--whitelist-file` to load a whitelist from a remote HTTP or HTTPS URI. For example, the following command allows the requested model only when it appears in the ORNL Research model catalog:

```sh
acorn download model openai/gpt-oss-20b --whitelist-file https://research.ornl.gov/api/models.json
```

The `--whitelist` option accepts inline model names; it does not fetch a URI supplied as its value.

Configure an inline whitelist with `whitelist.models`:

```json
{
    "models": [
        "openai/gpt-oss-20b",
        "meta-llama/Llama-3.1-8B",
        "https://huggingface.co/Qwen/Qwen3-8B"
    ],
    "whitelist": {
        "models": [
            "openai/gpt-oss-20b",
            "https://huggingface.co/Qwen/Qwen3-8B"
        ]
    }
}
```

The `models` value in the `whitelist` object accepts either one string or an array of strings:

- One HTTP or HTTPS URL is treated as a remote whitelist document and fetched when the command runs.
- An array is treated as inline whitelist entries. Each string may be a model name, model ID, or model URL. URLs inside an array are entries and are not fetched as nested whitelist documents.
- One non-URL string is treated as one inline whitelist entry.

For example, this configuration loads the whitelist from a remote document:

```json
{
    "models": [
        "openai/gpt-oss-20b",
        "meta-llama/Llama-3.1-8B"
    ],
    "whitelist": {
        "models": "https://example.org/acorn/models.json"
    }
}
```

The remote document may contain one model-details JSON object with a `name` or `id`:

```json
{
    "name": "openai/gpt-oss-20b"
}
```

It may instead contain a JSON array of model names, IDs, or URLs:

```json
[
    "openai/gpt-oss-20b",
    "meta-llama/Llama-3.1-8B",
    "https://huggingface.co/Qwen/Qwen3-8B"
]
```

An array of model-details objects is also accepted:

```json
[
    {
        "name": "gpt-oss",
        "id": "openai/gpt-oss-20b"
    },
    {
        "id": "meta-llama/Llama-3.1-8B"
    }
]
```

For model-details objects, ACORN accepts both `name` and `id` as whitelist entries when both are present. At least one of these fields is required. YAML arrays and newline-separated plain-text entries are also supported.

Remote whitelist documents cannot be loaded with `--offline`. Use `--whitelist-file <URI_OR_PATH>` when the whitelist document is a local path or `file://` URI. An explicitly supplied configuration path must exist; otherwise, ACORN reports `Configuration file does not exist`.

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Follow downloaded weights through the <a href="../model-workflows.md">model workflow</a>.</p>
