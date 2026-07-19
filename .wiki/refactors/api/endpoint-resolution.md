---
type: Plan
title: "Extract endpoint resolution helpers"
description: "Resolve named templates and provider domains through one shared API helper."
difficulty: not-assessed
impact: high
package: [acorn-lib]
tags: [refactor, api, endpoints]
status: planned
---

# Evidence

Provider modules repeat template lookup followed by domain substitution.

# Recommendation

Add root helpers for named endpoint lookup and template-plus-domain resolution, preserving provider-specific template names.

# Acceptance criteria

OpenAI, GitLab, RAiD, and other applicable providers no longer duplicate endpoint construction pipelines.
