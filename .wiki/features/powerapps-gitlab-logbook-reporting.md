---
type: Plan
title: "Implement GitLab JSONL intake and deploy Power Automate logbook reporting"
description: "Append Logbook submissions from Power Automate to a GitLab JSONL file, poll them from Vergil, and validate reviewed, duplicate-safe reporting."
difficulty: medium
impact: critical
package: [workspace, acorn-lib, acorn-cli, acorn-schema]
tags: [feature, automation, powerapps, powerautomate, gitlab, webhook, logbook, reporting, pdf]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Implement the selected sequence: Power Apps form → Power Automate flow → append to the JSONL Logbook inbox in GitLab → Vergil polls GitLab → the bot durably ingests new items and removes them from the inbox → the bot routes each item by `projectId` and augments the associated project's `index.json`. Use `.acorn/intake/submissions.jsonl` for the pending inbox. Each line wraps form-controlled Logbook fields in the existing version-1 submission envelope. Preserve the established review branch and human-merge policy for canonical changes, followed by PDF publication and notifications. Complete one single-project pilot before expanding mappings or submitters.

This plan is the sole setup reference while work remains active. It records the deployment decisions, stop gates, exact asset boundaries, ordered walkthrough, troubleshooting checks, and acceptance evidence. After acceptance, move the validated operational record to `../assets/powerautomate-logbook-reporting.md` and retire this plan from the active indexes.

# Current evidence

- **Selected transport (2026-10-11):** the user reports testing form data saved as a CSV row and selects GitLab JSONL for the planned intake. Power Automate will append to `.acorn/intake/submissions.jsonl` in the target project; Vergil will poll that file. The CSV test does not establish that the JSONL flow or poller exists. This turn updates planning only; no flow or remote project is changed.
- **Infrastructure:** the user has self-managed servers with self-signed TLS at best and has not provisioned a gateway. Organizational gateway availability is unverified and is not a dependency of the selected design. Power Automate must reach and authenticate to GitLab; Vergil must reach GitLab and the callback endpoint over outbound HTTPS. No Power Automate-to-Vergil route is required.
- **Implementation gap:** `Server::poll_once` in `crates/acorn-lib/src/io/api/gitlab/bot.rs` polls GitLab note events, not repository files. The JSONL reader, append retry contract, durable record outcomes, and deployment configuration remain planned. Existing `FormSubmission::entry` converts validated, authorized form fields into `LogbookEntry`; reuse this contract rather than inventing a second Logbook schema.
- `crates/acorn-lib/src/io/api/powerautomate/` implements strict form decoding, project and submitter authorization, shared-header authentication, durable intake, and direct callback resolution.
- `crates/acorn-lib/src/io/api/webhooks/` provides constant-time secret verification, durable idempotent operations, retryable callbacks, readiness, and graceful drain state.
- `crates/acorn-lib/src/io/api/gitlab/` implements rolling merge-request automation, authorized graduation commands, and immutable report-package publication.
- `crates/acorn-cli/src/commands/report/` composes bounded artifact creation and package publication; `templates/gitlab/reporting.gitlab-ci.yml` supplies the scheduled reporting job.
- `acorn create secret` supports explicit `--paste` clipboard delivery and newline-terminated output only when standard output is redirected or piped.
- Fixture-backed tests validate the 22-entry Power Automate sample, project routing, proposal persistence, graduation, and duplicate-safe operation handling.
- Generated help, focused tests, `make lint`, all 1,914 workspace tests, and `make book` passed on 2026-09-24 for the existing HTTP intake path. Those results do not validate the newly planned JSONL path.
- The former book page and navigation entry are removed, all durable references point to this active plan, and `make book` passes on 2026-10-06.
- The source data project is the Internal `research-enablement/buckets/nssd` project, numeric ID `17410`, default branch `main`. Treat it as read-only production throughout this walkthrough; the inspected snapshot is `036c7b0074b2cc1901c0a65c66c254a04663ce74`.
- The isolated deployment target is the Internal `research-enablement/experiments/power-automate` project, numeric ID `22464`, default branch `main`. Its default branch still contains only its existing README and response CSV; the pilot assets remain isolated on `acorn/powerautomate-pilot`.
- The NSSD snapshot contains 78 tracked files, 35 project `index.json` files, no Git LFS objects, and about 18 MiB of current working-tree data. All 35 indexes pass an offline ACORN schema check; existing readability warnings do not block the copy.
- An exact copy of the NSSD project data is published only to the isolated target's `acorn/powerautomate-pilot` branch at commit `7ccb1a7`. The source README and CI configuration were excluded, the target's existing files were preserved, and a checksum-aware dry run shows no copy drift. No merge request has been opened and the target default branch remains unchanged.
- Pilot commit `66f4bef` adds a cross-project CI include pinned to ACORN commit `aa36e93f76ec22a9b3d2be9ceccf2a8d9cd891aa`; GitLab's pipeline editor reports that the expanded configuration syntax is correct. The schedule-only job remains disabled until the branch is reviewed, merged, and explicitly scheduled.
- Pilot branch commit `ecbbda4` configured `.acorn.json` for project `22464`, mapped `acorn` to `acorn/index.json`, and allowlisted `o9w@ornl.gov`, the address returned by the signed-in Power Apps session. A dry run against that exact commit passed with `no-changes`; no package was published and no callback was sent. Commit `447fbd45084496006a70aa9c0bdd2ef5db4c1b75` adds `c5c@ornl.gov` to the same member allowlist without changing the project mapping or touching production NSSD.
- The local ACORN 0.4.0 debug binary and container tooling are available for setup work. **Vergil is the selected bot host.** The existing `vergil` Docker context reaches an ARM64 Ubuntu 24.04 host running Docker 29.1.3 and the user confirms it can host an ACORN server port. The candidate service origin is `https://vergil:3000`, which the user expects can be exposed on ORNL networks only. DNS resolution, certificate trust, and reachability from an ORNL-managed gateway host are not yet verified; Vergil has no known public HTTPS route, and `kubectl` has no current context. The user confirms that the ORNL enterprise account has Power Automate Premium with HTTP actions.
- Vergil does not currently have an ACORN image. The published `ghcr.io/jhwohlgemuth/acorn:latest` manifest is `linux/amd64` only, the repository Dockerfile builds the `x86_64-unknown-linux-musl` target, and no `aarch64-unknown-linux-musl` GitLab package is available. The pilot therefore needs a native ARM64 build on Vergil or an explicitly reviewed multi-architecture image; do not assume the published image runs natively there.
- ACORN startup resolves `POWER_AUTOMATE_INBOUND_SECRET` immediately, but resolves `POWER_AUTOMATE_CALLBACK_URL` only when delivering a callback. Polling mode does not require a public URL or GitLab webhook credentials, so a bounded first boot can prove `/health` and `/ready` before the callback flow and new JSONL poller exist. Do not submit a real intake until the callback URL is configured.
- The built-in detached-container helper forwards the GitLab token variables but not `POWER_AUTOMATE_INBOUND_SECRET` or `POWER_AUTOMATE_CALLBACK_URL`. For this pilot, run the native process under a managed service or launch a container explicitly with all protected variables; do not use `acorn serve bot --detach` unchanged.
- The published canvas draft loads successfully for the signed-in ORNL user in a GCC Moderate Power Apps environment. The private environment ID is `221f44a6-1468-ea4e-b93d-cd52f1723079`, and the draft app ID is `15ae605b-585b-49f5-a413-2b091f919a35`; do not copy the launch URL or its `hint` query parameter into public documentation. Microsoft documents `https://make.gov.powerapps.us` and `https://make.gov.powerautomate.us` as the GCC maker portals. The earlier environment-loading failure occurred at the commercial Power Automate portal, so it is not evidence that the ORNL account lacks an environment or license.
- The app's Share dialog identifies `221f…` as a developer environment and warns that it is for making and testing rather than production. Publishing created a runnable version but did not initially share it. Adding `c5c@ornl.gov` to the ACORN project allowlist authorized only a later ACORN intake and did not grant Power Apps access or licensing.
- On 2026-10-07, the obsolete generated `Logbooks screen` and its `Logbooks` Dataverse source were removed. After save, publish, and a full editor reload, the tree contains only `SubmitEntryScreen` and the Data pane reports **There is no data in your app**. The earlier Dataverse-backed version was consistent with the tester's Power Apps trial prompt; a new tester launch must confirm the cleaned version's effective license designation.
- The cleaned, published app was shared successfully with the ORNL directory identity **Clark, Niki** (`clarkcn@ornl.gov`), which Power Apps resolved from `c5c@ornl.gov`. Access is **User — Can use the app only**, not co-owner. Power Apps states that recipients are not notified automatically, so the existing clean launch link must be sent separately. The tester subsequently opened the cleaned app successfully without a Power Apps trial prompt. The developer-environment warning remains, and successful pilot access does not make this an approved production environment.
- A read-only review of the published draft on 2026-10-06 found a generated browse/detail/edit app: the Power Apps shell calls it **Sponsor Highlights**, the in-app heading says **Logbooks**, and the contributor can browse existing entries and invoke edit or delete actions. Its New form exposes `identifier`, `timestamp`, `category`, `classification`, `eventType`, `state`, `origin`, `discovery`, `summary`, `items`, and `archived`; all except the timestamp and archived controls are free text. It does not expose a Project choice, signed-in reporter identity, Power Automate receipt, or submission status.
- The draft is a useful schema inventory but is not yet safe as the contributor intake surface. `identifier`, `origin`, `discovery`, and `archived` must not be user-editable or sent by the app; the intake contract derives or excludes them. Category, state, event type, classification, and project must be controlled choices. Existing-entry browse, edit, and delete controls must be removed from the contributor experience or restricted to an explicitly authorized maintainer role. The current `New` affordance and edit, delete, submit, and cancel icons also lack usable accessible button names in the published accessibility tree.
- A canvas update completed and was published on 2026-10-07. It adds a responsive `SubmitEntryScreen`, makes it `App.StartScreen`, and replaces the prior generated `Logbooks screen` as the launch experience. The screen displays fixed project **ACORN**, signed-in name and email, a current-date picker, controlled category/state/event-type dropdowns, a required multiline summary, optional one-item-per-line input, and a conditionally enabled **Submit highlight** button with a visible request status. It omits classification for the pilot and does not expose generated or maintainer-owned fields. The environment-local notification flow is attached as described below, but no secret, callback URL, repository path, HTTP action, or endpoint was added.
- The new contributor screen passes formula checking and was visually verified in the default preview and a 360-by-800 phone preview. App checker reports no formula issues; its 23 accessibility errors all name controls on the pre-existing generated `Logbooks screen`, not the new contributor controls. The environment-local flow is now connected. The submit button remains disabled only while the request is pending or the summary, category, or state is invalid.
- The signed-in Power Apps session evaluates `User().Email` as `o9w@ornl.gov`. Pilot commit `447fbd45084496006a70aa9c0bdd2ef5db4c1b75` authorizes both `c5c@ornl.gov` and `o9w@ornl.gov`; the previous `wohlgemuthjh@ornl.gov` entry remains unauthorized. Do not hard-code one person's address in the reusable canvas.
- The tenant default environment contains a saved instant cloud flow named **ACORN Logbook Intake**, flow ID `e3a4ad29-ac14-4afe-887b-8b905f9e8dbb`. Its Power Apps (V2) trigger defines typed inputs for Project ID, Reporter Email, Entry Date, Category, State, Event Type, Summary, and Items. Flow-owned steps generate one submission GUID, capture `utcNow()`, convert the selected date to a UTC RFC 3339 timestamp, split optional item lines, and filter blank items. It also demonstrates a Teams direct-chat action, an email action, and a Power Apps response with `Notifications sent` plus the submission ID. Flow Checker reports zero errors and zero warnings. It has not been run and cannot be attached to the canvas because the flow and app are in different Power Platform environments; retain it only as the later ACORN-path scaffold. Its fixed pilot recipient values are not the reusable routing contract.
- The environment-local flow **ACORN Logbook Canvas Notifications**, flow ID `18677f6b-21af-435c-a000-a2d471551019`, is saved in the canvas app's `221f…` environment for the notification-only pilot. Its Power Apps (V2) trigger has six typed inputs: Project Key, Requester Email, Occurred On, Highlight Category, Workflow Status, and Highlight Summary. Microsoft Teams posts a direct `Chat with Flow bot` message to the dynamic Requester Email input and includes project, date, category, state, and summary. Office 365 Outlook `Send an email (V2)` sends **ACORN logbook highlight received** to the same dynamic input and includes all six trigger values, including the requester email. Both retain the explicit notification-only/no-ACORN-or-GitLab-write notice. Flow Checker reports zero errors and zero warnings, and Power Automate confirmed the updated flow was saved. The flow contains no fixed personal recipient, HTTP action, ACORN URL, GitLab action, intake credential, or Power Apps response action.
- The environment-local flow is attached to the published canvas. `SubmitButton.OnSelect` passes fixed project `acorn`, `User().Email`, the selected date, category, state, and trimmed summary to `ACORNLogbookCanvasNotifications.Run`; it exposes pending, success, and failure states without clearing the contributor's values. `SubmitButton.DisplayMode` enforces the required summary, category, and state and prevents repeat activation while pending. `SubmissionStatus.Text` announces readiness and the current request state. Power Apps reports no formula errors, App checker reports no formula issues, and the published launch was verified to open `SubmitEntryScreen`. The first explicit test displayed **Notifications sent.** and produced successful Teams and Office 365 Outlook actions in flow run `08584102113387025966515574751CU09`.
- Neither inspected flow has an HTTP action, ACORN URL, or intake credential. These earlier flow observations are distinct from the user's reported CSV test. Identify that tested flow before implementation and verify its target, authentication, and branch; do not assume it is either inspected scaffold.
- Access to the GCC maker portal and instant-flow creation are confirmed. Remaining gates include the chosen flow's GitLab API access and permissions, callback-trigger policy, and named operational owners.

# Architecture decisions

- GitLab holds the pending JSONL Logbook inbox. Power Automate appends records; Vergil durably ingests and removes them, then routes the retained work to the configured project `index.json`. Both writers use optimistic concurrency and recompute their changes after conflicts. Completed submission identities and processing outcomes remain in durable bot storage after removal, so absence from the file does not mean an ID is new. No additional cloud queue is required.
- Use the existing GitLab data project for `.acorn.json`, project `index.json` files, review merge requests, pipeline schedules, and generic-package artifacts. Do not place deployment configuration or operational research data in the ACORN source project.
- Begin with one project mapping and the two explicitly approved tester addresses. Add projects or any other submitters and recipients only after the pilot passes the duplicate-submission smoke test.
- Use an organization-only Power Apps canvas app as the primary intake interface and an instant flow with the Power Apps (V2) trigger as its adapter to ACORN. Microsoft Forms is not required by the ACORN interface.
- Treat the current developer environment as a build-and-test workspace only. The obsolete generated `Logbooks screen` and unused `cr1d0_logbooks` dependency are removed for the notification-only pilot, but confirm the cleaned app's effective license designation with the shared tester. Before broader or production use, move or recreate the app and its environment-local flow in an ORNL-approved shared or production GCC environment with sufficient sharing capacity, then share the app and each dependent resource with the approved users or security group.
- Run ACORN on Vergil as a persistent service with protected credentials and durable state. Keep health/readiness access local for the pilot. Self-signed server TLS on Vergil does not affect outbound requests; remote GitLab and callback certificates must still validate.
- Retain `--event-source poll` for GitLab review events and add an explicitly configured repository-file intake poller. The existing flag alone does not enable JSONL intake. No GitLab webhook registration, public Vergil endpoint, gateway, or relay is required for this design.
- The intake flow uses authenticated GitLab repository-file API calls. Confirm cloud reachability to the actual GitLab instance; a successful browser or local CLI request does not establish Power Automate reachability.
- Direct callbacks use the SAS credential embedded in a Power Automate HTTP trigger URL and require the trigger's legacy `Anyone` mode documented in [OAuth authentication for HTTP request triggers](https://learn.microsoft.com/en-us/power-automate/oauth-authentication). If tenant policy prohibits that mode, use an approved intermediary or open a separate implementation plan for Microsoft Entra service-principal support; do not weaken tenant policy.
- Protect the flow's GitLab write credential and Vergil's GitLab credentials separately. The flow does not need ACORN's HTTP intake secret. Current runtime startup may still require `POWER_AUTOMATE_INBOUND_SECRET`; retain it locally until implementation makes transport-specific requirements explicit.
- Preserve one `submissionId` and the same serialized content across append retries and ambiguous responses. Reusing an ID with different content is a conflict. The JSONL adapter must preserve the existing canonical operation identity when admitting a submission.
- Power Automate owns Teams and email recipients and message formatting. ACORN callbacks contain artifact metadata, never GitLab credentials or PDF bytes.
- A person reviews, graduates, and merges the monthly merge request. The bot updates the review branch and publishes reports but never merges its own changes.

# Public interfaces

## Alternative under consideration: direct project updates from Power Automate

Power Automate could resolve a controlled `projectId` to its configured `index.json`, read that file, append a normalized Logbook entry, and commit with conflict detection. This would replace the pending JSONL inbox and repository-file consumer for intake. It is feasible in principle but not selected or implemented by this planning update.

The flow would then own full Logbook construction, validation, submitter authorization, duplicate detection, preservation of unrelated JSON fields, and read-modify-write retries. Those rules must remain aligned with ACORN's schema; writing a file successfully is not validation. Prefer a review-branch update followed by ACORN checks and human merge. Direct writes to the default branch would change the established review policy and require an explicit decision. Verify concurrent edits and repeated submissions before adopting this alternative.

## Selected JSONL transport

- Selected intake: UTF-8 JSONL at `.acorn/intake/submissions.jsonl` on one explicitly configured GitLab branch. Each nonempty line is one compact version-1 submission object, terminated by LF. The existing `POST /webhooks/powerautomate/forms` remains supported but is not the flow's destination.
- The version-1 form envelope requires `schemaVersion`, `submissionId`, `projectId`, `submittedAt`, `submitter.email`, and an `entry` containing `timestamp`, `category`, `state`, and `summary`. `classification`, `eventType`, and `items` are optional; omit inapplicable fields instead of sending `null` or extra properties.
- Categories are `administration`, `collaboration`, `data`, `engagement`, `funding`, `infrastructure`, `method`, `other`, `personnel`, `publication`, `research`, and `software`.
- States are `completed`, `in-progress`, and `upcoming`; event types are `conference`, `exhibition`, `meeting`, `other`, `presentation`, `tour`, `training`, and `workshop`; classifications are `unclassified`, `confidential`, `secret`, and `top-secret`.
- The flow returns `queued`, `submissionId`, and the GitLab commit SHA after verifying persistence. This is distinct from ACORN acceptance. The bot records admission, rejection, and operation ID separately; its existing callback events report later workflow progress. Do not invent an HTTP `202` receipt from a repository commit.
- Callback events are `mr-created`, `review-needed`, `artifact-ready`, `report-no-changes`, and `failed`. Every callback contains `event` and `projectId`; event-specific fields carry merge-request URLs, package URLs, checksums, cutoffs, or failure messages.
- The existing `.acorn.json` names runtime secret variables, never their values. Repository-file source configuration (project, branch, path, interval, and read limits) is planned and must be documented with implemented field names before deployment. Polling does not require `GITLAB_WEBHOOK_SIGNING_TOKEN`.
- The scheduled reporting job runs only for a default-branch schedule with `ACORN_POWER_AUTOMATE_REPORTING=true`; `ACORN_REPORT_CONFIG` may select a non-default configuration path.

# Multi-project routing model

- The current pilot is intentionally fixed to `projectId` `acorn`; it proves intake, authorization, review, publication, and notification before the project list expands.
- ACORN already supports multiple mappings in `.acorn.json`. Each mapping owns one stable `project_id`, one safe repository-relative `index.json` path, and an allowlist of submitter email addresses.
- After the pilot passes, expose a required, controlled **Project** choice in the organization-only canvas app. The app maps each display label to a stable configured `projectId`; users never supply a repository path or free-form routing identifier.
- The app-triggered flow sends the selected `projectId` and `User().Email` from the signed-in ORNL session. ACORN rejects an unknown project or a submitter who is not a member of that project before any repository write; app filtering is user experience, not the authorization boundary.
- The GitLab bot resolves the authorized mapping and publishes the entry only to that mapping's `path`, on a project- and month-specific review branch. The `projectId` remains attached through graduation, callbacks, and report packaging.
- Routing is explicit rather than inferred. A user who works on several projects selects the related project in the app; the current implementation does not guess a project from email or logbook prose.
- The HTTP action makes the app-triggered flow premium. Before sharing beyond the pilot, every invoking user must have suitable Premium rights or ORNL must assign a Power Automate Process license to the flow; a shared service account is not a licensing workaround.

The callback flow uses this request schema:

```json
{
  "type": "object",
  "properties": {
    "checksum": { "type": "string" },
    "event": {
      "type": "string",
      "enum": [
        "artifact-ready",
        "failed",
        "mr-created",
        "report-no-changes",
        "review-needed"
      ]
    },
    "mergeRequestUrl": { "type": "string" },
    "message": { "type": "string" },
    "packageUrl": { "type": "string" },
    "projectId": { "type": "string" },
    "through": { "type": "string" }
  },
  "required": ["event", "projectId"]
}
```

# Asset boundaries and integration locks

- **This update:** planning only, limited to this concept and its index, network, and execution-lane entries. No live flow, GitLab file, runtime, or source implementation is changed.
- **Production source project:** `research-enablement/buckets/nssd` is a read-only input. Do not commit, push, register webhooks, create variables or schedules, or run write-capable automation against it.
- **Isolated GitLab data project:** `research-enablement/experiments/power-automate` owns the copied data, `.acorn.json`, `.gitlab-ci.yml`, and proposed `.acorn/intake/submissions.jsonl`. Power Automate appends; Vergil removes durably ingested records. The bot augments only the `index.json` selected by the trusted project mapping, through the established review workflow.
- **Power Platform:** one organization-only Power Apps canvas app, one app-triggered intake cloud flow, and one callback notification cloud flow.
- **Managed runtime:** one ACORN service with protected GitLab credentials, callback URL, persistent operation/outcome storage, and any secret still required by existing startup. No webhook signing token is required for polling.
- **GitLab control plane:** approved intake-branch write permissions, protected CI variables, and one monthly default-branch reporting schedule. Configure intake-only commits so they do not trigger unintended reporting jobs.
- **Future implementation:** repository-file polling, source configuration, durable admission/outcomes, and tests are required. Before touching source, declare the exact files and coordinate locks on GitLab bot dispatch, Power Automate admission/configuration, durable store migrations, and CLI configuration with other active plans. This planning update authorizes no implementation.
- After the deployed acceptance criteria pass, convert the verified process and redacted evidence requirements into `.wiki/assets/powerautomate-logbook-reporting.md`, redirect dependent wiki plans to that asset, delete this plan, and remove its index, network, and lane entries.

# Dependencies and stop gates

## Next steps with the available servers

1. Finish the Vergil bot health, readiness, and isolated GitLab access checks in polling mode.
2. Test a policy-approved Power Automate callback flow from Vergil over outbound HTTPS and confirm Teams/email delivery. Store the callback URL securely before real intake.
3. Identify the user's tested CSV flow and verify its GitLab project, branch, credential, and cloud reachability. Plan its JSONL replacement using the existing Logbook submission envelope.
4. Implement and test the repository-file poller, then configure the flow to append JSONL. Preserve submission UUIDs across retries, durably record admission, and distinguish queued receipts from ACORN acceptance. This is planned implementation, not a gateway deployment.
5. Once transport is validated, run one isolated submission through human review, graduation, merge, report publication, callback delivery, and duplicate replay as described in steps 7–9.

The selected route is Power Automate → GitLab → Vergil. Verify reachability and TLS trust on these actual connections; direct HTTP intake on Vergil is not part of this walkthrough.

## Gates for full intake

Do not enable full intake or production flows until all of these gates pass. A protected local intake credential is permitted for the bounded bot boot. The notification-only canvas pilot is explicitly allowed before gates 2 and 3 because it does not call ACORN, GitLab, or an HTTP trigger. An isolated callback test may proceed before gate 2 once callback policy and recipients are approved:

1. Power Automate Premium with HTTP actions and maker access to a GCC Moderate Power Apps developer environment are confirmed for `o9w@ornl.gov`; this proves build capability, not a deployable shared environment or end-user entitlement. The notification-only app no longer has its Dataverse source, is shared with the one approved tester, and the tester has launched the cleaned version without a trial prompt. Confirm that she can establish the required Teams and Office 365 Outlook connections and complete one notification-only submission. Before broader use, obtain an ORNL-approved shared or production GCC environment with sufficient sharing capacity and verify the resulting app license designation and dependent-resource access. Use the GCC maker portals, not the commercial endpoints.
2. Power Automate can read and update the selected GitLab file, Vergil can read it, and the new JSONL poller passes its acceptance tests. Keep the existing app notification-only until the complete intake path is ready. Callback testing and local bot checks may proceed independently.
3. The tenant permits the callback trigger's `Anyone` mode, or an approved intermediary exists for callbacks.
4. The isolated GitLab data project supplies its numeric project ID, default branch, package permission, one safe repository-relative `index.json` path, one stable pilot identifier, and the explicitly approved tester email allowlist. No deployment credential or integration may target the production source project.
5. Named owners exist for the Vergil ACORN service, Power Apps canvas app and flows, GitLab review and merge, Teams destination, and email recipient list.

The implemented webhook runtime, GitLab automation, reporting-window selection, PDF renderer, package manifest, and reporting CI template are otherwise sufficient for the pilot.

# Human and Codex responsibilities

| Owner | Responsibilities |
| --- | --- |
| **You** | Identify the tested CSV flow; confirm GitLab access, intake branch, trusted writers, and credentials; coordinate the shared GCC environment and licensing; approve notification destinations; publish the accepted canvas draft; review, graduate, and merge GitLab changes. |
| **Codex** | Maintain the private plan; prepare and validate GitLab branches, `.acorn.json`, CI, controlled project mappings, flow expressions, schemas, service manifests, and non-production tests; keep NSSD read-only; present exact human instructions at each handoff. |
| **Together** | Confirm the project roster and member allowlists; place generated credentials into protected stores without exposing them; run the end-to-end and duplicate-submission smoke tests; review redacted acceptance evidence. |

Codex stops at actions that require the user's enterprise authority, policy choice, secret entry, representational approval, or human review. Each such handoff must state the exact expected result before automated work resumes.

# Deployment walkthrough

## 1. Freeze the pilot record

Use GitLab project `22464` (`research-enablement/experiments/power-automate`), pilot `projectId` `acorn`, canonical path `acorn/index.json`, and testers `c5c@ornl.gov` and `o9w@ornl.gov`. Keep initial setup on `acorn/powerautomate-pilot` until its data/configuration is reviewed and merged. Explicitly record the inbox branch for both writer and reader; move to `main` only after review and coordinated reconfiguration. The older pinned ACORN commit is HTTP-path evidence only; choose a reviewed revision containing the new poller before deployment. Record Teams/email recipients and owners. Exclude production project `17410`.

- **Verify:** every value has an owner and storage location, and none is still represented by an example such as `12345`, `pilot`, or `example.org`.

## 2. Refine the existing Power Apps canvas draft

Reuse the published GCC Moderate draft rather than creating another app. Rename it consistently to **ACORN Logbook** (or another approved single name) and make a dedicated contributor submission screen the pilot's entry point. The current gallery and detail screen may remain only as a separately authorized maintainer experience; ordinary contributors must not receive direct edit or delete controls for existing entries.

Build the contributor screen around this field ownership:

| Surface | Control and behavior |
| --- | --- |
| Project | For the pilot, display **ACORN** as read-only and pass the fixed `acorn` identifier. After pilot acceptance, replace it with the approved controlled Project choice; never accept a repository path or free-form project identifier. |
| Reporting as | Display `User().FullName` and `User().Email` read-only. Pass the signed-in email to the flow; the user cannot override it. |
| When | Default to the current local date and time. The flow converts it to UTC RFC 3339 before serializing the JSONL submission. Do not leave the generated `12/31/2001 00:00` placeholder as the effective default. |
| Category | Required choice limited to the ACORN category values under Public interfaces. |
| State | Required choice limited to `completed`, `in-progress`, and `upcoming`; choose an explicit pilot default only if the owner approves it. |
| Event type | Optional choice limited to the ACORN event-type values plus an unselected state. |
| Classification | Optional choice limited to the ACORN classification values plus an unselected state. Omit this control entirely if the pilot has no classification use case. |
| Summary | Required multiline plain text with a visible character limit and inline error. |
| Items | Optional multiline plain text with one item per line; the flow removes blank lines and sends a JSON string array. Do not ask contributors to author JSON. |
| Submit | Call the Power Apps (V2) flow once, disable repeat activation while pending, and show queued status, submission ID, and commit receipt after verified GitLab persistence. Preserve values and retry identity on failure. ACORN acceptance and review happen later. |

Do not expose `identifier`, `origin`, `discovery`, or `archived` on the contributor screen. ACORN generates the identifier, the intake adapter sets the manual origin, the public intake contract does not accept discovery, and new submissions are not archived. Keep the shared secret, callback URL, and repository path out of all app formulas and controls.

Give every input a persistent label, mark required fields in text as well as color, set a logical tab order, and assign accessible labels to New, Submit, Cancel, Edit, and Delete if any of those controls remain. A submission status must be announced as text, not only by icon or color.

- **Published evidence (2026-10-07):** `SubmitEntryScreen` implements the pilot field set and controlled values, opens first, and calls the environment-local notification flow with six inputs. Required-field validation, pending-state disabling, success/failure status text, and a current Teams-and-email tooltip are active. Default and 360-by-800 previews render the complete form with a scrollable body and fixed status/footer. The pilot ACORN allowlist contains the signed-in `User().Email` value plus `c5c@ornl.gov`; that allowlist is independent of the Power Apps grant. The obsolete generated screen and Dataverse source are removed, the saved Data pane is empty, and the cleaned version is published. Power Apps successfully granted Clark, Niki user-only access after resolving `c5c@ornl.gov`, and she opened the clean launch without a trial prompt. Her notification submission confirmation remains pending.

- **Verify:** the shared tester opens the clean launch link without a Power Apps trial prompt, sees the contributor screen and her signed-in ORNL identity, establishes the required flow, Teams, and Office 365 Outlook connections, and can submit the notification-only pilot. In an approved shared or production GCC environment, explicitly share the published app only with approved testers, confirm every tester has the required entitlement for the app's final license designation, and ensure all dependent resources are accessible. The app contains no shared secret, callback URL, or repository path; a blank form defaults to the current time, cannot submit an invalid controlled value, has no ordinary-contributor edit or delete path, and exposes meaningful accessibility names for every actionable control.

## 2a. Prove the canvas-to-notification path

Before introducing Vergil, GitLab, HTTP actions, or credentials, create **ACORN Logbook Canvas Notifications** in the same `221f…` Power Platform environment as the canvas app. Use the Power Apps (V2) trigger with the six reduced pilot inputs already represented by the contributor screen: fixed project key, signed-in requester email, selected date, category, state, and summary. Use that Requester Email input as the recipient for both the direct Teams chat through Flow bot and the Office 365 Outlook email, then let the canvas show a short success status when the flow call completes. Never hard-code a tester address into either notification action. This is a transport demonstration, not an ACORN intake: it deliberately omits event type, items, submission persistence, GitLab routing, and all secrets. Reserve a structured Power Apps response for the GitLab queued receipt and submission ID.

Connect that environment-local flow to **Submit highlight**. Disable the button while the call is pending, validate the required fields, display success or failure in `SubmissionStatus`, and preserve the entered values on failure. Keep the app unpublished until the tester approves the draft behavior.

- **Pilot evidence (2026-10-07):** flow `18677f6b-21af-435c-a000-a2d471551019` and the canvas wiring are saved. Both notification actions route through the dynamic Requester Email input, Flow Checker reports zero errors and zero warnings, and App checker reports no formula issues. One explicit click from the unpublished canvas completed run `08584102113387025966515574751CU09` in three seconds. The Power Apps trigger, Teams direct-chat action, and Office 365 Outlook email action each report **Succeeded**, and the canvas displayed **Notifications sent.** No HTTP, ACORN, GitLab, or NSSD action exists in the run. After that run, the saved message templates were enriched: Teams now includes project, date, category, state, and summary; email includes all six trigger fields. A new published-app submission is required to validate the enriched content and visible delivery in the tester's clients.

- **Verify:** one explicit test click from the unpublished canvas produces exactly one Teams direct-chat message and one email to the signed-in submitter identified by Requester Email, the canvas reports success, and no request reaches Vergil, GitLab, or the production NSSD project. Record the flow run ID and redacted screenshots without copying message content or addresses into public documentation.

## 3. Configure GitLab credentials

The flow needs an approved credential capable of reading and updating the JSONL file in project `22464` on the configured intake branch. Store it in the protected connection or secret facility and enable Secure Inputs/Outputs on credential-bearing actions. Vergil uses its own protected GitLab credentials and durable operation store. Keep tokens out of the canvas, JSONL, ordinary Compose actions, logs, and repository configuration.

The flow does not use ACORN's inbound HTTP secret. The current runtime may still require `POWER_AUTOMATE_INBOUND_SECRET` at startup; provision it locally if needed for the bounded boot:

```bash
POWER_AUTOMATE_INBOUND_SECRET="$(acorn create secret)"
export POWER_AUTOMATE_INBOUND_SECRET
```

- **Verify:** the flow can read/write the isolated intake branch and Vergil can read the same source. Unauthorized credentials fail; production project `17410` is excluded.

## 4. Create the callback notification flow

This can be tested independently of the unavailable inbound route after callback policy and recipient approval are confirmed. Vergil initiates the request to Power Automate; Power Automate does not connect to Vergil for this test. Verify outbound DNS, HTTPS, and certificate validation from the actual service environment.

Create an automated flow with **When an HTTP request is received**, the callback schema under Public interfaces, and the policy-approved trigger mode. Add a Switch on `event`. Send both Teams and email messages for `mr-created`, `review-needed`, `artifact-ready`, and `failed`; record `report-no-changes` and only announce it if operators want the noise. Ensure accepted callbacks receive a successful 2xx response.

Save the flow, copy the complete generated URL including its `sig` query parameter, and store it as the protected `POWER_AUTOMATE_CALLBACK_URL` runtime value. Enable Secure Inputs on the trigger and Secure Inputs and Secure Outputs on actions that can reveal callback content.

- **Verify:** a schema-valid test callback reaches the intended Teams channel and email recipients, the flow returns 2xx, and neither the body nor SAS URL is visible in ordinary run-history details.

## 5. Configure the GitLab data project and ACORN service

Add the following `powerAutomate` section to the data project's `.acorn.json` with the resolved project values. Keep the optional timezone absent or set to `UTC`; no other reporting timezone is supported.

```json
{
  "powerAutomate": {
    "callback_destination": "powerautomate",
    "variables": {
      "callback_url": "POWER_AUTOMATE_CALLBACK_URL",
      "inbound_secret": "POWER_AUTOMATE_INBOUND_SECRET"
    },
    "gitlab_project": "22464",
    "package": "acorn-automated-artifacts",
    "projects": [
      {
        "members": ["c5c@ornl.gov", "o9w@ornl.gov"],
        "path": "acorn/index.json",
        "project_id": "acorn"
      }
    ]
  }
}
```

Deploy the service with protected runtime credentials on Vergil. First use a native ARM64 ACORN binary and polling mode to prove the process and configuration without registering a webhook. `POWER_AUTOMATE_CALLBACK_URL` may remain absent for this health/readiness-only boot; do not send a real intake while it is absent.

```bash
acorn serve bot 22464 \
  --bind 0.0.0.0:3000 \
  --config .acorn.json \
  --event-source poll
```

- **Verify the bounded boot:** local requests to `/health` return `ok`, `/ready` returns HTTP 200, and startup logs contain no secret values. Stop here if the executable is not native ARM64 or the configuration names production project `17410`.
- **Verify the selected integration route:** the actual Power Automate run can authenticate to GitLab, and Vergil can read the same committed JSONL file and deliver outbound callbacks. No incoming cloud connection to Vergil is part of this test.

## 5a. Implement repository-file polling (planned)

Add an explicitly configured source with the isolated GitLab project ID, exact branch, `.acorn/intake/submissions.jsonl`, poll interval, and bounded file/record sizes. Record the final configuration field names and exact source write set before implementation; the example above describes existing configuration only.

- Resolve the branch head and read JSONL at that immutable commit. Retain the complete payload plus source project, branch, path, commit, record location, submission ID, and content digest for audit and recovery.
- Parse records independently as `FormSubmission`; validate Logbook fields, project membership, and report cutoff before admission to the existing durable operation queue. Restrict intake-branch writers to trusted producers; the submitted email alone does not authenticate the human. The flow must derive reporter identity from its authenticated app context.
- Persist outcomes and deduplication across restarts. Track trusted source plus submission ID and content digest; identical retries reuse the canonical operation, while changed content under the same ID is rejected. Reconcile existing operation identity across transports before enqueueing.
- Persist the complete work item and its stable identity in the durable operation queue before removing it from GitLab. A crash between admission and removal must replay safely. Distinguish admitted, completed, permanently rejected, and retryable failure; do not treat removal as successful `index.json` augmentation.
- Remove only exact ID/content matches already durably admitted. Read the latest file and use its `last_commit_id` for the cleanup commit. On a competing append or cleanup, re-read and recompute the remaining lines; preserve new submissions, unadmitted records, and malformed lines byte-for-byte. Keep an empty JSONL file when drained rather than deleting it. Retry uncertain cleanup safely; never overwrite with a stale remainder.
- Route each admitted item using the configured `projectId` → repository-relative `index.json` mapping and augment that project's Logbook through the existing review workflow. Unknown/unauthorized IDs cannot choose arbitrary paths. Once removed from the inbox, processing failures retry from durable bot storage; a permanently failed item remains visible there for operator recovery.
- Record malformed complete lines as rejected and continue with other lines. Never silently discard a nonterminated tail. Transient GitLab, storage, or admission failures remain retryable without advancing past unrecorded work.
- Treat commit cursors and line positions as optimizations, not deduplication identities: bot cleanup legitimately shrinks the file. Recognize recorded cleanup commits, retain ID/content receipts after removal, and surface unexplained removals or changed content. Do not prune durable payloads until processing completes and recovery/retention requirements are met. Fail visibly at configured size limits.
- Verify that polling-mode review commands execute authorized graduation. Retrieving or logging a GitLab note alone is not sufficient evidence.

Tests must cover escaped newlines/quotes, Unicode, optional arrays, invalid/unauthorized records alongside valid ones, unchanged polls, duplicate/conflicting IDs before and after removal, and restart/crash replay before admission, after admission, after removal, and after project updates. Include append-versus-cleanup races, empty-file draining, GitLab failures, size limits, and project routing isolation. Run focused tests and repository lint/test gates after implementation.

## 6. Append Logbook submissions to GitLab JSONL (planned)

Identify the user's tested CSV flow and reuse its verified GitLab connection where applicable. Keep the app-triggered flow in the canvas app's environment. Replace CSV serialization with compact JSON objects appended to `.acorn/intake/submissions.jsonl` on the explicitly selected pilot branch in project `22464`.

Each line uses the existing version-1 `FormSubmission` envelope. Its nested `entry` models the contributor-owned fields of `LogbookEntry`; ACORN derives `identifier` and owns `origin`, `discovery`, and `archived`. Use typed Power Apps (V2) inputs, UTC RFC 3339 timestamps, and optional fields only when supplied. The expanded object below documents the fields; serialize it compactly onto one physical line followed by LF. A JSON serializer must escape embedded newlines and quotes rather than interpolating raw form text.

```json
{
  "schemaVersion": 1,
  "submissionId": "<Submission ID output>",
  "projectId": "acorn",
  "submittedAt": "<UTC RFC 3339 timestamp>",
  "submitter": {
    "email": "<submitter email>"
  },
  "entry": {
    "timestamp": "<UTC RFC 3339 timestamp>",
    "category": "<allowed category>",
    "eventType": "<optional allowed event type>",
    "items": ["<optional item>"],
    "state": "<allowed state>",
    "summary": "<app summary>"
  }
}
```

Planned append algorithm:

1. Validate inputs and generate the UUID and submission timestamp once. Freeze the payload for the entire append attempt and all retries; retain it for recovery after timeouts.
2. Serialize flow runs writing this file. Read content and `last_commit_id` using the [GitLab Repository Files API](https://docs.gitlab.com/api/repository_files/). Treat only a confirmed missing file as initial creation, never authentication or network failure.
3. Check for the UUID before appending. Identical content means the earlier write succeeded; different content is a visible conflict. If Vergil already removed the record, absence does not prove the append failed: use the saved commit receipt when available, otherwise retry with the same UUID/content and let durable bot deduplication reconcile it.
4. Preserve existing bytes and append one object plus LF. Reject a malformed existing tail rather than silently repairing it. Update using the observed `last_commit_id` for conflict detection.
5. On a concurrent append, bot cleanup, initial-create race, or ambiguous response, re-read, check the UUID, and recompute against current content. Bound retries and backoff. Never retry a stale whole-file replacement blindly; preserve the ID when reporting exhausted retries or uncertain persistence.
6. Verify persistence and return `queued`, `submissionId`, and GitLab commit SHA. The app says “Saved to GitLab; awaiting ACORN processing.” The bot records acceptance or rejection later; a GitLab commit is not an ACORN HTTP acceptance receipt.
7. Protect GitLab request/response content in run history and retain redacted run/commit evidence.

- **Draft evidence (2026-10-07):** the eight-input default-environment scaffold prepares UUIDs, timestamps, and optional item arrays but cannot connect to the canvas across environments. The environment-local six-input flow only sends notifications; if selected, it needs optional event type/items wired. Inspect the user's separately reported CSV flow before choosing the implementation target. JSONL append and consumption are not yet implemented or tested.

- **Verify:** simultaneous appends and bot cleanup lose no work; retrying an append after its record was consumed produces no duplicate project update; quotes, multiline summaries, Unicode, and arrays round-trip. A queued receipt refers to confirmed commit persistence even if the bot has since removed the row. Invalid records never change a canonical `index.json`.

## 7. Exercise human review and graduation

Submit one response from an allowlisted tester. Confirm the JSONL append, durable bot admission, and removal of that exact record without disturbing other submissions. Confirm routing by `projectId` augments only the associated `index.json` on `acorn/powerautomate/<project>/<year-month>` and creates or updates its draft MR. Review the entry, request corrections through GitLab discussions if necessary, and add `/acorn graduate` as a user with at least the Developer role. Inspect the updated branch, approve it, and have a person merge it.

- **Verify:** the graduated entry retains the envelope's project identifier and only reviewed content reaches the default branch.

## 8. Enable scheduled artifact publication

Because the data project is separate from ACORN, use GitLab's [cross-project include](https://docs.gitlab.com/ci/yaml/includes/#includeproject) and pin it to the reviewed ACORN tag or commit:

```yaml
include:
  - project: research-enablement/acorn
    ref: <pinned-acorn-tag-or-commit>
    file: templates/gitlab/reporting.gitlab-ci.yml
```

Add protected, masked callback and GitLab credentials. Test report generation without publication or callbacks:

```bash
env CI_SERVER_HOST=code.ornl.gov acorn report \
  --config .acorn.json \
  --commit <exact-commit-sha> \
  --root . \
  --through <UTC-month-cutoff> \
  --output .acorn/reports \
  --dry-run
```

Then create a [monthly default-branch schedule](https://docs.gitlab.com/ci/pipelines/schedules/) shortly after the start of the UTC month with `ACORN_POWER_AUTOMATE_REPORTING=true`. The scheduled job publishes through the [GitLab generic package registry](https://docs.gitlab.com/user/packages/generic_packages/).

- **Verify:** the dry run produces the expected local report set without package upload or callback; the scheduled job is absent from push and merge-request pipelines.

## 9. Complete the duplicate-safe end-to-end smoke test

Run the scheduled reporting job manually. Confirm the GitLab generic package contains the immutable manifest and expected PDF, then confirm `artifact-ready` reaches both Teams and email with the package link, checksum, and cutoff. Retry the original append with the same UUID and content, then restart the bot and reread the same snapshot.

- **Verify:** the replay returns the same operation as a duplicate, creates no second entry, and the complete pilot produces one reviewed entry, one package version, and one accepted artifact notification event.

## 10. Expand to controlled multi-project routing

After the single-project pilot passes, the human owner supplies the approved project labels, stable identifiers, repository paths, and member email allowlists. Codex validates every path against the isolated repository and extends `.acorn.json` on a review branch. The human owner approves the required Project choice in the organization-only canvas app, and Codex supplies the label-to-`projectId` mapping used by the app and flow. No default branch may accept an unknown label. Before sharing, the human owner confirms that each user has suitable Premium rights or that the flow has a Process license.

Submit authorized test entries for two different projects and one deliberately unauthorized project/member combination. Review the resulting branches and diffs before merging any entry.

- **Verify:** each authorized entry changes only its configured `index.json`, uses a distinct project/month review branch, and retains the selected `projectId`; the unauthorized combination produces no project-data change or review MR; its inbox record and durable rejection remain inspectable.

# Troubleshooting during the walkthrough

| Symptom | Check |
| --- | --- |
| JSONL record is rejected | Check envelope fields, controlled values, mapping, trusted writer, allowlisted identity, timestamps, nulls, and extra properties. |
| GitLab returns `401` or `403` | Check the protected GitLab credential, API permissions, and branch protection. |
| Append fails or times out | Re-read and check the UUID before retrying with current content and `last_commit_id`; preserve the original payload. |
| File exists but no MR appears | Confirm the new repository-file poller is installed and configured for the same project/branch/path; inspect durable rejection/admission records. Existing note polling alone does not read JSONL. |
| Entry timestamp is rejected | Check whether the entry predates the latest closed reporting cutoff. |
| Callback repeats or returns `401`/`403` | Check for a successful 2xx response, the trigger's approved mode, and the complete current SAS URL including `sig`. |
| Package contains only a manifest | Confirm whether the reporting interval contains any graduated entries; a no-change interval intentionally omits the PDF. |
| `/acorn graduate` has no effect | Check command spelling, commenter role, polling progress, and actual command dispatch; a retrieved note is not proof of execution. |

# Validation evidence

- **2026-10-11 transport status:** the user selects a GitLab JSONL inbox based on a reported CSV-row test. JSONL append, repository polling, and the full workflow remain unimplemented/unvalidated. Gateway availability is not a prerequisite. Only this plan and linked wiki records were updated.
- The 2026-10-07 canvas version is published. The live launch confirms `SubmitEntryScreen` is the start screen; preview also verifies it at the default canvas size and at 360 by 800. The controlled category list contains only the placeholder plus the twelve documented categories. The environment-local notification flow is attached, the blank form keeps **Submit highlight** disabled, the tooltip is `Submit highlight to Teams and email`, and the status is `Ready to send Teams and email notifications.`
- App checker reports zero formula issues for the saved draft. Its 21 missing-label and two missing-tab-stop errors are confined to named controls on the pre-existing generated `Logbooks screen`; no new contributor control appears in those findings.
- Notification-only run `08584102113387025966515574751CU09` completed successfully in three seconds on 2026-10-07. Its Power Apps trigger, Microsoft Teams direct-chat action, and Office 365 Outlook email action all report **Succeeded**, and the canvas displayed **Notifications sent.** The run contains no HTTP, ACORN, GitLab, or NSSD action. Record human confirmation that the Teams chat and email are visible before treating delivery as accepted.
- The enriched Teams and email templates were saved after Flow Checker reported zero errors and zero warnings. No test was triggered during the template update; the next tester submission must confirm the expected field values and formatting in both delivered messages.
- Record screenshots or redacted exports showing the premium HTTP action and callback trigger policy without exposing credentials.
- Record health/readiness, the flow's queued receipt and commit SHA, and the bot's durable admission/rejection outcome and operation ID.
- Record the GitLab branch and merge-request URLs, authorized graduation note, merged commit, pipeline URL, package version, manifest, and PDF checksum.
- Record the Teams and email deliveries for `artifact-ready` and the callback flow's successful 2xx run.
- Record the duplicate replay response and confirm the data-project diff contains only one normalized entry.
- Confirm generated secrets, shared-secret headers, callback URLs, SAS query parameters, and GitLab tokens are absent from repository history, service logs, artifacts, and visible flow histories.
- Historical documentation builds cover the earlier walkthrough only. Run focused poller/append tests and repository lint/tests after implementation; document live append, concurrency, restart, and callback evidence separately.

# Acceptance criteria

- Power Automate appends one compact, schema-valid Logbook submission envelope per line to `.acorn/intake/submissions.jsonl`. Concurrent appends and bot removals lose no submissions; retries after removal produce no duplicate processing effects.
- Vergil reads immutable snapshots, durably ingests complete items before removing exact matches from the inbox, and routes each by `projectId` to augment only its configured `index.json`. Rejections stay visible, admitted payloads survive removal, and restart/crash replay produces no duplicate effects.
- All licensing, GitLab reachability/authentication, callback policy, ownership, and pilot-mapping gates pass. No gateway or public Vergil endpoint is required.
- One organization-only Power Apps canvas app and two protected Power Automate flows implement the documented intake and callback contracts.
- The Vergil service remains healthy and ready, authenticates the configured source and review events, and preserves durable receipts after inbox cleanup. Failed project updates remain retryable from bot storage.
- Unauthorized writes/records fail closed. The app distinguishes GitLab persistence from ACORN admission; one valid queued submission produces one expected monthly review entry and merge request.
- Human graduation preserves the routed project identifier, and a person—not the bot—merges the reviewed entry.
- The scheduled job publishes one immutable report package and Power Automate sends both Teams and email notifications for the accepted artifact.
- Replaying the same submission UUID is idempotent and does not create a second entry, merge request operation, package, or notification event.
- Before multi-project rollout, two authorized project selections route only to their configured `index.json` files and an unauthorized project/member combination creates no canonical project-data change.
- No credential appears in source control, logs, artifacts, app controls, ordinary Compose actions, or visible Power Automate run history.
- The redacted evidence set is sufficient for another operator to verify every acceptance criterion; the verified process can then transition to `.wiki/assets/powerautomate-logbook-reporting.md`, durable references can be redirected, and this plan and its active index entries can be deleted.

# Execution phases

## PA-10 — Scope the JSONL implementation phases

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: selected JSONL transport contract above
- Exact write set: this plan, `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: JSONL intake planning
- Outcome: Verify current code and add separately bounded implementation phases for snapshot reading/configuration, durable admission/replay, conflict-safe cleanup, and integration validation. Give each an exact file write set and coordinate the existing GitLab, admission, store, and CLI owners before implementation.
- Focused validation: Trace each transport acceptance criterion to a phase and verify that no phase exceeds five active hours or assumes the existing note poller consumes repository files.
- Handoff: Replace PA-10 with the scoped phases using fresh stable IDs. PA-05 remains blocked until those phases implement and validate the JSONL path; service boot and callback checks can proceed independently.

## PA-01 — Verify the published notification pilot

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: tester availability in the `221f…` Power Platform environment
- Exact write set: Power Apps app `15ae605b-585b-49f5-a413-2b091f919a35`, flow `18677f6b-21af-435c-a000-a2d471551019`, this plan
- Primary integration lock: Power Platform pilot
- Outcome: Record one tester-visible enriched Teams and email delivery from the published canvas.
- Focused validation: Inspect the flow run, app status, and redacted tester evidence.
- Handoff: Remove PA-01 and make PA-02 ready when delivery is proven.

## PA-02 — Resolve production owners and access gates

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-01
- Exact write set: Power Platform environment policy, isolated GitLab project `22464` access policy, this plan
- Primary integration lock: deployment approvals
- Outcome: Name owners and record approved environment, GitLab API access from the selected flow, callback-trigger, licensing, branch permissions, and secret-storage decisions. Identify the user's tested CSV flow before selecting the JSONL flow target. No gateway approval is required.
- Focused validation: Verify every stop gate has an accountable owner and dated evidence.
- Handoff: Remove PA-02 and record the approved GitLab target and outbound access requirements for PA-03 and PA-04.

## PA-03 — Prove a native ARM64 Vergil service boot

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-02
- Exact write set: Vergil managed service definition, protected Vergil runtime environment, this plan
- Primary integration lock: Vergil service
- Outcome: Run the pinned native ARM64 ACORN service in polling mode with healthy readiness and redacted logs.
- Focused validation: Check `/health`, `/ready`, process supervision, architecture, and absence of secret values.
- Handoff: Remove PA-03 and record the verified service revision and outbound GitLab access. Healthy note polling does not establish JSONL intake readiness.

## PA-04 — Provision GitLab and callback secrets

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-02
- Exact write set: selected intake flow's protected GitLab credential, Vergil's protected GitLab credential and runtime environment, protected `POWER_AUTOMATE_CALLBACK_URL`, callback flow
- Primary integration lock: Power Automate credentials
- Outcome: Establish separate protected GitLab credentials for the flow and bot, plus one schema-valid callback flow. Keep any startup-required `POWER_AUTOMATE_INBOUND_SECRET` local to Vergil; the JSONL flow does not use it.
- Focused validation: Send a redacted test callback and inspect 2xx, Teams, email, and secure run-history behavior.
- Handoff: Remove PA-04 and provide only secret references to PA-05.

## PA-05 — Connect the GitLab JSONL intake flow

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: PA-03, PA-04, implementation and validation phases scoped by PA-10
- Exact write set: selected Power Apps intake flow, canvas submit action, isolated project `22464` inbox `.acorn/intake/submissions.jsonl`, Vergil JSONL source configuration
- Primary integration lock: intake route
- Outcome: Append the strict version-1 envelope with conflict-safe GitLab retries, configure the implemented repository-file poller, and return a confirmed queued receipt with submission ID and commit SHA.
- Focused validation: Verify stable UUID/content across retries, concurrent append and cleanup, ambiguous responses, serialization round-trips, protected credentials, and app distinction between GitLab persistence and ACORN admission.
- Handoff: Remove PA-05 and record the tested flow revision, source project/branch/path, and poller revision.

## PA-06 — Create the first duplicate-safe review operation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-05
- Exact write set: isolated GitLab project `22464`, Power Automate intake run, this plan
- Primary integration lock: isolated pilot repository
- Outcome: Submit one authorized response and create exactly one routed branch and draft merge request.
- Focused validation: Inspect the queued receipt, durable admission and operation ID, exact-record inbox removal, routed branch path, proposed entry, callback, and duplicate-safe persistence across restart.
- Handoff: Remove PA-06 and pass the merge request to human review.

## PA-07 — Graduate and merge the reviewed entry

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-06
- Exact write set: isolated GitLab merge request, `acorn/index.json`
- Primary integration lock: pilot merge request
- Outcome: Complete human review, authorized graduation, approval, and human merge while preserving project identity.
- Focused validation: Compare the merged entry with the accepted submission and reviewed diff.
- Handoff: Remove PA-07 and record the exact merged commit for reporting.

## PA-08 — Publish and replay the single-project pilot

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-07
- Exact write set: project `22464` schedule, generic package `acorn-automated-artifacts`, callback flow
- Primary integration lock: scheduled artifact publication
- Outcome: Publish one immutable report package, deliver `artifact-ready`, and prove replay creates no duplicate effect.
- Focused validation: Inspect manifest, PDF, checksum, cutoff, callback, replay disposition, and package count.
- Handoff: Remove PA-08 and preserve the redacted single-project acceptance bundle.

## PA-09 — Validate routing and retire the deployment plan

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PA-08
- Exact write set: isolated `.acorn.json`, canvas project choices, `.wiki/assets/powerautomate-logbook-reporting.md`, wiki indexes
- Primary integration lock: Power Automate operational record
- Outcome: Prove two authorized project routes and one unauthorized rejection, publish the durable runbook, and remove this plan.
- Focused validation: Verify isolated file changes, rejected unauthorized work, internal links, and wiki validation.
- Handoff: Delete this plan and all index, network, and lane entries.
