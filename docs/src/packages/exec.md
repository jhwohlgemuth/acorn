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
acorn format ./metadata --config ./.acorn.json
acorn format ./metadata --enrich --config ./.acorn.json
```

The format command prepares every proposed document change, resolves and statically compiles every matching script, then evaluates configured `format` and optional `enrich` gates in configuration order. Error findings or execution failures prevent all planned writes. Lower-severity findings are rendered through the analyzer output boundary.

## Inference

`MimeType::infer` checks binary signatures first, then performs a bounded Rhai probe for UTF-8 source with the ABI marker and one `evaluate(context)` entry point. Path-based inference continues to recognize an explicit `.rhai` extension. Inference never executes source.

## Help

Run `acorn exec --help` and `acorn exec <COMMAND> --help` for current usage information.
