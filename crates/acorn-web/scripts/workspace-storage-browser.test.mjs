import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { chromium } from "playwright";

const script = await readFile(new URL("../assets/workspace-storage.js", import.meta.url), "utf8");

test("IndexedDB migrates legacy data, stores large text and binary images, preserves failed saves, and deletes data", async () => {
    const browser = await chromium.launch();
    try {
        const page = await browser.newPage();
        await page.route("http://127.0.0.1:7899/**", route => route.fulfill({ body: "<!doctype html><title>storage test</title>", contentType: "text/html" }));
        await page.goto("http://127.0.0.1:7899/");
        await page.evaluate(() => localStorage.setItem("acorn.workspace.v1", JSON.stringify({ schema: 1, workspace: { files: [{ path: "old.json", content: "{}" }], validations: {}, selected: "old.json", skipped: 0 } })));
        await page.addScriptTag({ content: script });
        assert.deepEqual(await page.evaluate(async () => {
            const workspace = await window.acornWorkspaceStorage.restore();
            return [workspace.files[0].path, localStorage.getItem("acorn.workspace.v1")];
        }), ["old.json", null]);
        const image = "data:image/png;base64,iVBORw0KGgo=";
        const outcome = await page.evaluate(async (image) => {
            const storage = window.acornWorkspaceStorage;
            const big = "x".repeat(6 * 1024 * 1024);
            const workspace = { files: [{ path: "large.txt", content: big }, { path: "cover.png", content: image }], selected: "large.txt", validations: {}, skipped: 0 };
            const saved = await storage.save(workspace);
            const db = await new Promise((resolve, reject) => { const request = indexedDB.open("acorn.workspace", 2); request.onsuccess = () => resolve(request.result); request.onerror = () => reject(request.error); });
            const binary = await new Promise((resolve, reject) => { const request = db.transaction("files").objectStore("files").get("cover.png"); request.onsuccess = () => resolve(request.result.content instanceof Blob); request.onerror = () => reject(request.error); });
            db.close();
            const restored = await storage.restore();
            const status = await storage.status();
            return { saved, binary, length: restored.files.find(file => file.path === "large.txt").content.length, image: restored.files.find(file => file.path === "cover.png").content, bytes: status.bytes, quota: status.quota, local: localStorage.getItem("acorn.workspace.v1") };
        }, image);
        assert.equal(outcome.saved, true);
        assert.equal(outcome.binary, true);
        assert.equal(outcome.length, 6 * 1024 * 1024);
        assert.equal(outcome.image, image);
        assert.ok(outcome.bytes > 6 * 1024 * 1024);
        assert.ok(outcome.quota > 0);
        assert.equal(outcome.local, null);
        const unchangedFiles = await page.evaluate(async () => {
            const original = IDBObjectStore.prototype.put;
            let writes = 0;
            IDBObjectStore.prototype.put = function (value, ...args) { if (this.name === "files") writes += 1; return original.call(this, value, ...args); };
            const workspace = await window.acornWorkspaceStorage.restore();
            await window.acornWorkspaceStorage.save({ ...workspace, selected: "cover.png" });
            IDBObjectStore.prototype.put = original;
            return writes;
        });
        assert.equal(unchangedFiles, 0, "selection changes do not rewrite file records");
        const failure = await page.evaluate(async () => {
            const original = IDBObjectStore.prototype.put;
            IDBObjectStore.prototype.put = function (value, ...args) { if (value?.path === "new.json") throw new DOMException("Quota exceeded", "QuotaExceededError"); return original.call(this, value, ...args); };
            const saved = await window.acornWorkspaceStorage.save({ files: [{ path: "new.json", content: "{}" }], validations: {}, selected: "new.json", skipped: 0 });
            IDBObjectStore.prototype.put = original;
            const restored = await window.acornWorkspaceStorage.restore();
            return { saved, path: restored.files[0].path, failed: (await window.acornWorkspaceStorage.status()).failed };
        });
        assert.deepEqual(failure, { saved: false, path: "cover.png", failed: true });
        const failedDelete = await page.evaluate(async () => {
            const original = IDBObjectStore.prototype.clear;
            IDBObjectStore.prototype.clear = function (...args) { if (this.name === "files") throw new DOMException("Unavailable", "SecurityError"); return original.call(this, ...args); };
            const deleted = await window.acornWorkspaceStorage.delete();
            IDBObjectStore.prototype.clear = original;
            return { deleted, path: (await window.acornWorkspaceStorage.restore()).files[0].path };
        });
        assert.deepEqual(failedDelete, { deleted: false, path: "cover.png" });
        assert.equal(await page.evaluate(() => window.acornWorkspaceStorage.delete()), true);
        assert.equal(await page.evaluate(() => window.acornWorkspaceStorage.restore()), null);
    } finally { await browser.close(); }
});
test("a stale tab cannot recreate files after another tab deletes the workspace", async () => {
    const browser = await chromium.launch();
    try {
        const context = await browser.newContext();
        await context.route("http://127.0.0.1:7899/**", route => route.fulfill({ body: "<!doctype html><title>storage test</title>", contentType: "text/html" }));
        const first = await context.newPage();
        const second = await context.newPage();
        for (const page of [first, second]) {
            await page.goto("http://127.0.0.1:7899/");
            await page.addScriptTag({ content: script });
        }
        const workspace = { files: [{ path: "record.json", content: "{}" }], selected: "record.json", validations: {}, skipped: 0 };
        assert.equal(await first.evaluate(value => window.acornWorkspaceStorage.save(value), workspace), true);
        assert.equal((await second.evaluate(() => window.acornWorkspaceStorage.restore())).files.length, 1);
        assert.equal(await first.evaluate(() => window.acornWorkspaceStorage.delete()), true);
        assert.equal(await second.evaluate(value => window.acornWorkspaceStorage.save(value), { ...workspace, selected: null }), false);
        assert.equal((await second.evaluate(() => window.acornWorkspaceStorage.status())).failed, true);
        const fresh = await context.newPage();
        await fresh.goto("http://127.0.0.1:7899/");
        await fresh.addScriptTag({ content: script });
        assert.equal(await fresh.evaluate(() => window.acornWorkspaceStorage.restore()), null);
    } finally { await browser.close(); }
});
test("a local fallback remains authoritative after IndexedDB access recovers", async () => {
    const browser = await chromium.launch();
    try {
        const page = await browser.newPage();
        await page.route("http://127.0.0.1:7899/**", route => route.fulfill({ body: "<!doctype html><title>storage test</title>", contentType: "text/html" }));
        await page.goto("http://127.0.0.1:7899/");
        await page.evaluate(() => {
            window.originalOpen = IDBFactory.prototype.open;
            IDBFactory.prototype.open = () => { throw new DOMException("Blocked", "SecurityError"); };
        });
        await page.addScriptTag({ content: script });
        const workspace = { files: [{ path: "fallback.json", content: "{}" }], selected: "fallback.json", validations: {}, skipped: 0 };
        assert.equal(await page.evaluate(value => window.acornWorkspaceStorage.save(value), workspace), true);
        assert.ok(await page.evaluate(() => localStorage.getItem("acorn.workspace.fallback.v1")));
        await page.evaluate(() => { IDBFactory.prototype.open = window.originalOpen; });
        await page.addScriptTag({ content: script });
        assert.equal((await page.evaluate(() => window.acornWorkspaceStorage.restore())).files[0].path, "fallback.json");
        assert.equal(await page.evaluate(() => window.acornWorkspaceStorage.delete()), true);
        assert.equal(await page.evaluate(() => window.acornWorkspaceStorage.restore()), null);
    } finally { await browser.close(); }
});
