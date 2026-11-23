import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, resolve, sep } from "node:path";
import { chromium } from "playwright";

const root = resolve(process.argv[2]);
const prefix = `/${(process.argv[3] ?? "").replace(/^\/+|\/+$/g, "")}`.replace(/\/$/, "");
const types = { ".css": "text/css", ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".svg": "image/svg+xml", ".wasm": "application/wasm" };
const server = createServer(async (request, response) => {
    try {
        const path = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
        assert.ok(path.startsWith(`${prefix}/`));
        const relative = path.slice(prefix.length);
        const file = resolve(root, `.${relative}`);
        assert.ok(file === root || file.startsWith(`${root}${sep}`));
        const candidate = relative === "/" ? `${root}/index.html` : file;
        const bytes = await readFile(candidate).catch(() => readFile(`${root}/index.html`));
        response.writeHead(200, { "Content-Type": types[extname(candidate)] ?? "text/html" });
        response.end(bytes);
    } catch {
        response.writeHead(404);
        response.end();
    }
});
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
const browser = await chromium.launch();
try {
    const page = await browser.newPage();
    page.setDefaultTimeout(15000);
    await page.goto(`http://127.0.0.1:${server.address().port}${prefix}/`);
    await page.getByRole("button", { name: "Try Sample Data" }).click();
    await page.getByRole("heading", { name: "Portfolio", exact: true }).waitFor();
    await page.locator(".dashboard-settings > summary").click();
    const popupPromise = page.context().waitForEvent("page");
    await page.getByRole("link", { name: "Open dashboard in new tab" }).click();
    const dashboard = await popupPromise;
    await dashboard.getByRole("heading", { name: "Dashboard", exact: true }).waitFor();
    await dashboard.waitForFunction(() => Number(document.querySelector('.metric[data-tile-id="activities"] strong')?.textContent) > 0);
    await page.locator('nav a[title="Settings"]').click();
    await page.getByRole("button", { name: "Delete all data" }).click();
    await dashboard.waitForFunction(() => document.querySelector('.metric[data-tile-id="activities"] strong')?.textContent === "0");
    await page.locator('nav a[title="Home"]').click();
    await page.getByRole("button", { name: "Try Sample Data" }).click();
    await dashboard.waitForFunction(() => Number(document.querySelector('.metric[data-tile-id="activities"] strong')?.textContent) > 0);
    await dashboard.close();
    await page.locator('nav a[title="Review"]').click();
    await page.locator(".code-editor:has(#content).cm-ready").waitFor();
    const source = await page.locator("#content").inputValue();
    const editorPromise = page.context().waitForEvent("page");
    await page.locator(".code-editor:has(#content)").getByRole("button", { name: "Open code editor in new tab" }).click();
    const editor = await editorPromise;
    await editor.locator("#popout-content-editor").waitFor();
    assert.equal(await editor.locator("#popout-content").inputValue(), source);
    await editor.locator("#popout-content-editor").press("ControlOrMeta+End");
    await editor.keyboard.insertText(" ");
    await page.waitForFunction(text => document.getElementById("content")?.value === `${text} `, source);
    await editor.close();
    console.log("Dashboard and code editor tabs synchronized.");
} finally {
    await browser.close();
    await new Promise(resolve => server.close(resolve));
}
