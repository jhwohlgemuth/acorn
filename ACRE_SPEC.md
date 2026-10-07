# Application Configuration for Research Enablement (ACRE)

ACRE is ACORN's versioned configuration format for research-enablement applications. It describes reusable application inputs such as research-activity buckets, model sources, synchronization targets, service endpoints, runners, deployment policy, and constrained script hooks. ACRE configures the application; it does not contain Research Activity Data (RAD).

This document describes ACRE schema version `1.0.0`. The generated JSON Schema is the machine-readable authority for accepted types, unions, enum values, and validation constraints:

```sh
acorn export --schema --standard acre
```

## Files and discovery

ACRE supports JSON, JSON with Comments (JSONC), and YAML. ACORN discovers the first canonical file in the current directory in this order:

1. `.acorn.json`
2. `.acorn.jsonc`
3. `.acorn.yaml`
4. `.acorn.yml`

Use `--config <FILE>` with commands that consume a non-canonical filename. JSONC permits line comments, block comments, and trailing commas. Property names are case-sensitive.

An ACRE document is a single object. Every top-level attribute is optional, and omitted attributes have no effect. New files should set `schemaVersion` explicitly; an omitted version is treated as `1.0.0` for compatibility. Unknown top-level attributes and unsupported explicit schema versions are rejected.

## Example

```json
{
  "schemaVersion": "1.0.0",
  "buckets": [
    {
      "name": "science",
      "description": "Reviewed research activities",
      "codeRepository": {
        "provider": "gitlab",
        "id": 42,
        "location": "https://gitlab.example/research/science"
      }
    }
  ],
  "models": [
    "Qwen/Qwen2.5-0.5B-Instruct-GGUF"
  ],
  "config": {
    "opencode": {
      "path": "./opencode.jsonc"
    }
  },
  "whitelist": {
    "buckets": ["science"],
    "models": ["Qwen/Qwen2.5-0.5B-Instruct-GGUF"]
  }
}
```

Validate, preview formatting, and export the exact schema with:

```sh
acorn check .acorn.json
acorn format .acorn.json --standard acre --dry-run
acorn export --schema --standard acre
```

Use `--standard acre` with `acorn check` when the input has a non-canonical filename or comes from standard input.

## Top-level attributes

| Attribute | Type | Description |
| --- | --- | --- |
| `schemaVersion` | string | ACRE schema version. The only supported explicit value is `1.0.0`; omission also resolves to `1.0.0`. |
| `buckets` | array | Research-activity collections that ACORN can import or download. Entries may be concise source strings or detailed bucket objects. |
| `config` | object | Output targets for synchronizing downloaded models into llama-swap, OpenCode, VS Code, and Goose. |
| `endpoints` | array | Named HTTP API endpoints and templated resources available to ACORN integrations. |
| `models` | array | Models to download or synchronize, expressed as selector strings or detailed model entries. |
| `powerAutomate` | object | Deployment-owned PowerAutomate intake, project routing, and callback configuration. |
| `registries` | object | Map of profile names to credential-free OCI or Harbor registry settings referenced by model sources. |
| `runners` | array | CI/CD runners that ACORN can create and start locally or on a remote container host. |
| `whitelist` | object | Optional allowlists that restrict selectable bucket and model identifiers. An empty whitelist imposes no restriction. |
| `workflow` | object | Deployment-owned allowlist of repository workflows, inputs, actions, outputs, and external effects. |
| `scriptManifest` | string or object | Safe deployment-manifest filename or inline authorization manifest for executable hooks. |
| `scripts` | array | Ordered Rhai script registrations and the workflow hooks to which they subscribe. |

## Attribute definitions

### `schemaVersion`

`schemaVersion` is a string containing `1.0.0`. Values such as `v1.0.0`, numeric versions, and unknown future versions are invalid. Producers should always emit the attribute even though consumers accept its omission for compatibility.

### `buckets`

Each bucket identifies a source of research-activity files. A concise entry is a local path, `file:` URI, or credential-free HTTPS repository URL:

```yaml
buckets:
  - ./activities
  - https://github.com/example/research-activities
```

A detailed bucket has these attributes:

| Attribute | Required | Description |
| --- | --- | --- |
| `name` | No | Human-readable bucket name. |
| `description` | No | Human-readable explanation of the collection. |
| `codeRepository` | Yes | Source string or repository object used to enumerate and retrieve items. `repository` and `source` are accepted input aliases. |

A repository object uses `provider` (`git`, `github`, `gitlab`, `huggingface`, or `oci`) and `location`. `location` may be a string or an object containing `scheme`, `uri`, and optional `revision`. GitLab sources may add numeric `id`; OCI sources may add a named `registry` profile.

### `config`

`config` selects application configuration files updated by `acorn sync`. It has four optional target attributes. Every target accepts an optional `path`; if omitted, ACORN uses that application's platform default.

| Attribute | Target-specific attributes |
| --- | --- |
| `goose` | `host` sets the OpenAI-compatible origin, `basePath` sets the chat-completions path, and `defaultModel` selects the active model. |
| `llamaSwap` | `modelsDirectory`, `executable`, `contextSize`, `ttl`, `extraArgs`, and `environment` set global defaults. `models` maps ACRE model names to overrides with `contextSize`, `ttl`, `aliases`, `extraArgs`, `environment`, and `executable`. |
| `opencode` | `baseUrl`, `providerId`, `providerName`, and `defaultModel` describe the managed OpenAI-compatible provider. |
| `vscode` | `mcpPath`, `url`, `providerName`, `apiKey`, `maxInputTokens`, and `maxOutputTokens` describe the custom chat-model endpoint and optional MCP output. |

`extraArgs` contains strings. `environment` contains `KEY=VALUE` strings. Token limits and context sizes must be positive; llama-swap `ttl` must be non-negative, where `0` means no expiry.

### `endpoints`

Each endpoint describes an API root and its callable resource templates.

| Attribute | Required | Description |
| --- | --- | --- |
| `domain` | Yes | API host or domain. |
| `name` | Yes | Stable name used for lookup and logging. |
| `scheme` | No | URI scheme override. |
| `port` | No | Network port. |
| `authentication` | No | Authentication object whose `scheme` selects the supported authentication mechanism. |
| `root` | No | Path prefix, such as `v3.0`. |
| `resources` | Yes | Resource definitions used to construct request paths. |

Each resource requires `name` and a Handlebars `template`; optional `method` defaults to `get`.

### `models`

Each model is either a selector string (model ID, local path, or URL) or a detailed object:

| Attribute | Required | Description |
| --- | --- | --- |
| `name` | Yes | User-facing model name used by download and synchronization commands. |
| `source` | Yes | Repository object containing a `provider` and `location`. |
| `revision` | No | Repository branch, tag, commit, or other revision. |
| `auth` | No | Authentication policy: `none`, `optional`, or `required`. |
| `filter` | No | Regular expressions that include matching files. |
| `ignore` | No | Regular expressions that exclude matching files. |
| `quantization` | No | One quantization or an ordered quantization allowlist for GGUF selection. |
| `gpuMemory` | No | Maximum memory available for model weights, such as `4GB`. |
| `copy` | No | Copy a local model into the model directory instead of referencing it in place. |
| `symlink` | No | Symlink a local model into the model directory instead of referencing it in place. |

`copy` and `symlink` are alternative local-materialization policies and must not both be enabled.

### `powerAutomate`

This deployment-owned object routes validated PowerAutomate form submissions. Its serialized nested names use snake case.

| Attribute | Required | Description |
| --- | --- | --- |
| `callback_destination` | Yes | Stable name persisted with callback intents. |
| `variables` | Yes | Names of the runtime environment variables `callback_url` and `inbound_secret`; values are variable names, not credentials. |
| `package` | Yes | GitLab generic-package name for generated artifacts. |
| `gitlab_project` | Yes | GitLab project containing bucket files and packages. |
| `projects` | Yes | Non-empty project routing list. Each item has `project_id`, `path`, and a `members` email allowlist. |
| `timezone` | No | Reporting timezone. Version `1.0.0` supports `UTC`. |

### `registries`

`registries` maps an arbitrary profile name to a registry object. Model sources refer to the name through their repository `registry` attribute.

| Attribute | Required | Description |
| --- | --- | --- |
| `kind` | No | `oci` (default) or `harbor`. |
| `endpoint` | Yes | Registry or API endpoint; HTTPS is expected unless `plainHttp` is enabled for development. |
| `credentialEnv` | No | Name of an environment variable containing a token or password. |
| `username` | No | Username paired with `credentialEnv`. |
| `registryConfig` | No | Docker-compatible registry configuration path. |
| `caFile` | No | Custom certificate-authority file. |
| `clientCert` | No | Client certificate file. |
| `clientKey` | No | Client private-key file. |
| `plainHttp` | No | Permit development-only plain HTTP; defaults to `false`. |

Secrets are supplied at runtime and must not be embedded in ACRE.

### `runners`

Each entry describes a CI/CD runner and where its container should run.

| Attribute | Required | Description |
| --- | --- | --- |
| `codeRepository` | Yes | Git hosting repository, group, or project that owns the runner. `repository` is an accepted input alias. |
| `type` | Yes | Runner scope: group, instance, or project. |
| `name` | No | Runner name shown by the hosting service. |
| `description` | No | Human-readable runner description. |
| `executor` | No | Runner executor, defaulting to Docker. |
| `gpuEnabled` | No | Whether the runner requires GPU support; defaults to `false`. |
| `tags` | No | Job-selection tags assigned to the runner. |
| `runUntagged` | No | Whether the runner accepts untagged jobs; defaults to `false`. |
| `host` | No | GitLab domain override for self-managed instances. |
| `target` | No | SSH URI, SSH alias, or `{ "host": "..." }` object for the machine running the container. Omission runs it locally. |
| `dockerImage` | No | Runner container image; defaults to `gitlab/gitlab-runner:latest`. |
| `identifier` | No | Runner identifier assigned by GitLab. |

### `whitelist`

| Attribute | Required | Description |
| --- | --- | --- |
| `buckets` | No | Allowed bucket names or identifiers. |
| `models` | No | One model ID/URL, an array of model IDs/URLs, or one URL to a JSON whitelist document. |

An absent or empty whitelist allows all items.

### `workflow`

`workflow` is deployment policy for repository automation. Its nested names use kebab case.

| Attribute | Required | Description |
| --- | --- | --- |
| `allow-forks` | No | Permit merge requests from forks to publish to the explicit destination; defaults to `false`. |
| `destination` | No | Explicit project receiving bot branches and merge requests. It is required when `allow-forks` is true. |
| `workflows` | Yes | Map of registered workflow names to policy objects. |

Each workflow policy requires:

| Attribute | Description |
| --- | --- |
| `actions` | Non-empty allowlist of `check`, `enrich`, `format`, `link`, and `validate` processing actions. |
| `effects` | Non-empty allowlist of external effects such as `branch`, `commit`, `commit-status`, `completion-comment`, `draft-merge-request`, `label`, and `note`. |
| `formats` | Non-empty allowlist of accepted input MIME types. |
| `outputs` | Object whose `mode` selects output behavior; version `1.0.0` supports `source-and-artifacts`. |
| `paths` | Non-empty list of safe repository-relative input glob patterns. |

### `scriptManifest`

`scriptManifest` is either a safe relative filename beneath ACORN's deployment manifest directory or an inline object with `version` and `scripts`. Each inline script authorization has:

| Attribute | Required | Description |
| --- | --- | --- |
| `version` | Yes | Rhai executable-extension ABI version. |
| `id` | Yes | Stable identifier matched by an entry in `scripts`. |
| `digest` | Yes | Expected SHA-256 identity of the script source. |
| `entry` | No | Required script function name; the host supplies its default when omitted. |
| `hooks` | Yes | Hooks the script is authorized to handle. |
| `capabilities` | No | Bounded `read`, `write`, `mutation`, and `network` permissions. |
| `limits` | No | `maxCalls`, `maxOperations`, and `maxRuntime` ceilings that may narrow host limits. |

Filesystem capabilities contain named absolute `roots` plus `maxBytes` and `maxFiles`. Network capability contains exact HTTPS `hosts` plus `maxRequests`, `maxResponseBytes`, and millisecond `timeout`. See [Rhai exec](docs/src/packages/exec.md) for trust and execution details.

### `scripts`

Each script registration has:

| Attribute | Required | Description |
| --- | --- | --- |
| `id` | No | Stable identifier used to match a manifest authorization. |
| `path` | Yes | Script path relative to `<project-root>/.acorn/`, unless `absolute` is true. |
| `hooks` | Yes | Non-empty, duplicate-free list containing `check`, `format`, `link`, `enrich`, or `validate`. |
| `absolute` | No | Interpret `path` as a platform-native absolute path; defaults to `false`. |

Script registrations select when code runs. `scriptManifest` independently defines what that code is authorized to do.

## Compatibility and conformance

Consumers conform to ACRE `1.0.0` when they accept the documented root object, reject unsupported explicit versions, and interpret supported attributes according to the exported schema. Producers conform when they emit `schemaVersion: "1.0.0"`, use canonical attribute names, and emit values accepted by that schema.

Before committing an ACRE file, run `acorn check` and use `acorn format --standard acre` to normalize supported JSON, JSONC, or YAML syntax. For operational examples, see the [ACORN configuration guide](docs/src/cli-reference/configuration.md).
