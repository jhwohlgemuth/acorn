// SPDX-License-Identifier: MIT
import assert from "node:assert/strict";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";
import { startModelViewer } from "./model-viewer.mjs";

const root = resolve(process.argv[2] ?? `${dirname(fileURLToPath(import.meta.url))}/../assets/models`);
const category = process.argv[3] === "--contacts-only" ? undefined : process.argv[3];
const contactsOnly = process.argv.includes("--contacts-only");
const catalog = JSON.parse(await readFile(`${root}/catalog.json`, "utf8"));
const assets = catalog.assets.filter((asset) => !category || asset.category === category);
assert.ok(assets.length);
const output = resolve(root, "previews");
await mkdir(output, { recursive: true });
const server = await startModelViewer(0, root);
const origin = `http://127.0.0.1:${server.address().port}`;
let browser;
try {
    browser = await chromium.launch();
    const page = await browser.newPage({ viewport: { width: 760, height: 760 } });
    const failures = [];
    page.on("pageerror", (error) => failures.push(String(error)));
    await page.route("**/*", (route) => {
        if (route.request().url().startsWith(`${origin}/`)) void route.continue();
        else { failures.push(route.request().url()); void route.abort(); }
    });
    await page.goto(origin);
    await page.addStyleTag({ content: ".model-workspace main{margin:0;padding:0;min-height:0}.model-page{max-width:none}.model-workspace .model-layout{display:block}.stage{height:100vh;border:0;border-radius:0}.page-toolbar,.filterbar,.collection,.scene-footer{display:none!important}.scene-heading,.toolbar{opacity:0}" });
    const frames = () => page.evaluate(() => new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(done))));
    for (const asset of (contactsOnly ? [] : assets)) {
        await page.locator("#asset").selectOption(asset.id, { force: true });
        await page.waitForFunction((id) => document.getElementById("viewport").dataset.asset === id && document.getElementById("viewport").getAttribute("aria-busy") === "false", asset.id);
        const slug = asset.id.replaceAll("/", "-");
        for (const [view, button] of [["front", "Reset view"], ["rear", "Back"], ["top", "Top"]]) {
            await page.getByRole("button", { name: button, exact: true }).click({ force: true });
            await frames();
            await page.screenshot({ path: `${output}/${slug}-${view}.png` });
        }
        await page.setViewportSize({ width: 220, height: 220 });
        await page.getByRole("button", { name: "Reset view", exact: true }).click({ force: true });
        await frames();
        await page.screenshot({ path: `${output}/${slug}-small.png` });
        await page.setViewportSize({ width: 760, height: 760 });
        assert.equal(await page.locator("#status").getAttribute("data-error"), "false");
        console.log(`Rendered ${asset.id}: front, rear, top, small`);
    }
    assert.deepEqual(failures, []);
    await page.unroute("**/*");
    await page.setViewportSize({ width: 1440, height: 1024 });
    for (const group of (category ? [category] : ["collection", ...new Set(assets.map((asset) => asset.category))])) {
    const groupAssets = assets.filter((asset) => group === "collection" || asset.category === group);
    for (const view of ["front", "rear", "top", "small"]) {
        const cards = await Promise.all(groupAssets.map(async (asset) => `<figure><img src="data:image/png;base64,${(await readFile(`${output}/${asset.id.replaceAll("/", "-")}-${view}.png`)).toString("base64")}"><figcaption>${asset.label}</figcaption></figure>`));
        const html = `<!doctype html><style>body{margin:0;background:#101923;color:#eee6d4;font:18px system-ui}h1{font-size:24px;padding:24px 24px 0}section{display:grid;grid-template-columns:repeat(4,1fr);gap:10px;padding:16px}figure{margin:0;border:1px solid #30404f;border-radius:8px;overflow:hidden}img{width:100%;display:block}figcaption{padding:0 12px 16px;text-align:center}</style><h1>${group} / ${view} / rendered GLB collection</h1><section>${cards.join("")}</section>`;
        await page.setContent(html);
        await page.evaluate(() => Promise.all([...document.images].map((image) => image.decode())));
        await page.screenshot({ path: `${output}/${group}${view === "front" ? "" : `-${view}`}-overview.png`, fullPage: true });
    }
    await writeFile(`${output}/${group}-acceptance.json`, JSON.stringify({ renderer: "Cesium 1.147.0 / Chromium", assets: groupAssets.map((asset) => ({ id: asset.id, sha256: asset.sha256 })), externalRequests: 0, pageErrors: 0, views: ["front", "rear", "top", "small"], visualReview: "Inspect saved images before acceptance" }, null, 2));
    }
} finally {
    if (browser) await browser.close();
    await new Promise((done) => server.close(done));
}
