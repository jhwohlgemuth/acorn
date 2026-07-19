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

# Recommendation

Keep command and setup functions result-oriented, then convert their failures to rich reports and named exit codes at one outer boundary.

# Acceptance criteria

Commands do not independently terminate the process or duplicate exit-code mapping.

# Related plans

- [CLI startup separation](cli-startup-separation.md)
