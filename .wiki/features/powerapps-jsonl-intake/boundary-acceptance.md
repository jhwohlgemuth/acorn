# PA-15 boundary acceptance

October 25, 2026: **Waiting on validation.** The 1,200-character summary passed in both isolated environments, with one exact append and all prior bytes preserved. The next valid 1,000-character-detail case passed schema/envelope checks in each environment, then received an explicit network-range HTTP 403 at `Read current PA17v2`. These cases are blocked, not rejected-field passes. No operational component or inbox was changed.

## Isolation and source

| Environment | Boundary flow | Source |
| --- | --- | --- |
| Wohlgemuth, Jason | `1654dae8-ccd0-f111-a05c-001dd8062052` | Native Save As of PA17 TEST ONLY Native Intake |
| ORNL Personal Productivity | `7a1f9c5b-ced0-f111-a05c-001dd806277c` | Native Save As of restored PA17 TEST ONLY Native Intake |

Both copies are named **PA15 TEST ONLY Boundary Acceptance**, have no app associations, use existing protected test connections, and were returned to **Off** after testing. Their notification conditions contain constant `false == true`; no Teams or email notification was sent. The retained PA-17 source flows were not edited or enabled.

The connector is fixed to project `22464`, branch `main`, `.acorn/intake/acceptance/pa17-logbook.jsonl`. Because this is also the PA-17 fixture path, tests were serialized and the baseline restored after each environment. The operational `.acorn/intake/logbook.jsonl` was outside the write set.

The [Jason scope](_assets/boundary-tests/jason-copy-scope.json) and [ORNL scope](_assets/boundary-tests/ornl-copy-scope.json) have identical persistence logic; only environment-specific connector API IDs differ. [Notification guards and fixture instructions](_assets/boundary-tests/README.md) accompany the snapshots.

## Observed runs and byte proofs

| Environment / case | Run ID | Outcome |
| --- | --- | --- |
| Jason summary, exactly 1,200 | `08584086351886334203735033186CU11` | Passed: commit `6d9750fb332417822aaa6c358429312d1e874395`; 457 → 2,055 bytes; exactly one 1,598-byte JSONL line; existing bytes unchanged |
| Jason detail, exactly 1,000 | `08584086349885467069440049950CU24` | Field schema/envelope passed; branch/history/catalog calls succeeded; file read returned network-range 403; no append |
| ORNL summary, exactly 1,200 | `08584086347270902567817077681CU03` | Passed: commit `1dba1a06a8df4a3a980673b1b678ca141354db39`; 457 → 2,055 bytes; exactly one 1,598-byte JSONL line; existing bytes unchanged |
| ORNL detail, exactly 1,000 | `08584086346787841230598875135CU07` | Field schema/envelope passed; branch/history/catalog calls succeeded; file read returned network-range 403; no append |

Each accepted appended JSON object equals its frozen submission fixture exactly. File snapshots and GitLab metadata are retained beside each fixture. The local checks verify the prefix, exact JSON and one-line delta; the green flow-run status alone was not used as persistence proof.

Two fixture-preparation runs are excluded from positive boundary acceptance: Jason run `08584086353248183236772270521CU17` used `items: []` and correctly failed schema validation; run `08584086350544084656306899290CU29` included the local file's trailing newline and correctly failed the raw-envelope guard. The corrected payload used one valid detail and no raw trailing newline. These were fixture errors, not product defects or failures of the requested field limits.

## Remaining matrix

| Case | Current evidence | Remaining acceptance |
| --- | --- | --- |
| Summary 1,200 | Exact append passed in both environments | None for this dimension |
| Summary 1,201; 21 details; detail 1,001 | Three individual no-write server passes retained from October 24 in the [runbook](pilot-runbook.md#isolated-live-server-boundaries) | Repeat against the preserved native test revision in ORNL; preserve per-case file/revision proof |
| Detail 1,000 | Both native scopes passed field validation, then read 403 | Confirm one append and exact previous-byte preservation in both environments |
| 20 details | Frozen independent fixtures prepared | Runtime append proof in both environments |
| Whitespace summary/detail; empty detail array | Published client checks and empty-array schema rejection observed | Complete independent server guard/no-write evidence for both native test copies |
| Blank/rejected lines, mixed LF/CRLF, Unicode | Deterministic byte fixture prepared and locally checked | Real conditional append preserves every original byte in both environments |
| Malformed tail without terminal LF | Fixture prepared and locally checked | Runtime unsafe-tail result and unchanged file/revision in both environments |
| Exact append-size cap and one byte over | Saved-expression arithmetic checked; padded fixtures generated locally | Accepted/rejected runtime boundary cases in both environments |

No additional write cases were forced through after the explicit access denials. This does not reopen PA-02 or require an administrator ticket; it preserves the known limitation and the unpassed PA-15 cases. PA-16 concurrency and PA-17 restoration retain their independent gates.

## Actual byte limits

The current saved expression checks proposed-file base64 length ≤ **13,981,012**, equivalent to at most **10,485,759 UTF-8 bytes**. The pre-existing file guard allows at most 10,485,760 bytes. A proposed file of exactly 10 MiB therefore fails the append-size condition. This is a conservative bound, one byte below 10 MiB, and was not changed during testing. Simply raising the base64 threshold to 13,981,016 would also admit 10,485,761 and 10,485,762 bytes; any future change needs an exact-byte predicate and separate validation.

The [local checks](_assets/boundary-tests/local-checks.json) are source/arithmetic evidence, not a substitute for cloud size/tail acceptance. The [fixture preparer](_assets/boundary-tests/prepare-inbox-fixtures.py) writes only to a new local output directory. Large padded files were generated under a temporary directory and were not uploaded or added to wiki assets.

## Restoration and cleanup

Both accepted records were preserved as local evidence and then removed from the isolated fixture by conditional restoration using the exact expected bytes and last file commit. Jason restore commit: `1ebd78d91239026a080119ae4d0d63c44be68176`. Final ORNL restore commit: `5d74108011b3517cf01d9ed304ed6736f9e9c699`.

The [final readback](_assets/boundary-tests/final-restored-baseline-metadata.json) confirms **457 bytes**, SHA-256 `adcbd471fb91c075b4d48604648d118e6fd2c3de30b1688780de845827a71f15`, exactly matching the original baseline. There were no other writers to reconcile. Reuse the saved UUID only for the same frozen payload; immutable GitLab history still contains the accepted test commits even though the fixture file was restored.

Completed PA-14 apps/flows, the accepted failure mock, personal Teams screenshots and duplicate wiki files were removed separately. See the [cleanup inventory](test-artifact-cleanup.md).
