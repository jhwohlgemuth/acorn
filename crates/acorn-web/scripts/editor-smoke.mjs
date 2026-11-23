import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import { chromium } from "playwright";

const server = createServer(async (request, response) => {
    if (request.url === "/editor.js" || request.url === "/app.css") {
        response.setHeader("Content-Type", request.url === "/editor.js" ? "text/javascript" : "text/css");
        response.end(await readFile(new URL(request.url === "/editor.js" ? "../assets/generated/editor/editor.js" : "../assets/app.css", import.meta.url)));
    } else {
        response.end(`<div class="workspace" data-theme="dark"><div class="code-editor" data-language="json" style="--editor-height:170px"><div class="editor-toolbar"><button onclick="acornEditors.fold('content')">Fold all</button><button onclick="acornEditors.expand('content')">Expand all</button><div class="editor-issue-nav" role="group" aria-label="Issue navigation" hidden><button type="button" aria-label="Previous issue" aria-keyshortcuts="ArrowLeft" onclick="acornEditors.navigateIssue('content', -1)">prev</button><span class="editor-issue-position" aria-live="polite">0 of 0</span><button type="button" aria-label="Next issue" aria-keyshortcuts="ArrowRight" onclick="acornEditors.navigateIssue('content', 1)">next</button></div><button class="copy-code editor-fullscreen-toggle" aria-label="Full screen" onclick="acornEditors.toggleFullscreen('content')"><span class="enter-fullscreen-label">Full screen</span><span class="exit-fullscreen-label">Exit full screen</span></button></div><div class="editor-surface"></div><textarea id="content" aria-label="Content" rows="7"></textarea></div></div>`);
    }
});
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
    await page.evaluate(() => { window.acornPendingEditorReveal = { id: "content", line: 3, character: 15 }; });
    await page.addScriptTag({ url: `http://127.0.0.1:${server.address().port}/editor.js` });
    const editor = page.locator("#content-editor");
    await editor.waitFor();
    assert.equal(await page.evaluate(() => window.getSelection().toString()), "A", "pending source positions select once the editor mounts");
    assert.equal(await page.locator("#content").isVisible(), false);
    await page.evaluate(() => {
        const message = document.createElement("span");
        message.className = "editor-issue-message";
        document.querySelector(".editor-issue-nav").append(message);
    });
    await editor.press("ControlOrMeta+f");
    const searchPanel = page.locator(".cm-search");
    await searchPanel.getByRole("textbox", { name: "Find" }).fill("ACORN");
    assert.ok(await page.locator(".cm-searchMatch").count() > 0, "editor search finds source text");
    assert.ok(await searchPanel.getByRole("textbox", { name: "Replace" }).isVisible(), "editable editors offer replacement");
    await searchPanel.getByRole("textbox", { name: "Replace" }).fill("ACORN edited");
    await searchPanel.getByRole("button", { name: "replace", exact: true }).click();
    await searchPanel.getByRole("button", { name: "replace", exact: true }).click();
    assert.ok((await page.locator("#content").inputValue()).includes("ACORN edited"), "replacement updates the source input");
    await searchPanel.getByRole("button", { name: "close" }).click();
    await editor.press("ControlOrMeta+z");
    assert.equal(await page.locator("#content").inputValue(), original, "replacement remains undoable");
    await page.evaluate(() => window.acornEditors.setFontSize("content", "big"));
    assert.equal(await page.locator(".code-editor").evaluate(node => getComputedStyle(node).fontSize), "16px", "big font size changes the editor");
    await page.evaluate(() => window.acornEditors.setFontSize("content", "small"));
    assert.equal(await page.locator(".code-editor").evaluate(node => getComputedStyle(node).fontSize), "13px", "small font size restores the readable default");
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
    await page.getByRole("button", { name: "Full screen", exact: true }).click();
    const fullScreen = page.locator(".code-editor");
    assert.equal(await fullScreen.evaluate(node => getComputedStyle(node).position), "fixed");
    assert.equal(await page.locator(".editor-fullscreen-toggle").getAttribute("aria-pressed"), null, "Full screen is an action button");
    assert.ok((await page.locator(".editor-surface").boundingBox()).height > page.viewportSize().height - 100, "editor fills the viewport after resizing");
    assert.equal(await page.locator("body").evaluate(node => getComputedStyle(node).overflow), "hidden");
    await page.keyboard.press("Escape");
    assert.equal(await fullScreen.evaluate(node => node.classList.contains("is-fullscreen")), false, "Escape exits full screen");
    assert.equal(await page.locator(".editor-fullscreen-toggle").getAttribute("aria-pressed"), null);
    assert.equal((await page.locator(".editor-surface").boundingBox()).height, resizedHeight, "exiting restores the resized editor height");
    await page.getByRole("button", { name: "Full screen", exact: true }).click();
    await page.getByRole("button", { name: "Exit full screen", exact: true }).click();
    assert.equal(await fullScreen.evaluate(node => node.classList.contains("is-fullscreen")), false, "the toolbar control exits full screen");
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
    await page.getByRole("button", { name: "Fold all", exact: true }).click();
    await page.evaluate(() => window.acornEditors.reveal("content", 3, 15));
    assert.equal(await page.evaluate(() => window.getSelection().toString()), "A", "source positions select the matching character");
    assert.equal(await page.locator(".cm-foldPlaceholder").count(), 0, "source positions reveal folded content");
    await page.evaluate(source => window.acornEditors.setDiagnostics("content", [
        { line: 3, character: 14, endLine: 3, endCharacter: 23, severity: "error", message: "Title check" },
        { line: 3, character: 14, endLine: 3, endCharacter: 23, severity: "warning", message: "Second check" },
        { line: 3, character: 14, endLine: 3, endCharacter: 23, severity: "info", message: "Supported evidence" }
    ], source), original);
    assert.equal(await page.locator(".cm-diagnostic").first().getAttribute("title"), null, "native hover titles are replaced");
    await page.locator(".cm-diagnostic").first().hover();
    const tooltip = page.locator(".cm-diagnostic-tooltip");
    await tooltip.waitFor();
    assert.equal(await tooltip.locator(".cm-diagnostic-tooltip-badge").innerText(), "ERROR");
    assert.deepEqual(await tooltip.locator("p").allTextContents(), ["Title check", "Second check"], "custom tooltip groups issues and excludes positive evidence");
    assert.match(await tooltip.evaluate(node => getComputedStyle(node).borderLeftColor), /235, 133, 140/, "tooltip uses the error accent");
    assert.equal(await page.locator(".cm-lineNumbers .cm-diagnostic-line").count(), 1, "error line receives a gutter marker");
    assert.match(await page.locator(".cm-lineNumbers .cm-diagnostic-line").first().evaluate(node => getComputedStyle(node).boxShadow), /235, 133, 140/, "gutter marker is red");
    assert.equal(await page.locator(".cm-lineNumbers .cm-diagnostic-line").first().evaluate(node => getComputedStyle(node).color), "rgb(235, 133, 140)", "issue line number is red in dark mode");
    await page.locator(".workspace").evaluate(node => { node.dataset.theme = "light"; });
    assert.equal(await page.locator(".cm-lineNumbers .cm-diagnostic-line").first().evaluate(node => getComputedStyle(node).color), "rgb(155, 51, 45)", "issue line number is red in light mode");
    assert.equal(await tooltip.evaluate(node => getComputedStyle(node).borderLeftColor), "rgb(161, 38, 34)", "tooltip uses the light theme error accent");
    await page.locator(".workspace").evaluate(node => { node.dataset.theme = "dark"; });
    await page.getByRole("button", { name: "Full screen", exact: true }).click();
    assert.equal(await page.locator(".cm-diagnostic").count() > 0, true, "full screen retains check underlines");
    assert.equal(await page.locator(".cm-lineNumbers .cm-diagnostic-line").count(), 1, "full screen retains issue gutter markers");
    await page.locator(".cm-diagnostic").first().hover();
    await tooltip.waitFor();
    assert.equal(await tooltip.locator(".cm-diagnostic-tooltip-badge").innerText(), "ERROR", "full screen retains check tooltips");
    await page.getByRole("button", { name: "Exit full screen", exact: true }).click();
    await page.evaluate(source => window.acornEditors.setDiagnostics("content", [
        { line: 9, character: 13, endLine: 9, endCharacter: 21, severity: "warning", message: "Status check" },
        { line: 3, character: 14, endLine: 3, endCharacter: 23, severity: "error", message: "Title check" },
        { line: 3, character: 14, endLine: 3, endCharacter: 23, severity: "warning", message: "Same value check" },
        { line: 5, character: 7, severity: "info", message: "Informational check" },
        { severity: "error", message: "Document check" }
    ], source), original);
    const issueNav = page.getByRole("group", { name: "Issue navigation" });
    assert.equal(await issueNav.isVisible(), true, "source issues reveal navigation controls");
    assert.deepEqual(await issueNav.locator("button").allTextContents(), ["prev", "next"], "issue controls use text labels");
    assert.equal(await issueNav.evaluate(node => node.nextElementSibling?.classList.contains("editor-fullscreen-toggle")), true, "issue controls precede full screen");
    assert.equal(await issueNav.locator(".editor-issue-position").innerText(), "0 of 2", "same-value checks share one stop");
    assert.equal(await issueNav.locator(".editor-issue-message").innerText(), "", "unselected issues have no generic message");
    await page.getByRole("button", { name: "Previous issue" }).click();
    assert.equal(await page.evaluate(() => window.getSelection().toString()), '"active"', "previous starts at the last source issue");
    assert.equal(await issueNav.locator(".editor-issue-position").innerText(), "2 of 2");
    assert.equal(await issueNav.locator(".editor-issue-message").innerText(), "Warning, line 9: Status check", "issue navigation exposes the message without hovering");
    assert.equal(await issueNav.locator(".editor-issue-message").getAttribute("data-severity"), "warning", "warning messages retain their severity");
    assert.equal(await page.locator(".cm-diagnostic-line-warning").count(), 1, "warning lines have their own marker");
    assert.equal(await page.evaluate(() => document.activeElement?.getAttribute("aria-label")), "Previous issue", "clicking keeps issue controls focused");
    await page.keyboard.press("ArrowRight");
    assert.equal(await page.evaluate(() => window.getSelection().toString()), '"ACORN 🌳"', "right arrow wraps to the first source issue");
    assert.equal(await issueNav.locator(".editor-issue-position").innerText(), "1 of 2");
    await page.keyboard.press("ArrowRight");
    assert.equal(await issueNav.locator(".editor-issue-position").innerText(), "2 of 2");
    await page.keyboard.press("ArrowLeft");
    assert.equal(await issueNav.locator(".editor-issue-position").innerText(), "1 of 2");
    await page.getByRole("button", { name: "Next issue" }).click();
    assert.equal(await issueNav.locator(".editor-issue-position").innerText(), "2 of 2");
    await page.evaluate(() => window.acornEditors.reveal("content", 3, 14, 3, 23));
    assert.equal(await page.evaluate(() => window.getSelection().toString()), '"ACORN 🌳"', "source ranges select the entire value");
    assert.equal(await issueNav.locator(".editor-issue-position").innerText(), "1 of 2", "location links update navigation position");
    await page.evaluate(source => window.acornEditors.setDiagnostics("content", [{ line: 2, character: 14, endLine: 8, endCharacter: 4, severity: "error", message: "Project check" }], source), original);
    assert.equal(await page.locator(".cm-lineNumbers .cm-diagnostic-line").count(), 7, "multiline values mark every affected line");
    await page.evaluate(() => window.acornEditors.reveal("content", 2, 14, 8, 4));
    assert.match(await page.evaluate(() => window.getSelection().toString()), /^\{[\s\S]*"team"[\s\S]*\}$/, "multiline values select their full source range");
    await page.evaluate(source => window.acornEditors.setDiagnostics("content", [{ line: 1, character: 1, severity: "error", message: "Syntax check" }], source), original);
    assert.equal(await page.locator(".cm-lineNumbers .cm-diagnostic-line").count(), 1, "syntax issues mark a line without an underline range");
    assert.equal(await page.locator(".cm-diagnostic").count(), 0);
    assert.equal(await issueNav.locator(".editor-issue-position").innerText(), "0 of 1", "line-only syntax issues are navigable");
    await page.getByRole("button", { name: "Next issue" }).click();
    assert.equal(await issueNav.locator(".editor-issue-position").innerText(), "1 of 1");
    assert.equal(await issueNav.locator(".editor-issue-message").innerText(), "Error, line 1: Syntax check", "line-only issues expose their message");
    await page.evaluate(() => window.acornEditors.sync("content", null, "json", "Before"));
    assert.equal(await editor.getAttribute("aria-label"), "Before", "route changes update the accessible label");
    await editor.press("ArrowRight");
    assert.equal(await editor.getAttribute("aria-label"), "Before", "cursor movement preserves the updated label");
    assert.equal(await issueNav.locator(".editor-issue-position").innerText(), "1 of 1", "plain arrows in the editor keep moving the caret");
    async function replace(value, language = "json", fold = false) {
        await page.evaluate(({ value, language, fold }) => {
            const input = document.getElementById("content");
            input.value = value;
            input.closest(".code-editor").dataset.language = language;
            window.acornEditors.sync("content", value, language);
            if (fold) window.acornEditors.fold("content");
        }, { value, language, fold });
    }
    const longSource = Array.from({ length: 100 }, (_, index) => `line ${index + 1}`).join("\n");
    await replace(longSource, "markdown");
    await page.evaluate(() => {
        for (const side of ["top", "bottom"]) {
            const spacer = document.createElement("div");
            spacer.dataset.issueScrollSpacer = side;
            spacer.style.height = "900px";
            if (side === "top") document.body.prepend(spacer);
            else document.body.append(spacer);
        }
    });
    await page.evaluate(source => window.acornEditors.setDiagnostics("content", [{ line: 50, character: 1, severity: "error", message: "Middle line" }], source), longSource);
    await page.getByRole("button", { name: "Next issue" }).click();
    await page.waitForFunction(() => {
        const line = document.querySelector(".code-editor .cm-activeLine")?.getBoundingClientRect();
        const scroller = document.querySelector(".code-editor .cm-scroller")?.getBoundingClientRect();
        return line && scroller && Math.abs((line.top + line.bottom - scroller.top - scroller.bottom) / 2) < 25 && Math.abs((line.top + line.bottom - innerHeight) / 2) < 40;
    });
    await page.getByRole("button", { name: "Full screen", exact: true }).click();
    await page.evaluate(source => window.acornEditors.setDiagnostics("content", [{ line: 80, character: 1, severity: "error", message: "Fullscreen line" }], source), longSource);
    await page.getByRole("button", { name: "Next issue" }).click();
    await page.waitForFunction(() => {
        const line = document.querySelector(".code-editor.is-fullscreen .cm-activeLine")?.getBoundingClientRect();
        const scroller = document.querySelector(".code-editor.is-fullscreen .cm-scroller")?.getBoundingClientRect();
        return line && scroller && Math.abs((line.top + line.bottom - scroller.top - scroller.bottom) / 2) < 25;
    });
    await page.getByRole("button", { name: "Exit full screen", exact: true }).click();
    await page.evaluate(() => document.querySelectorAll("[data-issue-scroll-spacer]").forEach(spacer => spacer.remove()));
    const largeJson = JSON.stringify({ items: Array.from({ length: 2000 }, (_, id) => ({ id, title: "Research" })) }, null, 2);
    await replace(largeJson, "json", true);
    assert.equal(await page.locator(".cm-diagnostic").count(), 0, "replacing the document clears stale check underlines");
    assert.equal(await page.locator(".cm-lineNumbers .cm-diagnostic-line").count(), 0, "replacing the document clears stale gutter markers");
    assert.equal(await issueNav.isVisible(), false, "replacing the document hides stale issue navigation");
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
            await editor.locator(".tok-propertyName").filter({ hasText: '"title"' }).dblclick();
            assert.equal(await page.evaluate(() => window.getSelection().toString()), "title", "double-click selects a single word");
            const selection = page.locator(".cm-selectionBackground").first();
            await selection.waitFor();
            const bounds = await selection.boundingBox();
            const clip = { x: Math.ceil(bounds.x + 2), y: Math.floor(bounds.y + bounds.height - 2), width: Math.ceil(bounds.width + 8), height: 1 };
            const screenshot = await page.screenshot({ clip });
            const pixels = await page.evaluate(async (image) => {
                const bitmap = new Image();
                bitmap.src = `data:image/png;base64,${image}`;
                await bitmap.decode();
                const canvas = document.createElement("canvas");
                canvas.width = bitmap.width;
                canvas.height = 1;
                const context = canvas.getContext("2d");
                context.drawImage(bitmap, 0, 0);
                return [0, bitmap.width - 1].map(x => Array.from(context.getImageData(x, 0, 1, 1).data));
            }, screenshot.toString("base64"));
            const [selected, unselected] = pixels;
            assert.ok(selected[1] - selected[0] > unselected[1] - unselected[0] + 15, `${theme}: single-word selection visibly paints green over the active line`);
            await editor.press("ArrowRight");
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
    await page.evaluate(() => window.acornEditors.openFind("after"));
    const readonlyHost = page.locator(".code-editor").filter({ has: page.locator("#after") });
    assert.ok(await readonlyHost.locator(".cm-search input[aria-label='Find']").isVisible(), "read-only previews can be searched");
    assert.equal(await readonlyHost.locator(".cm-search input[aria-label='Replace']").count(), 0, "read-only previews do not offer replacement");
    await readonlyHost.locator(".cm-search button[name='close']").click();
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
    const mobileContext = await browser.newContext({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true, deviceScaleFactor: 2 });
    const mobile = await mobileContext.newPage();
    await mobile.goto(`http://127.0.0.1:${server.address().port}`);
    await mobile.evaluate(() => {
        const viewport = document.createElement("meta");
        viewport.name = "viewport";
        viewport.content = "width=device-width,initial-scale=1";
        document.head.append(viewport);
    });
    await mobile.addStyleTag({ content: await readFile(new URL("../assets/app.css", import.meta.url), "utf8") });
    await mobile.addScriptTag({ content: await readFile(new URL("../assets/generated/editor/editor.js", import.meta.url), "utf8") });
    await mobile.evaluate(() => {
        const toolbar = document.querySelector(".editor-toolbar");
        const find = document.createElement("button");
        find.className = "copy-code";
        find.textContent = "Find";
        find.onclick = () => window.acornEditors.openFind("content");
        const sizes = document.createElement("div");
        sizes.className = "editor-font-size";
        sizes.setAttribute("role", "group");
        sizes.setAttribute("aria-label", "Editor font size");
        for (const size of ["small", "big"]) {
            const button = document.createElement("button");
            button.dataset.size = size;
            button.textContent = size === "small" ? "Small" : "Big";
            button.onclick = () => window.acornEditors.setFontSize("content", size);
            sizes.append(button);
        }
        toolbar.prepend(find, sizes);
    });
    assert.equal(await mobile.locator(".code-editor").evaluate(node => getComputedStyle(node).fontSize), "16px", "touch editors start at a readable size");
    assert.equal(await mobile.locator(".editor-resize-handle").evaluate(node => getComputedStyle(node).height), "32px", "touch resize handle has a larger target");
    const touchGrip = await mobile.locator(".editor-resize-handle").boundingBox();
    const touchSurface = await mobile.locator(".editor-surface").boundingBox();
    const touchX = touchGrip.x + touchGrip.width / 2;
    const touchY = touchGrip.y + touchGrip.height / 2;
    const touchSession = await mobileContext.newCDPSession(mobile);
    await touchSession.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x: touchX, y: touchY }] });
    await touchSession.send("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: [{ x: touchX, y: touchY + 80 }] });
    await touchSession.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
    assert.ok((await mobile.locator(".editor-surface").boundingBox()).height >= touchSurface.height + 75, "touch dragging resizes the editor");
    await mobile.locator("#content-editor").tap();
    await mobile.locator("#content-editor").focus();
    await mobile.keyboard.insertText('{"touch":true}');
    await mobile.waitForFunction(() => document.getElementById("content")?.value === '{"touch":true}');
    assert.equal(await mobile.locator("#content").inputValue(), '{"touch":true}', "touch editing updates the source input");
    await mobile.getByRole("button", { name: "Big" }).tap();
    assert.equal(await mobile.locator(".code-editor").evaluate(node => getComputedStyle(node).fontSize), "20px", "touch font control changes size");
    await mobile.getByRole("button", { name: "Find" }).tap();
    assert.ok(await mobile.locator(".cm-search input[aria-label='Find']").isVisible(), "touch opens editor search");
    assert.ok(await mobile.locator(".code-editor").evaluate(node => node.scrollWidth <= node.clientWidth), "narrow editor controls do not overflow horizontally");
    await mobile.getByRole("button", { name: "Full screen", exact: true }).tap();
    assert.ok((await mobile.locator(".code-editor").boundingBox()).height <= 844, "touch full screen stays within the viewport");
    await mobile.getByRole("button", { name: "Exit full screen", exact: true }).tap();
    await mobileContext.close();
    assert.deepEqual(errors, []);
    console.log("CodeMirror editing, search and replace, diagnostics, touch layout, resizing, folding, undo/redo, clipboard, large inputs, remounts and both themes passed.");
} finally {
    await browser.close();
    await new Promise(resolve => server.close(resolve));
}
