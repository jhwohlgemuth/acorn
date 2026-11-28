# DOE laboratory marker logos

Official artwork for the 16 DOE laboratories other than Oak Ridge, whose marker is maintained separately. Sources were checked on November 27, 2026. [View all 17 rendered markers](../../previews/laboratory-markers-overview.png). [catalog.json](catalog.json) records the download URLs, source pages, local files, and rendering notes.

| Laboratory | Viewer abbreviation | Retained artwork | Official source |
| --- | --- | --- | --- |
| Ames National Laboratory | AMES | [SVG](ames.svg) | [Ames homepage](https://www.ameslab.gov/) |
| Argonne National Laboratory | ANL | [PNG](argonne.png) | [ALCF media kit](https://www.alcf.anl.gov/promotional-materials/argonne-logo-cmyk-black) |
| Brookhaven National Laboratory | BNL | [SVG](brookhaven.svg) | [Brand Center](https://www.bnl.gov/brandcenter/logo.php) |
| Fermi National Accelerator Laboratory | FNAL | [PNG](fermilab.png) | [Fermilab logo standards](https://www.fnal.gov/faw/designstandards/logo.html) |
| Idaho National Laboratory | INL | [PNG](idaho.png) | [INL homepage](https://inl.gov/) |
| Lawrence Berkeley National Laboratory | LBNL | [PNG](berkeley.png) | [Berkeley Lab logo resources](https://dreambeam.lbl.gov/about/acknowledgements/logos) |
| Lawrence Livermore National Laboratory | LLNL | [SVG](livermore.svg) | [LLNL Software Portal](https://software.llnl.gov/) |
| Los Alamos National Laboratory | LANL | [SVG](los-alamos.svg), [CSS](los-alamos.css) | [LANL homepage](https://www.lanl.gov/) |
| National Energy Technology Laboratory | NETL | [PNG](netl.png) | [NETL logo resources](https://netl.doe.gov/about/netl-logo) |
| National Laboratory of the Rockies | NLR | [SVG](rockies.svg) | [NLR logo standards](https://www.nlr.gov/comm-standards/brand/logo) |
| Pacific Northwest National Laboratory | PNNL | [SVG](pnnl.svg) | [PNNL homepage](https://www.pnnl.gov/) |
| Princeton Plasma Physics Laboratory | PPPL | [PNG](pppl.png), [source PDF](pppl.source.pdf) | [FY2025 Research Accomplishments, page 1](https://www.pppl.gov/sites/g/files/toruqf286/files/documents/research_accomplishments_fy25_final.pdf#page=1) |
| SLAC National Accelerator Laboratory | SLAC | [GIF](slac.gif) | [SLAC logo resources](https://www6.slac.stanford.edu/about/resources/logos) |
| Sandia National Laboratories | SNL | [JPEG](sandia.jpg) | [Sandia Combustion Research Facility](https://crf.sandia.gov/2017/11/29/new-engine-optics-to-fuel-future-research/) |
| Savannah River National Laboratory | SRNL | [SVG](savannah-river.svg) | [SRNL homepage](https://www.srnl.gov/) |
| Thomas Jefferson National Accelerator Facility | JLAB | [PNG](jefferson.png) | [Jefferson Lab logo resources](https://www.jlab.org/about/comms/resources/jlab-and-partners) |

DOE renamed the National Renewable Energy Laboratory (NREL) to the National Laboratory of the Rockies (NLR), effective December 1, 2025. This collection uses its current name and artwork. [DOE announcement](https://www.energy.gov/cmei/articles/energy-department-renames-nrel-national-lab-rockies).

## Text-free marker geometry

All 17 laboratory markers, including ORNL, now contain only a raised white graphic emblem on a pin in its laboratory’s brand color. The former white plaques, full laboratory names and separate abbreviations are removed. Viewer labels identify each laboratory; those labels are not part of the GLB geometry.

[emblems/catalog.json](emblems/catalog.json) records the selected source components, source hashes, transparent white masks and any adaptations. Most symbols are isolated directly from original vector paths or raster artwork. NETL uses its three rounded graphic bars; SLAC uses the accelerator arc and particle portion of its mark. These are graphic excerpts, not officially published standalone logos. Idaho and Jefferson require reconstruction of short orbit sections obscured by their wordmarks; their visible contours, proportions and orbital motifs are retained, and the reconstruction is recorded in the manifest. Original source artwork is retained unchanged, including additional official sources in `emblems/sources/`.

The original artwork remains in its published colors. The requested marker treatment uses monochrome white silhouettes, preserving open cutouts and separated components. The retained transparent masks are the editable input for contour tracing. Tracing removes pixel-scale stair steps; integer-coordinate triangulation preserves holes and avoids degenerate faces. Emblem meshes are shallow closed extrusions, not textured rectangles. Both faces are readable, with the rear emblem rotated to face outward.

## Laboratory body colors

[colors.json](colors.json) records the chosen HEX values, evidence and selection rationale. Published primary standards take precedence; where a public standard was unavailable, the color is sampled from official identity artwork and explicitly identified as a fallback. Multiple-primary identities use one representative color. Sandia’s published primary specification is from its 2011 guide and is identified as such. ORNL retains ACORN’s existing primary green. White emblem geometry and metallic rims are unchanged.

| Lab | Body color | Source |
| --- | --- | --- |
| AMES | `#3C3B72` — Ames identity purple | [Source](https://www.ameslab.gov/themes/unobtainium/images/logo-narrow.svg) |
| ANL | `#0093D0` — Argonne logo blue | [Source](https://www.alcf.anl.gov/sites/default/files/2020-01/Argonne_cmyk_black.png) |
| LBNL | `#00313C` — Berkeley Dark Blue | [Source](https://creative.lbl.gov/visual-identity/) |
| BNL | `#105C78` — Brookhaven Teal | [Source](https://www.bnl.gov/brandcenter/palette.php) |
| FNAL | `#004C97` — Fermilab NAL Blue | [Source](https://www.fnal.gov/faw/designstandards/color-palette.html) |
| INL | `#07519E` — INL identity blue | [Source](https://inl.gov/content/uploads/2025/02/logo-inl-two-tone-blue.png) |
| JLAB | `#A6192E` — Jefferson Lab Red | [Source](https://www.jlab.org/sites/default/files/documents/comms/cs/JLabGraphicStandards2023_F1.pdf#page=8) |
| LLNL | `#0032A1` — LLNL logo blue | [Source](https://software.llnl.gov/assets/images/logomark-llnl.svg) |
| LANL | `#000F7E` — LANL Ultramarine | [Source](https://www.osti.gov/servlets/purl/1764177#page=22) |
| NETL | `#97C83D` — NETL logo green | [Source](https://netl.doe.gov/sites/default/files/2023-05/NETL%20Stacked%20Color.png) |
| ORNL | `#00662C` — ORNL Green | [Source](../../../../../acorn-lib/src/util/constants/app.rs) |
| PNNL | `#D77900` — PNNL Emblem Orange | [Source](https://www.pnnl.gov/themes/gesso/images/pnnl-logo.svg) |
| PPPL | `#F46F1B` — PPPL Logo Orange | [Source](https://www.pppl.gov/sites/g/files/toruqf286/files/documents/research_accomplishments_fy25_final.pdf#page=1) |
| NLR | `#003A69` — NLR Rocky Blue | [Source](https://www.nlr.gov/comm-standards/brand/colors-typography) |
| SNL | `#003359` — Sandia Primary Navy | [Source](https://www.osti.gov/servlets/purl/1661563#page=14) |
| SRNL | `#005288` — SRNL Blue | [Source](https://www.srnl.gov/wp-content/uploads/srnl-logo.svg) |
| SLAC | `#8C1515` — SLAC Red | [Source](https://www6.slac.stanford.edu/about/resources/logos) |

## Rights

Laboratory names, logos and associated trademarks retain their owners' rights. They are not relicensed under ACORN's MIT license. The marker geometry and authoring code are MIT-licensed. These source-derived marker adaptations identify laboratory locations and do not represent endorsement or newly approved brand assets. Source pages and original artwork remain available in the table above and both manifests.

## Regeneration

From the repository root, with Pillow available for tracing and the web crate's installed Earcut dependency available to Node:

```sh
python3 crates/acorn-web/scripts/model-logo-emblems.py
python3 crates/acorn-web/scripts/model-lab-markers.py
python3 crates/acorn-web/scripts/model-ornl-marker.py
python3 crates/acorn-web/scripts/build-models.py
python3 crates/acorn-web/scripts/build-models.py --check
```

The first command traces retained masks into `.contours.json` files. The second authors the 16 laboratory recipes and six unchanged plain variants; the third authors the separate ORNL marker. The builder creates GLBs and the catalog from saved recipes. Logo images are not embedded in the current markers. Regeneration uses local sources and needs no downloads. The old plaque-texture generator and its baked white-background textures have been removed.
