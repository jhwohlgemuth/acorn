---
type: Plan
title: "Normalize API environment bootstrap"
description: "Apply consistent dotenv loading and prioritized environment-variable discovery across providers."
difficulty: low
impact: medium
package: [acorn-lib]
tags: [refactor, api, environment, configuration]
status: planned
---

# Project code boundaries

- **Update:** environment/config bootstrap in `crates/acorn-lib/src/io/{config,api/mod}.rs`, CLI startup, and tests; later `acorn-host` composition.
- **Create:** an immutable resolved API configuration input consumed by `acorn-api` clients.
- **Delete:** provider-local environment reads after callers inject resolved configuration.

# Dependencies

Follow endpoint and authentication contracts; coordinate CLI bootstrap with [startup separation](../core/cli-startup-separation.md).

# Evidence

OpenAI, RAiD, and GitLab options repeat environment loading and fallback rules.

# Recommendation

Provide one bootstrap helper that distinguishes required from optional values and accepts an explicit precedence list.

# Acceptance criteria

Provider configuration uses documented, deterministic precedence and consistent missing-secret errors.
