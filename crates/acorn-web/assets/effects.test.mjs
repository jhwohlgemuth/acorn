import assert from "node:assert/strict";
import { setTimeout } from "node:timers/promises";
import test from "node:test";

async function effectContext(extra = {}) {
    const { readFile } = await import("node:fs/promises");
    const { runInNewContext } = await import("node:vm");
    const context = { window: {}, document: { getElementById: () => null, querySelector: () => null }, TextDecoder, btoa, setTimeout, clearTimeout, ...extra };
    runInNewContext(await readFile(new URL("effects.js", import.meta.url), "utf8"), context);
    return context;
}
function fakeWorker(respond) {
    const worker = { posted: [], terminated: false, terminate() { this.terminated = true; } };
    worker.postMessage = (data) => {
        worker.posted.push(data);
        queueMicrotask(() => worker.onmessage({ data: { id: data.id, ...respond(data) } }));
    };
    return worker;
}
test("a payload that cannot load disables the worker and a trapped run restarts it", async () => {
    const context = await effectContext();
    const broken = fakeWorker(() => ({ stage: "load", failed: "no WASM" }));
    let spawned = 0;
    const unavailable = context.createOperations(() => { spawned += 1; return broken; });
    assert.equal((await unavailable.run("a")).unavailable, "no WASM");
    assert.ok((await unavailable.run("b")).unavailable);
    assert.equal(spawned, 1);
    assert.equal(broken.terminated, true);
    const workers = [fakeWorker(() => ({ stage: "run", failed: "unreachable" })), fakeWorker(() => ({ reply: "ok" }))];
    const restarting = context.createOperations(() => workers.shift());
    assert.match((await restarting.run("a")).error, /stopped unexpectedly/);
    assert.equal((await restarting.run("b")).reply, "ok");
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
test("chrome follows system changes until an explicit theme choice and resets to system", async () => {
    const { readFile } = await import("node:fs/promises");
    const { runInNewContext } = await import("node:vm");
    const context = { window: {}, document: { getElementById: () => null, querySelector: () => null }, setTimeout, clearTimeout };
    runInNewContext(await readFile(new URL("effects.js", import.meta.url), "utf8"), context);
    const states = [];
    const theme = context.createTheme(true, (rendered, mode) => states.push([rendered, mode]));
    theme.sync();
    theme.update(false);
    theme.toggle();
    theme.update(false);
    theme.reset();
    theme.update(true);
    assert.deepEqual(states, [["dark", "system"], ["light", "system"], ["dark", "dark"], ["dark", "dark"], ["light", "system"], ["dark", "system"]]);
});
test("explicit theme tiles remain selected across system changes", async () => {
    const context = await effectContext();
    const states = [];
    const theme = context.createTheme(true, (rendered, mode) => states.push([rendered, mode]));
    theme.choose("light"); theme.update(true); theme.choose("dark"); theme.update(false); theme.choose("system"); theme.choose("invalid");
    assert.deepEqual(states, [["light", "light"], ["light", "light"], ["dark", "dark"], ["dark", "dark"], ["light", "system"]]);
});
test("USWDS persists, ignores system changes, and URL preview stays temporary", async () => {
    const context = await effectContext();
    const saved = new Map();
    const storage = () => ({ getItem: (key) => saved.get(key), setItem: (key, value) => saved.set(key, value), removeItem: (key) => saved.delete(key) });
    const states = [];
    const theme = context.createTheme(true, (rendered, mode, fromUrl) => states.push([rendered, mode, fromUrl]), storage);
    theme.choose("uswds");
    theme.update(false);
    assert.equal(saved.get("acorn.theme.v1"), "uswds");
    const preview = context.createTheme(true, (rendered, mode, fromUrl) => states.push([rendered, mode, fromUrl]), storage, "dark");
    preview.sync();
    assert.equal(saved.get("acorn.theme.v1"), "uswds");
    theme.toggle();
    theme.reset();
    assert.deepEqual(states, [["uswds", "uswds", false], ["uswds", "uswds", false], ["dark", "dark", true], ["dark", "dark", false], ["light", "system", false]]);
    assert.equal(saved.has("acorn.theme.v1"), false);
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
            documentElement: { dataset: {} },
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
test("notifications stack and expire independently", async () => {
    const { readFile } = await import("node:fs/promises");
    const { runInNewContext } = await import("node:vm");
    const context = { window: {}, document: { getElementById: () => null, querySelector: () => null }, setTimeout, clearTimeout };
    runInNewContext(await readFile(new URL("effects.js", import.meta.url), "utf8"), context);
    const pending = new Map();
    let nextId = 0;
    const shown = new Map();
    const notifications = context.createNotifier((message, tone, id) => { if (message) shown.set(id, { message, tone }); else shown.delete(id); }, (callback) => { const id = ++nextId; pending.set(id, callback); return id; }, (id) => pending.delete(id));
    const first = notifications.notify("first", "success");
    const second = notifications.notify("second");
    assert.deepEqual([...shown.values()], [{ message: "first", tone: "success" }, { message: "second", tone: "info" }]);
    assert.equal(pending.size, 2);
    pending.get(first)();
    assert.deepEqual([...shown.values()], [{ message: "second", tone: "info" }]);
    notifications.dismiss(second);
    assert.equal(shown.size, 0);
    notifications.notify("third");
    notifications.dismiss();
    assert.equal(shown.size, 0);
    assert.equal(pending.size, 0);
});
test("operations fall back in-thread when no worker can be created", async () => {
    const context = await effectContext();
    assert.deepEqual({ ...await context.window.acornOperations.run("{}") }, { unavailable: "The operation worker is unavailable." });
    let spawned = 0;
    const throwing = context.createOperations(() => { spawned += 1; throw new Error("blocked"); });
    assert.ok((await throwing.run("{}")).unavailable);
    assert.ok((await throwing.run("{}")).unavailable);
    assert.equal(spawned, 1, "an unavailable worker is not retried for every operation");
});
test("operations return worker replies and count only served operations", async () => {
    const context = await effectContext();
    const worker = fakeWorker(({ message }) => ({ reply: message === null ? null : `reply:${message}` }));
    const operations = context.createOperations(() => worker);
    operations.warm();
    const [first, second] = await Promise.all([operations.run("a"), operations.run("b")]);
    assert.equal(first.reply, "reply:a");
    assert.equal(second.reply, "reply:b");
    assert.equal(operations.status.served, 2);
    assert.deepEqual(worker.posted.map(({ message }) => message), [null, "a", "b"]);
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
test("theme choices persist and resetting clears the saved choice", async () => {
    const context = await effectContext();
    const values = new Map();
    const storage = {
        getItem: (key) => values.get(key) || null,
        setItem: (key, value) => values.set(key, value),
        removeItem: (key) => values.delete(key),
    };
    const states = [];
    const apply = (rendered, mode) => states.push([rendered, mode]);
    const first = context.createTheme(false, apply, () => storage);
    first.sync();
    first.choose("dark");
    assert.equal(values.get("acorn.theme.v1"), "dark");
    const restored = context.createTheme(false, apply, () => storage);
    restored.sync();
    restored.reset();
    assert.equal(values.has("acorn.theme.v1"), false);
    context.createTheme(false, apply, () => storage).sync();
    assert.deepEqual(states, [["light", "system"], ["dark", "dark"], ["dark", "dark"], ["light", "system"], ["light", "system"]]);
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
    let sampleLoaded = false;
    let mobile = false;
    let tourTheme = "light";
    let completed = 0;
    let reset = 0;
    const notices = [];
    const trigger = { isConnected: true, focus: () => { focused += 1; } };
    const tour = context.createTour(async () => ({ driver: (options) => {
        config = options;
        return { drive: () => { driven += 1; }, destroy: () => {} };
    } }), { complete: () => { completed += 1; }, find: (selector) => missing && selector === "#local-entry" || sampleLoaded && selector === "#sample-data" ? null : trigger, notify: (message) => notices.push(message), reducedMotion: () => true, mobile: () => mobile, reset: () => { reset += 1; return true; }, theme: () => tourTheme });
    await tour.start();
    assert.equal(config.animate, false);
    assert.equal(config.allowKeyboardControl, true);
    assert.equal(config.closeBtnLabel, "Close tour");
    assert.equal(config.showProgress, true);
    assert.deepEqual(Array.from(config.steps, (step) => step.popover.title), ["Start with an example", "Open your data", "Load a repository bucket", "Explore your portfolio", "Review metadata", "Find and inspect identifiers", "Convert metadata", "Compare and analyze", "Manage your workspace", "Switch appearance"]);
    config.onDestroyStarted(null, null, { reason: "done" });
    assert.equal(completed, 1);
    assert.equal(focused, 1);
    assert.match(notices.at(-1), /Tour complete/);
    assert.equal(tour.reset(), true);
    assert.equal(reset, 1);
    await tour.start();
    tour.cancel();
    assert.equal(completed, 1, "route teardown must not complete the tour");
    assert.equal(focused, 1, "route teardown must not steal heading focus");
    assert.equal(notices.length, 1);
    sampleLoaded = true;
    mobile = true;
    await tour.start();
    assert.deepEqual(Array.from(config.steps, (step) => step.popover.title), ["Open your data", "Load a repository bucket", "Explore the workspace", "Switch appearance"]);
    tour.cancel();
    tourTheme = "uswds";
    await tour.start();
    assert.deepEqual(Array.from(config.steps, (step) => step.popover.title), ["Open your data", "Load a repository bucket", "Explore the workspace"]);
    tour.cancel();
    missing = true;
    await tour.start();
    assert.equal(driven, 4);
    assert.match(notices.at(-1), /unavailable/);
});
test("URL theme overrides a saved choice without changing storage", async () => {
    const context = await effectContext();
    const values = new Map([["acorn.theme.v1", "dark"]]);
    const storage = {
        getItem: (key) => values.get(key) || null,
        setItem: (key, value) => values.set(key, value),
        removeItem: (key) => values.delete(key),
    };
    const states = [];
    const theme = context.createTheme(true, (rendered, mode, fromUrl) => states.push([rendered, mode, fromUrl]), () => storage, "light");
    theme.sync();
    theme.update(false);
    assert.equal(values.get("acorn.theme.v1"), "dark");
    theme.choose("dark");
    assert.deepEqual(states, [["light", "light", true], ["light", "light", true], ["dark", "dark", false]]);
    assert.equal(values.get("acorn.theme.v1"), "dark");
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
test("workspace input enforces UTF-8, size, path, duplicate and partial-success boundaries", async () => {
    const context = await effectContext();
    const file = (path, content) => ({ path, file: new Blob([content]) });
    const result = await context.readWorkspaceFiles([
        file("b/valid.json", "{}"), file("a.txt", "hello"), file("a.txt", "duplicate"), file("../escape.json", "{}"),
        file("binary.json", new Uint8Array([0xff])), file("large.txt", "x".repeat(2 * 1024 * 1024 + 1)), file("wrong.exe", "x"),
        { path: "unreadable.txt", file: { size: 1, arrayBuffer: async () => { throw new Error("denied"); } } },
    ]);
    assert.deepEqual(JSON.parse(JSON.stringify(result)), { files: [{ path: "a.txt", content: "hello" }, { path: "b/valid.json", content: "{}" }], skipped: 6 });
    assert.equal(context.validWorkspacePath("a/".repeat(20) + "x.json"), false);
    assert.equal(context.validWorkspacePath("a\\x.json"), false);
    const many = await context.readWorkspaceFiles(Array.from({ length: 501 }, (_, i) => file(`${i}.json`, "{}")));
    assert.equal(many.files.length, 500);
    assert.equal(many.skipped, 1);
    const total = await context.readWorkspaceFiles(Array.from({ length: 17 }, (_, i) => file(`${i}.txt`, "x".repeat(2 * 1024 * 1024))));
    assert.equal(total.files.length, 16);
    assert.equal(total.skipped, 1);
});
test("workspace input retains only signature-matched raster images", async () => {
    const context = await effectContext();
    const png = new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10, 0]);
    const jpeg = new Uint8Array([255, 216, 255, 224, 0]);
    const webp = new Uint8Array([82, 73, 70, 70, 4, 0, 0, 0, 87, 69, 66, 80]);
    const result = await context.readWorkspaceFiles([
        { path: "project/index.json", file: new Blob(["{}"]) },
        { path: "project/cover.png", file: new Blob([png]) },
        { path: "project/detail.jpeg", file: new Blob([jpeg]) },
        { path: "project/chart.webp", file: new Blob([webp]) },
        { path: "project/bad.jpg", file: new Blob([png]) },
        { path: "project/bad.webp", file: new Blob(["not webp"]) },
    ]);
    assert.equal(result.files.length, 4);
    assert.equal(result.skipped, 2);
    assert.equal(result.files.find(file => file.path.endsWith("cover.png")).content, `data:image/png;base64,${Buffer.from(png).toString("base64")}`);
    assert.equal(result.files.find(file => file.path.endsWith("detail.jpeg")).content, `data:image/jpeg;base64,${Buffer.from(jpeg).toString("base64")}`);
    assert.equal(result.files.find(file => file.path.endsWith("chart.webp")).content, `data:image/webp;base64,${Buffer.from(webp).toString("base64")}`);
    assert.equal(context.validWorkspacePath("project/cover.jpeg"), true);
});
