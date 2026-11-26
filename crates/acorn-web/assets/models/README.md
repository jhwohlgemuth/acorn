# Geospatial model assets

This managed library contains **40 original GLBs**: 12 research objects, 8 U.S. landmarks, 8 world wonders and 12 drones. The separate **12-piece Risk collection** is delivered under `/Users/o9w/dev/risk/assets/models/`. Models follow the crate-owned asset pattern used by migrations and prompts: versioned files with editable sources, stable IDs, provenance and deterministic output.

Models retain ACORN's shared ORNL greens, blues and neutrals. At the user's request, orange roles now use natural earth finishes: burnt umber (`#9A563A`), oxide red (`#A5543B`), warm ochre (`#9F794B`), canyon clay (`#95674C`), weathered sandstone (`#A9755B`), limestone (`#C2AF90`) and antique brass (`#9B804A`). Liberty retains blue-green copper patina (`#78A99A`). The supercomputer uses bright cyan (`#00E5FF`) illuminated accent strips in place of yellow/ochre. These are illustrative material colors, not measured samples. [palette.json](palette.json) distinguishes official brand values from natural finishes; material factors are converted from sRGB to glTF linear RGB and retain color names/hex values in `extras`.

The detail pass adds mechanical vents, fans, fasteners, cables and lenses; architectural columns, stone courses, stairs and trim; terrain strata; and clothing and game-piece details. Collaboration is three connected spheres. Research Project is a computer workstation: a monitor with line, bar, donut and network visualizations, stand, rear vents/ports, keyboard, mouse, tower, open book, stacked papers, pencil and mug. The chemistry setup and task board have been removed. Embedded roughness grain and material-specific metal/paint/stone finishes provide surface variation. These remain illustrative miniatures rather than photogrammetric replicas.

Cargo crate packages exclude this entire directory and all model generation, validation, preview and viewer tools through workspace exclusions. `acorn-web` inherits them and remains `publish = false`. These authoring assets are also not automatically staged into browser releases.

## Collection

The [catalog](catalog.json) is the authoritative inventory, including SHA-256, bytes, source path, bounds, axes, revision, nodes, material count, triangle count and draw primitives. Current ACORN GLBs total 43,150,812 bytes, uncompressed. Counts describe artifacts; they are not a production scene performance budget.

| Category | Models | GLB bytes | Triangles | Actual renders |
|---|---:|---:|---:|---|
| research | 12 | 9,671,840 | 99,584 | [Preview](previews/research-overview.png) |
| landmarks | 8 | 8,121,912 | 83,890 | [Preview](previews/landmarks-overview.png) |
| wonders | 8 | 8,673,084 | 89,622 | [Preview](previews/wonders-overview.png) |
| drone | 12 | 16,683,976 | 172,644 | [Preview](previews/drone-overview.png) |

- **Research:** laboratory, generic DOE national laboratory, user facility, supercomputer, project, dataset, software, publication, team, funding, collaboration and field site.
- **Landmarks:** Statue of Liberty, Golden Gate Bridge, U.S. Capitol, Gateway Arch, Grand Canyon, Yellowstone, Kennedy Space Center and Hoover Dam.
- **Wonders:** Great Wall, Petra, Christ the Redeemer, Machu Picchu, Chichen Itza, Colosseum, Taj Mahal and the Great Pyramid of Giza.
- **Drones:** the twelve platforms and source/selection notes are in [drones/README.md](drones/README.md). The set is representative; public evidence does not establish a global top-12 usage ranking or ORNL's complete fleet inventory.
- **Risk:** infantry, cavalry, artillery, scout, squadron, legion, and circle/triangle/diamond/square/plus/star faction bases. They have their own catalog, source recipes, build tools, license and renders at the requested separate destination.

## File and coordinate contract

Keep each `.model.json` beside its `.glb`, and commit the regenerated catalog. Increment revision when appearance, geometry, materials or coordinates change. The detail/earth-finish collection is revision 4; Liberty, the supercomputer and the redesigned Research Project are revision 5. Stable IDs do not change with revision.

Files are self-contained glTF 2.0 GLB with embedded geometry buffers and a deterministic 64×64 roughness texture, with no external textures, URIs or decoder dependencies. Units are meters, +Y is up, +Z front, and the origin is at the bottom center. These are fitted miniature dimensions, not surveyed geographic footprints or actual aircraft sizes. The `pedestal` and `body` are separate nodes; supercomputer racks retain their separate nodes. Risk base symbols are separately addressable. Hide the display pedestal when using a standalone object if appropriate.

The geometry and authoring code are original MIT-licensed work under the repository license. Drone references guide simplified silhouettes; no manufacturer mesh was imported. Generic laboratory symbols do not identify real facilities. [Original design sheets and prompts](designs/) preserve the concept references; their earlier color choices and concept-only brief are superseded by the generated models and current palette. None of these assets implement game rules or bind an object to a real-world location.

## Browse the collection

From the ACORN repository root:

```sh
npm --prefix crates/acorn-web run models
```

Open [localhost:8085](http://127.0.0.1:8085/) and refresh the picker after generation. It starts with the supercomputer. Drag to orbit, scroll or pinch to zoom, or use Front, Back, Top and Reset view. Focus the preview for arrow-key orbit, `+`/`−` zoom and `R` reset. The single-page viewer uses ACORN’s shared application stylesheet and theme control, without a left navbar or page title. Collection/search filters, the picker, preview and asset inspector remain available; preview controls use the shared compact choice-control styles. Dark/light theme choices persist locally. Filters narrow the picker while leaving the current preview visible until another model is selected. Failed selections retain the last successful model.

The server only exposes the viewer, shared ACORN stylesheet/icons, catalog, GLBs and installed Cesium files over loopback; rendering uses no ion token or network service. Install the existing Cesium dependency with `npm ci --prefix crates/acorn-web/geospatial` if missing. Ctrl+C stops the server. To browse Risk separately:

```sh
node crates/acorn-web/scripts/model-viewer.mjs 8086 /Users/o9w/dev/risk/assets/models
```

Open [localhost:8086](http://127.0.0.1:8086/). Any model root with the same catalog contract can be supplied as the second server argument.

## Author, rebuild and validate

Recipes are the canonical editable source. The collection generator can re-author a whole category from procedural designs; this overwrites that category's recipes, so preserve manual recipe edits before using it. The supercomputer remains its own hand-authored recipe.

```sh
python3 crates/acorn-web/scripts/model-collection.py --category drone
python3 crates/acorn-web/scripts/build-models.py
python3 crates/acorn-web/scripts/build-models.py --check
npm install --prefix /tmp/acorn-model-validation --ignore-scripts --no-audit --no-fund gltf-validator@2.0.0-dev.3.10
node crates/acorn-web/scripts/validate-models.mjs /tmp/acorn-model-validation/node_modules/gltf-validator
node crates/acorn-web/scripts/model-collection-smoke.mjs crates/acorn-web/assets/models drone
npm --prefix crates/acorn-web run test:models-viewer
```

Drone files live in `drones/`; stable IDs and the generator category remain `drone` to preserve existing references. Categories are `drone`, `research`, `landmarks`, `wonders` and `risk`. Keep Risk exports in its own root by passing `--root /Users/o9w/dev/risk/assets/models` to both Python tools. Copied build tools also work directly from the Risk directory. Python uses only the standard library. Mesh helpers support extruded concave profiles, lathed bodies, beams, terrain, spheres, tori and beveled boxes; `model-detail.py` adds category-specific construction details; the builder batches triangles by node and material with deterministic normals, projected UVs and an embedded roughness texture.

The builder's `--check` compares regenerated bytes/catalog metadata without writing. Khronos validation also verifies catalog counts/hashes, embedded resources and every material against the recorded ORNL palette or requested natural finishes. The validator accepts an optional model root and optional report destination after the validator-module path. The collection smoke harness loads every selected GLB in Cesium, rejects external requests/page errors and saves front, rear, top and 220-pixel views. It accepts an optional category after the model root. `--contacts-only` refreshes contact sheets from existing per-model renders. Playwright and its browser binary must already be installed through the existing web tooling. Inspect the saved renders; successful loading alone is not visual acceptance.

## Acceptance and runtime integration

On November 26, 2026, all 52 models across ACORN and Risk passed deterministic regeneration, Khronos glTF Validator 2.0.0-dev.3.10 with zero errors/warnings, metadata and palette checks. Installed Cesium 1.147.0/Chromium loaded all models without external requests or page errors. Front, rear, top and small-size renders were inspected; [validation.json](validation.json) and category acceptance records under [previews](previews/) retain artifact hashes and results. The standalone viewer's picker, failure recovery, orbit/zoom, keyboard controls, downloads, search/category filters, persistent dark/light themes and mobile layout passed its focused checks. Cargo package listings exclude GLBs and associated tooling.

Live Geospatial-page integration, terrain placement, repeated-instance performance, memory budgets, distance switching and accessible production selection remain with ABW-37 after ABW-18 measurements. Asset-level rendering does not accept those production gates. Future browser packaging should copy only catalog-listed runtime files using base-path-safe URLs; recipes, concepts and previews remain authoring material.
