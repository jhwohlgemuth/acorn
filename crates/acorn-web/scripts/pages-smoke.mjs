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
    try {
        await page.waitForFunction(() => {
            const status = document.getElementById("webmcp-status")?.textContent;
            const editors = !document.querySelector(".code-editor") || document.querySelector(".cm-content");
            return window.acornSyntax && editors && status && status !== "WebMCP: initializing";
        });
    } catch (error) {
        const state = await page.evaluate(() => ({
            title: document.title,
            syntax: Boolean(window.acornSyntax),
            editor: Boolean(document.querySelector(".cm-content")),
            codeEditor: Boolean(document.querySelector(".code-editor")),
            webmcp: document.getElementById("webmcp-status")?.textContent,
        }));
        throw new Error(`Browser services did not initialize: ${JSON.stringify({ state, failures })}`, { cause: error });
    }
    assert.match(await page.locator("#webmcp-status").textContent(), /^WebMCP: (installed|unsupported)$/);
}
const browser = await chromium.launch().catch((error) => { server.close(); throw error; });
try {
    const failedStylesPage = await browser.newPage();
    await failedStylesPage.route(/\/assets\/app(?:-dxh[^/]+)?\.css$/, (route) => route.abort());
    await failedStylesPage.goto(`${origin}${prefix}/`, { waitUntil: "domcontentloaded" });
    await failedStylesPage.getByRole("button", { name: "Retry" }).waitFor();
    assert.equal(await failedStylesPage.locator("#boot").getAttribute("role"), "alert");
    assert.equal(await failedStylesPage.locator("#boot").getAttribute("aria-label"), null, "The loading label is removed when startup fails");
    assert.equal(await failedStylesPage.locator("#main").getAttribute("data-styles-ready"), null);
    await failedStylesPage.close();
    const page = await browser.newPage({ colorScheme: "light" });
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
    await page.getByRole("heading", { name: "Research Intelligence" }).waitFor();
    assert.equal(await page.locator(".workspace").getAttribute("data-theme"), "light", "The first rendered workspace follows the system theme");
    assert.equal(await page.locator("#boot").getAttribute("aria-label"), "Loading ACORN Research Intelligence");
    assert.equal(await page.locator("#boot span").textContent(), "Plant an acorn, grow your science");
    await page.waitForFunction(() => { const image = document.querySelector("#boot img"); return image && image.complete && image.naturalWidth > 0; });
    await page.locator("#boot").waitFor({ state: "hidden" });
    await page.waitForFunction(() => { const image = document.querySelector(".home-banner img"); return image?.complete && image.naturalWidth > 0; });
    await page.waitForFunction(() => getComputedStyle(document.querySelector(".sidebar")).position !== "static");
    await page.getByRole("button", { name: "Collapse sidebar" }).click();
    for (const [title, heading] of [["Review", "Review"], ["Install", "Install ACORN Research Intelligence"], ["Home", "Research Intelligence"]]) {
        await page.locator(`#workspace-navigation nav a[title="${title}"] svg`).click();
        await page.getByRole("heading", { name: heading, exact: true }).waitFor();
        await page.waitForTimeout(200);
        assert.ok(await page.locator(".workspace").evaluate((workspace) => workspace.classList.contains("compact")), `${title} leaves the sidebar collapsed`);
        assert.equal(await page.locator(".sidebar").evaluate((sidebar) => sidebar.getBoundingClientRect().width), 62, `${title} keeps the sidebar narrow`);
        assert.equal(await page.evaluate(() => window.acornInitialSidebarCollapsed), true, `${title} keeps the shared sidebar state collapsed`);
    }
    await page.locator("#skill-entry").click();
    await page.getByRole("heading", { name: "ACORN Skill", exact: true }).waitFor();
    assert.equal(await page.locator(".sidebar").evaluate((sidebar) => sidebar.getBoundingClientRect().width), 62, "In-page navigation keeps the sidebar narrow");
    await page.locator('#workspace-navigation nav a[title="Home"]').click();
    await page.getByRole("heading", { name: "Research Intelligence" }).waitFor();
    await page.reload();
    await page.getByRole("heading", { name: "Research Intelligence" }).waitFor();
    assert.ok(await page.locator(".workspace").evaluate((workspace) => workspace.classList.contains("compact")), "Refreshing a page keeps compact navigation available");
    await page.locator('#workspace-navigation nav a[title="Review"] svg').click();
    await page.getByRole("heading", { name: "Review", exact: true }).waitFor();
    assert.equal(await page.locator(".sidebar").evaluate((sidebar) => sidebar.getBoundingClientRect().width), 62, "Icon navigation after reload keeps the sidebar narrow");
    await page.locator('#workspace-navigation nav a[title="Home"] svg').click();
    await page.getByRole("heading", { name: "Research Intelligence" }).waitFor();
    await page.getByRole("button", { name: "Expand sidebar" }).click();
    await page.getByRole("button", { name: "Try Sample Data" }).click();
    await page.getByRole("heading", { name: "Portfolio", exact: true }).waitFor();
    assert.equal(await page.getByRole("heading", { name: "Activity types" }).count(), 0);
    assert.equal(await page.getByRole("heading", { name: "Top topics" }).count(), 1);
    assert.equal(await page.locator(".topic-bars .proportion").count(), 6);
    const topicTile = await page.locator(".topic-bars").boundingBox();
    const donutTiles = await page.locator(".distribution-stack").boundingBox();
    assert.ok(topicTile && donutTiles && Math.abs(topicTile.height - donutTiles.height) < 1, "Top topics matches the combined donut tile height");
    assert.equal(await page.locator(".status-donut .donut-legend-row").count(), 4);
    assert.match(await page.locator(".status-donut .donut-chart").evaluate((donut) => getComputedStyle(donut).backgroundImage), /^conic-gradient\(/);
    assert.equal(await page.locator(".metrics .metric").count(), 3);
    assert.deepEqual(await page.locator(".identifier-donut .donut-legend-label").allTextContents(), ["DOI", "ORCID", "ROR"]);
    assert.match(await page.locator(".identifier-donut .donut-chart").evaluate((donut) => getComputedStyle(donut).backgroundImage), /^conic-gradient\(/);
    assert.equal(Number(await page.locator(".identifier-donut .donut-center strong").textContent()),
        (await page.locator(".identifier-donut .donut-legend-row strong").allTextContents()).map(Number).reduce((sum, count) => sum + count, 0));
    await page.locator(".activity-select").first().click();
    const drawer = page.getByRole("dialog");
    await drawer.waitFor();
    await drawer.getByRole("tab", { name: "Preview" }).waitFor();
    assert.equal(await drawer.getByRole("button", { name: /View project:/ }).count(), 0);
    await drawer.getByRole("button", { name: "Open in Review", exact: true }).click();
    await drawer.waitFor({ state: "hidden" });
    await page.getByRole("heading", { name: "Review", exact: true }).waitFor();
    await waitForServices(page);
    assert.equal(new URL(page.url()).pathname, `${prefix}/review`);
    assert.deepEqual(await page.locator("#workspace-navigation nav > .nav-heading").allTextContents(), ["WORKSPACE", "CONVERT", "ANALYZE", "APPLICATION"]);
    assert.deepEqual(await page.locator("#workspace-navigation nav > a.nav-item .nav-label").evaluateAll((labels) => labels.slice(0, 5).map((label) => label.textContent)), ["Home", "Portfolio", "Review", "Discover", "Inspect"]);
    const editorLicense = await page.locator('link[title="CodeMirror dependency licenses"]').first().getAttribute("href");
    assert.ok(editorLicense, "The bundled editor exposes its dependency licenses");
    const licenseResponse = await page.request.get(new URL(editorLicense, page.url()).href);
    assert.equal(licenseResponse.status(), 200, "Editor licenses load under the deployment prefix");
    assert.match(await licenseResponse.text(), /@codemirror\//);
    await page.getByRole("button", { name: "Validate", exact: true }).click();
    await page.getByRole("button", { name: "Copy result", exact: true }).waitFor();
    const reviewInput = await page.locator("#content").inputValue();
    assert.equal(await page.getByRole("group", { name: "Review type" }).getByRole("button").count(), 3);
    for (const mode of ["FAIR assessment", "Readability", "Metadata"]) {
        await page.getByRole("button", { name: mode, exact: true }).click();
        await page.getByText("Ready when you are").waitFor();
        assert.equal(await page.locator("#content").inputValue(), reviewInput, `${mode} preserves the review input`);
        let fragment = "";
        if (mode === "FAIR assessment") fragment = "#fair";
        else if (mode === "Readability") fragment = "#readability";
        await page.waitForFunction((expected) => location.hash === expected, fragment);
    }
    await page.getByRole("button", { name: "FAIR assessment", exact: true }).click();
    await page.waitForFunction(() => location.hash === "#fair");
    await page.reload();
    await waitForServices(page);
    await page.getByRole("button", { name: "FAIR assessment", exact: true, pressed: true }).waitFor();
    await page.getByRole("button", { name: "Readability", exact: true }).click();
    await page.waitForFunction(() => location.hash === "#readability");
    await page.reload();
    await waitForServices(page);
    await page.getByRole("button", { name: "Readability", exact: true, pressed: true }).waitFor();
    await page.goBack();
    await page.getByRole("button", { name: "FAIR assessment", exact: true, pressed: true }).waitFor();
    await page.getByRole("button", { name: "Metadata", exact: true }).click();
    await page.waitForFunction(() => location.hash === "");
    await page.getByRole("button", { name: /Use example/ }).click();
    const toolSettings = page.locator(".tool-settings");
    assert.equal(await toolSettings.getAttribute("open"), null, "Tool settings start collapsed");
    await toolSettings.locator("summary").first().click();
    const settingLabels = () => toolSettings.locator(".options > .field > span, .options > .check-settings-row legend, .options > .vocabulary-field legend").allTextContents();
    assert.deepEqual(await settingLabels(), ["Standard", "Format", "Skip checks", "Vocabulary matching"]);
    const standardSelect = toolSettings.getByRole("combobox", { name: "Standard", exact: true });
    await standardSelect.selectOption("datacite");
    assert.equal(await page.locator(".vocabulary-choices").count(), 0, "Other standards hide RAD-specific controls");
    assert.deepEqual(await settingLabels(), ["Standard", "Format"]);
    await standardSelect.selectOption("research-activity-data");
    assert.deepEqual(await settingLabels(), ["Standard", "Format", "Skip checks", "Vocabulary matching"]);
    for (const width of [1280, 320]) {
        await page.setViewportSize({ width, height: 720 });
        const buttons = await page.locator(".vocabulary-choices button").evaluateAll((items) => items.map((item) => item.getBoundingClientRect()));
        assert.equal(buttons.length, 3, "Research Activity shows all vocabulary options");
        assert.ok(buttons.every((button) => button.top === buttons[0].top), `Vocabulary options stay on one line at ${width}px`);
        assert.ok(buttons.every((button) => button.right <= width), `Vocabulary options fit the viewport at ${width}px`);
    }
    await page.setViewportSize({ width: 1280, height: 720 });
    const metadataStandardWidth = await standardSelect.evaluate((select) => select.getBoundingClientRect().width);
    const readabilityControls = page.locator(".activity-readability-controls");
    const readabilitySkip = page.getByRole("group", { name: "Skip check types" }).getByRole("button", { name: "Readability", exact: true });
    const skipBox = await readabilitySkip.boundingBox();
    const metricBox = await readabilityControls.getByRole("combobox").boundingBox();
    const thresholdBox = await readabilityControls.getByLabel("Maximum readability score").boundingBox();
    const checkRowBox = await page.locator(".check-settings-row").boundingBox();
    assert.ok(Math.abs(skipBox.y - metricBox.y) <= 4, `Readability settings align with the skip controls (${JSON.stringify({ skipBox, metricBox, checkRowBox })})`);
    assert.equal(metricBox.y, thresholdBox.y, "Readability metric and threshold share a row");
    await readabilityControls.getByRole("combobox").selectOption("fres");
    await readabilityControls.getByLabel("Minimum readability score").fill("70");
    const validationTypes = page.getByRole("group", { name: "Skip check types" });
    assert.equal(await validationTypes.locator("button .nav-icon").count(), 5);
    assert.ok((await validationTypes.getByRole("button", { name: "FAIR", exact: true }).boundingBox()).height <= 32);
    const settingsHeader = toolSettings.locator(":scope > summary");
    const headerHeight = (await settingsHeader.boundingBox()).height;
    await validationTypes.getByRole("button", { name: "FAIR", exact: true }).click();
    assert.equal((await settingsHeader.boundingBox()).height, headerHeight, "Skip count does not change Settings height");
    await page.getByRole("button", { name: "Validate", exact: true }).click();
    await page.getByText("Skipped: fair").waitFor();
    assert.equal(await page.locator('.diagnostic-group[data-category="fair"]').count(), 0);
    assert.ok(await page.locator('.diagnostic-group[data-category="readability"]').count());
    await page.locator('.diagnostic-group[data-category="readability"]').getByText(/FRES = .*\/70/).waitFor();
    await validationTypes.getByRole("button", { name: "Readability", exact: true }).click();
    assert.equal((await settingsHeader.boundingBox()).height, headerHeight, "Additional skipped checks keep Settings height");
    assert.equal(await readabilityControls.count(), 0);
    await page.getByRole("button", { name: "Validate", exact: true }).click();
    await page.getByText("Skipped: fair, readability").waitFor();
    assert.equal(await page.locator('.diagnostic-group[data-category="readability"]').count(), 0);
    await validationTypes.getByRole("button", { name: /^All/ }).click();
    await page.getByRole("button", { name: "Validate", exact: true }).click();
    await page.getByText("Schema not checked").waitFor();
    await page.getByText("No diagnostics from the selected checks.").waitFor();
    await validationTypes.getByRole("button", { name: /^All/ }).click();
    assert.equal(await validationTypes.getByRole("button", { name: /^All/ }).getAttribute("aria-pressed"), "false");
    await page.getByRole("button", { name: "FAIR assessment", exact: true }).click();
    await page.getByRole("button", { name: /Use example/ }).click();
    if (await page.locator(".tool-settings").getAttribute("open") === null) await page.locator(".tool-settings > summary").click();
    const fairStandardWidth = await page.getByRole("combobox", { name: "Standard", exact: true }).evaluate((select) => select.getBoundingClientRect().width);
    assert.ok(Math.abs(metadataStandardWidth - fairStandardWidth) <= 1, "Standard control keeps the same width across Review tabs");
    const fairTypes = page.getByRole("group", { name: "Skip check types" });
    await fairTypes.getByRole("button", { name: /^All/ }).click();
    await page.getByRole("button", { name: "Assess FAIR", exact: true }).click();
    await page.getByText("No FAIR checks were run.").waitFor();
    assert.equal(await page.locator('.diagnostic-group[data-category="fair"] .diagnostic').count(), 0);
    await fairTypes.getByRole("button", { name: /^All/ }).click();
    await page.getByRole("button", { name: "Metadata", exact: true }).click();
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
    await page.getByRole("button", { name: "Readability", exact: true, pressed: true }).waitFor();
    await page.reload();
    await waitForServices(page);
    await page.getByRole("button", { name: "Readability", exact: true, pressed: true }).waitFor();
    for (const legacy of ["metadata", "research-activity", "metadata-standard"]) {
        await page.goto(`${origin}${prefix}/${legacy}`);
        await page.getByRole("heading", { name: "Review", exact: true }).waitFor();
        await waitForServices(page);
        assert.equal(await page.getByRole("group", { name: "Review type" }).getByRole("button").count(), 3);
    }
    await page.goto(`${origin}${prefix}/review#readability`);
    await waitForServices(page);
    await page.getByRole("button", { name: "Readability", exact: true, pressed: true }).waitFor();
    assert.equal(new URL(page.url()).hash, "#readability", "Direct Review links retain their tab fragment");
    for (const [name, slug] of [["ASPECT", "aspect"], ["S3A", "s3a"]]) {
        await page.locator(`nav a[title="${name}"]`).click();
        await page.getByRole("heading", { name, exact: true }).waitFor();
        await page.getByRole("heading", { name: "Under construction" }).waitFor();
        const constructionGif = page.locator(`.page[data-page="${slug}"] .construction-gif`);
        assert.equal(await constructionGif.getAttribute("alt"), "My acorn is missing");
        await constructionGif.evaluate((image) => image.decode());
        assert.equal(await constructionGif.evaluate((image) => image.naturalWidth), 400);
        assert.equal(await constructionGif.evaluate((image) => getComputedStyle(image).objectFit), "cover");
        assert.equal(Math.round((await constructionGif.boundingBox()).width), await page.locator(`.page[data-page="${slug}"] .construction-card`).evaluate((card) => card.clientWidth));
        if (slug === "aspect") {
            assert.equal(await page.getByRole("heading", { name: "ASPECT", exact: true }).getByRole("link").count(), 0);
            assert.equal(await page.locator('.page[data-page="aspect"] .card').getByRole("link", { name: "ASPECT" }).getAttribute("href"), "https://acorn.ornl.gov/concepts/aspect/index.html");
        }
        assert.equal(new URL(page.url()).pathname, `${prefix}/${slug}`);
        await page.reload();
        await waitForServices(page);
        await page.getByRole("heading", { name: "Under construction" }).waitFor();
        assert.equal(await page.locator(".page[data-page] p").filter({ hasText: `This page will visualize ${name} data.` }).count(), 1);
    }
    await page.locator('nav a[title="Geospatial"]').click();
    await page.locator("#geospatial-map canvas").waitFor({ timeout: 60000 });
    const layerButton = page.locator(".map-layer-widget > summary");
    const layerButtonBox = await layerButton.boundingBox();
    assert.ok(layerButtonBox && layerButtonBox.width <= 32 && layerButtonBox.height <= 36, "Layer selector matches the compact Cesium controls");
    const layerIconBox = await layerButton.locator("svg").boundingBox();
    assert.ok(layerIconBox && layerIconBox.width >= 23 && layerIconBox.height >= 23, "Layer icon matches the visual scale of the Cesium controls");
    assert.ok(layerIconBox && Math.abs(layerIconBox.x + layerIconBox.width / 2 - layerButtonBox.x - layerButtonBox.width / 2) <= 1 && Math.abs(layerIconBox.y + layerIconBox.height / 2 - layerButtonBox.y - layerButtonBox.height / 2) <= 1, "Layer icon is centered within its control");
    const homeButtonBox = await page.locator(".cesium-home-button").boundingBox();
    const perspectiveButtonBox = await page.locator(".cesium-sceneModePicker-button3D").boundingBox();
    assert.ok(homeButtonBox && perspectiveButtonBox && layerButtonBox.height === homeButtonBox.height && layerButtonBox.height === perspectiveButtonBox.height, `Layers, Home and Perspective controls have equal rendered heights (${layerButtonBox.height}, ${homeButtonBox?.height}, ${perspectiveButtonBox?.height})`);
    assert.ok(homeButtonBox && homeButtonBox.x - (layerButtonBox.x + layerButtonBox.width) >= 6, "Layer selector is separated from the Home control");
    assert.match(await layerButton.getAttribute("aria-label"), /^Map layers: /);
    await layerButton.click();
    await page.locator(".map-basemap input").first().waitFor();
    assert.equal(await page.locator(".map-basemap input").count(), 4, "Every basemap is directly discoverable");
    await page.locator("#geospatial-map").screenshot({ path: resolve(root, "geospatial-layer-control.png") });
    await page.getByText("Natural Earth II", { exact: true }).click();
    assert.equal(await layerButton.getAttribute("aria-label"), "Map layers: Natural Earth II");
    await page.locator('nav a[title="Install"]').click();
    await page.getByRole("heading", { name: "Desktop channels" }).waitFor();
    assert.deepEqual(await page.locator(".channel-mark").evaluateAll((marks) => marks.map((mark) => mark.dataset.brand)), ["windows", "windows", "debian", "appimage", "homebrew"]);
    await page.waitForFunction(() => [...document.querySelectorAll(".channel-mark img")].every((image) => image.complete && image.naturalWidth > 0));
    for (const title of ["Devicon icon license", "Simple Icons license"]) {
        const href = await page.locator(`link[title="${title}"]`).getAttribute("href");
        assert.equal((await page.request.get(new URL(href, page.url()).href)).status(), 200, `${title} is bundled`);
    }
    await page.setViewportSize({ width: 320, height: 720 });
    assert.ok(await page.locator(".channel-mark").evaluateAll((marks) => marks.every((mark) => mark.getBoundingClientRect().right <= innerWidth)), "Channel marks fit a narrow viewport");
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.locator('nav a[title="Home"]').click();
    await page.getByRole("heading", { name: "Research Intelligence" }).waitFor();
    assert.equal(new URL(page.url()).pathname, `${prefix}/`);
    const contentPage = await browser.newPage();
    await contentPage.goto(`${origin}${prefix}/`);
    await waitForServices(contentPage);
    await contentPage.getByRole("button", { name: "Try Sample Data" }).click();
    await contentPage.getByRole("heading", { name: "Portfolio", exact: true }).waitFor();
    const contentTree = contentPage.locator(".loaded-tree");
    const selectedBeforeImage = await contentTree.locator('.file-leaf[aria-pressed="true"]').allTextContents();
    assert.equal(selectedBeforeImage.length, 1, "A research activity is selected before the image preview");
    await contentTree.locator(":scope > summary").click();
    await contentTree.locator(".file-tree details > summary").filter({ hasText: "amaterasu-mirror-garden" }).click();
    await contentTree.locator(".file-leaf").filter({ hasText: "00.svg" }).first().click();
    await contentPage.getByRole("dialog").locator(".details-standalone-image").waitFor();
    assert.deepEqual(await contentTree.locator('.file-leaf[aria-pressed="true"]').allTextContents(), selectedBeforeImage, "Image preview keeps the selected activity");
    await contentPage.getByRole("button", { name: "Close details" }).click();
    await contentPage.locator('nav a[title="Review"]').click();
    await contentPage.getByRole("heading", { name: "Review", exact: true }).waitFor();
    await contentPage.locator(".tool-settings > summary").click();
    const formatSelect = contentPage.getByRole("combobox", { name: "Format", exact: true });
    const reviewTree = contentPage.locator(".loaded-tree");
    await reviewTree.locator(":scope > summary").click();
    for (const [folder, file, format] of [["bamboo-shoots-after-rain", "index.md", "markdown"], ["kitsune-lantern-atlas", "index.yaml", "yaml"]]) {
        const folderSummary = reviewTree.locator(".file-tree details > summary").filter({ hasText: folder });
        await folderSummary.click();
        await folderSummary.locator("..").locator(":scope > .file-tree > li > .file-leaf").filter({ hasText: file }).click();
        assert.equal(await formatSelect.inputValue(), format, `${file} selects its format`);
    }
    await contentPage.close();
    const themePage = await browser.newPage({ colorScheme: "light" });
    await themePage.goto(`${origin}${prefix}/settings`);
    await themePage.getByRole("heading", { name: "Settings", exact: true }).waitFor();
    assert.equal(await themePage.getByRole("button", { name: "System", exact: true }).getAttribute("aria-pressed"), "true", "System is the initial theme choice");
    await themePage.getByRole("button", { name: "Dark", exact: true }).click();
    assert.equal(await themePage.evaluate(() => localStorage.getItem("acorn.theme.v1")), "dark");
    await themePage.reload();
    await themePage.getByRole("heading", { name: "Settings", exact: true }).waitFor();
    assert.equal(await themePage.locator(".workspace").getAttribute("data-theme"), "dark", "A saved theme is applied on reload");
    assert.equal(await themePage.getByRole("button", { name: "Dark", exact: true }).getAttribute("aria-pressed"), "true");
    await themePage.getByRole("button", { name: "Collapse sidebar" }).click();
    await themePage.getByRole("button", { name: "Clear saved settings" }).click();
    assert.equal(await themePage.evaluate(() => localStorage.getItem("acorn.theme.v1")), null);
    assert.equal(await themePage.evaluate(() => sessionStorage.getItem("acorn-sidebar-collapsed")), null);
    assert.equal(await themePage.getByRole("button", { name: "System", exact: true }).getAttribute("aria-pressed"), "true");
    assert.equal(await themePage.locator(".workspace").getAttribute("data-theme"), "light");
    await themePage.getByRole("button", { name: "Collapse sidebar" }).waitFor();
    await themePage.reload();
    await themePage.getByRole("heading", { name: "Settings", exact: true }).waitFor();
    assert.equal(await themePage.getByRole("button", { name: "System", exact: true }).getAttribute("aria-pressed"), "true", "Clearing settings restores the System default after reload");
    await themePage.close();
    const previewPage = await browser.newPage({ colorScheme: "dark" });
    await previewPage.goto(`${origin}${prefix}/settings?preview-loader=1&theme=light`);
    await waitForServices(previewPage);
    assert.equal(await previewPage.getByRole("status", { name: "Loading ACORN Research Intelligence" }).count(), 1, "The preview announces that startup is in progress");
    assert.equal(await previewPage.locator("#boot").isVisible(), true, "Loader preview stays visible after the app is ready");
    assert.equal(await previewPage.locator("#main").evaluate((main) => getComputedStyle(main).visibility), "hidden");
    assert.equal(await previewPage.locator(".workspace").getAttribute("data-theme"), "light", "URL theme overrides the dark system preference");
    assert.equal(await previewPage.locator("#boot").evaluate((boot) => getComputedStyle(boot).backgroundColor), "rgb(247, 248, 250)");
    assert.equal(await previewPage.evaluate(() => localStorage.getItem("acorn.theme.v1")), null, "URL theme is not saved");
    assert.equal(await previewPage.locator("#boot span").textContent(), "Plant an acorn, grow your science");
    await previewPage.waitForFunction(() => { const image = document.querySelector("#boot img"); return image && image.complete && image.naturalWidth > 0; });
    await previewPage.evaluate(() => localStorage.setItem("acorn.theme.v1", "light"));
    await previewPage.goto(`${origin}${prefix}/settings?preview-loader=1&theme=dark`);
    await waitForServices(previewPage);
    assert.equal(await previewPage.locator(".workspace").getAttribute("data-theme"), "dark", "URL theme overrides the saved light preference");
    assert.equal(await previewPage.locator("#boot").evaluate((boot) => getComputedStyle(boot).backgroundColor), "rgb(31, 34, 41)");
    assert.equal(await previewPage.evaluate(() => localStorage.getItem("acorn.theme.v1")), "light", "URL theme leaves the saved preference unchanged");
    await previewPage.goto(`${origin}${prefix}/settings`);
    await previewPage.getByRole("heading", { name: "Settings", exact: true }).waitFor();
    await previewPage.locator("#boot").waitFor({ state: "hidden" });
    assert.equal(await previewPage.locator(".workspace").getAttribute("data-theme"), "light", "Removing the override restores the saved preference");
    await previewPage.close();
    assert.deepEqual(failures, []);
    console.log(`Pages browser smoke passed at ${prefix || "/"}: assets, WASM operation, navigation and direct-route refresh.`);
} finally {
    await browser.close();
    await new Promise((closed) => server.close(closed));
}
