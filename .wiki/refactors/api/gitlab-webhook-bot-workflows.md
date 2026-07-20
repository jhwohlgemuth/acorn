---
type: Plan
title: "Add webhook-driven GitLab bot workflows"
description: "Receive authenticated merge request, note, and work item events and dispatch idempotent bot workflows."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [feature, api, gitlab, bot, webhook]
status: planned
---

# Goal

Make authenticated GitLab webhooks the bot's primary event source for merge request, comment, and work item workflows, while retaining polling as an optional reconciliation path.

# Current evidence

- `acorn-lib/src/io/api/gitlab/bot.rs` runs an Axum server and a background project-events poller.
- The router exposes only `GET /health` and `GET /state`.
- Polling requests use `target_type=note`, and `MergeRequestNoteEvent` rejects notes attached to issues or other work items.
- The default note handler only logs, and `poll_forever` retains a `TODO` for workflow processing.
- `acorn-lib/src/io/api/gitlab/mod.rs` can create merge request notes but has no work item GraphQL operations or webhook registration operation.
- `acorn-cli/src/commands/create/bot/mod.rs` starts a detached container without publishing its HTTP port, so an external GitLab instance cannot reach the bot without separate container networking or a reverse proxy.
- Bot state and the polling cursor are held only in memory, so they do not provide durable webhook deduplication across restarts.

# Decisions

- Add `POST /webhooks/gitlab` to the existing Axum server instead of creating another service.
- Keep authentication, payload decoding, normalization, command filtering, and workflow execution as separate boundaries.
- Acknowledge valid webhook requests after bounded enqueueing; do not wait for GitLab API mutations or agent work before responding.
- Normalize supported payloads into a closed bot-event enum before workflow dispatch.
- Use the webhook event UUID or Standard Webhooks message ID to deduplicate deliveries, plus a canonical workflow-operation key to deduplicate the same action across webhook and polling sources.
- Keep the inbound webhook credential separate from the GitLab API token used for outbound requests.
- Support both `object_kind: "issue"` and `object_kind: "work_item"`; select webhook registration fields according to the target GitLab version.
- Use GitLab's WorkItem GraphQL API for new work item reads and mutations.
- Keep polling available as an explicit reconciliation mode until webhook delivery and recovery are verified in production.

# Implementation steps

Steps are ordered by impact, then by estimated difficulty within the same impact level. Each step must meet its verification gate before the next dependent step begins.

| Order | Impact | Difficulty | Step | Verification gate |
|---:|---|---|---|---|
| 1 | high | low | Make the detached service reachable: bind the containerized server to `0.0.0.0`, publish an explicitly configured host/container port, and document the reverse-proxy alternative. Do not silently expose a port when the user has not selected webhook mode. | CLI parsing and container-argument tests prove the resolved bind and published port; `/health` is reachable through the published port in a container smoke test. |
| 2 | high | medium | Add `POST /webhooks/gitlab` and webhook configuration. Capture the exact raw body and required headers, enforce a body-size limit, verify the HMAC signing token when supported, and provide a constant-time `X-Gitlab-Token` compatibility path for older GitLab versions. Reject missing, stale, or invalid authentication before deserialization. | Router tests cover valid authentication, invalid authentication, stale timestamps, malformed JSON, unsupported event types, and oversized bodies without contacting GitLab. |
| 3 | high | medium | Introduce immutable payload DTOs and a normalized `BotEvent` enum for merge request, comment, issue, and non-issue work item events. Route by `X-Gitlab-Event` plus `object_kind`, preserve project ID, item IID, author identity, action, and event ID, and ignore bot-authored or unconfigured-project events. | Checked-in GitLab payload fixtures deserialize into the expected variants; fixtures for unknown fields, bot-authored comments, and unsupported work item types are ignored deterministically. |
| 4 | high | high | Add a bounded asynchronous dispatch queue and durable idempotency records through the existing database abstraction. Track both the inbound delivery ID and a canonical workflow-operation key. Return `202 Accepted` only after enqueueing; return a retryable failure when capacity is unavailable. Record event state so duplicate delivery and process restart cannot repeat an outbound mutation. Keep parsing and command selection pure and keep GitLab calls in workers. | Tests deliver the same event repeatedly and across a simulated database-backed restart and observe one workflow execution; saturation and worker-failure tests produce observable state without losing the event silently. |
| 5 | high | high | Implement the first vertical workflow: an explicit `/acorn status` comment on a merge request produces exactly one bot note. Ignore system notes, non-command comments, and the bot's own responses. Use project ID and MR IID from the normalized event rather than ambient CI variables. | An end-to-end test posts a signed comment fixture, runs the worker against a mock GitLab API, and observes one note for retries and no recursive response to the bot-authored note. |
| 6 | high | high | Add work item workflow support. Implement the minimum GraphQL queries and mutations required by selected commands, preserve work item global IDs and IIDs as distinct types, and support project work items before optional group-level epic handling. | GraphQL request/response fixtures cover Issue and Task payloads, API errors, and authorization failures; a webhook-to-mutation integration test completes one work item command. |
| 7 | medium | low | Convert polling into an explicit reconciliation mode. Reuse the same normalization and dispatch boundary where the Events API contains sufficient data, derive the same canonical workflow-operation key used by webhook ingestion, and expose source/queue/deduplication counters from `/state`. | Webhook-only, poll-only, and hybrid tests show that hybrid delivery does not execute the same supported event twice. |
| 8 | medium | medium | Add idempotent webhook registration to the GitLab client and `acorn create bot`. Discover an existing matching hook before creating one, enable merge request, note, and appropriate work item event fields, and report unsupported fields for the detected GitLab version. Never print either credential. | Mock API tests cover create, already-present, version-compatible field selection, permission errors, and rerunning the command without creating a duplicate hook. |
| 9 | medium | medium | Complete operational hardening and documentation: graceful queue drain, structured delivery IDs in logs, readiness distinct from liveness, secret injection guidance, webhook history/resend troubleshooting, generated CLI help, and deployment examples. | `make help`, focused tests, `cargo check -p acorn-lib`, applicable CLI checks, `cargo fmt --check`, and `git diff --check` pass; a staging webhook test and resend are visible in GitLab's delivery history. |

# Planned files

| File | Change |
|---|---|
| `acorn-lib/src/io/api/gitlab/bot.rs` | Add authenticated webhook ingress, normalization, dispatch, deduplication state, and reconciliation integration. |
| `acorn-lib/src/io/api/gitlab/mod.rs` | Add webhook registration DTOs/API calls and the minimum WorkItem GraphQL client surface. |
| `acorn-lib/src/io/api/gitlab/tests/mod.rs` | Add unit, router, fixture, duplicate-delivery, and mock API coverage. |
| `acorn-lib/src/io/database/schema.rs` | Add durable bot delivery and workflow-operation records using the existing SQLite/DuckDB backend boundary. |
| `acorn-lib/Cargo.toml` and `Cargo.toml` | Add only the cryptographic dependencies required for signature verification. |
| `acorn-cli/src/cli/arguments.rs` | Add explicit webhook URL, secret/signing-token source, port publication, registration, and event-source options. |
| `acorn-cli/src/commands/serve/bot/mod.rs` | Build webhook-aware server configuration without logging credentials. |
| `acorn-cli/src/commands/create/bot/mod.rs` | Publish the receiver port, pass secrets safely, and optionally register the project webhook. |
| `acorn-cli/src/commands/mod.rs` | Forward the new bot options through create and serve dispatch. |
| `CommandLineHelp.md` | Regenerate command help after CLI changes. |

# Validation

1. Run focused bot unit and router tests after each step.
2. Run the relevant `acorn-lib` and `acorn-cli` checks after changes to shared DTOs or command wiring.
3. Start the bot in a container and verify health plus authenticated webhook receipt through the published endpoint.
4. Register a staging project webhook and test merge request, note, Issue, and Task deliveries.
5. Resend the same delivery from GitLab and verify that no second mutation occurs.
6. Restart the bot, resend the delivery again, and verify that deduplication remains effective.
7. Confirm an `/acorn status` bot response does not trigger a response loop.
8. Exercise webhook-only and hybrid reconciliation modes and compare state counters with GitLab delivery history.

# Acceptance criteria

- A valid authenticated merge request or work item webhook is acknowledged promptly and dispatched through a normalized event boundary.
- Invalid authentication, stale signed requests, oversized bodies, unconfigured projects, and unsupported events cannot start workflow work.
- An `/acorn status` comment produces exactly one response despite delivery retries or a bot restart.
- Bot-authored and system-generated notes do not create feedback loops.
- Project Issue and Task workflows use the WorkItem GraphQL API and report GitLab errors without losing the originating event.
- Detached bot creation produces a deliberately reachable HTTPS deployment path and can register the required webhook without duplicating it on rerun.
- Polling is optional, shares the workflow boundary where possible, and does not duplicate webhook-driven work.
- Health, readiness, queue state, last error, and deduplication counts are observable without exposing credentials or payload secrets.

# Citations

[1] [GitLab webhook events](https://docs.gitlab.com/user/project/integrations/webhook_events/) - Merge request, comment, and work item triggers, payload types, and group-hook requirements for epics.

[2] [GitLab project webhooks API](https://docs.gitlab.com/api/project_webhooks/) - Project-hook creation, event flags, permissions, testing, history, and resend operations.

[3] [GitLab webhook signing tokens](https://docs.gitlab.com/user/project/integrations/webhooks/#signing-tokens) - HMAC-SHA256 verification, constant-time comparison, timestamp validation, and secret-token compatibility.

[4] [GitLab WorkItem API migration guide](https://docs.gitlab.com/api/graphql/epic_work_items_api_migration_guide/) - Direction to use the GraphQL WorkItem API for new work item features.

[5] [GitLab Events API](https://docs.gitlab.com/api/events/) - Polling limitations, including unsupported `DiscussionNote` merge request notes.
