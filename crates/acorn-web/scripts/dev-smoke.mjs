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
const appCssPath = "crates/acorn-web/assets/app.css";
const badgePath = "crates/acorn-web/assets/ai-assisted.svg";
const cssPath = "crates/acorn-web/geospatial/src/map.css";
const mapPath = "crates/acorn-web/geospatial/src/map.tsx";
const uiPath = "crates/acorn-web/src/ui.rs";
const originalAppCss = await readFile(appCssPath, "utf8");
const originalBadge = await readFile(badgePath, "utf8");
const originalCss = await readFile(cssPath, "utf8");
const originalMap = await readFile(mapPath, "utf8");
const originalUi = await readFile(uiPath, "utf8");
const browser = await chromium.launch({ args: ["--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader"] });
let page;
const errors = [];
try {
    page = await browser.newPage();
    page.on("pageerror", (error) => errors.push(error.message));
    page.on("console", (message) => { if (message.type() === "error") errors.push(message.text()); });
    await page.goto(origin);
    await page.locator(".workspace").waitFor();
    assert.equal(await page.locator("main").count(), 1, "Development startup mounts one app root");
    assert.equal(await page.locator('script[src*="effects-"]').count(), 1, "Development startup loads effects once");
    await page.locator('nav a[title="Content diff"]').click();
    await page.locator("#content-editor").fill("---\n");
    await page.locator("#after-editor").fill("+++\n");
    await page.getByRole("button", { name: "Compare", exact: true }).click();
    await page.getByText("1 added lines · 1 removed lines", { exact: true }).waitFor();
    await page.locator('nav a[title="Home"]').click();
    await page.getByRole("button", { name: "Try Sample Data" }).click();
    await page.locator(".activity-table tbody tr").first().waitFor();
    await page.locator('nav a[title="Geospatial"]').click();
    await page.locator("#geospatial-map canvas").waitFor({ timeout: 60000 });
    await page.locator(".organization-table tbody button").first().click();
    await page.getByRole("dialog").waitFor();
    await page.keyboard.press("Escape");
    await page.waitForFunction(() => performance.getEntriesByType("resource").some((entry) => entry.name.includes(":8083/src/map.tsx")));
    const navigation = await page.evaluate(() => performance.timeOrigin);
    await writeFile(mapPath, originalMap.replace("</Viewer>", '</Viewer><span data-hmr-check="updated" />'));
    await page.locator('[data-hmr-check="updated"]').waitFor({ state: "attached", timeout: 30000 });
    await writeFile(cssPath, originalCss + '\n#geospatial-map { --hmr-check: updated; }\n');
    await page.waitForFunction(() => getComputedStyle(document.querySelector("#geospatial-map")).getPropertyValue("--hmr-check").trim() === "updated");
    assert.equal(await page.evaluate(() => performance.timeOrigin), navigation, "Map edits must not navigate or rebuild the Dioxus app");
    assert.equal(await page.locator('.organization-table button[aria-pressed="true"]').count(), 1);
    await page.locator('nav a[title="Home"]').click();
    assert.ok(originalUi.includes('"Plant an acorn, grow your science 🌱"'));
    await writeFile(uiPath, originalUi.replace('"Plant an acorn, grow your science 🌱"', '"Plant an acorn, grow your science 🌱 · hot reload"'));
    await page.getByText("Plant an acorn, grow your science 🌱 · hot reload").waitFor({ timeout: 60000 });
    await page.locator('nav a[title="Portfolio"]').click();
    await page.locator(".activity-table tbody tr").first().waitFor();
    await writeFile(appCssPath, `${originalAppCss}\n:root { --abw-hmr-check: updated; }\n`);
    await page.waitForFunction(() => getComputedStyle(document.documentElement).getPropertyValue("--abw-hmr-check").trim() === "updated", null, { timeout: 60000 });
    await page.locator('nav a[title="About"]').click();
    assert.ok(originalBadge.includes("<title>AI Assisted badge</title>"));
    await writeFile(badgePath, originalBadge.replace("<title>AI Assisted badge</title>", "<title>AI Assisted badge</title><desc>ABW live asset</desc>"));
    await page.waitForFunction(async () => {
        const badge = document.querySelector('img[alt="AI Assisted"]');
        if (!badge) return false;
        const response = await fetch(badge.src);
        return response.ok && (await response.text()).includes("ABW live asset");
    }, null, { timeout: 60000 });
    assert.deepEqual(errors, []);
    console.log(`Development smoke passed at ${origin}: exact-port CORS, separator diff counts, map TSX/CSS and Dioxus RSX/CSS/asset refresh.`);
} catch (error) {
    console.error(errors, await page?.locator("main").allTextContents());
    throw error;
} finally {
    await writeFile(mapPath, originalMap);
    await writeFile(cssPath, originalCss);
    await writeFile(uiPath, originalUi);
    await writeFile(appCssPath, originalAppCss);
    await writeFile(badgePath, originalBadge);
    await browser.close();
}
