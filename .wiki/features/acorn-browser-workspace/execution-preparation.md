# Browser parity execution handoff — November 12, 2026

The user authorized review items 1–5: chrome/design reconciliation, tool presentations, browser effects/folders/Portfolio/local Geospatial, Settings, and browser Skill/Install. Those implementations are present in the current worktree. Desktop packaging and publication were not part of this batch. The November 7 portfolio currency review remains current.

## Verified handoff

[The browser README](../../../crates/acorn-web/README.md) owns durable behavior, commands, asset budgets, provenance and validation evidence. `scripts/parity-smoke.mjs` provides isolated Chromium route, file/folder, download, theme, responsive, map/fallback and no-network checks. `scripts/dev-smoke.mjs` checks map TSX/CSS refresh while preserving route/selection and restores its temporary edits. The SVG About credit and collapsed/responsive Home banners now match implemented artwork.

ABW-10 and ABW-60–62 are accepted and removed. The [canonical phases](index.md#execution-phases) retain only remaining work: ABW-03/04/05/32 operation boundary/announcement matrices; ABW-06/16 cross-engine effects/folder acceptance; ABW-19 hosted initial catalog acceptance; ABW-63/64/65 source-aware multiple-bucket state, controls and integrated acceptance; ABW-44/18 project-map and resource/performance acceptance; ABW-22 native export; ABW-31 published metadata. ABW-19 is independent of the broader ABW-03/16 matrices. ABW-08 owns the final browser/accessibility matrix after multi-bucket acceptance. Browser Skill/Install acceptance no longer depends on building native installers. Local Dioxus hot-reload acceptance passed on November 19; ABW-36 separately waits for mirrored Pages/HTTPS evidence. Map Fast Refresh and coordinated teardown are verified.

## Ownership and next execution

One integration owner serializes actual `src/ui.rs`, `operations.rs`, `results.rs`, `presentation.rs`, `session.rs`, `workspace.rs`, `skill.rs`, `geospatial.rs` and `geospatial_page.rs`, shared effects/styles, manifests, Make staging and planning indexes. Preserve unrelated concurrent PA/CI changes. Use focused leaf tests during implementation and serialize broad read-only gates; do not run workspace formatting or `clippy --fix` over concurrent edits.

Next, execute the remaining focused acceptance matrices in canonical dependency order, then ABW-08 with available Firefox/WebKit and assistive technology. Record unavailable rows explicitly. Native integration begins with ABW-11 only when its browser/map prerequisites are accepted and the user authorizes that scope. No commits, publication, signing, account changes or hosted deployment were performed in this batch.
