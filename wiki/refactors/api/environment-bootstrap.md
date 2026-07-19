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

# Evidence

OpenAI, RAiD, and GitLab options repeat environment loading and fallback rules.

# Recommendation

Provide one bootstrap helper that distinguishes required from optional values and accepts an explicit precedence list.

# Acceptance criteria

Provider configuration uses documented, deterministic precedence and consistent missing-secret errors.
