---
type: Plan
title: "Normalize API request parameter composition"
description: "Share required-input validation and optional query or body parameter assembly."
difficulty: low
impact: high
package: [acorn-lib]
tags: [refactor, api, validation, requests]
status: in-progress
---

# Project code boundaries

- **Update:** query/body parameter construction across `crates/acorn-lib/src/io/api/` and snapshots/tests; later `acorn-api`.
- **Create:** pure required/optional parameter combinators or typed request builders.
- **Delete:** repeated mutable vectors/maps and empty-value filtering at migrated providers.

# Dependencies

Follow endpoint resolution and precede invoke/parse wrappers and operation-specific request migrations.

# Evidence

`Params` now provides immutable `with_auth`, `with_api_key`, `with_template`, `with_keyvalue`, and body builders, and shared tests cover empty-value omission. `require_non_empty_secret` is shared by RAiD and GitLab. CiteAs, Handle, Hugging Face, and some provider-specific request paths still construct raw `Vec<Param>` values, so migration and required identifier/body validation remain incomplete.

# Recommendation

1. Provide helpers for required non-empty identifiers, bodies, and secrets.
2. Provide composable builders for common token, template, JSON body, and optional query shapes.
3. Keep provider-specific parameter names at the call site.

# Acceptance criteria

Required input failures are consistent and optional parameters are omitted when absent or empty.
