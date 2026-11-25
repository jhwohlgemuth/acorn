import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { runInNewContext } from "node:vm";

test("project download contains every UTF-8 source file", async () => {
    const source = await readFile(new URL("../assets/scripts/download-project.js", import.meta.url), "utf8");
    let archive;
    const context = {
        __acornArgs: [
            { path: "starter/index.json", content: "{\"title\":\"Café\"}" },
            { path: "notes/readme.txt", content: "Line one\nLine two\n" },
        ],
        Blob,
        DataView,
        TextEncoder,
        Uint8Array,
        URL: { createObjectURL: (blob) => { archive = blob; return "blob:project"; }, revokeObjectURL: () => {} },
        document: { createElement: () => ({ click() {} }) },
        setTimeout: () => {},
    };
    runInNewContext(source, context);
    const directory = await mkdtemp(join(tmpdir(), "acorn-template-zip-"));
    const path = join(directory, "project.zip");
    try {
        await writeFile(path, Buffer.from(await archive.arrayBuffer()));
        const listing = execFileSync("unzip", ["-Z", "-1", path], { encoding: "utf8" });
        assert.deepEqual(listing.trim().split("\n"), ["starter/index.json", "notes/readme.txt"]);
        assert.equal(execFileSync("unzip", ["-p", path, "starter/index.json"], { encoding: "utf8" }), "{\"title\":\"Café\"}");
        assert.match(execFileSync("unzip", ["-t", path], { encoding: "utf8" }), /No errors detected/);
    } finally {
        await rm(directory, { recursive: true, force: true });
    }
});
