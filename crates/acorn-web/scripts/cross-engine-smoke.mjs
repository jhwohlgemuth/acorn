import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { extname, resolve, sep } from "node:path";
import { firefox, webkit } from "playwright";

const engine = process.argv[2];
const browserType = { firefox, webkit }[engine];
assert.ok(browserType, "Choose firefox or webkit");
const root = resolve(process.argv[3] ?? "target/dx/acorn-web/release/web/public");
const prefix = `/${(process.argv[4] ?? "").replace(/^\/+|\/+$/g, "")}`.replace(/\/$/, "");
const types = { ".css": "text/css", ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".svg": "image/svg+xml", ".wasm": "application/wasm" };
const server = createServer(async (request, response) => {
    const requested = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
    if (!requested.startsWith(`${prefix}/`)) { response.writeHead(404); response.end(); return; }
    const path = resolve(root, `.${requested.slice(prefix.length)}`);
    if (path !== root && !path.startsWith(`${root}${sep}`)) { response.writeHead(404); response.end(); return; }
    const file = await stat(path).then((info) => info.isDirectory() ? resolve(path, "index.html") : path).catch(() => path);
    const bytes = await readFile(file).catch(() => null);
    response.writeHead(bytes ? 200 : 404, { "Content-Type": types[extname(file)] ?? (bytes ? "application/octet-stream" : "text/html") });
    response.end(bytes ?? await readFile(resolve(root, "index.html")));
});
await new Promise((ready) => server.listen(0, "127.0.0.1", ready));
const origin = `http://127.0.0.1:${server.address().port}`;
const temporary = await mkdtemp(resolve(tmpdir(), `acorn-${engine}-effects-`));
const browser = await browserType.launch();
try {
    const context = await browser.newContext({ acceptDownloads: true });
    const page = await context.newPage();
    page.setDefaultTimeout(20000);
    const failures = [];
    page.on("pageerror", (error) => failures.push(error.message));
    page.on("request", (request) => { if (request.url().startsWith("http") && !request.url().startsWith(`${origin}${prefix}/`)) failures.push(`External request: ${request.url()}`); });
    await page.goto(`${origin}${prefix}/`);
    await page.getByRole("heading", { name: "Research Intelligence" }).waitFor();
    await page.waitForFunction(() => Boolean(window.acornWorkspace && window.acornDownload));
    // Playwright Firefox intermittently cancels native dialogs before setFiles; inject files into the same input.
    if (engine === "firefox") await page.evaluate(() => {
        const click = HTMLInputElement.prototype.click;
        HTMLInputElement.prototype.click = function () { if (this.type !== "file") click.call(this); };
    });
    await page.evaluate(() => { window.__fileEvents = []; for (const type of ["change", "cancel"]) document.addEventListener(type, (event) => { if (event.target.type === "file") window.__fileEvents.push({ type, names: Array.from(event.target.files).map(file => file.name) }); }, true); });
    const choose = async (kind, files) => {
        const before = await page.evaluate(() => window.__fileEvents.length);
        await page.getByRole("button", { name: "Open file or folder", exact: false }).click();
        if (engine === "firefox") {
            await page.getByRole("button", { name: kind, exact: true }).click();
            await page.locator('input[type="file"]').setInputFiles(files);
        } else {
            const selection = page.waitForEvent("filechooser").then((chooser) => chooser.setFiles(files));
            await Promise.all([page.getByRole("button", { name: kind, exact: true }).click(), selection]);
        }
        await page.waitForFunction((count) => window.__fileEvents.length > count, before);
        return page.evaluate((count) => window.__fileEvents[count], before);
    };
    const fileSelection = await choose("Files", resolve("crates/acorn-web/assets/examples/publications.json"));
    assert.equal(fileSelection.type, "change", `Automated ${engine} file selection was cancelled`);
    await page.locator(".sidebar-session").getByText("1 activity · 1 file", { exact: true }).waitFor();
    await page.locator('nav a[title="Review"]').click();
    await page.getByRole("heading", { name: "Review", exact: true }).waitFor();
    await page.getByRole("button", { name: "Use example", exact: true }).click();
    await page.getByRole("button", { name: "Validate", exact: true }).click();
    await page.getByRole("button", { name: "Copy result", exact: true }).waitFor();
    const nativeClipboard = await page.evaluate(() => Boolean(navigator.clipboard?.writeText));
    await page.evaluate(() => {
        window.__copied = null;
        Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText: async (value) => { window.__copied = value; } } });
    });
    await page.getByRole("button", { name: "Copy result", exact: true }).click();
    await page.waitForFunction(() => Boolean(window.__copied));
    assert.ok(JSON.parse(await page.evaluate(() => window.__copied)).diagnostics, "Copy emits the structured result");
    await page.evaluate(() => Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText: async () => { throw Error("denied"); } } }));
    await page.getByRole("button", { name: "Copy result", exact: true }).click();
    await page.locator(".notifications .notification-text").getByText("Clipboard unavailable. Select and copy the code manually.", { exact: true }).waitFor();
    await page.evaluate(() => {
        window.__download = { created: [], revoked: [] };
        const create = URL.createObjectURL.bind(URL);
        const revoke = URL.revokeObjectURL.bind(URL);
        URL.createObjectURL = (blob) => { window.__download.created.push(blob.type); return create(blob); };
        URL.revokeObjectURL = (url) => { window.__download.revoked.push(url); return revoke(url); };
    });
    const downloading = page.waitForEvent("download");
    await page.getByRole("button", { name: "Download result", exact: true }).click();
    const download = await downloading;
    assert.match(download.suggestedFilename(), /\.json$/);
    assert.ok(JSON.parse(await readFile(await download.path(), "utf8")).diagnostics);
    await page.waitForFunction(() => window.__download.revoked.length > 0);
    assert.ok((await page.evaluate(() => window.__download.created)).some((mime) => mime.startsWith("application/json")));
    await page.locator('nav a[title="Home"]').click();
    await page.getByRole("heading", { name: "Research Intelligence" }).waitFor();
    const supportsFolder = await page.evaluate(() => "webkitdirectory" in document.createElement("input"));
    let folderResult = "browser fallback";
    if (supportsFolder) {
        await mkdir(resolve(temporary, "folder/nested"), { recursive: true });
        await writeFile(resolve(temporary, "folder/nested/activity.json"), await readFile("crates/acorn-web/assets/examples/publications.json"));
        await writeFile(resolve(temporary, "folder/readme.txt"), "Cross-engine folder text");
        await writeFile(resolve(temporary, "folder/ignored.bin"), "unsupported");
        const selection = await choose("Folder", resolve(temporary, "folder"));
        if (selection.type === "change") {
            await page.getByText(/2 files loaded; 1 entries skipped/).waitFor();
            await page.locator(".loaded-tree > summary").click();
            await page.locator(".loaded-tree summary").filter({ hasText: /^\s*folder\s*$/ }).click();
            await page.getByRole("button", { name: "readme.txt", exact: true }).focus();
            await page.keyboard.press("Enter");
            const drawer = page.getByRole("dialog");
            await drawer.waitFor();
            await drawer.getByRole("tab", { name: "Source", exact: true }).click();
            assert.equal(await drawer.locator("#drawer-source").inputValue(), "Cross-engine folder text");
            await page.keyboard.press("Escape");
            await drawer.waitFor({ state: "hidden" });
            folderResult = "nested folder and keyboard tree";
        } else {
            folderResult = "directory picker cancelled by automation; manual selection unverified";
        }
    } else {
        await page.getByRole("button", { name: "Open file or folder", exact: false }).click();
        await page.getByRole("button", { name: "Folder", exact: true }).click();
        await page.locator(".notifications .notification-text").getByText(/Folder selection is unavailable/).waitFor();
        await choose("Files", resolve("crates/acorn-web/assets/examples/publications.json"));
        await page.locator(".sidebar-session").getByText("1 activity · 1 file", { exact: true }).waitFor();
    }
    assert.deepEqual(failures, []);
    const cancelPage = await context.newPage();
    await cancelPage.goto(`${origin}${prefix}/`);
    await cancelPage.getByRole("heading", { name: "Research Intelligence" }).waitFor();
    if (engine === "firefox") await cancelPage.evaluate(() => {
        const click = HTMLInputElement.prototype.click;
        HTMLInputElement.prototype.click = function () { if (this.type !== "file") click.call(this); };
    });
    const beforeCancel = await cancelPage.locator(".sidebar-session").innerText();
    await cancelPage.getByRole("button", { name: "Open file or folder", exact: false }).click();
    if (engine === "firefox") {
        await cancelPage.getByRole("button", { name: "Files", exact: true }).click();
        await cancelPage.locator('input[type="file"]').dispatchEvent("cancel");
    } else {
        const cancellation = cancelPage.waitForEvent("filechooser").then((chooser) => chooser.setFiles([]));
        await Promise.all([cancelPage.getByRole("button", { name: "Files", exact: true }).click(), cancellation]);
    }
    await cancelPage.waitForFunction(() => !document.querySelector('input[type="file"]'));
    assert.equal(await cancelPage.locator(".sidebar-session").innerText(), beforeCancel, "Cancelling the picker preserves the prior session");
    await cancelPage.close();
    console.log(`${engine} ${browser.version()}: ${engine === "firefox" ? "synthetic chooser cancellation and file selection" : "native chooser cancellation and file selection"}, ${folderResult}, copy feedback, JSON download, URL cleanup, and no external requests passed; native clipboard API ${nativeClipboard ? "present" : "unavailable"}.`);
} finally {
    await browser.close();
    await new Promise((closed) => server.close(closed));
    await rm(temporary, { recursive: true, force: true });
}
