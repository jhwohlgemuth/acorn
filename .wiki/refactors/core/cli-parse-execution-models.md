---
type: Plan
title: "Separate CLI parse and execution models"
description: "Keep clap-facing structures isolated from normalized command execution options."
difficulty: medium
impact: low
package: [acorn-cli]
tags: [refactor, cli, clap]
status: planned
---

# Evidence

- `crates/acorn-cli/src/cli/mod.rs`
- `crates/acorn-cli/src/main.rs`

# Recommendation

Keep parser-derived structures at the CLI boundary and perform conversion into execution options in one normalization layer.

# Acceptance criteria

Command implementations receive normalized execution options without depending on clap parsing details.
