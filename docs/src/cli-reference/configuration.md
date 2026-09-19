# Configuration

## Flat file
ACORN can be configured using JSON, JSONC (JSON with Comments), or YAML format flat files. These configuration files allow users to specify options and settings for the ACORN CLI tool, including input and output directories and logging levels.

### Example
`.acorn.json` file
```json
{
    "buckets": [
        {
            "name": "bessd",
            "repository": {
                "provider": "gitlab",
                "id": 17603,
                "uri": "https://code.ornl.gov/research-enablement/buckets/bessd"
            }
        }
    ],
    "models": [
        {
            "name": "qwen-0.5b",
            "source": {
                "provider": "huggingface",
                "location": "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF"
            },
            "quantization": [
                "Q4_K_M"
            ],
            "gpuMemory": "4GB"
        }
    ],
    "runners": [
        {
            "name": "oklahoma-runner",
            "description": "Oklahoma Docker Runner",
            "repository": {
                "provider": "gitlab",
                "id": 24758,
                "uri": "https://code.ornl.gov/research-enablement"
            },
            "type": "group",
            "executor": "docker",
            "gpuEnabled": false,
            "tags": ["docker", "oklahoma"],
            "runUntagged": false,
            "host": "code.ornl.gov",
            "target": {
                "host": "oklahoma"
            },
            "dockerImage": "gitlab/gitlab-runner:latest"
        }
    ]
}
```

The runner example includes every user-configurable runner field. The top-level runner `host` is the GitLab domain override, while `target.host` is an SSH URI or alias for the machine running Docker. The shorter `"target": "oklahoma"` form is equivalent to `"target": {"host": "oklahoma"}`. Bare aliases use the local SSH client configuration and are normalized to `ssh://oklahoma`. Omit `target` to run the container locally.

Set `GITLAB_TOKEN`, `PRIVATE_TOKEN`, or `CI_JOB_TOKEN`, then create every configured runner with:

```sh
acorn create runner
```

Runner containers always start detached. Command-line `--host` or `--context` overrides `target.host` for every runner in that invocation.

`.acorn.jsonc` file
```jsonc
{
    // ACORN bucket configuration
    "buckets": [
        {
            "name": "bessd",
            "repository": {
                "provider": "gitlab",
                "id": 17603,
                "uri": "https://code.ornl.gov/research-enablement/buckets/bessd"
            }
        },
    ]
}
```

> [!TIP]
> `.jsonc` files permit `//` line comments, `/* block */` comments, and trailing commas. All other syntax follows standard JSON rules. Rename a `.json` file to `.jsonc` if you need comments.
> [!TIP]
> ACORN looks for `.acorn.json`, `.acorn.jsonc`, `.acorn.yaml`, then `.acorn.yml` in the current working directory. You can also specify a different configuration file using the `--config <FILE>` flag when running ACORN commands.

## .env file and Environment Variables
The ACORN CLI tool can also be configured using `.env` files and/or environment variables.

Enrichment providers use optional API tokens and host overrides. See [Metadata enrichment](../concepts/enrichment.md#providers) for the complete provider-by-provider table and defaults.

### Example
`.env` file
```env
ACORN_LOG_LEVEL=info
READABILITY_METRIC=ari
MAX_ALLOWED_ARI=12
```

`ACORN_CITATION_FORMAT` selects the DOI citation family used by `acorn gather --raw --resolve` when `--citation-format` is not provided. Accepted case-insensitive values are `apa`, `chicago`, `harvard`, `ieee`, `mla`, and `vancouver`; IEEE is the default. The precedence is CLI option, environment variable, then IEEE. An unsupported environment value emits a warning and falls back to IEEE.

```powershell
$env:ACORN_CITATION_FORMAT = "apa"
acorn gather report.pdf --raw --resolve
```
