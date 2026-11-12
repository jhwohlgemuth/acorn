import assert from "node:assert/strict";
import { setTimeout } from "node:timers/promises";
import test from "node:test";

test("WebMCP survives cached navigation and disposes on final unload", async (t) => {
    const globals = ["document", "window"].map((name) => [name, Object.getOwnPropertyDescriptor(globalThis, name)]);
    t.after(() => globals.forEach(([name, descriptor]) => {
        if (descriptor) Object.defineProperty(globalThis, name, descriptor);
        else delete globalThis[name];
    }));
    const tools = new Map();
    const modelContext = {
        async registerTool(tool, { signal }) {
            tools.set(tool.name, tool);
            signal.addEventListener("abort", () => tools.delete(tool.name), { once: true });
        },
    };
    const status = { dataset: { adapter: new URL("../../acorn-wasm/web/webmcp.js", import.meta.url).href }, textContent: "initializing" };
    const names = ["assessFairMetadata", "checkReadability", "compareBigO", "crosswalkMetadata", "diffContent", "discoverPersistentIdentifiers", "inspectPersistentIdentifier", "validateMetadataStandard", "validateResearchActivity"];
    const bindings = names.map((name) => `export function ${name}() { return { valid: true, diagnostics: [] }; }`).join("\n");
    const window = new EventTarget();
    Object.defineProperties(globalThis, {
        document: { configurable: true, value: {
            getElementById: () => status,
            modelContext,
            querySelector: () => ({ src: `data:text/javascript,${encodeURIComponent(bindings)}` }),
        } },
        window: { configurable: true, value: window },
    });
    await import("./effects.js");
    for (let attempt = 0; attempt < 100 && status.textContent === "initializing"; attempt += 1) await setTimeout(5);
    assert.equal(status.textContent, "WebMCP: installed");
    assert.equal(tools.size, 9);
    for (let visit = 0; visit < 2; visit += 1) {
        window.dispatchEvent(Object.assign(new Event("pagehide"), { persisted: true }));
        window.dispatchEvent(Object.assign(new Event("pageshow"), { persisted: true }));
        assert.equal(tools.size, 9);
        assert.equal(status.textContent, "WebMCP: installed");
        const validation = tools.get("acorn_validate_research_activity");
        assert.ok(validation, "validation remains registered after restoration");
        assert.equal((await validation.execute({ json: "{}" })).ok, true);
    }
    window.dispatchEvent(Object.assign(new Event("pagehide"), { persisted: false }));
    assert.equal(tools.size, 0, "final unload still unregisters tools after cached visits");
});
