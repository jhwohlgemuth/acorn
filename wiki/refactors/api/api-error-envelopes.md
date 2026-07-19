---
type: Plan
title: "Introduce shared API error envelopes"
description: "Normalize user-facing messages from provider-specific error payloads."
difficulty: medium
impact: not-assessed
package: [acorn-lib]
tags: [refactor, api, errors]
status: planned
---

# Evidence

GitLab, RAiD, and OpenAI return different error response shapes.

# Recommendation

Define a small trait for extracting a user-facing error message and implement it for stable provider error envelopes.

# Acceptance criteria

Provider details remain available while callers use one error-message extraction boundary.
