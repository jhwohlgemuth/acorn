---
type: Plan
title: "Introduce a shared export context"
description: "Extract Research Activity fields once for HTML and PowerPoint renderers."
difficulty: medium
impact: high
package: [acorn-cli, acorn-lib]
tags: [refactor, export, html, powerpoint]
status: planned
priority: next
---

# Evidence

- `acorn-cli/src/template/mod.rs`
- `acorn-lib/src/io/powerpoint/mod.rs`

# Recommendation

1. Define a library-owned `ExportContext`.
2. Extract fields once and consume the context in every format-specific renderer.
3. Validate required placeholders before rendering.

# Acceptance criteria

HTML and PowerPoint exports use the same field mapping and fail before rendering when required values are unavailable.

# Related plans

- [Shared interpolation primitives](interpolation-primitives.md)
