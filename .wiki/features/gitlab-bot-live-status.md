---
type: Plan
title: Live GitLab bot status feed
description: Publish bounded bot status changes over SSE for desktop and browser dashboards.
difficulty: medium
impact: medium
package: [acorn-lib]
tags: [feature, gitlab, bot, sse, observability]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Expose a live, read-only status feed from the GitLab bot. Keep GitLab ingress as webhook POST and event polling, and keep the durable operation queue authoritative. The feed informs connected viewers; it does not deliver work or provide a management command channel.

This executable plan is coordinated by the [SSE bot observability epic](sse-bot-observability.md).

# Current evidence

November 24 source review: `crates/acorn-lib/src/io/api/gitlab/bot.rs` exposes `GET /state` with `after`, poll/processed counts, last error, and `WebhookRuntimeState`; it has no SSE route. `poll_once`, `record_success`, `record_error`, and `work_forever` update state while webhook operations and callbacks use a durable queue. The router also accepts `POST /webhooks/gitlab`. `GET /state` has no authentication check. No SSE behavior or live integration test was run in this planning pass.

# Boundaries and dependencies

- The [Vergil automation plan](vergil-gitlab-jsonl-automation/index.md) owns one deployment's authority, callbacks, review, and acceptance. This plan owns reusable bot observation and does not gate that pilot.
- The [browser and desktop workspace](acorn-browser-workspace/index.md) owns presentation and the native bridge. Its ABW-70/71 phases consume this feed after authentication is accepted; neither implements bot transport in the UI.
- Initial SSE carries only bounded, secret-free fields already available in `/state`. It is a live hint with an initial snapshot and heartbeat, not a replayable audit log. After reconnect, clients fetch a fresh snapshot. Durable per-operation history would require a separate contract and storage decision.
- No remote management action is implied by subscribing. Explicit bot control needs a separately reviewed operation and authority contract; ABW-53 remains local `acornd` management.

# Acceptance and validation

An SSE client receives one initial state and subsequent changes after poll success/failure and worker settlement; disconnects do not block or alter processing. Slow clients have bounded buffering, clean cancellation, and a reconnect path that cannot claim missed events were replayed. Authentication is a later phase and must be accepted before the desktop or hosted browser consumes the feed across a host boundary.

# Execution phases

## GBO-01 — Publish bounded live bot state

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-lib/src/io/api/gitlab/bot.rs`, `crates/acorn-lib/src/io/api/gitlab/tests/bot.rs`, `.wiki/features/gitlab-bot-live-status.md`
- Primary integration lock: GitLab bot HTTP and runtime state
- Outcome: Add `GET /events` as an SSE view of the existing secret-free state, with an initial snapshot, updates, heartbeat, bounded lag handling, and disconnect cleanup. Preserve `GET /state` and the durable queue. Document that this first phase has the same access boundary as `/state` and is not approved for remote dashboard use.
- Focused validation: Bot router tests for initial state, poll/worker change, reconnect snapshot, slow client, and cancellation; focused `acorn-lib` GitLab bot tests and format/clippy checks for touched code.
- Handoff: Record the wire event shape and accepted local behavior, remove GBO-01, and hand endpoint access to GBO-02.

## GBO-02 — Authenticate bot observation

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: GBO-01
- Exact write set: `crates/acorn-lib/src/io/api/gitlab/bot.rs`, `crates/acorn-lib/src/io/api/gitlab/tests/bot.rs`, `crates/acorn-cli/src/commands/intake/mod.rs`, `.wiki/features/gitlab-bot-live-status.md`
- Primary integration lock: GitLab bot observation authority
- Outcome: Require an explicitly configured, scoped credential on `/events` and `/state`; define a browser-compatible connection path without putting a secret in a URL. Reject unauthorized subscriptions before streaming and keep health/readiness behavior explicit.
- Focused validation: Authorized/unauthorized state and SSE requests, reconnect after credential change, redaction, wrong-host and cancellation fixtures; focused bot and CLI tests.
- Handoff: Record the accepted access and origin contract for ABW-70/71, remove GBO-02, and retire this plan when documentation and dependent indexes are reconciled.
