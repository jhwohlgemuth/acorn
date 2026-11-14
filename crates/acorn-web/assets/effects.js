// Explicit user-mediated output effects; no content leaves the browser.
window.acornDownload = (text, filename = "acorn-result.json", mime = "application/json;charset=utf-8") => {
    const url = URL.createObjectURL(new Blob([text], { type: ["application/json;charset=utf-8", "application/json-patch+json;charset=utf-8", "text/plain;charset=utf-8"].includes(mime) ? mime : "text/plain;charset=utf-8" }));
    const link = document.createElement("a");
    link.href = url;
    link.download = filename.replace(/[^a-zA-Z0-9._-]/g, "_").slice(0, 100) || "acorn-result.txt";
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
};
window.acornCopy = async (text) => {
    const status = document.getElementById("output-status");
    const report = (message) => {
        if (status) status.textContent = message;
        window.acornChrome?.notifications.notify(message);
    };
    try { await navigator.clipboard.writeText(text); report("Copied to clipboard."); }
    catch { report("Clipboard unavailable. Select and copy the code manually."); }
};
window.acornCelebrate = (() => {
    let celebrated = false;
    return () => {
        const heading = document.getElementById("result-heading");
        if (celebrated || !heading) return;
        celebrated = true;
        const burst = document.createElement("span");
        burst.className = "acorn-celebration";
        burst.setAttribute("aria-hidden", "true");
        const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
        for (const index of Array.from({ length: reduced ? 4 : 48 }, (_, index) => index)) {
            const angle = index * 2.4;
            const radius = 28 + index % 7 * 8;
            const piece = document.createElement("span");
            piece.className = "celebration-piece";
            piece.dataset.shape = ["circle", "square", "triangle", "strip"][index % 4];
            piece.style.setProperty("--x", `${20 + Math.cos(angle) * radius}px`);
            piece.style.setProperty("--y", `${Math.sin(angle) * radius * .75 - 18}px`);
            piece.style.setProperty("--rotation", `${index * 71}deg`);
            piece.style.setProperty("--delay", `${index % 6 * 20}ms`);
            piece.style.setProperty("--size", `${5 + index % 4}px`);
            piece.style.setProperty("--still-x", `${index % 4 * 12}px`);
            burst.append(piece);
        }
        heading.append(burst);
        setTimeout(() => burst.remove(), 1700);
    };
})();
async function installWebMcp() {
    const status = document.getElementById("webmcp-status");
    const moduleScript = document.querySelector('script[type="module"][src*="acorn-web"]');
    if (!status || !moduleScript) return;
    try {
        const [adapterModule, bindings] = await Promise.all([
            import(status.dataset.adapter), import(moduleScript.src),
        ]);
        const adapter = adapterModule.createAcornWebMcpAdapter(bindings);
        const installation = adapter.install();
        status.textContent = `WebMCP: ${await installation.ready}`;
        window.addEventListener("pagehide", (event) => {
            if (!event.persisted) installation.dispose();
        });
    } catch (error) {
        status.textContent = "WebMCP: failed (manual tools available)";
        console.error("WebMCP installation failed", error);
    }
}
installWebMcp();

function createTheme(preference, apply) {
    let systemDark = preference;
    let explicit = null;
    const sync = () => apply(explicit ?? systemDark, explicit === null);
    return {
        sync,
        choose(mode) { if (["dark", "light", "system"].includes(mode)) { explicit = mode === "system" ? null : mode === "dark"; sync(); } },
        reset() { explicit = null; sync(); },
        toggle() { explicit = !(explicit ?? systemDark); sync(); },
        update(dark) { systemDark = dark; sync(); },
    };
}
function createNotifier(show, schedule = setTimeout, cancel = clearTimeout) {
    let timer;
    const dismiss = () => { cancel(timer); timer = undefined; show(""); };
    return {
        dismiss,
        notify(message) { dismiss(); show(message); timer = schedule(dismiss, 6000); },
    };
}
function createTour(loadDriver, environment) {
    let active;
    let generation = 0;
    let silent = false;
    const cancel = () => {
        generation += 1;
        silent = true;
        active?.destroy();
        active = undefined;
    };
    return {
        cancel,
        async start() {
            cancel();
            const request = generation;
            const trigger = environment.find("#tour-start");
            const steps = [
                ["#sample-data", "Start with an example", "Try Sample Data to explore the tools without opening a file."],
                ["#skill-entry", "Use the ACORN Skill", "Read the release's workflows for your coding agent."],
                ["#local-entry", "Open your data", "Choose local UTF-8 files or a folder. Loading keeps tools under your control."],
            ].map(([element, title, description]) => ({ element, popover: { title, description } }));
            try {
                const { driver } = await loadDriver();
                if (request !== generation) return;
                if (!trigger?.isConnected || steps.some((step) => !environment.find(step.element))) {
                    environment.notify("The tour is unavailable on this page. Return Home and try again.");
                    return;
                }
                silent = false;
                active = driver({
                    steps,
                    animate: !environment.reducedMotion(),
                    smoothScroll: false,
                    allowClose: true,
                    allowKeyboardControl: true,
                    disableActiveInteraction: true,
                    showProgress: true,
                    progressText: "{{current}} of {{total}}",
                    nextBtnText: "Next",
                    prevBtnText: "Back",
                    doneBtnText: "Done",
                    closeBtnLabel: "Close tour",
                    popoverClass: `acorn-tour ${environment.theme()}`,
                    onDestroyed(_element, _step, { reason }) {
                        active = undefined;
                        if (!silent) {
                            if (trigger.isConnected) trigger.focus();
                            environment.notify(reason === "done" ? "Tour complete. Choose sample data or open a file." : "Tour closed. You can restart it from Home.");
                        }
                    },
                });
                active.drive();
            } catch {
                if (request === generation) environment.notify("The tour could not load. All research tools remain available.");
            }
        },
    };
}
function installChrome() {
    const workspace = document.querySelector(".workspace");
    if (!workspace || !window.matchMedia) return;
    const preference = window.matchMedia("(prefers-color-scheme: dark)");
    const theme = createTheme(preference.matches, (dark, system) => {
        workspace.dataset.theme = dark ? "dark" : "light";
        const button = document.getElementById("theme-toggle");
        button?.setAttribute("aria-pressed", String(dark));
        button?.setAttribute("aria-label", dark ? "Dark theme" : "Light theme");
        const label = document.getElementById("theme-label");
        if (label) label.textContent = dark ? "Dark" : "Light";
        for (const choice of ["dark", "light", "system"]) {
            document.querySelector(`[data-theme-choice="${choice}"]`)?.setAttribute("aria-pressed", String(choice === (system ? "system" : dark ? "dark" : "light")));
        }
        const mode = document.getElementById("theme-mode");
        if (mode) mode.textContent = system ? "Following your system preference." : "Using your explicit choice for this session.";
    });
    const notifications = createNotifier((message) => {
        const region = document.getElementById("notification");
        const text = document.getElementById("notification-text");
        if (region) region.hidden = !message;
        if (text) text.textContent = message;
    });
    const tour = createTour(() => import(document.getElementById("tour-start").dataset.driver), {
        find: (selector) => document.querySelector(selector),
        notify: notifications.notify,
        reducedMotion: () => window.matchMedia("(prefers-reduced-motion: reduce)").matches,
        theme: () => workspace.dataset.theme,
    });
    window.acornChrome = { theme, notifications, tour };
    preference.addEventListener("change", (event) => theme.update(event.matches));
    window.addEventListener("resize", () => {
        const toggle = document.getElementById("nav-toggle");
        if (toggle?.getAttribute("aria-expanded") === "true" && window.getComputedStyle(toggle.parentElement).display === "none") {
            // Close through Dioxus so drawer state, aria-expanded and main inertness agree.
            toggle.click();
        }
    });
    window.addEventListener("pagehide", () => { tour.cancel(); notifications.dismiss(); });
    theme.sync();
}
installChrome();

const workspaceLimits = { count: 500, depth: 20, file: 262144, total: 16777216, entries: 10000 };
function validWorkspacePath(path) {
    const parts = path.split("/");
    return !/[\\:\x00-\x1f\x7f]/.test(path) && parts.length <= workspaceLimits.depth
        && parts.every((part) => part && part !== "." && part !== "..") && /\.(json|ya?ml|md|txt)$/i.test(path);
}
async function readWorkspaceFiles(candidates, skipped = 0) {
    const files = [];
    const paths = new Set();
    let bytes = 0;
    for (const { path, file } of [...candidates].sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0)) {
        if (!validWorkspacePath(path) || paths.has(path) || files.length >= workspaceLimits.count || file.size > workspaceLimits.file || bytes + file.size > workspaceLimits.total) { skipped += 1; continue; }
        try {
            const buffer = await file.arrayBuffer();
            if (buffer.byteLength > workspaceLimits.file || bytes + buffer.byteLength > workspaceLimits.total) { skipped += 1; continue; }
            const content = new TextDecoder("utf-8", { fatal: true }).decode(buffer);
            files.push({ path, content }); paths.add(path); bytes += buffer.byteLength;
        } catch { skipped += 1; }
    }
    return { files, skipped };
}
async function readWorkspaceEntries(entries) {
    const candidates = [];
    let skipped = 0;
    let visited = 0;
    const visit = async (entry, prefix = "") => {
        visited += 1;
        const path = prefix + entry.name;
        if (visited > workspaceLimits.entries || path.split("/").length > workspaceLimits.depth) { skipped += 1; return; }
        try {
            if (entry.isFile) candidates.push({ path, file: await new Promise((resolve, reject) => entry.file(resolve, reject)) });
            else if (entry.isDirectory) {
                const reader = entry.createReader();
                let children;
                do {
                    children = await new Promise((resolve, reject) => reader.readEntries(resolve, reject));
                    for (const child of children) {
                        if (visited >= workspaceLimits.entries) { skipped += 1; break; }
                        await visit(child, path + "/");
                    }
                } while (children.length && visited < workspaceLimits.entries);
            } else skipped += 1;
        } catch { skipped += 1; }
    };
    for (const entry of entries) {
        if (visited >= workspaceLimits.entries) { skipped += 1; break; }
        await visit(entry);
    }
    return readWorkspaceFiles(candidates, skipped);
}
function createWorkspaceInput() {
    let subscriber;
    let generation = 0;
    let picker;
    const cancel = () => { generation += 1; picker?.remove(); picker = undefined; window.acornRemote?.cancel(); };
    const deliver = async (work) => {
        window.acornRemote?.cancel();
        const request = ++generation;
        try {
            const result = await work;
            if (request === generation) subscriber?.(result);
        } catch { if (request === generation) subscriber?.({ files: [], skipped: 1 }); }
    };
    return {
        cancel,
        connect(callback) { cancel(); subscriber = callback; },
        disconnect() { cancel(); subscriber = undefined; },
        pick(mode) {
            cancel();
            const input = document.createElement("input");
            input.type = "file"; input.multiple = true; input.accept = ".json,.yaml,.yml,.md,.txt";
            if (mode === "folder") {
                if (!("webkitdirectory" in input)) { window.acornChrome?.notifications.notify("Folder selection is unavailable in this browser. Choose files instead."); return; }
                input.webkitdirectory = true;
            }
            input.hidden = true; document.body.append(input); picker = input;
            input.addEventListener("cancel", () => { input.remove(); if (picker === input) picker = undefined; }, { once: true });
            input.addEventListener("change", () => {
                if (picker !== input) return;
                const all = Array.from(input.files || []);
                const candidates = all.slice(0, workspaceLimits.entries).map((file) => ({ path: file.webkitRelativePath || file.name, file }));
                input.remove(); if (picker === input) picker = undefined;
                void deliver(readWorkspaceFiles(candidates, Math.max(0, all.length - workspaceLimits.entries)));
            }, { once: true });
            input.click();
        },
        drop(event) {
            // Capture entry references synchronously while the drop's data store is readable.
            const entries = Array.from(event.dataTransfer?.items || []).map((item) => item.webkitGetAsEntry?.()).filter(Boolean);
            const files = Array.from(event.dataTransfer?.files || []);
            void deliver(entries.length ? readWorkspaceEntries(entries) : readWorkspaceFiles(files.slice(0, workspaceLimits.entries).map((file) => ({ path: file.name, file })), Math.max(0, files.length - workspaceLimits.entries)));
        },
    };
}
window.acornWorkspace = { ...createWorkspaceInput(), validPath: validWorkspacePath };
if (document.addEventListener) {
    document.addEventListener("click", (event) => {
        document.querySelectorAll(".workflow-menu[open]").forEach((menu) => { if (!menu.contains(event.target)) menu.removeAttribute("open"); });
    });
    document.addEventListener("keydown", (event) => {
        if (event.key === "Escape") document.querySelectorAll(".workflow-menu[open]").forEach((menu) => { menu.removeAttribute("open"); menu.querySelector("summary").focus(); });
    });
    document.addEventListener("dragover", (event) => { if (event.target.closest?.("[data-workspace-drop]")) event.preventDefault(); });
    document.addEventListener("drop", (event) => {
        if (event.target.closest?.("[data-workspace-drop]")) { event.preventDefault(); window.acornWorkspace.drop(event); }
    });
}

async function verifySkill(manifest, definition, archive, version, hash) {
    if (manifest.schemaVersion !== 1 || manifest.appVersion !== version || manifest.compatibility?.cli !== version
        || manifest.definition?.file !== "SKILL.md" || manifest.archive?.file !== "acorn-skill.zip"
        || !/^[a-f0-9]{64}$/.test(manifest.skillRevision || "")
        || manifest.files?.["SKILL.md"] !== manifest.definition?.sha256) throw new Error("The bundled skill does not match this app. Reload the matching app release before exporting.");
    if (await hash(definition) !== manifest.definition.sha256 || await hash(archive) !== manifest.archive.sha256) throw new Error("The bundled skill failed its integrity check. Reload or reinstall the matching app release before exporting.");
    return { manifest, definition: new TextDecoder("utf-8", { fatal: true }).decode(definition) };
}
const skillBundles = new Map();
window.acornSkill = {
    async load(base, version) {
        const url = new URL(base + "/", document.baseURI);
        if (url.origin !== location.origin) throw new Error("The skill must be bundled with this app.");
        const key = `${url.href}:${version}`;
        if (!skillBundles.has(key)) {
            const fetchBytes = async (name) => {
                const response = await fetch(new URL(name, url), { credentials: "omit", redirect: "error" });
                if (!response.ok) throw new Error("The bundled skill is missing. Rebuild or reload this app release.");
                const bytes = await response.arrayBuffer();
                if (bytes.byteLength > 8 * 1024 * 1024) throw new Error("The bundled skill exceeds its size limit.");
                return bytes;
            };
            const promise = Promise.all([fetchBytes("manifest.json"), fetchBytes("SKILL.md"), fetchBytes("acorn-skill.zip")]).then(async ([metadata, definition, archive]) => {
                const hash = async (bytes) => Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))).map((byte) => byte.toString(16).padStart(2, "0")).join("");
                const result = await verifySkill(JSON.parse(new TextDecoder().decode(metadata)), definition, archive, version, hash);
                return { ...result, archive };
            });
            skillBundles.set(key, promise);
            promise.catch(() => skillBundles.delete(key));
        }
        const { manifest, definition } = await skillBundles.get(key);
        return { manifest, definition };
    },
    async download(base, version) {
        try {
            await this.load(base, version);
            const url = new URL(base + "/", document.baseURI);
            const { archive } = await skillBundles.get(`${url.href}:${version}`);
            const objectUrl = URL.createObjectURL(new Blob([archive], { type: "application/zip" }));
            const link = document.createElement("a"); link.href = objectUrl; link.download = `acorn-skill-${version}.zip`; link.click();
            setTimeout(() => URL.revokeObjectURL(objectUrl), 1000);
            window.acornChrome?.notifications.notify("Skill archive downloaded. Review it before adding it to your agent environment.");
        } catch (error) { window.acornChrome?.notifications.notify(error.message); }
    },
};

function isLocalMapUrl(url, page) {
    const loopback = (hostname) => ["127.0.0.1", "localhost"].includes(hostname);
    return url.origin === page.origin || (url.protocol === "http:" && page.protocol === "http:" && loopback(url.hostname) && loopback(page.hostname));
}
window.acornMap = (() => {
    let bridge;
    let observer;
    let generation = 0;
    let state = { markers: [], selected: null, theme: "dark" };
    const theme = () => document.querySelector(".workspace")?.dataset.theme || "dark";
    return {
        update(next) { state = { ...next, theme: theme() }; bridge?.update(state); },
        async connect(moduleUrl, cssUrl, notify) {
            const request = ++generation;
            try {
                const host = document.getElementById("geospatial-map");
                const url = new URL(moduleUrl, document.baseURI);
                if (!isLocalMapUrl(url, location)) throw new Error("Map assets must be local to this app.");
                if (cssUrl && !document.querySelector("link[data-map-style]")) {
                    const link = document.createElement("link"); link.rel = "stylesheet"; link.href = cssUrl; link.dataset.mapStyle = "true"; document.head.append(link);
                }
                const module = await import(url.href);
                if (request !== generation || !host?.isConnected) return;
                bridge = module.mount(host, { ...state, theme: theme() }, notify);
                observer = new MutationObserver(() => bridge?.update({ ...state, theme: theme() }));
                const workspace = document.querySelector(".workspace");
                if (workspace) observer.observe(workspace, { attributes: true, attributeFilter: ["data-theme"] });
            } catch (error) {
                console.error("Local map initialization failed", error);
                if (request === generation) notify({ type: "error", message: "Map unavailable. Organization data remains available in the table." });
            }
        },
        disconnect() { generation += 1; observer?.disconnect(); bridge?.dispose(); bridge = undefined; observer = undefined; state = { markers: [], selected: null, theme: "dark" }; },
    };
})();

if (window.dispatchEvent) window.dispatchEvent(new Event("acorn-effects-ready"));
