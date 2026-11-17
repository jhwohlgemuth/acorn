---
type: Epic
title: Research Activity Index capability
description: Coordinate a bounded, accessible index of loaded and validated Research Activity Data.
difficulty: mixed
impact: high
package: [acorn-web, acorn-schema]
tags: [feature, research-activity, index, browser, accessibility]
status: planned
execution: coordination
---

# Outcome

Users can load a public or token-protected GitHub/GitLab bucket, explicitly restore its bounded workspace, and explore validated Research Activity Data as a deterministic project/activity index. The Portfolio follows the inspected ORNL Research Index interaction model for search, counts, sorting, list/card layouts, navigation, and available detail sections while retaining semantic controls and ACORN ownership. Completion requires ABW-40 private GitLab bucket loading, ABW-42 projection, ABW-43 catalog/navigation, and ABW-19 integrated acceptance. The browser workspace plan owns credential and loaded-data lifecycles; this epic adds no remote index service.

# Related implementation plans

- [Browser workspace](../acorn-browser-workspace/index.md#abw-19--accept-the-initial-research-activity-index-capability) — ABW-19 accepts the complete token-backed, restored, searchable index and maximum-session behavior.

# Current coordination

Accepted ABW-39 supplies secure-context sessionStorage credentials with exact provider/host isolation for newly saved tokens, accepted ABW-41 supplies scoped GitHub loading, and accepted ABW-38 persists the full bounded workspace snapshot in separate localStorage. ABW-42/43 now derive deterministic project/activity rows from the validated/restored workspace and expose search, sorting, list/cards, detail sections, relation navigation, and source provenance. Focused Rust, Chromium, restoration, and 500-file checks passed on November 16; the browser README and tests preserve the contract. ABW-40 private GitLab loading was accepted on November 16 at the hosted HTTPS origin: the user loaded private NSSD, and Chromium showed 38 files with the token masked and configured for `code.ornl.gov`. Successful browser fetches confirmed CORS; the app showed the `read_api` requirement before the user entered the token, while the raw token scope and commit ID are not exposed in the UI. ABW-19 still waits for ABW-03/16 and the browser plan's other acceptance prerequisites before integrated acceptance. ABW-44 → ABW-18 → ABW-37 then cover project mapping and model assessment.
