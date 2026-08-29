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

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/api/{openapi,mod}.rs`, endpoint fixtures/tests, and later `acorn-api`.
- **Create:** validation comparing configured endpoint templates with imported OpenAPI operations.
- **Delete:** stale endpoint assumptions only when diagnostics and migration coverage exist.

# Dependencies

Follow [endpoint resolution](endpoint-resolution.md); precedes optional [module code generation](openapi-module-codegen.md).

# Recommendation

Add utilities that validate endpoint template names and required rendering keys against the embedded API description.

# Acceptance criteria

Template drift produces a focused validation error before a network request is attempted.
