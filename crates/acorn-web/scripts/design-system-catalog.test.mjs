import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import { test } from "node:test";
import { chromium } from "playwright";

const root = new URL("../", import.meta.url);
const page = await readFile(new URL("design-system/index.html", root), "utf8");
const catalog = JSON.parse(await readFile(new URL("design-system/components.json", root), "utf8"));

test("catalog IDs and specimens have one-to-one coverage", () => {
    assert.equal(catalog.schemaVersion, 1);
    assert.deepEqual(catalog.themes, ["dark", "light", "uswds"]);
    const ids = catalog.components.map(({ id }) => id);
    assert.equal(new Set(ids).size, ids.length);
    const sections = [...page.matchAll(/<section class="specimen" id="([a-z-]+)"/g)].map((match) => match[1]);
    assert.deepEqual(sections.sort(), [...ids].sort());
    const references = [...page.matchAll(/<a class="catalog-ref" href="#([a-z-]+)" title="([^"]+)">#([a-z-]+)<\/a>/g)].map(([, href, title, label]) => ({ href, title, label }));
    assert.deepEqual(references.map(({ href, label }) => ({ href, label })), ids.map((id) => ({ href: id, label: id })));
    for (const { title } of references) assert.match(title, /^Jump to /);
    for (const component of catalog.components) {
        assert.equal(component.anchor, `#${component.id}`);
        for (const key of ["name", "category", "source", "selector", "role", "keyboard"]) assert.ok(component[key], `${component.id}: ${key}`);
        assert.ok(component.states.length, `${component.id}: states`);
    }
});

test("catalog uses local production assets and existing source paths", async () => {
    for (const path of ["assets/app.css", "geospatial/src/map.css", "design-system/catalog.css", "design-system/components.json", "scripts/parity-smoke.mjs", "scripts/acceptance.test.mjs"]) {
        await stat(new URL(path, root));
    }
    for (const { source } of catalog.components) {
        const [path, line] = source.split(":");
        const content = await readFile(new URL(path, root), "utf8");
        assert.ok(Number(line) > 0 && Number(line) <= content.split("\n").length, source);
    }
    assert.match(page, /href="\.\.\/assets\/app\.css"/);
    assert.match(page, /href="\.\.\/geospatial\/src\/map\.css"/);
    assert.doesNotMatch(page, /(?:src|href)="https?:\/\//);
});

test("catalog renders offline at desktop and mobile widths", async () => {
    const browser = await chromium.launch();
    try {
        const page = await browser.newPage({ viewport: { width: 1200, height: 800 } });
        const errors = [];
        page.on("pageerror", (error) => errors.push(error.message));
        await page.goto(new URL("design-system/index.html", root).href);
        assert.equal(await page.locator("section.specimen").count(), catalog.components.length);
        const references = await page.locator("section.specimen .catalog-item-label").evaluateAll((labels) => labels.map((label) => ({
            id: label.closest("[id]")?.id,
            label: label.textContent.trim(),
            section: label.closest("section.specimen")?.id,
            title: label.getAttribute("title"),
        })));
        assert.ok(references.length >= 60);
        assert.equal(new Set(references.map(({ id }) => id)).size, references.length);
        assert.deepEqual(new Set(references.map(({ section }) => section)), new Set(catalog.components.map(({ id }) => id)));
        for (const { id, label, section, title } of references) {
            assert.equal(label, `#${id}`);
            assert.ok(id.startsWith(`${section}-`), id);
            assert.ok(title.endsWith(`(#${id})`), id);
        }
        const sampleSelectors = ["button", "a", "input", "select", "textarea", "table", "svg", "details", ".status", ".badge", ".diagnostic", ".sample-panel", ".sample-chart", ".sample-map"];
        const unlabeled = await page.locator(sampleSelectors.map((selector) => `section.specimen .workspace ${selector}`).join(", ")).evaluateAll((elements) => elements.filter((element) => !element.closest(".catalog-item")?.querySelector(":scope > .catalog-item-label")).map((element) => element.outerHTML.slice(0, 80)));
        assert.deepEqual(unlabeled, []);
        assert.equal(await page.locator("section.specimen details").evaluateAll((details) => details.every((item) => item.firstElementChild?.tagName === "SUMMARY")), true);
        for (const theme of catalog.themes) {
            await page.locator("#catalog-theme").selectOption(theme);
            assert.equal(await page.locator("#buttons .workspace").getAttribute("data-theme"), theme);
            assert.notEqual(await page.locator("#buttons .workspace").evaluate((element) => getComputedStyle(element).backgroundColor), "rgba(0, 0, 0, 0)");
            const expectedPrimary = theme === "uswds" ? "rgb(0, 94, 162)" : "rgb(0, 102, 44)";
            await page.waitForFunction((expected) => getComputedStyle(document.querySelector("#buttons button.primary")).backgroundColor === expected, expectedPrimary);
            if (process.env.ACORN_CATALOG_SCREENSHOT_DIR) await page.screenshot({ path: `${process.env.ACORN_CATALOG_SCREENSHOT_DIR}/catalog-${theme}.png`, fullPage: true });
        }
        await page.setViewportSize({ width: 390, height: 844 });
        assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), true);
        assert.deepEqual(errors, []);
    } finally {
        await browser.close();
    }
});
