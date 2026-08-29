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

# Project code boundaries

- **Update:** OpenAPI importer, `xtask` or dedicated generation tooling, generated-module tests, and documentation.
- **Create:** generated provider skeleton/DTO output under an explicit generated source boundary in `acorn-api`.
- **Delete:** hand-written boilerplate only after generated output is reviewed, deterministic, and API-compatible.

# Dependencies

Later work: requires [OpenAPI endpoint validation](openapi-endpoint-validation.md), [response DTO](response-dtos.md) conventions, and a stable extracted `acorn-api` layout.

# Recommendation

Create an internal generation path for endpoint definitions and baseline DTOs after endpoint validation and response conventions stabilize.

# Acceptance criteria

Generated output is reproducible, reviewable, and leaves provider-specific behavior in handwritten modules.

# Related plans

- [OpenAPI endpoint validation](openapi-endpoint-validation.md)
- [Response DTOs](response-dtos.md)
