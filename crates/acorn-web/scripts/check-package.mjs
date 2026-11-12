import assert from "node:assert/strict";
import { readdir, readFile, stat } from "node:fs/promises";
import { join } from "node:path";
const root = process.argv[2];
assert.ok(root, "Pass the built public directory");
const walk = async (path) => (await Promise.all((await readdir(path, { withFileTypes: true })).map(async (entry) => entry.isDirectory() ? walk(join(path, entry.name)) : [join(path, entry.name)]))).flat();
const files = await walk(root);
const wasm = files.filter((p) => p.endsWith(".wasm"));
assert.equal(wasm.length, 1, "The UI and ACORN must share exactly one WASM payload");
const compiled = await WebAssembly.compile(await readFile(wasm[0]));
for (const section of [".debug_info", ".debug_line", ".debug_abbrev", ".debug_str"]) {
    assert.equal(WebAssembly.Module.customSections(compiled, section).length, 0, `Release payload retains ${section}; build with --debug-symbols false`);
}
const modulePath = files.find((p) => /acorn-web(?:-dxh[^/]+)?\.js$/.test(p));
assert.ok(modulePath, "Missing application JS module");
const bindings = await readFile(modulePath, "utf8");
for (const name of ["assessFairMetadata", "checkReadability", "compareBigO", "crosswalkMetadata", "diffContent", "discoverPersistentIdentifiers", "inspectPersistentIdentifier", "validateMetadataStandard", "validateResearchActivity"]) {
    assert.ok(bindings.includes(`export function ${name}(`) || bindings.includes(`as ${name},`) || bindings.includes(`as ${name}}`), `Missing ${name}`);
}
const html = await readFile(join(root, "index.html"), "utf8");
assert.ok(!html.includes("fonts.googleapis.com"), "Runtime fonts must be local");
console.log(`Single WASM payload verified: ${(await stat(wasm[0])).size} bytes; bindings: ${Buffer.byteLength(bindings)} bytes; nine operation exports.`);
