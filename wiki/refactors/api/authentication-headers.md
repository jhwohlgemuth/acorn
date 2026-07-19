---
type: Plan
title: "Standardize authentication header construction"
description: "Share bearer-token and custom-token header parameter builders across API providers."
difficulty: low
impact: high
package: [acorn-lib]
tags: [refactor, api, authentication]
status: planned
---

# Evidence

OpenAI and RAiD use bearer authorization while GitLab uses `PRIVATE-TOKEN`, with repeated parameter construction.

# Recommendation

Add reusable helpers for bearer and named custom-token headers, then replace provider-local assembly.

# Acceptance criteria

Authentication values are constructed consistently without changing provider wire formats.
