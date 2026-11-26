// SPDX-License-Identifier: MIT
// Supply a locally installed Khronos gltf-validator module path; never fetch at runtime.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { dirname, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const validator = require(process.argv[2] ? resolve(process.argv[2]) : "gltf-validator");
const root = process.argv[3] ? resolve(process.argv[3]) : resolve(dirname(fileURLToPath(import.meta.url)), "../assets/models");
const catalog = JSON.parse(await readFile(`${root}/catalog.json`, "utf8"));
const palette = JSON.parse(await readFile(`${root}/palette.json`, "utf8"));
const reports = [];
assert.equal(catalog.version, 1);
assert.ok(catalog.assets.length > 0);
assert.equal(new Set(catalog.assets.map((asset) => asset.id)).size, catalog.assets.length);
for (const asset of catalog.assets) {
    const file = resolve(root, asset.file);
    assert.ok(file.startsWith(`${root}${sep}`));
    const bytes = await readFile(file);
    assert.equal(bytes.length, asset.bytes);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), asset.sha256);
    const gltf = JSON.parse(bytes.subarray(20, 20 + bytes.readUInt32LE(12)).toString("utf8"));
    for (const material of gltf.materials) {
        const natural = palette.naturalFinishes?.[material.extras.colorName];
        const accent = palette.customAccents?.[material.extras.colorName];
        assert.equal(material.extras.brand, accent ? "custom accent" : natural ? "natural finish" : "ORNL");
        const hex = accent ?? natural ?? palette.srgb[material.extras.colorName];
        assert.equal(material.extras.srgbHex, hex);
        assert.ok(hex);
        const linear = [1, 3, 5].map((offset) => {
            const value = Number.parseInt(hex.slice(offset, offset + 2), 16) / 255;
            return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
        });
        linear.forEach((value, axis) => assert.ok(Math.abs(value - material.pbrMetallicRoughness.baseColorFactor[axis]) < 1e-7));
    }
    const report = await validator.validateBytes(new Uint8Array(bytes), { uri: asset.file });
    assert.equal(report.issues.numErrors, 0, JSON.stringify(report.issues));
    assert.equal(report.issues.numWarnings, 0, JSON.stringify(report.issues));
    assert.equal(report.info.totalTriangleCount, asset.triangles);
    assert.equal(report.info.materialCount, asset.materials);
    assert.equal(report.info.drawCallCount, asset.primitives);
    assert.ok(report.info.resources.every((resource) => ["glb", "buffer-view"].includes(resource.storage)), "External model resource");
    assert.ok(gltf.buffers.every((buffer) => buffer.uri === undefined));
    assert.ok(gltf.images.every((image) => image.uri === undefined && Number.isInteger(image.bufferView)));
    assert.equal(gltf.textures.length, asset.textures);
    reports.push({ id: asset.id, sha256: asset.sha256, palette: "ORNL with recorded natural finishes and requested custom accents", ...report });
    console.log(`${asset.id}: Khronos ${report.validatorVersion}, zero errors/warnings; catalog hash, size and geometry counts match`);
}
if (process.argv[4]) await writeFile(resolve(process.argv[4]), `${JSON.stringify(reports, null, 2)}\n`);
