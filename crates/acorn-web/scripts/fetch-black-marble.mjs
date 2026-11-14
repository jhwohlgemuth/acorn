import { createHash } from "node:crypto";
import { mkdir, writeFile } from "node:fs/promises";

const destination = new URL("../geospatial/imagery/BlackMarble/", import.meta.url);
const source = "https://gibs.earthdata.nasa.gov/wmts/epsg3857/best/VIIRS_Black_Marble/default/2016-01-01/GoogleMapsCompatible_Level8/";
const tiles = {};
for (let level = 0; level <= 3; level += 1) {
    for (let row = 0; row < 2 ** level; row += 1) {
        await mkdir(new URL(`${level}/${row}/`, destination), { recursive: true });
        const downloaded = await Promise.all(Array.from({ length: 2 ** level }, async (_, column) => {
            const path = `${level}/${row}/${column}.png`;
            const response = await fetch(source + path, { signal: AbortSignal.timeout(30000) });
            if (!response.ok) throw new Error(`${path}: HTTP ${response.status}`);
            const bytes = Buffer.from(await response.arrayBuffer());
            if (bytes.subarray(0, 8).toString("hex") !== "89504e470d0a1a0a") throw new Error(`${path}: expected PNG`);
            await writeFile(new URL(path, destination), bytes);
            return [path, createHash("sha256").update(bytes).digest("hex")];
        }));
        Object.assign(tiles, Object.fromEntries(downloaded));
    }
}
await writeFile(new URL("manifest.json", destination), JSON.stringify({ source, tiles }, null, 2) + "\n");
console.log(`Saved ${Object.keys(tiles).length} NASA Black Marble tiles and SHA-256 manifest.`);
