# ORNL East Campus quad

`research/ornl-campus` depicts the six-building East Campus group around Oak Ridge National Laboratory's central lawn: 5100, 5200, 5300, 5600, 5700 and 5800. The scene includes surrounding paths, roads, landscaping and the adjacent pond. It is an original architectural interpretation based on public maps and photographs. The crop does not cover the full ORNL reservation, SNS, HFIR or the older 4500 complex.

The model uses meter units and +Y up, with horizontal axes aligned to the campus map. Campus north differs from true north; the GLB has no geographic coordinate system, surveyed anchor or automatic globe placement. Footprints, facade divisions, heights, roof equipment, vegetation and terrain are approximations for visualization, rather than engineering CAD or a photogrammetric reconstruction.

## Buildings and layout

| Building | Identity | Spatial and visual reference |
|---|---|---|
| 5100 | Joint Institute for Computational Sciences | West side of the lawn; compact low-rise brick building with tall vertical windows and a pale roof. |
| 5200 | Research Support Center / visitor and conference center | East side of the lawn beside the pond; brick volumes and a prominent glazed visitor entrance nearest Bethel Valley Road. |
| 5300 | Multiprogram Research Facility | Northwest of 5100; stepped building footprint with brick, pale facade panels and large glazed sections. |
| 5600 | Computational Sciences Building | Southwestern part of the adjoining three-building complex; includes the OLCF computing facility. |
| 5700 | Research Office Building | Long frontage south of Central Avenue, facing the lawn. |
| 5800 | Engineering Technology Facility | Southeastern part of the adjoining complex, with laboratory and high-bay volumes. |

Building placement follows the campus map reproduced on page 3 of the [DOE-hosted November 2024 presentation](https://www.energy.gov/sites/default/files/2024-11/APPROVED%20SSAB%20Oak%20Ridge%20November%202024%20Meeting.pdf). The [official interactive campus map](https://map.ornl.org/) provides additional building context. Building numbers are identifiers for the public facilities, not labels for every modeled rooftop component.

Facade forms, masonry colors, glazing, roof equipment and planting were interpreted from ORNL's [main-campus aerial photograph](https://www.ornl.gov/wp-content/uploads/2026/08/ORNL_Aerial_main-campus-scaled.jpg), published with its [ORNL overview](https://www.ornl.gov/overview-ornl/). The photograph looks across the East Campus toward the older central campus. Architectural details inferred from that oblique view are simplified where obscured.

## Evidence and scale

The public [FEMA/ORNL USA Structures layer](https://services2.arcgis.com/FiaPA4ga0iQKduv3/arcgis/rest/services/USA_Structures_View/FeatureServer/0) supplies an independent footprint check. A November 26, 2026 query around the campus returned polygons attributed to ORNL. The relevant records have no height values. Their approximate dimensions after rotation into campus axes are:

| Building or group | Footprint bounding dimensions | Centroid relative to 5100, campus east/north |
|---|---|---|
| 5100 | 50 × 76 m | 0, 0 m |
| 5200 | 44 × 83 m | +144, +2 m |
| 5300 | 119 × 66 m | −41, +82 m |
| 5600 / 5700 / 5800, combined polygon | 188 × 116 m | +60, −116 m |

These figures are calculated from public geographic polygons using a local planar approximation; they are reference checks, not survey dimensions or guarantees of the model's exact bounds. The six-building footprint group spans approximately 265 × 286 m before the surrounding landscape. The [DOE modernization environmental assessment](https://www.energy.gov/sites/prod/files/2019/01/f58/ea-fonsi-ea-1618-ornl-modernization-2008-07-28.pdf) distinguishes ORNL North from True North by 34°12′51″, which was used to compare the footprint axes. Current GLB bounds, geometry counts and revision are recorded in [the catalog](../catalog.json).

Additional primary references establish the building identities:

- [UT National Institute for Computational Sciences facilities](https://nics.utk.edu/facilities/) identifies its facility in Building 5100.
- [Cope Architecture's visitor and conference center project](https://www.copearchitecture.com/project/ornl-visitors-conference-center/) identifies the Research Support Center and provides exterior architectural photography.
- [ORNL's Research Support Center construction report](https://web.ornl.gov/info/reporter/no49/june03.htm) places the center beside the pond and identifies its visitor, conference and cafeteria functions.
- [ORNL's 2013 sustainability report](https://info.ornl.gov/sites/publications/files/Pub47424.pdf) identifies Building 5300 as part of the Multi-Program Research Facility.
- [ORNL's 5600 / 5700 / 5800 energy study](https://info.ornl.gov/sites/publications/Files/Pub207081.pdf) identifies the Computational Sciences, Research Office and Engineering Technology buildings.
- [OLCF's Frontier visitor information](https://www.olcf.ornl.gov/wp-content/uploads/June-2022-Concall-QA.pdf) places Frontier's observation area in Building 5600. The campus model represents the building exterior; it does not recreate the computer room.

The asset's geometry is authored for ACORN under the repository license. Source imagery and maps retain their respective publishers' terms.

## Asset and rebuild

[Download the GLB](ornl-campus.glb) or choose **Research → ORNL — East Quad Campus** in the [local viewer](http://127.0.0.1:8085/). The self-contained file contains 436,262 triangles, 26 materials, 14 named scene nodes and six embedded textures (46,065,580 bytes). The campus is approximately 483 × 435 m including the pond and landscape crop. The six buildings, grounds, trees, parking, furniture and curtain walls have separate nodes.

The five color maps are 1024 × 1024 pixels, with a shared 64 × 64 roughness map. Natural brick, stone, asphalt, foliage and glass finishes are recorded in the collection palette; signage uses ORNL brand colors. The building envelopes and rooftop equipment are exterior approximations, without interior rooms.

```sh
python3 crates/acorn-web/scripts/model-ornl.py
python3 crates/acorn-web/scripts/build-models.py
python3 crates/acorn-web/scripts/build-models.py --check
```

The dedicated generator preserves this asset when other research symbols are regenerated. [The 2400-pixel preview](../previews/research-ornl-campus-hd.png) renders the actual GLB in Cesium with daylight, fill illumination and shadows; the ordinary viewer retains its shared lighting. The GLB has no baked lighting.
