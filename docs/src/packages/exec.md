# Rhai Exec

Constrained Rhai extensions and capability-brokered local workflow hooks for ACORN.

## ABI

- Header: `// @acorn-exec-v0.2.0` must be the first non-empty line and occur once.
- Entry: `fn evaluate(context)` with exactly one parameter.
- Engine: `Engine::new_raw()`, `eval`/imports disabled, bounded operations and wall time, fresh scope and capability broker per invocation.

## Result

```rhai
#{ findings: [ #{ rule_id: "my-rule", severity: "error", message: "msg", locator: "optional" } ] }
```

Extensions return the shared, validated analyzer `Finding` type in an `ExecutionResult<T>` envelope. The ABI version matches the ACORN package version. `check`, `validate`, and `link` hooks cannot return a document. A manifest-authorized `format` or `enrich` hook may return a structured document proposal:

```rhai
context.proposed.title = "Normalized title";
#{ findings: [], output: #{ document: context.proposed } }
```

The host deserializes this proposal into the workflow's concrete document type and runs its validators before applying anything.

## Complete example

This capability-free quality gate rejects research activities that are still marked as drafts. Save the script as `.acorn/draft-policy.rhai`:

```rhai
// @acorn-exec-v0.2.0
fn evaluate(context) {
    if context.input.meta.draft {
        #{ findings: [#{
            rule_id: "draft-project",
            severity: "warning",
            message: "Project metadata is still marked as draft",
            locator: "meta.draft"
        }] }
    } else {
        #{ findings: [] }
    }
}
```

Register it in `.acorn.json`:

```json
{
  "scripts": [
    { "id": "draft-policy", "path": "draft-policy", "hooks": ["check"] }
  ]
}
```

Run the configured check against a research activity:

```console
acorn check ./activity.json --config ./.acorn.json
```

An activity with `meta.draft` set to `true` produces the `draft-project` finding at `meta.draft`; setting it to `false` passes this gate. Because this script only inspects the structured input and returns findings, it needs no manifest or capabilities.

## Configuration

Top-level `scripts` in `.acorn.json`:

```json
{
  "scripts": [
    { "id": "project-quality", "path": "project-quality", "hooks": ["check", "format"] },
    { "path": "/opt/acorn/shared.rhai", "absolute": true, "hooks": ["check"] }
  ]
}
```

Relative paths anchored to `<project-root>/.acorn/`, `.rhai` suffix optional, no glob or parent traversal, exactly one file must exist.

Absolute paths require `absolute: true`, are local-only, and emit security warnings with canonical path and digest.

## Script manifests

`scriptManifest` accepts either a deployment-manifest path or a manifest object directly. A path is a safe relative name beneath the platform ACORN configuration directory's `manifests/` directory:

```json
{
  "scriptManifest": "research-hooks.json",
  "scripts": [
    { "id": "project-quality", "path": "project-quality", "hooks": ["check", "format"] }
  ]
}
```

The deployment file contains one digest-pinned authorization per script:

```json
{
  "version": "1.0.0",
  "scripts": [
    {
      "id": "project-quality",
      "version": "0.2.0",
      "entry": "evaluate",
      "digest": "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      "hooks": ["check", "format"],
      "capabilities": {
        "mutation": true,
        "read": {
          "roots": { "project": "/srv/research" },
          "maxBytes": 1048576,
          "maxFiles": 16
        },
        "write": {
          "roots": { "output": "/srv/research/generated" },
          "maxBytes": 1048576,
          "maxFiles": 16
        },
        "network": {
          "hosts": ["api.example.org"],
          "maxRequests": 2,
          "maxResponseBytes": 65536,
          "timeout": 5000
        }
      },
      "limits": {
        "maxCalls": 32,
        "maxOperations": 500000,
        "maxRuntime": 10000
      }
    }
  ]
}
```

The same object may be placed directly in application configuration:

```json
{
  "scriptManifest": {
    "version": "1.0.0",
    "scripts": [
      {
        "id": "project-quality",
        "version": "0.2.0",
        "digest": "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "hooks": ["check"],
        "capabilities": {}
      }
    ]
  },
  "scripts": [
    { "id": "project-quality", "path": "project-quality", "hooks": ["check"] }
  ]
}
```

An inline manifest that grants any effect is trusted only when the application configuration was explicitly selected with `--config`. Configuration discovery can still enable capability-free quality gates, but cannot establish effectful trust. Deployment paths are canonicalized beneath the manifest directory and group/world-writable manifests are rejected on Unix.

## Capabilities

Only declared host functions are registered:

- `exists(root, path)` and `read_text(root, path)` use named read roots.
- `write_text(root, path, content)` creates a host-owned file proposal beneath a named write root; it never writes during script evaluation and is unavailable to `check` and `validate` hooks.
- `http_get(url)` accepts exact allowlisted HTTPS authorities. Offline mode, credentials, fragments, redirects, private/local/link-local destinations, oversized responses, excessive requests, and timeouts are rejected.

Paths must be relative and traversal-free. Root handles are opened for each invocation, destination ancestors and targets cannot be symlinks, and aggregate file, byte, call, operation, request, and runtime limits cannot exceed host ceilings. Process execution, credentials, and inference are not grantable.

Build with the `exec` feature and pass configuration explicitly when needed:

```console
acorn check ./metadata --config ./.acorn.json
acorn format ./metadata --config ./.acorn.json
acorn format ./metadata --enrich --config ./.acorn.json
acorn link ./metadata --config ./.acorn.json
```

Configured execution is local-CLI only. TUI, repository snapshots, JSON-RPC, MCP, webhooks, and GitLab automation cannot run project scripts. Absolute scripts are an explicitly warned, non-portable local trust exception.

Every matching script is opened into one bounded snapshot, identified by digest, and compiled before the first evaluation. The context contains `hook`, `path`, `index`, `input`, and `proposed`; `input` and `proposed` are independent structured copies without host handles. Error findings or execution failures prevent planned writes, while lower-severity findings use analyzer rendering.

`check` gates join native findings. A `validate` hook runs as part of `acorn check` when schema checks are enabled and is omitted by `--skip schema`. `format` and optional `enrich` gates run after the complete proposal is prepared. `link` gates evaluate every planned companion artifact before any artifact is written.

Hook order, document order, and configured script order are deterministic. All source, linked-document, and script-proposed file changes are staged and synced before commit; a later commit failure restores earlier files. Audit records contain script ID and digest, hook, input identity, manifest provenance, granted and used effects, duration, and proposal digests without file or network contents.

## Inference

`MimeType::infer` checks binary signatures first, then performs a bounded Rhai probe for UTF-8 source with the ABI marker and one `evaluate(context)` entry point. Path-based inference continues to recognize an explicit `.rhai` extension. Inference never executes source.

## Help

Run `acorn exec --help` and `acorn exec <COMMAND> --help` for current usage information.

```console
acorn exec manifest check ./research-hooks.json
acorn exec hook check ./project-quality.rhai --manifest ./research-hooks.json --hook format --input ./activity.json
```

The hook check command evaluates and reports proposals without applying them. A manifest argument that is not an existing file is resolved as a deployment-manifest name.
