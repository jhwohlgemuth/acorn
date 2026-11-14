import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { extname, resolve, sep } from "node:path";
import { chromium } from "playwright";

const actionLabels = {
    "Research Activity": "Validate",
    "Metadata standard": "Validate",
    "Crosswalk": "Convert",
    "FAIR assessment": "Assess FAIR",
    "Readability": "Analyze readability",
    "Content diff": "Compare",
    "Big-O comparison": "Compare",
    "Discover identifiers": "Discover identifiers",
    "Inspect identifier": "Inspect identifier"
};

const root = resolve(process.argv[2] || "target/dx/acorn-web/release/web/public");
const validActivity = await readFile(new URL("../assets/examples/publications.json", import.meta.url), "utf8");
const fairRules = JSON.parse(await readFile(new URL("../../acorn-analysis/assets/fair.json", import.meta.url), "utf8"));
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
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);
    page.setDefaultTimeout(20000);
    if (process.env.ACORN_REVIEW_CSS) await page.route(/\/app[^/]*\.css$/, async (route) => route.fulfill({ contentType: "text/css", body: await readFile(process.env.ACORN_REVIEW_CSS, "utf8") }));
    page.on("pageerror", (error) => failures.push(error.message));
    const mockedRemote = (url) => url.startsWith("https://api.github.com/repos/acorn-test/bucket") || url.startsWith("https://gitlab.com/api/v4/projects/acorn-test%2Fbucket");
    const mockedFile = (url) => url.startsWith("https://files.acorn.test/") || url.startsWith("https://raw.githubusercontent.com/acorn-test/files/");
    let fileContent = "";
    let releaseFile;
    await page.route(url => mockedFile(url.href), async route => {
        const url = route.request().url();
        assert.equal(route.request().headers().cookie, undefined);
        assert.equal(route.request().headers().referer, undefined);
        if (url.endsWith("/blocked.txt")) return route.abort();
        if (url.endsWith("/slow.txt")) await new Promise(resolve => { releaseFile = resolve; });
        await route.fulfill({ status: url.endsWith("/missing.txt") ? 404 : 200, contentType: "text/plain", headers: { "Access-Control-Allow-Origin": "*" }, body: url.endsWith("/large.txt") ? "x".repeat(262145) : fileContent }).catch(() => {});
    });
    page.on("request", (request) => { if (/^https?:/.test(request.url()) && !request.url().startsWith(origin + "/") && !mockedRemote(request.url()) && !mockedFile(request.url())) failures.push(`External request: ${request.url()}`); });
    page.on("response", (response) => { if (response.status() >= 400 && response.request().resourceType() !== "document" && !mockedFile(response.url())) failures.push(`${response.status()} ${response.url()}`); });
    const tiles = new Set();
    page.on("response", (response) => { if (response.ok() && /BlackMarble\/.+\.png/.test(response.url())) tiles.add(response.url()); });
    const nav = async (name) => {
        if (await page.locator("#nav-toggle").isVisible() && await page.locator("#nav-toggle").getAttribute("aria-expanded") !== "true") await page.locator("#nav-toggle").click();
        await page.locator(`nav a[title="${name}"]`).click();
    };
    const shot = async (name) => page.screenshot({ path: resolve(output, `${name}.png`), fullPage: true });
    const noOverflow = async (name) => {
        assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), `Horizontal overflow: ${name}`);
        assert.deepEqual(await page.locator("main h1,main h2,main h3").allTextContents().then((headings) => headings.filter((heading) => /^\s*\d+[.)\s]|^\s*\d+(Input|Results)/.test(heading))), [], `Numbered section on ${name}`);
    };
    const openFileUrl = async () => {
        await page.getByRole("button", { name: "Load content", exact: false }).click();
        await page.getByRole("button", { name: "From URL", exact: false }).click();
        await page.waitForFunction(() => document.activeElement === document.querySelector(".url-file-loader input"));
    };
    const copyCodes = async () => {
        for (const button of await page.locator("main .copy-code:visible").all()) {
            const expected = await button.evaluate((node) => node.closest(".code-block")?.querySelector("code")?.textContent ?? (node.textContent.trim() === "Copy code" && node.closest(".code-editor") ? node.closest(".code-editor").querySelector("textarea").value : node.previousElementSibling?.matches("code") ? node.previousElementSibling.textContent : null));
            if (expected === null) continue;
            await page.evaluate(() => navigator.clipboard.writeText("clipboard sentinel"));
            await button.click();
            await page.waitForFunction(async (text) => await navigator.clipboard.readText() === text, expected);
        }
    };
    await page.goto(origin + prefix + "/");
    await page.getByRole("heading", { name: "RAD Workspace" }).waitFor();
    await page.waitForFunction(() => Boolean(window.acornChrome));
    const favicon = page.locator('link[rel="icon"][href*="acorn-app-icon"]');
    await favicon.waitFor({ state: "attached" });
    assert.ok((await page.request.get(new URL(await favicon.getAttribute("href"), page.url()).href)).ok(), "Brown ACORN SVG favicon loads");
    await page.locator(".sidebar-session").getByText("0 activities · 0 files", { exact: true }).waitFor();
    await shot("home-dark");
    assert.ok(await page.locator("#sample-data").evaluate((node) => { const sample = node.getBoundingClientRect(); const tour = document.getElementById("tour-start").getBoundingClientRect(); return Math.abs(sample.top - tour.top) < 2 && sample.left > tour.right; }), "Sample action sits right of tour");
    assert.equal(await page.locator(".sidebar img").count(), 0);
    assert.equal(await page.locator(".nav-item.active").evaluate((node) => getComputedStyle(node).boxShadow), "none");
    for (const viewport of [{ width: 1440, height: 900 }, { width: 1280, height: 720 }, { width: 1024, height: 768 }, { width: 390, height: 844 }, { width: 390, height: 667 }, { width: 844, height: 390 }]) {
        await page.setViewportSize(viewport);
        await noOverflow(`Empty Home ${viewport.width}×${viewport.height}`);
        const height = await page.evaluate(() => document.documentElement.scrollHeight);
        assert.ok(height <= viewport.height + 1, `Empty Home scrolls at ${viewport.width}×${viewport.height}: ${height}`);
        assert.ok(await page.locator(".credit-line").evaluate((node) => Math.abs(node.getBoundingClientRect().bottom - innerHeight) < 1), "Credit stays at bottom edge");
        await shot(`home-${viewport.width}-${viewport.height}`);
    }
    await page.setViewportSize({ width: 1440, height: 900 });
    checks.push("Empty Home fits six viewports, with adjacent primary actions and bottom credit");
    await page.goto(origin + prefix + "/home");
    await page.getByRole("heading", { name: "RAD Workspace", exact: true }).waitFor();
    const closedIcon = await page.locator(".collapse svg").innerHTML();
    await page.getByRole("button", { name: "Collapse sidebar", exact: true }).click();
    assert.notEqual(await page.locator(".collapse svg").innerHTML(), closedIcon);
    await shot("home-collapsed");
    await page.getByRole("button", { name: "Expand sidebar", exact: true }).click();
    checks.push("Home deep link, reference action order and sidebar toggle labels");
    await page.getByRole("button", { name: "Take a tour" }).click();
    await page.locator(".driver-popover").waitFor();
    await page.keyboard.press("Escape");
    await page.locator(".driver-popover").waitFor({ state: "hidden" });
    assert.equal(await page.locator("#tour-start").evaluate((node) => document.activeElement === node), true);
    checks.push("Reduced-motion tour closes and restores focus");
    await page.getByRole("button", { name: "Try Sample Data" }).click();
    await page.getByRole("heading", { name: "Portfolio", exact: true }).waitFor();
    assert.equal(await page.locator(".activity-table tbody tr").count(), 3);
    await page.locator(".sidebar-session").getByText("3 activities · 3 files", { exact: true }).waitFor();
    await page.getByRole("button", { name: "Collapse sidebar", exact: true }).click();
    assert.equal(await page.locator(".session-count").innerText(), "3 · 3");
    assert.equal(await page.locator(".session-stats").isVisible(), false);
    await page.getByRole("button", { name: "Expand sidebar", exact: true }).click();
    await shot("portfolio-dark");
    checks.push("Brown SVG favicon loads; loaded RAD summary reflects samples and collapses to a count");
    await page.getByRole("searchbox", { name: "Search activities" }).fill("no-matching-activity-xyz");
    await page.getByRole("heading", { name: "No matching activities", exact: true }).waitFor();
    await page.getByRole("button", { name: "Reset filters", exact: true }).click();
    assert.equal(await page.locator(".activity-table tbody tr").count(), 3);
    checks.push("Empty search has a working filter reset");
    const activityButton = page.locator(".activity-select").first();
    const activityQuery = await activityButton.innerText();
    await page.getByRole("searchbox", { name: "Search activities" }).fill(activityQuery);
    await activityButton.scrollIntoViewIfNeeded();
    const portfolioScroll = await page.evaluate(() => scrollY);
    await activityButton.click();
    const drawer = page.getByRole("dialog");
    await drawer.waitFor();
    assert.ok(await drawer.getByRole("heading", { name: "Contributors", exact: false }).isVisible());
    assert.equal(Math.round((await drawer.boundingBox()).width), 480);
    const drawerGrip = drawer.getByRole("separator", { name: "Resize details drawer" });
    await drawerGrip.press("ArrowLeft");
    assert.equal(Math.round((await drawer.boundingBox()).width), 504);
    const gripBox = await drawerGrip.boundingBox();
    await page.mouse.move(gripBox.x + 4, gripBox.y + 200);
    await page.mouse.down(); await page.mouse.move(gripBox.x - 44, gripBox.y + 200, { steps: 6 }); await page.mouse.up();
    assert.equal(Math.round((await drawer.boundingBox()).width), 552);
    await drawer.screenshot({ path: resolve(output, "drawer-overview-dark.png") });
    await drawer.getByRole("tab", { name: "Overview", exact: true }).press("ArrowRight");
    await drawer.locator("#drawer-source-editor").waitFor();
    const previewText = await drawer.locator("#drawer-source").inputValue();
    assert.equal(await drawer.locator("#drawer-source-editor").getAttribute("aria-readonly"), "true");
    await drawer.locator("#drawer-source-editor").press("ControlOrMeta+Home");
    await page.keyboard.insertText("changed");
    assert.equal(await drawer.locator("#drawer-source").inputValue(), previewText);
    await drawer.getByRole("button", { name: "Fold all", exact: true }).click();
    await drawer.locator(".cm-foldPlaceholder").first().click();
    assert.ok(await drawer.locator(".cm-foldPlaceholder").count() > 0);
    await drawer.getByRole("button", { name: "Copy code", exact: true }).click();
    assert.equal(await page.evaluate(() => navigator.clipboard.readText()), previewText);
    const previewDownload = page.waitForEvent("download");
    await drawer.getByRole("button", { name: "Download source" }).click();
    assert.equal(await readFile(await (await previewDownload).path(), "utf8"), previewText);
    await drawer.getByRole("tab", { name: "Checks", exact: true }).click();
    await drawer.getByRole("button", { name: "Copy result", exact: true }).waitFor();
    await page.keyboard.press("Escape");
    await drawer.waitFor({ state: "hidden" });
    assert.equal(await activityButton.evaluate(node => node === document.activeElement), true);
    assert.equal(await page.getByRole("searchbox", { name: "Search activities" }).inputValue(), activityQuery);
    assert.ok(Math.abs(await page.evaluate(() => scrollY) - portfolioScroll) < 2, "closing the drawer preserves the Portfolio scroll position");
    await page.locator("#theme-toggle").click();
    await activityButton.click();
    await drawer.screenshot({ path: resolve(output, "drawer-overview-light.png") });
    await page.setViewportSize({ width: 390, height: 844 });
    assert.equal(Math.round((await drawer.boundingBox()).width), 390);
    assert.ok(await drawer.evaluate(node => node.scrollWidth <= node.clientWidth));
    await drawer.screenshot({ path: resolve(output, "drawer-mobile.png") });
    await page.keyboard.press("Escape");
    await drawer.waitFor({ state: "hidden" });
    await page.waitForFunction(() => document.body.style.overflow === "");
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.locator("#theme-toggle").click();
    await activityButton.click();
    await drawer.getByRole("button", { name: "Open in Research Activity", exact: true }).click();
    await drawer.waitFor({ state: "hidden" });
    assert.equal(await page.locator("#content").inputValue(), previewText);
    checks.push("RAD drawer previews, folds, copies and downloads read-only source; resizing, mobile layout, checks and focus return work");
    const tools = ["Research Activity", "Metadata standard", "Crosswalk", "FAIR assessment", "Readability", "Content diff", "Big-O comparison", "Discover identifiers", "Inspect identifier"];
    for (const name of tools) {
        await nav(name);
        if (name === "Research Activity" || name === "Crosswalk") {
            await page.getByRole("button", { name: "Clear", exact: true }).click();
            if (name === "Crosswalk") await page.locator("#content-editor").fill("{");
            await page.getByRole("button", { name: actionLabels[name], exact: true }).click();
            const error = page.locator(".operation-error");
            await error.waitFor();
            assert.equal(await error.getAttribute("data-tone"), name === "Research Activity" ? "warning" : "danger");
            assert.equal(await page.locator("#content").inputValue(), name === "Research Activity" ? "" : "{");
            await error.screenshot({ path: resolve(output, `operation-error-${name === "Research Activity" ? "empty" : "invalid"}-dark.png`) });
            await page.locator("#theme-toggle").click();
            await error.screenshot({ path: resolve(output, `operation-error-${name === "Research Activity" ? "empty" : "invalid"}-light.png`) });
            await page.locator("#theme-toggle").click();
            await error.getByRole("button", { name: "Back to input" }).click();
            assert.equal(await page.locator("#content-editor").evaluate(node => node === document.activeElement), true);
        }
        await page.getByRole("button", { name: "Use example", exact: true }).click();
        assert.ok(await page.getByRole("button", { name: "Use example", exact: true }).evaluate((node) => node.classList.contains("sample-action") && node.closest(".input-heading")), "Examples use the Home sample style in the Input header");
        fileContent = await page.locator("#content").inputValue();
        await page.locator("#content-editor").fill("Replace this from the URL");
        assert.equal(await page.locator(".source-tray").count(), 0);
        await openFileUrl();
        await page.getByLabel("File URL", { exact: true }).fill("https://files.acorn.test/example.txt");
        await page.getByRole("button", { name: "Load URL", exact: true }).click();
        await page.locator(".source-loaded").filter({ hasText: "Loaded example.txt" }).waitFor();
        await page.locator(".source-tray").waitFor({ state: "hidden" });
        await page.waitForFunction(() => document.activeElement === document.getElementById("content-editor"));
        assert.equal(await page.locator("#content").inputValue(), fileContent, `${name} loads a file URL`);
        if (name === "Research Activity") {
            await page.locator(".file-picker").screenshot({ path: resolve(output, "source-collapsed.png") });
            await openFileUrl();
            assert.equal(await page.locator(".source-tips").getAttribute("open"), null);
            await page.locator(".source-tray").screenshot({ path: resolve(output, "source-tray-dark.png") });
            await page.locator("#theme-toggle").click();
            await page.locator(".source-tray").screenshot({ path: resolve(output, "source-tray-light.png") });
            await page.setViewportSize({ width: 390, height: 844 });
            await noOverflow("Mobile content loader");
            await page.locator(".source-tray").screenshot({ path: resolve(output, "source-tray-mobile.png") });
            await page.setViewportSize({ width: 1440, height: 900 });
            await page.locator("#theme-toggle").click();
            for (const [url, error] of [["http://files.acorn.test/file.txt", /HTTPS/], ["https://files.acorn.test/missing.txt", /HTTP 404/], ["https://files.acorn.test/large.txt", /size limit/], ["https://files.acorn.test/blocked.txt", /CORS/]]) {
                await page.getByLabel("File URL", { exact: true }).fill(url);
                await page.getByRole("button", { name: "Load URL", exact: true }).click();
                await page.locator(".url-file-loader").getByRole("alert").filter({ hasText: error }).waitFor();
                assert.equal(await page.locator("#content").inputValue(), fileContent);
            }
            await page.getByLabel("File URL", { exact: true }).fill("https://files.acorn.test/slow.txt");
            const requested = page.waitForRequest("https://files.acorn.test/slow.txt");
            await page.getByRole("button", { name: "Load URL", exact: true }).click();
            await requested;
            await page.locator(".url-file-loader").getByRole("button", { name: "Cancel", exact: true }).click();
            await page.locator(".url-file-loader").getByRole("status").filter({ hasText: /cancelled/ }).waitFor();
            releaseFile?.();
            assert.equal(await page.locator("#content").inputValue(), fileContent);
            const localContent = fileContent;
            fileContent = "stale remote content";
            const pendingLocal = page.waitForRequest("https://files.acorn.test/slow.txt");
            await page.getByRole("button", { name: "Load URL", exact: true }).click();
            await pendingLocal;
            await page.locator('.file-picker input[type="file"]').setInputFiles({ name: "local.json", mimeType: "application/json", buffer: Buffer.from(localContent) });
            await page.locator(".source-loaded").filter({ hasText: "Loaded local.json" }).waitFor();
            await page.locator(".source-tray").waitFor({ state: "hidden" });
            releaseFile?.();
            await page.waitForFunction(content => document.getElementById("content").value === content, localContent);
            await openFileUrl();
            await page.getByLabel("File URL", { exact: true }).fill("https://files.acorn.test/slow.txt");
            const pendingEdit = page.waitForRequest("https://files.acorn.test/slow.txt");
            await page.getByRole("button", { name: "Load URL", exact: true }).click();
            await pendingEdit;
            await page.locator("#content-editor").fill("Newer user input");
            await page.locator(".url-file-loader").getByRole("status").filter({ hasText: /cancelled/ }).waitFor();
            releaseFile?.();
            assert.equal(await page.locator("#content").inputValue(), "Newer user input");
            const pendingNavigation = page.waitForRequest("https://files.acorn.test/slow.txt");
            await page.getByRole("button", { name: "Load URL", exact: true }).click();
            await pendingNavigation;
            await nav("Metadata standard");
            const nextInput = await page.locator("#content").inputValue();
            releaseFile?.();
            assert.equal(await page.locator(".source-tray").count(), 0);
            await openFileUrl();
            assert.equal(await page.getByLabel("File URL", { exact: true }).inputValue(), "");
            assert.equal(await page.locator("#content").inputValue(), nextInput);
            await nav("Research Activity");
            fileContent = localContent;
            await openFileUrl();
            await page.getByLabel("File URL", { exact: true }).fill("https://files.acorn.test/slow.txt");
            const pendingClose = page.waitForRequest("https://files.acorn.test/slow.txt");
            await page.getByRole("button", { name: "Load URL", exact: true }).click();
            await pendingClose;
            await page.keyboard.press("Escape");
            await page.locator(".source-tray").waitFor({ state: "hidden" });
            await page.waitForFunction(() => document.activeElement?.classList.contains("source-trigger"));
            releaseFile?.();
            await page.getByRole("button", { name: "Load content", exact: false }).click();
            const localPickerEvent = page.waitForEvent("filechooser");
            await page.getByRole("button", { name: "Choose file", exact: false }).click();
            await (await localPickerEvent).setFiles({ name: "reselected.json", mimeType: "application/json", buffer: Buffer.from(localContent) });
            await page.locator(".source-loaded").filter({ hasText: "Loaded reselected.json" }).waitFor();
            await page.locator(".source-tray").waitFor({ state: "hidden" });
            assert.equal(await page.locator("#content").inputValue(), localContent);
            checks.push("On-demand content tray supports mobile layouts, keyboard dismissal, focus return and native file selection");
            await openFileUrl();
            await page.getByLabel("File URL", { exact: true }).fill("https://files.acorn.test/example.txt");
            await page.getByRole("button", { name: "Load URL", exact: true }).click();
            await page.locator(".source-loaded").filter({ hasText: "Loaded example.txt" }).waitFor();
            await page.locator(".source-tray").waitFor({ state: "hidden" });
            await page.waitForFunction(() => document.activeElement === document.getElementById("content-editor"));
            checks.push("File URL failures, byte limits, blocked browser access, cancellation, local selection and navigation preserve editor content");
            const editor = page.locator("#content-editor");
            const controls = page.getByRole("group", { name: "Content editor controls", exact: true });
            const source = await page.locator("#content").inputValue();
            const surface = page.locator(".editor-surface").first();
            const beforeResize = await surface.boundingBox();
            const handle = page.locator(".editor-resize-handle").first();
            await handle.scrollIntoViewIfNeeded();
            const grip = await handle.boundingBox();
            await page.mouse.move(grip.x + grip.width / 2, grip.y + grip.height / 2);
            await page.mouse.down();
            await page.mouse.move(grip.x + grip.width / 2, grip.y + grip.height / 2 - 60, { steps: 8 });
            await page.mouse.up();
            const resizedHeight = (await surface.boundingBox()).height;
            assert.ok(Math.abs(resizedHeight - beforeResize.height + 60) < 5, "Code editors resize by dragging the bottom handle");
            await controls.getByRole("button", { name: "Fold all", exact: true }).click();
            await page.locator(".cm-foldPlaceholder").first().waitFor();
            assert.ok(await editor.locator(".cm-line").count() <= 2, "Fold all includes offscreen parts of the loaded RAD example");
            await page.locator('.cm-foldGutter [title="Unfold line"]:visible').first().click();
            assert.ok(await page.locator(".cm-foldPlaceholder").count() > 1, "opening the RAD root retains nested folds");
            assert.equal(await editor.innerText().then(text => text.includes('"archive"')), false, "RAD metadata stays folded");
            await page.locator('.cm-foldGutter [title="Unfold line"]:visible').first().click();
            assert.ok(await page.locator(".cm-foldPlaceholder").count() > 1, "opening RAD metadata retains its nested arrays and objects");
            assert.equal(await page.locator("#content").inputValue(), source);
            await controls.getByRole("button", { name: "Fold all", exact: true }).click();
            await page.locator(".code-editor").first().screenshot({ path: resolve(output, "editor-folded-dark.png") });
            await page.locator("#theme-toggle").click();
            await page.locator(".code-editor").first().screenshot({ path: resolve(output, "editor-folded-light.png") });
            await page.locator("#theme-toggle").click();
            await editor.press("ControlOrMeta+Home");
            await page.keyboard.insertText(" ");
            await page.waitForFunction(text => document.getElementById("content").value === " " + text, source);
            assert.ok(await page.locator(".cm-foldPlaceholder").count() > 0, "Dioxus input echoes preserve folding");
            await editor.press("ControlOrMeta+z");
            await page.waitForFunction(text => document.getElementById("content").value === text, source);
            assert.equal((await surface.boundingBox()).height, resizedHeight, "Dioxus updates preserve the resized height");
            await controls.getByRole("button", { name: "Copy code", exact: true }).click();
            assert.equal(await page.evaluate(() => navigator.clipboard.readText()), source, "folded editor copy includes the full source");
            await controls.getByRole("button", { name: "Expand all", exact: true }).click();
            assert.equal(await page.locator(".cm-foldPlaceholder").count(), 0);
            const vocabulary = page.getByRole("group", { name: "Vocabulary matching" });
            const content = await page.locator("#content").inputValue();
            assert.equal(await vocabulary.getByRole("button", { pressed: true }).innerText(), "Off");
            for (const mode of ["Relaxed", "Strict", "Off"]) {
                await vocabulary.getByRole("button", { name: mode, exact: true }).click();
                assert.equal(await vocabulary.getByRole("button", { pressed: true }).innerText(), mode);
                assert.equal(await page.locator("#content").inputValue(), content);
            }
        }
        if (!["Readability", "Content diff", "Discover identifiers"].includes(name)) await page.locator('#content-editor [class^="tok-"]').first().waitFor();
        assert.equal(await page.locator("#content-editor").getAttribute("aria-label"), name === "Content diff" ? "Before" : name === "Big-O comparison" ? "Left expression" : "Content");
        await page.getByRole("button", { name: actionLabels[name], exact: true }).click();
        await page.getByRole("button", { name: "Copy result", exact: true }).waitFor();
        await page.waitForFunction(() => document.getElementById("result-heading") === document.activeElement);
        if (name === "Research Activity") {
            const beforeInspection = await page.locator("#content").inputValue();
            const inspect = page.locator(".inspect-diagnostic").first();
            await inspect.click();
            await drawer.getByRole("heading", { name: "Affected value", exact: true }).waitFor();
            await drawer.getByRole("heading", { name: "Surrounding data", exact: true }).waitFor();
            await drawer.screenshot({ path: resolve(output, "drawer-diagnostic.png") });
            await drawer.getByRole("tab", { name: "Source", exact: true }).click();
            assert.equal(await drawer.locator("#drawer-source").inputValue(), beforeInspection);
            await page.keyboard.press("Escape");
            await drawer.waitFor({ state: "hidden" });
            assert.equal(await page.locator("#content").inputValue(), beforeInspection);
            assert.equal(await inspect.evaluate(node => node === document.activeElement), true);
            checks.push("Diagnostic inspection locates values and context without changing the editor");
        }
        if (["Research Activity", "Metadata standard"].includes(name)) {
            const errors = await page.locator('.diagnostic-group[data-category="schema"] .diagnostic').count();
            assert.ok(errors >= 15 && errors <= 25, "Validation example keeps a focused selection of schema errors");
            for (const tone of ["info", "suggestion"]) assert.ok(await page.locator(`.diagnostic-group[data-category="fair"] .diagnostic[data-tone="${tone}"]`).count() > 0, `RAD validation includes ${tone}`);
        }
        if (["Research Activity", "Metadata standard", "FAIR assessment"].includes(name)) {
            const findings = page.locator('.diagnostic-group[data-category="fair"] .diagnostic');
            assert.equal(await findings.count(), fairRules.length);
            const descriptions = await findings.allTextContents();
            for (const rule of fairRules) assert.ok(descriptions.some(text => text.includes(rule.id) && text.includes(rule.description)), `Missing rule description: ${rule.id}`);
            const first = findings.first();
            const rule = fairRules.find(rule => descriptions[0].includes(rule.id));
            await first.getByRole("button", { name: "Inspect", exact: true }).click();
            await drawer.waitFor({ state: "visible" });
            assert.ok((await drawer.innerText()).includes(rule.description), "Inspector retains the RDA rule description");
            await page.keyboard.press("Escape");
            await drawer.waitFor({ state: "hidden" });
            checks.push(`${name} includes RDA descriptions for every FAIR finding and in the inspector`);
        }
        if (name === "Discover identifiers") {
            const kinds = await page.locator(".identifier-row .badge").allTextContents();
            for (const kind of ["ark", "arxiv", "doi", "handle", "isbn", "isni", "orcid", "patent", "raid", "ror", "rrid", "swhid"]) assert.ok(kinds.filter((value) => value === kind).length >= 2, `Missing multiple ${kind} examples`);
            const filters = page.getByRole("group", { name: "Filter identifiers by type", exact: true });
            const originalInput = await page.locator("#content").inputValue();
            const all = filters.getByRole("button", { name: /^All/ });
            assert.equal(await all.getAttribute("aria-pressed"), "true");
            await all.click();
            assert.equal(await all.getAttribute("aria-pressed"), "false");
            assert.equal(await page.locator(".identifier-row").count(), 0);
            await page.getByText("No PID types selected. Choose a type or All to show results.", { exact: true }).waitFor();
            const selected = new Set();
            const types = [...new Set(kinds)].sort();
            for (const kind of types) {
                const filter = filters.locator(`[data-pid="${kind}"]`);
                const count = kinds.filter(value => value === kind).length;
                assert.equal(await filter.locator(".pid-filter-count").innerText(), String(count));
                await filter.click();
                selected.add(kind);
                const expected = kinds.filter(value => selected.has(value));
                assert.equal(await filter.getAttribute("aria-pressed"), "true");
                assert.equal(await filters.locator('[data-pid][aria-pressed="true"]').count(), selected.size);
                assert.equal(await all.getAttribute("aria-pressed"), selected.size === types.length ? "true" : "mixed");
                assert.deepEqual(await page.locator(".identifier-row .badge").allTextContents(), expected);
                assert.equal(await page.locator(".pid-filter-summary").innerText(), `Showing ${expected.length} of ${kinds.length} identifiers`);
            }
            const first = filters.locator(`[data-pid="${types[0]}"]`);
            await first.focus();
            await page.keyboard.press("Space");
            assert.equal(await first.getAttribute("aria-pressed"), "false");
            assert.equal(await all.getAttribute("aria-pressed"), "mixed");
            assert.deepEqual(await page.locator(".identifier-row .badge").allTextContents(), kinds.filter(kind => kind !== types[0]));
            await page.getByRole("button", { name: "Copy result", exact: true }).click();
            assert.equal(JSON.parse(await page.evaluate(() => navigator.clipboard.readText())).identifiers.length, kinds.length);
            await all.click();
            assert.deepEqual(await page.locator(".identifier-row .badge").allTextContents(), kinds);
            await all.click();
            assert.equal(await page.locator(".identifier-row").count(), 0);
            await page.keyboard.press("Enter");
            assert.deepEqual(await page.locator(".identifier-row .badge").allTextContents(), kinds);
            assert.equal(await page.locator("#content").inputValue(), originalInput);
            checks.push("PID toggles combine types, support keyboard removal and All show/hide, and preserve source and full exports");
            for (const theme of ["dark", "light"]) {
                if (await page.locator(".workspace").getAttribute("data-theme") !== theme) await page.locator("#theme-toggle").click();
                const badges = await page.locator(".identifier-row .pid-badge").evaluateAll(nodes => {
                    const canvas = document.createElement("canvas");
                    canvas.width = canvas.height = 1;
                    const context = canvas.getContext("2d");
                    const luminance = color => {
                        context.fillStyle = color;
                        context.fillRect(0, 0, 1, 1);
                        const linear = Array.from(context.getImageData(0, 0, 1, 1).data).slice(0, 3).map(channel => {
                            const value = channel / 255;
                            return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
                        });
                        return linear[0] * 0.2126 + linear[1] * 0.7152 + linear[2] * 0.0722;
                    };
                    return nodes.map(node => {
                        const style = getComputedStyle(node);
                        const foreground = luminance(style.color);
                        const background = luminance(style.backgroundColor);
                        return { kind: node.dataset.pid, brand: style.getPropertyValue("--pid-brand").trim(), contrast: (Math.max(foreground, background) + 0.05) / (Math.min(foreground, background) + 0.05) };
                    });
                });
                assert.equal(badges.length, kinds.length);
                assert.equal(new Set(badges.map(badge => badge.brand)).size, 12);
                for (const badge of badges) assert.ok(badge.contrast >= 4.5, `${theme} ${badge.kind} contrast: ${badge.contrast}`);
                await page.locator(".results").screenshot({ path: resolve(output, `pid-labels-${theme}.png`) });
            }
            await page.locator("#theme-toggle").click();
            checks.push("All PID labels have distinct accents and readable text contrast in both themes");
        }
        if (name === "Content diff") assert.equal(await page.getByRole("button", { name: "JSON Patch", exact: true }).getAttribute("aria-pressed"), "true");
        if (["Research Activity", "Metadata standard", "Crosswalk", "FAIR assessment"].includes(name)) {
            const controls = page.getByRole("group", { name: "Sort diagnostics", exact: true });
            const initial = await page.locator(".diagnostics").allTextContents();
            const field = name === "FAIR assessment" ? "Indicator" : "Field";
            await controls.getByRole("button", { name: field, exact: true }).click();
            assert.equal(await controls.getByRole("button", { name: field, exact: true }).getAttribute("aria-pressed"), "true");
            const orderedFields = () => page.locator(".diagnostics").evaluateAll(lists => lists.map(list => Array.from(list.querySelectorAll(".diagnostic-heading strong"), node => node.textContent.toLowerCase())));
            for (const values of await orderedFields()) assert.deepEqual(values, [...values].sort());
            await controls.getByRole("button", { name: "Sort direction: A–Z", exact: true }).click();
            for (const values of await orderedFields()) assert.deepEqual(values, [...values].sort().reverse());
            if (name !== "Crosswalk") {
                await controls.getByRole("button", { name: "Severity", exact: true }).click();
                const severities = () => page.locator(".diagnostics").evaluateAll(lists => lists.map(list => Array.from(list.querySelectorAll(".diagnostic"), node => ({ danger: 0, warning: 1, suggestion: 2, info: 3 })[node.dataset.tone])));
                for (const values of await severities()) assert.deepEqual(values, [...values].sort((a,b) => a-b));
                await controls.getByRole("button", { name: "Sort direction: Highest first", exact: true }).click();
                for (const values of await severities()) assert.deepEqual(values, [...values].sort((a,b) => b-a));
            }
            const rule = name === "Crosswalk" ? "Loss type" : name === "FAIR assessment" ? "State" : "Rule";
            await controls.getByRole("button", { name: rule, exact: true }).click();
            assert.equal(await controls.getByRole("button", { name: rule, exact: true }).getAttribute("aria-pressed"), "true");
            await controls.getByRole("button", { name: "Original", exact: true }).click();
            assert.deepEqual(await page.locator(".diagnostics").allTextContents(), initial);
            assert.ok(await page.locator("details.diagnostic-group").count() > 0);
            for (const category of await page.locator("details.diagnostic-group").all()) {
                const header = category.locator(":scope > summary");
                const count = await header.locator(".badge").innerText();
                assert.ok(await category.evaluate(node => node.open));
                await header.click();
                assert.equal(await category.evaluate(node => node.open), false);
                assert.equal(await category.locator(".diagnostic").first().isVisible(), false);
                assert.equal(await header.locator(".badge").innerText(), count);
                if (["Research Activity", "Metadata standard"].includes(name)) {
                    await controls.getByRole("button", { name: "Severity", exact: true }).click();
                    assert.equal(await category.evaluate(node => node.open), false, "Sorting preserves collapsed categories");
                    await controls.getByRole("button", { name: "Original", exact: true }).click();
                }
                await header.focus();
                await page.keyboard.press("Enter");
                assert.ok(await category.evaluate(node => node.open));
                assert.ok(await category.locator(".diagnostic").first().isVisible());
            }

        }
        assert.equal(await page.locator(".credit-line,.acronym-note").count(), 0);
        await page.getByText("Raw result JSON", { exact: true }).click();
        await copyCodes();
        await page.getByText("Raw result JSON", { exact: true }).click();
        await noOverflow(name);
        await shot(name.toLowerCase().replaceAll(" ", "-"));
        checks.push(`${name}: example operation, unnumbered sections, code copying and focus`);
    }
    await nav("Research Activity");
    await page.locator("#content-editor").fill('{"title":1}');
    await page.getByRole("button", { name: "Validate", exact: true }).click();
    await page.getByText(/Review needed ·/).waitFor();
    assert.ok(await page.locator(".diagnostics li").count() > 0);
    assert.equal(await page.locator("#content").inputValue(), '{"title":1}');
    await shot("validation-invalid");
    await nav("Readability");
    const scoreColors = new Map();
    for (const metric of ["fres", "ari"]) {
        for (const [expected, content] of [["pass", "The cat sat. The dog ran. We had fun."], ["review", "Interdisciplinary institutionalization of epistemological methodologies necessitates comprehensive contextualization, operationalization, and methodological reconceptualization of extraordinarily heterogeneous organizational infrastructures."]]) {
            await page.getByRole("combobox").selectOption(metric);
            await page.locator("#content-editor").fill(content);
            await page.getByRole("button", { name: "Analyze readability", exact: true }).click();
            await page.locator(`.readability-result[data-status="${expected}"]`).waitFor();
            await page.getByText("Raw result JSON", { exact: true }).click();
            const check = JSON.parse(await page.locator(".results .code-block code").textContent()).checks[0];
            assert.equal(check.success, metric === "fres" ? check.score >= check.threshold : check.score <= check.threshold);
            const colors = await page.locator(".readability-result").evaluate((node) => [getComputedStyle(node.querySelector(".score")).color, getComputedStyle(node.querySelector(".score-marker")).borderLeftColor]);
            assert.equal(colors[0], colors[1]);
            scoreColors.set(expected, colors[0]);
            await page.getByText("Raw result JSON", { exact: true }).click();
        }
    }
    assert.notEqual(scoreColors.get("pass"), scoreColors.get("review"));
    await shot("readability-threshold");
    checks.push("Readability colors score and marker for both threshold directions");
    await nav("Content diff");
    await page.getByRole("button", { name: "Unified diff", exact: true }).click();
    await page.locator("#content-editor").fill("before\n"); await page.locator("#after-editor").fill("<script>literal</script>\n");
    await page.getByRole("button", { name: "Compare", exact: true }).click();
    const downloaded = page.waitForEvent("download");
    await page.getByRole("button", { name: "Download result", exact: true }).click();
    const diff = await downloaded;
    assert.equal(diff.suggestedFilename(), "acorn-changes.diff");
    const diffText = await readFile(await diff.path(), "utf8");
    assert.match(diffText, /\+<script>literal<\/script>/);
    await page.getByRole("button", { name: "Copy diff", exact: true }).click();
    await page.waitForFunction(async (text) => await navigator.clipboard.readText() === text, diffText);
    await page.evaluate(async () => {
        const clipboard = navigator.clipboard;
        Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText: async () => { throw new Error("denied"); } } });
        await window.acornCopy("denied copy");
        Object.defineProperty(navigator, "clipboard", { configurable: true, value: clipboard });
    });
    await page.locator("#notification-text").getByText("Clipboard unavailable. Select and copy the code manually.", { exact: true }).waitFor();
    checks.push("Invalid metadata preserves input; unified export preserves literal bytes");
    await page.locator("#content-editor").fill('{"title":"Before"}');
    await page.locator("#after-editor").fill('{"title":"After"}');
    const diffFormats = page.getByRole("group", { name: "Diff format", exact: true });
    await diffFormats.getByRole("button", { name: "JSON Patch", exact: true }).click();
    assert.equal(await diffFormats.getByRole("button", { name: "JSON Patch", exact: true }).getAttribute("aria-pressed"), "true");
    assert.equal(await diffFormats.getByRole("button", { pressed: true }).count(), 1);
    await page.getByRole("button", { name: "Compare", exact: true }).click();
    await page.getByText("1 JSON Patch operations", { exact: true }).waitFor();
    await shot("diff-json-patch");
    await diffFormats.getByRole("button", { name: "Unified diff", exact: true }).click();
    assert.equal(await diffFormats.getByRole("button", { pressed: true }).count(), 1);
    assert.equal(await page.locator("#after").inputValue(), '{"title":"After"}');
    checks.push("Icon diff buttons switch formats exclusively and preserve both inputs");
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
    assert.ok(await page.locator(".theme-sun").isVisible());
    assert.equal(await page.locator(".theme-moon").isVisible(), false);
    assert.equal(await page.locator(".credit-line,.acronym-note").count(), 0);
    await shot("settings-light");
    checks.push("Explicit and system themes, live preference changes, session metrics and matching skill");
    await nav("ACORN Skill");
    await page.getByRole("button", { name: "Download skill folder", exact: true }).waitFor();
    const archiveEvent = page.waitForEvent("download");
    await page.getByRole("button", { name: "Download skill folder", exact: true }).click();
    const archive = await archiveEvent;
    const archiveHash = createHash("sha256").update(await readFile(await archive.path())).digest("hex");
    assert.ok(await page.locator("main").textContent().then((text) => text.includes(archiveHash)));
    await page.getByText("Read bundled SKILL.md", { exact: true }).click();
    await copyCodes();
    await page.getByText("Read bundled SKILL.md", { exact: true }).click();
    const filters = page.getByRole("group", { name: "Filter workflow examples" });
    await filters.getByRole("button", { name: "Review 1" }).click();
    assert.equal(await page.locator(".workflow-example").count(), 1);
    const workflow = page.getByRole("article", { name: "Review research text", exact: true });
    const menu = workflow.getByLabel("Actions for Review research text");
    await menu.click();
    await page.keyboard.press("Escape");
    assert.equal(await workflow.locator("details").getAttribute("open"), null);
    await menu.click();
    const promptDownload = page.waitForEvent("download");
    await workflow.getByRole("button", { name: "Download prompt" }).click();
    const promptFile = await promptDownload;
    assert.equal(await readFile(await promptFile.path(), "utf8"), await workflow.locator("code").textContent());
    await menu.click();
    await workflow.getByRole("link", { name: "Open related tool" }).click();
    await page.getByRole("heading", { name: "Readability", exact: true }).waitFor();
    await nav("ACORN Skill");
    await page.locator(".workflow-example").last().waitFor();
    assert.equal(await page.locator(".workflow-example").count(), 4);
    checks.push("Workflow categories filter cards; menus support Escape, prompt download and related tools");
    await shot("skill-light");
    await page.locator("#theme-toggle").click();
    await shot("skill-dark");
    await page.locator("#theme-toggle").click();
    checks.push("Skill archive bytes match the displayed manifest digest");
    await nav("Install"); await shot("install-light");
    assert.equal(await page.locator(".channel-row").count(), 5);
    await nav("Geospatial");
    await page.getByText(/ROR 2.15/).waitFor({ timeout: 60000 });
    await page.locator(".organization-table tbody tr").first().waitFor();
    await page.locator("#geospatial-map canvas").waitFor({ timeout: 60000 });
    const mapHeight = await page.locator("#geospatial-map").evaluate((node) => node.clientHeight);
    await page.locator(".cesium-fullscreenButton").click();
    await page.waitForFunction(() => document.fullscreenElement?.id === "geospatial-map");
    await page.waitForFunction(() => { const canvas = document.querySelector("#geospatial-map canvas"); return Math.abs(canvas.clientWidth - innerWidth) < 2 && Math.abs(canvas.clientHeight - innerHeight) < 2; });
    await shot("geospatial-fullscreen");
    await page.locator(".cesium-fullscreenButton").click();
    await page.waitForFunction(() => document.fullscreenElement === null);
    await page.waitForFunction((height) => document.querySelector("#geospatial-map").clientHeight === height, mapHeight);
    await page.waitForFunction(() => performance.getEntriesByType("resource").some((entry) => /BlackMarble\/.+\.png/.test(entry.name)));
    await page.locator(".organization-table tbody button").first().click();
    await drawer.getByRole("heading", { name: "Related activities", exact: true }).waitFor();
    await drawer.screenshot({ path: resolve(output, "drawer-organization.png") });
    await drawer.locator(".details-related").first().click();
    await drawer.getByRole("button", { name: "Open in Research Activity", exact: true }).waitFor();
    await page.keyboard.press("Escape");
    await drawer.waitFor({ state: "hidden" });
    await page.getByTitle("Center on ORNL", { exact: true }).click();
    await page.waitForTimeout(2000);
    const globe = page.locator("#geospatial-map canvas");
    const globeBox = await globe.boundingBox();
    await globe.click({ position: { x: globeBox.width / 2, y: globeBox.height / 2 } });
    await drawer.getByRole("heading", { name: "Oak Ridge National Laboratory", exact: true }).waitFor();
    await page.keyboard.press("Escape");
    await drawer.waitFor({ state: "hidden" });
    checks.push("Globe markers and organization rows open the shared drawer with related activities");
    assert.ok(tiles.size > 0, "Local NASA Black Marble tiles loaded");
    await shot("geospatial-light");
    checks.push("Pinned local ROR table and WebGL globe render");
    checks.push("Cesium fullscreen resizes and restores the map; Home centers on ORNL");
    await nav("Settings");
    await page.getByRole("button", { name: "Clear loaded session", exact: true }).click();
    await nav("Discover identifiers");
    await page.locator("#content-editor").fill("ROR: https://ror.org/01qz5mb56 DOI: 10.5281/zenodo.10000001");
    await page.getByRole("button", { name: "Discover identifiers", exact: true }).click();
    const rorRow = page.locator(".identifier-row").filter({ has: page.locator('[data-pid="ror"]') });
    assert.equal(await page.getByRole("button", { name: "View on map", exact: true }).count(), 1);
    await rorRow.getByRole("button", { name: "View on map", exact: true }).click();
    await page.locator(".map-focus").getByText("Oak Ridge National Laboratory", { exact: true }).waitFor({ timeout: 60000 });
    await page.locator("#geospatial-map canvas").waitFor({ timeout: 60000 });
    assert.equal(await page.locator(".organization-table tbody tr").count(), 1);
    assert.equal(await page.locator(".organization-table tbody tr td").last().innerText(), "0");
    assert.equal(await page.getByRole("heading", { name: "No validated activities", exact: true }).count(), 0);
    await page.locator(".map-focus").getByRole("button", { name: "Inspect", exact: true }).click();
    await drawer.getByRole("heading", { name: "Oak Ridge National Laboratory", exact: true }).waitFor();
    await drawer.getByRole("tab", { name: "Source", exact: true }).click();
    const mappedRor = JSON.parse(await drawer.locator("#drawer-source").inputValue());
    assert.equal(mappedRor.id, "https://ror.org/01qz5mb56");
    assert.equal(mappedRor.references, 0);
    assert.deepEqual(mappedRor.activities, []);
    assert.ok(Number.isFinite(mappedRor.location.latitude));
    await page.keyboard.press("Escape");
    await drawer.waitFor({ state: "hidden" });
    await page.waitForTimeout(1500);
    const pinCanvas = page.locator("#geospatial-map canvas");
    const pinBox = await pinCanvas.boundingBox();
    await pinCanvas.click({ position: { x: pinBox.width / 2, y: pinBox.height / 2 } });
    await drawer.getByRole("heading", { name: "Oak Ridge National Laboratory", exact: true }).waitFor();
    await page.keyboard.press("Escape");
    await drawer.waitFor({ state: "hidden" });
    await shot("ror-map-pin");
    await nav("Inspect identifier");
    await page.locator("#content-editor").fill("01qz5mb56");
    await page.getByRole("button", { name: "Inspect identifier", exact: true }).click();
    await page.getByRole("button", { name: "View on map", exact: true }).click();
    await page.locator(".map-focus").getByText("Oak Ridge National Laboratory", { exact: true }).waitFor();
    assert.equal(await page.locator(".organization-table tbody tr").count(), 1);
    const localRors = JSON.parse(await readFile(new URL("../assets/geospatial/ror-locations.json", import.meta.url), "utf8"));
    const locationless = Object.entries(localRors.organizations).find(([id]) => id !== "https://ror.org/01qz5mb56");
    assert.ok(locationless);
    const mapFixture = { ...localRors, organizations: {
        "https://ror.org/01qz5mb56": localRors.organizations["https://ror.org/01qz5mb56"],
        [locationless[0]]: { ...locationless[1], locations: [] }
    } };
    await page.route("**/ror-locations*.json", route => route.fulfill({ json: mapFixture }));
    await nav("Inspect identifier");
    await page.locator("#content-editor").fill(locationless[0]);
    await page.getByRole("button", { name: "Inspect identifier", exact: true }).click();
    await page.getByRole("button", { name: "View on map", exact: true }).click();
    await page.locator(".map-focus").getByText(/no usable coordinates/).waitFor();
    assert.equal(await page.locator(".organization-table tbody tr").count(), 2);
    const unresolved = Object.keys(localRors.organizations).find(id => !mapFixture.organizations[id]);
    await nav("Inspect identifier");
    await page.locator("#content-editor").fill(unresolved);
    await page.getByRole("button", { name: "Inspect identifier", exact: true }).click();
    await page.getByRole("button", { name: "View on map", exact: true }).click();
    await page.locator(".map-focus").getByText(/not in the local index/).waitFor();
    assert.equal(await page.locator(".organization-table tbody tr").count(), 3);
    await page.unroute("**/ror-locations*.json");
    await page.getByRole("button", { name: "Clear added locations", exact: true }).click();
    await page.getByRole("heading", { name: "No validated activities", exact: true }).waitFor();
    assert.equal(await page.locator(".organization-table tbody tr").count(), 0);
    await nav("Inspect identifier");
    await page.locator("#content-editor").fill("https://ror.org/01qz5mb56xxx");
    await page.getByRole("button", { name: "Inspect identifier", exact: true }).click();
    assert.equal(await page.getByRole("button", { name: "View on map", exact: true }).count(), 0);
    checks.push("ROR discovery and inspection map standalone organizations, focus selectable Cesium entities, deduplicate pins, and explain missing coordinates without changing RAD");

    await nav("Home");
    fileContent = validActivity;
    await page.getByRole("button", { name: "Open file or folder", exact: false }).click();
    await page.getByRole("button", { name: "URL", exact: true }).click();
    await page.locator(".home-source-tray").screenshot({ path: resolve(output, "home-source-tray.png") });
    await page.getByRole("button", { name: "Close content loader", exact: true }).click();
    await page.locator(".home-source-tray").waitFor({ state: "hidden" });
    assert.equal(await page.locator("#home-source-trigger").evaluate(node => node === document.activeElement), true);
    await page.getByRole("button", { name: "Open file or folder", exact: false }).click();
    await page.getByRole("button", { name: "URL", exact: true }).click();
    await page.getByLabel("File URL", { exact: true }).fill("https://github.com/acorn-test/files/blob/main/activity.json");
    await page.getByRole("button", { name: "Load URL", exact: true }).click();
    await page.getByRole("status").filter({ hasText: "File loaded. Choose Portfolio" }).waitFor();
    await page.locator(".home-source-tray").waitFor({ state: "hidden" });
    await page.waitForFunction(() => document.activeElement === document.getElementById("home-source-trigger"));
    assert.match(await page.locator(".loaded-tree > summary").textContent(), /1 loaded files/);
    checks.push("Home loads GitHub file URLs into the workspace; all nine tools load direct file URLs");
    await page.getByRole("button", { name: "Open file or folder", exact: false }).click();
    const pickerEvent = page.waitForEvent("filechooser");
    await page.getByRole("button", { name: "Files", exact: true }).click();
    const picker = await pickerEvent;
    await picker.setFiles(resolve("crates/acorn-web/assets/examples/publications.json"));
    await page.getByText(/1 files loaded/).waitFor();
    await page.locator(".loaded-tree summary").click();
    await page.locator(".loaded-tree .file-leaf").first().click();
    await drawer.getByRole("tab", { name: "Checks", exact: true }).click();
    await drawer.getByRole("heading", { name: "No checks yet", exact: true }).waitFor();
    await page.keyboard.press("Escape");
    await drawer.waitFor({ state: "hidden" });
    assert.ok(page.url().endsWith("/"));
    checks.push("Loaded file inspection preserves the page and does not run validation implicitly");
    await page.getByText(/1 files loaded/).waitFor();
    await page.locator(".sidebar-session").getByText("0 activities · 1 file", { exact: true }).waitFor();
    await nav("Portfolio");
    await page.getByRole("button", { name: "Validate loaded activities", exact: true }).click();
    await page.locator(".activity-table tbody tr").first().waitFor();
    assert.equal(await page.locator(".activity-table tbody tr").count(), 1);
    await page.locator(".sidebar-session").getByText("1 activity · 1 file", { exact: true }).waitFor();
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
    await page.locator(".loaded-tree summary").filter({ hasText: /^\s*folder\s*$/ }).click();
    await page.getByRole("button", { name: "readme.txt", exact: true }).click();
    await drawer.getByRole("tab", { name: "Source", exact: true }).click();
    assert.equal(await drawer.locator("#drawer-source").inputValue(), "Local folder text");
    assert.doesNotMatch(await page.locator(".sidebar-session").getAttribute("title"), /Selected: folder\/readme\.txt/);
    await drawer.getByRole("button", { name: "Open in Research Activity", exact: true }).click();
    await drawer.waitFor({ state: "hidden" });
    assert.match(await page.locator(".sidebar-session").getAttribute("title"), /Selected: folder\/readme\.txt/);
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
        for (const name of ["Home", "Portfolio", "Research Activity", "Content diff", "Readability", "Geospatial", "ACORN Skill", "Install", "Settings", "About"]) {
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
    await page.locator(".sidebar-session").getByText("0 activities · 0 files", { exact: true }).waitFor();
    checks.push("Session clearing removes loaded files and validated records");
    await nav("Home");
    await page.locator(".banner-light").waitFor({ state: "visible" });
    assert.equal(await page.locator(".banner-dark").isVisible(), false);
    await shot("home-light");
    const sha = "a".repeat(40);
    const remoteContent = await readFile("crates/acorn-web/assets/examples/publications.json", "utf8");
    let pendingRemote;
    const observedRemote = [];
    await page.route((url) => mockedRemote(url.href), async (route) => {
        const url = route.request().url(); observedRemote.push(url);
        if (url.includes("/commits/pending")) { pendingRemote = route; return; }
        const body = url.endsWith("/bucket") || url.endsWith("%2Fbucket") ? { default_branch: "main" }
            : url.includes("/commits/") ? { sha, id: sha }
            : url.includes("/git/trees/") ? { tree: [{ type: "blob", path: "activity.json", sha, size: remoteContent.length }] }
            : url.includes("/git/blobs/") ? { encoding: "base64", content: Buffer.from(remoteContent).toString("base64") }
            : url.includes("/repository/tree?") ? [{ type: "blob", path: "activity.json" }]
            : remoteContent;
        await route.fulfill({ status: 200, contentType: "application/json", body: typeof body === "string" ? body : JSON.stringify(body) });
    });
    await page.locator(".remote-entry > summary").click();
    for (const source of ["acorn-test/bucket", "https://gitlab.com/acorn-test/bucket"]) {
        await page.getByRole("combobox", { name: "Repository or URL", exact: true }).fill(source);
        await page.getByRole("combobox", { name: "Branch / tag / commit (optional)", exact: true }).fill(source.startsWith("https") ? "release/v1" : "main");
        await page.getByRole("button", { name: "Load bucket", exact: true }).click();
        await page.getByText(/1 remote files loaded/).waitFor();
        await page.getByRole("button", { name: "Load bucket", exact: true }).waitFor();
    }
    await page.locator(".sidebar-session").getByText("0 activities · 1 file", { exact: true }).waitFor();
    await page.locator(".sidebar-session").click();
    await page.getByRole("heading", { name: "Portfolio", exact: true }).waitFor();
    await nav("Home");
    await page.locator(".remote-entry > summary").click();
    await page.waitForFunction(() => document.querySelectorAll("#bucket-repositories option").length === 2);
    assert.deepEqual(await page.locator("#bucket-refs option").evaluateAll((options) => options.map((option) => option.value)), ["release/v1", "main"]);
    assert.equal(await page.getByRole("combobox", { name: "Repository or URL", exact: true }).getAttribute("list"), "bucket-repositories");
    await shot("remote-loader");
    assert.ok(observedRemote.some((url) => url.includes("/git/trees/" + sha)));
    assert.ok(observedRemote.some((url) => url.endsWith("/raw?ref=" + sha)));
    await page.getByRole("combobox", { name: "Repository or URL", exact: true }).fill("acorn-test/bucket");
    await page.getByRole("combobox", { name: "Branch / tag / commit (optional)", exact: true }).fill("pending");
    await page.getByRole("button", { name: "Load bucket", exact: true }).click();
    await page.getByRole("button", { name: "Cancel", exact: true }).click();
    await page.getByRole("alert").filter({ hasText: /cancelled/ }).waitFor();
    if (pendingRemote) await pendingRemote.abort().catch(() => {});
    await page.getByRole("combobox", { name: "Repository or URL", exact: true }).fill("http://github.com/acorn-test/bucket");
    await page.getByRole("button", { name: "Load bucket", exact: true }).click();
    await page.getByRole("alert").filter({ hasText: /public HTTPS/ }).waitFor();
    assert.ok(await page.locator(".loaded-tree > summary").textContent().then((text) => text.includes("1 loaded files")));
    await nav("Research Activity");
    assert.equal(await page.locator("#content").inputValue(), remoteContent);
    await page.locator("#content-editor .tok-propertyName").first().waitFor();
    await page.locator("#content-editor").fill('{"safe":"<img src=x onerror=alert(1)>"}');
    await page.waitForFunction(() => document.getElementById("content-editor").textContent.includes("onerror=alert(1)"));
    assert.equal(await page.locator("#content-editor img").count(), 0);
    const longCode = JSON.stringify({ lines: Array.from({ length: 70 }, (_, index) => `line-${index}`) }, null, 2);
    await page.locator("#content-editor").fill(longCode);
    await page.locator("#content-editor").press("ControlOrMeta+End");
    await page.waitForFunction(() => document.getElementById("content-editor").textContent.includes("line-69"));
    assert.ok(await page.locator(".code-editor .cm-scroller").evaluate(node => node.scrollTop > 0));
    assert.equal(await page.locator("#content").inputValue(), longCode);
    checks.push("Mocked GitHub/GitLab loading pins revisions, cancellation and invalid URLs preserve data, editable syntax escapes markup");
    const remembered = await context.newPage();
    await remembered.goto(origin + prefix + "/home");
    await remembered.locator(".remote-entry > summary").click();
    await remembered.waitForFunction(() => document.querySelectorAll("#bucket-repositories option").length === 2);
    assert.deepEqual(await remembered.locator("#bucket-refs option").evaluateAll((options) => options.map((option) => option.value)), ["pending", "release/v1", "main"]);
    assert.equal(await remembered.locator("#bucket-repositories option").evaluateAll((options) => options.some((option) => option.value.startsWith("http:"))), false);
    await remembered.getByRole("button", { name: "Clear suggestions", exact: true }).click();
    await remembered.reload();
    await remembered.locator(".remote-entry > summary").click();
    await remembered.waitForFunction(() => Boolean(window.acornRemote));
    assert.equal(await remembered.locator("#bucket-repositories option,#bucket-refs option").count(), 0);
    assert.equal(await remembered.evaluate(() => localStorage.getItem("acorn.bucket-history.v1")), null);
    await remembered.close();
    checks.push("Bucket suggestions persist across pages and reloads, exclude rejected URLs, and can be cleared");
    const fallback = await context.newPage();
    await fallback.addInitScript(() => {
        const original = HTMLCanvasElement.prototype.getContext;
        HTMLCanvasElement.prototype.getContext = function (kind, ...args) { return /webgl/.test(kind) ? null : original.call(this, kind, ...args); };
    });
    await fallback.goto(origin + prefix + "/");
    await fallback.getByRole("button", { name: "Try Sample Data" }).click();
    await fallback.locator('nav a[title="Geospatial"]').click();
    await fallback.getByText(/map is unavailable/i).waitFor({ timeout: 60000 });
    await fallback.locator(".organization-table tbody button").first().click();
    await fallback.getByRole("dialog").getByRole("heading", { name: "Related activities", exact: true }).waitFor();
    await fallback.keyboard.press("Escape");
    await fallback.screenshot({ path: resolve(output, "geospatial-no-webgl.png"), fullPage: true });
    checks.push("Forced WebGL failure retains the selectable organization table");
    for (const tool of ["Research Activity", "Metadata standard"]) {
        const identifiers = await context.newPage();
        await identifiers.goto(origin + prefix + "/");
        await identifiers.locator(`nav a[title="${tool}"]`).click();
        await identifiers.getByRole("button", { name: "Use example", exact: true }).click();
        const content = JSON.parse(validActivity);
        const invalid = structuredClone(content);
        invalid.meta.organizations[0].identifiers[0].value = "https://ror.org/01qz5mb56xxx";
        invalid.meta.organizations[0].roles[0].organization.value = "01qz5mb55";
        await identifiers.locator("#content-editor").fill(JSON.stringify(invalid));
        await identifiers.getByRole("button", { name: actionLabels[tool], exact: true }).click();
        await identifiers.locator(".results .status.invalid").waitFor();
        assert.deepEqual(await identifiers.locator('.diagnostic-group[data-category="schema"] .diagnostics strong').allTextContents(), [
            "meta.organizations[0].identifiers[0].value",
            "meta.organizations[0].roles[0].organization.value",
        ]);
        assert.equal(await identifiers.locator(".acorn-celebration").count(), 0, `${tool} does not celebrate invalid identifiers`);
        await identifiers.locator("#content-editor").fill(JSON.stringify(content));
        await identifiers.getByRole("button", { name: actionLabels[tool], exact: true }).click();
        await identifiers.locator(".results .status:not(.invalid)").waitFor();
        await identifiers.close();
    }
    checks.push("Both validation pages reject malformed RORs and bad checksums with exact nested paths, then accept corrected values");
    for (const tool of ["Research Activity", "Metadata standard"]) {
        const validation = await context.newPage();
        await validation.goto(origin + prefix + "/");
        await validation.locator(`nav a[title="${tool}"]`).click();
        await validation.getByRole("button", { name: "Use example", exact: true }).click();
        const content = JSON.parse(validActivity);
        const invalid = structuredClone(content);
        invalid.aspect = { data: [{ Real: { license: " " } }] };
        invalid.contact.givenName = " ";
        invalid.contact.url = "https://:8080/";
        invalid.meta.languages = [{ id: "zzz" }];
        Object.assign(invalid.meta.organizations[0].roles[0], { startDate: "2026-01-02T00:00:00Z", endDate: "2026-01-01T00:00:00Z" });
        invalid.meta.infrastructure = [0, 1].map(() => ({ identifier: "duplicate", name: "Environment", platforms: [] }));
        invalid.sections.impact = [" "];
        invalid.logbook = { entries: [{ identifier: "entry", projectId: "other-project", timestamp: "2026-01-01T00:00:00Z", category: "research", state: "completed", summary: "Research complete" }] };
        await validation.locator("#content-editor").fill(JSON.stringify(invalid));
        await validation.getByRole("button", { name: actionLabels[tool], exact: true }).click();
        await validation.locator(".results .status.invalid").waitFor();
        const paths = await validation.locator(".diagnostics strong").allTextContents();
        for (const expected of ["aspect.data[0].Real.license", "contact.givenName", "contact.url", "meta.languages[0].id", "meta.organizations[0].roles[0].endDate", "meta.infrastructure[1].identifier", "sections.impact[0]", "logbook.entries[0].projectId"]) {
            assert.ok(paths.includes(expected), `${tool}: missing ${expected} in ${paths.join(", ")}`);
        }
        assert.equal(await validation.locator(".acorn-celebration").count(), 0);
        content.sections.impact = ["É".repeat(150)];
        await validation.locator("#content-editor").fill(JSON.stringify(content));
        await validation.getByRole("button", { name: actionLabels[tool], exact: true }).click();
        await validation.locator(".results .status:not(.invalid)").waitFor();
        await validation.close();
    }
    checks.push("Both validation pages enforce nested data, required text, language registry, URLs, chronology, scoped IDs and logbook ownership; Unicode prose remains valid");
    for (const reducedMotion of ["no-preference", "reduce"]) {
        const celebration = await context.newPage();
        await celebration.emulateMedia({ reducedMotion, colorScheme: reducedMotion === "reduce" ? "light" : "dark" });
        await celebration.goto(origin + prefix + "/");
        await celebration.waitForFunction(() => Boolean(window.acornCelebrate));
        await celebration.locator('nav a[title="Research Activity"]').click();
        await celebration.locator("#content-editor").fill('{"title":1}');
        await celebration.getByRole("button", { name: "Validate", exact: true }).click();
        await celebration.locator(".results .status.invalid").waitFor();
        assert.equal(await celebration.locator(".acorn-celebration").count(), 0, "Invalid data does not celebrate");
        await celebration.locator("#content-editor").fill(validActivity);
        await celebration.getByRole("button", { name: "Validate", exact: true }).click();
        const burst = celebration.locator(".acorn-celebration");
        await burst.waitFor({ state: "attached" });
        assert.equal(await burst.getAttribute("aria-hidden"), "true");
        assert.equal(await burst.evaluate((node) => getComputedStyle(node).pointerEvents), "none");
        const pieces = burst.locator(".celebration-piece");
        assert.equal(await pieces.count(), reducedMotion === "reduce" ? 4 : 48);
        assert.equal(await pieces.evaluateAll((nodes) => new Set(nodes.map((node) => node.dataset.shape)).size), 4);
        assert.equal(await pieces.evaluateAll((nodes) => new Set(nodes.map((node) => getComputedStyle(node).color)).size), reducedMotion === "reduce" ? 4 : 6);
        assert.equal(await pieces.first().evaluate((node) => getComputedStyle(node).animationName), reducedMotion === "reduce" ? "none" : "confetti-burst");
        await celebration.waitForTimeout(300);
        await celebration.screenshot({ path: resolve(output, `celebration-${reducedMotion}.png`) });
        await burst.waitFor({ state: "detached" });
        await celebration.getByRole("button", { name: "Validate", exact: true }).click();
        await celebration.locator('nav a[title="Metadata standard"]').click();
        await celebration.locator("#content-editor").fill(validActivity);
        await celebration.getByRole("button", { name: "Validate", exact: true }).click();
        await celebration.locator(".results .status:not(.invalid)").waitFor();
        await celebration.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));
        assert.equal(await burst.count(), 0, "Celebration does not repeat across tools");
        await celebration.close();
    }
    checks.push("First valid validation celebrates once per visit, cleans up, and respects reduced motion");
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
