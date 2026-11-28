# ACORN browser component catalog

Run `make design-preview` to serve the catalog on port 8000 and open it in your default browser. Set `DESIGN_SYSTEM_PORT` to use another port. The page also opens as a local file. It uses the production `assets/app.css` and local catalog layout CSS. Each section shows its `#<id>` beside its title, and each example has a `#<section>-<example>` label. Hover a label for its full name. Section IDs also appear in `components.json` schema version 1. Switch appearance at the top to inspect Dark, Light, and USWDS. Use Tab to inspect focus styles. At 390px, the catalog becomes one column.

The **Agent avatar** specimen shows Oakley's static states and links to the interactive [Oakley preview](oakley/README.md) in `oakley/` (`make oakley-preview` opens it directly), which holds the avatar's design spec, SVG generator, and SVGs. `npm --prefix crates/acorn-web run test:design-system` runs the catalog and Oakley checks. This folder and those tests are excluded from Cargo packages.

The catalog contains synthetic visual examples. Interactive Dioxus, editor, drawer, map, and dashboard behavior is exercised by `scripts/parity-smoke.mjs` against the built application. A static specimen does not establish keyboard or screen-reader behavior.
