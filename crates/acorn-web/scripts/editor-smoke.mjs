import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import { chromium } from "playwright";

const server = createServer((_request, response) => response.end('<div class="workspace" data-theme="dark"><div class="code-editor" data-language="json" style="--editor-height:170px"><div class="editor-toolbar"><button onclick="acornEditors.fold(\'content\')">Fold all</button><button onclick="acornEditors.expand(\'content\')">Expand all</button></div><div class="editor-surface"></div><textarea id="content" aria-label="Content" rows="7"></textarea></div></div>'));
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
const browser = await chromium.launch().catch(error => { server.close(); throw error; });
try {
    const page = await browser.newPage({ permissions: ["clipboard-read", "clipboard-write"] });
    const errors = [];
    page.on("pageerror", error => errors.push(error.message));
    await page.goto(`http://127.0.0.1:${server.address().port}`);
    await page.addStyleTag({ content: await readFile(new URL("../assets/app.css", import.meta.url), "utf8") });
    await page.addStyleTag({ content: "::-webkit-scrollbar{width:16px;height:16px}" });
    assert.ok(await page.locator("#content").isVisible(), "native editing works before the bundle loads");
    const original = '{\n  "project": {\n    "title": "ACORN 🌳",\n    "team": [\n      "one",\n      "two"\n    ]\n  },\n  "status": "active"\n}';
    await page.locator("#content").fill(original);
    await page.addScriptTag({ content: await readFile(new URL("../assets/generated/editor/editor.js", import.meta.url), "utf8") });
    const editor = page.locator("#content-editor");
    await editor.waitFor();
    assert.equal(await page.locator("#content").isVisible(), false);
    async function resizeEditor(delta) {
        const surface = page.locator(".editor-surface").first();
        const before = await surface.boundingBox();
        const grip = await page.locator(".editor-resize-handle").first().boundingBox();
        const x = grip.x + grip.width / 2;
        const y = grip.y + grip.height / 2;
        await page.mouse.move(x, y);
        await page.mouse.down();
        await page.mouse.move(x, y + delta, { steps: 8 });
        await page.mouse.up();
        const after = await surface.boundingBox();
        assert.ok(Math.abs(after.height - before.height - delta) < 5, "dragging the resize handle changes editor height");
        return after.height;
    }
    await resizeEditor(120);
    const resizedHeight = await resizeEditor(-60);
    await page.getByRole("separator").press("ArrowDown");
    assert.equal((await page.locator(".editor-surface").boundingBox()).height, resizedHeight + 24);
    await page.getByRole("separator").press("ArrowUp");
    await page.evaluate(() => {
        window.changes = [];
        document.getElementById("content").addEventListener("input", event => {
            window.changes.push(event.target.value);
            window.acornEditors.sync("content", null, event.target.closest(".code-editor").dataset.language);
        });
    });
    await page.locator('.cm-foldGutter [title="Fold line"]').nth(1).click();
    assert.equal(await page.locator(".cm-foldPlaceholder").count(), 1);
    assert.equal(await page.locator("#content").inputValue(), original);
    assert.equal(await page.evaluate(() => changes.length), 0, "folding never changes input");
    await editor.press("ControlOrMeta+Home");
    await page.keyboard.insertText(" ");
    await page.waitForFunction(() => document.getElementById("content").value.startsWith(" {"));
    assert.equal(await page.locator(".cm-foldPlaceholder").count(), 1, "editing outside a fold keeps it folded");
    await editor.press("ControlOrMeta+z");
    assert.equal(await page.locator("#content").inputValue(), original, "undo restores source text");
    assert.equal((await page.locator(".editor-surface").first().boundingBox()).height, resizedHeight, "editing and folding preserve the chosen height");
    await editor.press("ControlOrMeta+Shift+z");
    assert.equal(await page.locator("#content").inputValue(), " " + original, "redo restores the edit");
    await editor.press("ControlOrMeta+z");
    await editor.press("ControlOrMeta+a");
    await editor.press("ControlOrMeta+c");
    assert.equal(await page.evaluate(() => navigator.clipboard.readText()), original, "copy includes folded content, without gutter numbers");
    await editor.press("ControlOrMeta+x");
    await page.waitForFunction(() => document.getElementById("content").value === "");
    await editor.press("ControlOrMeta+v");
    await page.waitForFunction(text => document.getElementById("content").value === text, original);
    await page.getByRole("button", { name: "Fold all", exact: true }).click();
    await page.getByRole("button", { name: "Fold all", exact: true }).click();
    assert.ok(await page.locator(".cm-foldPlaceholder").count() > 0);
    async function checkNestedFolds() {
        await page.locator(".cm-foldPlaceholder").first().click();
        assert.equal(await page.locator(".cm-foldPlaceholder").count(), 1, "opening the root leaves the project folded");
        assert.equal(await editor.textContent().then(text => text.includes("title")), false);
        await page.locator(".cm-foldPlaceholder").first().click();
        assert.equal(await page.locator(".cm-foldPlaceholder").count(), 1, "opening the project leaves its team array folded");
        assert.equal(await editor.textContent().then(text => text.includes("one")), false);
        assert.equal(await page.locator("#content").inputValue(), original, "nested folding preserves the complete source");
    }
    await checkNestedFolds();
    await editor.press("Control+Alt+]");
    assert.equal(await page.locator(".cm-foldPlaceholder").count(), 0);
    await editor.press("Control+Alt+[");
    assert.ok(await page.locator(".cm-foldPlaceholder").count() > 0);
    await checkNestedFolds();
    await page.getByRole("button", { name: "Expand all", exact: true }).click();
    assert.equal(await page.locator(".cm-foldPlaceholder").count(), 0);
    await page.evaluate(() => window.acornEditors.sync("content", null, "json", "Before"));
    assert.equal(await editor.getAttribute("aria-label"), "Before", "route changes update the accessible label");
    await editor.press("ArrowRight");
    assert.equal(await editor.getAttribute("aria-label"), "Before", "cursor movement preserves the updated label");
    async function replace(value, language = "json", fold = false) {
        await page.evaluate(({ value, language, fold }) => {
            const input = document.getElementById("content");
            input.value = value;
            input.closest(".code-editor").dataset.language = language;
            window.acornEditors.sync("content", value, language);
            if (fold) window.acornEditors.fold("content");
        }, { value, language, fold });
    }
    const largeJson = JSON.stringify({ items: Array.from({ length: 2000 }, (_, id) => ({ id, title: "Research" })) }, null, 2);
    await replace(largeJson, "json", true);
    assert.equal(await editor.locator(".cm-line").count(), 1, "Fold all parses offscreen JSON immediately after replacement");
    assert.equal(await page.locator("#content").inputValue(), largeJson);
    await page.locator(".cm-foldPlaceholder").first().click();
    assert.equal(await page.locator(".cm-foldPlaceholder").count(), 1, "large JSON retains its folded array inside the root");
    await page.locator(".cm-foldPlaceholder").first().click();
    await editor.press("ControlOrMeta+End");
    assert.ok(await page.locator(".cm-foldPlaceholder").count() > 0, "offscreen nested objects stay folded at the end of a large document");
    assert.equal(await editor.textContent().then(text => text.includes('"title"')), false);
    for (const [language, text] of [["yaml", "project:\n  title: ACORN\n  team:\n    - one\n    - two\n"], ["markdown", "# Science\n\nResearch notes\n\n## Methods\n\nMore notes\n"], ["xml", "<project>\n  <team>\n    <member>ACORN</member>\n  </team>\n</project>"], ["javascript", "function science() {\n  if (ready) {\n    run();\n  }\n}"]]) {
        await replace(text, language);
        await page.locator('.cm-foldGutter [title="Fold line"]').first().waitFor();
        await page.getByRole("button", { name: "Fold all", exact: true }).click();
        assert.ok(await page.locator(".cm-foldPlaceholder").count() > 0, `${language} supports folding`);
        await page.locator(".cm-foldPlaceholder").first().click();
        assert.ok(await page.locator(".cm-foldPlaceholder").count() > 0, `${language} retains nested folds when the parent opens`);
        assert.equal(await page.locator("#content").inputValue(), text);
    }
    await replace('{\n  "incomplete": [\n    1,\n    2');
    await editor.press("ControlOrMeta+End");
    await page.keyboard.insertText("\n  ]\n}");
    await page.waitForFunction(() => document.getElementById("content").value.endsWith("\n  ]\n}"));
    assert.deepEqual(JSON.parse(await page.locator("#content").inputValue()), { incomplete: [1, 2] });
    for (const text of ["", "\n\n", "Some research words here. ".repeat(4923) + "\n", "\n".repeat(100_000), original]) {
        const started = Date.now();
        await replace(text, "auto", true);
        await editor.press("ControlOrMeta+End");
        assert.equal(await page.locator("#content").inputValue(), text);
        assert.ok(await page.locator(".cm-line").count() < 500, "large documents keep a bounded rendered viewport");
        assert.ok(Date.now() - started < 3000, "large inputs remain responsive");
    }
    for (const theme of ["light", "dark"]) {
        for (const width of [220, 640]) {
            await page.evaluate(({ theme, width }) => {
                document.querySelector(".workspace").dataset.theme = theme;
                document.querySelector(".code-editor").style.width = `${width}px`;
            }, { theme, width });
            await resizeEditor(40);
            await resizeEditor(-40);
            assert.ok(await editor.isVisible());
            assert.ok(await page.locator(".cm-lineNumbers").isVisible());
            await page.getByRole("button", { name: "Fold all", exact: true }).click();
            assert.ok(await page.locator(".cm-foldPlaceholder").count() > 0);
            await page.getByRole("button", { name: "Expand all", exact: true }).click();
        }
    }
    await editor.press("ControlOrMeta+z");
    assert.equal(await page.locator("#content").inputValue(), original, "external replacements clear stale undo history");
    await page.evaluate(() => {
        const clone = document.querySelector(".code-editor").cloneNode(true);
        clone.classList.remove("cm-ready");
        clone.querySelector(".editor-surface").replaceChildren();
        clone.querySelector("textarea").id = "after";
        clone.querySelector("textarea").readOnly = true;
        clone.querySelector("textarea").value = '{\n  "after": [1, 2]\n}';
        document.querySelector(".workspace").append(clone);
    });
    await page.locator("#after-editor").waitFor();
    const readonlySource = await page.locator("#after").inputValue();
    await page.locator("#after-editor").press("ControlOrMeta+Home");
    await page.keyboard.insertText("must not change");
    assert.equal(await page.locator("#after").inputValue(), readonlySource, "read-only previews reject edits");
    await page.evaluate(() => window.acornEditors.fold("after"));
    assert.ok(await page.locator("#after-editor .cm-foldPlaceholder").count() > 0);
    assert.equal(await page.locator("#content-editor .cm-foldPlaceholder").count(), 0, "editors fold independently");
    await editor.focus();
    await editor.press("Tab");
    assert.equal(await editor.evaluate(node => node === document.activeElement), false, "Tab does not trap keyboard focus");
    await page.locator("#after").evaluate(node => node.closest(".code-editor").remove());
    await page.evaluate(() => {
        const host = document.querySelector(".code-editor");
        const clone = host.cloneNode(true);
        clone.classList.remove("cm-ready");
        clone.querySelector(".editor-surface").replaceChildren();
        clone.querySelector("textarea").value = "{}";
        host.remove();
        document.querySelector(".workspace").append(clone);
    });
    await page.waitForFunction(() => document.querySelectorAll(".cm-editor").length === 1);
    assert.equal(await editor.textContent(), "{}", "remount starts from the new input, without stale editor state");
    assert.deepEqual(errors, []);
    console.log("CodeMirror resizing, folding, editing, undo/redo, clipboard, invalid JSON, language changes, large inputs, remounts and both themes passed.");
} finally {
    await browser.close();
    await new Promise(resolve => server.close(resolve));
}
