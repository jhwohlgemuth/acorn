---
type: Epic
title: Live bot observability over SSE
description: Coordinate the GitLab bot event feed, access boundary, and desktop and browser status views.
difficulty: mixed
impact: medium
package: [acorn-lib, acorn-desktop, acorn-web]
tags: [feature, gitlab, bot, sse, dashboard]
status: planned
execution: coordination
---

# Outcome

An operator can watch current GitLab bot health and processing state in ACORN desktop, and later in the hosted browser workspace, without refreshing a page. The bot remains the source of state; GitLab ingress and durable work processing retain their existing contracts. A disconnected viewer shows stale status and obtains a fresh snapshot on reconnect. The epic is complete when the feed, its access boundary, and both client views have accepted evidence.

# Child plans

- [Live GitLab bot status feed](gitlab-bot-live-status.md) — GBO-01 adds bounded, read-only SSE state; GBO-02 authenticates observation in a later phase and defines browser access.
- [Browser and desktop workspace](acorn-browser-workspace/index.md) — ABW-70 adds the desktop bridge and dashboard after GBO-02; ABW-71 reuses that view for authorized, read-only hosted-web access.

# Current coordination

November 24 source and plan review: the bot has `GET /state`, polling, webhook admission, a worker, and durable operations, but no SSE route. `acorn-desktop` is not yet present; `acorn-web` has a shared frontend and planned native host boundary. The linked executable phases are planned, not implemented. GBO-01 precedes GBO-02; authentication is deliberately the later bot phase, and both remote consumers wait for it. The Vergil deployment and callback acceptance plan remains independent of this feature.

SSE carries observation from bot to viewer. It does not replace GitLab webhook POST or polling, and it does not authorize bot controls. ABW-53 separately owns local `acornd` management. Any future GitLab bot control action needs its own typed operation, authority, and acceptance plan before it joins this epic.
