# Rhai Exec

Constrained Rhai extensions for ACORN.

## ABI

- Header: `// @acorn-exec-v0.2.0` must be the first non-empty line and occur once.
- Entry: `fn evaluate(context)` with exactly one parameter.
- Engine: `Engine::new_raw()` with no registered host capabilities, `eval`/imports disabled, limits enforced, fresh scope per eval.

## Result

```rhai
#{ findings: [ #{ rule_id: "my-rule", severity: "error", message: "msg", locator: "optional" } ] }
```

Extensions return the shared, validated analyzer `Finding` type in an `ExecutionResult<T>` envelope. The ABI version matches the ACORN package version.

## Configuration

Top-level `scripts` in `.acorn.json`:

```json
{
  "scripts": [
    { "path": "project-quality", "hooks": ["check", "format"] },
    { "path": "/opt/acorn/shared.rhai", "absolute": true, "hooks": ["check"] }
  ]
}
```

Relative paths anchored to `<project-root>/.acorn/`, `.rhai` suffix optional, no glob or parent traversal, exactly one file must exist.

Absolute paths require `absolute: true`, are local-only, and emit security warnings with canonical path and digest.

Build with the `exec` feature and pass configuration explicitly when needed:

```console
acorn check ./metadata --config ./.acorn.json
acorn format ./metadata --config ./.acorn.json
acorn format ./metadata --enrich --config ./.acorn.json
acorn link ./metadata --config ./.acorn.json
```

Configured execution is local-CLI only. TUI, repository snapshots, JSON-RPC, MCP, webhooks, and GitLab automation cannot run project scripts. Absolute scripts are an explicitly warned, non-portable local trust exception.

Every matching script is opened into one bounded snapshot, identified by digest, and compiled before the first evaluation. The context contains `hook`, `path`, `index`, `input`, and `proposed`; input and proposal values are serialized copies without host handles. Error findings or execution failures prevent planned writes, while lower-severity findings use analyzer rendering.

`check` gates join native findings. A `validate` hook runs as part of `acorn check` when schema checks are enabled and is omitted by `--skip schema`. `format` and optional `enrich` gates run after the complete proposal is prepared. `link` gates evaluate every planned companion artifact before any artifact is written.

## Inference

`MimeType::infer` checks binary signatures first, then performs a bounded Rhai probe for UTF-8 source with the ABI marker and one `evaluate(context)` entry point. Path-based inference continues to recognize an explicit `.rhai` extension. Inference never executes source.

## Help

Run `acorn exec --help` and `acorn exec <COMMAND> --help` for current usage information.
