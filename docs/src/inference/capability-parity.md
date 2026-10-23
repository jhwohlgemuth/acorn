# Native capability parity inventory

The [machine-readable inventory](../../../tests/fixtures/routing/parity.json) records 72 distinct CLI behaviors at the PAR-00 baseline: 46 in the bounded native routing milestone and 26 explicit exclusions. Aliases such as `ask`/`infer`, `lint`/`check`, and `fetch`/`download` share their behavior row. The inventory is a denominator, not a coverage claim: none of the pending equivalence entries counts as covered until its named phase supplies a passing CLI/service-to-registry comparison.

Each row identifies its command, inputs and options, output or artifact, effect, host or network requirement, current registry relationship, phase owner, and equivalence obligation or exclusion. The legacy `acorn.check_research_activity_data` entry performs schema validation. The new routed `acorn.check` binding calls the full local `analyze_paths` service and reports category status and structured findings. Remote, Git, OCI, fix-preview, and direct CLI equivalence obligations remain open. Existing format and export tools cover narrower RAD operations. The remaining native families have no matching registry binding yet.

The bounded milestone uses the existing command behaviors recorded in the routing plan. In particular, check includes category status, offline handling, and typed findings; effectful families require explicit destination and apply authority. Direct prompt and MCP commands remain available while generic adapters develop their routed bindings. Clipboard and TUI interaction, persistent servers, infrastructure or credential creation, automatic repair, unrestricted automation, and durable intake operations are counted as exclusions.

## Extraction seams

`acorn-lib::analyzer::host::service::analyze_paths` and its structured `AnalysisReport` provide the reusable local analysis operation. The routed binding delegates to it; the CLI still owns broader input selection and rendering. PAR-02 completes the remaining command forms and equivalence evidence. Other capability leaves must use an existing library service where one is present; if a command combines operation and presentation, its named native phase extracts only that operation before registration. The PAR-00 inventory identifies each leaf and owner so this can be checked before work on that leaf begins.

## Counting and evidence

The bounded denominator is 46 behavior rows. The total CLI denominator is 72 rows, including the 26 exclusions. A row becomes covered only after its focused equivalence test proves matching results, failure behavior, offline policy, effects, and artifacts for the applicable input form. Report both covered/46 and covered/72; keep exclusions visible and do not convert these ratios to a percentage until the corresponding evidence is recorded.

The first 50 routing cases in `tests/fixtures/routing/cases.json` cover forbidden requests, explicit negation, ungrounded targets, missing inputs, and ambiguous actions. Each class has ten distinct prompts, with five calibration and five held-out labels. Every expected result has `execute: false` and no effects. Later batches add effectful and selection-boundary cases to the same fixture without changing these labels.

The next 50 cases are grounded read-only requests for check, diff, gather, doctor, SWHID, and bounded script checks. They name the intended capability and existing fixture inputs where applicable, and split evenly between calibration and held-out prompts. Their expected operation count is one, with read effects only; they do not authorize a write.

The following 50 cases exercise format, link, export, import, download, sync, report, and script-check selection. Each records caller mutation policy, an exact destination when authority is granted, expected effects, and artifact references. Preview requests permit only their declared read effects; denied apply requests execute nothing. The corpus uses symbolic `$CASE_TMP` destinations and does not perform remote effects during routing evaluation.

The final 50 cases test explicit binding choices, availability, offline policy, equivalent bindings, and the remaining mandatory input families. A missing or denied explicit binding must not silently fall back to a different executor. Local operations remain eligible offline. Together, the 200 cases reference all 46 bounded inventory rows; a reference is routing coverage, not behavioral parity evidence.

## Frozen routing gate

The PAR-17 baseline is 200 unique case IDs and prompts, split into 100 calibration and 100 held-out cases. Each of the four 50-case batches has a 25/25 split. All 46 bounded inventory behaviors have at least one case reference. The 50 safety cases cover five classes equally and expect zero executions and effects. Denied apply cases also expect zero effects. Declared local input paths exist; symbolic destinations and configured bindings are interpreted by a future routing harness. Remote examples are selection inputs and do not authorize live network effects in the corpus check.

The held-out prompts, labels, and expected outcomes are fixed before router scoring. Selection or threshold changes may use calibration cases; held-out results are reported without tuning on them. Any later correction to the frozen cases must be documented as a new corpus revision before scoring, preserving this baseline for comparison. The frozen SHA-256 values are `2d34af03f47ae80fbe1b68573b15d5ed35d82ab91702da140659b4359cfc092c` for `parity.json` and `c598546499da225fafa58bb2a1e2e6dca886ffde153d82144aa6c9e9e1d002aa` for `cases.json`.
