import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { runInNewContext } from "node:vm";

const script = await readFile(new URL("dashboard-layout.js", import.meta.url), "utf8");
const create = (storage) => {
    const context = { window: {}, JSON, Object, Set };
    runInNewContext(script, context);
    return context.createDashboardLayoutStorage(() => storage);
};
const plain = (value) => JSON.parse(JSON.stringify(value));
const values = () => {
    const data = new Map();
    return { data, storage: { getItem: (key) => data.get(key) ?? null, setItem: (key, value) => data.set(key, value), removeItem: (key) => data.delete(key) } };
};

test("corrupt and unsupported saved layouts retain safe defaults", () => {
    const { data, storage } = values();
    const layout = create(storage);
    data.set("acorn.dashboard.layout.v1", "{");
    assert.equal(plain(layout.load("portfolio")).order[0], "activities");
    assert.equal(layout.save("portfolio", { order: ["identifiers"] }), true);
    data.set("acorn.dashboard.layout.v1", JSON.stringify({ schema: 2, pages: { portfolio: { order: ["identifiers"] } } }));
    assert.equal(plain(layout.load("portfolio")).order[0], "activities");
    assert.equal(layout.save("portfolio", { order: ["statuses"] }), false);
    assert.equal(JSON.parse(data.get("acorn.dashboard.layout.v1")).schema, 2);
    assert.equal(layout.reset("portfolio"), false);
    assert.equal(layout.clear(), true);
    assert.equal(data.size, 0);
});
test("failed saves leave the prior layout intact", () => {
    const { data, storage } = values();
    const layout = create(storage);
    assert.equal(layout.save("portfolio", { order: ["statuses"] }), true);
    const previous = data.get("acorn.dashboard.layout.v1");
    const quota = create({ ...storage, setItem() { throw Error("quota"); } });
    assert.equal(quota.save("portfolio", { order: ["identifiers"] }), false);
    assert.equal(data.get("acorn.dashboard.layout.v1"), previous);
});
test("page-scoped layouts restore normalized order and visibility", () => {
    const { data, storage } = values();
    const layout = create(storage);
    assert.deepEqual(plain(layout.load("portfolio")), { order: ["activities", "organizations", "contributors", "top-topics", "statuses", "identifiers"], hidden: [] });
    assert.equal(layout.save("portfolio", { order: ["statuses", "statuses", "unknown", "activities"], hidden: ["contributors", "contributors", "unknown"] }), true);
    assert.deepEqual(plain(create(storage).load("portfolio")), { order: ["statuses", "activities", "organizations", "contributors", "top-topics", "identifiers"], hidden: ["contributors"] });
    assert.equal(JSON.parse(data.get("acorn.dashboard.layout.v1")).schema, 1);
    assert.equal(layout.load("aspect"), null);
    assert.equal(layout.save("aspect", { order: [] }), false);
    assert.equal(layout.reset("portfolio"), true);
    assert.equal(data.size, 0);
});
test("free tile positions persist and invalid cells are ignored", () => {
    const { storage } = values();
    const layout = create(storage);
    assert.equal(layout.save("portfolio", { positions: {
        activities: { row: 1, column: 4 },
        statuses: { row: 2, column: 3 },
        identifiers: { row: 1, column: 4 },
        contributors: { row: 9, column: 1 },
        unknown: { row: 1, column: 1 },
    } }), true);
    assert.deepEqual(plain(create(storage).load("portfolio").positions), {
        activities: { row: 1, column: 4 },
        statuses: { row: 2, column: 3 },
    });
});
