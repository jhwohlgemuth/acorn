---
type: Plan
title: "Introduce a shared safe-path abstraction"
description: "Centralize path normalization, containment, and safe export file operations."
difficulty: not-assessed
impact: high
package: [acorn-cli, acorn-lib]
tags: [refactor, paths, security, export]
status: planned
---

# Context

Path normalization and containment are cross-cutting correctness boundaries for CLI resolution and exported archive entries.

# Recommendation

Define a library-owned `SafePath` abstraction and use it for CLI path resolution and export file operations.

# Acceptance criteria

Callers share one normalization and containment policy, and absolute host paths cannot leak into generated archives.
