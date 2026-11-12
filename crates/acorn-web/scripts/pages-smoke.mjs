import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, resolve, sep } from "node:path";
import { chromium } from "playwright";

const root = resolve(process.argv[2]);
const prefix = `/${(process.argv[3] ?? "").replace(/^\/+|\/+$/g, "")}`.replace(/\/$/, "");
const types = { ".css": "text/css", ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".svg": "image/svg+xml", ".wasm": "application/wasm" };
assert.deepEqual(await readFile(`${root}/404.html`), await readFile(`${root}/index.html`), "Pages must serve the application on direct routes");
const server = createServer(async (request, response) => {
    try {
        const path = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
        assert.ok(path.startsWith(`${prefix}/`), `Request escaped deployment prefix: ${path}`);
        const file = resolve(root, `.${path.slice(prefix.length)}`);
        assert.ok(file === root || file.startsWith(`${root}${sep}`));
        const candidate = await stat(file).then((info) => info.isDirectory() ? `${file}/index.html` : file).catch(() => file);
        const bytes = await readFile(candidate).catch(() => null);
        response.writeHead(bytes ? 200 : 404, { "Content-Type": types[extname(candidate)] ?? (bytes ? "application/octet-stream" : "text/html") });
        response.end(bytes ?? await readFile(`${root}/404.html`));
    } catch {
        response.writeHead(404);
        response.end();
    }
});
await new Promise((ready) => server.listen(0, "127.0.0.1", ready));
const origin = `http://127.0.0.1:${server.address().port}`;
const failures = [];
const browser = await chromium.launch().catch((error) => { server.close(); throw error; });
try {
    const page = await browser.newPage();
    page.setDefaultTimeout(15000);
    page.on("pageerror", (error) => failures.push(error.message));
    page.on("requestfailed", (request) => failures.push(`${request.url()}: ${request.failure()?.errorText}`));
    page.on("response", (response) => {
        if (response.status() >= 400 && response.request().resourceType() !== "document") failures.push(`${response.status()} ${response.url()}`);
    });
    page.on("request", (request) => {
        if (request.url().startsWith("http") && !request.url().startsWith(`${origin}${prefix}/`)) failures.push(`Unexpected request: ${request.url()}`);
    });
    await page.goto(`${origin}${prefix}/`);
    await page.getByRole("heading", { name: "ACORN Browser" }).waitFor();
    await page.waitForFunction(() => { const image = document.querySelector(".home-banner img"); return image?.complete && image.naturalWidth > 0; });
    await page.waitForFunction(() => getComputedStyle(document.querySelector(".sidebar")).position !== "static");
    await page.getByRole("button", { name: "Try sample data" }).click();
    await page.getByRole("heading", { name: "Research Activity", exact: true }).waitFor();
    assert.equal(new URL(page.url()).pathname, `${prefix}/research-activity`);
    await page.getByRole("button", { name: "Run Research Activity", exact: true }).click();
    await page.getByRole("button", { name: "Copy JSON", exact: true }).waitFor();
    const direct = await page.goto(`${origin}${prefix}/readability`);
    assert.equal(direct.status(), 404, "Exercise GitHub Pages' custom 404 response");
    await page.getByRole("heading", { name: "Readability", exact: true }).waitFor();
    await page.reload();
    await page.getByRole("heading", { name: "Readability", exact: true }).waitFor();
    await page.locator('nav a[title="Home"]').click();
    await page.getByRole("heading", { name: "ACORN Browser" }).waitFor();
    assert.equal(new URL(page.url()).pathname, `${prefix}/`);
    assert.deepEqual(failures, []);
    console.log(`Pages browser smoke passed at ${prefix || "/"}: assets, WASM operation, navigation and direct-route refresh.`);
} finally {
    await browser.close();
    await new Promise((closed) => server.close(closed));
}
