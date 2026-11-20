import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { runInNewContext } from "node:vm";
import test from "node:test";

const script = await readFile(new URL("workspace-storage.js", import.meta.url), "utf8");
const memory = () => {
    const values = new Map();
    return { values, storage: { getItem: key => values.get(key) ?? null, setItem: (key, value) => values.set(key, value), removeItem: key => values.delete(key) } };
};
const create = (storage, indexedDB = () => undefined) => {
    const context = { window: {}, JSON, Event, Promise, Map };
    runInNewContext(script, context);
    return context.createWorkspaceStorage(() => storage, indexedDB, () => undefined);
};
test("a blocked IndexedDB open saves to the local fallback", async () => {
    const { values, storage } = memory();
    const blocked = () => ({ open() { throw Error("blocked"); } });
    values.set("acorn.workspace.v1", JSON.stringify({ schema: 1, workspace: { files: [{ path: "old.json", content: "{}" }] } }));
    const saved = create(storage, blocked);
    assert.equal((await saved.restore()).files[0].path, "old.json");
    assert.equal(await saved.save({ files: [{ path: "new.json", content: "{}" }] }), true);
    assert.equal(values.has("acorn.workspace.v1"), false);
    assert.equal((await create(storage, blocked).restore()).files[0].path, "new.json");
    assert.equal((await saved.status()).failed, false);
    assert.equal(await saved.delete(), false, "IndexedDB deletion cannot be verified while blocked");
    assert.equal(values.size, 0, "local fallback still clears");
});
test("localStorage fallback retains the previous copy on quota failure", async () => {
    const { values, storage } = memory();
    let quota = false;
    const limited = { ...storage, setItem(key, value) { if (quota) throw Error("quota"); storage.setItem(key, value); } };
    const saved = create(limited);
    assert.equal(await saved.save({ files: [{ path: "old.json", content: "{}" }] }), true);
    quota = true;
    assert.equal(await saved.save({ files: [{ path: "new.json", content: "{}" }] }), false);
    assert.equal((await saved.status()).failed, true);
    assert.equal((await create(limited).restore()).files[0].path, "old.json");
    quota = false;
    assert.equal(await saved.save({ files: [{ path: "new.json", content: "{}" }] }), true);
    assert.equal((await create(limited).restore()).files[0].path, "new.json");
    assert.equal(await saved.save({ files: [{ path: "expanded.json", content: "x".repeat(17 * 1024 * 1024) }] }), true);
    assert.equal(await saved.save({ files: [{ path: "oversized.json", content: "x".repeat(33 * 1024 * 1024) }] }), false);
    assert.equal((await create(limited).restore()).files[0].path, "expanded.json");
    assert.equal(await saved.delete(), true);
    assert.equal(values.size, 0);
});
test("fallback ignores corrupt snapshots and unavailable storage", async () => {
    const { values, storage } = memory();
    const saved = create(storage);
    values.set("acorn.workspace.v1", "{");
    assert.equal(await saved.restore(), null);
    values.set("acorn.workspace.v1", JSON.stringify({ schema: 2, workspace: { files: [] } }));
    assert.equal(await saved.restore(), null);
    values.set("acorn.workspace.fallback.v1", "{");
    assert.equal(await saved.restore(), null);
    const unavailable = create({ getItem() { throw Error("disabled"); }, setItem() { throw Error("disabled"); }, removeItem() { throw Error("disabled"); } });
    assert.equal(await unavailable.restore(), null);
    assert.equal(await unavailable.save({ files: [{ path: "a.json", content: "{}" }] }), false);
    assert.equal(await unavailable.delete(), false);
});
