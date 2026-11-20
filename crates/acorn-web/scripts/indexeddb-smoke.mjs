import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, resolve, sep } from "node:path";
import { chromium } from "playwright";

const root = resolve(process.argv[2] || "target/dx/acorn-web/release/web/public");
const types = { ".css": "text/css", ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".png": "image/png", ".svg": "image/svg+xml", ".wasm": "application/wasm" };
const server = createServer(async (request, response) => {
    const path = resolve(root, `.${new URL(request.url, "http://localhost").pathname}`);
    const safe = path === root || path.startsWith(root + sep);
    const file = safe ? await stat(path).then(info => info.isDirectory() ? resolve(path, "index.html") : path).catch(() => path) : "";
    const bytes = file ? await readFile(file).catch(() => null) : null;
    response.writeHead(bytes ? 200 : 404, { "Content-Type": bytes ? types[extname(file)] || "application/octet-stream" : "text/html" });
    response.end(bytes || await readFile(resolve(root, "index.html")));
});
await new Promise(ready => server.listen(0, "127.0.0.1", ready));
const browser = await chromium.launch();
try {
    const page = await browser.newPage();
    page.setDefaultTimeout(20000);
    await page.goto(`http://127.0.0.1:${server.address().port}/`);
    await page.getByRole("button", { name: "Try Sample Data" }).click();
    const summary = page.locator(".sidebar-session").getByText(/[1-9]\d* activities · [1-9]\d* files/);
    await summary.waitFor();
    const label = await summary.innerText();
    const fileCount = Number(label.match(/(\d+) files/)?.[1]);
    await page.waitForFunction(async count => (await window.acornWorkspaceStorage.restore())?.files?.length === count, fileCount);
    const saved = await page.evaluate(async () => ({ status: await window.acornWorkspaceStorage.status(), legacy: localStorage.getItem("acorn.workspace.v1") }));
    assert.ok(saved.status.bytes > 0);
    assert.equal(saved.legacy, null);
    await page.reload();
    await page.locator(".sidebar-session").getByText(label, { exact: true }).waitFor();
    await page.locator('nav a[title="Settings"]').click();
    await page.getByRole("button", { name: "Delete all data" }).click();
    await page.waitForFunction(async () => await window.acornWorkspaceStorage.restore() === null);
    console.log("Built app IndexedDB save, reload, and delete passed.");
} finally {
    await browser.close();
    server.close();
}
