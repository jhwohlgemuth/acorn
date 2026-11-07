---
type: Plan
title: "Power Automate Teams workflow bot"
description: "Package and accept a Workflows-authored Teams assistant whose solution-aware flows provide durable, policy-bounded capabilities."
difficulty: high
impact: high
package: [power-platform, teams, dataverse]
tags: [powerautomate, teams, workflows, bot, dataverse, ai-builder]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Build a feature-rich Teams assistant as a Power Platform solution. Microsoft Teams supplies the conversation surface, the tenant's **Workflows** identity supplies outbound messages, Power Automate child flows supply capabilities, and Dataverse supplies durable session, request, job, effect and receipt state. The release must behave like one bounded product even though its implementation is a graph of independently testable flows.

The first release is a Power Automate-native assistant. Copilot Studio or a custom Teams app is an optional ingress/identity upgrade, not a prerequisite. PAB-02 is an explicit stop gate: if the native Workflows chat cannot provide acceptable correlation, privacy and user experience, preserve the service flows and replace only the conversation adapter rather than weakening the acceptance criteria.

# Current evidence

The November 5, 2026 test vertical slice is accepted and therefore is not retained as a remaining execution phase.

- Environment: Power Platform environment `221f44a6-1468-ea4e-b93d-cd52f1723079` (**Wohlgemuth, Jason**, GCC).
- Request flow: [`[TEST] Summarize Text - Request`](https://make.gov.powerautomate.us/environments/221f44a6-1468-ea4e-b93d-cd52f1723079/flows/a4fa6058-0c5c-41df-aa23-a29f845b9558?v3=true). The failed run `08584077092374123994299799564CU05` used **Create a chat** with the same account twice and received HTTP 400, `Duplicate chat members is specified in the request body`. The repaired flow removes chat creation and posts directly as **Flow bot**, displayed in Teams as **Workflows**, to the triggering user's `x-ms-user-email`. Run `08584077085257915081702175710CU05` succeeded in 638 ms and the prompt appeared in the Workflows chat.
- Reply flow: [`[TEST] Summarize Text - Reply`](https://make.gov.powerautomate.us/environments/221f44a6-1468-ea4e-b93d-cd52f1723079/flows/60809869-9eb3-4219-a973-1ada706b5c2a?v3=true). It listens for new chat messages, retrieves message details, requires `plainTextContent` to **start with** `SUMMARIZE:`, invokes the AI Builder summarize prompt, and posts the result as **Flow bot/Workflows** to `item()?['conversationId']`. The prefix check prevents the instructional prompt and generated summary from recursively invoking AI.
- End-to-end acceptance: a synthetic user reply at 12:14 ET produced a Workflows-authored summary in the same chat. The AI-bearing reply run `08584077080029842355610407254CU30` succeeded in four seconds. The trigger also created short successful, no-effect runs for adjacent Workflows messages; these demonstrate the need for explicit filtering and durable deduplication before scale.
- Tenant capability: the environment can send Teams messages as Workflows and can execute the AI Builder prompt. This proves the narrow outbound identity and summarization path, not production licensing, tenant-wide inbound privacy, concurrency or ALM readiness.
- Native Workflows behavior: the user-authored test message also caused the Workflows app to emit its own generic status message before the flow-authored summary. PAB-02 must decide whether that extra message is acceptable for the pilot. A branded conversational experience may require Copilot Studio or a custom Teams app while retaining the same service-flow contracts.

# Product boundary

This plan owns the generalized Teams assistant, its Power Platform solution, conversation adapter, durable request/effect state, capability flows, response rendering and operations. It does not own the existing Sponsor Highlights canvas app, SharePoint list/exporter, GitLab intake/catalog flows, Vergil deployment or ACORN CLI behavior.

The [PowerApp and Power Automate intake portfolio](../powerapps-jsonl-intake/index.md) coordinates the operational Sponsor Highlights app, immutable SharePoint receipts, report-first review ledger, approval-gated collected JSONL and shared-environment migration. Its atomic child plans own the interfaces and release gates. The [Vergil GitLab bot plan](../vergil-gitlab-jsonl-automation/index.md) remains authoritative for unattended GitLab admission, cleanup, review, reporting and callbacks. A future bot capability may call those systems only through an approved, typed contract; this plan creates no implicit SharePoint-to-GitLab bridge and never drains SharePoint history.

The two test flows remain test-only evidence until PAB-06 replaces them with solution-aware components. Do not rename them into production, add broad write capabilities to them, or treat successful owner testing as multi-user acceptance.

# Architecture decisions

## Conversation model

Use a small conversation adapter around a durable command-processing core:

1. A user starts a session through an approved entry point.
2. Workflows posts the prompt and records a correlation token, initiating user, chat/conversation ID and expiry.
3. The reply listener normalizes only messages that satisfy the accepted interaction contract.
4. A claim step creates or reuses the durable request before any AI or external effect.
5. Authorization and routing choose one capability revision from a deployment-owned allowlist.
6. The capability flow records its job/effect state, performs bounded work and returns a typed result.
7. A single response renderer posts the Workflows message and records the delivery receipt.
8. Replay, restart or duplicate triggers reuse the durable request/effect keys and cannot repeat an accepted external effect.

Conversation state must never be inferred only from the latest chat message, a flow run ID or an in-memory variable. A message without a valid open session must receive a bounded help/expiry response or no effect according to the approved interaction matrix.

## Canonical request contract

Every ingress adapter emits one normalized request before routing:

| Field | Contract |
|---|---|
| `schemaVersion` | Integer contract version; reject unsupported versions. |
| `requestId` | Stable GUID generated or recovered before capability execution. |
| `sessionId` | Stable session GUID linking prompts, replies and responses. |
| `sourceMessageId` | Teams message ID used for replay detection. |
| `conversationId` | Exact Teams chat/channel conversation used for the reply. |
| `initiatorAadObjectId` | Entra object ID used for authorization; email is display/routing data, not the authority key. |
| `initiatorUpn` | Normalized user principal name retained for diagnostics and Workflows addressing. |
| `command` | Deployment-owned capability key such as `summarize`. |
| `arguments` | Typed JSON object validated by the selected capability. |
| `receivedAt` | UTC timestamp from the normalized ingress event. |
| `correlationId` | End-to-end trace key returned in receipts and operator views. |

## Canonical result contract

Capability flows return data; they do not format or send Teams messages directly:

| Field | Contract |
|---|---|
| `requestId` | Matches the normalized request. |
| `status` | `succeeded`, `needs_input`, `denied`, `failed_retryable` or `failed_terminal`. |
| `summary` | Short user-facing outcome; never raw connector output. |
| `body` | Optional structured sections for the response renderer. |
| `links` | Approved deep links with labels; SharePoint links use the accepted receipt/item identity. |
| `effectIds` | Durable external-effect keys created by the capability. |
| `diagnosticCode` | Stable operator code without secrets or raw sensitive payloads. |
| `completedAt` | UTC completion time. |

## Durable Dataverse state

| Table | Purpose and key |
|---|---|
| `Bot Session` | One conversation window per initiator and entry point. Alternate key: `sessionId`; stores conversation ID, prompt message ID, state and expiry. |
| `Bot Request` | Normalized immutable command. Alternate keys: `requestId` and `(sourceMessageId, commandRevision)`; stores authorization and terminal outcome. |
| `Bot Job` | One capability execution attempt. Alternate key: `(requestId, capabilityRevision)`; stores lease, retry count, heartbeat and error code. |
| `Bot Effect` | Idempotency ledger for writes, approvals and outbound calls. Alternate key: `(requestId, effectType, targetKey)`; stores intent, accepted proof and reconciliation state. |
| `Bot Receipt` | Outbound Workflows delivery state. Alternate key: `(requestId, channel, templateRevision)`; stores destination, message ID and uncertain-send hold. |
| `Bot Policy` | Deployment-owned capability, role, target and limit rules. Rules are versioned; user text never supplies connector names, URLs or arbitrary flow IDs. |

Payload retention must be capability-specific. Summarization content should be deleted or redacted after the approved interval while preserving correlation, outcome and non-sensitive diagnostics. Never put secrets, bearer tokens or complete sensitive message bodies in flow names, run-history annotations or error messages.

# Solution component inventory

The target solution contains twelve cloud flows. Names below are logical names; PAB-03 assigns solution publisher prefixes and environment-variable references.

| ID | Flow | Trigger and responsibility | Direct dependencies |
|---|---|---|---|
| F01 | Start Session | Manual/Teams-approved entry point; resolves the initiating user, creates `Bot Session`, and asks for input as Workflows. | Dataverse, Teams connection reference, F10 |
| F02 | Capture Teams Reply | Teams message trigger; retrieves details, rejects bot/self/noise events, correlates an open session and emits the canonical request. | Teams, Dataverse, F03 |
| F03 | Normalize and Claim Request | Child flow; validates schema/size/prefix, normalizes identity and atomically claims `Bot Request`. | Dataverse |
| F04 | Authorize and Route | Child flow; evaluates `Bot Policy`, assigns an allowlisted capability revision and rejects unauthorized/expired requests. | Dataverse, Entra/Office 365 Users if approved |
| F05 | Summarize | Child flow; validates text limits, invokes the approved AI Builder prompt and returns the canonical result. | AI Builder |
| F06 | Search and Status | Child flow; provides read-only status/search over explicitly approved ACORN or SharePoint surfaces. | Approved read-only connection references |
| F07 | Intake and Submission | Child flow; creates an approved intake request and durable `Bot Effect`; it never writes arbitrary paths or bypasses the intake plan. | Approved intake API/flow, Dataverse |
| F08 | Review and Approval | Child flow; creates/reconciles an approval and returns a typed outcome; no self-approval. | Approvals, Dataverse |
| F09 | Reports and Artifacts | Child flow; requests an approved report/artifact job and returns a receipt/status link. | Approved reporting endpoint, Dataverse |
| F10 | Render and Send Response | Child flow; selects a versioned response template, posts as Workflows to the recorded conversation and records `Bot Receipt`. | Teams, Dataverse |
| F11 | Retry and Dead Letter | Scheduled flow; reclaims expired leases, reconciles uncertain effects and moves terminal failures to an operator queue. | Dataverse, capability child flows |
| F12 | Monitor and Maintain | Scheduled/owner flow; reports stale sessions, queue age, error rates, connection expiry and policy/config drift without exposing payloads. | Dataverse, approved operator destination |

F02 may trigger on more messages than the bot consumes; it must terminate before capability execution when the sender, prefix/session, message type or replay key does not match. F10 is the only normal Teams sender. F05-F09 must not post directly, which keeps formatting, Workflows identity, deduplication and delivery reconciliation in one owner.

# Security, ownership and ALM

- Package all production components in one unmanaged development solution and promote a managed solution to the pilot/production environment. Test-only evidence flows stay outside the managed release or are removed after PAB-06 acceptance.
- Use connection references and environment variables for Teams, Dataverse, AI Builder, approved site/list IDs, API base URLs, operator destinations, limits and feature flags. Never embed credentials or protected URLs in expressions or documentation.
- Use a service-owned or approved process identity where licensing permits. Record at least two human co-owners, a connection rotation owner and an incident owner. Personal connection ownership is not a production continuity plan.
- Apply least privilege per capability. Read-only capabilities receive read-only connectors. Mutation capabilities use a separate connection reference and cannot be enabled by editing chat text.
- Enforce tenant DLP before pilot import. Validate that Teams, Dataverse, AI Builder, SharePoint and any custom connector are permitted in the same data group and that government-cloud endpoints are supported.
- Treat AI output as untrusted content. Bound input/output length, label generated summaries, prevent HTML/mention injection, and never let model output choose a connector, URL, table, path, recipient or approval result.
- Prefer child-flow contracts over direct cross-flow HTTP calls. If an external trigger is required, it needs authenticated ingress, schema validation, durable acceptance before 2xx and replay-safe delivery IDs.
- Export the solution and deployment settings for recovery. Acceptance requires import into an isolated target, connection rebinding and a no-secret source inspection.

# Capability release policy

Capabilities enter the router only after they have a typed request/result contract, named owner, explicit authorization rule, input/output limit, idempotency design, focused tests, operator diagnostics and rollback switch. Read-only capabilities precede mutations. A capability that cannot supply a stable target/effect key remains advisory and must not perform the write.

Initial release order:

1. `summarize` — no external write beyond the Workflows response.
2. `help` and `status` — read-only and bounded.
3. approved intake lookup/submission receipt — uses the intake plan's exact identities and links.
4. approvals and artifact requests — only after effect reconciliation and human-authority gates.

# Validation and acceptance

Production acceptance requires all of the following:

- One authorized start creates one open session, one Workflows prompt, one accepted reply, one capability job and one Workflows response in the recorded conversation.
- Exact replay of the start, Teams message, child-flow invocation or sender step produces no second accepted AI charge, write, approval or message receipt.
- The bot's prompt, generated response and native Workflows status message cannot invoke a capability.
- Two concurrent users and two concurrent sessions for one user cannot cross-route content, results, conversation IDs or authorization.
- Unauthorized, expired, oversized, malformed and unsupported commands create no external effect and return the approved bounded response or silent discard.
- A lost connector response, flow restart and expired lease reconcile against `Bot Effect`/`Bot Receipt`; ambiguous non-idempotent sends remain held for operator review.
- Solution import into an isolated target succeeds with environment-variable settings and connection rebinding; source export contains no secret values.
- Operator evidence includes flow and solution revisions, run IDs, Dataverse keys, Teams message IDs, policy revision, latency and redacted error codes.

Use focused Power Platform and Teams acceptance; no Rust workspace tests are required unless a later capability changes ACORN code. If implementation discovers an ACORN defect, create a separate bounded code phase under the appropriate plan rather than expanding an external deployment phase.

# Execution phases

Only remaining work appears below. Phases are sequential unless a prerequisite explicitly permits otherwise. Each phase is capped at five active hours and owns one primary integration lock.

## PAB-02 — Decide the supported Teams interaction matrix

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted November 5 test vertical slice; test users and isolated Teams conversations approved
- Exact write set: named external systems only — Power Automate environment `221f44a6-1468-ea4e-b93d-cd52f1723079`; Microsoft Teams Workflows chat; the two existing `[TEST] Summarize Text` flows; this plan's interaction-matrix and adapter-decision sections
- Primary integration lock: Teams conversation adapter
- Outcome: Decide the supported entry/reply surface and whether native Workflows UX is acceptable; select Power Automate-native, Copilot Studio or custom Teams ingress without changing the service-flow contracts.
- Focused validation: Test owner and second user, two concurrent sessions, 1:1/group/channel as applicable, late/expired reply, bot-authored prompt/summary, native Workflows status response, unrelated chats and tenant trigger scope. Record what the trigger can see and what identity/correlation fields it supplies.
- Handoff: Approved interaction matrix, privacy/licensing finding and adapter decision. Stop the plan if reliable per-user correlation or acceptable privacy cannot be demonstrated.

## PAB-03 — Create the solution and ALM skeleton

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAB-02 adapter decision; approved publisher prefix and development/isolated target environments
- Exact write set: new Power Platform solution; connection references; environment variables; placeholder child-flow interfaces; `.wiki/features/powerautomate-teams-bot/index.md`
- Primary integration lock: Power Platform solution package
- Outcome: Create one solution-aware component graph with named owners, settings, feature flags and deployment settings; import an empty/disabled managed build into the isolated target.
- Focused validation: Export/import, bind connections without editing flow definitions, inspect package for secrets, turn all production capabilities Off by default and verify rollback/uninstall behavior.
- Handoff: Solution unique name/version, publisher, environment IDs, connection-reference inventory and import evidence.

## PAB-04 — Implement durable state and idempotency

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAB-03; Dataverse capacity and table-creation authority
- Exact write set: `Bot Session`, `Bot Request`, `Bot Job`, `Bot Effect`, `Bot Receipt` and `Bot Policy` tables, alternate keys, least-privilege security roles and retention settings
- Primary integration lock: Dataverse schema
- Outcome: Provide atomic request claiming, job leases, effect/receipt reconciliation, policy revisions and bounded retention before any production capability runs.
- Focused validation: Duplicate creates converge on one key; concurrent claims yield one lease; expired lease recovery preserves accepted effects; unauthorized roles cannot read payloads or mutate policy; retention removes content without breaking audit keys.
- Handoff: Table/column/alternate-key manifest, role matrix and replay/concurrency evidence.

## PAB-05 — Build shared ingress, authorization and routing

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAB-02, PAB-04
- Exact write set: F01 Start Session, F02 Capture Teams Reply, F03 Normalize and Claim Request, F04 Authorize and Route; deployment-owned `Bot Policy` seed records
- Primary integration lock: Teams conversation adapter
- Outcome: Normalize approved Teams events into one durable request and dispatch only an allowlisted capability revision for an authorized Entra identity.
- Focused validation: Interaction matrix plus unsupported/noise events, exact message replay, expired sessions, malformed payloads, unauthorized user, policy revision change and two-user/two-session concurrency. No capability is allowed to send directly.
- Handoff: Versioned canonical request, router decision codes and accepted ingress evidence for PAB-06.

## PAB-06 — Replace the test flows with production summarization

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAB-05; approved AI Builder prompt, content limits, retention and licensing
- Exact write set: F05 Summarize; summarize policy record; prompt/environment-variable reference; disabled or removed `[TEST] Summarize Text` flows after replacement acceptance
- Primary integration lock: AI Builder capability
- Outcome: Deliver solution-aware summarization through the shared request/result contracts without retaining unnecessary input text or allowing prompt/response recursion.
- Focused validation: Empty/minimum/maximum/one-over-limit text, HTML/mention-like content, prompt injection, timeout, connector denial, exact replay and restart. Compare the typed result to F10 fixtures; record AI usage/latency without logging full sensitive content.
- Handoff: Prompt revision, limits, redacted acceptance runs and explicit disposition of both test flows.

## PAB-07 — Centralize Workflows responses and receipts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAB-04, PAB-05; accepted response templates and Workflows destinations
- Exact write set: F10 Render and Send Response; response templates; `Bot Receipt` delivery/reconciliation rules
- Primary integration lock: Teams outbound identity
- Outcome: Make F10 the only normal Teams sender and deliver versioned success, needs-input, denied, retryable and terminal templates as Workflows to the recorded conversation.
- Focused validation: 1:1/group/channel destinations from the approved matrix, links, Unicode, maximum body, HTML escaping, duplicate send request, simulated lost response and inaccessible conversation. Verify message IDs and uncertain-send holds.
- Handoff: Template revisions, destination rules and receipt evidence usable by every later capability.

## PAB-08 — Add bounded read-only capabilities

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAB-07; named data owners and approved read-only sources
- Exact write set: F06 Search and Status; read-only connection references; policy records and result templates
- Primary integration lock: capability registry
- Outcome: Add `help`, bot/session status and one approved read-only ACORN or SharePoint lookup without exposing broad tenant search or arbitrary query execution.
- Focused validation: Authorized/unauthorized identity, empty/many/no-result cases, source timeout, maximum result count, link allowlist, payload redaction and replay. Confirm the connector principal cannot write.
- Handoff: Capability contracts, source/field allowlist, owner and acceptance evidence.

## PAB-09 — Add intake, approval and artifact mutations

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAB-08; PAB-04 effect ledger; explicit approval from each owning plan/system; named human approval authority; PA-41A before any Sponsor Highlights approval/publication mutation
- Exact write set: F07 Intake and Submission, F08 Review and Approval, F09 Reports and Artifacts; separate mutation connection references; policy and response records
- Primary integration lock: external effect ledger
- Outcome: Add approved mutations only through stable target/effect keys and existing system contracts; return receipt/status links instead of claiming completion without proof. Sponsor Highlights commands may request an existing flow operation but cannot write review rows, set Approved state, or publish collected JSONL directly.
- Focused validation: Unauthorized target, changed payload under same request ID, duplicate/restart/lost-response, approval rejection/expiry, target conflict and inaccessible receipt. Verify no self-approval, arbitrary path/URL, direct review-state/export mutation, or SharePoint-to-GitLab bridge.
- Handoff: Per-capability authority, idempotency/reconciliation record and rollback switch. Any unmet target-system gate remains outside the router.

## PAB-10 — Add retry, dead-letter and operations

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAB-07; at least one accepted capability
- Exact write set: F11 Retry and Dead Letter, F12 Monitor and Maintain; operator views/alerts; runbook sections for recovery, replay and connection rotation
- Primary integration lock: bot operations
- Outcome: Recover expired jobs and retryable effects without duplicate outcomes; surface terminal/ambiguous work to named operators with payload-safe diagnostics.
- Focused validation: AI timeout, Teams timeout/lost response, expired job lease, disabled connection, policy mismatch, dead-letter replay, alert deduplication and connection-expiry warning. Confirm an ambiguous non-idempotent send is held, not blindly retried.
- Handoff: Alert destination, severity/code catalog, recovery procedure, queue-age thresholds and proof of operator access.

## PAB-11 — Accept security, ownership, licensing and DLP

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAB-03 through PAB-10 component inventory; tenant admin, data owner and service owner available
- Exact write set: named external systems and governance records only — Power Platform admin center; tenant DLP policies; Teams; Dataverse; AI Builder; approved SharePoint/custom connectors; solution owner identities; this plan's release checklist and owner/rotation matrix
- Primary integration lock: release governance
- Outcome: Approve the production identity, two-owner continuity, licensing/capacity, DLP grouping, data classification/retention, connector privileges and incident/rotation ownership.
- Focused validation: License a representative non-owner user, import/bind under the approved identity, rotate one non-production connection, review run-history exposure, inspect security roles and verify disabled-owner continuity. Record denials as release blockers.
- Handoff: Signed release checklist, owner/rotation matrix, licensing/capacity evidence and approved target environment.

## PAB-12 — Pilot, release and retire this plan

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAB-06, PAB-07, PAB-10, PAB-11; pilot audience and rollback window approved; any PAB-08/09 capability included in release has its own acceptance
- Exact write set: managed solution import and environment settings in the approved pilot target; pilot sharing/policy records; durable operator guide; this plan plus portfolio index/network/lane entries
- Primary integration lock: Power Platform deployment
- Outcome: Run a bounded multi-user pilot, accept the release/rollback path, remove obsolete test components and retire this plan after durable documentation owns operations.
- Focused validation: Authorized and unauthorized users, two concurrent users, two sessions for one user, exact replay, restart/lost response, latency/AI usage, Workflows delivery, operator alerts, managed upgrade and rollback. Inspect every enabled capability's effect/receipt keys.
- Handoff: Solution version, environment/connection references, enabled capability revisions, pilot results, rollback point and operator ownership. Delete this plan and all portfolio entries only after acceptance evidence is preserved outside the planning wiki.
