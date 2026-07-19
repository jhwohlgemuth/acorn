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

# Evidence

ORCID, ROR, and CiteAs expose similar status-to-health wrappers.

# Recommendation

Add a root helper that maps a response status through a caller-provided health predicate.

# Acceptance criteria

Health checks share transport handling while provider-specific success rules remain explicit.
