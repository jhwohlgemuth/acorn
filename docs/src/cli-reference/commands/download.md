# 📥 Download
> Obtain files from an ACORN bucket or download model weights for local inference

- See the [configuration](../configuration.md) documentation for details on configuring ACORN commands.
- By default, `download` will save files to `./content` in the working directory unless an output path is specified via the `--output` flag.

## Example Usage
```sh
# Download research activity data from a single bucket repository URL
acorn download https://code.ornl.gov/research-enablement/buckets/nssd

# Download research activity data from a list of buckets
acorn download --config /path/to/.acorn.json

# Download research activity data to a specific output directory
acorn download --config /path/to/.acorn.yml --output /path/to/output

# Download using JSONC configuration (supports comments and trailing commas)
acorn download --config /path/to/.acorn.jsonc
```

## Buckets

### Local vs Remote
The `download` command copies files from a *local* ACORN bucket when local `file://` URIs are used for associated buckets in the [configuration](../configuration.md) file. Use `"git"` as the `provider` for local buckets.

```json
"buckets": [
    {
        "name": "test (local)",
        "repository": {
            "provider": "git",
            "location": "file:./tests/fixtures/data/bucket/"
        }
    },
    {
        "name": "nssd (remote)",
        "repository": {
            "provider": "gitlab",
            "location": {
                "scheme": "https",
                "uri": "https://code.ornl.gov/research-enablement/buckets/nssd"
            }
        }
    }
]
```

### GitLab vs GitHub
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

```sh
# Download default GGUF (Q4_K_M) from a Hugging Face repo
acorn download model meta-llama/Llama-3.1-8B

# Download a specific quantization
acorn download model meta-llama/Llama-3.1-8B --filter "Q8_0.*\\.gguf$"

# Exclude low-quality quantizations
acorn download model meta-llama/Llama-3.1-8B --ignore "Q2_|Q3_|imatrix"

# Download models from a config file
acorn download model --config .acorn.json
```

### Filter and Ignore Semantics

The `--filter` flag behaves as an **include** rule: only sources matching at least one pattern will be downloaded. The `--ignore` flag behaves as an **exclude** rule: sources matching any ignore pattern are excluded even if they match a filter pattern. Both flags accept regular expression patterns, not Hugging Face glob patterns. Simple patterns are automatically optimized to glob matching for Hugging Face repositories.

### Note for Transformers/PyTorch Users

Users of Transformers or PyTorch typically need the full repository or a broader filter that includes `config.json`, tokenizer files, `tokenizer_config.json`, and optionally custom code files. The default GGUF-only filter is designed for llama.cpp inference.

## Whitelists

A model whitelist restricts downloads to matching user-facing model names. Whitelist entries from the application configuration are combined with values supplied through `--whitelist` and `--whitelist-file`. An empty whitelist does not restrict downloads.

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
