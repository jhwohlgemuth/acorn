---
type: Plan
title: "Validate GitLab JSONL intake and deploy Power Automate logbook reporting"
description: "Validate mixed-project JSONL routing and deploy reviewed, duplicate-safe Power Automate logbook reporting."
difficulty: medium
impact: critical
package: [workspace, acorn-lib, acorn-cli, acorn-schema]
tags: [feature, automation, powerapps, powerautomate, gitlab, webhook, logbook, reporting, pdf]
status: in-progress
priority: next
execution: phased
phase_budget_hours: 5
---

# Goal

**Top blocking priorities:** The ACRE 2.0.0 contract, multiplexed JSONL intake, and local MVP commands are implemented. The notification-only tester pilot is verified. PA-05 depends on external ownership, access, service boot, and callback preparation in PA-02 through PA-04. Live append, merge, package publication, and notification evidence remain in PA-05 through PA-09.

[Complete proposed JSON, JSONC, YAML, and deployment environment examples](../assets/acre-logbook-configuration-examples.md) define the intended layout and behavior. Schema version (`2.0.0` hard cutover, no 1.0.0 migration) and ACRE field names are settled per `ACRE_SPEC.md`; the `options.powerAutomate.intake` source settings and JSONL poller are implemented.

Implement the selected sequence: Power Apps form → Power Automate flow → append to the JSONL Logbook inbox in GitLab → Vergil polls GitLab → the bot durably ingests new items and removes them from the inbox → the bot routes each item by `projectId` and augments the associated project's `index.json`. Use `.acorn/intake/logbook.jsonl` for the pending inbox. Each line wraps form-controlled Logbook fields in the existing version-1 submission envelope. Preserve the established review branch and human-merge policy for canonical changes, followed by PDF publication and notifications. Complete one single-project pilot before expanding mappings or submitters.

The intended end state permits each authorized member to submit one or more forms for an allowed project folder in the designated multi-folder GitLab project. Power Apps sharing and ACRE `members` authorization are separate gates; both must allow the contributor. Power Automate appends one JSONL line per submission. ACORN then supports either a bot-driven or an operator-driven path from that same inbox through routing, project-file updates, a reviewable exported diff artifact, merge requests, and artifact generation. GitLab merge requests carry human review; Teams and email deliveries are sent through the callback flow. The operator-driven path must use ACORN's public API/CLI operations for the processing steps and preserve the same validation, authorization, idempotency, and review rules as the bot path.

**Manual path (implemented 2026-10-17):** `acorn intake stage/apply` handles standalone local indexes. `acorn intake run` polls one configured GitLab inbox snapshot and note page, then executes bounded durable review work. `acorn intake export` writes a UTF-8 unified diff of the selected project file at the open MR's computed base/head revisions, plus a JSON receipt with source digest, route, MR identity, revisions, and SHA-256 checksum. `acorn intake deliver` explicitly drains ready durable callback intents. `acorn report --dry-run` remains the separate PDF/manifest path. The full live sequence still needs PA-05 through PA-09 evidence.

The native MVP executable is `target/debug/acorn`, built with `cargo build --package acorn-cli --bin acorn`. [ACRE operator documentation](../../docs/src/concepts/acre.md#run-one-intake-cycle-from-the-native-executable) gives the exact commands and protected environment. The local stage/apply fixture, mocked GitLab diff export/replay, JSONL cleanup/restart, note authorization, durable callbacks, and headless PDF rendering have focused test evidence. The local operator and Vergil bot must not independently consume the same inbox/branch with separate databases at the same time. The isolated live pilot still needs its GitLab credential, flow, and Vergil service before these commands can be exercised against real submissions.

This plan is the sole setup reference while work remains active. It records the deployment decisions, stop gates, exact asset boundaries, ordered walkthrough, troubleshooting checks, and acceptance evidence. After acceptance, move the validated operational record to `../assets/powerautomate-logbook-reporting.md` and retire this plan from the active indexes.

# Current evidence

- **PA-01 verified (2026-10-17):** the published canvas in environment `221f44a6-1468-ea4e-b93d-cd52f1723079` has `SubmitEntryScreen` and the attached **ACORN Logbook Canvas Notifications** flow. Tester submission run `08584102031146888583116268486CU06` on 2026-10-07 at 16:09 followed the enriched template save at 15:39; its Power Apps trigger, Teams action, and Office 365 Outlook action succeeded. The run's requester was the approved tester `c5c@ornl.gov`. In the private Teams conversation, the tester shared a screenshot of the received message at 16:10 and confirmed at 16:11 that the Teams message matched the email. The screenshot shows the pilot project, requester, date, category, state, summary, and notification-only notice; it is retained in Teams, not copied into this repository. The tester reported that the in-app completion feedback was easy to miss and that optional items were absent from the notification-only message. Carry that feedback into the later queued-receipt and intake-flow review.
- **PA-24 and PA-25 implemented locally (2026-10-17):** a two-folder JSONL fixture and mocked GitLab integration exercise one- and three-record streams, canonical IDs and aliases, immutable admitted routes, selected review index writes, rejection isolation, and replay. Standalone local intake is available through `acorn intake stage/apply`: exact-line proposals are stored under `.acorn/intake/proposals/`, canonical indexes stay unchanged until explicit apply, changed bases require restaging, and double apply and crash recovery are idempotent. Focused tests, scoped clippy, the 2,493-test workspace suite, and `acorn-lib` WASM feature check pass. The broader `make check-wasm` stops at an unchanged `acorn-schema` core-only `OneOrMany`/`alloc` feature error; it does not reach the local intake adapter, which is agentic/host-only. No external deployment was changed.
- **PA-22 and PA-23 landed locally (2026-10-16):** exact-byte admission, source observations, atomic queue persistence, canonical alias routing, verified bucket association, and immutable worker routes are implemented. `Server::poll_jsonl_once` and the independent configured poll loop admit before cleanup; snapshots preserve original content and the file `last_commit_id`; cleanup rereads/recomputes after conflicts or uncertain responses, keeps the drained file, and preserves unadmitted bytes. Focused GitLab, Power Automate, webhook, and configuration suites pass 238/238, including restart, conflict, rollback, missing metadata, concurrent append/cleanup, and stored-route regressions. Formatting, scoped clippy with no new warnings, the `analysis` feature mirror, and the no-default-features check pass. The pre-existing analyzer test `ValeOutput` unused-import warning remains. Intake uses `poll_interval: Seconds` with a numeric wire value; file and record limits use `Memory`. Added code has no propagation operators or function early returns, and provider dependencies use module namespaces.

- **ACRE 2.0.0 gate landed (2026-10-14/15):** commits `984dcb65` and `14970672` completed the refactor on top of `b6e847a4`. `AcreSchemaVersion` declares only `2.0.0` with 1.0.0 rejected (hard cutover, no migration tool per `ACRE_SPEC.md` §Compatibility); root `powerAutomate`/`projects` mappings are removed in favor of root `folders`, `members`, `variables` plus `options.folders.root`, `options.powerAutomate`, and `options.scripts` ownership; the intake envelope carries no wire `schemaVersion`. A `Send`/`Sync` `FolderRegistry` snapshot is derived from the document because `ApplicationConfiguration` holds a non-`Send` JSONC CST. `OneOrMany` covers schema arrays (`buckets`, `folders`, `members`, `aliases`, `endpoints`, `models`, `runners`, `scripts`, whitelist/model/runner/llama-swap nested lists) with singleton and array forms tested across JSON/JSONC/YAML (`with_folders.*` fixtures). Bucket name/index references, folder resolution precedence, global/folder authorization with `variables.members` (process-environment lookup), and casing (snake_case new attributes; grandfathered `schemaVersion`, `scriptManifest`, `options.powerAutomate`) are implemented, tested (`io::config` ACRE tests), and specified (`ACRE_SPEC.md`, `docs/src/concepts/acre.md`). Deliberately deferred as non-blocking for JSONL: a fully typed built-in variable catalog with cross-repository coverage gate, per-root `options.<name>` counterparts beyond the documented set, and recursive empty-object omission.

- **ACRE landings (2026-10-13/14):** `b6e847a4` consolidated the schema wrapper onto `acorn_core::OneOrMany`; follow-ups `984dcb65` and `14970672` finished the refactor (tests fixed). Earlier notes about `Vec` in `powerautomate/config.rs` and `llama_swap.rs`, missing root folders/members/variables, and missing options counterparts no longer apply — those files now use `OneOrMany` and the root/options layout above. Treat the landed contract as the JSONL baseline.
- **Xylem boundary (landed):** Xylem does not parse ACRE and has no `ApplicationConfiguration` consumer. No second ACRE model was introduced into Xylem. A full cross-repository variable catalog with coverage gate is deferred as non-blocking for JSONL (see Variable references).
- **Selected transport (corrected 2026-10-17):** the user tested only form input → Teams/email notifications; no Power Apps submission was saved to CSV or JSONL. GitLab JSONL is the planned intake format: Power Automate will append to `.acorn/intake/logbook.jsonl` in the target project, and Vergil will poll that file. The external append flow has not been implemented. No remote project was changed during the notification pilot.
- **PA-05 draft checkpoint (2026-10-17):** after the user requested PA-05 ahead of its deployment prerequisites, an app-environment flow named **ACORN Logbook GitLab JSONL Intake (Draft)** was saved with flow ID `6039f419-420f-6922-46ef-04b00d2290f1`. Its Power Apps (V2) trigger has typed Project ID, Reporter Email, Entry Date, Category, State, optional Event Type, Summary, and optional Items inputs. Two Compose actions generate `guid()` and `utcNow()` once per run. The portal shows the draft **Off**, with no associated apps, connection references, or runs. It has no GitLab write, callback, queued receipt, or app submit binding; PA-05 remains blocked on the approved branch/API route, protected credentials, Vergil service, and callback flow.
- **Infrastructure:** the user has self-managed servers with self-signed TLS at best and has not provisioned a gateway. Organizational gateway availability is unverified and is not a dependency of the selected design. Power Automate must reach and authenticate to GitLab; Vergil must reach GitLab and the callback endpoint over outbound HTTPS. No Power Automate-to-Vergil route is required.
- **Remaining deployment work:** note polling remains `Server::poll_once`; configured repository intake uses the independent `poll_jsonl_once` path. The mixed-project review publication gate passed locally; PA-05 still needs the external append flow and its prerequisites. Existing `FormSubmission::entry_after` remains the shared authorization and conversion contract.
- `crates/acorn-lib/src/io/api/powerautomate/` implements strict form decoding, project and submitter authorization, shared-header authentication, durable intake, and direct callback resolution.
- `crates/acorn-lib/src/io/api/webhooks/` provides constant-time secret verification, durable idempotent operations, retryable callbacks, readiness, and graceful drain state.
- `crates/acorn-lib/src/io/api/gitlab/` implements rolling merge-request automation, authorized graduation commands, and immutable report-package publication.
- `crates/acorn-cli/src/commands/report/` composes bounded artifact creation and package publication; `templates/gitlab/reporting.gitlab-ci.yml` supplies the scheduled reporting job.
- `acorn create secret` supports explicit `--paste` clipboard delivery and newline-terminated output only when standard output is redirected or piped.
- Fixture-backed tests validate the 22-entry Power Automate sample, project routing, proposal persistence, graduation, and duplicate-safe operation handling.
- Generated help, focused tests, `make lint`, all 1,914 workspace tests, and `make book` passed on 2026-09-24 for the existing HTTP intake path. Those results do not validate the newly planned JSONL path.
- The former book page and navigation entry are removed, all durable references point to this active plan, and `make book` passes on 2026-10-06.
- The source data project is the Internal `research-enablement/buckets/nssd` project, numeric ID `17410`, default branch `main`. Treat it as read-only production throughout this walkthrough; the inspected snapshot is `036c7b0074b2cc1901c0a65c66c254a04663ce74`.
- The isolated deployment target is the Internal `research-enablement/experiments/power-automate` project, numeric ID `22464`, default branch `main`. Its default branch still contains only its pre-existing README and `responses.csv`; that file was not written by the tested Power Apps notification flow. The pilot assets remain isolated on `acorn/powerautomate-pilot`.
- The NSSD snapshot contains 78 tracked files, 35 project `index.json` files, no Git LFS objects, and about 18 MiB of current working-tree data. All 35 indexes pass an offline ACORN schema check; existing readability warnings do not block the copy.
- An exact copy of the NSSD project data is published only to the isolated target's `acorn/powerautomate-pilot` branch at commit `7ccb1a7`. The source README and CI configuration were excluded, the target's existing files were preserved, and a checksum-aware dry run shows no copy drift. No merge request has been opened and the target default branch remains unchanged.
- Pilot commit `66f4bef` adds a cross-project CI include pinned to ACORN commit `aa36e93f76ec22a9b3d2be9ceccf2a8d9cd891aa`; GitLab's pipeline editor reports that the expanded configuration syntax is correct. The schedule-only job remains disabled until the branch is reviewed, merged, and explicitly scheduled.
- Pilot branch commit `ecbbda4` configured `.acorn.json` for project `22464`, mapped `acorn` to `acorn/index.json`, and allowlisted `o9w@ornl.gov`, the address returned by the signed-in Power Apps session. A dry run against that exact commit passed with `no-changes`; no package was published and no callback was sent. Commit `447fbd45084496006a70aa9c0bdd2ef5db4c1b75` adds `c5c@ornl.gov` to the same member allowlist without changing the project mapping or touching production NSSD.
- The local ACORN 0.4.0 debug binary and container tooling are available for setup work. **Vergil is the selected bot host.** The existing `vergil` Docker context reaches an ARM64 Ubuntu 24.04 host running Docker 29.1.3 and the user confirms it can host an ACORN server port. The candidate service origin is `https://vergil:3000`, which the user expects can be exposed on ORNL networks only. DNS resolution, certificate trust, and reachability from an ORNL-managed gateway host are not yet verified; Vergil has no known public HTTPS route, and `kubectl` has no current context. The user confirms that the ORNL enterprise account has Power Automate Premium with HTTP actions.
- Vergil does not currently have an ACORN image. The published `ghcr.io/jhwohlgemuth/acorn:latest` manifest is `linux/amd64` only, the repository Dockerfile builds the `x86_64-unknown-linux-musl` target, and no `aarch64-unknown-linux-musl` GitLab package is available. The pilot therefore needs a native ARM64 build on Vergil or an explicitly reviewed multi-architecture image; do not assume the published image runs natively there.
- ACORN startup resolves `POWER_AUTOMATE_INBOUND_SECRET` immediately, but resolves `POWER_AUTOMATE_CALLBACK_URL` only when delivering a callback. Polling mode does not require a public URL or GitLab webhook credentials, so a bounded first boot can prove `/health` and `/ready` before callback deployment and real JSONL intake. Do not submit a real intake until the callback URL is configured.
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
- None of the inspected flows has an HTTP action, ACORN URL, GitLab action, or intake credential. The user clarified that the test exercised only Power Apps input → Teams/email; no row was saved. The current **Sponsor Highlights** app links only to the notification flow; the new JSONL draft is separate and off. The separate **Highlight** app links to no flow and uses Dataverse. The pre-existing `responses.csv` in GitLab project `22464` is unrelated to the verified test.
- Access to the GCC maker portal and instant-flow creation are confirmed. The current environment is explicitly marked as a developer environment. The published **Sponsor Highlights** app is owned by Jason Wohlgemuth and currently has a Standard license designation; that designation does not settle licensing for the future HTTP-based intake flow. On 2026-10-17, GitLab project `22464` was confirmed Internal with `main` as its default branch; `main` permits Maintainer push and merge only. The unprotected `acorn/powerautomate-pilot` branch remains at `447fbd45084496006a70aa9c0bdd2ef5db4c1b75`, and the signed-in owner can push. No project CI variables are configured. These facts do not approve a flow credential, cloud API access, a shared GCC environment, or production owners.

# Architecture decisions

- GitLab holds the pending JSONL Logbook inbox. Power Automate appends records; Vergil durably ingests and removes them, then routes the retained work to the configured project `index.json`. Both writers use optimistic concurrency and recompute their changes after conflicts. Completed submission identities and processing outcomes remain in durable bot storage after removal, so absence from the file does not mean an ID is new. No additional cloud queue is required.
- Use the existing GitLab data project for `.acorn.json`, project `index.json` files, review merge requests, pipeline schedules, and generic-package artifacts. Do not place deployment configuration or operational research data in the ACORN source project.
- Begin with one project mapping and the two explicitly approved tester addresses. Add projects or any other submitters and recipients only after the pilot passes the duplicate-submission smoke test.
- Use an organization-only Power Apps canvas app as the primary intake interface and an instant flow with the Power Apps (V2) trigger as its adapter to ACORN. Microsoft Forms is not required by the ACORN interface.
- Treat the current developer environment as a build-and-test workspace only. The obsolete generated `Logbooks screen` and unused `cr1d0_logbooks` dependency are removed for the notification-only pilot, but confirm the cleaned app's effective license designation with the shared tester. Before broader or production use, move or recreate the app and its environment-local flow in an ORNL-approved shared or production GCC environment with sufficient sharing capacity, then share the app and each dependent resource with the approved users or security group.
- Run ACORN on Vergil as a persistent service with protected credentials and durable state. Keep health/readiness access local for the pilot. Self-signed server TLS on Vergil does not affect outbound requests; remote GitLab and callback certificates must still validate.
- Retain `--event-source poll` for GitLab review events and use the explicitly configured repository-file intake poller. The existing flag alone does not enable JSONL intake. No GitLab webhook registration, public Vergil endpoint, gateway, or relay is required for this design.
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

- Selected intake: UTF-8 JSONL at `.acorn/intake/logbook.jsonl` on one explicitly configured GitLab branch. Each nonempty line is one compact version-1 submission object, terminated by LF. The existing `POST /webhooks/powerautomate/forms` remains supported but is not the flow's destination.
- The existing form envelope requires `submissionId`, `projectId`, `submittedAt`, `submitter.email`, and an `entry` containing `timestamp`, `category`, `state`, and `summary`. `classification`, `eventType`, and `items` are optional; omit inapplicable fields instead of sending `null` or extra properties.
- Categories are `administration`, `collaboration`, `data`, `engagement`, `funding`, `infrastructure`, `method`, `other`, `personnel`, `publication`, `research`, and `software`.
- States are `completed`, `in-progress`, and `upcoming`; event types are `conference`, `exhibition`, `meeting`, `other`, `presentation`, `tour`, `training`, and `workshop`; classifications are `unclassified`, `confidential`, `secret`, and `top-secret`.
- The flow returns `queued`, `submissionId`, and the GitLab commit SHA after verifying persistence. This is distinct from ACORN acceptance. The bot records admission, rejection, and operation ID separately; its existing callback events report later workflow progress. Do not invent an HTTP `202` receipt from a repository commit.
- Callback events are `mr-created`, `review-needed`, `artifact-ready`, `report-no-changes`, and `failed`. Every callback contains `event` and `projectId`; event-specific fields carry merge-request URLs, package URLs, checksums, cutoffs, or failure messages.
- The existing `.acorn.json` names runtime secret variables, never their values. Repository-file source configuration is implemented under `options.powerAutomate.intake`; see `ACRE_SPEC.md` for branch, path, interval, limits, and admission/cleanup behavior. Polling does not require `GITLAB_WEBHOOK_SIGNING_TOKEN`.
- The scheduled reporting job runs only for a default-branch schedule with `ACORN_POWER_AUTOMATE_REPORTING=true`; `ACORN_REPORT_CONFIG` may select a non-default configuration path.

# Multi-project routing model

- **First implementation priority:** one JSONL inbox must accept interleaved records for multiple projects as well as a single-project stream. Each record resolves to exactly one canonical project; multiplexing means distributing records to their selected projects, not copying every record to every project. Prove this locally before enabling reporting or expanding live submitters.
- Accept a stable project ID or an explicitly configured folder-name alias as the intake routing selector. Resolve folder names through an approved alias-to-project-ID-to-`index.json` mapping before constructing the existing version-1 envelope; keep its required canonical `projectId` and strict schema unchanged. The settled contract above specifies the selector representation and normalization boundary. Never add an undeclared folder field to `FormSubmission` or infer a destination from prose, email, or an arbitrary filesystem path.
- Require unique, exact folder aliases within the configured source repository. Reject missing, unknown, ambiguous, traversal-like, or conflicting selectors; if an ID and folder name are both provided, they must resolve to the same project. Distinguish the research project ID from the numeric GitLab repository ID. Preserve the original selector and canonical route in durable intake evidence so a retry cannot silently switch destinations after a mapping change.
- Name the optional per-project alias list `aliases` (not `folder_aliases`). It contains explicit routing aliases, including folder names, that resolve to that project's canonical ID; omitted lists default to empty.
- Use root `folders` for the shared destination registry, with stable `id`, optional `aliases`, `members`, `variables`, and `path` in each entry. This supersedes the earlier proposed root `projects` and mapping `projectId`/`project_id` names; existing submission/RAD `projectId` fields remain compatible. Name the GitLab repository setting `gitlabProjectId` in JSON/JSONC and `gitlab_project_id` in YAML.
- Add `options.folders.root` as an absolute local or remote URI identifying the base directory/container for folder destinations (for example, `file:///C:/research/` or `https://data.example.org/research/`). An explicit root takes precedence over `options.powerAutomate.gitlabProjectId`; that GitLab setting supplies repository-relative resolution only when the root is absent. Never fall back to the ACRE file directory or process working directory. YAML uses `options.folders.root` and `options.power_automate.gitlab_project_id`.
- Resolve each folder `path` with this order: an explicit absolute URI identifies its own destination; otherwise resolve a relative path against `options.folders.root`; otherwise use the configured GitLab repository; otherwise reject the path because it must be an absolute URI. An omitted path becomes `<id>/index.json` only when a root or GitLab repository is configured. With neither configured, require an explicit absolute URI for every folder. Paths continue to designate the index file, and aliases never affect the destination.
- Treat roots as directory bases, normalizing a missing trailing slash before URI joining. Preserve URI escaping and reject traversal outside a configured root. Local absolute filesystem paths remain supported with a configured local file root, normalized to `file:` URIs on the executing host; without either root or GitLab configuration, native `/srv/...`, `C:/...`, and UNC paths are not substitutes for the required absolute URI. Use `file:///srv/research/acorn/index.json`, `file:///C:/research/acorn/index.json`, or a valid file URI for UNC destinations. Reject native paths against remote roots rather than interpreting them as remote paths.
- URI designation does not imply write support for every scheme. Validate against the selected consumer/adapter and fail explicitly for unsupported schemes or mutation operations; an HTTPS root alone does not grant repository commit capability. For GitLab writes, resolve the destination inside the selected repository or an explicitly mapped checkout and convert it to a repository-relative API path; reject conflicting repository/root targets and symlink/junction escapes. Never send a host path to the GitLab API. HTTP intake, JSONL intake, and reporting share this resolver.
- Add a global email allowlist as the top-level `members` attribute of the ACRE document, alongside root `variables` and `folders`. If this attribute is absent, its inline contribution defaults to empty; resolve any `variables.members` contribution as specified below before validating access. A submitter is authorized when present in top-level `members` OR the resolved project's `members`. Global members have intake access to every configured project, including projects added later; an omitted or empty project `members` list defaults to empty and is valid when the effective global member list is nonempty. Reject a project configuration when both its effective project member list and effective global member list are empty. Apply the same email validation and normalization to both lists. This is intake authorization, not GitLab administrator, graduation, or merge permission; unknown projects remain invalid.
- Authorize against the combined global and project member allowlists and preserve unrelated `index.json` fields and existing Logbook entries. Isolate per-record failures: a rejected or retryable record for one project must not prevent valid records for another project from being admitted. Retain existing UUID/content conflict checks, durable outcomes, and exact-record cleanup rules across all routes.
- The current pilot is intentionally fixed to `projectId` `acorn`; it proves intake, authorization, review, publication, and notification before the project list expands.
- ACORN already supports multiple mappings in `.acorn.json`. Each mapping owns one stable `project_id`, one safe repository-relative `index.json` path, and an allowlist of submitter email addresses.
- After the pilot passes, expose a required, controlled **Project** choice in the organization-only canvas app. The app maps each display label to a stable configured `projectId`; users never supply a repository path or free-form routing identifier.
- The app-triggered flow sends the selected `projectId` and `User().Email` from the signed-in ORNL session. ACORN rejects an unknown project or a submitter absent from both the global and selected project's member allowlists before any repository write; app filtering is user experience, not the authorization boundary.
- The GitLab bot resolves the authorized mapping and publishes the entry only to that mapping's resolved destination path, on a project- and month-specific review branch. The `projectId` remains attached through graduation, callbacks, and report packaging.
- Routing is explicit rather than inferred. A user who works on several projects selects the related project in the app; the current implementation does not guess a project from email or logbook prose.
- The HTTP action makes the app-triggered flow premium. Before sharing beyond the pilot, every invoking user must have suitable Premium rights or ORNL must assign a Power Automate Process license to the flow; a shared service account is not a licensing workaround.

## Member lists from environment variables

Support an optional `variables` object at the ACRE document root and within each folder, with `variables.members` naming the member-list environment variable. Root `folders` is the shared destination registry; neither root attribute belongs inside `options.powerAutomate`. Power Automate consumes this shared configuration; its integration-specific settings belong under `options.powerAutomate`, not at the ACRE root. The ACRE 2.0.0 hard cutover removed root `powerAutomate`/`projects` with no compatibility adapter: existing deployments move `powerAutomate` settings to `options.powerAutomate`, `variables` to root `variables`, and `projects` to root `folders`; conflicting legacy/root declarations are rejected by `deny_unknown_fields` rather than merged. The `members` value inside `variables` names one environment variable; it does not contain email addresses. Keep inline `members` scalar-or-array forms supported. At each scope, the effective member list is the union of inline members and the referenced variable's members, normalized and deduplicated. Authorization remains effective global membership OR effective folder membership. A folder may omit `members` when either its `variables.members` or the global sources supply access. Apply the nonempty-access validation after resolving all sources.

Landed ACRE-root configuration fragment (validated by the `with_folders.*` fixtures):

```json
{
  "variables": { "members": "ACORN_LOGBOOK_MEMBERS" },
  "folders": [
    {
      "id": "acorn",
      "aliases": ["acorn-tooling"],
      "variables": { "members": "ACORN_PROJECT_MEMBERS" }
    }
  ]
}
```

The same variables may be set in the process environment; the deployment-owned `.env` next to the resolved configuration loads once at startup with process-over-file precedence:

```dotenv
ACORN_LOGBOOK_MEMBERS=admin@example.org,maintainer@example.org
ACORN_PROJECT_MEMBERS=contributor@example.org
```

- Parse variable values as comma-separated email addresses, trimming surrounding whitespace and applying the same email normalization/validation as inline members. Empty or whitespace-only values contribute no members; reject malformed addresses and empty items within a nonempty list.
- Resolve once at startup before authorization validation, consistently for HTTP intake, JSONL intake, and reporting. Keep configuration parsing independent of process environment access; pass the resolved member lists into shared authorization. Restart the service to apply changed variables. Do not expand arbitrary JSON strings or infer folder-specific variable names from aliases.
- Keep the existing literal-member configuration valid. Document that environment members augment inline members: clearing a variable removes only its contribution, and revoking a member requires removing every applicable global/folder grant. Keep deployment `.env` contents out of source control and use placeholder examples in documentation.

## Complete ACORN and Xylem variable contract

## Attribute casing across ACRE formats

Use **snake_case attribute names in every format** — JSON, JSONC, and YAML alike — recursively across new ACRE attributes, including root configuration, options, folders, built-in variable references, and nested integration settings. Rust fields remain idiomatic snake_case, so the serialized spelling matches the field name and needs no per-format adapter. All formats decode into the same typed model and generated schemas use the same names. Pre-existing camelCase attributes (`schemaVersion`, `scriptManifest`, the `options.powerAutomate` entry name, and nested provider/model/runner profile fields such as `codeRepository`) are grandfathered and keep their current spelling; do not introduce camelCase in new attributes.

| Meaning | JSON/JSONC | YAML |
| --- | --- | --- |
| Schema version | `schemaVersion` | `schemaVersion` |
| Folder identifier | `id` | `id` |
| Power Automate options | `options.powerAutomate` | `options.powerAutomate` |
| Callback destination | `callback_destination` | `callback_destination` |
| GitLab repository | `gitlab_project` | `gitlab_project` |
| Callback URL variable | `power_automate.callback_url` | `power_automate.callback_url` |
| Inbound secret variable | `power_automate.inbound_secret` | `power_automate.inbound_secret` |

Only the grandfathered `schemaVersion`, `scriptManifest`, and `options.powerAutomate` entry names stay camelCase in every format. `members`, `variables`, `folders`, `aliases`, `path`, and `custom` have identical spelling in both formats. This casing rule applies to schema-defined attributes, not values or user-defined map keys: preserve environment-variable names, folder IDs, aliases, paths, and keys within `variables.custom` exactly. Likewise preserve opaque third-party payloads. Prose elsewhere in this plan may refer to internal snake_case field names; earlier examples marked proposed or pre-migration do not override this canonical wire-format rule.

There is one spelling per field across formats — no per-format adapters and no legacy duplicate spellings. Equivalent JSON/JSONC and YAML fixtures resolve identically, as the `with_folders.*` fixtures prove. The separate Power Automate submission/callback wire contracts retain their existing casing unless separately versioned; ACRE casing does not rewrite JSONL envelope fields.

## Variable coverage and custom references

## Buckets, folders, and local operation

A **bucket** identifies a collection of research-activity files and its source/provider configuration. A **folder** identifies one addressable activity destination, with a stable ID, aliases, index location, and membership. Current bucket import/download operations enumerate and transfer collection contents; merely declaring a bucket does not copy anything. A folder's resolved path specifies where an operation acts, independently of its collection membership.

- Add optional `folders[].bucket` as a string bucket name or zero-based integer index into the declared `buckets` array. Strings resolve only unique explicit `buckets[].name` values; integers reference concise or detailed entries by position before filtering, discovery, or transfer. Never coerce a numeric string such as `"0"` into an index or infer references from display/repository names. Reject unknown names, negative/fractional/out-of-range indexes, booleans, and invalid types. Prefer stable names for long-lived configurations; indexes support concise bucket syntax.
- Preserve bucket array order during formatting and serialization. Index `0` is a meaningful value and must never be omitted as empty. Reordering buckets intentionally changes index associations for newly admitted work; persist the resolved credential-free bucket source identity and destination with each admitted operation so retries cannot be redirected by later reordering. Preserve string-versus-integer reference types across JSON/JSONC/YAML round trips.
- One bucket may contain many folders; each folder has at most one explicit bucket association. Standalone folders are valid. Keep folder IDs unique across the ACRE document so JSONL selectors remain unambiguous. Downloaded copies and snapshots do not create additional writable owners automatically.
- Association does not introduce a path-resolution fallback: absolute URI, explicit `options.folders.root`, and GitLab fallback retain their established order. With neither root nor GitLab configuration, a folder still requires an explicit absolute URI even when it has a bucket reference. Do not infer a folder root from the bucket source.
- Validate that an associated folder belongs to the referenced collection using provider-aware identity and containment, not a textual URI prefix. A local mirror of a remote collection requires an explicit verified mapping before it can serve as an associated writable destination. Conflicting or unverifiable associations fail clearly; the reference representation (`BucketReference`: unique string name or zero-based integer index) is settled and implemented, while provider-aware containment validation is implemented in JSONL admission.
- A standalone `file:` URI points directly to an existing local `index.json`; its parent directory is the activity's associated-file base. Supported operations work in place without requiring a bucket import/copy step. Do not infer a Git repository, remote destination, synchronization, or GitLab review workflow merely because the path happens to lie inside a checkout.
- A local bucket describes a collection that an import command may copy to a requested destination. Declaration alone performs no transfer. Adding or removing `folders[].bucket` does not itself copy, relocate, or synchronize data: membership and transfer are separate concerns. Downloads do not silently redirect later folder writes to the downloaded copy.
- Bucket discovery does not automatically register folders for JSONL routing or grant intake access. Only registered folders use the global/folder membership contract; a bucket reference grants no additional authorization.
- Preserve human review for standalone local JSONL intake. `acorn intake stage` writes a durable, reviewable proposal; `acorn intake apply` explicitly changes canonical `index.json` after checking the staged base. Local paths do not create GitLab merge requests. Bucket-associated folders and nonlocal destinations fail explicitly in this adapter.

Document these distinctions in `ACRE_SPEC.md` and the ACRE/configuration guides during implementation, including examples of a bucket with associated folders, a standalone local folder, and a local bucket import into a separate destination. Explain that standalone local access avoids an import/copy step, while bucket association alone never triggers copying. Document discovery versus registration, path precedence, authorization, local review, and remote-mirror limitations together.

## Options coverage

## OneOrMany for all ACRE array attributes

Refactor every schema-defined array-valued ACRE attribute, at the root and recursively in nested configuration, to use the shared `OneOrMany<T>` abstraction. Accept either one item in its existing item shape or an array of those items in JSON, JSONC, and YAML. This includes `buckets`, `folders`, global/folder `members`, `aliases`, `models`, `endpoints`, `runners`, `scripts`, hook lists, whitelist lists, workflow capability/action lists, and all other nested array fields found by the schema inventory. Maps remain maps, and opaque third-party payload arrays retain their own contracts.

Current evidence: `acorn-core::util::types::OneOrMany` already exposes slice/iterator access and shape-preserving mapping; `acorn-schema::types::OneOrMany` also exists, and ACRE already uses it for some whitelist settings. Verify their serialization/schema/validation capabilities and reuse the appropriate shared implementation rather than adding a third wrapper. Scope any shared-type changes to avoid changing unrelated RAD or provider wire contracts.

- Every ACRE array schema path and its item type, including feature-gated configuration mirrors and ACORN consumers, is migrated: declarations, constructors/builders, iteration, nested validation, schema generation, configuration writers, and downstream consumers. Preserve existing array input compatibility and item validation; scalar syntax is not permission to coerce invalid item types.
- Preserve the supplied one-versus-many representation when serializing nonempty values: singleton input remains one item, array input remains an array even with one item. Programmatic constructors choose the variant explicitly; conversion between formats preserves it. Empty optional `Many` values are omitted under the existing empty-attribute rule; required nonempty collections still fail validation. Do not wrap a scalar in a new array or collapse an existing array merely during formatting.
- Normalize to a logical ordered sequence for application behavior. A singleton `buckets` source has length one, so bucket index `0` selects it and `1` is invalid. Preserve order, duplicate checks, numeric-string-versus-integer references, and durable resolved identity. A singleton detailed object must not be mistaken for a map of entries; retain concise/detailed bucket item unions inside `OneOrMany`.
- Inline `members: "person@example.org"` means one validated email; multiple inline emails use an array. Comma-separated parsing remains specific to environment-variable values. A single folder object can still contain its own scalar or array `members` and `aliases` independently.
- Generate format-specific schemas accepting both forms for every migrated field. Test singleton, one-element array, multi-element array, empty optional omission, invalid item types, nested validation paths, duplicate members/IDs, index zero, and semantic parity across JSON/JSONC/YAML. If an item type itself accepts arrays, settle its disambiguation explicitly in the field inventory rather than relying on untagged-deserializer ordering.
- Update `ACRE_SPEC.md`, ACORN configuration documentation, and the complete examples with scalar/object and array alternatives. This refactor is landed as the PA-05 baseline; it is not a later convenience feature.

## Option counterpart implementation

Implemented `options.<name>` attributes are `options.crosswalk`, `options.folders`, `options.powerAutomate`, and `options.scripts`, with explicit exclusions: no `options.schemaVersion`, `options.config`, or `options.options`. Script-manifest and script settings combine under `options.scripts`; there is no `options.scriptManifest`. Root `schemaVersion`, `config`, `options`, and `scriptManifest` remain unchanged. The implemented root inventory is `schemaVersion`, `buckets`, `config`, `endpoints`, `folders`, `members`, `mcp`, `models`, `options`, `registries`, `runners`, `scriptManifest`, `scripts`, `variables`, `whitelist`, and `workflow`. Internal skipped Rust fields such as `cst` are not ACRE attributes. Broader per-root option counterparts are deferred as non-blocking for JSONL.

`options.powerAutomate` and `options.crosswalk` remain additional operation-specific settings even though they have no root counterpart after migration. `options.scripts` is the single owner of options affecting both root `scripts` and root `scriptManifest`; retain their distinct root data representations and avoid duplicate or conflicting option definitions. There are no reserved-slot placeholders or `unimplemented!()` option handlers: every implemented option is functional, unsupported fields fail validation, and omitted options simply have no effect.

## Variable catalog requirements

## Omit empty configuration attributes on serialization

Canonical ACRE JSON, JSONC, and YAML output omits absent/`None` optional attributes and empty maps via `skip_serializing_if`. This covers unused `options` counterparts when absent, empty `members`/`aliases` when `None`, and empty `variables.custom`. An accepted empty `options.scripts: {}` is preserved with `enabled` defaulting to `true`. Meaningful `false`, numeric zero, nonempty values, and explicitly configured values equal to defaults are preserved. Do not emit placeholder `{}`, `[]`, or `null` attributes just to advertise schema support; document available options in the schema/specification instead.

Validate before serialization: missing required values and invalid explicit empty paths or environment-variable references remain errors, not values silently removed to activate defaults. Preserve opaque payloads and user-defined data according to their schema; do not implement this as an indiscriminate JSON cleanup pass that removes array elements or rewrites custom values. Keep the document root valid even when all optional fields are omitted. Where an existing field uses explicit emptiness to mean something different from absence, preserve that distinction through an explicit typed representation and migration rather than silently changing behavior.

Use the same omission policy for model serialization, configuration writing, format conversion, and JSONC formatting, including the existing CST-preserving writer. Preserve surviving comments/order where supported. Add idempotence and semantic round-trip tests for each format, including false/zero values, required-field errors, and meaningful opaque values. This is ACRE configuration serialization; it does not change JSONL wire envelopes or environment precedence for explicitly empty process variables.

## Variable references (landed scope)

Landed `variables` support covers intake authorization and named Power Automate references: `variables.members` plus the `custom` map (including `power_automate.callback_url` and `power_automate.inbound_secret`), globally and per folder. A full typed catalog covering **all environment variables used by ACORN and `../xylem`** — grouped built-ins, generated/read-only workflow variables, forwarding contracts, and a cross-repository coverage gate — is deferred as non-blocking for JSONL and is not claimed here.

The notes below record that deferred scope for a future plan; they are not PA-20 prerequisites. Xylem does not parse ACRE and owns no duplicate ACRE model.

Initial source evidence reviewed on 2026-10-13 includes ACORN's `crates/acorn-lib/src/util/constants/env.rs`, CLI environment bindings, server configuration, provider token fallback chains, dynamic MCP/OCI credential references, and detached-container forwarding. Xylem revision `93cbfc0` includes `XYLEM_LOG_LEVEL`, `CI`, explicit `--env` loading with process-value precedence, and generated workflow variables through `Config::get_environment_variables`. This is starting evidence, not an exhaustive inventory or proof of implemented coverage.

- Inventory both repositories' runtime reads, environment-name constants, CLI bindings, dynamic references, template expansion, subprocess/container forwarding, CI/build scripts, examples, tests, and documented variables. Record every name or dynamic family, source location, owning consumer, type/parser, fallback order, default, sensitivity, scope, and whether it is read, generated, or forwarded. Include platform variables and compile-time/test variables with their actual lifecycle; do not silently omit them from the catalog.
- Give every fixed built-in variable a typed catalog entry. Cover provider credentials/hosts, server authentication/TLS/tracing, database/cache settings, logging/TUI, inference/model acquisition, HTTP policy, export/reporting, GitLab/CI, Power Automate, platform paths, and Xylem runtime/workflow settings. Preserve established aliases and fallback precedence. Generated workflow and compiler-owned variables remain represented as generated/read-only entries rather than becoming overridable runtime inputs.
- Add `variables.custom` as a map from a logical custom name to an environment-variable reference. Example: `"custom": { "notification_channel": "ORG_NOTIFICATION_CHANNEL" }`. Values name variables, never embed resolved values. Resolve through the same process-environment-over-deployment-`.env` mechanism; validate names and missing explicitly referenced values. Custom values are opaque strings unless their explicit consumer supplies a parser.
- Reject unknown built-in field names and collisions with reserved built-in/generated names; custom entries must be declared under `custom`. A project may supply its own custom map, overriding global custom references by logical key only for that project's consumers. Member authorization still unions global and project grants. Do not permit project configuration to override process-wide server credentials, CI identity, or compiler/workflow-owned state.
- Expose resolved custom values to explicitly named workflow/template consumers without exporting the entire parent environment or mutating global process state. Require an explicit forwarding contract for child processes/containers, preserve Xylem's generated-variable ownership, and redact resolved custom values by default. Adding a custom variable does not automatically grant it execution or forwarding semantics.
- Add a coverage gate that compares actual environment-use sites in both repositories against the catalog, including reviewed dynamic-name families. Newly introduced uncatalogued reads, bindings, or forwarding entries must fail that gate. Generate schema/documentation from the same catalog and verify behavior with representative runtime, credential, generated, and custom-variable cases. No claim of complete coverage is accepted until both repositories reconcile with zero unexplained omissions.

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

- **Production source project:** `research-enablement/buckets/nssd` is a read-only input. Do not commit, push, register webhooks, create variables or schedules, or run write-capable automation against it.
- **Isolated GitLab data project:** `research-enablement/experiments/power-automate` owns the copied data, `.acorn.json`, `.gitlab-ci.yml`, and proposed `.acorn/intake/logbook.jsonl`. Power Automate appends; Vergil removes durably ingested records. The bot augments only the `index.json` selected by the trusted project mapping, through the established review workflow.
- **Power Platform:** one organization-only Power Apps canvas app, one app-triggered intake cloud flow, and one callback notification cloud flow.
- **Managed runtime:** one ACORN service with protected GitLab credentials, callback URL, persistent operation/outcome storage, and any secret still required by existing startup. No webhook signing token is required for polling.
- **GitLab control plane:** approved intake-branch write permissions, protected CI variables, and one monthly default-branch reporting schedule. Configure intake-only commits so they do not trigger unintended reporting jobs.
- **Local MVP evidence:** The native executable now exposes one-shot intake, diff export, and explicit callback delivery. Source polling, durable admission/outcomes, mixed-project routing, note authorization, diff receipt replay, and callback worker tests passed locally. Live GitLab and Power Platform acceptance remains in PA-05 through PA-09.
- After the deployed acceptance criteria pass, convert the verified process and redacted evidence requirements into `.wiki/assets/powerautomate-logbook-reporting.md`, redirect dependent wiki plans to that asset, delete this plan, and remove its index, network, and lane entries.

# Dependencies and stop gates

## Next steps with the available servers

The completed PA-24 integration gate covers a mixed-project inbox using both project IDs and folder aliases. Keep that routing contract during reporting and notification deployment.

1. Finish the Vergil bot health, readiness, and isolated GitLab access checks in polling mode.
2. Test a policy-approved Power Automate callback flow from Vergil over outbound HTTPS and confirm Teams/email delivery. Store the callback URL securely before real intake.
3. Select an app-triggered flow in the canvas app's environment and verify the approved GitLab target, branch, credential, and cloud reachability for the new JSONL append using the existing Logbook submission envelope.
4. Configure the flow to append JSONL and enable the implemented poller after the remaining access and callback prerequisites. Preserve submission UUIDs across retries, durably record admission, and distinguish queued receipts from ACORN acceptance. This is GitLab-mediated intake without a gateway dependency.
5. Once transport is validated, run one isolated submission through human review, graduation, merge, report publication, callback delivery, and duplicate replay as described in steps 7–9.

The selected route is Power Automate → GitLab → Vergil. Verify reachability and TLS trust on these actual connections; direct HTTP intake on Vergil is not part of this walkthrough.

## Gates for full intake

Do not enable full intake or production flows until all of these gates pass. A protected local intake credential is permitted for the bounded bot boot. The notification-only canvas pilot is explicitly allowed before gates 2 and 3 because it does not call ACORN, GitLab, or an HTTP trigger. An isolated callback test may proceed before gate 2 once callback policy and recipients are approved:

1. Power Automate Premium with HTTP actions and maker access to a GCC Moderate Power Apps developer environment are confirmed for `o9w@ornl.gov`; this proves build capability, not a deployable shared environment or end-user entitlement. The notification-only app no longer has its Dataverse source, is shared with the one approved tester, and the tester launched the cleaned version without a trial prompt. Her 2026-10-07 submission produced both visible notifications. Before broader use, obtain an ORNL-approved shared or production GCC environment with sufficient sharing capacity and verify the resulting app license designation and dependent-resource access. Use the GCC maker portals, not the commercial endpoints.
2. Power Automate can read and update the selected GitLab file, Vergil can read it, and the new JSONL poller passes its acceptance tests. Keep the existing app notification-only until the complete intake path is ready. Callback testing and local bot checks may proceed independently.
3. The tenant permits the callback trigger's `Anyone` mode, or an approved intermediary exists for callbacks.
4. The isolated GitLab data project supplies its numeric project ID, default branch, package permission, one safe repository-relative `index.json` path, one stable pilot identifier, and the explicitly approved tester email allowlist. No deployment credential or integration may target the production source project.
5. Named owners exist for the Vergil ACORN service, Power Apps canvas app and flows, GitLab review and merge, Teams destination, and email recipient list.

The implemented webhook runtime, GitLab automation, reporting-window selection, PDF renderer, package manifest, and reporting CI template are otherwise sufficient for the pilot.

# Human and Codex responsibilities

| Owner | Responsibilities |
| --- | --- |
| **You** | Confirm GitLab access, intake branch, trusted writers, and credentials; coordinate the shared GCC environment and licensing; approve notification destinations; publish the accepted canvas draft; review, graduate, and merge GitLab changes. |
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

- **Published evidence (2026-10-07):** `SubmitEntryScreen` implements the pilot field set and controlled values, opens first, and calls the environment-local notification flow with six inputs. Required-field validation, pending-state disabling, success/failure status text, and a current Teams-and-email tooltip are active. Default and 360-by-800 previews render the complete form with a scrollable body and fixed status/footer. The pilot ACORN allowlist contains the signed-in `User().Email` value plus `c5c@ornl.gov`; that allowlist is independent of the Power Apps grant. The obsolete generated screen and Dataverse source are removed, the saved Data pane is empty, and the cleaned version is published. Power Apps granted Clark, Niki user-only access after resolving `c5c@ornl.gov`; she opened the clean launch without a trial prompt and later confirmed both notifications.

- **Verify:** the shared tester opens the clean launch link without a Power Apps trial prompt, sees the contributor screen and her signed-in ORNL identity, establishes the required flow, Teams, and Office 365 Outlook connections, and can submit the notification-only pilot. In an approved shared or production GCC environment, explicitly share the published app only with approved testers, confirm every tester has the required entitlement for the app's final license designation, and ensure all dependent resources are accessible. The app contains no shared secret, callback URL, or repository path; a blank form defaults to the current time, cannot submit an invalid controlled value, has no ordinary-contributor edit or delete path, and exposes meaningful accessibility names for every actionable control.

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

Use the current ACRE 2.0.0 layout with the resolved pilot values. Keep `intake` absent during the bounded health/readiness boot so existing inbox records cannot start real work before callback readiness. Only UTC reporting is supported.

```json
{
  "schemaVersion": "2.0.0",
  "folders": [{
    "id": "acorn",
    "members": ["c5c@ornl.gov", "o9w@ornl.gov"],
    "path": "acorn/index.json"
  }],
  "variables": {
    "custom": {
      "power_automate.callback_url": "POWER_AUTOMATE_CALLBACK_URL",
      "power_automate.inbound_secret": "POWER_AUTOMATE_INBOUND_SECRET"
    }
  },
  "options": {
    "powerAutomate": {
      "callback_destination": "powerautomate",
      "gitlab_project": "22464",
      "package": "acorn-automated-artifacts"
    }
  }
}
```

Deploy the service with protected runtime credentials on Vergil. The paths below are examples to adapt to the approved service account and installation. Install an ACORN binary built for Vergil's ARM64 Linux host; the published image is AMD64 only. Keep `/etc/acorn/vergil.env` readable only by the service account. It must supply `CI_SERVER_HOST=code.ornl.gov`, a bot-specific `GITLAB_TOKEN`, a persistent `ACORN_DATABASE_PATH`, and `POWER_AUTOMATE_INBOUND_SECRET` while `options.powerAutomate` is configured. Supply `POWER_AUTOMATE_CALLBACK_URL` before enabling real intake. Store values only in the protected environment file, never in this plan, the repository, or the command line.

For the bounded first boot, leave `options.powerAutomate.intake` absent and use polling mode without registering a webhook. `POWER_AUTOMATE_CALLBACK_URL` may remain absent until the callback flow is ready. Run the service in the foreground while checking its startup logs and local endpoints:

```bash
cd /srv/acorn-pilot
uname -m
set -a
. /etc/acorn/vergil.env
set +a
/usr/local/bin/acorn serve bot 22464 \
  --bind 127.0.0.1:3000 \
  --config /srv/acorn-pilot/.acorn.json \
  --event-source poll
```

In another shell on Vergil while the process runs:

```bash
curl -fsS http://127.0.0.1:3000/health
curl -fsS http://127.0.0.1:3000/ready
```

- **Verify the bounded boot:** `uname -m` identifies ARM64, the installed executable runs natively, `/health` returns `ok`, `/ready` returns HTTP 200, and startup logs contain no secret values. Stop here if the configuration names production project `17410`. Do not use the current detached container helper for this boot: it does not forward these protected Power Automate values, and the published image is AMD64 only.
- **Verify the selected integration route:** the actual Power Automate run can authenticate to GitLab, and Vergil can read the same committed JSONL file and deliver outbound callbacks. No incoming cloud connection to Vergil is part of this test.

## 5a. Enable the implemented repository-file intake

After callback readiness, set `POWER_AUTOMATE_CALLBACK_URL` in the protected environment file, configure `options.powerAutomate.intake` with the same branch/path used by the selected flow, and restart with the command in Step 5. The defaults are `main`, `.acorn/intake/logbook.jsonl`, 30 seconds, `max_file: "10MB"`, and `max_record: "64KB"`; set the pilot branch explicitly until its reviewed cutover. See `ACRE_SPEC.md` and the ACRE guide for the implemented contract. The `--event-source poll` flag handles GitLab review notes; it does not enable JSONL polling without `options.powerAutomate.intake`.

Restrict intake-branch writers to trusted producers and derive reporter identity from the authenticated app context. Confirm durable outcomes through the operation store; removal means the work is recoverable, while later worker success establishes review publication. Verify authorized graduation through note polling separately. Record the source coordinates, reviewed service revision, and redacted acceptance evidence before enabling real submissions.

## 6. Append Logbook submissions to GitLab JSONL (planned)

Keep the app-triggered flow in the canvas app's environment. Build the GitLab write path to append compact JSON objects to `.acorn/intake/logbook.jsonl` on the explicitly selected pilot branch in project `22464`. The verified notification flow provides no existing GitLab connection or persistence action to reuse.

Each line uses the existing version-1 `FormSubmission` envelope. Its nested `entry` models the contributor-owned fields of `LogbookEntry`; ACORN derives `identifier` and owns `origin`, `discovery`, and `archived`. Use typed Power Apps (V2) inputs, UTC RFC 3339 timestamps, and optional fields only when supplied. The expanded object below documents the fields; serialize it compactly onto one physical line followed by LF. A JSON serializer must escape embedded newlines and quotes rather than interpolating raw form text.

```json
{
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

- **Draft evidence (corrected 2026-10-17):** the eight-input default-environment scaffold prepares UUIDs, timestamps, and optional item arrays but cannot connect to the canvas across environments. The environment-local six-input flow only sends notifications. A separate environment-local eight-input JSONL draft now generates a submission ID and UTC timestamp, but is off and has no persistence or app binding. The user confirmed there was no CSV write in the tested path. The external JSONL append remains unimplemented; repository consumption and mixed-project publication are implemented and locally tested.

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

- **Source configuration landed:** branch/path/interval/size limits, numeric GitLab project reconciliation, and startup `.env` loading with process-over-file precedence are implemented and covered by the focused configuration suite.
- **Snapshot reader landed:** immutable commit reads reuse the existing GitLab client and SHA-256 spelling, enforce file/record bounds, reject nonterminated tails, and preserve content, line positions, and file concurrency metadata. Missing inboxes are idle; authentication, network, and size failures remain errors.
- The 2026-10-07 canvas version is published. The live launch confirms `SubmitEntryScreen` is the start screen; preview also verifies it at the default canvas size and at 360 by 800. The controlled category list contains only the placeholder plus the twelve documented categories. The environment-local notification flow is attached, the blank form keeps **Submit highlight** disabled, the tooltip is `Submit highlight to Teams and email`, and the status is `Ready to send Teams and email notifications.`
- App checker reports zero formula issues for the saved draft. Its 21 missing-label and two missing-tab-stop errors are confined to named controls on the pre-existing generated `Logbooks screen`; no new contributor control appears in those findings.
- Notification-only run `08584102113387025966515574751CU09` completed successfully in three seconds on 2026-10-07 from the unpublished canvas. The enriched templates were saved afterward. Published-app tester run `08584102031146888583116268486CU06` succeeded after that save; the Power Apps trigger, Microsoft Teams direct-chat action, and Office 365 Outlook email action all report **Succeeded**, with no HTTP, ACORN, GitLab, or NSSD action. The tester's 16:10 Teams screenshot shows the expected pilot fields, and her 16:11 message confirms the Teams and email deliveries matched. The private conversation is the retained evidence location; no unredacted image is committed here.
- Record screenshots or redacted exports showing the premium HTTP action and callback trigger policy without exposing credentials.
- Record health/readiness, the flow's queued receipt and commit SHA, and the bot's durable admission/rejection outcome and operation ID.
- Record the GitLab branch and merge-request URLs, authorized graduation note, merged commit, pipeline URL, package version, manifest, and PDF checksum.
- Record the Teams and email deliveries for `artifact-ready` and the callback flow's successful 2xx run.
- Record the duplicate replay response and confirm the data-project diff contains only one normalized entry.
- Confirm generated secrets, shared-secret headers, callback URLs, SAS query parameters, and GitLab tokens are absent from repository history, service logs, artifacts, and visible flow histories.
- **Local MVP validation (2026-10-17):** `cargo build --package acorn-cli --bin acorn` produced `target/debug/acorn`; all four intake CLI tests passed, including mocked MR-base diff export/replay and missing-branch rejection. A one-shot CLI smoke against a local empty GitLab mock and an idle callback CLI smoke both returned parseable zero-work JSON. The standalone fixture `stage`/`apply` smoke changed only its scratch `index.json`. The GitLab JSONL cleanup/restart suite (13 tests), single/mixed review publication test, authorized graduation through the manual worker, note poll authorization tests, and durable callback success/retry tests passed. Headless Chrome produced the report PDF in the focused report test after running outside the filesystem sandbox. Live append, concurrency, restart, and callback evidence still belong to the external pilot. Preserve redacted evidence separately from local test output.

# Acceptance criteria

- A single-project stream and an interleaved multi-project JSONL stream both ingest successfully. Tests route records by canonical ID and configured folder alias, including an alias different from its ID, and verify only each destination's mapped `index.json` changes through review.
- Missing/unknown selectors, duplicate folder aliases, conflicting ID/folder pairs, unsafe paths, and unauthorized project/member combinations produce inspectable rejections without project-data changes. A bad record does not block other valid projects; replay and restart do not duplicate entries or change a previously admitted route. These are ingestion implementation gates before reporting rollout.
- Power Automate appends one compact, schema-valid Logbook submission envelope per line to `.acorn/intake/logbook.jsonl`. Concurrent appends and bot removals lose no submissions; retries after removal produce no duplicate processing effects.
- Vergil reads immutable snapshots, durably ingests complete items before removing exact matches from the inbox, and routes each by `projectId` to augment only its configured `index.json`. Rejections stay visible, admitted payloads survive removal, and restart/crash replay produces no duplicate effects.
- All licensing, GitLab reachability/authentication, callback policy, ownership, and pilot-mapping gates pass. No gateway or public Vergil endpoint is required.
- One organization-only Power Apps canvas app and two protected Power Automate flows implement the documented intake and callback contracts.
- The Vergil service remains healthy and ready, authenticates the configured source and review events, and preserves durable receipts after inbox cleanup. Failed project updates remain retryable from bot storage.
- Unauthorized writes/records fail closed. The app distinguishes GitLab persistence from ACORN admission; one valid queued submission produces one expected monthly review entry and merge request.
- Human graduation preserves the routed project identifier, and a person—not the bot—merges the reviewed entry.
- The scheduled job publishes one immutable report package and Power Automate sends both Teams and email notifications for the accepted artifact. An operator can explicitly deliver queued callbacks through the ACORN CLI after the callback flow is ready.
- An operator can use ACORN's public API/CLI operations to process the same GitLab JSONL submissions without a running bot: admit and route records, review and apply proposed changes under the same authorization and duplicate rules, produce the GitLab review update, and export a diff artifact for the changed project files. Artifact generation is independently invocable. Record the exact commands or API calls and compare their results with the bot path; this manual-parity gate is not yet implemented or validated.
- Replaying the same submission UUID is idempotent and does not create a second entry, merge request operation, package, or notification event.
- Before multi-project rollout, two authorized project selections route only to their configured `index.json` files and an unauthorized project/member combination creates no canonical project-data change.
- No credential appears in source control, logs, artifacts, app controls, ordinary Compose actions, or visible Power Automate run history.
- The redacted evidence set is sufficient for another operator to verify every acceptance criterion; the verified process can then transition to `.wiki/assets/powerautomate-logbook-reporting.md`, durable references can be redirected, and this plan and its active index entries can be deleted.

# Execution phases

## Settled JSONL contract (PA-10 output)

- **Selector:** the wire `projectId` remains the single routing selector and accepts a canonical folder ID or a configured folder-name alias. Resolution uses `FolderRegistry::folder` (exact match; unknown, ambiguous, and conflicting selectors rejected). No new envelope field; destinations are never inferred from prose, email, or filesystem paths.
- **Source configuration:** an optional `intake` sub-object under `options.powerAutomate` carries branch, path (default `.acorn/intake/logbook.jsonl`), poll interval, and file/record size limits, following the asset examples. PA-20 reconciles the config `gitlab_project: String` with the GitLab `Options` numeric identifier.
- **Member resolution:** deployment-owned `.env` next to the resolved configuration plus process-over-file precedence, resolved once at startup and shared by HTTP and JSONL intake (implemented). Current callers share the startup-loaded process environment.
- **Canonicalization:** each JSONL line parses as strict `FormSubmission` (`deny_unknown_fields`) and converts via `entry_after` (trim/normalize, then validate/authorize/cutoff). No second Logbook schema (implemented).
- **Stable routing:** the resolved folder ID, destination, and content digest persist at admission; retries and replays keep the original route across later mapping changes (implemented).
- **Local destinations:** GitLab phases fail clearly on local-only destinations; standalone-local intake uses `acorn intake stage/apply` and never implies GitLab review.

## PA-02 — Resolve production owners and access gates

- Status: in-progress
- Budget: 5 active hours maximum
- Prerequisites: none; the notification-only pilot is verified
- Exact write set: Power Platform environment policy, isolated GitLab project `22464` access policy, this plan
- Primary integration lock: deployment approvals
- Outcome: Name owners and record approved environment, GitLab API route for the future app-triggered flow, callback-trigger, licensing, branch permissions, and secret-storage decisions. No gateway approval is required.
- Focused validation: Verify every stop gate has an accountable owner and dated evidence.
- Handoff: Remove PA-02 and record the approved GitLab target and outbound access requirements for PA-03 and PA-04.

### PA-02 gate audit, 2026-10-17

| Gate | Verified evidence | Decision still needed |
| --- | --- | --- |
| Power Platform | Published tester pilot works in the `221f…` developer environment; **Sponsor Highlights** is owned by Jason Wohlgemuth and currently designated Standard. Enterprise Test and Enterprise Prod appear as viewable environments. | ORNL-approved shared GCC environment, maker and sharing rights, dependent-resource access, and final-flow licensing under [Microsoft's current instant-flow rules](https://learn.microsoft.com/en-us/power-platform/admin/power-automate-licensing/faqs). |
| GitLab | Internal project `22464`, `main` protected for Maintainers, pilot branch unprotected at `447fbd45084496006a70aa9c0bdd2ef5db4c1b75`; signed-in owner can push. Pre-existing `responses.csv` on `main` is unrelated to the Power Apps test; no project CI variables are configured. | Approved branch and separate flow/bot principals with the required scopes; approved API route for the future flow. Live reachability and write proof belong to PA-05. |
| App-triggered intake flow | The tested **Sponsor Highlights** app attaches only the notification flow. A separate JSONL draft in the same environment is off and has no associated app; **Highlight** has a Dataverse source and no attached flow. The default environment has **ACORN Logbook Intake** and **Test**. The user confirmed no CSV or JSONL save occurred in the tested path. | After the environment and access decisions, finish the draft's JSONL append, secure its actions, and attach it to the canvas only when the queued receipt is valid. |
| Callback | No callback flow or trigger policy was established by the notification pilot. [Microsoft describes `Anyone` as a legacy open-access trigger mode](https://learn.microsoft.com/en-us/power-automate/oauth-authentication). | Tenant approval for that mode or an approved intermediary, plus approved recipients. |
| Secrets and operations | Vergil is the selected ARM64 host; no deployment credential was placed in the repository during this audit. | Protected stores and owners for flow/bot GitLab credentials, callback URL, and Vergil service; named app/flow, review/merge, Teams, and email owners. |

### Handoff

- Verified baseline: published tester notification delivery and the read-only Power Platform and isolated GitLab observations above.
- Changed surfaces: this plan and its portfolio references. Opening the separate **Highlight** app for inspection caused Power Apps to create unpublished version 2 at 14:09:58 on 2026-10-17; version 1 remains live. No Save or Publish action was used, and no flow, GitLab branch, credential, or service was changed.
- Validation run: Power Automate run details, tester-shared Teams evidence, Power Apps editor inventories, and read-only GitLab project, branch, protection, member-role, and file-list queries.
- Remaining defect: written deployment decisions are not yet identified. Current branch access for the signed-in user does not establish service-credential access or cloud reachability; no CSV or JSONL write was part of the tested flow.
- Next action: obtain the existing policy/owner records, verify the approved integration route, and record dated decisions before removing PA-02.

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
- Prerequisites: PA-03, PA-04 (source configuration, snapshot reader, durable admission, cleanup, poller, and mixed-project publication are landed)
- Exact write set: selected Power Apps intake flow, canvas submit action, isolated project `22464` inbox `.acorn/intake/logbook.jsonl`, Vergil JSONL source configuration
- Primary integration lock: intake route
- Outcome: Append the strict version-1 envelope with conflict-safe GitLab retries, configure the implemented repository-file poller, and return a confirmed queued receipt with submission ID and commit SHA.
- Focused validation: Verify stable UUID/content across retries, concurrent append and cleanup, ambiguous responses, serialization round-trips, protected credentials, and app distinction between GitLab persistence and ACORN admission.
- Handoff: Remove PA-05 and record the tested flow revision, source project/branch/path, and poller revision.
- Draft checkpoint: Flow `6039f419-420f-6922-46ef-04b00d2290f1` is saved and off with typed trigger inputs plus one-time ID and timestamp actions. Resume with the approved branch and secret references from PA-02/04, service/callback readiness from PA-03/04, then implement GitLab read/conditional write/retry, secure run history, confirm commit persistence, return the queued receipt, bind the canvas, and configure the poller. Do not enable or attach the draft before those pieces work together.

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
