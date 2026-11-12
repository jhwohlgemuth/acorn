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

## Remaining acceptance

ABW-20 retains only empty/single-record Portfolio, invalid/oversized tool input, WebGL-unavailable map and skill version-mismatch visual states, plus final comparison of those states with the established design. The revised normal-state boards, theme variants and responsive navigation are the current design specification and must change with subsequent design decisions.

ABW-08 owns live keyboard, screen-reader, responsive reflow, interaction and browser acceptance. Static SVG rendering cannot establish those behaviors. ACORN product crates remain absent at repository baseline `75cc88ec`; no running ACORN UI or distribution channel is accepted by this preview update.
