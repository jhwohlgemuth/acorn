# Configuration

## Flat-file
ACORN can be configured using JSON or YAML format files. These configuration files allow users to specify various options and settings for the ACORN CLI tool, including input and output directories, logging levels, and other parameters.

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
> [!TIP]
> `.acorn.json` is the default configuration file name that ACORN looks for in the current working directory. You can also specify a different configuration file using the `--config <FILE>` flag when running ACORN commands.

## .env file and Environment Variables
The ACORN CLI tool can also be configured using `.env` files and/or environment variables.

### Example
`.env` file
```env
ACORN_LOG_LEVEL=info
READABILITY_METRIC=ari
MAX_ALLOWED_ARI=12
```