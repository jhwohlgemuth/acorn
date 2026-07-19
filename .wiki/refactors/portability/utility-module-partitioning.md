---
type: Plan
title: "Partition pure and std utility modules"
description: "Separate portable string and interpolation helpers from console and embedded-asset presentation."
difficulty: medium
impact: medium
package: [acorn-lib]
tags: [refactor, utilities, no-std]
status: planned
---

# Evidence

`acorn-lib/src/util/mod.rs` combines pure helpers with console tables, colors, and embedded assets.

# Recommendation

Move pure string, identifier, and interpolation helpers into a portable module; move console formatting and rendering into a std module; re-export both conditionally.

# Acceptance criteria

Using portable utility helpers does not enable console or embedded-asset dependencies.
