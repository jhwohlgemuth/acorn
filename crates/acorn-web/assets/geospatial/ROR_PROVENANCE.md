# Bundled organization locations

Source: [ROR Data 2.15, October 20, 2026](https://doi.org/10.5281/zenodo.23500708), Schema 2.1, [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/).

- Archive: `v2.15-2026-10-20-ror-data.zip`, 39,263,398 bytes.
- Archive SHA-256: `7a7bf58d5ac18436ed98daf5da906913761d54869edf86d405e51966609dd133`.
- JSON SHA-256: `16b2ec951d498848bf44f6a53e17fdacbcbba22b7b0d921d93a829e5d36b9930`.
- 146,716 organization names, identifiers and location arrays; no contacts, external identifiers or relationships are retained.
- Generated JSON: 22,282,930 bytes; gzip: 4,962,218 bytes. Loaded only on the Geospatial route. This is the initial data budget, not a measured browser memory budget.

Download the immutable archive from `https://zenodo.org/api/records/23500708/files/v2.15-2026-10-20-ror-data.zip/content`, verify its digest, and extract the JSON. Regenerate with:

```sh
node crates/acorn-web/scripts/build-ror-location-index.mjs /path/to/v2.15-2026-10-20-ror-data.json crates/acorn-web/assets/geospatial/ror-locations.json
```

The generator sorts IDs and locations deterministically, retains locationless organizations and excludes invalid coordinates. The map uses the first location in country/coordinate order; the full location array remains in this index. It does not infer an organization’s headquarters. Unknown RORs remain visible as unresolved. Refreshing the index is a reviewed build-time operation; the app never queries ROR at runtime.

The renderer uses locally bundled NASA Black Marble (2016) imagery, an ellipsoid terrain surface and no ion access token, geocoder, online base-layer picker or remote terrain. [Cesium’s offline guide](https://github.com/CesiumGS/cesium/blob/main/Documentation/OfflineGuide/README.md) describes this configuration. Cesium is Apache-2.0; React, Resium and Vite are MIT. Upstream notices accompany the generated map assets.
