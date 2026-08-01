---
name: acorn
description: Operate and develop the ACORN CLI and research activity data (RAD) workflows. Use when creating, editing, validating, formatting, linking, gathering, downloading, importing, or exporting RAD and related metadata; working with RAD, RAiD, CFF, DOCX, or crosswalk schemas; diagnosing ACORN installations; downloading GGUF models; running ACORN from source; or verifying PDF, PowerPoint, BagIt, JSON, YAML, Markdown, and CFF artifacts.
---

# ACORN

ACORN validates, analyzes, transforms, and exports research activity metadata. Treat its generated schema and current CLI help as authoritative; do not invent fields, standards, flags, or output paths from memory.

When working in the ACORN repository, inspect `AGENTS.md` first and follow its required repository skills and verification rules.

## Workflow

```text
1. Identify the operation and whether it reads, mutates, downloads, or exports.
2. Confirm the executable, input selection, metadata standard, and network mode.
3. Inspect current help or generate the relevant schema when syntax or shape matters.
4. Preview mutating operations with --dry-run when supported.
5. Run the narrowest command that satisfies the request.
6. Validate the resulting data or artifact, not only the command exit status.
7. Report the exact command, affected paths, and verification result.
```

## Invocation

Use an installed binary:

```shell
acorn --version
acorn <command> --help
```

Run the checkout without installing:

```shell
cargo run --bin acorn -- <command> [arguments]
```

Place global flags before the subcommand:

```shell
acorn --offline check ./content --skip prose
cargo run --bin acorn -- -vv check ./content
```

Use `CommandLineHelp.md` in the repository for a complete generated command reference. Re-run `<invocation> --help` when the checkout may be newer than this skill.

## Input selection

Choose one source scope intentionally:

- Pass a file or directory for explicit filesystem scope.
- Use `--commit <rev>`, `--branch <name>`, or `--merge-request` only for commands that support Git-based selection.
- Use `--filter` to include matching paths and `--ignore` to exclude matching paths. Treat values as regular expressions unless current help states otherwise.
- Quote regexes in the shell, especially on Windows or when they contain backslashes, brackets, whitespace, or `|`.
- Pass `--standard` explicitly for non-RAD input when inference is uncertain.
- Use `--offline` for disconnected work and disable network-dependent checks when necessary.

Do not combine selection modes speculatively. Inspect `<command> --help` if precedence is unclear.

## Commands

### Inspect schemas

Generate the schema before creating or substantially editing metadata:

```shell
acorn schema rad
acorn schema raid
acorn schema rad > rad-schema.json
```

Use the generated schema to confirm required fields, nested shapes, accepted enum values, and unknown-field behavior. Validate examples with ACORN instead of treating a remembered sample as canonical.

### Check data

Use `check` as the primary verification command:

```shell
acorn check ./content/project/index.json
acorn check ./content
acorn check CITATION.cff --standard cff
acorn check report.docx --standard docx --skip conventions,schema
acorn check --commit HEAD
```

Use `--skip` only for categories the task intentionally excludes. Prefer `--disable-website-checks` or global `--offline` when network checks are inappropriate. Do not use `--no-fail` as proof that validation succeeded.

### Format or link data

Both commands can modify files. Preview first unless the user explicitly requested immediate mutation:

```shell
acorn format ./content --dry-run
acorn format ./content
acorn link ./content --dry-run
acorn link ./content
```

After mutation, inspect the diff and run `acorn check` on the affected scope.

### Export artifacts

Select the format and output directory explicitly when artifact location matters:

```shell
acorn export ./content --dry-run
acorn export ./content/project/index.json --format pdf --output ./export
acorn export ./content --target highlight --format powerpoint --output ./export
acorn export ./content --format bag --output ./export
acorn export ./content --format markdown --combine --output ./export
```

Supported formats and targets can change; read `acorn export --help` rather than relying on a fixed list.

Validate the final artifact:

- Confirm the expected file exists at the resolved output path.
- Confirm PDF output starts with a PDF header and can be opened or parsed.
- Confirm PowerPoint output is a readable OOXML ZIP package with required relationships.
- Confirm BagIt output has the expected archive root and manifests.
- Parse JSON or YAML output and re-run `acorn check` where applicable.

A success message without a usable artifact is not completion.

### Download RAD or models

Download configured research content:

```shell
acorn download https://github.com/org/repository
acorn download --config .acorn.json --output ./content
```

Download model weights:

```shell
acorn download model owner/model
acorn download model owner/model --filter "Q8_0.*\\.gguf$"
acorn download model owner/model --ignore "Q2_|Q3_|imatrix"
acorn download model owner/model --interactive
```

ACORN prefers GGUF model files and can discover community quantization repositories when the requested repository contains none. Use `--no-fallback` only when discovery is unwanted. Remember that model `--filter` and `--ignore` values are regular expressions, not Hugging Face globs.

Use global `--offline` to prevent network access. Do not claim a download succeeded until the expected files exist and checksum verification, when available, has passed.

### Gather, import, create, and serve

Inspect help before these networked or environment-changing workflows:

```shell
acorn gather --help
acorn import spec --help
acorn create runner --help
acorn create bot <PROJECT_ID> --remote ssh://user@docker-host
acorn serve mcp --help
```

Use `import spec --dry-run` before writing generated endpoint metadata. `create bot` is already detached. A create command using `--remote` requires the local Docker client, an SSH-configured remote Docker host, and permission for the SSH user to access the remote Docker socket; published ports and volumes belong to that remote host. Treat runner, bot, server, and external API operations as stateful actions and confirm their target, credentials, runtime, and network access before execution.

### Diagnose or use the TUI

```shell
acorn doctor
acorn doctor --report
acorn doctor --fix
acorn doctor --fix --interactive
acorn tui
```

Run diagnostics before applying fixes. Use `--report` when the user needs a machine-readable issue report. Treat `--fix` as a mutation and summarize what changed.

## Common workflows

Create or repair RAD data:

```text
schema rad -> edit -> format --dry-run -> format -> check
```

Produce a deliverable:

```text
check -> export --dry-run -> export -> inspect the artifact
```

Process only changed files:

```text
choose commit/branch/MR scope -> preview if available -> run -> inspect diff/artifacts -> check
```

Work disconnected:

```text
add global --offline -> disable or skip only network-dependent checks -> run local validation -> report skipped coverage
```

## Guiding ACORN work

- Prefer the narrowest command and input scope that proves the requested outcome.
- Preserve user-authored metadata; do not normalize or rewrite adjacent content without authorization.
- Use dry runs and diffs to explain mutations before applying them.
- Keep command syntax separate from schema advice.
- Distinguish validation failures from tool/runtime failures.
- Verify external side effects and generated artifacts directly.
- Report skipped checks, offline limitations, and unverified assumptions.

## Common errors

- **Unknown field or invalid enum** — Generate the relevant schema and correct the data shape; do not silence schema validation.
- **No files selected** — Check the resolved path, standard, Git selection, and include/exclude regexes.
- **Website, API, or download failure** — Check global `--offline`, credentials, connectivity, and the command's network-specific flags.
- **No GGUF files found** — Allow fallback discovery, use `--interactive`, or provide an intentional regex filter; use `--no-fallback` only deliberately.
- **Export reports success but no usable file exists** — Resolve the output path, confirm file creation, inspect its signature/package structure, and reproduce with verbose logging.
- **PowerPoint cannot be opened or repaired** — Inspect the OOXML ZIP layout and relationship targets; passing helper tests alone is insufficient.
- **PDF is not saved** — Confirm the final path and PDF header; do not rely on a success banner.
- **Command or flag is rejected** — Run `acorn <command> --help` or consult the checkout's `CommandLineHelp.md`; the installed binary and repository may differ.
- **Behavior changes under `--offline`** — Report which network-backed checks or downloads were unavailable instead of presenting reduced coverage as full success.

## Installation

Use a release channel appropriate to the user's platform and verify it immediately:

```shell
cargo install --locked acorn-cli
acorn --version
```

For repository development:

```shell
cargo install --locked --path ./acorn-cli
acorn --version
```

Consult the current repository documentation for binary, Scoop, and container locations rather than hardcoding release-version URLs in this skill.
