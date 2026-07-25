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
        },
        {
            "name": "ccsd",
            "repository": {
                "provider": "gitlab",
                "id": 17602,
                "uri": "https://code.ornl.gov/research-enablement/buckets/ccsd"
            }
        },
        {
            "name": "nssd",
            "repository": {
                "provider": "gitlab",
                "id": 17410,
                "uri": "https://code.ornl.gov/research-enablement/buckets/nssd"
            }
        }
    ]
}
```

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
> ACORN looks for `.acorn.json`, `.acorn.jsonc`, `.acorn.yaml`, `.acorn.yml`, then `.acorn` in the current working directory. The extensionless `.acorn` form uses JSONC. You can also specify a different configuration file using the `--config <FILE>` flag when running ACORN commands.

## .env file and Environment Variables
The ACORN CLI tool can also be configured using `.env` files and/or environment variables.

### Example
`.env` file
```env
ACORN_LOG_LEVEL=info
READABILITY_METRIC=ari
MAX_ALLOWED_ARI=12
```
