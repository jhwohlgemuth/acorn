# ACRE

Application Configuration for Research Enablement (ACRE) is ACORN's versioned configuration format. It keeps application settings—such as data sources, model downloads, synchronization targets, runners, and deployment policy—separate from Research Activity Data (RAD). ACRE files can be reused by other research-enablement applications.

## Use an ACRE file

Create `.acorn.json`, `.acorn.jsonc`, `.acorn.yaml`, or `.acorn.yml` in the project directory. ACORN checks those names in that order. New files should declare schema version `2.0.0`:

```yaml
schemaVersion: 2.0.0
models:
  - Qwen/Qwen2.5-0.5B-Instruct-GGUF
whitelist:
  models:
    - Qwen/Qwen2.5-0.5B-Instruct-GGUF
```

Use `--config <FILE>` when a command should load another filename. Validate and format ACRE independently of RAD:

```sh
acorn check .acorn.yaml
acorn format .acorn.yaml --standard acre --dry-run
acorn export --schema --standard acre
```

Canonical filenames are detected automatically by `acorn check`. For a custom filename or piped input, add `--standard acre`.

## Attributes

Every top-level attribute is optional. Unknown attributes and unsupported explicit schema versions are rejected.

| Attribute | Description |
| --- | --- |
| `schemaVersion` | Schema version. Use `2.0.0`; omission resolves to `2.0.0`. |
| `buckets` | Research-activity collections to import or download, as source strings or detailed repository objects. |
| `config` | llama-swap, OpenCode, VS Code, and Goose targets updated by `acorn sync`. |
| `endpoints` | Named HTTP API endpoints with authentication settings and templated resources. |
| `models` | Model IDs, paths, URLs, or detailed download entries. |
| `folders` | Shared destination registry. Each entry has `id` and accepts `aliases`, `bucket`, `members`, `path`, and `variables`. |
| `members` | Global submitter allowlist applying to every configured folder. |
| `variables` | Document-scoped environment-variable references, including `members` and `custom`. |
| `registries` | Named OCI or Harbor profiles used by model sources; credentials remain in environment variables. |
| `runners` | CI/CD runner definitions, including repository scope, executor, tags, and local or remote container host. |
| `whitelist` | Optional bucket and model allowlists. An empty whitelist allows all items. |
| `workflow` | Deployment allowlist for repository workflow paths, formats, actions, outputs, and external effects. |
| `scriptManifest` | Deployment manifest filename or inline authorization policy for executable hooks. |
| `scripts` | Ordered Rhai script paths and their `check`, `format`, `link`, `enrich`, or `validate` hooks. |
| `options` | Command defaults, resolved from the CLI. `options.scripts` (`enabled`) governs both root `scripts` and root `scriptManifest`; `options.folders.root` sets the folder destination base. |

The [complete ACRE specification](https://code.ornl.gov/research-enablement/acorn/-/blob/main/ACRE_SPEC.md) describes every object field and accepted shorthand. See [CLI configuration](../cli-reference/configuration.md) for working examples, [Model workflows](../cli-reference/model-workflows.md) for downloads, and [Rhai exec](../packages/exec.md) for constrained script hooks.
