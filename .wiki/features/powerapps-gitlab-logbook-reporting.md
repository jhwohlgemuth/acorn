---
type: Plan
title: "Deploy and validate Power Automate logbook reporting"
description: "Deploy a Power Apps, Power Automate, ACORN, and GitLab workflow and prove authenticated, project-routed, duplicate-safe reporting."
difficulty: medium
impact: critical
package: [workspace, acorn-lib, acorn-cli, acorn-schema]
tags: [feature, automation, powerapps, powerautomate, gitlab, webhook, logbook, reporting, pdf]
status: waiting-on-validation
---

# Goal

Deploy one reviewed, duplicate-safe path from a Power Apps canvas app through Power Automate, ACORN, and a rolling GitLab merge request to an immutable PDF package and Power Automate notifications. Complete one single-project pilot before expanding the project map, submitter allowlist, or recipient lists.

This plan is the sole setup reference while work remains active. It records the deployment decisions, stop gates, exact asset boundaries, ordered walkthrough, troubleshooting checks, and acceptance evidence. After acceptance, move the validated operational record to `../assets/powerautomate-logbook-reporting.md` and retire this plan from the active indexes.

# Current evidence

- **Infrastructure correction (2026-10-11):** the user has no on-premises data gateway or managed relay. Available infrastructure consists of self-managed servers with self-signed TLS at best. Do not treat either a gateway or a cloud-reachable, publicly trusted HTTPS origin as available. Bot health and GitLab access can be verified independently, but the direct Power Automate intake deployment remains blocked on transport. The gateway/relay instructions below are conditional future options, not the user's next available setup step.
- `crates/acorn-lib/src/io/api/powerautomate/` implements strict form decoding, project and submitter authorization, shared-header authentication, durable intake, and direct callback resolution.
- `crates/acorn-lib/src/io/api/webhooks/` provides constant-time secret verification, durable idempotent operations, retryable callbacks, readiness, and graceful drain state.
- `crates/acorn-lib/src/io/api/gitlab/` implements rolling merge-request automation, authorized graduation commands, and immutable report-package publication.
- `crates/acorn-cli/src/commands/report/` composes bounded artifact creation and package publication; `templates/gitlab/reporting.gitlab-ci.yml` supplies the scheduled reporting job.
- `acorn create secret` supports explicit `--paste` clipboard delivery and newline-terminated output only when standard output is redirected or piped.
- Fixture-backed tests validate the 22-entry Power Automate sample, project routing, proposal persistence, graduation, and duplicate-safe operation handling.
- Generated help, focused tests, `make lint`, all 1,914 workspace tests, and `make book` passed on 2026-09-24. No source-code implementation remains unless deployment exposes a defect.
- The former book page and navigation entry are removed, all durable references point to this active plan, and `make book` passes on 2026-10-06.
- The source data project is the Internal `research-enablement/buckets/nssd` project, numeric ID `17410`, default branch `main`. Treat it as read-only production throughout this walkthrough; the inspected snapshot is `036c7b0074b2cc1901c0a65c66c254a04663ce74`.
- The isolated deployment target is the Internal `research-enablement/experiments/power-automate` project, numeric ID `22464`, default branch `main`. Its default branch still contains only its existing README and response CSV; the pilot assets remain isolated on `acorn/powerautomate-pilot`.
- The NSSD snapshot contains 78 tracked files, 35 project `index.json` files, no Git LFS objects, and about 18 MiB of current working-tree data. All 35 indexes pass an offline ACORN schema check; existing readability warnings do not block the copy.
- An exact copy of the NSSD project data is published only to the isolated target's `acorn/powerautomate-pilot` branch at commit `7ccb1a7`. The source README and CI configuration were excluded, the target's existing files were preserved, and a checksum-aware dry run shows no copy drift. No merge request has been opened and the target default branch remains unchanged.
- Pilot commit `66f4bef` adds a cross-project CI include pinned to ACORN commit `aa36e93f76ec22a9b3d2be9ceccf2a8d9cd891aa`; GitLab's pipeline editor reports that the expanded configuration syntax is correct. The schedule-only job remains disabled until the branch is reviewed, merged, and explicitly scheduled.
- Pilot branch commit `ecbbda4` configured `.acorn.json` for project `22464`, mapped `acorn` to `acorn/index.json`, and allowlisted `o9w@ornl.gov`, the address returned by the signed-in Power Apps session. A dry run against that exact commit passed with `no-changes`; no package was published and no callback was sent. Commit `447fbd45084496006a70aa9c0bdd2ef5db4c1b75` adds `c5c@ornl.gov` to the same member allowlist without changing the project mapping or touching production NSSD.
- The local ACORN 0.4.0 debug binary and container tooling are available for setup work. **Vergil is the selected bot host.** The existing `vergil` Docker context reaches an ARM64 Ubuntu 24.04 host running Docker 29.1.3 and the user confirms it can host an ACORN server port. The candidate service origin is `https://vergil:3000`, which the user expects can be exposed on ORNL networks only. DNS resolution, certificate trust, and reachability from an ORNL-managed gateway host are not yet verified; Vergil has no known public HTTPS route, and `kubectl` has no current context. The user confirms that the ORNL enterprise account has Power Automate Premium with HTTP actions.
- Vergil does not currently have an ACORN image. The published `ghcr.io/jhwohlgemuth/acorn:latest` manifest is `linux/amd64` only, the repository Dockerfile builds the `x86_64-unknown-linux-musl` target, and no `aarch64-unknown-linux-musl` GitLab package is available. The pilot therefore needs a native ARM64 build on Vergil or an explicitly reviewed multi-architecture image; do not assume the published image runs natively there.
- ACORN startup resolves `POWER_AUTOMATE_INBOUND_SECRET` immediately, but resolves `POWER_AUTOMATE_CALLBACK_URL` only when delivering a callback. Polling mode does not require a public URL or GitLab webhook credentials, so a bounded first boot can prove `/health` and `/ready` before the callback flow and approved Power Automate-to-Vergil route exist. Do not submit a real intake until the callback URL is configured.
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
- Neither flow has an HTTP action, ACORN URL, or intake credential. The notification-only pilot must prove Power Apps-to-flow invocation, Teams chat, email, and visible app status without contacting Vergil or GitLab. The full ACORN action remains gated on an approved route from the app environment to Vergil.
- Access to the GCC Power Automate maker portal and permission to create and save an instant cloud flow are confirmed. The remaining Power Platform stop gates are permission to add and run the premium HTTP action, approval and availability of the private gateway/custom-connector route or an organization-managed HTTPS intermediary, and the tenant's `Anyone` HTTP-trigger policy for callbacks. The named operational owners also remain unresolved.

# Architecture decisions

- With the currently available infrastructure, first verify outbound access from Vergil to the tenant's Power Automate callback endpoint. A server initiating an HTTPS request does not need a publicly trusted server certificate of its own; it must validate the remote endpoint's certificate. For intake, evaluate an outbound worker that retrieves submissions from an approved cloud-accessible durable queue and forwards them to ACORN locally. This is a proposed alternative requiring transport design, permissions, implementation, and validation; it is not supplied by GitLab `--event-source poll`. No queue service or tenant application permissions are assumed available.
- Use the existing GitLab data project for `.acorn.json`, project `index.json` files, review merge requests, pipeline schedules, and generic-package artifacts. Do not place deployment configuration or operational research data in the ACORN source project.
- Begin with one project mapping and the two explicitly approved tester addresses. Add projects or any other submitters and recipients only after the pilot passes the duplicate-submission smoke test.
- Use an organization-only Power Apps canvas app as the primary intake interface and an instant flow with the Power Apps (V2) trigger as its adapter to ACORN. Microsoft Forms is not required by the ACORN interface.
- Treat the current developer environment as a build-and-test workspace only. The obsolete generated `Logbooks screen` and unused `cr1d0_logbooks` dependency are removed for the notification-only pilot, but confirm the cleaned app's effective license designation with the shared tester. Before broader or production use, move or recreate the app and its environment-local flow in an ORNL-approved shared or production GCC environment with sufficient sharing capacity, then share the app and each dependent resource with the approved users or security group.
- Run `acorn serve bot` on Vergil as a persistent managed service with protected environment variables. Treat `https://vergil:3000` as the candidate ORNL-internal origin after its DNS name and TLS certificate validate from the selected gateway host. A public address is not required when Power Automate reaches that private endpoint through an ORNL-managed [on-premises data gateway](https://learn.microsoft.com/en-us/data-integration/gateway/service-gateway-onprem) and approved custom connector; the gateway host must be an ORNL-managed Windows host with network access to Vergil, not Vergil itself. The gateway uses outbound cloud connections, so this route does not require opening inbound Internet access to Vergil. An organization-managed public HTTPS reverse proxy or relay is the alternative. A temporary developer tunnel is not an accepted deployment endpoint.
- Use `--event-source poll` for the bounded Vergil boot and health/readiness check and retain polling for production if the private gateway route is selected. Move to `hybrid`, provide both GitLab webhook credentials, and use `--register-webhook` only if an organization-managed public HTTPS origin for Vergil is later approved.
- The full intake path requires a premium action capable of reaching ACORN. A normal cloud HTTP action cannot call an unrouteable private Vergil address. Prefer an ORNL-approved custom connector through the on-premises data gateway; otherwise use an approved HTTPS intermediary. If neither route is approved, stop after the notification-only pilot and do not weaken tenant or network policy.
- Direct callbacks use the SAS credential embedded in a Power Automate HTTP trigger URL and require the trigger's legacy `Anyone` mode documented in [OAuth authentication for HTTP request triggers](https://learn.microsoft.com/en-us/power-automate/oauth-authentication). If tenant policy prohibits that mode, use an approved intermediary or open a separate implementation plan for Microsoft Entra service-principal support; do not weaken tenant policy.
- `acorn create secret --paste` generates the 256-bit lowercase hexadecimal intake credential. Store the same value in the ACORN runtime and the intake flow's approved secret facility, then clear the clipboard. ACORN does not persist it.
- Intake requests reuse one submission UUID as `submissionId` and `x-acorn-delivery-id`. Retries preserve that UUID so ACORN can return the same operation without creating another entry.
- Power Automate owns Teams and email recipients and message formatting. ACORN callbacks contain artifact metadata, never GitLab credentials or PDF bytes.
- A person reviews, graduates, and merges the monthly merge request. The bot updates the review branch and publishes reports but never merges its own changes.

# Public interfaces

- `POST /webhooks/powerautomate/forms` requires `Content-Type: application/json`, `x-acorn-delivery-id`, and `x-acorn-webhook-secret`.
- The version-1 form envelope requires `schemaVersion`, `submissionId`, `projectId`, `submittedAt`, `submitter.email`, and an `entry` containing `timestamp`, `category`, `state`, and `summary`. `classification`, `eventType`, and `items` are optional; omit inapplicable fields instead of sending `null` or extra properties.
- Categories are `administration`, `collaboration`, `data`, `engagement`, `funding`, `infrastructure`, `method`, `other`, `personnel`, `publication`, `research`, and `software`.
- States are `completed`, `in-progress`, and `upcoming`; event types are `conference`, `exhibition`, `meeting`, `other`, `presentation`, `tour`, `training`, and `workshop`; classifications are `unclassified`, `confidential`, `secret`, and `top-secret`.
- An accepted intake returns HTTP `202` with `disposition` and `operationId`. A retry may return a duplicate disposition with the same operation ID.
- Callback events are `mr-created`, `review-needed`, `artifact-ready`, `report-no-changes`, and `failed`. Every callback contains `event` and `projectId`; event-specific fields carry merge-request URLs, package URLs, checksums, cutoffs, or failure messages.
- `.acorn.json` names `POWER_AUTOMATE_INBOUND_SECRET` and `POWER_AUTOMATE_CALLBACK_URL`; it never contains their values. The runtime also receives a GitLab token and `GITLAB_WEBHOOK_SIGNING_TOKEN`.
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

- **This preparatory update:** remove the former book page and navigation entry, make `.wiki/features/powerapps-gitlab-logbook-reporting.md` the sole process source, and keep every reference inside the planning wiki.
- **Production source project:** `research-enablement/buckets/nssd` is a read-only input. Do not commit, push, register webhooks, create variables or schedules, or run write-capable automation against it.
- **Isolated GitLab data project:** `research-enablement/experiments/power-automate` owns the copied NSSD project directories, `.acorn.json`, and `.gitlab-ci.yml`; the selected project `index.json` is changed only through the authenticated intake, graduation, and human merge workflow.
- **Power Platform:** one organization-only Power Apps canvas app, one app-triggered intake cloud flow, and one callback notification cloud flow.
- **Managed runtime:** one ACORN service definition with protected `GITLAB_TOKEN` or equivalent, `GITLAB_WEBHOOK_SIGNING_TOKEN`, `POWER_AUTOMATE_CALLBACK_URL`, and `POWER_AUTOMATE_INBOUND_SECRET` settings.
- **GitLab control plane:** one registered webhook, protected CI variables, and one monthly default-branch schedule.
- No ACORN source files are in the deployment write set. If the smoke test exposes a product defect, pause the walkthrough, change this plan to `in-progress`, record the failing evidence and exact source write set, and only then modify code.
- After the deployed acceptance criteria pass, convert the verified process and redacted evidence requirements into `.wiki/assets/powerautomate-logbook-reporting.md`, redirect dependent wiki plans to that asset, delete this plan, and remove its index, network, and lane entries.

# Dependencies and stop gates

## Next steps with the available servers

1. Finish the Vergil bot health, readiness, and isolated GitLab access checks in polling mode.
2. Test a policy-approved Power Automate callback flow from Vergil over outbound HTTPS and confirm Teams/email delivery. Store the callback URL securely before real intake.
3. Establish whether an approved cloud-accessible durable queue is available to both the app's flow and a worker on Vergil, including permission for unattended authentication. This is the next intake feasibility check; do not assume a gateway can be installed or a managed relay supplied.
4. If a queue is available, design and implement the outbound worker and queue-writing flow as separately scoped work. Preserve the submission UUID across retries, acknowledge only after durable ACORN acceptance, and distinguish queued receipts from accepted operations. If no queue or direct trusted route is available, retain the notification-only app.
5. Once transport is validated, run one isolated submission through human review, graduation, merge, report publication, callback delivery, and duplicate replay as described in steps 7–9.

The direct HTTP walkthrough below remains a conditional alternative if infrastructure changes. Microsoft's [workflow TLS guidance](https://learn.microsoft.com/azure/logic-apps/logic-apps-securing-a-logic-app) documents self-signed server-certificate trust failures; do not plan around disabling certificate verification.

## Gates for full intake

Do not enable full intake or production flows until all of these gates pass. A protected local intake credential is permitted for the bounded bot boot. The notification-only canvas pilot is explicitly allowed before gates 2 and 3 because it does not call ACORN, GitLab, or an HTTP trigger. An isolated callback test may proceed before gate 2 once callback policy and recipients are approved:

1. Power Automate Premium with HTTP actions and maker access to a GCC Moderate Power Apps developer environment are confirmed for `o9w@ornl.gov`; this proves build capability, not a deployable shared environment or end-user entitlement. The notification-only app no longer has its Dataverse source, is shared with the one approved tester, and the tester has launched the cleaned version without a trial prompt. Confirm that she can establish the required Teams and Office 365 Outlook connections and complete one notification-only submission. Before broader use, obtain an ORNL-approved shared or production GCC environment with sufficient sharing capacity and verify the resulting app license designation and dependent-resource access. Use the GCC maker portals, not the commercial endpoints.
2. Intake transport is proven. No gateway or managed relay is currently available, and self-signed TLS alone establishes neither cloud reachability nor cloud certificate trust. Either an approved direct route with cloud-trusted TLS becomes available, or a separately designed and validated outbound queue worker supplies local intake. Until then, keep the app notification-only; callback testing and local bot checks may proceed independently.
3. The tenant permits the callback trigger's `Anyone` mode, or an approved intermediary exists for callbacks.
4. The isolated GitLab data project supplies its numeric project ID, default branch, package permission, one safe repository-relative `index.json` path, one stable pilot identifier, and the explicitly approved tester email allowlist. No deployment credential or integration may target the production source project.
5. Named owners exist for the Vergil ACORN service, Power Apps canvas app and flows, GitLab review and merge, Teams destination, and email recipient list.

The implemented webhook runtime, GitLab automation, reporting-window selection, PDF renderer, package manifest, and reporting CI template are otherwise sufficient for the pilot.

# Human and Codex responsibilities

| Owner | Responsibilities |
| --- | --- |
| **You** | Coordinate an ORNL-approved shared or production GCC environment and the intended user-license model with the Power Platform administrator; approve the Teams destination and email recipients; coordinate approval for the private gateway/custom-connector route or organization-managed HTTPS intermediary; explicitly share and publish the organization-owned canvas app when the draft is accepted; review, graduate, approve, and merge GitLab changes. |
| **Codex** | Maintain the private plan; prepare and validate GitLab branches, `.acorn.json`, CI, controlled project mappings, flow expressions, schemas, service manifests, and non-production tests; keep NSSD read-only; present exact human instructions at each handoff. |
| **Together** | Confirm the project roster and member allowlists; place generated credentials into protected stores without exposing them; run the end-to-end and duplicate-submission smoke tests; review redacted acceptance evidence. |

Codex stops at actions that require the user's enterprise authority, policy choice, secret entry, representational approval, or human review. Each such handoff must state the exact expected result before automated work resumes.

# Deployment walkthrough

## 1. Freeze the pilot record

Use GitLab project `22464` (`research-enablement/experiments/power-automate`) on `main`, pilot `projectId` `acorn`, path `acorn/index.json`, testers `c5c@ornl.gov` and `o9w@ornl.gov`, and pinned ACORN commit `aa36e93f76ec22a9b3d2be9ceccf2a8d9cd891aa`. Record the approved gateway or HTTPS intermediary, Teams destination, email recipients, and named owners. Keep production source project `17410` out of all configuration and credentials.

- **Verify:** every value has an owner and storage location, and none is still represented by an example such as `12345`, `pilot`, or `example.org`.

## 2. Refine the existing Power Apps canvas draft

Reuse the published GCC Moderate draft rather than creating another app. Rename it consistently to **ACORN Logbook** (or another approved single name) and make a dedicated contributor submission screen the pilot's entry point. The current gallery and detail screen may remain only as a separately authorized maintainer experience; ordinary contributors must not receive direct edit or delete controls for existing entries.

Build the contributor screen around this field ownership:

| Surface | Control and behavior |
| --- | --- |
| Project | For the pilot, display **ACORN** as read-only and pass the fixed `acorn` identifier. After pilot acceptance, replace it with the approved controlled Project choice; never accept a repository path or free-form project identifier. |
| Reporting as | Display `User().FullName` and `User().Email` read-only. Pass the signed-in email to the flow; the user cannot override it. |
| When | Default to the current local date and time. The flow converts it to UTC RFC 3339 before calling ACORN. Do not leave the generated `12/31/2001 00:00` placeholder as the effective default. |
| Category | Required choice limited to the ACORN category values under Public interfaces. |
| State | Required choice limited to `completed`, `in-progress`, and `upcoming`; choose an explicit pilot default only if the owner approves it. |
| Event type | Optional choice limited to the ACORN event-type values plus an unselected state. |
| Classification | Optional choice limited to the ACORN classification values plus an unselected state. Omit this control entirely if the pilot has no classification use case. |
| Summary | Required multiline plain text with a visible character limit and inline error. |
| Items | Optional multiline plain text with one item per line; the flow removes blank lines and sends a JSON string array. Do not ask contributors to author JSON. |
| Submit | Call the Power Apps (V2) flow once, disable repeated activation while pending, and show the returned disposition and operation ID. Clear the form only after an accepted response; preserve it on failure. |

Do not expose `identifier`, `origin`, `discovery`, or `archived` on the contributor screen. ACORN generates the identifier, the intake adapter sets the manual origin, the public intake contract does not accept discovery, and new submissions are not archived. Keep the shared secret, callback URL, and repository path out of all app formulas and controls.

Give every input a persistent label, mark required fields in text as well as color, set a logical tab order, and assign accessible labels to New, Submit, Cancel, Edit, and Delete if any of those controls remain. A submission status must be announced as text, not only by icon or color.

- **Published evidence (2026-10-07):** `SubmitEntryScreen` implements the pilot field set and controlled values, opens first, and calls the environment-local notification flow with six inputs. Required-field validation, pending-state disabling, success/failure status text, and a current Teams-and-email tooltip are active. Default and 360-by-800 previews render the complete form with a scrollable body and fixed status/footer. The pilot ACORN allowlist contains the signed-in `User().Email` value plus `c5c@ornl.gov`; that allowlist is independent of the Power Apps grant. The obsolete generated screen and Dataverse source are removed, the saved Data pane is empty, and the cleaned version is published. Power Apps successfully granted Clark, Niki user-only access after resolving `c5c@ornl.gov`, and she opened the clean launch without a trial prompt. Her notification submission confirmation remains pending.

- **Verify:** the shared tester opens the clean launch link without a Power Apps trial prompt, sees the contributor screen and her signed-in ORNL identity, establishes the required flow, Teams, and Office 365 Outlook connections, and can submit the notification-only pilot. In an approved shared or production GCC environment, explicitly share the published app only with approved testers, confirm every tester has the required entitlement for the app's final license designation, and ensure all dependent resources are accessible. The app contains no shared secret, callback URL, or repository path; a blank form defaults to the current time, cannot submit an invalid controlled value, has no ordinary-contributor edit or delete path, and exposes meaningful accessibility names for every actionable control.

## 2a. Prove the canvas-to-notification path

Before introducing Vergil, GitLab, HTTP actions, or credentials, create **ACORN Logbook Canvas Notifications** in the same `221f…` Power Platform environment as the canvas app. Use the Power Apps (V2) trigger with the six reduced pilot inputs already represented by the contributor screen: fixed project key, signed-in requester email, selected date, category, state, and summary. Use that Requester Email input as the recipient for both the direct Teams chat through Flow bot and the Office 365 Outlook email, then let the canvas show a short success status when the flow call completes. Never hard-code a tester address into either notification action. This is a transport demonstration, not an ACORN intake: it deliberately omits event type, items, submission persistence, GitLab routing, and all secrets. Reserve a structured Power Apps response action for the full intake disposition and operation ID.

Connect that environment-local flow to **Submit highlight**. Disable the button while the call is pending, validate the required fields, display success or failure in `SubmissionStatus`, and preserve the entered values on failure. Keep the app unpublished until the tester approves the draft behavior.

- **Pilot evidence (2026-10-07):** flow `18677f6b-21af-435c-a000-a2d471551019` and the canvas wiring are saved. Both notification actions route through the dynamic Requester Email input, Flow Checker reports zero errors and zero warnings, and App checker reports no formula issues. One explicit click from the unpublished canvas completed run `08584102113387025966515574751CU09` in three seconds. The Power Apps trigger, Teams direct-chat action, and Office 365 Outlook email action each report **Succeeded**, and the canvas displayed **Notifications sent.** No HTTP, ACORN, GitLab, or NSSD action exists in the run. After that run, the saved message templates were enriched: Teams now includes project, date, category, state, and summary; email includes all six trigger fields. A new published-app submission is required to validate the enriched content and visible delivery in the tester's clients.

- **Verify:** one explicit test click from the unpublished canvas produces exactly one Teams direct-chat message and one email to the signed-in submitter identified by Requester Email, the canvas reports success, and no request reaches Vergil, GitLab, or the production NSSD project. Record the flow run ID and redacted screenshots without copying message content or addresses into public documentation.

## 3. Generate and store the intake credential

Run `acorn create secret --paste`. Store the generated value as `POWER_AUTOMATE_INBOUND_SECRET` in the protected ACORN runtime and as the intake flow's approved secret or [Key Vault-backed environment variable](https://learn.microsoft.com/en-us/power-apps/maker/data-platform/environmentvariables-azure-key-vault-secrets). Clear or replace the clipboard immediately. For a headless host, capture redirected standard output rather than running the silent interactive form:

```bash
POWER_AUTOMATE_INBOUND_SECRET="$(acorn create secret)"
export POWER_AUTOMATE_INBOUND_SECRET
```

- **Verify:** the two protected locations contain the same 64-character lowercase hexadecimal value, while source control, app controls, ordinary Compose actions, logs, and flow run history do not expose it.

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

If the approved design later supplies an organization-managed public TLS endpoint, callback URL, and GitLab webhook credentials, restart the same managed service in hybrid mode and register the webhook:

```bash
acorn serve bot 22464 \
  --bind 0.0.0.0:3000 \
  --config .acorn.json \
  --event-source hybrid \
  --public-url https://<managed-acorn-host> \
  --register-webhook
```

- **Verify the bounded boot:** local requests to `/health` return `ok`, `/ready` returns HTTP 200, and startup logs contain no secret values. Stop here if the executable is not native ARM64 or the configuration names production project `17410`.
- **Verify the selected integration route:** for the gateway design, an ORNL-managed Windows gateway host can reach Vergil's private health, readiness, and form endpoints while Vergil remains unreachable from the public Internet; the custom connector is bound to the canvas environment. For the relay design, the organization-managed HTTPS origin reaches those same endpoints. Register a GitLab webhook only if the public route exists; otherwise retain polling. Startup and gateway logs must contain no secret or SAS values.

## 6. Create the Power Apps intake flow

**Blocked for the current infrastructure:** do not add a cloud HTTP action targeting `https://vergil:3000` on the assumption that a self-signed certificate or working local browser connection makes it usable from Power Automate. The direct-flow instructions below apply only after a reachable, trusted route exists. If the outbound queue option is selected, revise this step to persist the version-1 envelope and stable submission UUID in the selected queue, return a queued receipt to the app, and separately report ACORN acceptance after the worker delivers it. Do not report a queued item as ACORN-accepted.

Evolve or replace the environment-local notification pilot with an instant cloud flow in the canvas app's environment. Use the **Power Apps (V2)** trigger and typed inputs matching the full canvas controls. Generate one `guid()` output per invocation and reuse it for `submissionId` and `x-acorn-delivery-id`. Through the approved custom connector/gateway or organization-managed HTTPS intermediary, POST to the ACORN intake endpoint with the three required headers and the strict version-1 JSON body. Use UTC RFC 3339 timestamps and omit null or additional fields. Return the HTTP status, disposition, and operation ID to the app so it can show a clear success or failure message. Microsoft documents the supported canvas-app invocation pattern in [Create a canvas app that can trigger a Power Automate flow](https://learn.microsoft.com/en-us/power-apps/maker/canvas-apps/how-to/trigger-flow).

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

Enable Secure Inputs and Secure Outputs on the HTTP action. Treat HTTP `202` as accepted and retain the response ID, returned disposition, and operation ID in an approved operational record.

- **Draft evidence (2026-10-07):** the saved default-environment scaffold uses the Power Apps (V2) trigger with the eight planned typed inputs. `Generate Submission ID` calls `guid()`, `Set Submitted At` calls `utcNow()`, `Set Entry Timestamp` formats the selected date as a UTC RFC 3339 timestamp, and `Split Item Lines` plus `Filter Nonblank Items` prepare the optional item array. It now demonstrates the intended Teams, email, and Power Apps response shapes, but cannot connect to the canvas across environments. The environment-local six-input notification flow is the immediate canvas pilot. The ACORN request, approved private route, secure header value, full app response, and first intake run remain deliberately absent.

- **Verify:** the flow definition contains one source for the submission UUID, maps each choice to an allowed wire value, and preserves the same HTTP action inputs during automatic retries.

## 7. Exercise human review and graduation

Submit one response from an allowlisted tester. Confirm ACORN creates or updates `acorn/powerautomate/<project>/<year-month>` and its draft merge request. Review the proposed entry, request corrections through GitLab discussions if necessary, and add `/acorn graduate` as a user with at least the Developer role. Inspect the updated branch, approve it, and have a person merge it.

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

Run the scheduled reporting job manually. Confirm the GitLab generic package contains the immutable manifest and expected PDF, then confirm `artifact-ready` reaches both Teams and email with the package link, checksum, and cutoff. Replay the original intake using the same submission UUID and delivery ID.

- **Verify:** the replay returns the same operation as a duplicate, creates no second entry, and the complete pilot produces one reviewed entry, one package version, and one accepted artifact notification event.

## 10. Expand to controlled multi-project routing

After the single-project pilot passes, the human owner supplies the approved project labels, stable identifiers, repository paths, and member email allowlists. Codex validates every path against the isolated repository and extends `.acorn.json` on a review branch. The human owner approves the required Project choice in the organization-only canvas app, and Codex supplies the label-to-`projectId` mapping used by the app and flow. No default branch may accept an unknown label. Before sharing, the human owner confirms that each user has suitable Premium rights or that the flow has a Process license.

Submit authorized test entries for two different projects and one deliberately unauthorized project/member combination. Review the resulting branches and diffs before merging any entry.

- **Verify:** each authorized entry changes only its configured `index.json`, uses a distinct project/month review branch, and retains the selected `projectId`; the unauthorized combination produces no branch, merge request, or repository change.

# Troubleshooting during the walkthrough

| Symptom | Check |
| --- | --- |
| Intake returns HTTP `400` | Check exact JSON field names, controlled values, project mapping, allowlisted email, safe `index.json` path, timestamps, nulls, and additional properties. |
| Intake returns HTTP `401` | Check that the flow resolved the protected value and that it matches `POWER_AUTOMATE_INBOUND_SECRET`. |
| Intake returns HTTP `503` | Wait for `/ready`, then retry with the same submission UUID and delivery ID. |
| Entry timestamp is rejected | Check whether the entry predates the latest closed reporting cutoff. |
| Callback repeats or returns `401`/`403` | Check for a successful 2xx response, the trigger's approved mode, and the complete current SAS URL including `sig`. |
| Package contains only a manifest | Confirm whether the reporting interval contains any graduated entries; a no-change interval intentionally omits the PDF. |
| `/acorn graduate` has no effect | Check the exact command, the commenter's GitLab role, webhook registration, and webhook signing token. |

# Validation evidence

- **2026-10-11 transport status:** no gateway or managed relay is available; self-managed servers have self-signed TLS at best. No cloud-to-Vergil intake route has been demonstrated. Next steps after bot verification are an outbound callback test and a transport feasibility decision. An outbound intake worker remains a proposal, not implemented or validated functionality.
- The 2026-10-07 canvas version is published. The live launch confirms `SubmitEntryScreen` is the start screen; preview also verifies it at the default canvas size and at 360 by 800. The controlled category list contains only the placeholder plus the twelve documented categories. The environment-local notification flow is attached, the blank form keeps **Submit highlight** disabled, the tooltip is `Submit highlight to Teams and email`, and the status is `Ready to send Teams and email notifications.`
- App checker reports zero formula issues for the saved draft. Its 21 missing-label and two missing-tab-stop errors are confined to named controls on the pre-existing generated `Logbooks screen`; no new contributor control appears in those findings.
- Notification-only run `08584102113387025966515574751CU09` completed successfully in three seconds on 2026-10-07. Its Power Apps trigger, Microsoft Teams direct-chat action, and Office 365 Outlook email action all report **Succeeded**, and the canvas displayed **Notifications sent.** The run contains no HTTP, ACORN, GitLab, or NSSD action. Record human confirmation that the Teams chat and email are visible before treating delivery as accepted.
- The enriched Teams and email templates were saved after Flow Checker reported zero errors and zero warnings. No test was triggered during the template update; the next tester submission must confirm the expected field values and formatting in both delivered messages.
- Record screenshots or redacted exports showing the premium HTTP action and callback trigger policy without exposing credentials.
- Record the ACORN health and readiness results, HTTP `202` intake receipt, disposition, and operation ID.
- Record the GitLab branch and merge-request URLs, authorized graduation note, merged commit, pipeline URL, package version, manifest, and PDF checksum.
- Record the Teams and email deliveries for `artifact-ready` and the callback flow's successful 2xx run.
- Record the duplicate replay response and confirm the data-project diff contains only one normalized entry.
- Confirm generated secrets, shared-secret headers, callback URLs, SAS query parameters, and GitLab tokens are absent from repository history, service logs, artifacts, and visible flow histories.
- `make book` passes after removing the former book page and navigation entry. Repository lint and tests need rerunning only if the walkthrough exposes a defect that requires source changes.

# Acceptance criteria

- The chosen intake transport is demonstrated from the actual runtime. If an outbound queue worker is adopted, document its approved queue and authentication, durable retry/acknowledgement behavior, preservation of submission UUIDs, and queued-versus-accepted app status before claiming end-to-end acceptance. Self-signed TLS on Vergil and GitLab polling alone do not satisfy this criterion.
- All licensing, approved-reachability, callback-authentication, ownership, and pilot-mapping gates pass without weakening organizational policy; public ingress to Vergil is not required when the approved private gateway path is used.
- One organization-only Power Apps canvas app and two protected Power Automate flows implement the documented intake and callback contracts.
- The managed ACORN service on Vergil remains healthy and ready, receives authenticated intake through the validated transport, and authenticates GitLab review events. A gateway or managed HTTPS route is required only if that transport is selected.
- Missing or incorrect intake credentials fail closed, while one valid response returns HTTP `202` and creates the expected monthly review branch and merge request.
- Human graduation preserves the routed project identifier, and a person—not the bot—merges the reviewed entry.
- The scheduled job publishes one immutable report package and Power Automate sends both Teams and email notifications for the accepted artifact.
- Replaying the same submission UUID is idempotent and does not create a second entry, merge request operation, package, or notification event.
- Before multi-project rollout, two authorized project selections route only to their configured `index.json` files and an unauthorized project/member combination creates no repository change.
- No credential appears in source control, logs, artifacts, app controls, ordinary Compose actions, or visible Power Automate run history.
- The redacted evidence set is sufficient for another operator to verify every acceptance criterion; the verified process can then transition to `.wiki/assets/powerautomate-logbook-reporting.md`, durable references can be redirected, and this plan and its active index entries can be deleted.
