---
type: Plan
title: "Define an API provider capability matrix"
description: "Describe pagination, streaming, authentication, and related provider capabilities as data."
difficulty: high
impact: not-assessed
package: [acorn-lib]
tags: [refactor, api, capabilities]
status: planned
---

# Recommendation

Define a capability descriptor for behavior that command or library callers must select dynamically, such as pagination, streaming, and token requirements.

# Acceptance criteria

Callers consult capability metadata instead of hard-coding provider-name branches for represented behaviors.
