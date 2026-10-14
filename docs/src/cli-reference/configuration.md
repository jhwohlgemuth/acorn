# Configuration

## Flat file
Application Configuration for Research Enablement (ACRE) is the versioned configuration standard used by ACORN and reusable by other research-enablement applications such as Xylem. ACRE files may use JSON, JSONC (JSON with Comments), or YAML.

The current schema version is `2.0.0`. New files should declare it with the `schemaVersion` property. Files that omit the property are interpreted as version `2.0.0`. Other explicit versions, including a `v` prefix, are rejected. There is no backwards compatibility: version `1.0.0` documents are rejected.

Attribute names are `snake_case` in every format — JSON, JSONC, and YAML — so one document shape is valid everywhere. Pre-existing camelCase attributes such as `schemaVersion` keep their current spelling.

### Example
`.acorn.json` file
```json
{
    "schemaVersion": "2.0.0",
    "buckets": [
        {
            "name": "bessd",
            "source": {
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
            "source": {
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

Validate canonical filenames directly; ACORN detects ACRE from the exact filename:

```sh
acorn check .acorn.json
```

For custom filenames or standard input, select the standard explicitly:

```sh
acorn check settings.jsonc --standard acre
cat settings.yaml | acorn check --paste --format yaml --standard acre
```

Export the JSON Schema with `acorn export --schema --standard acre`.

### Bucket sources

Repository-backed buckets accept a bare path or HTTPS URL when no label is needed:

```json
{
    "buckets": [
        "./science",
        "https://github.com/example/research-activities",
        "https://gitlab.example/research/science"
    ]
}
```

Use a `source` string to add a name or description:

```yaml
buckets:
  - name: science
    description: Reviewed science activities
    source: https://gitlab.example/research/science
```

Provider-specific fields use the detailed `source` form. The legacy `repository` and `codeRepository` keys remain accepted as input aliases:

```json
{
    "buckets": [{
        "name": "science",
        "source": {
            "provider": "gitlab",
            "id": 42,
            "location": "https://gitlab.example/research/science"
        }
    }]
}
```

Do not specify more than one of `source`, `repository`, and `codeRepository` on the same bucket. Concise remote sources must be credential-free HTTPS URLs; use local paths or `file:` URIs for local buckets.

### MCP clients

The optional `mcp` map configures outbound MCP servers by stable alias. Every server requires a tagged transport and a non-empty tool allowlist:

```yaml
mcp:
  research-tools:
    transport:
      type: child-process
      command: research-mcp
      args: [serve, --stdio]
      environment:
        RESEARCH_TOKEN: RESEARCH_MCP_TOKEN
    allowed:
      resolve_identifier:
        effects: [network-read]
        max_output: 256KB
        timeout: 20
```

Child-process `environment` values and Streamable HTTP `bearer_token_env` values are environment-variable names, not credentials. Streamable HTTP requires HTTPS except on `localhost` or a loopback IP address:

```yaml
mcp:
  institutional-tools:
    transport:
      type: streamable-http
      url: https://mcp.example.org/service
      bearer_token_env: INSTITUTIONAL_MCP_TOKEN
    allowed:
      lookup_record:
        effects: [network-read]
```

Supported effects are `credentials`, `destructive`, `filesystem-read`, `filesystem-write`, `inference`, `mutation`, `network-read`, and `process`. Duplicate effects and zero time or output limits are rejected. Omitted tool limits default to 30 seconds and 1 MiB. See [`acorn mcp`](commands/mcp.md) for runtime policy and examples.

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
