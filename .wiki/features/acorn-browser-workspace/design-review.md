# OKF Browser design specification — November 11, 2026

The [OKF Browser](https://www.jasonwohlgemuth.com/okf-plan-browser/home) is the design authority for ACORN's browser and desktop UI. The individual SVGs for all 16 sidebar tabs linked from the [plan](index.md#design-preview) apply its shell and page hierarchy to ACORN, with additional theme, responsive and grouped comparison boards. They specify intended appearance; they are not screenshots of an implemented ACORN application.

## Verified reference and preview evidence

The live reference Home, About, Install, Planning Skill, Settings and loaded Portfolio were inspected at 1440×900. Settings was compared in dark and light themes. Portrait 390×844 and landscape 844×390 layouts were also inspected. The revised SVGs were rendered locally and inspected for text, icon, card and navigation bounds. Each tab has a standalone 1440×900 desktop drawing plus its annotation strip. Grouped tool boards retain identical full-size views for comparison.

| Element | Current ACORN specification |
|---|---|
| Desktop shell | 1440×900 viewport, 220 px expanded sidebar, compact outlined icons, 33 px navigation rows, muted borders and restrained active-row tint. A separate 32 px strip identifies sample design content outside the viewport. |
| Palette | Charcoal canvas `#20232a`, sidebar `#262932`, cards `#282c34`, borders `#3c424e`, text `#e4e7ed`, muted text `#a0a7b4`, green active tint `#294337`, primary action `#00662c`. Light mode uses white and pale-gray surfaces. |
| Home | Centered 420 px column: ACORN mark/title, adjacent tour/sample actions, prominent skill card, local file/folder loading, supporting privacy copy and installation link. |
| About | 624 px narrative column with source card, app/skill relationship, platform notes, existing technology artwork and AI-assistance disclosure. |
| Install and Skill | 848 px content columns. Install separates desktop distribution from skill acquisition. Skill exposes the bundled release version, source, raw definition, complete download and workflow examples. |
| Settings | Narrow 464 px column with appearance, session state and app/skill release identity; both themes are drawn. |
| Portfolio and tools | Compact filters and table treatment follow the reference while retaining RAD metrics and selected-record details. All nine adapter tools have manual routes, including Big-O comparison. Portfolio type/status bars and selected-record relationships, FAIR proportion bars, a readability target gauge, a diff before/after diagram, and the labeled Big-O example supplement visible data. |
| Responsive navigation | Separate portrait, landscape, collapsed-rail and drawer boards. Portrait stacks cards and uses list-first record presentation; landscape uses a top bar to preserve workspace height. |

The reference [Install page](https://www.jasonwohlgemuth.com/okf-plan-browser/install) separates desktop-app and skill acquisition. Its [Planning Skill page](https://www.jasonwohlgemuth.com/okf-plan-browser/planning) identifies a canonical source, describes build-time embedding, and offers the complete skill folder. These are observed presentation and ownership patterns; reference artifacts and release automation were not installed or verified.

## Deliberate ACORN adaptations

- Preserve the repository's ACORN mark, green accent, existing technology logos, author-only footer and About disclosure.
- Keep adjacent tour/sample actions and local file/folder entry points. The loaded workspace represents Research Activities, not OKF plans.
- Expose Install and ACORN Skill alongside Settings/About. WinGet, Scoop, Debian, AppImage and Homebrew cask remain visibly pending until actual GUI publication is verified; the previews invent no public package identities or download endpoints.
- Bundle the canonical ACORN skill with the matching application release. Exported agent copies remain user-managed; app upgrades and removal do not silently replace or delete them.
- Provide the ACORN-specific geospatial route with an equivalent organization table. Its offline, selection and WebGL behavior requires independent acceptance because the reference does not supply that workflow.
- Use top-bar/drawer navigation at short landscape heights. The observed reference retained its desktop sidebar at 844×390; ACORN's adaptation preserves usable content height.

## Exceptional-state acceptance — November 11, 2026

Native Chrome access was restored for this pass. Home, empty Portfolio and Planning Skill were inspected with the responsive viewport explicitly set to 1440×900. Portfolio initially failed to load a stale dynamic module; reloading the route restored the current reference. The reference retains a 220 px sidebar, compact outlined navigation, restrained borders and an 848 px Skill column. ACORN keeps that shell and content rhythm, with the documented green branding and Research Activity content. Its empty Portfolio offers an explicit Home action instead of the reference's empty filtered-plan message.

All six new SVGs were parsed, rendered with `rsvg-convert`, and visually inspected at 1440×900 plus the 32 px annotation strip. The two data tables were corrected to use positioned columns after the first render exposed collapsed whitespace. Final renders have readable text, distinct columns, contained actions and no overlap with navigation or footer.

| State | Accepted design evidence |
|---|---|
| Empty Portfolio | [Preview](_assets/acorn-browser-portfolio-empty-preview.svg): no invented counts or charts; Home entry action. |
| Single-record Portfolio | [Preview](_assets/acorn-browser-portfolio-single-preview.svg): one-record counts, labeled 100% bar, table and selection details. |
| Invalid input | [Preview](_assets/acorn-browser-validation-invalid-preview.svg): preserved input, diagnostic paths and retry action; illustrative diagnostics are labeled. |
| Oversized input | [Preview](_assets/acorn-browser-validation-oversized-preview.svg): byte limit, selected size, recovery and preserved previous content. |
| WebGL unavailable | [Preview](_assets/acorn-browser-geospatial-unavailable-preview.svg): local organization table, unresolved location and no remote-map fallback. |
| Skill version mismatch | [Preview](_assets/acorn-browser-skill-mismatch-preview.svg): both versions, unavailable exports and recovery without inspecting agent folders. |

These states complete ABW-20's visual specification. Their ACORN-specific behaviors have no equivalent in the reference and remain implementation acceptance requirements for their owning phases. ABW-02 and ABW-08 own live keyboard, screen-reader, responsive reflow, interaction and browser acceptance. Static SVG rendering cannot establish those behaviors. The earlier `75cc88ec` review preceded the accepted local nine-operation browser; current implementation evidence is in the browser README. No distribution channel is accepted by this design review.
