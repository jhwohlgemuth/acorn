# Initial SharePoint read recovery

## Diagnosis — November 24, 2026

Flow: [ACORN SharePoint Logbook Export](https://make.gov.powerautomate.us/environments/221f44a6-1468-ea4e-b93d-cd52f1723079/flows/51efa5f9-6e69-4d75-a53b-2c2965fd51bd/details).

The failed-run filter showed six failed runs in the available history: October 30; November 13, 16, 17, 20 and 23. The November 23 and November 20 runs were inspected individually. Both failed at `Read_latest_receipt_LB` with `BadRequest` and this connector message:

> Http request failed as there is an error: 'An error occurred while sending the request.'.

- [November 23 run](https://make.gov.powerautomate.us/environments/221f44a6-1468-ea4e-b93d-cd52f1723079/flows/51efa5f9-6e69-4d75-a53b-2c2965fd51bd/runs/08584061644854188583987374992CU26): 846 ms overall; first read failed; downstream actions skipped.
- [November 20 run](https://make.gov.powerautomate.us/environments/221f44a6-1468-ea4e-b93d-cd52f1723079/flows/51efa5f9-6e69-4d75-a53b-2c2965fd51bd/runs/08584064236849435293859640176CU19): 401 ms overall; same first-read error.

The first read explicitly uses `retryPolicy: none`. The flow was On, last modified October 28, with recent five-minute runs succeeding, including November 24 at 13:30 local time. Its SharePoint connection was resolved. The observed failure is a connector transport error; the underlying Microsoft network/service cause is not exposed. This is not evidence of malformed stored JSONL or a persistent authentication failure. Neither inspected failed run reached publication.

## Deployed repair

The builder wraps only the initial `GetItems` call in `Retry_latest_receipt_LB`. It makes at most three attempts, waits ten seconds before attempts two and three, and has a two-minute Until limit with `FailWhenLimitsReached`. It retries the observed transport-message `BadRequest`, timeouts, 408, 429 and 5xx responses. Other errors stop immediately. The final failed read leaves the loop failed; exhaustion cannot admit a snapshot as successful. The high-water calculation runs only after the loop succeeds.

The read request, protected inputs/outputs, source list, subsequent reads, JSONL validation, publication, exact readback, five-minute recurrence, single-run concurrency and connection bindings are preserved. No write is retried by this repair. The approval-gated beta exporter and GitLab sync flows are outside this repair.

`before.native.zip` is a fresh native export obtained before edits. `before.definition.json` is its extracted definition. `after.definition.json` and `repair.import.zip` contain the prepared update. The latter is a modified import package, not an independent native backup. Its other package entries are byte-identical to the backup.

## Deployment status

**Saved to the existing flow on November 24, 2026 at 13:46 America/New_York.** The classic designer accepted the repaired scope; Flow checker reported zero errors and zero warnings. The flow remains On with the existing connection and schedule.

The [13:47:28 verification run](https://make.gov.powerautomate.us/environments/221f44a6-1468-ea4e-b93d-cd52f1723079/flows/51efa5f9-6e69-4d75-a53b-2c2965fd51bd/runs/08584060608373988299867797841CU28?v3=false) succeeded. The retry loop completed in one iteration. `Write_logbook_LB` was skipped because the bytes were unchanged; `Read_published_logbook_LB`, `Exact_logbook_LB` and `Export_receipt_LB` succeeded. Protected content remained hidden in run history. See [run screenshot](live-success.jpg) and [verification record](deployment-verification.json).

`deployed.native.zip` is the independent post-save native export; `deployed.definition.json` is its extracted definition. Comparison with the prepared definition found only designer normalization: metadata IDs for the three new actions, omitted empty else/description, expression comma spacing, and save timestamps. Recurrence, concurrency, connection bindings, targets and the entire downstream publication graph matched. No connector fault was injected. The successful hosted run validates the normal path; recovery from a real Microsoft transport outage remains unobserved.

## Validation

```sh
PYTHONDONTWRITEBYTECODE=1 python3 .wiki/features/powerapps-jsonl-intake/_assets/sharepoint/read-retry-repair/test_retry.py
PYTHONDONTWRITEBYTECODE=1 python3 .wiki/features/powerapps-jsonl-intake/_assets/sharepoint/export-failure-tests/test_exporter.py
```

Focused checks evaluate the generated WDL retry predicate for the observed error, permanent errors, throttling, timeouts, server errors and success; verify retry bounds and the success dependency; compare generated artifacts; and prove that the package preserves all unrelated flow content. These are offline checks, not a hosted connector-fault demonstration.

All eight retry checks and nineteen existing exporter checks passed. Wiki validation and scoped whitespace checks also passed.

Microsoft documents that [standard retry policies cover 408, 429 and 5xx](https://learn.microsoft.com/en-us/azure/logic-apps/error-exception-handling), so this repair explicitly handles the observed transport error wrapped as `BadRequest`. The [Until loop documentation](https://learn.microsoft.com/en-us/azure/logic-apps/logic-apps-control-flow-loops) defines last-iteration failure behavior and `FailWhenLimitsReached`.
