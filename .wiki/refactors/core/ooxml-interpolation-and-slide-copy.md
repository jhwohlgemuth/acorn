---
type: Plan
title: "Harden OOXML interpolation and slide-copy mutations"
description: "Replace remaining fragile PowerPoint XML selection and mutation with structured transforms."
difficulty: medium
impact: critical
package: [acorn-cli, acorn-lib]
tags: [refactor, export, powerpoint, ooxml]
status: in-progress
priority: next
---

# Context

Slide-copy failures now return typed errors and relationship updates are structured, but paragraph selection and bullet interpolation still contain fragile behavior.

# Evidence

- `crates/acorn-cli/src/commands/export/powerpoint.rs`
- `crates/acorn-lib/src/io/powerpoint/mod.rs`

# Recommendation

1. Replace remaining regex-based paragraph selection and bullet interpolation with parser-backed XML transforms.
2. Complete structured relationship and XML updates for slide-copy mutations.
3. Centralize the remaining OOXML helpers.

# Validation

Exercise interpolation and slide copying with focused tests, then inspect a generated PPTX for valid root-relative ZIP entries and relationship targets.

# Acceptance criteria

Malformed or unsupported OOXML produces typed errors, and valid generated presentations open without repair.
