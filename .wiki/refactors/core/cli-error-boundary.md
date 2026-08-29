---
type: Plan
title: "Create a single CLI error boundary"
description: "Map internal command failures to user-facing reports and exit codes in one place."
difficulty: not-assessed
impact: high
package: [acorn-cli]
tags: [refactor, cli, errors]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-cli/src/main.rs`, command dispatch/result aliases, `src/cli/`, output/logging helpers, and binary tests.
- **Create:** one application error-to-exit-code/report mapping boundary in `acorn-cli`.
- **Delete:** per-command process exits and duplicate top-level report installation after migration.

# Dependencies

Follow [CLI startup separation](cli-startup-separation.md) and owned library errors; remain in the CLI composition root after crate extraction.

# Recommendation

Keep command and setup functions result-oriented, then convert their failures to rich reports and named exit codes at one outer boundary.

# Acceptance criteria

Commands do not independently terminate the process or duplicate exit-code mapping.

# Related plans

- [CLI startup separation](cli-startup-separation.md)
