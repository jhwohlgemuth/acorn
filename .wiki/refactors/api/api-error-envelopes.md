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

# Project code boundaries

- **Update:** error parsing in `crates/acorn-lib/src/io/api/{mod,openai,openapi,raid,orcid,osti,ror,huggingface,harbor}.rs` and provider tests; later `acorn-api`.
- **Create:** lightweight provider error-envelope DTOs and redaction-aware conversion helpers.
- **Delete:** duplicated untyped error-body parsing at migrated providers.

# Dependencies

Follow stable [response DTO](response-dtos.md) and [invoke/parse](invoke-and-parse-wrappers.md) conventions; use owned errors from the crate boundary.

# Evidence

GitLab, RAiD, and OpenAI return different error response shapes.

# Recommendation

Define a small trait for extracting a user-facing error message and implement it for stable provider error envelopes.

# Acceptance criteria

Provider details remain available while callers use one error-message extraction boundary.
