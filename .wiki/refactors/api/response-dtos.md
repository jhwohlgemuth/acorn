---
type: Plan
title: "Add lightweight API response DTOs"
description: "Type stable provider response fields while retaining raw JSON support for evolving payloads."
difficulty: medium
impact: not-assessed
package: [acorn-lib]
tags: [refactor, api, dto]
status: planned
---

# Recommendation

Introduce optional DTOs only for stable fields such as identifiers, object kinds, and usage metadata, layered over baseline `serde_json::Value` handling.

# Acceptance criteria

Stable fields are typed without making evolving or provider-specific response fields mandatory.
