import assert from "node:assert/strict";
import { readFile, writeFile } from "node:fs/promises";
import { chromium } from "playwright";

const origin = process.argv[2] || "http://127.0.0.1:8084";
const configured = new URL(origin);
for (const hostname of ["127.0.0.1", "localhost"]) {
    const allowed = new URL(configured);
    allowed.hostname = hostname;
    const response = await fetch("http://127.0.0.1:8083/src/main.tsx", { headers: { Origin: allowed.origin } });
    assert.equal(response.headers.get("access-control-allow-origin"), allowed.origin);
}
for (const denied of ["https://example.org", `http://127.0.0.1:${Number(configured.port) + 1}`]) {
    const response = await fetch("http://127.0.0.1:8083/src/main.tsx", { headers: { Origin: denied } });
    assert.equal(response.headers.get("access-control-allow-origin"), null);
}
const mapPath = "crates/acorn-web/geospatial/src/map.tsx";
const cssPath = "crates/acorn-web/geospatial/src/map.css";
const originalMap = await readFile(mapPath, "utf8");
const originalCss = await readFile(cssPath, "utf8");
const browser = await chromium.launch({ args: ["--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader"] });
let page;
const errors = [];
try {
    page = await browser.newPage();
    page.on("pageerror", (error) => errors.push(error.message));
    page.on("console", (message) => { if (message.type() === "error") errors.push(message.text()); });
    await page.goto(origin);
    await page.locator('nav a[title="Content diff"]').click();
    await page.locator("#content").fill("---\n");
    await page.locator("#after").fill("+++\n");
    await page.getByRole("button", { name: "Run Content diff", exact: true }).click();
    await page.getByText("1 added lines · 1 removed lines", { exact: true }).waitFor();
    await page.locator('nav a[title="Home"]').click();
    await page.getByRole("button", { name: "Try sample data" }).click();
    await page.locator(".activity-table tbody tr").first().waitFor();
    await page.locator('nav a[title="Geospatial"]').click();
    await page.locator("#geospatial-map canvas").waitFor({ timeout: 60000 });
    await page.locator(".organization-table tbody button").first().click();
    await page.waitForFunction(() => performance.getEntriesByType("resource").some((entry) => entry.name.includes(":8083/src/map.tsx")));
    const navigation = await page.evaluate(() => performance.timeOrigin);
    await writeFile(mapPath, originalMap.replace("<Viewer full", '<Viewer containerProps={{ "data-hmr-check": "updated" }} full'));
    await page.locator('[data-hmr-check="updated"]').waitFor({ timeout: 30000 });
    await writeFile(cssPath, originalCss + '\n#geospatial-map { --hmr-check: updated; }\n');
    await page.waitForFunction(() => getComputedStyle(document.querySelector("#geospatial-map")).getPropertyValue("--hmr-check").trim() === "updated");
    assert.equal(await page.evaluate(() => performance.timeOrigin), navigation, "Map edits must not navigate or rebuild the Dioxus app");
    assert.equal(await page.locator('.organization-table button[aria-pressed="true"]').count(), 1);
    assert.deepEqual(errors, []);
    console.log(`Development smoke passed at ${origin}: exact-port CORS, separator diff counts, map loading and TSX/CSS refresh with route and selection retained.`);
} catch (error) {
    console.error(errors, await page?.locator("main").textContent());
    throw error;
} finally {
    await writeFile(mapPath, originalMap); await writeFile(cssPath, originalCss); await browser.close();
}
