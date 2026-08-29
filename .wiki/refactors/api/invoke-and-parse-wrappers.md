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

# Project code boundaries

- **Update:** repeated request/parse paths in `crates/acorn-lib/src/io/api/` and provider tests; later `acorn-api` plus transport ports.
- **Create:** one composable invocation wrapper over an injected transport and provider parser.
- **Delete:** duplicated send/status/body/deserialize sequences at migrated providers.

# Dependencies

Requires endpoint/auth/request contracts and feeds health/error normalization; concrete HTTP remains in `acorn-host`.

# Evidence

Provider functions repeat parameter construction followed by `invoke` or `invoke_with` and `handle` or `handle_or` parsing.

# Recommendation

Add narrowly scoped wrappers for the repeated invocation-and-parse shapes, with fallback behavior visible in the function signature.

# Acceptance criteria

Provider call sites retain their parsing choice without repeating transport boilerplate.
