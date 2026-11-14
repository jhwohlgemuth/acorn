import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import test from "node:test";

test("bundled Black Marble covers every requested tile with verified PNG bytes", async () => {
    const root = new URL("../imagery/BlackMarble/", import.meta.url);
    const manifest = JSON.parse(await readFile(new URL("manifest.json", root), "utf8"));
    assert.equal(Object.keys(manifest.tiles).length, 85);
    for (let level = 0; level <= 3; level += 1) {
        for (let row = 0; row < 2 ** level; row += 1) {
            for (let column = 0; column < 2 ** level; column += 1) {
                const path = `${level}/${row}/${column}.png`;
                const bytes = await readFile(new URL(path, root));
                assert.equal(bytes.subarray(0, 8).toString("hex"), "89504e470d0a1a0a", path);
                assert.equal(createHash("sha256").update(bytes).digest("hex"), manifest.tiles[path], path);
            }
        }
    }
});
