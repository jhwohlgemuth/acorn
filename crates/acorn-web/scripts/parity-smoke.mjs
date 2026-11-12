import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { extname, resolve, sep } from "node:path";
import { chromium } from "playwright";

const root = resolve(process.argv[2] || "target/dx/acorn-web/release/web/public");
const output = resolve(process.argv[3] || "target/abw-parity");
const prefix = (process.argv[4] || "").replace(/\/$/, "");
await mkdir(output, { recursive: true });
const types = { ".css": "text/css", ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".svg": "image/svg+xml", ".wasm": "application/wasm", ".png": "image/png", ".jpg": "image/jpeg", ".xml": "application/xml" };
const server = createServer(async (request, response) => {
    try {
        const requested = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
        assert.ok(requested.startsWith(prefix + "/"));
        const path = resolve(root, "." + requested.slice(prefix.length));
        assert.ok(path === root || path.startsWith(root + sep));
        const file = await stat(path).then((info) => info.isDirectory() ? resolve(path, "index.html") : path).catch(() => path);
        const bytes = await readFile(file).catch(() => null);
        response.writeHead(bytes ? 200 : 404, { "Content-Type": bytes ? types[extname(file)] || "application/octet-stream" : "text/html" });
        response.end(bytes || await readFile(resolve(root, "index.html")));
    } catch { response.writeHead(404); response.end(); }
});
await new Promise((ready) => server.listen(0, "127.0.0.1", ready));
const origin = `http://127.0.0.1:${server.address().port}`;
const browser = await chromium.launch({ args: ["--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader"] }).catch((error) => { server.close(); throw error; });
const failures = [];
const checks = [];
const temporary = await mkdtemp(resolve(tmpdir(), "acorn-parity-"));
try {
    const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, colorScheme: "dark", reducedMotion: "reduce", acceptDownloads: true });
    const page = await context.newPage();
    page.setDefaultTimeout(20000);
    page.on("pageerror", (error) => failures.push(error.message));
    page.on("request", (request) => { if (/^https?:/.test(request.url()) && !request.url().startsWith(origin + "/")) failures.push(`External request: ${request.url()}`); });
    page.on("response", (response) => { if (response.status() >= 400 && response.request().resourceType() !== "document") failures.push(`${response.status()} ${response.url()}`); });
    const tiles = new Set();
    page.on("response", (response) => { if (response.ok() && /NaturalEarthII\/.+\.jpg/.test(response.url())) tiles.add(response.url()); });
    const nav = async (name) => {
        if (await page.locator("#nav-toggle").isVisible() && await page.locator("#nav-toggle").getAttribute("aria-expanded") !== "true") await page.locator("#nav-toggle").click();
        await page.locator(`nav a[title="${name}"]`).click();
    };
    const shot = async (name) => page.screenshot({ path: resolve(output, `${name}.png`), fullPage: true });
    const noOverflow = async (name) => assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), `Horizontal overflow: ${name}`);
    await page.goto(origin + prefix + "/");
    await page.getByRole("heading", { name: "ACORN Browser" }).waitFor();
    await page.waitForFunction(() => Boolean(window.acornChrome));
    await shot("home-dark");
    await page.getByRole("button", { name: "Take a tour" }).click();
    await page.locator(".driver-popover").waitFor();
    await page.keyboard.press("Escape");
    await page.locator(".driver-popover").waitFor({ state: "hidden" });
    assert.equal(await page.locator("#tour-start").evaluate((node) => document.activeElement === node), true);
    checks.push("Reduced-motion tour closes and restores focus");
    await page.getByRole("button", { name: "Try sample data" }).click();
    await page.getByRole("heading", { name: "Portfolio", exact: true }).waitFor();
    assert.equal(await page.locator(".activity-table tbody tr").count(), 3);
    await shot("portfolio-dark");
    await page.getByRole("button", { name: "Open selected activity", exact: true }).click();
    const tools = ["Research Activity", "Metadata standard", "Crosswalk", "FAIR assessment", "Readability", "Content diff", "Big-O comparison", "Discover identifiers", "Inspect identifier"];
    for (const name of tools) {
        await nav(name);
        await page.getByRole("button", { name: "Use example", exact: true }).click();
        await page.getByRole("button", { name: `Run ${name}`, exact: true }).click();
        await page.getByRole("button", { name: "Copy result", exact: true }).waitFor();
        await page.waitForFunction(() => document.getElementById("result-heading") === document.activeElement);
        await noOverflow(name);
        await shot(name.toLowerCase().replaceAll(" ", "-"));
        checks.push(`${name}: example operation, structured result and focus`);
    }
    await nav("Research Activity");
    await page.locator("#content").fill('{"title":1}');
    await page.getByRole("button", { name: "Run Research Activity", exact: true }).click();
    await page.getByText(/Review needed ·/).waitFor();
    assert.ok(await page.locator(".diagnostics li").count() > 0);
    assert.equal(await page.locator("#content").inputValue(), '{"title":1}');
    await shot("validation-invalid");
    await nav("Content diff");
    await page.locator("#content").fill("before\n"); await page.locator("#after").fill("<script>literal</script>\n");
    await page.getByRole("button", { name: "Run Content diff", exact: true }).click();
    const downloaded = page.waitForEvent("download");
    await page.getByRole("button", { name: "Download result", exact: true }).click();
    const diff = await downloaded;
    assert.equal(diff.suggestedFilename(), "acorn-changes.diff");
    assert.match(await readFile(await diff.path(), "utf8"), /\+<script>literal<\/script>/);
    checks.push("Invalid metadata preserves input; unified export preserves literal bytes");
    await nav("Settings");
    await page.locator('[data-theme-choice="light"]').click();
    assert.equal(await page.locator(".workspace").getAttribute("data-theme"), "light");
    await page.emulateMedia({ colorScheme: "dark" });
    assert.equal(await page.locator(".workspace").getAttribute("data-theme"), "light");
    await page.locator('[data-theme-choice="system"]').click();
    assert.equal(await page.locator(".workspace").getAttribute("data-theme"), "dark");
    await page.emulateMedia({ colorScheme: "light" });
    await page.waitForFunction(() => document.querySelector(".workspace").dataset.theme === "light");
    await page.getByText(/App and bundled skill match/).waitFor();
    await shot("settings-light");
    checks.push("Explicit and system themes, live preference changes, session metrics and matching skill");
    await nav("ACORN Skill");
    await page.getByRole("button", { name: "Download skill folder", exact: true }).waitFor();
    const archiveEvent = page.waitForEvent("download");
    await page.getByRole("button", { name: "Download skill folder", exact: true }).click();
    const archive = await archiveEvent;
    const archiveHash = createHash("sha256").update(await readFile(await archive.path())).digest("hex");
    assert.ok(await page.locator("main").textContent().then((text) => text.includes(archiveHash)));
    await shot("skill-light");
    checks.push("Skill archive bytes match the displayed manifest digest");
    await nav("Install"); await shot("install-light");
    assert.equal(await page.locator(".channel-row").count(), 5);
    await nav("Geospatial");
    await page.getByText(/ROR 2.15/).waitFor({ timeout: 60000 });
    await page.locator(".organization-table tbody tr").first().waitFor();
    await page.locator("#geospatial-map canvas").waitFor({ timeout: 60000 });
    await page.waitForFunction(() => performance.getEntriesByType("resource").some((entry) => /NaturalEarthII\/.+\.jpg/.test(entry.name)));
    await page.locator(".organization-table tbody button").first().click();
    await page.waitForTimeout(2000);
    assert.ok(tiles.size > 0, "Local Natural Earth II tiles loaded");
    await shot("geospatial-light");
    checks.push("Pinned local ROR table and WebGL globe render");
    await nav("Home");
    await page.getByRole("button", { name: "Open file or folder", exact: false }).click();
    const pickerEvent = page.waitForEvent("filechooser");
    await page.getByRole("button", { name: "Files", exact: true }).click();
    const picker = await pickerEvent;
    await picker.setFiles(resolve("crates/acorn-web/assets/examples/publications.json"));
    await page.getByText(/1 files loaded/).waitFor();
    await nav("Portfolio");
    await page.getByRole("button", { name: "Validate loaded activities", exact: true }).click();
    await page.locator(".activity-table tbody tr").first().waitFor();
    assert.equal(await page.locator(".activity-table tbody tr").count(), 1);
    checks.push("User-mediated file picker, bounded reader and explicit Portfolio validation");
    await nav("Home");
    await mkdir(resolve(temporary, "folder/nested"), { recursive: true });
    await writeFile(resolve(temporary, "folder/nested/activity.json"), await readFile("crates/acorn-web/assets/examples/publications.json"));
    await writeFile(resolve(temporary, "folder/readme.txt"), "Local folder text");
    await writeFile(resolve(temporary, "folder/ignored.bin"), "unsupported");
    const choose = async (kind, files) => {
        await page.getByRole("button", { name: "Open file or folder", exact: false }).click();
        const event = page.waitForEvent("filechooser");
        await page.getByRole("button", { name: kind, exact: true }).click();
        await (await event).setFiles(files);
    };
    await choose("Folder", resolve(temporary, "folder"));
    await page.getByText(/2 files loaded; 1 entries skipped/).waitFor();
    await page.locator(".loaded-tree > summary").click();
    await page.locator(".loaded-tree summary").filter({ hasText: /^folder$/ }).click();
    await page.getByRole("button", { name: "readme.txt", exact: true }).click();
    await nav("Research Activity");
    assert.equal(await page.locator("#content").inputValue(), "Local folder text");
    await nav("Home");
    await choose("Files", [
        { name: "large.txt", mimeType: "text/plain", buffer: Buffer.alloc(262145) },
        { name: "invalid.txt", mimeType: "text/plain", buffer: Buffer.from([0xff]) },
    ]);
    await page.getByText(/No files loaded; 2 entries skipped/).waitFor();
    await page.locator(".loaded-tree > summary").getByText("2 loaded files").waitFor();
    await choose("Files", []);
    assert.ok(await page.locator(".loaded-tree > summary").textContent().then((text) => text.includes("2 loaded files")));
    await page.locator("[data-workspace-drop]").evaluate((node) => {
        const transfer = new DataTransfer();
        transfer.items.add(new File(["Dropped local text"], "dropped.txt", { type: "text/plain" }));
        node.dispatchEvent(new DragEvent("drop", { bubbles: true, cancelable: true, dataTransfer: transfer }));
    });
    await page.getByText(/1 files loaded; 0 entries skipped/).waitFor();
    checks.push("Nested folder picker, skipped extension, leaf selection, rejected UTF-8/size, cancellation and file drop");
    await nav("Home");
    await page.getByRole("button", { name: "Take a tour" }).click();
    await page.locator(".driver-popover").waitFor();
    await page.keyboard.press("Escape");
    await page.locator(".driver-popover").waitFor({ state: "hidden" });
    await page.evaluate(() => { document.documentElement.style.zoom = "2"; });
    for (const name of ["Home", "Portfolio", "Settings", "About"]) { await nav(name); await noOverflow(`${name} 200%`); }
    await shot("about-200-percent");
    await page.evaluate(() => { document.documentElement.style.zoom = ""; });
    checks.push("Light tour and 200% CSS zoom reflow");
    for (const viewport of [{ width: 390, height: 844 }, { width: 844, height: 390 }]) {
        await page.setViewportSize(viewport);
        for (const name of ["Home", "Portfolio", "Readability", "Geospatial", "ACORN Skill", "Install", "Settings", "About"]) {
            await nav(name); await noOverflow(`${name} ${viewport.width}x${viewport.height}`);
        }
        await shot(`about-${viewport.width}`);
        await page.locator("#nav-toggle").click();
        assert.equal(await page.locator("main").evaluate((node) => node.inert), true);
        await page.setViewportSize({ width: 1440, height: 900 });
        await page.waitForFunction(() => !document.querySelector("main").inert);
        assert.equal(await page.locator("#nav-toggle").getAttribute("aria-expanded"), "false");
    }
    checks.push("Portrait and landscape routes avoid horizontal overflow; desktop resize clears drawer inertness");
    await nav("Settings");
    await page.getByRole("button", { name: "Clear loaded session", exact: true }).click();
    await nav("Portfolio");
    await page.getByRole("heading", { name: "No research activities loaded" }).waitFor();
    checks.push("Session clearing removes loaded files and validated records");
    const fallback = await context.newPage();
    await fallback.addInitScript(() => {
        const original = HTMLCanvasElement.prototype.getContext;
        HTMLCanvasElement.prototype.getContext = function (kind, ...args) { return /webgl/.test(kind) ? null : original.call(this, kind, ...args); };
    });
    await fallback.goto(origin + prefix + "/");
    await fallback.getByRole("button", { name: "Try sample data" }).click();
    await fallback.locator('nav a[title="Geospatial"]').click();
    await fallback.getByText(/map is unavailable/i).waitFor({ timeout: 60000 });
    await fallback.locator(".organization-table tbody button").first().click();
    await fallback.screenshot({ path: resolve(output, "geospatial-no-webgl.png"), fullPage: true });
    checks.push("Forced WebGL failure retains the selectable organization table");
    assert.deepEqual(failures, []);
    await writeFile(resolve(output, "checks.json"), JSON.stringify({ browser: browser.version(), checks, failures }, null, 2) + "\n");
    console.log(`Browser parity smoke passed: ${checks.length} checks. Screenshots: ${output}`);
} catch (error) {
    await writeFile(resolve(output, "failure.json"), JSON.stringify({ error: String(error), checks, failures }, null, 2));
    for (const page of browser.contexts().flatMap((context) => context.pages())) await page.screenshot({ path: resolve(output, "failure.png"), fullPage: true }).catch(() => {});
    throw error;
} finally {
    await browser.close(); await new Promise((closed) => server.close(closed)); await rm(temporary, { recursive: true, force: true });
}
