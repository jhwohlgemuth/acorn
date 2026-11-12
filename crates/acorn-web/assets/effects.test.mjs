import assert from "node:assert/strict";
import { setTimeout } from "node:timers/promises";
import test from "node:test";

async function effectContext(extra = {}) {
    const { readFile } = await import("node:fs/promises");
    const { runInNewContext } = await import("node:vm");
    const context = { window: {}, document: { getElementById: () => null, querySelector: () => null }, TextDecoder, setTimeout, clearTimeout, ...extra };
    runInNewContext(await readFile(new URL("effects.js", import.meta.url), "utf8"), context);
    return context;
}
test("map origins allow both loopback names and reject remote development imports", async () => {
    const context = await effectContext();
    for (const page of ["http://127.0.0.1:9000", "http://localhost:9000"]) {
        for (const module of ["http://127.0.0.1:8083/src/main.tsx", "http://localhost:8083/src/main.tsx"]) {
            assert.equal(context.isLocalMapUrl(new URL(module), new URL(page)), true);
        }
    }
    assert.equal(context.isLocalMapUrl(new URL("https://example.org/map.js"), new URL("https://example.org/app")), true);
    for (const [module, page] of [
        ["http://127.0.0.1:8083/map.js", "https://example.org"],
        ["https://example.org/map.js", "http://localhost:9000"],
        ["http://localhost.evil:8083/map.js", "http://localhost:9000"],
        ["ftp://localhost:8083/map.js", "http://localhost:9000"],
    ]) assert.equal(context.isLocalMapUrl(new URL(module), new URL(page)), false);
});
test("rejected or malformed map URLs deliver the accessible fallback", async () => {
    const errors = [];
    const context = await effectContext({ URL, location: new URL("http://localhost:9000"), console: { error() {} }, document: { baseURI: "http://localhost:9000/", getElementById: () => null, querySelector: () => null } });
    for (const url of ["https://remote.example/map.js", "http://["]) {
        await context.window.acornMap.connect(url, "", (event) => errors.push(event));
    }
    assert.equal(errors.length, 2);
    assert.ok(errors.every((event) => event.type === "error" && /table/.test(event.message)));
});
test("workspace input enforces UTF-8, size, path, duplicate and partial-success boundaries", async () => {
    const context = await effectContext();
    const file = (path, content) => ({ path, file: new Blob([content]) });
    const result = await context.readWorkspaceFiles([
        file("b/valid.json", "{}"), file("a.txt", "hello"), file("a.txt", "duplicate"), file("../escape.json", "{}"),
        file("binary.json", new Uint8Array([0xff])), file("large.txt", "x".repeat(262145)), file("wrong.exe", "x"),
        { path: "unreadable.txt", file: { size: 1, arrayBuffer: async () => { throw new Error("denied"); } } },
    ]);
    assert.deepEqual(JSON.parse(JSON.stringify(result)), { files: [{ path: "a.txt", content: "hello" }, { path: "b/valid.json", content: "{}" }], skipped: 6 });
    assert.equal(context.validWorkspacePath("a/".repeat(20) + "x.json"), false);
    assert.equal(context.validWorkspacePath("a\\x.json"), false);
    const many = await context.readWorkspaceFiles(Array.from({ length: 501 }, (_, i) => file(`${i}.json`, "{}")));
    assert.equal(many.files.length, 500);
    assert.equal(many.skipped, 1);
    const total = await context.readWorkspaceFiles(Array.from({ length: 65 }, (_, i) => file(`${i}.txt`, "x".repeat(262144))));
    assert.equal(total.files.length, 64);
    assert.equal(total.skipped, 1);
});
test("folder drops exhaust directory batches and preserve sorted relative paths", async () => {
    const context = await effectContext();
    const leaf = (name) => ({ name, isFile: true, file: (resolve) => resolve(new Blob(["{}"])) });
    let reads = 0;
    const folder = { name: "root", isDirectory: true, createReader: () => ({ readEntries(resolve) { resolve([[leaf("z.json")], [leaf("a.json")], []][reads++]); } }) };
    const result = await context.readWorkspaceEntries([folder]);
    assert.equal(reads, 3);
    assert.deepEqual(Array.from(result.files, (file) => file.path), ["root/a.json", "root/z.json"]);
});
test("a pending drop cannot replace the session after leaving Home", async () => {
    const context = await effectContext();
    const deliveries = [];
    let resolve;
    const file = { name: "late.json", size: 2, arrayBuffer: () => new Promise((done) => { resolve = done; }) };
    context.window.acornWorkspace.connect((batch) => deliveries.push(batch));
    context.window.acornWorkspace.drop({ dataTransfer: { files: [file], items: [] } });
    context.window.acornWorkspace.disconnect();
    resolve(new TextEncoder().encode("{}").buffer);
    await setTimeout(0);
    assert.equal(deliveries.length, 0);
});
test("skill exports reject mismatched, corrupt and incomplete bundles", async () => {
    const { createHash } = await import("node:crypto");
    const context = await effectContext();
    const definition = new TextEncoder().encode("Skill definition");
    const archive = new TextEncoder().encode("Archive bytes");
    const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
    const manifest = { schemaVersion: 1, appVersion: "1.0", compatibility: { cli: "1.0" }, skillRevision: "a".repeat(64), files: { "SKILL.md": hash(definition) }, definition: { file: "SKILL.md", sha256: hash(definition) }, archive: { file: "acorn-skill.zip", sha256: hash(archive) } };
    assert.equal((await context.verifySkill(manifest, definition, archive, "1.0", hash)).definition, "Skill definition");
    await assert.rejects(context.verifySkill(manifest, definition, archive, "2.0", hash), /does not match/);
    await assert.rejects(context.verifySkill(manifest, definition, new Uint8Array([1]), "1.0", hash), /integrity/);
    await assert.rejects(context.verifySkill({ ...manifest, files: {} }, definition, archive, "1.0", hash), /does not match/);
    await assert.rejects(context.verifySkill({ ...manifest, schemaVersion: 2 }, definition, archive, "1.0", hash), /does not match/);
});
test("format-specific downloads preserve bytes and revoke object URLs", async () => {
    const blobs = [];
    const timers = [];
    const revoked = [];
    const link = { click() { this.clicked = true; } };
    const context = await effectContext({ Blob, URL: { createObjectURL(blob) { blobs.push(blob); return "blob:test"; }, revokeObjectURL(url) { revoked.push(url); } }, setTimeout: (callback) => timers.push(callback), document: { getElementById: () => null, querySelector: () => null, createElement: () => link } });
    context.window.acornDownload("+literal <script>\n", "../changes.diff", "text/plain;charset=utf-8");
    assert.equal(link.download, ".._changes.diff");
    assert.equal(link.clicked, true);
    assert.equal(blobs[0].type, "text/plain;charset=utf-8");
    assert.equal(await blobs[0].text(), "+literal <script>\n");
    timers[0]();
    assert.deepEqual(revoked, ["blob:test"]);
});
test("explicit theme tiles remain selected across system changes", async () => {
    const context = await effectContext();
    const states = [];
    const theme = context.createTheme(true, (dark, system) => states.push([dark, system]));
    theme.choose("light"); theme.update(true); theme.choose("dark"); theme.update(false); theme.choose("system"); theme.choose("invalid");
    assert.deepEqual(states, [[false, false], [false, false], [true, false], [true, false], [false, true]]);
});

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
        window, Event,
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
