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
test("a failed automatic save retains the previous snapshot and reports failure", () => {
    const { values, storage } = memory();
    let quota = false;
    const limited = { ...storage, setItem(key, value) { if (quota && key === "acorn.workspace.v1") throw Error("quota"); storage.setItem(key, value); } };
    const saved = create(limited);
    assert.equal(saved.save({ files: [{ path: "old.json", content: "{}" }] }), true);
    quota = true;
    assert.equal(saved.save({ files: [{ path: "new.json", content: "{}" }] }), false);
    assert.equal(saved.status().failed, true);
    assert.ok(saved.status().bytes > 0);
    assert.equal(create(limited).restore().files[0].path, "old.json");
    quota = false;
    assert.equal(saved.save({ files: [{ path: "new.json", content: "{}" }] }), true);
    assert.equal(saved.status().failed, false);
    assert.equal(saved.save({ files: [{ path: "expanded.json", content: "x".repeat(17 * 1024 * 1024) }] }), true);
    assert.equal(saved.save({ files: [{ path: "oversized.json", content: "x".repeat(33 * 1024 * 1024) }] }), false);
    assert.equal(create(limited).restore().files[0].path, "expanded.json");
});
test("loaded workspace saves automatically and survives reload until deleted", () => {
    const { values, storage } = memory();
    const saved = create(storage);
    assert.equal(saved.save({ files: [] }), true);
    assert.equal(values.size, 0);
    assert.equal(saved.restore(), null);
    const image = "data:image/png;base64,iVBORw0KGgo=";
    assert.equal(saved.save({ files: [{ path: "a.json", content: "{}" }, { path: "cover.png", content: image }], selected: "a.json", validations: { "a.json": { valid: true } } }), true);
    assert.ok(saved.status().bytes > 0);
    assert.equal(create(storage).restore().files[0].path, "a.json");
    assert.equal(create(storage).restore().selected, "a.json");
    assert.equal(create(storage).restore().files[1].content, image);
    assert.equal(Object.keys(create(storage).restore().validations).length, 1);
    assert.equal(saved.save({ files: [] }), true);
    assert.equal(create(storage).restore().files[0].path, "a.json");
    values.set("other", "keep");
    assert.equal(saved.delete(), true);
    assert.equal(saved.restore(), null);
    assert.equal(values.get("other"), "keep");
});
test("workspace storage tolerates corruption and unavailable storage", () => {
    const { values, storage } = memory();
    const saved = create(storage);
    values.set("acorn.workspace.v1", "{");
    assert.equal(saved.restore(), null);
    values.set("acorn.workspace.v1", JSON.stringify({ schema: 2, workspace: { files: [] } }));
    assert.equal(saved.restore(), null);
    assert.equal(create({ getItem() { throw Error("disabled"); } }).restore(), null);
    const full = create({ getItem: () => null, setItem() { throw Error("quota"); }, removeItem() {} });
    assert.equal(full.save({ files: [{ path: "a.json", content: "{}" }] }), false);
});
