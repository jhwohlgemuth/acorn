---
type: Epic
title: Research Activity Index capability
description: Coordinate a bounded, accessible index of loaded and validated Research Activity Data.
difficulty: mixed
impact: high
package: [acorn-web, acorn-schema]
tags: [feature, research-activity, index, browser, accessibility]
status: in-progress
execution: coordination
---

# Outcome

Users can load multiple public or token-protected GitHub/GitLab buckets, automatically restore their combined bounded workspace until deleting app data, and explore validated Research Activity Data as a deterministic project/activity index. The Portfolio follows the inspected ORNL Research Index interaction model for search, counts, sorting, list/card layouts, navigation, and available detail sections while retaining semantic controls and ACORN ownership. The initial single-bucket contract is accepted by ABW-19; ABW-63–65 add and accept multiple buckets. The browser workspace plan owns credential and loaded-data lifecycles; this epic adds no remote index service.

# Related implementation plans

- [Browser workspace](../acorn-browser-workspace/index.md#abw-19--accept-the-initial-research-activity-index-capability) — ABW-19 accepts the initial token-backed index; [ABW-63](../acorn-browser-workspace/index.md#abw-63--compose-multiple-repository-buckets-in-one-bounded-workspace) composes source-aware state, ABW-64 adds controls, and ABW-65 accepts the combined browser index.

# Current coordination

Accepted ABW-39 supplies secure-context sessionStorage credentials with exact provider/host isolation for newly saved tokens, accepted ABW-41 supplies scoped GitHub loading, and accepted ABW-38 supplied the separate bounded localStorage snapshot. The current browser implementation saves after a load and restores automatically until **Delete all data**; ABW-19 must accept that changed lifecycle. ABW-42/43 derive deterministic project/activity rows from the validated/restored workspace and expose search, sorting, list/cards, detail sections, relation navigation, and source provenance. ABW-40 private GitLab loading was accepted on November 16 at the hosted HTTPS origin: the user loaded private NSSD and Chromium showed 38 files with a masked host-scoped token. ABW-60–62 accepted bounded local/repository raster images and their catalog presentation on November 17 with Rust, JavaScript, and Chromium checks. Current `Workspace::replace` and the Home remote handler replace the prior bucket on each successful load; combined sources and per-bucket removal are unimplemented. The older ABW-03/16 tool/folder cross-engine matrices do not gate the index contract. ABW-19 remains active for the new build's hosted private-provider recheck; ABW-63 → ABW-64 → ABW-65 then add and accept multiple buckets before ABW-44 → ABW-18 → ABW-37 project mapping and model assessment.
