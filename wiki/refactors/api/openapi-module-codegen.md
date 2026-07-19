---
type: Plan
title: "Generate API module skeletons from OpenAPI"
description: "Generate endpoint definitions and baseline DTOs as a starting point for provider modules."
difficulty: high
impact: not-assessed
package: [acorn-lib, workspace]
tags: [refactor, api, openapi, codegen]
status: planned
---

# Recommendation

Create an internal generation path for endpoint definitions and baseline DTOs after endpoint validation and response conventions stabilize.

# Acceptance criteria

Generated output is reproducible, reviewable, and leaves provider-specific behavior in handwritten modules.

# Related plans

- [OpenAPI endpoint validation](openapi-endpoint-validation.md)
- [Response DTOs](response-dtos.md)
