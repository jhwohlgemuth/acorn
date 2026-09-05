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

# Project code boundaries

- **Update:** path normalization/containment in `crates/acorn-lib/src/io/{archive,bagit,download,source,powerpoint}/`, CLI export/import/create consumers, and security tests; later `acorn-host` or `acorn-core` only for lexical components.
- **Create:** a shared safe relative-path value plus native materialization checks at the host boundary.
- **Delete:** duplicated lexical containment and unsafe join logic after migrated call sites preserve platform behavior.

# Dependencies

Separate pure lexical validation from filesystem canonicalization under the [portability roadmap](../portability/roadmap.md); coordinate with archive and export plans.

# Context

Path normalization and containment are cross-cutting correctness boundaries for CLI resolution and exported archive entries.

# Recommendation

Define a library-owned `SafePath` abstraction and use it for CLI path resolution and export file operations.

# Acceptance criteria

Callers share one normalization and containment policy, and absolute host paths cannot leak into generated archives.
