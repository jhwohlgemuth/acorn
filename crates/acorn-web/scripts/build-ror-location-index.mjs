import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";

export function locationIndex(records) {
    return Object.fromEntries(records.map((record) => {
        if (!/^https:\/\/ror.org\/0[a-z0-9]{8}$/.test(record.id)) throw new Error("Invalid ROR record ID");
        const name = record.names.find((name) => name.types.includes("ror_display"))?.value;
        if (!name) throw new Error(`Missing display name: ${record.id}`);
        const locations = (record.locations || []).map(({ geonames_details: details }) => details)
            .filter((details) => details && Number.isFinite(details.lat) && Number.isFinite(details.lng) && Math.abs(details.lat) <= 90 && Math.abs(details.lng) <= 180)
            .map((details) => ({ country: details.country_name, latitude: details.lat, longitude: details.lng }))
            .sort((a, b) => a.country < b.country ? -1 : a.country > b.country ? 1 : a.latitude - b.latitude || a.longitude - b.longitude);
        return [record.id, { name, locations }];
    }).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0));
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
    const [input, output] = process.argv.slice(2);
    if (!input || !output) throw new Error("Usage: node build-ror-location-index.mjs pinned-dump.json output.json");
    const bytes = await readFile(input);
    const digest = createHash("sha256").update(bytes).digest("hex");
    if (digest !== "16b2ec951d498848bf44f6a53e17fdacbcbba22b7b0d921d93a829e5d36b9930") throw new Error("Source digest does not match the pinned ROR release; review provenance before refreshing it.");
    const records = JSON.parse(bytes);
    const result = { version: "2.15", date: "2026-10-20", source: "https://doi.org/10.5281/zenodo.23500708", sourceSha256: digest, license: "CC0-1.0", organizations: locationIndex(records) };
    await writeFile(output, JSON.stringify(result) + "\n");
    console.log(JSON.stringify({ records: records.length, sourceSha256: digest, output }));
}
