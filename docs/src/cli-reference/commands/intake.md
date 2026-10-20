# 📥 Intake

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Review logbook submissions from a JSONL inbox with <code>acorn intake</code>. Use <code>stage</code>, <code>inspect</code>, and <code>apply</code> for local files, or <code>run</code> for a configured GitLab inbox.</p>
</blockquote>

## Workflow

Both routes check submissions against the configured folder and member rules before proposing a project change. The local route writes a proposal for review; the GitLab route creates a merge request for review.

```pikchr
scale = 0.72
LocalInbox: box "Local JSONL" "inbox" wid 1.15in ht 0.52in rad 0.08in fill 0xF3F0FF thickness 0
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

## Local files

For one reviewed batch across existing standalone `file:` project indexes:

```sh
acorn intake stage --batch --config .acorn.json --inbox logbook.jsonl
acorn intake inspect --config .acorn.json --batch-id <batch-ID>
acorn intake apply --config .acorn.json --batch-id <batch-ID>
acorn intake inspect --config .acorn.json --batch-id <batch-ID>
```

`stage --batch` prints a stable batch ID and stores a private manifest under `.acorn/intake/batches/`. It groups entries by canonical project and folds repeated project lines in source order. Review each line outcome, destination, and proposed diff with `inspect` before applying. A rejected line prevents applying the whole batch. The summary's `canApply` field reports this decision. Stage and inspect leave the inbox and indexes untouched. Apply rechecks the exact source, configuration, routes, access, and index digests before its first write. If interrupted, repeat apply with the same batch ID; inspect shows each project's current state. An unrelated index change requires a new reviewed batch.

An entry may supply `identifier`, `global`, `archived`, `origin`, and `discovery` when preserving existing report records. Without them, intake generates a submission-based identifier and uses a manual, active, project-only entry. Discovered entries require discovery provenance.

For the existing single-submission workflow:

```sh
acorn intake stage --config .acorn.json --inbox logbook.jsonl
# Review .acorn/intake/proposals/<submission-UUID>.json, then:
acorn intake apply --config .acorn.json --submission-id <submission-UUID>
```

`stage` writes proposals without changing the inbox or RAD index. `apply --submission-id` rechecks the destination and base index digest before writing one reviewed proposal. If the index changed, stage the submission again. A GitLab-sourced inbox needs an explicit handoff: stop its intake worker, use an exact local snapshot, reconcile reviewed index changes into the approved repository, and handle only the exact completed lines before restarting the worker.

## GitLab review

```sh
acorn intake run --config .acorn.json
acorn intake export --config .acorn.json --submission-id <submission-UUID> --output change.diff
acorn intake deliver --config .acorn.json
```

`run` polls the configured inbox and review notes once, then processes bounded durable work. `export` writes a unified diff and an adjacent JSON receipt for an admitted submission. `deliver` attempts ready callback intents without running the bot. GitLab commands need a persistent `ACORN_DATABASE_PATH`; see [ACRE's GitLab logbook inbox setup](../../concepts/acre.md#gitlab-logbook-inbox) for configuration, retry behavior, and prerequisites.
