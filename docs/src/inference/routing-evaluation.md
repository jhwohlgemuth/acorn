# Native Needle routing evaluation

## Current status

Native Needle routing is experimental. Unscoped `--route` requests only preview a capability choice; running a handler requires an explicit capability and exact caller JSON. The pinned runner selected 44/75 supported capability names in a fresh November 9 calibration, so automatic selection is excluded from the explicit-dispatch release. The former PAR-15 held-out release gate is retired; a future automatic selector needs a separately versioned evaluation.

| Revised calibration measure (reproduced 2026-11-09) | Result |
| --- | ---: |
| Supported cases | 75 |
| Expected capability retained by shortlist | 74/75 |
| Raw Needle call names matching expected capability | 44/75 |
| Correct previews accepted by the execution gate | 7/75 |
| Forbidden, negated, or ungrounded cases eligible to execute | 0 |

The revised candidate has **not** been run on the held-out split. An earlier baseline did score all 200 cases, including held-out cases, before the repair work. That baseline is preserved below; its held-out result is not a score for the revised candidate. The PAR-34 safety harness replays all 30 negative corpus rows through synthetic refusals, negation, and grounding flags against a safe terminal handler and observes zero calls or artifacts. Separate tests verify that caller JSON overrides model arguments only after canonical validation, offline and remote policy withholds ineligible bindings, exact mutation grants are required, response errors never run a handler, and mismatched version, schema, or effects reject a binding. Compatible alternative-binding policy is replayed separately; the frozen corpus lacks version/schema declarations for its incompatible-binding examples, so those rows remain marked as missing contract evidence. These tests do not establish what flags the live model would emit.

## Initial 200-case baseline

The first real Needle measurement ran all 200 cases in corpus revision 3 on 2026-11-03. It is preserved in [`results.json`](../../../tests/fixtures/routing/results.json) (tracked SHA-256 `dfc292d99c0e205f75ac3576941a29e758cfde997b3b60f2b0ff12b89eb46a84`). The tracked corpus SHA-256 is `86467c8aff2dbeff4033aff67fa590d5a4d9d7943c3a4a42012a3a2d53865379`.

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

The required gate is 95% correct selection on unambiguous supported cases and zero executions on forbidden, negated, or ungrounded cases. This baseline was far below the selection threshold, and preview mode alone could not prove the negative-case execution gate. PAR-16 and ADP-06/07 release prerequisites remain pending.

PAR-34 calibration replays on 2026-11-04 tested a compact model-facing schema projection with a 6-KiB prompt shortlist. The frozen calibration split still produced only 18/75 raw correct capability names and 1/75 accepted correct previews. Outcomes across 100 cases were 71 withheld, 16 needs-input, 10 tool-failed (all `truncated`), two invalid-input, and one proposed. The shortlist retained 71/75 expected capabilities; serialized catalogs ranged from 1,517 to 6,143 bytes. The harness used case-specific offline policy and caller-confirmed local check paths, but did not yet apply every declared binding or mutation policy. These results were diagnostic calibration evidence. Single-capability `acorn.check` with caller-confirmed JSON produced a correct proposed preview at 238 catalog bytes; the larger shortlist reduced its confidence below the 0.8 execution gate. The later PAR-34 safety matrix is described above.

A separate calibration-only two-stage probe used 13 simple Needle family descriptors. Its first stage chose the correct family for 12/75 supported cases. Replacing `acorn.route::<family>` names with direct `acorn.<family>` names and narrowing to the lexical candidate families still misclassified the `READ-001` metadata check; an explicit family-selection instruction also misclassified it. The probe was removed from production code. This result does not support shipping a second Needle decision with the pinned runner.

The replacement first stage is deterministic: ACORN infers a capability family from the prompt and sends that family's eligible tools to Needle. A parser repair also preserves family names in IDs with `::` variants. The first exact-family calibration replay retained 73/75 expected capability IDs, produced 36/75 raw correct Needle call names, and accepted 12/75 correct previews. `READ-001` improved from a truncated 749-byte three-family catalog to an accepted `acorn.check` preview with a 238-byte exact-family catalog and 0.9788 confidence. That replay omitted the caller JSON from the model prompt, unlike the CLI route, so it is diagnostic only.

After aligning caller JSON presentation and binding preference with the CLI, a second 100-case calibration replay retained 73/75 expected IDs, produced 39/75 raw correct calls, and accepted 10/75 correct previews. Its outcomes were 44 withheld, 22 needs-input, 14 proposed, 11 tool-failed (all `truncated`), five invalid-input, and four unavailable. Catalogs ranged from 224 to 6,143 bytes. The remaining shortlist misses were `EFFECT-017`, which mentions both crosswalk and export, and offline `BOUND-025`, whose publication binding is unavailable. A subsequent lexical rule gives explicit export requests priority. A single `EFFECT-017` replay confirmed that its expected capability is now retained; Needle returned `needs-input` without selecting a tool. The revised rule has not had a full calibration replay. No held-out scores were used to tune these rules.

Review repairs after these replays preserve canonical union argument shapes, retain tagged check inputs, exclude path components from exclusive family hints, and resolve the corpus's native binding aliases before applying overrides and preferences. The measurements above predate those repairs; the later calibration measurements below include them.

The repaired canonical catalog replay on 2026-11-05 retained 73/75 labeled calibration targets, but Needle named only 16/75 correctly; 34/100 responses were truncated and no correct preview passed the gate. A separate capability-only routing catalog, which asks Needle to choose a name and requires exact caller JSON for execution, eliminated truncation and raised raw correct names to 41/75. It retained 74/75 targets and accepted two correct previews. An explicit name-only instruction lowered raw correct names to 23/75 and was removed. A small optional source hint improved the current calibration candidate to 44/75 raw correct names and seven accepted correct previews, with 74/75 shortlist retention and no truncated responses; selection catalogs were at most 4,172 bytes. No forbidden, negated, or ungrounded calibration case was eligible to execute without preview mode. These are calibration results; the revised candidate has not been evaluated on held-out because the 95% gate is unmet. The harness records declared binding candidates and replays compatible alternative-binding policy separately from model choice. The corpus does not contain schemas for its incompatible binding examples, so those contracts are reported as missing contract evidence rather than scored as faithful policy replays.

The 95% correct-selection gate is unmet. The pinned Needle runner frequently abstains or chooses the wrong variant even when the expected family is present. A fresh November 9 run reproduced 44/75 correct raw names, seven accepted correct previews, and 74/75 shortlist retention. The preview-only live harness does not prove negative-case nonexecution under a live handler. PAR-34 separately establishes synthetic policy and safe-handler evidence for explicit dispatch; automatic selection stays preview-only.

A calibration-only preview fallback was measured after the capability-only replay. It left Needle's raw score at 44/75 and raised a combined model-or-rule suggestion to 66/75. It also replaced some correct Needle variants with wrong suggestions. The fallback was removed. Native routing remains opt-in, and unscoped `--route` requests are preview-only; an explicit capability and caller JSON are required to run a handler. The pinned runner is not qualified as an automatic selector across the near-parity catalog. A future automatic-selector candidate must be frozen before held-out scoring.
