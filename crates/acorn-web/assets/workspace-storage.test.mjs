import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { runInNewContext } from "node:vm";
import test from "node:test";

const script = await readFile(new URL("workspace-storage.js", import.meta.url), "utf8");
const memory = () => {
    const values = new Map();
    return { values, storage: { getItem: key => values.get(key) ?? null, setItem: (key, value) => values.set(key, value), removeItem: key => values.delete(key) } };
};
const create = storage => {
    const context = { window: {}, JSON, Event };
    runInNewContext(script, context);
    return context.createWorkspaceStorage(() => storage);
};
test("workspace storage requires opt-in and separates clear from delete", () => {
    const { values, storage } = memory();
    const saved = create(storage);
    assert.equal(saved.save({ files: [] }), false);
    assert.equal(values.size, 0);
    assert.equal(saved.restore(), null);
    assert.equal(saved.enable({ files: [{ path: "a.json", content: "{}" }], validations: { "a.json": { valid: true } } }), true);
    assert.equal(saved.status().enabled, true);
    assert.equal(create(storage).restore().files[0].path, "a.json");
    assert.equal(Object.keys(create(storage).restore().validations).length, 1);
    values.set("acorn.workspace.v1", JSON.stringify({ schema: 2, workspace: { files: [] } }));
    assert.equal(saved.restore(), null);
    saved.save({ files: [] });
    saved.save({ files: [] });
    assert.equal(saved.restore().files.length, 0);
    values.set("other", "keep");
    assert.equal(saved.delete(), true);
    assert.equal(saved.restore(), null);
    assert.equal(values.get("other"), "keep");
});
test("workspace storage tolerates corruption, disabled storage and quota errors", () => {
    const { values, storage } = memory();
    const saved = create(storage);
    saved.enable({ files: [] });
    values.set("acorn.workspace.v1", "{");
    assert.equal(saved.restore(), null);
    assert.equal(create({ getItem() { throw Error("disabled"); } }).restore(), null);
    const full = create({ getItem: () => null, setItem() { throw Error("quota"); }, removeItem() {} });
    assert.equal(full.enable({ files: [] }), false);
});
test("a failed automatic save removes the stale snapshot and reports failure", () => {
    const { values, storage } = memory();
    let quota = false;
    const limited = { ...storage, setItem(key, value) { if (quota && key === "acorn.workspace.v1") throw Error("quota"); storage.setItem(key, value); } };
    const saved = create(limited);
    assert.equal(saved.enable({ files: [{ path: "old.json", content: "{}" }] }), true);
    quota = true;
    assert.equal(saved.save({ files: [{ path: "new.json", content: "{}" }] }), false);
    assert.equal(saved.status().failed, true);
    assert.equal(saved.status().enabled, false);
    assert.equal(values.has("acorn.workspace.v1"), false);
    assert.equal(create(limited).restore(), null);
    quota = false;
    assert.equal(saved.enable({ files: [{ path: "new.json", content: "{}" }] }), true);
    assert.equal(saved.status().failed, false);
    assert.equal(saved.save({ files: [{ path: "oversized.json", content: "x".repeat(17 * 1024 * 1024) }] }), false);
    assert.equal(saved.status().enabled, false);
    assert.equal(create(limited).restore(), null);
});
