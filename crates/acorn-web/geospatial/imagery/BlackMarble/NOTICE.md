# NASA Black Marble

NASA Black Marble (VIIRS, Suomi NPP), 2016 composite. Imagery credit:
NASA Earth Observatory / NOAA NGDC. Distributed by NASA Global Imagery
Browse Services (GIBS).

These unchanged PNG tiles use the `VIIRS_Black_Marble` layer dated
`2016-01-01`, Web Mercator `GoogleMapsCompatible_Level8`, levels 0–3.
The URL layout is `{level}/{row}/{column}.png`; Cesium caps requests at
level 3 and magnifies those tiles for closer views. All 85 tiles are bundled
so viewing the map makes no requests to NASA or Cesium ion.

Sources: [NASA Black Marble](https://blackmarble.gsfc.nasa.gov/),
[GIBS access documentation](https://nasa-gibs.github.io/gibs-api-docs/access-basics/),
[NASA imagery usage guidelines](https://www.nasa.gov/nasa-brand-center/images-and-media/).
NASA imagery is generally available for educational and informational use;
no NASA endorsement is implied. Preserve attribution when redistributing.

Regenerate with `node crates/acorn-web/scripts/fetch-black-marble.mjs`.
`manifest.json` records the source URL and SHA-256 of each downloaded tile.
