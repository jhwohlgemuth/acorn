---
type: Plan
title: "Partition pure and std utility modules"
description: "Separate portable string and interpolation helpers from console and embedded-asset presentation."
difficulty: medium
impact: medium
package: [acorn-lib]
tags: [refactor, utilities, no-std]
status: in-progress
priority: next
---

# Evidence

Terminal output now lives in `util::terminal`, and minimal/alloc builds no longer resolve console, color, diff, or generic embedded-asset dependencies. `util::portable` still re-exports implementations owned by the mixed root module, so ownership is not yet extractable.

# Recommendation

Move pure string, identifier, and interpolation implementations into the portable module and re-export them from legacy root paths. Keep terminal formatting std-only. Move immutable validation vocabularies to the schema boundary rather than treating them as generic utility assets.

# Acceptance criteria

Using portable utility helpers does not enable console or embedded-asset dependencies.
