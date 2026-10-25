# 📥 Intake

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Review logbook submissions from a JSONL source with <code>acorn intake</code>. Use <code>stage --source</code>, <code>inspect</code>, and <code>apply</code> for local review, whether the source is a local file or a GitLab file URL. Use <code>run</code> for the configured GitLab merge-request workflow.</p>
</blockquote>

## Workflow

Both workflows check submissions against the configured folder and member rules before proposing a project change. Local review writes proposals for existing local indexes; the GitLab worker creates merge requests. The source location does not change the review workflow or destination.

```pikchr
scale = 0.72
LocalInbox: box "Local / GitLab" "JSONL source" wid 1.15in ht 0.52in rad 0.08in fill 0xF3F0FF thickness 0
Stage: box "intake stage" wid 1.1in ht 0.52in rad 0.08in fill 0xE8F1FB thickness 0 at LocalInbox+(1.48in,0)
Proposal: box "intake inspect" wid 1.1in ht 0.52in rad 0.08in fill 0xFFF4D6 thickness 0 at Stage+(1.48in,0)
Apply: box "intake apply" wid 1.1in ht 0.52in rad 0.08in fill 0xE7F3E9 thickness 0 at Proposal+(1.48in,0)
Index: box "RAD index" wid 1.05in ht 0.52in rad 0.08in fill 0xF2F2F2 thickness 0 at Apply+(1.45in,0)
arrow from LocalInbox.e to Stage.w thick color 0x00662C
arrow from Stage.e to Proposal.w thick color 0x00662C
arrow from Proposal.e to Apply.w thick color 0x00662C
arrow from Apply.e to Index.w thick color 0x00662C
GitLabInbox: box "GitLab JSONL" "inbox" wid 1.15in ht 0.52in rad 0.08in fill 0xF3F0FF thickness 0 at LocalInbox+(0,-0.95in)
Run: box "intake run" wid 1.1in ht 0.52in rad 0.08in fill 0xE8F1FB thickness 0 at Stage+(0,-0.95in)
Review: box "Review merge" "request" wid 1.1in ht 0.52in rad 0.08in fill 0xFFF4D6 thickness 0 at Proposal+(0,-0.95in)
Export: box "intake export" wid 1.1in ht 0.52in rad 0.08in fill 0xE7F3E9 thickness 0 at Apply+(0,-0.95in)
Diff: box "Diff + receipt" wid 1.05in ht 0.52in rad 0.08in fill 0xF2F2F2 thickness 0 at Index+(0,-0.95in)
arrow from GitLabInbox.e to Run.w thick color 0x00662C
arrow from Run.e to Review.w thick color 0x00662C
arrow from Review.e to Export.w thick color 0x00662C
arrow from Export.e to Diff.w thick color 0x00662C
```

<div id="intake-workflow" data-pikchr-fit hidden></div>

The arrows show the main review path. `intake export` is an optional, read-only artifact step after a GitLab review branch exists. Run `intake run` again to process review notes. `intake deliver` separately sends ready callbacks from the durable queue; it is not required to export a diff.

## JSONL sources for local review

`--source` accepts an ordinary filesystem path, an absolute `file:` URI, or a GitLab HTTPS file link. For example:

```sh
acorn intake stage --batch --config .acorn.json --source logbook.jsonl
acorn intake stage --batch --config .acorn.json --source 'file:///Users/me/project/logbook.jsonl'
CI_SERVER_HOST=code.ornl.gov acorn intake stage --batch --config .acorn.json \
  --source 'https://code.ornl.gov/research-enablement/experiments/power-automate/-/raw/main/.acorn/intake/logbook.jsonl'
```

GitLab links may use `/-/raw/` or `/-/blob/`, followed by a branch, tag, or commit and the file path. Encode a slash inside a revision as `%2F` (for example, `feature%2Freport`). Encode spaces in URI paths as `%20`; ordinary local paths can be quoted. Use links without credentials, query parameters, or fragments. Other remote services and URI schemes are not supported by this adapter.

GitLab access uses the existing `GITLAB_TOKEN`, `PRIVATE_TOKEN`, or `CI_JOB_TOKEN` environment references. The link's origin must match `CI_SERVER_HOST` (default `gitlab.com`); credentials are never taken from the link. The remote adapter resolves the revision to an immutable commit, reads that exact file, and saves its original bytes plus a `source.json` provenance receipt under `.acorn/intake/sources/`. The receipt records the GitLab service, project, branch, path, commit, last file commit, and SHA-256 content digest. Missing files fail explicitly. Sources are limited to 10 MiB and individual records to 64 KiB.

Subsequent inspection and apply use the captured file, including when offline. They do not re-fetch a moving branch. Keep the snapshot directory with the proposals or batch manifest; changing or deleting the captured bytes invalidates a batch. Staging the same revision and content reuses the snapshot without overwriting it. To review newer remote content, stage the link again. `--offline` rejects remote links before fetching; it accepts local paths and file URIs, including previously captured snapshots.

Remote staging does not require a bot, operation database, or Power Automate callback configuration. It does not drain the remote inbox, create a merge request, or send notifications. Destination and membership rules still come from the explicit ACRE configuration.

For one reviewed batch across existing standalone `file:` project indexes:

```sh
acorn intake stage --batch --config .acorn.json --source logbook.jsonl
acorn intake inspect --config .acorn.json --batch-id <batch-ID>
acorn intake apply --config .acorn.json --batch-id <batch-ID>
acorn intake inspect --config .acorn.json --batch-id <batch-ID>
```

`stage --batch` prints a stable batch ID and stores a private manifest under `.acorn/intake/batches/`. It groups entries by canonical project and folds repeated project lines in source order. Review each line outcome, destination, and proposed diff with `inspect` before applying. A rejected line prevents applying the whole batch. The summary's `canApply` field reports this decision. Stage and inspect leave the inbox and indexes untouched. Apply rechecks the exact source, configuration, routes, access, and index digests before its first write. If interrupted, repeat apply with the same batch ID; inspect shows each project's current state. An unrelated index change requires a new reviewed batch.

Both `apply` selectors display the terminal **Yes / No** buttons before changing an index. No is selected initially: Enter declines, Tab or the left/right arrows switches the selection, and `y` or `n` chooses directly. Escape or Ctrl+C cancels. Declining or cancelling returns a nonzero exit status without applying changes. Review the staged proposal or batch diff first; the prompt identifies the selection and configuration.

For a previously reviewed unattended invocation, add `--yes`. This is required if either input or output is redirected, including piping the receipt to another program. Missing terminal access never implies approval. Apply reloads its configuration after approval so membership changes during the prompt take effect. `--yes` skips only the prompt; admission, source, route, digest and replay checks still run.

An entry may supply `identifier`, `global`, `archived`, `origin`, and `discovery` when preserving existing report records. Without them, intake generates a submission-based identifier and uses a manual, active, project-only entry. Discovered entries require discovery provenance.

For the existing single-submission workflow:

```sh
acorn intake stage --config .acorn.json --source logbook.jsonl
# Review .acorn/intake/proposals/<submission-UUID>.json, then:
acorn intake apply --config .acorn.json --submission-id <submission-UUID>
```

`stage` writes proposals without changing the inbox or RAD index. `apply --submission-id` rechecks the destination and base index digest before writing one reviewed proposal. If the index changed, stage the submission again. A GitLab-sourced inbox needs an explicit handoff: stop its intake worker, use an exact local snapshot, reconcile reviewed index changes into the approved repository, and handle only the exact completed lines before restarting the worker.

## Local graduation

After ingestion, `run --index` promotes supported logbook entries into canonical metadata. Choose how to approve them:

```sh
acorn intake run --index project/index.json --dry-run > candidates.json
acorn intake run --index project/index.json --interactive

# Accept every eligible promotion without prompts:
acorn intake run --index project/index.json --force

# Alternatively, accept specific entries from an exact reviewed preview:
acorn intake run --index project/index.json \
  --accept <reviewed-entry-ID> --expected-digest sha256:<digest-from-preview>
```

The preview prints candidates, source logbook context, and base/proposed SHA-256 digests without changing the file. Personnel entries propose contributor names; supported administration entries can propose classification. Review the proposed target and value before accepting. Repeat `--accept` for multiple reviewed entries; this mode requires `--expected-digest` and writes without prompting.

| Mode | Behavior |
| --- | --- |
| `--dry-run` | Validate and preview every eligible promotion; write nothing. |
| `--interactive` | Ask Yes / No for each candidate, then confirm the selected write. |
| `--force` | Accept every eligible promotion without prompts. |
| `--accept ID --expected-digest DIGEST` | Apply the specified reviewed entries without prompts. |

Interactive buttons default to No. No skips an individual candidate; Escape or Ctrl+C cancels the entire operation. Declining the final write also cancels without creating a backup. Interactive mode requires terminal input and output. `--dry-run --interactive` previews the selected entries without a final write prompt. `--dry-run --force` remains read-only. `--force` conflicts with `--interactive` and `--accept`. With eligible candidates, a bare `run --index` requests an explicit approval mode; with none, it returns `unchanged` without writing.

Graduation validates the whole selection and both input and output, retains and archives the source entries, and saves exact original bytes in an adjacent `.graduation-<SHA256>.before` file before atomic replacement. `--force` preserves these checks and leaves unsupported entries unchanged. You can also supply `--expected-digest` with `--force` or `--interactive` to pin a previous preview. The index is checked again after interactive approval. JSON, JSONC, YAML/YML, and ZONF indexes are supported; serialization may normalize formatting and remove JSONC comments.

Unknown, archived, ambiguous, or stale selections fail without modifying the index. A repeated invocation with the old digest fails safely; rerun the preview to see remaining candidates. Use an isolated checkout with exclusive ownership while applying: the digest is a review guard, not a lock against unrelated concurrent editors. Preserve the emitted before/after digest receipt and backup. These local commands neither approve a remote merge nor remove any GitLab inbox lines.

## GitLab review

```sh
acorn intake run --config .acorn.json
acorn intake export --config .acorn.json --submission-id <submission-UUID> --output change.diff
acorn intake deliver --config .acorn.json
```

`run --config` polls the configured inbox and review notes once, then processes bounded durable work. `export` writes a unified diff and an adjacent JSON receipt for an admitted submission. `deliver` attempts ready callback intents without running the bot. This mode rejects local graduation flags and cannot run offline. GitLab commands need a persistent `ACORN_DATABASE_PATH`; see [ACRE's GitLab logbook inbox setup](../../concepts/acre.md#gitlab-logbook-inbox) for configuration, retry behavior, and prerequisites.
