import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, resolve, sep } from "node:path";
import { chromium } from "playwright";

const root = resolve(process.argv[2] ?? "target/dx/acorn-web/release/web/public");
const activity = await readFile(new URL("../assets/examples/publications.json", import.meta.url), "utf8");
const withoutNulls = (value) => JSON.parse(JSON.stringify(value, (_key, item) => item === null ? undefined : item));
const types = { ".css": "text/css", ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".svg": "image/svg+xml", ".wasm": "application/wasm" };
const server = createServer(async (request, response) => {
    try {
        const requested = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
        const path = resolve(root, `.${requested}`);
        assert.ok(path === root || path.startsWith(`${root}${sep}`));
        const file = await stat(path).then((info) => info.isDirectory() ? resolve(path, "index.html") : path).catch(() => path);
        const bytes = await readFile(file).catch(() => null);
        response.writeHead(bytes ? 200 : 404, { "Content-Type": bytes ? types[extname(file)] ?? "application/octet-stream" : "text/html" });
        response.end(bytes ?? await readFile(resolve(root, "index.html")));
    } catch {
        if (!response.headersSent) response.writeHead(404);
        response.end();
    }
});
await new Promise((ready) => server.listen(0, "127.0.0.1", ready));
const origin = `http://127.0.0.1:${server.address().port}`;
const browser = await chromium.launch();
try {
    const failures = [];
    const supported = await browser.newContext();
    await supported.addInitScript(() => {
        window.__acornTools = new Map();
        Object.defineProperty(document, "modelContext", { configurable: true, value: {
            async registerTool(tool, { signal }) {
                window.__acornTools.set(tool.name, tool);
                signal.addEventListener("abort", () => window.__acornTools.delete(tool.name), { once: true });
            },
        } });
    });
    const page = await supported.newPage();
    page.on("pageerror", (error) => failures.push(error.message));
    page.on("request", (request) => { if (/^https?:/.test(request.url()) && !request.url().startsWith(`${origin}/`)) failures.push(`External request: ${request.url()}`); });
    await page.goto(`${origin}/settings`);
    await page.waitForFunction(() => document.getElementById("webmcp-status")?.textContent === "WebMCP ✔ installed" && window.__acornTools?.size === 9);
    const names = await page.evaluate(() => [...window.__acornTools.keys()]);
    assert.deepEqual(names, [
        "acorn_assess_fair_metadata", "acorn_check_readability", "acorn_compare_big_o", "acorn_crosswalk_metadata",
        "acorn_diff_content", "acorn_discover_persistent_identifiers", "acorn_inspect_persistent_identifier",
        "acorn_validate_metadata_standard", "acorn_validate_research_activity",
    ]);
    assert.equal(await page.evaluate(() => [...window.__acornTools.values()].every((tool) => tool.inputSchema?.additionalProperties === false && tool.annotations?.readOnlyHint === true && tool.annotations?.consequentialHint === false)), true);
    await page.locator('nav a[title="Home"]').click();
    await page.getByRole("heading", { name: "Research Intelligence", exact: true }).waitFor();
    await page.locator('nav a[title="Settings"]').click();
    await page.waitForFunction(() => document.getElementById("webmcp-status")?.textContent === "WebMCP ✔ installed");
    assert.equal(await page.evaluate(() => window.__acornTools.size), 9);
    const specs = [
        [names[0], { json: activity, standard: "research-activity-data" }, "fair", { input: activity }, ["findings", "profile"]],
        [names[1], { metric: "fkgl", text: "The team published clear research results." }, "readability", { input: "The team published clear research results.", metric: "fkgl" }, ["checks", "summary"]],
        [names[2], { left: "O(n)", right: "O(n^2)" }, "big-o", { input: "O(n)", after: "O(n^2)" }, ["dominance", "left", "right"]],
        [names[3], { content: activity, format: "json", source: "research-activity-data", target: "datacite" }, "crosswalk", { input: activity, target: "datacite" }, ["content", "warnings"]],
        [names[4], { before: "before\n", after: "after\n", mode: "unified" }, "diff", { input: "before\n", after: "after\n" }, ["changed", "diff"]],
        [names[5], { text: "doi:10.1000/182" }, "discover", { input: "doi:10.1000/182" }, ["count", "identifiers"]],
        [names[6], { value: "https://doi.org/10.1000/182" }, "inspect", { input: "https://doi.org/10.1000/182" }, ["status", "kind", "normalized"]],
        [names[7], { content: activity, format: "json", standard: "research-activity-data" }, "metadata-standard", { input: activity }, ["valid"]],
        [names[8], { json: activity, fuzzy: "off" }, "research-activity", { input: activity }, ["valid"]],
    ];
    const baseRequest = {
        input: "", after: "", standard: "research-activity-data", format: "json", target: "datacite", metric: "fres",
        threshold: "", activityMetric: "fkgl", activityMax: "", mode: "unified", kind: "", fuzzy: "off",
        skip: [], contact: "", localIdentifier: "", assumptions: "{}",
    };
    for (const [name, input, slug, request, fields] of specs) {
        const outputs = await page.evaluate(async ({ name, input, slug, request, baseRequest }) => {
            const adapter = await window.__acornTools.get(name).execute(input);
            const worker = await window.acornOperations.run(JSON.stringify([slug, { ...baseRequest, ...request }]));
            return { adapter, manual: worker.reply ? JSON.parse(worker.reply) : worker };
        }, { name, input, slug, request, baseRequest });
        assert.equal(outputs.adapter.ok, true, `${name}: ${JSON.stringify(outputs.adapter)}`);
        assert.ok(outputs.manual.ok, `${slug}: ${JSON.stringify(outputs.manual)}`);
        for (const field of fields) assert.deepEqual(withoutNulls(outputs.adapter.result[field]), withoutNulls(outputs.manual.ok[field]), `${name}.${field}`);
    }
    const policy = await page.evaluate(async () => {
        const tool = window.__acornTools.get("acorn_check_readability");
        const input = { metric: "fkgl", text: "The team published clear research results." };
        const first = tool.execute(input);
        const busy = await tool.execute(input);
        const completed = await first;
        const controller = new AbortController();
        controller.abort();
        const cancelled = await tool.execute(input, { signal: controller.signal });
        const oversized = await tool.execute({ metric: "fkgl", text: "x".repeat(16385) });
        const malformed = await tool.execute({ metric: "invalid", text: "Text." });
        return { busy, cancelled, completed, malformed, oversized };
    });
    assert.equal(policy.completed.ok, true);
    assert.equal(policy.busy.error.code, "busy");
    assert.equal(policy.cancelled.error.code, "cancelled");
    assert.equal(policy.oversized.error.code, "input_too_large");
    assert.equal(policy.malformed.error.code, "invalid_arguments");
    await page.evaluate(() => window.dispatchEvent(new PageTransitionEvent("pagehide", { persisted: false })));
    assert.equal(await page.evaluate(() => window.__acornTools.size), 0);
    await supported.close();

    const unsupported = await browser.newPage();
    unsupported.on("pageerror", (error) => failures.push(error.message));
    await unsupported.goto(`${origin}/settings`);
    await unsupported.waitForFunction(() => document.getElementById("webmcp-status")?.textContent === "WebMCP ❌ unsupported");
    await unsupported.goto(`${origin}/review`);
    await unsupported.getByRole("button", { name: "Use example", exact: true }).click();
    await unsupported.getByRole("button", { name: "Validate", exact: true }).click();
    await unsupported.locator(".results .status").first().waitFor();
    await unsupported.close();

    const rejected = await browser.newContext();
    await rejected.addInitScript(() => {
        window.__acornTools = new Map();
        Object.defineProperty(document, "modelContext", { configurable: true, value: {
            async registerTool(tool, { signal }) {
                if (tool.name === "acorn_diff_content") throw Error("registration rejected");
                window.__acornTools.set(tool.name, tool);
                signal.addEventListener("abort", () => window.__acornTools.delete(tool.name), { once: true });
            },
        } });
    });
    const rejectedPage = await rejected.newPage();
    rejectedPage.on("pageerror", (error) => failures.push(error.message));
    await rejectedPage.goto(`${origin}/settings`);
    await rejectedPage.waitForFunction(() => document.getElementById("webmcp-status")?.textContent === "WebMCP ❌ failed (manual tools available)");
    assert.equal(await rejectedPage.evaluate(() => window.__acornTools.size), 0);
    await rejectedPage.goto(`${origin}/review`);
    await rejectedPage.getByRole("button", { name: "Use example", exact: true }).click();
    await rejectedPage.getByRole("button", { name: "Validate", exact: true }).click();
    await rejectedPage.locator(".results .status").first().waitFor();
    await rejected.close();
    assert.deepEqual(failures, []);
    console.log(`WebMCP workspace smoke passed in Chromium ${browser.version()}: nine registered schemas and worker parity, route remount without duplicate registration, busy/cancel/bounds, partial-failure cleanup, unsupported manual route, and final disposal.`);
} finally {
    await browser.close();
    await new Promise((closed) => server.close(closed));
}
