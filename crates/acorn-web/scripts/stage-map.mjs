import { cp, mkdir, readFile, readdir, rm, writeFile } from "node:fs/promises";
const root = new URL("../geospatial/", import.meta.url);
const destination = new URL("../assets/generated/geospatial/", import.meta.url);
await rm(destination, { recursive: true, force: true });
await mkdir(destination, { recursive: true });
await cp(new URL("dist/", root), destination, { recursive: true });
const notices = [];
for (const entry of (await readdir(new URL("node_modules/", root))).sort()) {
    if (entry.startsWith(".")) continue;
    const packages = entry.startsWith("@") ? (await readdir(new URL(`node_modules/${entry}/`, root))).map((name) => `${entry}/${name}`) : [entry];
    for (const name of packages) {
        const directory = new URL(`node_modules/${name}/`, root);
        const files = await readdir(directory);
        for (const file of files.filter((file) => /^(license|licence|notice)(\.|$)/i.test(file))) {
            notices.push(`${name}/${file}\n${await readFile(new URL(file, directory), "utf8")}`);
        }
    }
}
await writeFile(new URL("THIRD_PARTY_NOTICES.txt", destination), notices.join("\n\n"));
console.log("Staged local map, workers, imagery and license notices.");
