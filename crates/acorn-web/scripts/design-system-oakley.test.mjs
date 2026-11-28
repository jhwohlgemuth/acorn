import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { chromium, firefox } from "playwright";

const root = new URL("../", import.meta.url);
const preview = new URL("design-system/oakley/index.html", root);
const catalog = await readFile(new URL("design-system/index.html", root), "utf8");
const browserType = { chromium, firefox }[process.env.ACORN_WEB_ENGINE ?? "chromium"];
assert.ok(browserType, "Supported Oakley browser engine");

async function assertCenteredBlink(page, selector, closedTime) {
    const blink = await page.locator(selector).first().evaluate((eye, time) => {
        const animation = eye.getAnimations()[0];
        animation.pause();
        animation.currentTime = 0;
        const open = { y: parseFloat(getComputedStyle(eye).y), height: parseFloat(getComputedStyle(eye).height) };
        animation.currentTime = time;
        const closed = { y: parseFloat(getComputedStyle(eye).y), height: parseFloat(getComputedStyle(eye).height) };
        animation.cancel();
        return { open, closed };
    }, closedTime);
    assert.ok(Math.abs(blink.open.y + blink.open.height / 2 - blink.closed.y - blink.closed.height / 2) < 0.5, `${selector} stays centered: ${JSON.stringify(blink)}`);
    assert.ok(blink.closed.height < blink.open.height / 2, `${selector} closes: ${JSON.stringify(blink)}`);
}

test("catalog links its Oakley specimen to the preview", () => {
    assert.match(catalog, /<section class="specimen" id="agent-avatar">/);
    assert.match(catalog, /href="oakley\/index\.html"/);
    assert.match(catalog, /href="#agent-avatar">Agent<\/a>/);
});

test("Oakley preview states, actions, themes, and launcher demo work offline", async () => {
    const browser = await browserType.launch();
    try {
        const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
        const errors = [];
        const external = [];
        page.on("pageerror", (error) => errors.push(String(error)));
        await page.route("**/*", (route) => {
            const local = route.request().url().startsWith("file:");
            void (local ? route.continue() : (external.push(route.request().url()), route.abort()));
        });
        await page.goto(preview.href);
        await assertCenteredBlink(page, "#states [data-st='idle'] .ok-blink", 4800);
        await page.locator("#actions [data-play='double-blink']").click();
        await assertCenteredBlink(page, "#actions [data-ac='double-blink'] .ok-dbl", 1776);
        assert.equal(await page.locator("[data-catalog-link]").getAttribute("href"), "../index.html#agent-avatar");
        assert.equal(await page.locator("#heroAv svg").count(), 1);
        assert.equal(await page.locator("#states .card").count(), 4);
        assert.equal(await page.locator("#actions .card").count(), 7);
        assert.equal(await page.locator("#heroAv svg [d^='M22.5 46']").count(), 0, "Idle Oakley has no mouth or happy eyes");
        await page.getByRole("button", { name: "Thinking", exact: true }).click();
        assert.equal(await page.locator("#roState").textContent(), "thinking");
        assert.equal(await page.locator("#heroAv .ok-bar").count(), 5);
        await page.locator("#actionBtns").getByRole("button", { name: "Wink", exact: true }).click();
        assert.equal(await page.locator("#roAction").textContent(), "wink");
        assert.equal(await page.locator("#heroAv .ok-caret").count(), 1);
        await page.locator("#actionBtns").getByRole("button", { name: "Huh?", exact: true }).click();
        assert.equal(await page.locator("#roAction").textContent(), "huh?");
        assert.equal(await page.locator("#heroAv .ok-huh").count(), 1);
        assert.equal(await page.locator("#panel").isHidden(), true);
        await page.getByRole("button", { name: "Run validation with issues" }).click();
        await page.locator("#fabAv .ok-bar").first().waitFor({ state: "attached" });
        await page.locator("#badge").waitFor({ state: "visible", timeout: 5000 });
        assert.equal(await page.locator("#fabAv .ok-arc").count(), 2);
        assert.match(await page.locator("#bubble").textContent(), /2 DOIs/);
        await page.getByRole("button", { name: "Open Oakley chat, 1 new message" }).click();
        assert.equal(await page.locator("#panel").isVisible(), true);
        assert.equal(await page.locator("#badge").isHidden(), true);
        await page.locator("#log .msg", { hasText: "doi.org?" }).waitFor({ timeout: 8000 });
        await page.getByRole("button", { name: "Close Oakley chat" }).click();
        assert.equal(await page.locator("#panel").isHidden(), true);
        await page.getByRole("button", { name: "Run clean validation" }).click();
        await page.locator("#bubble", { hasText: "All 14 records pass" }).waitFor({ timeout: 5000 });
        assert.equal(await page.locator("#sizesUswds figure").count(), 6);
        await page.locator("#themeBtns").getByRole("button", { name: "USWDS", exact: true }).click();
        assert.equal(await page.evaluate(() => document.documentElement.dataset.theme), "uswds");
        assert.equal(await page.locator("#heroAv svg").evaluate((svg) => getComputedStyle(svg).color), "rgb(0, 94, 162)");
        assert.equal(await page.locator("#fab").evaluate((fab) => getComputedStyle(fab).borderTopColor), "rgb(0, 94, 162)");
        assert.equal(await page.locator(".card").first().evaluate((card) => getComputedStyle(card).borderRadius), "0px");
        await page.reload();
        assert.equal(await page.evaluate(() => document.documentElement.dataset.theme), "uswds", "Theme choice survives reload");
        await page.locator("#themeBtns").getByRole("button", { name: "System", exact: true }).click();
        assert.equal(await page.evaluate(() => document.documentElement.hasAttribute("data-theme")), false);
        await page.locator("#styleBtns").getByRole("button", { name: "Pixel", exact: true }).click();
        await assertCenteredBlink(page, "#states [data-st='idle'] .okp-blink", 4800);
        await page.locator("#actions [data-play='double-blink']").click();
        await assertCenteredBlink(page, "#actions [data-ac='double-blink'] .okp-dbl", 1728);
        assert.equal(await page.locator("#heroAv svg.oakley-pixel").getAttribute("shape-rendering"), "crispEdges");
        await page.locator("#actionBtns").getByRole("button", { name: "Huh?", exact: true }).click();
        assert.equal(await page.locator("#heroAv .okp-huh").count(), 1);
        assert.equal(await page.locator("#states svg.oakley-pixel").count(), 4);
        await page.locator("#states [data-st='thinking'] .okp-bar").first().waitFor({ state: "attached" });
        assert.equal(await page.locator("#states [data-st='thinking'] .okp-bar").count(), 5);
        assert.equal(await page.locator("#heroAv svg [d^='M22.5 46']").count(), 0, "Pixel Oakley has no mouth");
        await page.locator("#styleBtns").getByRole("button", { name: "Smooth", exact: true }).click();
        assert.equal(await page.locator("#heroAv svg.oakley-pixel").count(), 0);
        await page.locator("#themeBtns").getByRole("button", { name: "Retro", exact: true }).click();
        assert.equal(await page.evaluate(() => document.documentElement.dataset.theme), "retro");
        assert.equal(await page.locator("#styleBtns [data-style='pixel']").getAttribute("aria-pressed"), "true", "Retro switches to the pixel style");
        assert.equal(await page.locator("#heroAv svg").evaluate((svg) => getComputedStyle(svg).color), "rgb(44, 54, 24)");
        assert.equal(await page.locator("#sizesRetro svg.oakley-pixel").count(), 4);
        await page.reload();
        assert.equal(await page.locator("#heroAv svg.oakley-pixel").count(), 1, "Pixel style survives reload");
        await page.locator("#themeBtns").getByRole("button", { name: "System", exact: true }).click();
        await page.locator("#styleBtns").getByRole("button", { name: "Smooth", exact: true }).click();
        await page.setViewportSize({ width: 390, height: 844 });
        assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, "No horizontal page scroll at 390px");
        assert.deepEqual(errors, []);
        assert.deepEqual(external, []);
    } finally {
        await browser.close();
    }
});

test("catalog Oakley specimen follows the catalog appearance", async () => {
    const browser = await browserType.launch();
    try {
        const page = await browser.newPage({ viewport: { width: 1200, height: 800 } });
        await page.goto(new URL("design-system/index.html#agent-avatar", root).href);
        const ink = () => page.locator("#agent-avatar-idle svg").evaluate((svg) => getComputedStyle(svg).color);
        const expected = { light: "rgb(0, 102, 44)", dark: "rgb(152, 195, 121)", uswds: "rgb(0, 94, 162)" };
        for (const [theme, color] of Object.entries(expected)) {
            await page.locator("#catalog-theme").selectOption(theme);
            assert.equal(await ink(), color, theme);
            assert.equal(await page.locator("#agent-avatar-pixel svg").evaluate((svg) => getComputedStyle(svg).color), color, `${theme} pixel`);
        }
        assert.equal(await page.locator("#agent-avatar-pixel svg").getAttribute("shape-rendering"), "crispEdges");
        assert.equal(await page.locator("#agent-avatar .sample-oakley-launcher").getAttribute("aria-label"), "Open Oakley chat, 1 new message");
    } finally {
        await browser.close();
    }
});
