import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { chromium } from "playwright";

const browser = await chromium.launch({ args: ["--disable-features=OverlayScrollbar"] });
try {
    const page = await browser.newPage();
    await page.setContent('<div class="workspace" data-theme="dark"><div class="code-block"><div class="code-viewport"><pre><code class="syntax"></code></pre><div class="line-numbers" aria-hidden="true"></div></div></div></div>');
    await page.addStyleTag({ content: await readFile(new URL("../assets/app.css", import.meta.url), "utf8") });
    // Force a measurable classic scrollbar even on hosts configured for overlay scrollbars.
    await page.addStyleTag({ content: "::-webkit-scrollbar{width:16px;height:16px}" });
    await page.addScriptTag({ content: await readFile(new URL("../assets/vendor/prism.js", import.meta.url), "utf8") });
    await page.addScriptTag({ content: await readFile(new URL("../assets/syntax.js", import.meta.url), "utf8") });
    const wrapped = JSON.stringify({ lines: Array.from({ length: 24 }, (_, index) => `${index}: ${"research words ".repeat(18)}`) }, null, 2);
    for (const theme of ["dark", "light"]) {
        for (const width of [220, 640]) {
            await page.evaluate(({ theme, width, wrapped }) => {
                document.querySelector(".workspace").dataset.theme = theme;
                const viewport = document.querySelector(".code-viewport");
                viewport.style.width = `${width}px`;
                viewport.querySelector("code").innerHTML = highlightCode(Prism, wrapped, "json");
            }, { theme, width, wrapped });
            for (const position of [0, 0.5, 1]) {
                await page.locator(".code-viewport pre").evaluate((node, position) => { node.scrollTop = (node.scrollHeight - node.clientHeight) * position; }, position);
                await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
                const alignment = await page.locator(".code-viewport").evaluate(host => {
                    const code = host.querySelector("code");
                    const walker = document.createTreeWalker(code, NodeFilter.SHOW_TEXT);
                    const nodes = [];
                    let offset = 0;
                    while (walker.nextNode()) { nodes.push({ node: walker.currentNode, offset }); offset += walker.currentNode.length; }
                    return Array.from(host.querySelectorAll(".line-numbers > span"), number => {
                        const line = Number(number.dataset.line);
                        const start = lineStarts(code.textContent)[line - 1];
                        const part = nodes.find(({ node, offset }) => start >= offset && start < offset + node.length);
                        const range = document.createRange();
                        range.setStart(part.node, start - part.offset);
                        range.setEnd(part.node, start - part.offset + 1);
                        const text = range.getBoundingClientRect();
                        const label = number.getBoundingClientRect();
                        return Math.abs((text.top + text.bottom) / 2 - (label.top + label.bottom) / 2);
                    });
                });
                assert.ok(alignment.length > 0, "output has visible line numbers");
                assert.ok(alignment.every(delta => delta < 1), "numbers align with logical line starts after wrapping and scrolling");
                assert.equal(await page.locator(".code-viewport").textContent(), wrapped, "line numbers never enter selected or copied text");
            }
        }
    }
    console.log("Read-only line numbers align with wrapping, scrolling and resizing; output preserves source text in both themes.");
} finally {
    await browser.close();
}
