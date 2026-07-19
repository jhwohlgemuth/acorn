---
type: Plan
title: "Validate OpenAPI-backed endpoint templates"
description: "Detect missing endpoint names and required template keys before invocation."
difficulty: medium
impact: not-assessed
package: [acorn-lib]
tags: [refactor, api, openapi, validation]
status: planned
---

# Recommendation

Add utilities that validate endpoint template names and required rendering keys against the embedded API description.

# Acceptance criteria

Template drift produces a focused validation error before a network request is attempted.
