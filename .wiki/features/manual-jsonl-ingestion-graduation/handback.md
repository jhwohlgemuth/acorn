# Manual JSONL handback

Use this procedure before a bot reads an inbox that people have already processed with local intake commands. Local receipts do not populate the bot's admission database. An archived logbook entry or an existing contributor is not evidence that the bot has admitted its source submission.

The offline MAN-06 rehearsal passed on October 25, 2026 using the captured TEST ONLY personnel record and the existing local CLI, whose binary hash is retained in the evidence. The source contract was reviewed at `d4799a11`. This completes the procedure and rehearsal; it does **not** authorize a production merge, remove remote inbox lines, certify the live pilot baseline, or start Vergil. PA-05 owns selecting and approving the actual deployment baseline using this procedure.

## Ownership and evidence

Jason Wohlgemuth is the recorded pilot owner. Before a real transition, the repository owner must name the reviewer with canonical merge authority and the operator responsible for the worker/database. Catalog membership and a submission's email address do not establish that authority. Keep the service and one-shot worker stopped throughout reconciliation. Never run the manual workflow and bot against the same inbox concurrently.

Retain the following together in a protected review bundle. Record references to credentials, never their values.

| Evidence | Required content |
| --- | --- |
| Source identity | GitLab origin, numeric project, branch, repository-relative inbox path; original capture commit, last file commit, full-file SHA-256 and exact snapshot bytes |
| Manual decisions | Selected submission IDs, original physical line/offset and exact bytes, accepted/rejected decision, reviewer and timestamp; selection alone does not mean completed |
| Application evidence | Inspected batch/proposal, successful apply receipt, canonical route, configuration/member registry revision and before/after index digests |
| Graduation evidence | If graduated: selected entry IDs, reviewed digest, result, byte-exact backup, retained archived source entry and canonical target values |
| Current baseline | Current branch head, current inbox bytes/digest/file commit, current index bytes/digest and canonical merge commit |
| Transition authority | Reviewed reconciliation diff, approval/merge reference, final source/index/config revisions, worker owner, persistent database location and activation decision |

The receipt must be reconciled with the current index. Verify each completed record's identity, target and content. For a graduated record, verify both the archived source entry and promoted value. If unrelated edits changed the index, review those changes and issue a new baseline; do not treat a matching contributor name as proof of completion. A submitted, staged, rejected, cancelled, or partially applied record is not eligible for removal.

## Reconcile exact records

1. Capture the source at an immutable revision. Preserve the original snapshot and source receipt. Perform manual review in an exclusively owned checkout using the [current CLI workflow](../../../docs/src/cli-reference/commands/intake.md). Resolve partial applications before handback. A human reviews and merges canonical changes through the repository's normal process.
2. With the bot still stopped, capture the **current** branch and inbox and compare them with the original capture. Do not replace the current inbox with an edited old snapshot. Match completed records by exact bytes and submission identity; line numbers can change. Preserve LF delimiters, CRLF bytes, blank lines, order and all unrelated bytes. Never parse and reserialize the whole inbox for cleanup.
3. Build a proposed removal set containing only records whose manual completion and current canonical state are verified. Require each selected record to occur exactly once in both snapshots with no other occurrence of its submission ID. Changed bytes, duplicate IDs, missing records, or contradictory receipts stop reconciliation for review. Do not automatically delete a same-ID conflict, even if one version was completed. An already missing line requires evidence of the earlier cleanup, not another deletion.
4. Preserve every new, rejected, unknown and unreviewed line. A malformed record is retained; a nonterminated tail stops the proposal and needs separate repair. Rejected records are not silently discarded or considered processed. New submissions require their own review or later bot admission.
5. Review a diff against the **current** source, showing only exact completed records removed. Verify the canonical index is unchanged by cleanup. Pin the source revision/digest and index revision/digest used for that review. A producer append or index change invalidates the reviewed proposal: capture again, recompute, and review again.
6. Apply through an authorized reconciliation branch and human-approved merge, with optimistic concurrency enforced against the captured revision. For a GitLab file/commit API operation, require the observed file `last_commit_id`; for a branch merge, require a current target and an exact reviewed merge result. Do not force-push, auto-resolve an inbox conflict, or retry an old replacement body after a conflict. Re-read the resulting target commit and verify exact remaining bytes and canonical digest before recording the handoff.
7. PA-05 may activate only against that approved source/index/config baseline. Recheck it immediately before activation. Keep the same persistent bot database on restarts. Do not manufacture bot admission rows from local receipts or substitute a fresh database for reconciliation. If the exact completed submission later reappears, pause and investigate; this procedure does not create a permanent bot deduplication tombstone.

The production source in the captured evidence is project `22464`, `main`, `.acorn/intake/logbook.jsonl`. None of those remote resources was changed by the rehearsal. Keep the operational form, flow, connections and notifications unchanged while preparing the transition. If producers remain active, expect their appends and repeat review when the current revision changes. An access denial leaves the source intact and the transition unaccepted.

## Fresh isolated alternative

PA-05 can instead use a separately approved, fresh isolated inbox/index pair. Its certificate must identify the repository, branch, inbox and index paths, exact initial commit and file digests, registry revision, owner and intended database. Verify the inbox is initially empty and that the index has no manually processed test submissions; admit only newly generated TEST ONLY IDs. The owner must explicitly confirm that the fixture has no prior manual processing. A renamed copy of the old inbox is not fresh. This alternative leaves the old source intact and does not authorize production activation.

## Reproduce the offline rehearsal

The [rehearsal script](_assets/rehearse-handback.py) accepts only the frozen TEST ONLY input whose hashes it verifies. It creates a new local repository without a remote, runs `stage`, `inspect`, approved synthetic `apply`, preview and graduation, then rehearses exact-line reconciliation. It is a fixture verifier, not a production cleanup command. The synthetic approval uses `--yes` and reviewed entry/digest selection; real reviewers use the existing confirmation buttons or their recorded explicit approval.

```sh
python3 .wiki/features/manual-jsonl-ingestion-graduation/_assets/rehearse-handback.py \
  --workspace /tmp/acorn-man06-new \
  --acorn target/debug/acorn
```

Supply `--git /path/to/git` if needed. The workspace must not exist. The script retains the command outputs, source/canonical commits, exact digests, final diff and evidence JSON. It uses `--offline` for every ACORN command, never invokes the service or `run --config`, and never imports receipts into a bot database.

The [retained evidence](_assets/handback-evidence.json) and [fixture archive](_assets/handback-rehearsal.zip) preserve the successful run. The archive contains a Git bundle with the original, canonical, intervening-append and reconciled commits, plus manual receipts and graduation backup. Extract it into a new directory; clone `fixture.bundle` into a separate checkout to inspect the commits. The recorded configuration and receipts contain historical temporary paths; rerun the script for a usable new workspace.

| Rehearsal check | Result |
| --- | --- |
| Local application and reviewed graduation | Exact contributor plus archived source entry verified; original inbox unchanged |
| Reconciliation | One exact completed line removed; all other original records retained |
| Intervening append | New valid record, rejected route, unknown object, malformed record, blanks, CRLF and Unicode retained byte for byte |
| Stale source or canonical index | Proposal refused; no file changes |
| Missing, duplicated or changed accepted bytes | Proposal refused; no file changes |
| Same-ID conflict or nonterminated tail | Proposal refused; no file changes |

Bot admission is a separate contract: `is_admitted` in `crates/acorn-lib/src/io/api/powerautomate/jsonl.rs` reads the operation queue's `intake_admission`, not local receipt files or canonical indexes. Its first-admission test queues a valid line in an empty database; the restart test avoids duplicate work only when the same database is retained. This is why the baseline reconciliation remains a deployment condition even after the offline rehearsal passes.

The [focused validation record](_assets/handback-validation.txt) records 18 passing existing admission, replay and cleanup tests, including append races, unchanged rejected/blank/Unicode bytes and conflicting replay bytes. These tests use local mock servers; no real bot was started. Run them with:

```sh
cargo nextest run --package acorn-lib --features agentic --lib \
  -E 'test(io::api::powerautomate::tests::jsonl) | test(test_jsonl_cleanup_)'
```
