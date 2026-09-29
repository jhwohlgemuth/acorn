# Local inference and tools

ACORN can publish a conservative tool catalog through two local interfaces:

- a managed [Needle sidecar](needle.md) for small, offline tool-selection loops; and
- `acorn serve mcp`, a standard-input/standard-output Model Context Protocol server for external agents.

Both interfaces are projections of the same registry and invoke the same handlers. A tool being described to a model does not grant extra authority: ACORN still enforces its exposure and execution policy at dispatch time.

For direct inference APIs, prompt instructions remain Markdown and the complete caller data object is appended once as canonical ZON. Library callers can explicitly choose pretty JSON for troubleshooting. This is model-facing text only: ACP still uses JSON-RPC and OpenAI-compatible endpoints still use JSON request envelopes.

The CLI exposes that boundary through [`acorn infer run`](../cli-reference/commands/infer.md). ACP/OpenCode is the default backend; `ACORN_INFERENCE_BACKEND` supplies a default selection and an explicit `--backend` wins over it. ACORN never falls back between backends. ACP permission requests remain governed by the fixed deny-mutation policy even when a session observer is attached.
