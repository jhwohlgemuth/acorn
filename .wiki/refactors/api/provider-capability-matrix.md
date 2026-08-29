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

# Project code boundaries

- **Update:** API documentation and provider metadata adjacent to `crates/acorn-lib/src/io/api/mod.rs`; later `acorn-api` docs/tests.
- **Create:** a data-driven capability matrix only when consumers need runtime inspection; otherwise documentation plus compile-time tests.
- **Delete:** duplicated prose claims superseded by the authoritative matrix.

# Dependencies

Populate from implemented provider contracts after the [API roadmap](roadmap.md) stabilizes; feature plans consume it but do not define unsupported capabilities.

# Recommendation

Define a capability descriptor for behavior that command or library callers must select dynamically, such as pagination, streaming, and token requirements.

# Acceptance criteria

Callers consult capability metadata instead of hard-coding provider-name branches for represented behaviors.
