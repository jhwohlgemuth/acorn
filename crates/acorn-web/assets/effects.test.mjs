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

test("chrome follows system changes only until an explicit theme choice and resets without persistence", async () => {
    const { readFile } = await import("node:fs/promises");
    const { runInNewContext } = await import("node:vm");
    const context = { window: {}, document: { getElementById: () => null, querySelector: () => null }, setTimeout, clearTimeout };
    runInNewContext(await readFile(new URL("effects.js", import.meta.url), "utf8"), context);
    const states = [];
    const theme = context.createTheme(true, (dark, system) => states.push([dark, system]));
    theme.sync();
    theme.update(false);
    theme.toggle();
    theme.update(false);
    theme.reset();
    theme.update(true);
    assert.deepEqual(states, [[true, true], [false, true], [true, false], [true, false], [false, true], [true, true]]);
});

test("leaving the drawer layout closes an open drawer through its state-owning control", async () => {
    const { readFile } = await import("node:fs/promises");
    const { runInNewContext } = await import("node:vm");
    const window = new EventTarget();
    const header = { display: "flex" };
    let expanded = false;
    let clicks = 0;
    const toggle = {
        parentElement: header,
        getAttribute: (name) => name === "aria-expanded" ? String(expanded) : null,
        click() { clicks += 1; expanded = !expanded; },
    };
    window.matchMedia = () => Object.assign(new EventTarget(), { matches: false });
    window.getComputedStyle = (element) => { assert.equal(element, header); return element; };
    const context = {
        window,
        document: {
            getElementById: (id) => id === "nav-toggle" ? toggle : null,
            querySelector: (selector) => selector === ".workspace" ? { dataset: {} } : null,
        },
        setTimeout, clearTimeout,
    };
    runInNewContext(await readFile(new URL("effects.js", import.meta.url), "utf8"), context);
    window.dispatchEvent(new Event("resize"));
    assert.equal(clicks, 0, "resizing a closed drawer must not open it");
    expanded = true;
    window.dispatchEvent(new Event("resize"));
    assert.equal(clicks, 0, "the drawer stays open while its mobile header is visible");
    header.display = "none";
    window.dispatchEvent(new Event("resize"));
    assert.equal(expanded, false, "desktop layout must clear the drawer state that controls main inertness");
    assert.equal(clicks, 1);
    window.dispatchEvent(new Event("resize"));
    header.display = "flex";
    window.dispatchEvent(new Event("resize"));
    assert.equal(clicks, 1, "further resize events must not reopen the drawer");
    expanded = true;
    header.display = "none";
    window.dispatchEvent(new Event("resize"));
    assert.equal(expanded, false, "later mobile-to-desktop transitions also clear the state");
});

test("notifications replace and expire without a stale timer removing a newer message", async () => {
    const { readFile } = await import("node:fs/promises");
    const { runInNewContext } = await import("node:vm");
    const context = { window: {}, document: { getElementById: () => null, querySelector: () => null }, setTimeout, clearTimeout };
    runInNewContext(await readFile(new URL("effects.js", import.meta.url), "utf8"), context);
    const pending = new Map();
    let nextId = 0;
    let shown = "";
    const notifications = context.createNotifier((message) => { shown = message; }, (callback) => { const id = ++nextId; pending.set(id, callback); return id; }, (id) => pending.delete(id));
    notifications.notify("first");
    notifications.notify("second");
    assert.equal(shown, "second");
    assert.equal(pending.size, 1);
    pending.values().next().value();
    assert.equal(shown, "");
    notifications.notify("third");
    notifications.dismiss();
    assert.equal(shown, "");
    assert.equal(pending.size, 0);
});

test("tour handles reduced motion, completion, route teardown and missing targets", async () => {
    const { readFile } = await import("node:fs/promises");
    const { runInNewContext } = await import("node:vm");
    const context = { window: {}, document: { getElementById: () => null, querySelector: () => null }, setTimeout, clearTimeout };
    runInNewContext(await readFile(new URL("effects.js", import.meta.url), "utf8"), context);
    let config;
    let focused = 0;
    let driven = 0;
    let missing = false;
    const notices = [];
    const trigger = { isConnected: true, focus: () => { focused += 1; } };
    const tour = context.createTour(async () => ({ driver: (options) => {
        config = options;
        return { drive: () => { driven += 1; }, destroy: () => options.onDestroyed(null, null, { reason: "api" }) };
    } }), { find: (selector) => missing && selector === "#local-entry" ? null : trigger, notify: (message) => notices.push(message), reducedMotion: () => true, theme: () => "light" });
    await tour.start();
    assert.equal(config.animate, false);
    assert.equal(config.allowKeyboardControl, true);
    assert.equal(config.closeBtnLabel, "Close tour");
    assert.equal(config.showProgress, true);
    config.onDestroyed(null, null, { reason: "done" });
    assert.equal(focused, 1);
    assert.match(notices.at(-1), /Tour complete/);
    await tour.start();
    tour.cancel();
    assert.equal(focused, 1, "route teardown must not steal heading focus");
    assert.equal(notices.length, 1);
    missing = true;
    await tour.start();
    assert.equal(driven, 2);
    assert.match(notices.at(-1), /unavailable/);
});

test("a tour import finishing after navigation cannot reopen the tour", async () => {
    const { readFile } = await import("node:fs/promises");
    const { runInNewContext } = await import("node:vm");
    const context = { window: {}, document: { getElementById: () => null, querySelector: () => null }, setTimeout, clearTimeout };
    runInNewContext(await readFile(new URL("effects.js", import.meta.url), "utf8"), context);
    let resolve;
    let opened = false;
    const tour = context.createTour(() => new Promise((ready) => { resolve = ready; }), { find: () => ({ isConnected: true }), notify: () => assert.fail("cancelled import must be silent") });
    const pending = tour.start();
    tour.cancel();
    resolve({ driver: () => { opened = true; } });
    await pending;
    assert.equal(opened, false);
});
