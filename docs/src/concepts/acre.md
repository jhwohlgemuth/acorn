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

## GitLab logbook inbox

Set `options.powerAutomate.intake` to enable JSONL polling when running `acorn serve bot --config .acorn.yaml`. Existing Power Automate callback, package, member, and protected-variable settings still apply:

```yaml
options:
  powerAutomate:
    callback_destination: logbook-flow
    gitlab_project: "42"
    package: automated-artifacts
    intake:
      branch: main
      path: .acorn/intake/logbook.jsonl
      poll_interval: 30
      max_file: 10MB
      max_record: 64KB
```

Each LF-terminated line uses the existing form envelope. Its `projectId` selects a registered folder ID or alias. The bot authorizes and durably queues each entry before removing its exact line. Preserve the UUID and serialized bytes across retries; changes under an accepted UUID are conflicts. Admission fixes the canonical destination so configuration changes cannot redirect retries.

Rejected records remain in the inbox with inspectable outcomes in the durable operation store. Cleanup preserves unrelated bytes and concurrent appends, keeps an empty file after draining, and retries conflicts from a fresh snapshot. GitLab intake proposes changes through the existing review workflow. Local destinations require a separate proposal/apply adapter.
