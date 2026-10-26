# PA-15 boundary acceptance fixtures

These are synthetic TEST ONLY submissions and isolated evidence. They must never enter sponsor reporting. The cloud copies use project `22464`, `main`, `.acorn/intake/acceptance/pa17-logbook.jsonl`; the operational `.acorn/intake/logbook.jsonl` is outside this write set. Both copies have a constant-false Teams guard and no app association.

See [boundary acceptance](../../boundary-acceptance.md) for actual results. A prepared fixture or passing local calculation is not cloud acceptance.

## Resume the remaining matrix

1. Confirm both native PA-17 intake flows and the other boundary copy are Off. Use only one boundary copy at a time, with its existing protected test connection. Confirm its fixed connector destination and disabled notification guard before enabling it.
2. Read the isolated file and preserve bytes plus its last commit. Do not overwrite another test operator's work. The captured baseline is `baseline.jsonl` (457 bytes); do not assume the live file still matches it.
3. Use one field dimension per fixture. Paste the compact JSON without trailing CR/LF into Submission JSON. The flow deliberately rejects raw line separators; the newline terminating a local JSON file is not part of the submitted envelope. Freeze the exact string and UUID across retries of that case.
4. For each accepted case, verify exactly one appended line, identical prior bytes, matching frozen JSON and immutable commit. For rejection, verify unchanged bytes/last file commit and the specific rejected/failed action. The overall run may say Succeeded after an unresolved response; that is not append proof.
5. For preservation/tail/size cases, prepare local bytes with `prepare-inbox-fixtures.py ENVELOPE OUTPUT_DIRECTORY`. Generate each size case using its own frozen envelope. Upload only the corresponding fixture to the isolated test path with a last-commit guard. The helper does not upload or access credentials. Use a temporary output directory; large padded files do not belong in the wiki.
6. After each case, restore the previous exact bytes using a conditional write only after confirming the current content/revision is the expected test result. If it changed independently, stop for reconciliation. Disable the boundary flow afterward.

## Exact current limits

Saved schema: summary 1–1,200 characters; 1–20 detail strings or null; each detail 1–1,000 characters. Trimmed blank summaries/details fail the separate envelope check. Empty `items: []` fails the schema; use a valid detail in unrelated tests.

The old-file check accepts at most 10,485,760 bytes. The proposed append check uses base64 length at most 13,981,012 characters, which means at most **10,485,759 UTF-8 bytes**. Thus 10 MiB exactly is rejected by the current append expression. Do not substitute character count for UTF-8 bytes or silently raise the base64 threshold (the next base64 bucket also admits bytes above 10 MiB).

`local-checks.json` records source comparison, the size calculation and the Jason exact-summary byte proof. The two scope snapshots differ only in environment-specific connector API IDs. The [five-attempt evidence](five-results.json) records ORNL tail and oversized rejection passes, the accepted ORNL count append, and the remaining runtime cases. Mixed-line and exact-cap appends remain unaccepted in both environments; Jason tail and oversized rejection remain unaccepted.
