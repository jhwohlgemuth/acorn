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

- [Browser workspace](../acorn-browser-workspace/index.md#abw-40--add-authenticated-gitlab-bucket-loading) — ABW-40 accepts private GitLab bucket loading with a verified minimum token scope.
- [Browser workspace](../acorn-browser-workspace/index.md#abw-19--accept-the-initial-research-activity-index-capability) — ABW-19 accepts the complete token-backed, restored, searchable index and maximum-session behavior.
- [Index projection and navigation](../research-activity-index-implementation.md) — ABW-42 and ABW-43 build the reusable index and expose catalog/detail navigation.

# Current coordination

Accepted ABW-39 supplies secure-context sessionStorage credentials with exact provider/host/project isolation, accepted ABW-41 supplies scoped GitHub loading, and accepted ABW-38 persists the full bounded workspace snapshot in separate localStorage. ABW-40 completes private GitLab loading after resolving the documented `read_repository` access boundary for its discovery calls. ABW-42 consumes the validated/restored workspace and freezes deterministic index identity; ABW-43 consumes that projection and owns the catalog/detail surfaces; ABW-19 then performs integrated acceptance. The live ORNL Research Index and public OKF Plan Browser source were inspected on November 16. The order is ABW-40 → ABW-42 → ABW-43 → ABW-19, followed by ABW-44 → ABW-18 → ABW-37 for project mapping and model assessment.
