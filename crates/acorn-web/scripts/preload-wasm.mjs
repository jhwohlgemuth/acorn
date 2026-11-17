import assert from "node:assert/strict";
import { readdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";

// Start the WASM download with the HTML instead of after the loader module evaluates.
const root = process.argv[2];
assert.ok(root, "Pass the built public directory");
const indexPath = join(root, "index.html");
const html = await readFile(indexPath, "utf8");
const script = html.match(/<script type="module"[^>]*src="([^"]+)"/)?.[1];
assert.ok(script, "Missing application module script");
const loader = await readFile(join(root, script.slice(script.indexOf("/assets/") + 1)), "utf8");
const wasm = loader.match(/module_or_path:"([^"]+\.wasm)"/)?.[1];
assert.ok(wasm, "Missing WASM path in the application module");
const stylesheets = (await readdir(join(root, "assets"))).filter((name) => /^app(?:-dxh[^/]+)?\.css$/.test(name));
assert.equal(stylesheets.length, 1, "Expected one application stylesheet");
const stylesheet = `${wasm.slice(0, wasm.lastIndexOf("/") + 1)}${stylesheets[0]}`;
const cssLink = `<link id="initial-app-stylesheet" rel="stylesheet" href="${stylesheet}" onload="window.markStylesReady()" onerror="window.showStylesError()">`;
const wasmLink = `<link rel="preload" as="fetch" type="application/wasm" href="${wasm}" crossorigin>`;
const additions = [cssLink, wasmLink].filter((link) => !html.includes(link)).join("");
if (additions) await writeFile(indexPath, html.replace("</head>", `${additions}</head>`));
console.log(`Application stylesheet: ${stylesheet}; WASM preload: ${wasm}`);
