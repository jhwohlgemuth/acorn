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

# Project code boundaries

- **Update:** untyped provider response parsing in `crates/acorn-lib/src/io/api/` and fixtures/snapshots; later `acorn-api`.
- **Create:** minimal stable DTOs for consumed fields, with unknown-field tolerance chosen per provider contract.
- **Delete:** broad `serde_json::Value` traversal where a stable consumed subfield is known.

# Dependencies

DTO conventions feed error envelopes, invoke/parse wrappers, OpenAPI generation, and the provider capability matrix.

# Recommendation

Introduce optional DTOs only for stable fields such as identifiers, object kinds, and usage metadata, layered over baseline `serde_json::Value` handling.

# Acceptance criteria

Stable fields are typed without making evolving or provider-specific response fields mandatory.
