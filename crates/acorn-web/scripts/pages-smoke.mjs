import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, resolve, sep } from "node:path";
import { setTimeout } from "node:timers/promises";
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
        // Expose navigation races with the asynchronously imported WebMCP adapter.
        if (/\/webmcp-[^/]+\.js$/.test(candidate)) await setTimeout(750);
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
async function waitForServices(page) {
    // A visible heading does not mean deferred module imports have finished.
    await page.waitForFunction(() => {
        const status = document.getElementById("webmcp-status")?.textContent;
        const editors = !document.querySelector(".code-editor") || document.querySelector(".cm-content");
        return window.acornSyntax && editors && status && status !== "WebMCP: initializing";
    });
    assert.match(await page.locator("#webmcp-status").textContent(), /^WebMCP: (installed|unsupported)$/);
}
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
    await waitForServices(page);
    await page.getByRole("heading", { name: "RAD Workspace" }).waitFor();
    await page.waitForFunction(() => { const image = document.querySelector(".home-banner img"); return image?.complete && image.naturalWidth > 0; });
    await page.waitForFunction(() => getComputedStyle(document.querySelector(".sidebar")).position !== "static");
    await page.getByRole("button", { name: "Try Sample Data" }).click();
    await page.getByRole("heading", { name: "Portfolio", exact: true }).waitFor();
    await page.locator(".activity-select").first().click();
    const drawer = page.getByRole("dialog");
    await drawer.waitFor();
    await drawer.getByRole("button", { name: "Open in Research Activity", exact: true }).click();
    await drawer.waitFor({ state: "hidden" });
    await page.getByRole("heading", { name: "Research Activity", exact: true }).waitFor();
    await waitForServices(page);
    assert.equal(new URL(page.url()).pathname, `${prefix}/research-activity`);
    const editorLicense = await page.locator('link[title="CodeMirror dependency licenses"]').first().getAttribute("href");
    assert.ok(editorLicense, "The bundled editor exposes its dependency licenses");
    const licenseResponse = await page.request.get(new URL(editorLicense, page.url()).href);
    assert.equal(licenseResponse.status(), 200, "Editor licenses load under the deployment prefix");
    assert.match(await licenseResponse.text(), /@codemirror\//);
    await page.getByRole("button", { name: "Validate", exact: true }).click();
    await page.getByRole("button", { name: "Copy result", exact: true }).waitFor();
    const discovery = await readFile(new URL("../assets/examples/discovery.txt", import.meta.url), "utf8");
    await page.locator('nav a[title="Discover identifiers"]').click();
    await waitForServices(page);
    await page.locator("#content-editor").fill(discovery.repeat(Math.floor(16384 / Buffer.byteLength(discovery))));
    await page.evaluate(() => {
        // Record main-thread frame gaps while the operation runs in the worker.
        const gaps = [];
        let last = performance.now();
        window.__acornFrames = { gaps, active: true };
        const tick = (now) => { gaps.push(now - last); last = now; if (window.__acornFrames.active) requestAnimationFrame(tick); };
        requestAnimationFrame(tick);
    });
    const started = Date.now();
    await page.getByRole("button", { name: "Discover identifiers", exact: true }).click();
    await page.getByText(/unique identifiers · normalized and deduplicated/).waitFor();
    const elapsed = Date.now() - started;
    const frames = await page.evaluate(() => { window.__acornFrames.active = false; return { longest: Math.max(...window.__acornFrames.gaps), served: window.acornOperations.status.served }; });
    assert.ok(frames.served >= 2, `Operations must run in the worker; it served ${frames.served}`);
    assert.ok(frames.longest < 250, `The main thread stalled for ${Math.round(frames.longest)} ms during discovery`);
    // Regex recompilation once made a full editor take seconds; keep generous headroom for CI hosts.
    assert.ok(elapsed < 3000, `Discovery at the editor limit took ${elapsed} ms`);
    console.log(`Discovery at the 16 KiB editor limit rendered in ${elapsed} ms in the worker; longest main-thread frame gap ${Math.round(frames.longest)} ms; ${frames.served} worker operations.`);
    const direct = await page.goto(`${origin}${prefix}/readability`);
    assert.equal(direct.status(), 404, "Exercise GitHub Pages' custom 404 response");
    await waitForServices(page);
    await page.getByRole("heading", { name: "Readability", exact: true }).waitFor();
    await page.reload();
    await waitForServices(page);
    await page.getByRole("heading", { name: "Readability", exact: true }).waitFor();
    await page.locator('nav a[title="Home"]').click();
    await page.getByRole("heading", { name: "RAD Workspace" }).waitFor();
    assert.equal(new URL(page.url()).pathname, `${prefix}/`);
    assert.deepEqual(failures, []);
    console.log(`Pages browser smoke passed at ${prefix || "/"}: assets, WASM operation, navigation and direct-route refresh.`);
} finally {
    await browser.close();
    await new Promise((closed) => server.close(closed));
}
