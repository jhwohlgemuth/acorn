---
type: Plan
title: "Extract invoke-and-parse wrappers"
description: "Combine repeated request invocation, response handling, parsing, and explicit fallback behavior."
difficulty: not-assessed
impact: high
package: [acorn-lib]
tags: [refactor, api, requests, parsing]
status: planned
---

# Evidence

Provider functions repeat parameter construction followed by `invoke` or `invoke_with` and `handle` or `handle_or` parsing.

# Recommendation

Add narrowly scoped wrappers for the repeated invocation-and-parse shapes, with fallback behavior visible in the function signature.

# Acceptance criteria

Provider call sites retain their parsing choice without repeating transport boilerplate.
