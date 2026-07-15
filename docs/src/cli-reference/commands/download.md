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

## Download Model Weights

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

## Local vs Remote
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

## GitLab vs GitHub
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