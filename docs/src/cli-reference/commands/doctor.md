# 👨‍⚕️ Doctor

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Inspect the host environment and capture a support report with <code>acorn doctor</code>.</p>
</blockquote>

Start broad, then narrow the diagnostic category:

```sh
acorn doctor
acorn doctor --check software
acorn doctor --check system,memory,network
acorn doctor --report > acorn-doctor.json
```

Doctor reports use the same discovery-record JSON schema as the interactive Doctor screen. The software probes cover ACORN, Chromium, Git, Node.js, npm, npx, OpenCode, ORAS, Pandoc, and Vale.

`--fix` applies only reversible, low-risk automatic fixes and then reports the verified host state. Package-manager and network installers require `--fix --interactive`: select findings with Space, preview the fixed operations with `f`, and confirm with `y`. `n` or Escape declines the plan. `--interactive` without `--fix` remains report-only, and `--report` cannot be combined with interactive mode.

## Rescue: ACORN cannot find Chrome

PDF export needs Chrome or Chromium. Point ACORN at an existing executable instead of relying on automatic discovery:

```sh
# Linux
acorn --offline export index.json --format pdf \
  --chrome-path /usr/bin/google-chrome-stable

# macOS
acorn --offline export index.json --format pdf \
  --chrome-path "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
```

On Windows, pass the executable path with `--chrome-path` or set `CHROME_PATH`. Connected export can download ACORN's pinned browser; offline export needs a configured, installed, or already cached browser. See [Chrome selection](export.md#chrome-selection-for-pdf-export).

## Rescue: offline mode unexpectedly fails

Global `--offline` forbids network reads; it does not turn remote inputs into local ones. Check these common causes:

1. Replace HTTP(S) inputs and model-list URLs with local paths or `file://` URIs.
2. Use `check --skip prose` if analyzer extensions have not already been synchronized.
3. Configure `--chrome-path` or cache the pinned browser before offline PDF export.
4. Do not combine `gather --resolve`, `gather --merge-request`, or `gather --osti` with offline mode.

Run the failing command with `-vv` to expose the resource ACORN attempted to resolve.

## Rescue: a model sync target is not detected

With no target flags, `acorn sync` selects applications whose commands it detects. Request a target explicitly to make the decision visible:

```sh
acorn sync --vscode --dry-run
acorn sync --vscode --force --dry-run
```

`--force` bypasses command detection only for targets you explicitly include. If the application is detected but weights are elsewhere, set `--models-dir`; use `--assume-models` only when another system guarantees those paths exist. Continue with the [Sync troubleshooting guide](sync.md).

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Preview application routing with <a href="sync.md">Sync</a>.</p>
