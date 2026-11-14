import { build } from "esbuild";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = new URL("../", import.meta.url);
const output = new URL("assets/generated/editor/", root);
await mkdir(output, { recursive: true });
const result = await build({
    absWorkingDir: fileURLToPath(root),
    entryPoints: ["editor/editor.js"],
    bundle: true, format: "iife", minify: true, target: "es2022", metafile: true,
    outfile: fileURLToPath(new URL("editor.js", output)),
});
const packages = [...new Set(Object.keys(result.metafile.inputs).flatMap((path) => {
    const match = path.match(/^node_modules\/((?:@[^/]+\/)?[^/]+)\//);
    return match ? [match[1]] : [];
}))].sort();
const notices = await Promise.all(packages.map(async (name) => {
    const directory = resolve(fileURLToPath(root), "node_modules", name);
    const manifest = JSON.parse(await readFile(resolve(directory, "package.json"), "utf8"));
    const license = await readFile(resolve(directory, "LICENSE"), "utf8");
    return `${name} ${manifest.version}\n${license}`;
}));
await writeFile(new URL("LICENSES.txt", output), notices.join("\n\n"));
console.log(`Bundled CodeMirror and ${packages.length} dependency notices locally.`);
