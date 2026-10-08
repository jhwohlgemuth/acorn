# 🧠 Infer

`acorn infer` provides two bounded inference modes. A prompt argument or piped text uses the managed [Needle sidecar](../../inference/needle.md) to select and invoke ACORN tools. The `run` subcommand hydrates one of ACORN's embedded prompt assets and optionally sends it to ACP or an OpenAI-compatible endpoint. `ask` is an alias for `infer`.

```sh
acorn infer "Validate this research activity"
echo "Validate this research activity" | acorn infer
acorn infer run summarize input.json --dry-run
acorn infer run summarize inputs/ --dry-run
acorn infer run summarize 'notes/*.json' --dry-run
acorn infer run summarize --data '{"text":"Inline JSON"}' --dry-run
acorn infer run summarize input.json --data '{"text":"Additional context"}' --dry-run
printf '%s' '{"text":"Additional context"}' | acorn infer run summarize input.json --data --dry-run
acorn infer run summarize --data --paste --dry-run
acorn infer run translate input.yaml --set language=French
cat input.zon | acorn infer run teach -
acorn infer run summarize input.json --agent codex
acorn infer run summarize --backend openai --model qwen3 --base-url http://localhost:8080
```

## Embedded prompts

Known assets are `eli5`, `extract-claim`, `find-gaps`, `suggest-fixes`, `summarize`, `teach`, and `translate`. Each resolved input file must contain a JSON, YAML, or ZON object. `infer run` merges those objects and supplies the result to the template as `data`.

### Prompt variables

The built-in templates read these data variables:

| Prompt | Variable | Type and default | Description |
| --- | --- | --- | --- |
| `eli5` | `text` | String, defaults to `""` | Material to explain in simple terms. |
| `extract-claim` | `text` | String, defaults to `""` | Material from which to extract explicit claims and supporting evidence. |
| `find-gaps` | `text` | String, defaults to `""` | Material to inspect for omissions and unanswered questions. |
| `suggest-fixes` | `version` | String, defaults to `"1"` | Version associated with the findings. |
| `suggest-fixes` | `findings` | Array, defaults to `[]` | Structured findings for which the prompt should suggest fixes. |
| `summarize` | `text` | String, defaults to `""` | Material to summarize. |
| `teach` | `text` | String, defaults to `""` | Material to turn into an instructional explanation. |
| `translate` | `language` | Required string | Target language. Supply it in an input object, `--data`, or `--set language=French`. |
| `translate` | `text` | String, defaults to `""` | Material to translate. |

Additional top-level fields are accepted and remain visible in the complete data block appended to every hydrated prompt, even when the template does not interpolate them directly. The names `config` and `data` are reserved and cannot be supplied as top-level data fields.

The templates also use internal configuration values. `infer run` currently uses their embedded defaults; `--data` and `--set` modify prompt data, not configuration. The prompt-specific settings are:

- `eli5`: `max_words` is 150 and `include_analogy` is `true`.
- `extract-claim`: `max_items` is 5 and `min_confidence` is 0.5.
- `find-gaps`: no prompt-specific settings.
- `suggest-fixes`: no prompt-specific settings.
- `summarize`: `max_items` is 5 and `max_words` is 160.
- `teach`: `include_examples` and `include_practice` are both `true`.
- `translate`: no prompt-specific settings.

All templates also carry configuration fields such as `version`, `temperature`, and `max_tokens`. Most carry `top_k` and `stop_sequences` as well. These are not accepted as `infer run` data variables.

Files, directories, and glob patterns may be combined for collective analysis. Directories recursively select JSON, YAML, YML, ZON, and ZONF files as if passed as `DIRECTORY/**/*`; matches are sorted, strings are joined with blank lines, arrays are concatenated, and conflicting scalar values are rejected.

Use `--data JSON` for an inline JSON object. Bare `--data` reads JSON from piped standard input; `--data --paste` reads it from the system clipboard instead. Piped standard input and `--paste` are mutually exclusive. Positional sources are loaded first, `--data` is merged after them, and repeated `--set KEY=VALUE` options are applied last. Bare piped standard input without `--data` retains automatic JSON, ZON, then YAML decoding. JSON booleans, numbers, and null supplied through `--set` retain their types, while other input becomes a string. Arrays, objects, reserved keys, invalid keys, and type changes are rejected.

The prompt data block defaults to canonical ZON; use `--format json` for troubleshooting. `--dry-run` prints the hydrated prompt without reading credentials, checking an executable, starting a process, or making a network request. `--output json` emits a tagged preview or result envelope.

### Backends

Backend precedence is an explicit `--backend`, then `ACORN_INFERENCE_BACKEND`, then `acp`. ACORN does not fall back if the selected backend fails.

- ACP uses `opencode acp` by default. Select `--agent codex` to use Codex through an installed `codex-acp` adapter. `--agent-executable`, `--working-directory`, and optional `--model` customize either agent session; ACORN does not install adapters or fall back between them.
- OpenAI-compatible inference requires `--model`. `--base-url` overrides `OPENAI_SERVER_HOST`, `--endpoint-mode` selects `responses` or `chat-completions`, and `OPENAI_API_KEY` is the only credential source.

Both backends use a 30-second default timeout. ACP tool mutation stays denied by policy.

#### Codex through ACP

ACORN starts `codex-acp`, requests a read-only Codex session, and rejects tool mutation requests. It does not invoke the `codex` executable directly. Use the software diagnostic to check the Codex CLI and adapter separately, then select Codex for a run:

```sh
acorn doctor --check software
acorn infer run summarize input.json --agent codex
acorn infer run summarize input.json --agent codex --model MODEL
acorn infer run summarize input.json --agent codex --agent-executable /path/to/codex-acp
```

If `codex-acp` is missing, install it with `npm install -g @agentclientprotocol/codex-acp`. That global installation does not require `npx`. If npm and npx are unavailable, install Node.js/npm or place a standalone binary from the [`codex-acp` releases](https://github.com/agentclientprotocol/codex-acp/releases) on `PATH`.
