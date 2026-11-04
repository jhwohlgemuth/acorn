# Native Needle routing evaluation

PAR-15 has **not passed**. The first real Needle measurement ran all 200 cases in corpus revision 3 on 2026-11-03. It is preserved in [`results.json`](../../../tests/fixtures/routing/results.json) (SHA-256 `f2d31a8fc2ab523b9d1181269ce8975365a504786bcf1ed00548486b53c8f4da`). The corpus SHA-256 is `d87c85d47df150fdb3aeaf18a90e5f8a403778eeac5d6f0ba62883cdb8045596`.

The harness started the pinned local Needle runner once per case with the native registry and a deterministic prompt shortlist. It invoked `route_once` in preview mode and asserted that no handler content or artifacts were produced. It measured capability choice, outcome, catalog size, and elapsed time. It did not execute the corpus's declared mutations, apply binding selections, or validate output equivalence. The first run used an online-built registry for offline prompts; the harness now builds the registry per case, so the offline rows require remeasurement before acceptance.

| Measure | Calibration | Held-out |
| --- | ---: | ---: |
| Cases | 100 | 100 |
| Cases with an expected capability | 75 | 75 |
| Expected capability retained by shortlist | 73/75 | 71/75 |
| Raw Needle call names matching expected capability | 1/75 | 2/75 |
| Accepted correct capability previews | 0/75 | 0/75 |

Across all cases, Needle returned 163 `tool-failed`, 29 `withheld`, seven `needs-input`, and one `proposed` outcome. No native handler ran, by construction of the preview. The median case time was 1,692.5 ms; nearest-rank p95 was 4,740 ms. Serialized shortlists ranged from 5,066 to 21,162 bytes, under the 22-KiB prompt target and 32-KiB hard catalog cap. A focused calibration test retains at least 95% of eligible targets under the corrected per-case offline policy.

A diagnostic replay of `READ-002` with only `acorn.check` in the catalog still returned Needle error code `truncated` and message `tool call truncated: token budget exhausted` at 4,221 catalog bytes. A single-capability `acorn.doctor` replay returned an accepted `proposed` preview at 266 bytes. This identifies a model-facing schema or output-budget problem in addition to shortlist ranking. The 163 failures were not all individually classified by error code in the first result fixture.

The required gate is 95% correct selection on unambiguous supported cases and zero executions on forbidden, negated, or ungrounded cases. The first result is far below the selection threshold, and preview mode alone cannot prove the negative-case execution gate. PAR-15 remains open. The next bounded repair must make complex native schemas usable by the pinned runner, capture each Needle error and accepted selection, apply corpus policy and binding context, then rerun the calibration and held-out scores without tuning on held-out labels. PAR-16 and ADP-06/07 release prerequisites remain pending.
