---
type: Plan
title: "Extract API health-check helpers"
description: "Share response-status interpretation while allowing provider-specific health predicates."
difficulty: not-assessed
impact: medium
package: [acorn-lib]
tags: [refactor, api, health]
status: planned
---

# Project code boundaries

- **Update:** provider health/status functions across `crates/acorn-lib/src/io/api/` and tests; later `acorn-api`.
- **Create:** shared status classification over transport-neutral response metadata.
- **Delete:** duplicated status-code interpretation after provider migration.

# Dependencies

Build after invoke/parse and error-envelope contracts establish what response metadata is available.

# Evidence

ORCID, ROR, and CiteAs expose similar status-to-health wrappers.

# Recommendation

Add a root helper that maps a response status through a caller-provided health predicate.

# Acceptance criteria

Health checks share transport handling while provider-specific success rules remain explicit.
