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

Rejected records remain in the inbox with inspectable outcomes in the durable operation store. Cleanup preserves unrelated bytes and concurrent appends, keeps an empty file after draining, and retries conflicts from a fresh snapshot. GitLab intake proposes changes through the existing review workflow.

### Run one intake cycle from the native executable

Build the local MVP with `cargo build --package acorn-cli --bin acorn`; the executable is `target/debug/acorn`. Keep a persistent operation database for this operator and use an isolated GitLab pilot project or branch. The configured `options.powerAutomate.gitlab_project` must be numeric, `options.powerAutomate.intake` must identify the JSONL source, and each folder must resolve to a path in that GitLab project. `tests/fixtures/automation/powerautomate/multi-config.json` shows the two-folder mapping used by mocked integration checks; substitute the pilot project ID and actual GitLab routes for a live run.

```sh
export CI_SERVER_HOST=code.ornl.gov
export GITLAB_TOKEN='REPLACE_WITH_PROTECTED_TOKEN'
umask 077
mkdir -p .acorn
export ACORN_DATABASE_PATH="$PWD/.acorn/operator-operations.sqlite"
export POWER_AUTOMATE_INBOUND_SECRET='REPLACE_WITH_PROTECTED_SECRET'
export SUBMISSION_ID='REPLACE_WITH_SUBMISSION_UUID'
target/debug/acorn intake run --config .acorn.json
target/debug/acorn intake export --config .acorn.json --submission-id "$SUBMISSION_ID" --output change.diff
```

`intake run` polls one inbox snapshot and all available review-note event pages, then attempts up to 256 durable review operations. Its JSON output includes intake, note, and queue counts. Repeating it after an authorized `/acorn graduate` merge request note advances the same review workflow as the bot. Replays use the durable database and do not create a second operation. A rejected JSONL line remains in the inbox with its recorded reason. A worker or intake failure exits nonzero; inspect the operation database before retrying. The command starts no HTTP listener and sends no callbacks.

The GitLab review-note poller reads every project-events page in ascending order before queuing events. It stores the last fully queued event in the same operation database, so a restart resumes without dropping older pages or duplicating completed commands. GitLab accepts a date for its `after` filter; ACORN overlaps that date and rechecks events at the cursor timestamp so late events with the same timestamp are still found. Set `--after` to a UTC date or timestamp before the first bot boot to bound the initial scan when the project already has a long event history. `/ready` reports the local operation runtime; verify GitLab reads and callback delivery separately.

`intake export` reads the admitted route from that database and uses the open merge request's computed base/head revisions. It verifies that GitLab's current review branch head still matches the MR head and that the file contains the admitted entry, then writes a UTF-8 unified diff. A newly created MR may need another invocation after [GitLab computes its diff](https://docs.gitlab.com/api/merge_requests/). The adjacent `change.diff.receipt.json` records the submission ID, source digest, GitLab service and project, repository path, branch, MR IID, target branch, base/head revisions, and SHA-256 digest of the diff. The diff reflects the full selected project file change on that review branch. If that branch carries multiple submissions, the artifact includes all of them. Confirm the branch and receipt before using the artifact. The command does not edit a project file or publish a package.

After the callback flow has a protected `POWER_AUTOMATE_CALLBACK_URL`, deliver ready durable intents explicitly:

```sh
export POWER_AUTOMATE_CALLBACK_URL='https://REPLACE_WITH_PROTECTED_CALLBACK_URL'
target/debug/acorn intake deliver --config .acorn.json
```

The callback command attempts up to 256 ready intents. Its JSON summary contains counts only, including pending retries and terminal failures; retryable intents remain durable for later attempts. Keep the URL and shared secret in protected environment variables. For a local PDF and manifest from a reviewed checkout, use an exact merged commit and an explicit cutoff; dry-run performs no package publication or callback:

```sh
export MERGED_SHA='REPLACE_WITH_MERGED_COMMIT_SHA'
target/debug/acorn report --dry-run --config .acorn.json --root . --commit "$MERGED_SHA" --through '2026-11-01T00:00:00Z' --project acorn --output .acorn/reports
```

Set `CHROME_PATH` or pass `--chrome-path` when ACORN cannot locate a Chromium-compatible browser for PDF rendering.

For an offline local proposal/apply smoke, copy `tests/fixtures/automation/powerautomate/logbook.jsonl` into a scratch directory and configure folder `acorn` with member `scientist@example.org` and a `file:` URI to a writable RAD `index.json`. Run `intake stage`, inspect `.acorn/intake/proposals/<UUID>.json`, then run `intake apply` as shown below. The repository's focused local tests create this fixture from `ResearchActivity::default()` and cover replay and conflict behavior. Keep the local operator stopped when a Vergil bot consumes the same inbox and review branch with a different operation database.

## Standalone local logbook review

For a registered folder with a `file:` destination and no bucket association, use a local JSONL inbox:

```sh
acorn intake stage --config .acorn.json --inbox logbook.jsonl
# Review .acorn/intake/proposals/<submission-UUID>.json, then:
acorn intake apply --config .acorn.json --submission-id <submission-UUID>
```

Staging checks the same form and member rules and writes a proposal containing the exact source line, normalized entry, destination, base index digest, and proposed index content. It does not edit the inbox or index. Rejected lines are reported separately, so valid lines can still be staged. Applying one reviewed UUID rechecks the route and index digest; if the index changed, stage again. Repeated application is safe. Use one writer per index. The local adapter does not create GitLab merge requests or import bucket data; bucket association alone never registers a folder or starts a copy. GitLab and remote URI destinations use their own workflows.
