# November 25 flow failure audit

Live Power Automate run histories were inspected in the existing pilot environment. Reported email counts are historical counts, not evidence of current failures. No finalization or publication was replayed, and the two approval publication flows remain Off.

| Flow | Verified finding | Disposition |
|---|---|---|
| ACORN SharePoint Logbook Export | The latest failed run is November 23. All six failures visible in the 28-day failed-run filter precede the November 24 retry repair. November 25 run `08584059859849344233917755856CU13` succeeded in five seconds. | Existing deployed repair remains healthy; no additional definition change. |
| [BETA] Sponsor Highlights Intake | Four failures on November 10/12 stopped in `Parse submission SEED` or `Parse submission SP` with `ValidationFailed`. The last failed run precedes the November 12 13:36 modification; later November 13 runs succeeded. | Historical schema defect already repaired. Participant-context correction acceptance remains open. |
| [BETA] Finalize Approval Batch | Latest failure `08584072418813039838277655393CU22` is the documented November 10 deliberate changed-source test. | Expected `SourceChangedPA37B` rejection before lease/write/commit. Preserve the guard and the synthetic batch; do not resubmit it. |
| [BETA] Approval Gated Combined Logbook Export | The failed-run filter contains only `08584072643992215426603596726cU14`, the documented deliberate corrupted-baseline test. Nine later displayed runs succeeded, including the post-write fault-recovery test. | Expected baseline-integrity rejection; no repair needed. Keep Off pending cutover acceptance. |
| [BETA] Request Reviewer Summary | All four November 13 failures reached `Respond_to_a_Power_App_or_flow`, which reported `ActionResponseTimedOut`; preceding authorization/request processing succeeded. | Response delivery failed after the caller connection closed. The exact cause of the caller timeout remains unresolved; a fresh synthetic request succeeded in 39 seconds. |

## Reviewer summary evidence

The four failed runs were individually inspected:

- `08584069986873310974247255165CU23` — November 13 17:16, 38 seconds.
- `08584069988840795410087602290CU31` — November 13 17:13, 39 seconds.
- `08584069993561803622576746570CU15` — November 13 17:05, 39 seconds.
- `08584069997983697270098761397CU02` — November 13 16:58, 39 seconds.

The action error says the client timed out waiting for the response and its connection was closed. Secure inputs/outputs remain enabled; private payloads were not exposed to investigate this error. The available run history does not establish whether the client navigated away, canceled, retried, or encountered an intermediary timeout. Do not treat these possibilities as a confirmed root cause.

The [fresh native backup](summary-before.native.zip) was downloaded before testing. Its trigger and action definitions match the retained [request builder output](../intake-approval/beta-repair/standard-summary-request.json). There is no trigger concurrency limit in that definition. The app-facing flow uses SharePoint only, authorizes reviewer membership, creates a restricted queue request, and polls every five seconds for up to 18 iterations / 95 seconds. The response retains the existing `summary` contract. Default connector retries mean the loop setting alone is not an end-to-end response deadline.

Synthetic health-check run `08584059853912658901697219640CU11` began November 25 at 10:44 Eastern and succeeded in 39 seconds. The authorization/request branch succeeded in 38.9 seconds and the response succeeded at 10:45:33. [The captured run](summary-health-success.jpg) verifies the response status. This disproves a consistent 30-second limit but does not explain the historical disconnects or prove participant-context reliability. Microsoft documents a [120-second inbound request limit](https://learn.microsoft.com/en-us/power-automate/limits-and-config#timeout), including Power Apps responses.

The published reviewer now opens `ApprovalReviewScreen` and says summaries are generated when the batch loads. This differs from the November 12 saved automatic-after-Save description; that historical description is not a current UI contract. The four-item Ready batch displayed two summaries pending after loading, without a new request appearing in the refreshed flow history during the observation. Selecting a pending item displayed “Generating a suggestion…”; explicit Regenerate Summary then invoked the flow and completed successfully. The subsequent [batch-summary startup repair](../batch-summary-repair/README.md) reproduced this separate dispatch defect and published an explicit timer-start sequence in app version 130. A fresh published launch then generated both queued suggestions automatically. Participant-context PA-40C acceptance remains open.

Published-app regeneration run `08584059852289954798080449922CU10` began November 25 at 10:47 Eastern and succeeded in 39 seconds. The app displayed Use Suggestion and reduced the pending count from two to one. Source item 24 remained Draft v1, ETag `"1"`, Needs review. Neither Use Suggestion nor Save was selected. This establishes an owner-context app-to-flow-to-app round trip; it does not establish participant access, automatic dispatch, or repair of the historical caller disconnect. No flow/app definition was changed in this audit.

## Related evidence and next acceptance

- [November 24 deployed logbook retry repair](../read-retry-repair/README.md). Its eight focused retry tests passed again on November 25.
- [November 12 intake schema repair](../intake-approval/beta-repair/verification.md), including the exact November 12 failed run and corrected activity/status parsing.
- [Finalizer changed-source acceptance](../intake-approval/acceptance/end-to-end.md#controlled-source-membership-fault).
- [Exporter corrupted-baseline acceptance](../intake-approval/exporter/failure-acceptance.md).
- [Remaining release gates](../../../approval-release/index.md). Participant correction, caller-context reviewer/summary acceptance, usability/PDF fidelity, denied-role, late-source, and lost-response evidence remain required before operational cutover.

If summary response delivery fails again in the published app, capture the caller operation and timestamps with the corresponding flow run before changing the contract. A durable request-ID/status polling interface is a possible coordinated app/flow repair; shortening the wait to return an empty summary or hiding the failed response would not establish reliable delivery.
