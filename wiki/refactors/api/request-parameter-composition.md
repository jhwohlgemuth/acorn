---
type: Plan
title: "Normalize API request parameter composition"
description: "Share required-input validation and optional query or body parameter assembly."
difficulty: low
impact: high
package: [acorn-lib]
tags: [refactor, api, validation, requests]
status: planned
---

# Evidence

Provider modules repeat non-empty identifier and body validation, token-plus-template/body parameter sets, and optional query insertion.

# Recommendation

1. Provide helpers for required non-empty identifiers, bodies, and secrets.
2. Provide composable builders for common token, template, JSON body, and optional query shapes.
3. Keep provider-specific parameter names at the call site.

# Acceptance criteria

Required input failures are consistent and optional parameters are omitted when absent or empty.
