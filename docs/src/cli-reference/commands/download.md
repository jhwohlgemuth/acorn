# 📥 Download
> Obtain files from an ACORN bucket to your local filesystem

- See the [configuration](../configuration.md) documentation for details on configuring ACORN commands.
- By default, `download` will save files to `./content` in the current working directory unless an output path is specified via the `--output` flag.

## Example Usage
```sh
# Download research activity data from a single bucket repository URL
acorn download https://code.ornl.gov/research-enablement/buckets/nssd

# Download research activity data from a list of buckets
acorn download --config /path/to/.acorn.json

# Download research activity data to a specific output directory
acorn download --config /path/to/.acorn.yml --output /path/to/output
```

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