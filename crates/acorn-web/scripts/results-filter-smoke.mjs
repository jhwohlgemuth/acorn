import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, resolve, sep } from "node:path";
import { chromium } from "playwright";

const root = resolve(process.argv[2] || "target/dx/acorn-web/release/web/public");
const types = { ".css": "text/css", ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".svg": "image/svg+xml", ".wasm": "application/wasm" };
const server = createServer(async (request, response) => {
    const path = resolve(root, "." + new URL(request.url, "http://localhost").pathname);
    if (path !== root && !path.startsWith(root + sep)) { response.writeHead(404); response.end(); return; }
    const file = await stat(path).then(info => info.isDirectory() ? resolve(path, "index.html") : path).catch(() => path);
    const bytes = await readFile(file).catch(() => null);
    response.writeHead(bytes ? 200 : 404, { "Content-Type": bytes ? types[extname(file)] || "application/octet-stream" : "text/html" });
    response.end(bytes || await readFile(resolve(root, "index.html")));
});
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
const browser = await chromium.launch();
try {
    const page = await browser.newPage({ reducedMotion: "reduce" });
    if (process.env.ACORN_REVIEW_CSS) await page.route(/\/app[^/]*\.css$/, async route => route.fulfill({ contentType: "text/css", body: await readFile(process.env.ACORN_REVIEW_CSS, "utf8") }));
    page.on("pageerror", error => console.error("Browser error:", error.message));
    page.on("response", response => { if (response.status() >= 400) console.error("HTTP error:", response.status(), response.url()); });
    await page.goto(`http://127.0.0.1:${server.address().port}`);
    await page.locator('nav a[title="Review"]').waitFor({ timeout: 10000 }).catch(async error => {
        console.error("App state:", await page.locator("body").innerText());
        throw error;
    });
    for (const [tool, action, label] of [
        ["Metadata", "Validate", "Show diagnostics by severity"],
        ["FAIR assessment", "Assess FAIR", "Show FAIR findings by severity"],
    ]) {
        await page.locator('nav a[title="Review"]').click();
        await page.getByRole("group", { name: "Review type" }).getByRole("button", { name: tool, exact: true }).click();
        await page.getByRole("button", { name: "Use example", exact: true }).click();
        await page.getByRole("button", { name: action, exact: true }).click();
        const settings = page.locator(".results .result-settings").first();
        assert.equal(await settings.getAttribute("open"), null, `${tool}: Settings start collapsed`);
        await settings.locator("summary").click();
        const filters = page.getByRole("group", { name: label });
        await filters.waitFor();
        if (tool === "FAIR assessment") {
            const positions = await filters.evaluate(node => ({
                showTop: node.getBoundingClientRect().top,
                searchBottom: node.parentElement.querySelector(".search-control").getBoundingClientRect().bottom,
            }));
            assert.ok(positions.showTop >= positions.searchBottom, "FAIR Show controls start below the search field");
        }
        const all = filters.getByRole("button", { name: /^All/ });
        const selectedStyle = node => {
            const style = getComputedStyle(node);
            return { background: style.backgroundColor, border: style.borderColor, shadow: style.boxShadow, transform: style.transform };
        };
        const layout = await filters.evaluate(node => {
            const sort = node.closest(".result-controls").nextElementSibling;
            const filterButton = node.querySelector("button");
            const sortButton = sort.querySelector("button");
            return {
                filterFont: getComputedStyle(node.querySelector(".muted")).fontSize,
                sortFont: getComputedStyle(sort.querySelector(".muted")).fontSize,
                filterHeight: filterButton.getBoundingClientRect().height,
                sortHeight: sortButton.getBoundingClientRect().height,
                gap: sort.getBoundingClientRect().top - node.closest(".result-controls").getBoundingClientRect().bottom,
            };
        });
        assert.equal(layout.filterFont, layout.sortFont, `${tool}: Filter and Sort labels match`);
        assert.equal(layout.filterHeight, layout.sortHeight, `${tool}: Filter and Sort buttons match`);
        assert.ok(layout.gap <= 16, `${tool}: Filter and Sort rows stay close (${layout.gap}px)`);
        const diagnostics = page.locator(".diagnostics .diagnostic");
        const initial = await diagnostics.count();
        assert.ok(initial > 1, `${tool}: expected multiple findings`);
        const summary = page.locator(".results .result-summary").first();
        const summaryBox = await summary.evaluate((element) => ({ gap: element.getBoundingClientRect().top - element.previousElementSibling.getBoundingClientRect().bottom, height: element.getBoundingClientRect().height }));
        assert.ok(summaryBox.height > 0, `${tool}: result count reserves space before filtering`);
        assert.equal(await summary.evaluate((element) => getComputedStyle(element).visibility), "hidden");
        assert.equal(await all.getAttribute("aria-pressed"), "true");
        const appearance = await all.evaluate(selectedStyle);
        assert.equal(appearance.border, "rgb(23, 109, 57)", `${tool}: selected toggle has a green border`);
        assert.match(appearance.shadow, /inset/);
        assert.equal(appearance.transform, "matrix(1, 0, 0, 1, 2, 2)");
        const sort = page.getByRole("group", { name: "Sort diagnostics" }).getByRole("button", { name: "Severity", exact: true });
        await sort.click();
        assert.equal(await sort.evaluate((button) => getComputedStyle(button).borderColor), await all.evaluate((button) => getComputedStyle(button).borderColor), `${tool}: selected Sort and Filter have the same border`);
        assert.equal(await sort.evaluate((button) => getComputedStyle(button).borderWidth), "1px", `${tool}: selected Sort has a single border`);
        assert.equal(await sort.evaluate((button) => getComputedStyle(button).outlineStyle), "none", `${tool}: selected Sort has no persistent outline`);
        assert.match((await sort.evaluate(selectedStyle)).shadow, /inset/);
        const review = page.getByRole("group", { name: "Review type" }).getByRole("button", { pressed: true });
        assert.match((await review.evaluate(selectedStyle)).shadow, /inset/);
        const regex = page.getByRole("button", { name: "Regular expression" });
        const caseSensitive = page.getByRole("button", { name: "Case sensitive" });
        const inlinePosition = await caseSensitive.evaluate((button) => button.getBoundingClientRect().left);
        const unselectedRegex = await regex.evaluate((button) => ({ color: getComputedStyle(button).color, weight: getComputedStyle(button).fontWeight }));
        assert.equal(unselectedRegex.weight, "400");
        assert.equal(await regex.locator(".search-mode-check").isVisible(), false);
        await regex.click();
        assert.equal(await regex.getAttribute("aria-pressed"), "true");
        assert.equal(await regex.locator(".search-mode-check").isVisible(), true);
        assert.equal(await regex.evaluate((button) => getComputedStyle(button).fontWeight), "700");
        assert.notEqual(await regex.evaluate((button) => getComputedStyle(button).color), unselectedRegex.color);
        assert.equal(await regex.evaluate((button) => getComputedStyle(button).boxShadow), "none");
        assert.equal(await caseSensitive.evaluate((button) => button.getBoundingClientRect().left), inlinePosition, "Inline search controls stay in place when selected");
        await caseSensitive.click();
        assert.equal(await regex.getAttribute("aria-pressed"), "false", "Selecting Aa turns Regex off");
        assert.equal(await caseSensitive.getAttribute("aria-pressed"), "true");
        assert.equal(await caseSensitive.evaluate((button) => getComputedStyle(button).fontWeight), "700");
        assert.equal(await caseSensitive.locator(".search-mode-check").isVisible(), true);
        await regex.click();
        assert.equal(await regex.getAttribute("aria-pressed"), "true");
        assert.equal(await caseSensitive.getAttribute("aria-pressed"), "false", "Selecting Regex turns Aa off");
        await regex.click();
        await page.locator("#theme-toggle").click();
        const darkAppearance = await all.evaluate(selectedStyle);
        assert.equal(darkAppearance.border, "rgb(152, 195, 121)", `${tool}: dark theme keeps a green selected border`);
        assert.equal(await sort.evaluate((button) => getComputedStyle(button).borderColor), darkAppearance.border);
        await page.locator("#theme-toggle").click();
        await all.click();
        assert.equal(await diagnostics.count(), 0, `${tool}: All hides findings`);
        assert.equal(await summary.evaluate((element) => getComputedStyle(element).visibility), "visible");
        const filteredSummaryBox = await summary.evaluate((element) => ({ gap: element.getBoundingClientRect().top - element.previousElementSibling.getBoundingClientRect().bottom, height: element.getBoundingClientRect().height }));
        assert.equal(filteredSummaryBox.gap, summaryBox.gap, `${tool}: filtering keeps the gap below Settings`);
        assert.equal(filteredSummaryBox.height, summaryBox.height, `${tool}: filtering keeps the count row height`);
        await all.click();
        assert.equal(await diagnostics.count(), initial, `${tool}: All restores findings`);
        assert.equal(await summary.evaluate((element) => getComputedStyle(element).visibility), "hidden");
        await all.click();
        for (const button of (await filters.getByRole("button").all()).slice(1)) await button.click();
        assert.equal(await all.getAttribute("aria-pressed"), "true", `${tool}: reselecting every available severity restores All`);
        assert.equal(await diagnostics.count(), initial);
        const error = filters.getByRole("button", { name: /^Error/ });
        if (await error.count()) {
            await error.click();
            assert.equal(await page.locator('.diagnostics .diagnostic[data-tone="danger"]').count(), 0, `${tool}: Error filter hides errors`);
            await error.click();
            assert.equal(await diagnostics.count(), initial, `${tool}: Error filter restores errors`);
        }
    }
    console.log("Results severity filters and pressed toggle styles passed in Chromium.");
} finally {
    await browser.close();
    await new Promise(resolve => server.close(resolve));
}
