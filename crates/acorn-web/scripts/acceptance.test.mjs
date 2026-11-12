import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

const assets = new URL("../assets/", import.meta.url);
const css = await readFile(new URL("app.css", assets), "utf8");
const tokens = (selector) => Object.fromEntries([...css.match(selector)[1].matchAll(/--([a-z]+):(#[a-f0-9]{3,6})/g)].map(([, key, value]) => [key, value.length === 4 ? "#" + [...value.slice(1)].map((digit) => digit + digit).join("") : value]));
const luminance = (color) => color.slice(1).match(/../g).map((hex) => parseInt(hex, 16) / 255).map((value) => value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4).reduce((sum, value, index) => sum + value * [0.2126, 0.7152, 0.0722][index], 0);
const contrast = (a, b) => (Math.max(luminance(a), luminance(b)) + 0.05) / (Math.min(luminance(a), luminance(b)) + 0.05);

test("both themes retain readable normal, hover, selected and disabled text and visible focus", () => {
    for (const theme of [tokens(/\.workspace\{([^}]+)/), tokens(/\.workspace\[data-theme=light\]\{([^}]+)/)]) {
        for (const foreground of [theme.text, theme.muted]) {
            for (const background of [theme.canvas, theme.side, theme.card, theme.hover, theme.active]) {
                assert.ok(contrast(foreground, background) >= 4.5, `${foreground} on ${background}`);
            }
        }
        for (const background of [theme.canvas, theme.side, theme.card, theme.hover, theme.active]) {
            assert.ok(contrast(theme.focus, background) >= 3, `focus ${theme.focus} on ${background}`);
        }
    }
    for (const background of ["#00662c", "#087f3c"]) assert.ok(contrast("#ffffff", background) >= 4.5);
});

test("technology views embed unchanged attributed bytes with no external image requests", async () => {
    const svg = await readFile(new URL("technology.svg", assets), "utf8");
    const notice = await readFile(new URL("NOTICE.md", assets), "utf8");
    const views = [...svg.matchAll(/<view id="([a-z]+)"[^>]+\/><image[^>]+href="data:image\/svg\+xml;base64,([^"]+)"/g)];
    assert.equal(views.length, 4);
    for (const [, name, data] of views) {
        const bytes = Buffer.from(data, "base64");
        const digest = createHash("sha256").update(bytes).digest("hex");
        assert.ok(notice.includes(`| ${name} |`) && notice.includes(digest), name);
        assert.doesNotMatch(bytes.toString(), /<script|(?:href|src)=["']https?:/);
        if (["rust", "webassembly"].includes(name)) assert.deepEqual(bytes, await readFile(new URL(`../../../docs/src/_assets/schema-logos/${name}.svg`, assets)));
    }
});

test("staged theme banners match the canonical repository artwork", async () => {
    for (const theme of ["dark", "light"]) {
        const name = `banner_${theme}_mode.png`;
        assert.deepEqual(await readFile(new URL(`generated/${name}`, assets)), await readFile(new URL(`../../../docs/src/_assets/${name}`, assets)));
    }
});

test("tour and disclosure assets match their recorded source hashes", async () => {
    const notice = await readFile(new URL("NOTICE.md", assets), "utf8");
    for (const file of ["vendor/driver.js", "vendor/driver.css", "vendor/driver.LICENSE", "ai-assisted.svg"]) {
        const bytes = await readFile(new URL(file, assets));
        assert.ok(notice.includes(createHash("sha256").update(bytes).digest("hex")), file);
    }
    assert.ok(notice.includes(await readFile(new URL("vendor/driver.LICENSE", assets), "utf8")), "the shipped notice retains Driver.js attribution");
    const marks = await readFile(new URL("agent-marks.svg", assets), "utf8");
    const embedded = [...marks.matchAll(/href="data:image\/svg\+xml;base64,([^"]+)"/g)];
    assert.equal(embedded.length, 3);
    for (const [, data] of embedded) {
        const bytes = Buffer.from(data, "base64");
        assert.ok(notice.includes(createHash("sha256").update(bytes).digest("hex")));
        assert.doesNotMatch(bytes.toString(), /<script|(?:href|src)=["']https?:/);
    }
});
