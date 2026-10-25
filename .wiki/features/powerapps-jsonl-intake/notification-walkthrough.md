# PowerApp and Power Automate walkthrough

Verified October 24, 2026, in the **Wohlgemuth, Jason** pilot. Jason Wohlgemuth remains the owner and initial notification recipient. The updated [contributor/reviewer guide](_assets/sponsor-highlights-user-reviewer-guide.pdf) and [developer guide](_assets/sponsor-highlights-developer-guide.pdf) begin with the current workflow; their later chapters retain dated evidence.

The user guide now includes a workflow map on **page 2**, styled like the ACORN book diagrams. It shows capture, review, apply, graduation and reporting, with solid automatic-notice and dashed owner-triggered notification branches. Editable [Pikchr source](_assets/notifications/workflow.pikchr), the rendered [vector diagram](_assets/notifications/workflow.svg), and the [printable page](_assets/notifications/workflow.html) are preserved here.

## Walk through the contributor experience

1. [Open Sponsor Highlights](https://apps.gov.powerapps.us/play/e/221f44a6-1468-ea4e-b93d-cd52f1723079/a/15ae605b-585b-49f5-a413-2b091f919a35). Use one app tab. The catalog loads automatically; **Load Project List** is available before a successful load and **Refresh Project List** afterward.
2. Select the project, date, category, and status. Describe what changed and why it matters. Add optional supporting details, event type, and report placement. Use only non-sensitive highlights: the pending draft is stored locally in the browser.
3. Select **Submit highlight** once. The app freezes the exact payload and receipt ID, disables editing during the request, and waits for a matching result.
4. **Highlight saved** means the exact envelope is in the GitLab JSONL logbook at the displayed commit. It still awaits ACORN review. Keep the receipt ID for support; use **Back to highlight** to return to the preserved entry.
5. If the result is uncertain, retain and retry the existing pending submission. Do not start another highlight for the same content. Rejected input can be edited; a conflicting identity needs owner review.

![Prepared LandScan test highlight](_assets/notifications/form-before-submit.png)

![Confirmed GitLab receipt](_assets/notifications/form-confirmed.png)

## Updates at the useful workflow stages

| Stage | Channel | Current trigger and meaning |
| --- | --- | --- |
| New highlight confirmed in GitLab | Jason's Teams self-chat | Automatic intake notice after the new-save response succeeds; includes project, receipt, and immutable commit link. |
| Review workspace created (`mr-created`) | Jason's Teams self-chat | Owner manually supplies the validated event. A later email requests review when ready. |
| No report changes (`report-no-changes`) | Jason's Teams self-chat | Owner manually records that the specified period produced no report changes. No action needed. |
| Review requested (`review-needed`) | Email to Jason | Important action request with the actual review link. |
| Completed report (`artifact-ready`) | Email to Jason | Important deliverable milestone with package metadata. |
| Actionable failure (`failed`) | Email to Jason | Owner attention needed; includes the supplied failure message. |
| Form validation, saving progress, uncertain result | App feedback | No message for keystrokes, project loading, or unconfirmed saves. |
| Exact duplicate callback / intake replay | No new notification | Keep the existing receipt and destination. |

These notices go to the pilot owner, not to every contributor. The callback flow is still owner-only and manually triggered. Local CLI ingestion, graduation, and reporting do not invoke it automatically. The Vergil integration and authenticated external callback endpoint remain future work. A later approved group/channel migration belongs to PA-12.

![Generic Teams-style illustration with fictional chats and notification messages](_assets/notifications/teams-example.png)

The illustration replaces the personal Teams screenshot in the shared guide. It explains the message format; it is not a captured test result.

## What Power Automate now does

**Intake** (`ff4a134b-5fd3-4465-ab19-0325c2f2bf1b`) retains its existing persistence scope and unresolved-response fallback. `Notify_confirmed_new_submission` runs after all outcomes of `Unresolved_failure`, but its condition requires `actions('Queued_new')?['status']` to be `Succeeded`. It does not run the send on replay, rejection, or unresolved persistence. `Post_intake_update_to_Teams` uses the existing Microsoft Teams Default-52fd4 connection, `PostMessageToSelf`, secure inputs/outputs, and no automatic retries. It omits the highlight summary/details from the message.

The notice is best effort and follows the confirmed response. A Teams failure can mark the overall run failed after the app already received a valid receipt; it does not undo the GitLab commit or trigger another app response. Inspect action outcomes rather than treating every Failed run as a failed save. Reconcile a missing notice separately; replay does not resend it.

The intake flow saved successfully. Its checker showed zero errors and nine connection-change warnings directing an app refresh. Studio's intake binding was refreshed, saved, and published, and the published app then submitted successfully with the new Teams action. The earlier screenshot records those warnings accurately.

**Callback** (`dd1d8f67-116c-f55d-7752-7c7d322691c5`) now contains a `Process_notification` scope around the existing claim and guarded delivery scopes. New `mr-created` and `report-no-changes` receipts freeze `teams-self:o9w@ornl.gov`; other validated events freeze `o9w@ornl.gov`. Delivery chooses the frozen route, so previously claimed receipts keep their original destination. Legacy Dataverse logical names `cr1d0_emailrecipient`, `cr1d0_emailstate`, and `cr1d0_emailattemptrun` are retained and now describe the one selected channel. They were not renamed or migrated.

Concurrency remains one. The unique receipt key, strict event schema, exact-payload comparison, Pending → Sending → Sent state, recorded attempt, and uncertain-delivery hold remain in place. Both sends have retries disabled and protected inputs/outputs. The milestone email subject is **ACORN Sponsor Highlights milestone**; synthetic IDs receive a TEST ONLY banner in the body. A separate Teams group/channel requires its own delivery state before migration.

Use [Microsoft's Teams connector documentation](https://learn.microsoft.com/en-us/connectors/teams/) for the signed-in-user self-chat operation and government-cloud constraints. The tested action uses Jason's existing connection; no new credentials or permissions were created.

## Owner's manual milestone walkthrough

Open [ACORN Callback Receipt Validation](https://make.gov.powerautomate.us/environments/221f44a6-1468-ea4e-b93d-cd52f1723079/flows/dd1d8f67-116c-f55d-7752-7c7d322691c5/details). Choose **Run**, enter a stable **Webhook ID** and exact **Event JSON** matching [the schema](../vergil-gitlab-jsonl-automation/_assets/callback-event.schema.json), then inspect the run and receipt. Use the actual event only after its stage occurs. Test fixtures are not evidence of real review or publication.

For exact retry, keep the same ID and JSON bytes. Sent skips delivery. A changed payload conflicts. For Sending/unknown state, inspect the recorded attempt and the Teams chat or Outlook Sent Items before reconciliation; never reset Pending solely because a connector timed out. Preserve the original destination and attempt. External acknowledgement, sender lost-response recovery, and Vergil restart acceptance remain unproven.

## Dated acceptance

| Test | Evidence | Outcome |
| --- | --- | --- |
| Published form + intake notice | `08584087205468981614810029297CU09` | Saved and Teams action succeeded; actual Teams receipt inspected. |
| JSONL append | `03ff5836289447034f40f2a4898c155441e6cb66` | Exactly one added line, zero removals; all previous bytes retained; UUID occurs once. Seven synthetic lines total. |
| Exact intake replay | `08584087203368929940105688293CU25` | Queued replay succeeded; Teams action skipped; remote main remained at the original append commit. |
| Routine callback | `08584087211445944033254400953CU14` | Teams succeeded, email skipped, Mark sent succeeded; actual Teams message inspected. |
| Exact routine callback replay | `08584087207751274720312650353CU31` | Duplicate no send succeeded; delivery branch skipped. Expected duplicate claim conflict was handled; overall run succeeded. |
| Milestone callback | `08584087205548109223433666322CU07` | TEST ONLY failed-event email succeeded; Teams skipped; overall run succeeded. Inbox receipt was not independently inspected. |

The captured form receipt is `82a8dd58-8c60-4ec5-abf5-cb251ab9e033`. `_assets/notifications/confirmed-submission.json` preserves that synthetic envelope; `_assets/notifications/append-check.json` records byte preservation and the snapshot hash. Runtime text and screenshots are retained beside this walkthrough. These tests do not establish reliable GitLab egress, concurrent-writer acceptance, or production licensing.

## Source and restoration limits

`../vergil-gitlab-jsonl-automation/_assets/notifications/process-notification.clipboard.json` is the complete current callback scope; `../vergil-gitlab-jsonl-automation/_assets/notifications/callback-claim.clipboard.json` and `../vergil-gitlab-jsonl-automation/_assets/notifications/callback-delivery.clipboard.json` are its constituent scopes. `_assets/notifications/intake-teams.clipboard.json` is the new top-level intake condition, appended after the existing fallback. Bind existing protected connections in the designer; never embed credentials. Imported top-level run-after settings must be verified explicitly. If the designer drops an expression during import, restore it using the Expression editor and inspect it before saving. The intake message and static email subject were verified through live execution.

This is a source backup, not an importable Power Platform solution. The durable JSONL and Dataverse receipt records remain authoritative; Teams and email are notifications.
