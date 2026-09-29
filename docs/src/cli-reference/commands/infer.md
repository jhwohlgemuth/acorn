# 🧠 Infer

`acorn infer` provides two bounded inference modes. A prompt argument or piped text uses the managed [Needle sidecar](../../inference/needle.md) to select and invoke ACORN tools. The `run` subcommand hydrates one of ACORN's embedded prompt assets and optionally sends it to ACP or an OpenAI-compatible endpoint. `ask` is an alias for `infer`.

```sh
acorn infer "Validate this research activity"
echo "Validate this research activity" | acorn infer
acorn infer run summarize input.json --dry-run
acorn infer run summarize 'notes/*.json' --dry-run
acorn infer run translate input.yaml --set language=French
cat input.zon | acorn infer run teach -
acorn infer run summarize --backend openai --model qwen3 --base-url http://localhost:8080
```

## Embedded prompts

Known assets are `eli5`, `extract-claim`, `find-gaps`, `suggest-fixes`, `summarize`, `teach`, and `translate`. Each positional input must be a JSON, YAML, or ZON object. Paths and glob patterns may be combined for collective analysis; matches are sorted, strings are joined with blank lines, arrays are concatenated, and conflicting scalar values are rejected. With piped standard input, ACORN attempts JSON, then ZON, then YAML. Repeated `--set KEY=VALUE` options replace top-level scalar values; JSON booleans, numbers, and null retain their types, while other input becomes a string. Arrays, objects, reserved keys, invalid keys, and type changes are rejected.

The prompt data block defaults to canonical ZON; use `--format json` for troubleshooting. `--dry-run` prints the hydrated prompt without reading credentials, checking an executable, starting a process, or making a network request. `--output json` emits a tagged preview or result envelope.

### Backends

Backend precedence is an explicit `--backend`, then `ACORN_INFERENCE_BACKEND`, then `acp`. ACORN does not fall back if the selected backend fails.

- ACP uses `opencode acp` by default. `--agent-executable`, `--working-directory`, and optional `--model` customize the session.
- OpenAI-compatible inference requires `--model`. `--base-url` overrides `OPENAI_SERVER_HOST`, `--endpoint-mode` selects `responses` or `chat-completions`, and `OPENAI_API_KEY` is the only credential source.

Both backends use a 30-second default timeout. ACP tool mutation stays denied by policy.
