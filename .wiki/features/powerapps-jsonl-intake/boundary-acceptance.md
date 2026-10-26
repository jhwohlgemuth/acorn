# PA-15 boundary acceptance

October 25, 2026: **Waiting on validation.** The 1,200-character summary and 1,000-character detail passed in both isolated environments. Jason's 20-detail count also passed. Each accepted case appended exactly once and preserved every prior byte. All listed field-rejection checks passed. The latest series added ORNL's exact-detail acceptance, followed by three attempts each for ORNL's count, mixed-line and tail cases; all nine stopped at GitLab HTTP 403 responses. An independent file-read timeout prevented installing the next size fixture. Remaining persistence cases are unaccepted. No operational component or inbox was changed.

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

### Later retry

The [retry evidence](_assets/boundary-tests/retry-results.json) consolidates 14 observed runs, decisive action statuses, fixture dimensions, nine byte/revision readbacks, and final Off-state checks. Transient full UI captures and duplicate file bodies were removed after consolidation.

| Environment / case | Run ID | Outcome |
| --- | --- | --- |
| Jason detail, exactly 1,000 | `08584086339199446408757739438CU14` | Passed: commit `04d0942f5abd29797ece197d8760c2a6382227cb`; 457 → 1,856 bytes; one exact 1,399-byte line; prior bytes unchanged |
| Jason 20 details | `08584086338865674950268384681CU10` | Blocked: `Read current PA17v2` returned network-range 403; no append |
| Jason whitespace summary, first attempt | `08584086338035189912077067962CU22` | Blocked: main-branch preflight returned network-range 403; validation skipped |
| Jason whitespace detail | `08584086337718606547670622201CU05` | Passed: `Rejected envelope PA17v2` succeeded; no write |
| Jason empty detail array | `08584086337511683187473333000CU01` | Passed: `Parse submission PA17v2` failed with `ValidationFailed`; no write |
| ORNL detail, exactly 1,000 | `08584086336732800057803637219CU16` | Blocked: catalog lookup returned 403; file read skipped; no append |
| ORNL summary, 1,201 | `08584086336168059693565909498CU13` | Passed: schema rejection; no write |
| ORNL 21 details | `08584086335281418730547740499CU26` | Passed: schema rejection; no write |
| ORNL detail, 1,001 | `08584086335072007845992062395CU00` | Passed: schema rejection; no write |
| ORNL whitespace summary | `08584086334958600906622055116CU25` | Passed: envelope rejection; no write |
| ORNL whitespace detail | `08584086334879198386699439959CU16` | Passed: envelope rejection; no write |
| ORNL empty detail array | `08584086334789241665735459220CU05` | Passed: schema rejection; no write |
| ORNL 20 details | `08584086334533126005387498966CU30` | Blocked: submission-history lookup returned network-range 403; file read skipped; no append |
| Jason whitespace summary, second attempt | `08584086333878321377087991318CU13` | Passed: envelope rejection; no write |

The six ORNL guard runs share an independent readback confirming the same 457-byte body and last file commit before and after the group. The Jason detail/empty-array guards preserved the accepted exact-detail snapshot and revision; the final whitespace-summary retry preserved the restored baseline and revision. These append-only scopes cannot hide an intervening write behind an unchanged last file commit. One case varies at a time, so no failed guard is inferred from a different field's failure.

Two fixture-preparation runs are excluded from positive boundary acceptance: Jason run `08584086353248183236772270521CU17` used `items: []` and correctly failed schema validation; run `08584086350544084656306899290CU29` included the local file's trailing newline and correctly failed the raw-envelope guard. The corrected payload used one valid detail and no raw trailing newline. These were fixture errors, not product defects or failures of the requested field limits.

### Second retry: count and file fixtures

The [second retry evidence](_assets/boundary-tests/retry2-results.json) retains six run outcomes and twelve independent file snapshots as hashes, sizes and commit IDs. Only the accepted append body is retained; large padded files remain outside the wiki.

| Environment / case | Run ID | Outcome |
| --- | --- | --- |
| Jason 20 details | `08584086320883016883829744137CU09` | Passed: commit `a7eda88d8d1753b48463726bd02d3ec90db7001e`; 457 → 1,444 bytes; one exact 987-byte line; previous bytes unchanged |
| Jason mixed blank/rejected lines, LF/CRLF and Unicode | `08584086320286375706413637199CU31` | Blocked: catalog lookup returned 403; the installed 512-byte fixture and its revision stayed unchanged |
| Jason malformed tail | `08584086319835097131863099025CU26` | Blocked: file read returned network-range 403; unsafe-tail guard skipped; 511-byte fixture and revision unchanged |
| Jason exact proposed-file cap | `08584086319438050132338584389CU17` | Blocked: history lookup returned network-range 403; size guard skipped; 10,485,336-byte fixture and revision unchanged |
| ORNL detail, exactly 1,000 | `08584086318612297066129422991CU06` | Blocked: history lookup returned network-range 403; baseline and revision unchanged |
| ORNL 20 details | `08584086318416333908211324542CU11` | Blocked: main-branch preflight returned network-range 403; validation skipped; baseline and revision unchanged |

Each fixture replacement checked the exact expected current bytes and last file commit before issuing a conditional write, then verified the uploaded bytes. The size fixture plus its frozen 423-byte submission line would total exactly 10,485,759 bytes. That arithmetic is verified; the cloud size condition did not run. The one-byte-over case and ORNL file-fixture cases were not run after these prerequisite denials. The test flows' definitions, protected connections and disabled notification guards stayed unchanged.

### Requested three-attempt series

The [three-attempt series evidence](_assets/boundary-tests/rounds-results.json) retains ten new cloud runs, their decisive action statuses and fifteen byte/revision snapshots. Run IDs for every attempt are in that record.

| Case | Attempts in this series | Result |
| --- | --- | --- |
| ORNL detail, exactly 1,000 | 1 | Passed in run `08584086313716113033206959982CU22`: commit `5e55d47713dd5a7741e451391bde94173052399a`; 457 → 1,855 bytes; one exact 1,398-byte line; original bytes preserved |
| ORNL 20 details | 3 | All blocked by 403, respectively at catalog lookup, main-branch preflight and history lookup; file bytes/revision unchanged |
| ORNL mixed blank/rejected lines, LF/CRLF and Unicode | 3 | All blocked at main-branch preflight with 403; installed 512-byte fixture and revision unchanged |
| ORNL malformed tail | 3 | First two blocked at main-branch preflight, third at catalog lookup, all with 403; unsafe-tail guard never ran; 511-byte fixture and revision unchanged |

The next ORNL exact-cap setup timed out during its initial independent GitLab file read after 45 seconds, before issuing any PUT. That fixture was not installed and no size run was started. Repeated denials and loss of timely independent file verification prevented continuing the intended three-attempt series for size and the Jason file cases. This was not three completed rounds of the entire remaining matrix. No unrun or skipped validation is counted as acceptance.

Restoration then succeeded under the original byte/revision guards, followed by two matching readbacks. The test-only flows were confirmed Off. Protected connections, networking controls and flow definitions were unchanged throughout.

## Remaining matrix

| Case | Current evidence | Remaining acceptance |
| --- | --- | --- |
| Summary 1,200 | Exact append passed in both environments | None for this dimension |
| Summary 1,201; 21 details; detail 1,001 | Jason's three individual no-write server passes are retained in the [runbook](pilot-runbook.md#isolated-live-server-boundaries); ORNL native-copy cases passed in this retry | None for these dimensions |
| Detail 1,000 | Exact append passed in both environments | None for this dimension |
| 20 details | Jason exact append passed; three more ORNL attempts blocked by 403 | Runtime append proof in ORNL |
| Whitespace summary/detail; empty detail array | Independent server guards passed in both native test copies, with unchanged bytes/revision | None for these dimensions |
| Blank/rejected lines, mixed LF/CRLF, Unicode | Jason attempt and three ORNL attempts blocked by 403 before append | Real conditional append preserves every original byte in both environments |
| Malformed tail without terminal LF | Jason attempt and three ORNL attempts blocked by 403 before the tail guard | Runtime unsafe-tail result and unchanged file/revision in both environments |
| Exact append-size cap and one byte over | Arithmetic checked; Jason exact-cap attempt blocked by 403; ORNL exact-cap setup timed out before fixture installation | Accepted/rejected runtime boundary cases in both environments |

Retries used unchanged authorized connections; no network or credential control was changed. After the second retry's successful count append, mixed-content, unsafe-tail and exact-cap fixtures were installed only at the isolated path and tested individually. Their upstream denials do not count as rejection or preservation passes. This does not reopen PA-02 or require an administrator ticket; it preserves the known limitation and the unpassed PA-15 cases. PA-16 concurrency and PA-17 restoration retain their independent gates.

## Actual byte limits

The current saved expression checks proposed-file base64 length ≤ **13,981,012**, equivalent to at most **10,485,759 UTF-8 bytes**. The pre-existing file guard allows at most 10,485,760 bytes. A proposed file of exactly 10 MiB therefore fails the append-size condition. This is a conservative bound, one byte below 10 MiB, and was not changed during testing. Simply raising the base64 threshold to 13,981,016 would also admit 10,485,761 and 10,485,762 bytes; any future change needs an exact-byte predicate and separate validation.

The [local checks](_assets/boundary-tests/local-checks.json) are source/arithmetic evidence, not a substitute for cloud size/tail acceptance. The [fixture preparer](_assets/boundary-tests/prepare-inbox-fixtures.py) writes only to a new local output directory. Large padded files were generated under a temporary directory; only Jason's exact-cap fixture was subsequently uploaded to the isolated test path. No large padded file was added to wiki assets.

## Restoration and cleanup

Both accepted records were preserved as local evidence and then removed from the isolated fixture by conditional restoration using the exact expected bytes and last file commit. Jason restore commit: `1ebd78d91239026a080119ae4d0d63c44be68176`. Final ORNL restore commit: `5d74108011b3517cf01d9ed304ed6736f9e9c699`.

The earlier [final readback](_assets/boundary-tests/final-restored-baseline-metadata.json) and the later `retry-final-baseline` readback in [retry evidence](_assets/boundary-tests/retry-results.json) confirm **457 bytes**, SHA-256 `adcbd471fb91c075b4d48604648d118e6fd2c3de30b1688780de845827a71f15`, exactly matching the original baseline. The later accepted detail record was restored conditionally in commit `d2d04425bfb54d63f3a5df7d7d5eba4431f8821c`; the remaining runs in that retry left that file revision unchanged. There were no other writers to reconcile. Reuse the saved UUID only for the same frozen payload; immutable GitLab history still contains the accepted test commits even though the fixture file was restored.

After the second retry, conditional restoration commit `835f1433a7ef78f900c9bbb2b42223bc7bffd5b1` returned the isolated file to the same original 457-byte baseline. ORNL's subsequent attempts left that revision unchanged, confirmed by the `final-baseline` snapshot in the second retry evidence. Both copies were reloaded and confirmed **Off**: [Jason](_assets/boundary-tests/retry2-jason-off.png), [ORNL](_assets/boundary-tests/retry2-ornl-off.png).

After the requested three-attempt series, the latest restoration commit is `ec84e2b6512529466511ca30d9ce2b6a198e2d76`. Both `rounds-restored` and `rounds-recovery-probe` snapshots confirm the original 457 bytes, the original SHA-256 above, and that same file revision. Final Off-state proofs: [Jason](_assets/boundary-tests/rounds-jason-off.png), [ORNL](_assets/boundary-tests/rounds-ornl-off.png).

Completed PA-14 apps/flows, the accepted failure mock, personal Teams screenshots and duplicate wiki files were removed separately. See the [cleanup inventory](test-artifact-cleanup.md).
